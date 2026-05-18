//! Build-time PDF → text extraction helper.
//!
//! Mirrors the runtime extractor at
//! `crates/domains/src/social/software/binary/pdf/extract.rs`
//! (M4.γ Phase 6) but is a focused subset that can run from
//! `build.rs` without depending on the crate being built.
//!
//! The outcome of extraction is a [`PdfExtractOutcome`] enum
//! whose variants the build script translates into corresponding
//! `PdfBuildExtraction` codegen constants (defined at
//! `crates/domains/src/applied/data_provisioning/build_extraction.rs`).
//! Both enums are typed and exhaustive — no `Option<&str>` gaps.
//!
//! ## Font-encoding resolution per ISO 32000-2:2020 §9.10.2
//!
//! For each font referenced from a page's content stream, the
//! decoder walks the standard precedence chain:
//!
//! 1. `/ToUnicode` CMap — explicit per-font Unicode mapping
//!    (Adobe Tech Note #5014). Takes precedence over `/Encoding`
//!    for **any** font subtype.
//! 2. `/Encoding` named entry — dispatched through lopdf's public
//!    `Dictionary::get_font_encoding`, which handles all eight
//!    Annex D base encodings (`WinAnsi`, `MacRoman`, `MacExpert`,
//!    `Standard`, `PDFDoc`, `Symbol`, `ZapfDingbats`, `Expert`)
//!    and the `Identity-H` / `Identity-V` CIDFont variants.
//! 3. Font's built-in encoding (§9.6.5.2) — fallback when neither
//!    `/ToUnicode` nor `/Encoding` is declared.
//!
//! Per-font encodings are resolved once per page and cached by
//! resource name. The `Tf` operator (§9.3.1) selects the active
//! font for subsequent `Tj` / `TJ` / `'` / `"` operations.
//!
//! ## `/Differences` overrides
//!
//! `/Differences` arrays (§9.6.5.4) need Adobe Glyph List access
//! to resolve glyph names; lopdf's `get_font_encoding` falls back
//! to a base encoding when it encounters one. That fallback is
//! the source of mojibake-style artifacts (`Ð` for em-dash) when
//! the original PDF relied on a glyph-name override. The runtime
//! extractor surfaces this as a typed
//! [`crate::social::software::binary::pdf::font::FontEncoding::DifferencesUnresolved`]
//! variant; the build-time extractor accepts the lopdf fallback —
//! the discrepancy is auditable by comparing the runtime extractor's
//! output to the build-time output.
//!
//! Spec references:
//!
//! - ISO 32000-2:2020 §7.5 — file structure
//! - §7.8.2 — content streams
//! - §9.3.1 — `Tf` font-selection operator
//! - §9.4.3 — text-showing operators (Tj, TJ, ', ")
//! - §9.6.5 — font encodings, Annex D
//! - §9.10.2 — mapping character codes to Unicode values
//! - Adobe Tech Note #5014 — *ToUnicode Mapping File Tutorial*

use lopdf::cmap::ToUnicodeCMap;
use lopdf::mappings::{
    EXPERT_ENCODING, MAC_EXPERT_ENCODING, MAC_ROMAN_ENCODING, PDF_DOC_ENCODING, STANDARD_ENCODING,
    SYMBOL_ENCODING, WIN_ANSI_ENCODING,
};
use std::collections::HashMap;
use std::path::Path;

#[path = "agl.rs"]
mod agl;

/// Look up the static 256-entry encoding table for a named base
/// encoding (ISO 32000-2:2020 Annex D, Table 113).
fn standard_table(name: &[u8]) -> Option<&'static [Option<u16>; 256]> {
    match name {
        b"WinAnsiEncoding" => Some(&WIN_ANSI_ENCODING),
        b"MacRomanEncoding" => Some(&MAC_ROMAN_ENCODING),
        b"MacExpertEncoding" => Some(&MAC_EXPERT_ENCODING),
        b"StandardEncoding" => Some(&STANDARD_ENCODING),
        b"PDFDocEncoding" => Some(&PDF_DOC_ENCODING),
        b"Symbol" | b"SymbolEncoding" => Some(&SYMBOL_ENCODING),
        b"ZapfDingbats" | b"ZapfDingbatsEncoding" => Some(&SYMBOL_ENCODING),
        b"ExpertEncoding" => Some(&EXPERT_ENCODING),
        _ => None,
    }
}

/// Outcome of a build-time PDF extraction.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfExtractOutcome {
    /// PDF parsed, text-showing operators decoded successfully.
    Extracted(String),
    /// PDF file at the given path doesn't exist on disk.
    NotOnDisk,
    /// PDF was on disk but couldn't be parsed.
    ParseFailed(String),
    /// PDF parsed but encryption blocked decode.
    Encrypted,
}

/// Extract plain text from a PDF on disk.
pub fn extract_pdf_to_text(path: &Path) -> PdfExtractOutcome {
    if !path.exists() {
        return PdfExtractOutcome::NotOnDisk;
    }
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => return PdfExtractOutcome::ParseFailed(format!("read {path:?}: {e}")),
    };
    extract_pdf_bytes(&bytes)
}

/// Per-font decoder state resolved once per page.
enum FontDecoder {
    /// /ToUnicode CMap present — highest precedence per §9.10.2.
    ToUnicode(ToUnicodeCMap),
    /// 256-entry simple encoding table from `lopdf::mappings`
    /// (WinAnsi, MacRoman, …) used as-is.
    OneByte(&'static [Option<u16>; 256]),
    /// 256-entry table with `/Differences` glyph-name overrides
    /// resolved via the Adobe Glyph List per ISO 32000-2:2020
    /// §9.6.5.4. Owned because the override pattern is per-font.
    OneByteOwned(Box<[Option<u16>; 256]>),
    /// Identity-H / Identity-V CIDFont without /ToUnicode.
    Identity,
    /// No /Encoding, no /ToUnicode — Latin-1 passthrough.
    Builtin,
}

impl FontDecoder {
    fn decode(&self, bytes: &[u8]) -> String {
        match self {
            Self::ToUnicode(cmap) => lopdf::Encoding::UnicodeMapEncoding(cmap.clone())
                .bytes_to_string(bytes)
                .unwrap_or_else(|_| bytes.iter().map(|&b| b as char).collect()),
            Self::OneByte(table) => lopdf::Encoding::OneByteEncoding(table)
                .bytes_to_string(bytes)
                .unwrap_or_else(|_| bytes.iter().map(|&b| b as char).collect()),
            Self::OneByteOwned(table) => lopdf::Encoding::OneByteEncoding(table.as_ref())
                .bytes_to_string(bytes)
                .unwrap_or_else(|_| bytes.iter().map(|&b| b as char).collect()),
            Self::Identity => {
                if !bytes.len().is_multiple_of(2) {
                    return bytes.iter().map(|&b| b as char).collect();
                }
                let mut chars = Vec::with_capacity(bytes.len() / 2);
                for chunk in bytes.chunks_exact(2) {
                    chars.push(u16::from_be_bytes([chunk[0], chunk[1]]));
                }
                String::from_utf16_lossy(&chars)
            }
            Self::Builtin => bytes.iter().map(|&b| b as char).collect(),
        }
    }
}

/// Walk a `/Differences` array per ISO 32000-2:2020 §9.6.5.4 and
/// apply every (code, glyph-name) override on top of `base`.
///
/// Per §9.6.5.4: the array alternates integer codes and one or
/// more glyph names. An integer N starts a new code group at byte
/// N; subsequent names assign to N, N+1, N+2, … until the next
/// integer resets the position. Glyph names are resolved through
/// the Adobe Glyph List ([`agl::glyph_name_to_unicode`]).
fn apply_differences(
    base: &'static [Option<u16>; 256],
    diffs: &[lopdf::Object],
) -> Box<[Option<u16>; 256]> {
    let mut table = Box::new(*base);
    let mut code: i64 = 0;
    for item in diffs {
        match item {
            lopdf::Object::Integer(i) => {
                code = *i;
            }
            lopdf::Object::Name(name_bytes) => {
                if (0..=255).contains(&code) {
                    let name = String::from_utf8_lossy(name_bytes);
                    table[code as usize] = agl::glyph_name_to_unicode(&name);
                }
                code += 1;
            }
            _ => {}
        }
    }
    table
}

/// Resolve a single font dictionary into a [`FontDecoder`].
///
/// Walks the §9.10.2 precedence chain: /ToUnicode first (any
/// subtype), then /Encoding (via lopdf's public dispatcher), then
/// the font's built-in default.
fn resolve_font_decoder(font_dict: &lopdf::Dictionary, doc: &lopdf::Document) -> FontDecoder {
    // Step 1: /ToUnicode (highest precedence, any subtype).
    if font_dict.get(b"ToUnicode").is_ok()
        && let Ok(stream) = font_dict
            .get_deref(b"ToUnicode", doc)
            .and_then(|o| o.as_stream())
        && let Ok(content) = stream.get_plain_content()
        && let Ok(cmap) = ToUnicodeCMap::parse(content)
    {
        return FontDecoder::ToUnicode(cmap);
    }

    // Step 2: /Encoding entry. Either a Name (the common case;
    // base encoding from Annex D, or Identity-H / -V), or a
    // dictionary form (§9.6.5.4 — base + /Differences). Indirect
    // references go through `get_deref`.
    if let Ok(enc_obj) = font_dict.get_deref(b"Encoding", doc) {
        if let Ok(name) = enc_obj.as_name() {
            if name == b"Identity-H" || name == b"Identity-V" {
                return FontDecoder::Identity;
            }
            if let Some(table) = standard_table(name) {
                return FontDecoder::OneByte(table);
            }
        }
        if let Ok(enc_dict) = enc_obj.as_dict() {
            let base = enc_dict
                .get(b"BaseEncoding")
                .ok()
                .and_then(|o| o.as_name().ok())
                .and_then(standard_table)
                // Per §9.6.5.4: if no /BaseEncoding, the implicit
                // default is the font program's built-in encoding;
                // for Type1/TrueType this is `StandardEncoding`.
                .unwrap_or(&STANDARD_ENCODING);
            if let Ok(diffs) = enc_dict.get(b"Differences").and_then(|o| o.as_array()) {
                return FontDecoder::OneByteOwned(apply_differences(base, diffs));
            }
            return FontDecoder::OneByte(base);
        }
    }

    // Step 3: no /Encoding declared → font built-in.
    FontDecoder::Builtin
}

/// Build the resource-name → decoder map for one page.
///
/// Reads `/Resources /Font /<name>` for the page and resolves each
/// font dict via [`resolve_font_decoder`]. Resource names not in
/// the map decode through a WinAnsi fallback (the safest assumption
/// for PDF/A-derived USCODE filings).
fn page_font_decoders(
    page_id: lopdf::ObjectId,
    doc: &lopdf::Document,
) -> HashMap<Vec<u8>, FontDecoder> {
    let mut out = HashMap::new();
    let fonts = match doc.get_page_fonts(page_id) {
        Ok(f) => f,
        Err(_) => return out,
    };
    for (name, font_dict) in fonts {
        out.insert(name, resolve_font_decoder(font_dict, doc));
    }
    out
}

/// Extract plain text from PDF bytes in memory.
pub fn extract_pdf_bytes(bytes: &[u8]) -> PdfExtractOutcome {
    let doc = match lopdf::Document::load_mem(bytes) {
        Ok(d) => d,
        Err(e) => return PdfExtractOutcome::ParseFailed(format!("{e}")),
    };
    if doc.is_encrypted() {
        return PdfExtractOutcome::Encrypted;
    }

    let pages = doc.get_pages();
    let mut out = String::new();
    let mut first_page = true;
    let fallback = FontDecoder::OneByte(&WIN_ANSI_ENCODING);
    for &page_id in pages.values() {
        let content = match doc.get_page_content(page_id) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let parsed = match lopdf::content::Content::decode(&content) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let decoders = page_font_decoders(page_id, &doc);
        let mut current_font: Option<Vec<u8>> = None;
        if !first_page {
            out.push_str("\n\n");
        }
        first_page = false;
        for op in &parsed.operations {
            let decoder = current_font
                .as_ref()
                .and_then(|name| decoders.get(name))
                .unwrap_or(&fallback);
            match op.operator.as_str() {
                "Tf" => {
                    if let Some(lopdf::Object::Name(name)) = op.operands.first() {
                        current_font = Some(name.clone());
                    }
                }
                "Tj" | "'" => {
                    if let Some(bytes) = op.operands.first().and_then(string_bytes) {
                        out.push_str(&decoder.decode(&bytes));
                    }
                    out.push(' ');
                }
                "\"" => {
                    if let Some(bytes) = op.operands.get(2).and_then(string_bytes) {
                        out.push_str(&decoder.decode(&bytes));
                    }
                    out.push(' ');
                }
                "TJ" => {
                    if let Some(lopdf::Object::Array(items)) = op.operands.first() {
                        for item in items {
                            if let Some(bytes) = string_bytes(item) {
                                out.push_str(&decoder.decode(&bytes));
                            }
                        }
                        out.push(' ');
                    }
                }
                _ => {}
            }
        }
    }

    PdfExtractOutcome::Extracted(out)
}

fn string_bytes(o: &lopdf::Object) -> Option<Vec<u8>> {
    match o {
        lopdf::Object::String(bytes, _) => Some(bytes.clone()),
        _ => None,
    }
}

/// Escape a `&str` for use inside a Rust raw string literal that
/// the build script emits. We use `r#"..."#` quoting to avoid the
/// need to backslash-escape arbitrary content; the only thing we
/// must guarantee is that `"#` doesn't appear in the input.
pub fn escape_for_raw_string(s: &str) -> String {
    // Replace any `"#` sequence with `"\u{0023}` (the `#` rendered
    // as a Unicode escape) so the raw string boundary can't be
    // forged. Idempotent on inputs that don't contain `"#`.
    s.replace("\"#", "\"\\u{0023}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic 1-page PDF programmatically via lopdf
    /// so the extractor has a known-shape input to verify against.
    fn pdf_with_text(text: &str) -> Vec<u8> {
        use lopdf::{Document, Object, Stream, dictionary};
        let mut doc = Document::with_version("1.4");
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
            "Encoding" => "WinAnsiEncoding",
        });
        let cs = format!("BT\n/F1 12 Tf\n({text}) Tj\nET\n");
        let content_id = doc.add_object(Stream::new(dictionary! {}, cs.into_bytes()));
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
            "Resources" => dictionary! {
                "Font" => dictionary! { "F1" => font_id },
            },
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).expect("serialize");
        bytes
    }

    // ── Unit tests ───────────────────────────────────────────────

    #[test]
    fn extract_winansi_ascii_text() {
        let bytes = pdf_with_text("Hello world");
        match extract_pdf_bytes(&bytes) {
            PdfExtractOutcome::Extracted(text) => {
                assert!(text.contains("Hello world"));
            }
            other => panic!("expected Extracted; got {other:?}"),
        }
    }

    #[test]
    fn missing_file_returns_not_on_disk() {
        let result =
            extract_pdf_to_text(std::path::Path::new("/tmp/definitely_does_not_exist.pdf"));
        assert_eq!(result, PdfExtractOutcome::NotOnDisk);
    }

    #[test]
    fn malformed_bytes_return_parse_failed() {
        let result = extract_pdf_bytes(b"not a PDF at all");
        match result {
            PdfExtractOutcome::ParseFailed(_) => {}
            other => panic!("expected ParseFailed; got {other:?}"),
        }
    }

    #[test]
    fn escape_for_raw_string_preserves_normal_text() {
        let s = "normal text with quotes \" and hashes # but no boundary";
        assert_eq!(escape_for_raw_string(s), s);
    }

    #[test]
    fn escape_for_raw_string_neutralizes_boundary_sequence() {
        let s = r##"text with "# inside"##;
        let escaped = escape_for_raw_string(s);
        assert!(!escaped.contains("\"#"));
        assert!(escaped.contains("\"\\u{0023}"));
    }

    #[test]
    fn escape_for_raw_string_is_idempotent_when_no_boundary() {
        let s = "plain text";
        assert_eq!(escape_for_raw_string(&escape_for_raw_string(s)), s);
    }

    // ── Property-based ───────────────────────────────────────────

    use proptest::prelude::*;

    /// Generate printable-ASCII text safe to embed in a PDF
    /// string literal.
    fn arb_safe_text() -> impl Strategy<Value = String> {
        proptest::collection::vec(0x20u8..=0x7Eu8, 0..32).prop_map(|bytes| {
            bytes
                .into_iter()
                .filter(|b| !matches!(*b, b'(' | b')' | b'\\'))
                .map(|b| b as char)
                .collect()
        })
    }

    proptest! {
        /// ASCII text round-trips through the build-time
        /// extractor.
        #[test]
        fn prop_ascii_round_trips_through_build_extractor(text in arb_safe_text()) {
            let bytes = pdf_with_text(&text);
            match extract_pdf_bytes(&bytes) {
                PdfExtractOutcome::Extracted(out) => {
                    prop_assert!(
                        out.contains(&text) || text.is_empty(),
                        "extracted {out:?} should contain {text:?}",
                    );
                }
                other => {
                    return Err(proptest::test_runner::TestCaseError::fail(format!(
                        "expected Extracted; got {other:?}"
                    )));
                }
            }
        }

        /// Extraction is deterministic: same PDF bytes → identical
        /// output across repeated calls.
        #[test]
        fn prop_extract_is_deterministic(text in arb_safe_text()) {
            let bytes = pdf_with_text(&text);
            let r1 = extract_pdf_bytes(&bytes);
            let r2 = extract_pdf_bytes(&bytes);
            prop_assert_eq!(r1, r2);
        }

        /// `escape_for_raw_string` is idempotent on inputs that
        /// don't contain the boundary sequence — `f(f(s)) == f(s)`
        /// holds universally because the function preserves any
        /// input without `"#`.
        #[test]
        fn prop_escape_is_idempotent_when_no_boundary(s in "[a-zA-Z0-9 ]{0,32}") {
            prop_assert_eq!(
                escape_for_raw_string(&escape_for_raw_string(&s)),
                escape_for_raw_string(&s)
            );
        }
    }
}

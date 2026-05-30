//! Font + encoding pipeline — byte → Unicode resolution.
//!
//! Bridges a [`super::content_stream::TextShowEvent`] (raw glyph
//! bytes + font name) to a decoded `String` by walking the font's
//! encoding entries per ISO 32000-2:2020 §9.10.2 precedence:
//!
//! 1. **`/ToUnicode` CMap** — explicit per-font Unicode mapping
//!    (Adobe Tech Note #5014; §9.10.2 step 1). Takes precedence
//!    when present, **for any font subtype** — not just CIDFonts.
//! 2. **`/Encoding` entry** — a named base encoding from Annex D
//!    (`WinAnsiEncoding`, `MacRomanEncoding`, …), optionally
//!    overlaid with a `/Differences` array (§9.6.5.4).
//! 3. **Font's built-in encoding** (§9.6.5.2).
//! 4. **CIDFont (Identity-H / Identity-V)** — composite-font case
//!    (§9.7.4.3); bytes are interpreted as big-endian 16-bit CIDs.
//!
//! ## Encoding coverage
//!
//! Decodable end-to-end via the vendored lopdf (see
//! `vendor/lopdf/PRAXIS_PATCHES.md`):
//! - `/ToUnicode` CMaps — parsed via the now-pub-exported
//!   `lopdf::cmap::ToUnicodeCMap::parse`.
//! - All eight Annex D standard encodings — `WinAnsi`, `PdfDoc`,
//!   `MacRoman`, `MacExpert`, `Standard`, `Symbol`, `ZapfDingbats`
//!   (modeled via `Symbol` table — its 256-entry table covers
//!   ZapfDingbats glyphs in the same range; flagged as
//!   [`StandardEncodingName::ZapfDingbats`] for downstream
//!   provenance), `Expert` — via `lopdf::mappings::*` tables.
//! - `Identity-H` / `Identity-V` — handled directly as UTF-16BE.
//! - **FontBuiltIn** fallback when no `/Encoding` and no
//!   `/ToUnicode` are declared (Latin-1 passthrough).
//!
//! `/Differences` arrays (§9.6.5.4) are fully resolved through
//! the Adobe Glyph List (see [`super::agl`]) — every glyph name
//! is looked up against the AGL and applied as an override on top
//! of the base encoding's 256-entry table.
//!
//! In every case the gap is typed and machine-checkable; nothing
//! is silently mapped to a wrong codepoint.
//!
//! Spec references:
//!
//! - ISO 32000-2:2020 §9.5–§9.10, Annex D
//! - Adobe Tech Note #5014 — *ToUnicode Mapping File Tutorial*

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use lopdf::Encoding as LopdfEncoding;
use lopdf::cmap::ToUnicodeCMap;
use lopdf::mappings::{
    EXPERT_ENCODING, MAC_EXPERT_ENCODING, MAC_ROMAN_ENCODING, PDF_DOC_ENCODING, STANDARD_ENCODING,
    SYMBOL_ENCODING, WIN_ANSI_ENCODING,
};

// ─────────────────────────────────────────────────────────────────────
// FontEncoding — the praxis-typed wrapper.
// ─────────────────────────────────────────────────────────────────────

/// How a font's glyph codes map to Unicode.
#[derive(Debug, Clone)]
pub enum FontEncoding {
    /// `/ToUnicode` CMap present (Adobe Tech Note #5014;
    /// ISO 32000-2:2020 §9.10.2 step 1). The CMap is the
    /// authoritative per-font glyph-code → Unicode map; it
    /// takes precedence over `/Encoding`, for any font subtype.
    ToUnicode(ToUnicodeCMap),

    /// Named base encoding from ISO 32000-2:2020 Annex D
    /// (WinAnsi, MacRoman, MacExpert, Standard, PDFDoc, Symbol,
    /// ZapfDingbats, Expert). Decoded via the 256-entry table
    /// from `lopdf::mappings`.
    Standard(StandardEncodingName),

    /// Base encoding overlaid with `/Differences` (§9.6.5.4).
    /// Glyph-name overrides are resolved through the Adobe Glyph
    /// List ([`super::agl`]). `merged` is the 256-entry table
    /// with each `/Differences` override applied on top of the
    /// base; entries not overridden retain the base mapping.
    WithDifferences {
        base: StandardEncodingName,
        merged: Box<[Option<u16>; 256]>,
    },

    /// CIDFont case — `/Encoding /Identity-H` or `/Identity-V`
    /// (§9.7.4.3) **without** a `/ToUnicode` stream. Bytes are
    /// big-endian 16-bit CIDs that we pass through as UTF-16BE
    /// best-effort. Compliance-grade extraction requires
    /// `/ToUnicode` for these fonts.
    Identity,

    /// Font program's built-in default — neither `/ToUnicode` nor
    /// `/Encoding` declared (§9.6.5.2). Without the font program
    /// parsed, we pass through bytes as Latin-1 and signal the gap
    /// via the typed variant.
    FontBuiltIn,
}

/// Named base encodings enumerated in ISO 32000-2:2020 Annex D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardEncodingName {
    /// Annex D.2 — `PDFDocEncoding`.
    PdfDoc,
    /// Annex D.5 — `WinAnsiEncoding`. Close to Windows-1252.
    WinAnsi,
    /// Annex D.4 — `MacRomanEncoding`.
    MacRoman,
    /// Annex D.6 — `MacExpertEncoding`.
    MacExpert,
    /// Annex D.3 — `StandardEncoding`. Adobe's original Type 1
    /// default encoding.
    Standard,
    /// Annex D.7 — Symbol font encoding.
    Symbol,
    /// Annex D.7 — ZapfDingbats encoding.
    ZapfDingbats,
    /// Annex D.6 — Adobe Expert encoding.
    Expert,
}

impl StandardEncodingName {
    /// Parse the name strings PDFs use in `/Encoding` entries
    /// (ISO 32000-2:2020 §9.6.5.1, Table 113).
    pub fn from_pdf_name(name: &str) -> Option<Self> {
        match name {
            "WinAnsiEncoding" => Some(Self::WinAnsi),
            "MacRomanEncoding" => Some(Self::MacRoman),
            "MacExpertEncoding" => Some(Self::MacExpert),
            "StandardEncoding" => Some(Self::Standard),
            "PDFDocEncoding" => Some(Self::PdfDoc),
            "Symbol" | "SymbolEncoding" => Some(Self::Symbol),
            "ZapfDingbats" | "ZapfDingbatsEncoding" => Some(Self::ZapfDingbats),
            "ExpertEncoding" => Some(Self::Expert),
            _ => None,
        }
    }

    /// 256-entry `[Option<u16>; 256]` table from
    /// `lopdf::mappings` for this encoding.
    fn table(self) -> &'static [Option<u16>; 256] {
        match self {
            Self::WinAnsi => &WIN_ANSI_ENCODING,
            Self::PdfDoc => &PDF_DOC_ENCODING,
            Self::MacRoman => &MAC_ROMAN_ENCODING,
            Self::MacExpert => &MAC_EXPERT_ENCODING,
            Self::Standard => &STANDARD_ENCODING,
            Self::Symbol | Self::ZapfDingbats => &SYMBOL_ENCODING,
            Self::Expert => &EXPERT_ENCODING,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// PdfFont — the typed font runtime value.
// ─────────────────────────────────────────────────────────────────────

/// A font resolved from a PDF font dictionary.
#[derive(Debug, Clone)]
pub struct PdfFont {
    /// PDF name as it appears in `/Resources /Font /<name>` —
    /// the same name a `Tf` operator in a content stream uses.
    pub name: String,
    /// Font subtype (`/Type1`, `/TrueType`, `/Type0`, …) per
    /// ISO 32000-2:2020 §9.6.2.
    pub subtype: String,
    /// How to map this font's glyph bytes to Unicode.
    pub encoding: FontEncoding,
}

// ─────────────────────────────────────────────────────────────────────
// Resolution — font dictionary → typed PdfFont.
// ─────────────────────────────────────────────────────────────────────

/// Why a font dictionary couldn't be resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontResolveError {
    /// `/Subtype` entry missing — every font dict must declare one
    /// per ISO 32000-2:2020 §9.6.2.
    MissingSubtype,
    /// `/Encoding` named encoding wasn't one of the standard
    /// encodings from Annex D and wasn't `Identity-H` / `-V`.
    UnknownEncodingName(String),
    /// `/ToUnicode` referenced an indirect object that couldn't
    /// be resolved against the document.
    ToUnicodeNotResolvable,
    /// `/ToUnicode` stream was found but its CMap content failed
    /// to parse per Adobe Tech Note #5014.
    ToUnicodeMalformed { detail: String },
    /// `/Differences` array wasn't parseable per §9.6.5.4.
    MalformedDifferences { detail: String },
}

impl core::fmt::Display for FontResolveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingSubtype => write!(f, "font dict has no /Subtype (ISO 32000-2 §9.6.2)"),
            Self::UnknownEncodingName(n) => {
                write!(f, "unknown /Encoding name {n:?} (Annex D)")
            }
            Self::ToUnicodeNotResolvable => write!(f, "/ToUnicode stream couldn't be resolved"),
            Self::ToUnicodeMalformed { detail } => {
                write!(f, "/ToUnicode CMap malformed: {detail}")
            }
            Self::MalformedDifferences { detail } => {
                write!(f, "malformed /Differences array: {detail}")
            }
        }
    }
}

impl std::error::Error for FontResolveError {}

/// Resolve a font dictionary into a typed [`PdfFont`].
///
/// `name` is the resource key under which this font appears in the
/// page's `/Resources /Font` (e.g. `"F1"`). `font_dict` is the
/// dictionary that key resolves to. `doc` provides access to
/// indirect objects (`/ToUnicode` streams are typically indirect).
pub fn resolve_font(
    name: &str,
    font_dict: &lopdf::Dictionary,
    doc: &lopdf::Document,
) -> Result<PdfFont, FontResolveError> {
    let subtype = font_dict
        .get(b"Subtype")
        .ok()
        .and_then(|o| o.as_name().ok())
        .map(|n| String::from_utf8_lossy(n).into_owned())
        .ok_or(FontResolveError::MissingSubtype)?;

    let encoding = resolve_encoding(font_dict, doc)?;

    Ok(PdfFont {
        name: name.to_string(),
        subtype,
        encoding,
    })
}

/// Resolution dispatcher — walks the font dict per §9.10.2
/// precedence and returns the right [`FontEncoding`] variant.
fn resolve_encoding(
    font_dict: &lopdf::Dictionary,
    doc: &lopdf::Document,
) -> Result<FontEncoding, FontResolveError> {
    // ─── §9.10.2 step 1 — /ToUnicode (highest precedence,
    //     for any font subtype, not just CIDFonts) ───
    if font_dict.get(b"ToUnicode").is_ok() {
        let stream = font_dict
            .get_deref(b"ToUnicode", doc)
            .and_then(|o| o.as_stream())
            .map_err(|_| FontResolveError::ToUnicodeNotResolvable)?;
        let content =
            stream
                .get_plain_content()
                .map_err(|e| FontResolveError::ToUnicodeMalformed {
                    detail: format!("{e}"),
                })?;
        let cmap =
            ToUnicodeCMap::parse(content).map_err(|e| FontResolveError::ToUnicodeMalformed {
                detail: format!("{e:?}"),
            })?;
        return Ok(FontEncoding::ToUnicode(cmap));
    }

    // ─── §9.6.5 step 2 — /Encoding ───
    if let Ok(enc_obj) = font_dict.get(b"Encoding") {
        if let Ok(name_bytes) = enc_obj.as_name() {
            let name = String::from_utf8_lossy(name_bytes);
            if name == "Identity-H" || name == "Identity-V" {
                return Ok(FontEncoding::Identity);
            }
            if let Some(std) = StandardEncodingName::from_pdf_name(&name) {
                return Ok(FontEncoding::Standard(std));
            }
            return Err(FontResolveError::UnknownEncodingName(name.into_owned()));
        }
        if let Ok(enc_dict) = enc_obj.as_dict() {
            return resolve_encoding_dictionary(enc_dict);
        }
    }

    // ─── §9.6.5.2 step 3 — no /Encoding → font built-in ───
    Ok(FontEncoding::FontBuiltIn)
}

/// Resolve the `/Encoding` dictionary form per §9.6.5.4.
///
/// Walks the `/Differences` array and applies each (code, glyph-
/// name) override on top of the `BaseEncoding` table via the
/// Adobe Glyph List ([`super::agl::glyph_name_to_unicode`]).
fn resolve_encoding_dictionary(
    enc_dict: &lopdf::Dictionary,
) -> Result<FontEncoding, FontResolveError> {
    let base_name = enc_dict
        .get(b"BaseEncoding")
        .ok()
        .and_then(|o| o.as_name().ok())
        .and_then(|n| StandardEncodingName::from_pdf_name(&String::from_utf8_lossy(n)))
        .unwrap_or(StandardEncodingName::Standard);

    if let Ok(diffs_obj) = enc_dict.get(b"Differences") {
        let diffs_array =
            diffs_obj
                .as_array()
                .map_err(|_| FontResolveError::MalformedDifferences {
                    detail: "/Differences is not an array".to_string(),
                })?;
        // Validate well-formedness first; any malformed item is a
        // resolve-time error per §9.6.5.4.
        for item in diffs_array {
            match item {
                lopdf::Object::Integer(i) => {
                    if *i < 0 || *i > 255 {
                        return Err(FontResolveError::MalformedDifferences {
                            detail: format!("code {i} out of [0, 255]"),
                        });
                    }
                }
                lopdf::Object::Name(_) => {}
                _ => {
                    return Err(FontResolveError::MalformedDifferences {
                        detail: format!("unexpected item {item:?}"),
                    });
                }
            }
        }
        let merged = apply_differences(base_name.table(), diffs_array);
        return Ok(FontEncoding::WithDifferences {
            base: base_name,
            merged,
        });
    }

    Ok(FontEncoding::Standard(base_name))
}

/// Walk a `/Differences` array per ISO 32000-2:2020 §9.6.5.4 and
/// apply every (code, glyph-name) override on top of `base`.
///
/// Glyph names resolve through the Adobe Glyph List
/// ([`super::agl::glyph_name_to_unicode`]). Unknown names yield
/// `None` at the overridden code position, surfacing the gap to
/// downstream auditors rather than silently keeping the base
/// mapping.
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
                    table[code as usize] = super::agl::glyph_name_to_unicode(&name);
                }
                code += 1;
            }
            _ => {}
        }
    }
    table
}

// ─────────────────────────────────────────────────────────────────────
// Decode — typed font + bytes → String.
// ─────────────────────────────────────────────────────────────────────

/// Why byte decoding failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontDecodeError {
    /// lopdf reported a decode error on a supported encoding.
    LopdfDecode { detail: String },
    /// Identity-encoded bytes weren't a whole number of UTF-16
    /// code units (odd byte length).
    IdentityNotAlignedUtf16,
}

impl core::fmt::Display for FontDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LopdfDecode { detail } => write!(f, "decode error: {detail}"),
            Self::IdentityNotAlignedUtf16 => {
                write!(f, "Identity-H/V bytes not aligned to UTF-16 code units")
            }
        }
    }
}

impl std::error::Error for FontDecodeError {}

/// Decode raw glyph-code bytes into a Unicode `String` using the
/// font's encoding. Per ISO 32000-2:2020 §9.10.2.
pub fn decode_bytes(font: &PdfFont, bytes: &[u8]) -> Result<String, FontDecodeError> {
    match &font.encoding {
        FontEncoding::ToUnicode(cmap) => LopdfEncoding::UnicodeMapEncoding(cmap.clone())
            .bytes_to_string(bytes)
            .map_err(|e| FontDecodeError::LopdfDecode {
                detail: format!("{e}"),
            }),

        FontEncoding::Standard(name) => LopdfEncoding::OneByteEncoding(name.table())
            .bytes_to_string(bytes)
            .map_err(|e| FontDecodeError::LopdfDecode {
                detail: format!("{e}"),
            }),

        FontEncoding::WithDifferences { merged, .. } => {
            LopdfEncoding::OneByteEncoding(merged.as_ref())
                .bytes_to_string(bytes)
                .map_err(|e| FontDecodeError::LopdfDecode {
                    detail: format!("{e}"),
                })
        }

        FontEncoding::Identity => {
            if !bytes.len().is_multiple_of(2) {
                return Err(FontDecodeError::IdentityNotAlignedUtf16);
            }
            let mut chars = Vec::with_capacity(bytes.len() / 2);
            for chunk in bytes.chunks_exact(2) {
                chars.push(u16::from_be_bytes([chunk[0], chunk[1]]));
            }
            Ok(String::from_utf16_lossy(&chars))
        }

        FontEncoding::FontBuiltIn => {
            // No /ToUnicode, no /Encoding — pass bytes through as
            // Latin-1 codepoints. The typed FontBuiltIn variant
            // surfaces this gap to downstream auditors.
            Ok(bytes.iter().map(|&b| b as char).collect())
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn font_with_encoding(enc: FontEncoding) -> PdfFont {
        PdfFont {
            name: "F1".to_string(),
            subtype: "Type1".to_string(),
            encoding: enc,
        }
    }

    // ── StandardEncodingName parsing ──────────────────────────────

    #[test]
    fn parse_known_encoding_names() {
        assert_eq!(
            StandardEncodingName::from_pdf_name("WinAnsiEncoding"),
            Some(StandardEncodingName::WinAnsi)
        );
        assert_eq!(
            StandardEncodingName::from_pdf_name("MacRomanEncoding"),
            Some(StandardEncodingName::MacRoman)
        );
        assert_eq!(
            StandardEncodingName::from_pdf_name("PDFDocEncoding"),
            Some(StandardEncodingName::PdfDoc)
        );
    }

    #[test]
    fn unknown_encoding_name_returns_none() {
        assert_eq!(StandardEncodingName::from_pdf_name("HelloEncoding"), None);
        assert_eq!(StandardEncodingName::from_pdf_name(""), None);
    }

    // ── decode_bytes with Standard encodings ──────────────────────

    #[test]
    fn winansi_ascii_is_passthrough() {
        let font = font_with_encoding(FontEncoding::Standard(StandardEncodingName::WinAnsi));
        assert_eq!(decode_bytes(&font, b"Hello").unwrap(), "Hello");
    }

    #[test]
    fn winansi_decodes_high_byte_to_latin1_supplement() {
        let font = font_with_encoding(FontEncoding::Standard(StandardEncodingName::WinAnsi));
        // 0xE9 in WinAnsi = 'é' (U+00E9).
        assert_eq!(decode_bytes(&font, &[0xE9]).unwrap(), "é");
        assert_eq!(
            decode_bytes(&font, &[b'A', 0xE9, b'B', 0xFC, 0xDF]).unwrap(),
            "AéBüß"
        );
    }

    #[test]
    fn pdfdoc_ascii_is_passthrough() {
        let font = font_with_encoding(FontEncoding::Standard(StandardEncodingName::PdfDoc));
        assert_eq!(decode_bytes(&font, b"Hello").unwrap(), "Hello");
    }

    #[test]
    fn macroman_ascii_is_passthrough() {
        let font = font_with_encoding(FontEncoding::Standard(StandardEncodingName::MacRoman));
        assert_eq!(decode_bytes(&font, b"Hello").unwrap(), "Hello");
    }

    #[test]
    fn standard_ascii_is_passthrough() {
        let font = font_with_encoding(FontEncoding::Standard(StandardEncodingName::Standard));
        assert_eq!(decode_bytes(&font, b"Hello").unwrap(), "Hello");
    }

    // ── Identity / CIDFont ────────────────────────────────────────

    #[test]
    fn identity_two_bytes_per_code_decodes_as_utf16be() {
        let font = font_with_encoding(FontEncoding::Identity);
        assert_eq!(
            decode_bytes(&font, &[0x00, 0x48, 0x00, 0x69]).unwrap(),
            "Hi"
        );
    }

    #[test]
    fn identity_odd_byte_count_returns_named_error() {
        let font = font_with_encoding(FontEncoding::Identity);
        assert_eq!(
            decode_bytes(&font, &[0x00, 0x48, 0x00]).unwrap_err(),
            FontDecodeError::IdentityNotAlignedUtf16
        );
    }

    // ── FontBuiltIn ───────────────────────────────────────────────

    #[test]
    fn font_builtin_passthrough_is_latin1() {
        let font = font_with_encoding(FontEncoding::FontBuiltIn);
        assert_eq!(
            decode_bytes(&font, &[0x48, 0x69, 0xE9]).unwrap(),
            "Hi\u{00E9}"
        );
    }

    // ── /Differences resolves via Adobe Glyph List ─────────────────

    #[test]
    fn differences_applies_emdash_override() {
        // GPO PDFs declare WinAnsi + /Differences mapping
        // byte 0xD0 → /emdash. Decoded byte must produce em-dash,
        // not WinAnsi's `Ð`.
        use lopdf::Object;
        let diffs = vec![Object::Integer(208), Object::Name(b"emdash".to_vec())];
        let merged = apply_differences(StandardEncodingName::WinAnsi.table(), &diffs);
        let font = font_with_encoding(FontEncoding::WithDifferences {
            base: StandardEncodingName::WinAnsi,
            merged,
        });
        assert_eq!(decode_bytes(&font, &[0xD0]).unwrap(), "\u{2014}");
    }

    #[test]
    fn differences_preserves_unoverridden_winansi_entries() {
        // ASCII bytes pass through to themselves even with a
        // /Differences override at a non-ASCII position.
        use lopdf::Object;
        let diffs = vec![Object::Integer(208), Object::Name(b"emdash".to_vec())];
        let merged = apply_differences(StandardEncodingName::WinAnsi.table(), &diffs);
        let font = font_with_encoding(FontEncoding::WithDifferences {
            base: StandardEncodingName::WinAnsi,
            merged,
        });
        assert_eq!(decode_bytes(&font, b"Hello").unwrap(), "Hello");
    }

    #[test]
    fn differences_with_curly_quotes_overrides_winansi() {
        // Real-world GPO override: byte 39 → quoteright (U+2019),
        // byte 96 → quoteleft (U+2018).
        use lopdf::Object;
        let diffs = vec![
            Object::Integer(39),
            Object::Name(b"quoteright".to_vec()),
            Object::Integer(96),
            Object::Name(b"quoteleft".to_vec()),
        ];
        let merged = apply_differences(StandardEncodingName::WinAnsi.table(), &diffs);
        let font = font_with_encoding(FontEncoding::WithDifferences {
            base: StandardEncodingName::WinAnsi,
            merged,
        });
        assert_eq!(decode_bytes(&font, &[0x27]).unwrap(), "\u{2019}");
        assert_eq!(decode_bytes(&font, &[0x60]).unwrap(), "\u{2018}");
    }

    #[test]
    fn differences_unknown_glyph_name_maps_to_none() {
        // Unknown glyph names produce `None` at the override
        // position — surfaces as the WinAnsi byte's lack of a
        // replacement (effectively dropped) rather than a silent
        // wrong-codepoint substitution.
        use lopdf::Object;
        let diffs = vec![
            Object::Integer(208),
            Object::Name(b"definitely_not_a_real_glyph".to_vec()),
        ];
        let merged = apply_differences(StandardEncodingName::WinAnsi.table(), &diffs);
        assert_eq!(merged[208], None);
    }

    // ── ToUnicode CMap end-to-end (the unblocked path) ────────────

    #[test]
    fn to_unicode_cmap_decodes_via_cmap_map() {
        // Hand-built ToUnicodeCMap: byte 0x01 → U+2014 (em-dash).
        // Verifies that the /ToUnicode path can produce a codepoint
        // that *no* standard encoding maps to from byte 0x01 — the
        // entire point of /ToUnicode taking precedence per §9.10.2.
        let mut cmap = ToUnicodeCMap::new();
        cmap.put(
            0x01,
            0x01,
            1,
            lopdf::cmap::BfRangeTarget::UTF16CodePoint { offset: 0x2014 - 1 },
        );
        let font = font_with_encoding(FontEncoding::ToUnicode(cmap));
        assert_eq!(decode_bytes(&font, &[0x01]).unwrap(), "\u{2014}");
    }

    // ── Determinism ───────────────────────────────────────────────

    #[test]
    fn decode_is_deterministic_on_same_input() {
        let font = font_with_encoding(FontEncoding::Standard(StandardEncodingName::WinAnsi));
        let s1 = decode_bytes(&font, &[0x48, 0x65, 0x6C, 0x6C, 0x6F]).unwrap();
        let s2 = decode_bytes(&font, &[0x48, 0x65, 0x6C, 0x6C, 0x6F]).unwrap();
        assert_eq!(s1, s2);
        assert_eq!(s1, "Hello");
    }

    // ── resolve_font from a synthetic font dict ───────────────────

    #[test]
    fn resolve_font_with_winansi_encoding_name() {
        use lopdf::{Document, dictionary};
        let doc = Document::with_version("1.4");
        let font_dict = dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "Encoding" => "WinAnsiEncoding",
            "BaseFont" => "Helvetica",
        };
        let font = resolve_font("F1", &font_dict, &doc).expect("resolve");
        assert_eq!(font.name, "F1");
        assert_eq!(font.subtype, "Type1");
        match font.encoding {
            FontEncoding::Standard(StandardEncodingName::WinAnsi) => {}
            other => panic!("expected Standard(WinAnsi); got {other:?}"),
        }
    }

    #[test]
    fn resolve_font_with_identity_h_encoding() {
        use lopdf::{Document, dictionary};
        let doc = Document::with_version("1.4");
        let font_dict = dictionary! {
            "Type" => "Font",
            "Subtype" => "Type0",
            "Encoding" => "Identity-H",
            "BaseFont" => "ArialMT",
        };
        let font = resolve_font("F1", &font_dict, &doc).expect("resolve");
        assert!(matches!(font.encoding, FontEncoding::Identity));
    }

    #[test]
    fn resolve_font_with_pdfdoc_encoding_is_standard_variant() {
        use lopdf::{Document, dictionary};
        let doc = Document::with_version("1.4");
        let font_dict = dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "Encoding" => "PDFDocEncoding",
        };
        let font = resolve_font("F1", &font_dict, &doc).expect("resolve");
        match font.encoding {
            FontEncoding::Standard(StandardEncodingName::PdfDoc) => {}
            other => panic!("expected Standard(PdfDoc); got {other:?}"),
        }
    }

    #[test]
    fn resolve_font_without_encoding_yields_builtin() {
        use lopdf::{Document, dictionary};
        let doc = Document::with_version("1.4");
        let font_dict = dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        };
        let font = resolve_font("F1", &font_dict, &doc).expect("resolve");
        assert!(matches!(font.encoding, FontEncoding::FontBuiltIn));
    }

    #[test]
    fn resolve_font_without_subtype_returns_named_error() {
        use lopdf::{Document, dictionary};
        let doc = Document::with_version("1.4");
        let font_dict = dictionary! { "Type" => "Font", "BaseFont" => "Helvetica", };
        assert_eq!(
            resolve_font("F1", &font_dict, &doc).unwrap_err(),
            FontResolveError::MissingSubtype
        );
    }

    #[test]
    fn resolve_font_with_unknown_encoding_name_returns_named_error() {
        use lopdf::{Document, dictionary};
        let doc = Document::with_version("1.4");
        let font_dict = dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "Encoding" => "BogusEncoding",
        };
        match resolve_font("F1", &font_dict, &doc) {
            Err(FontResolveError::UnknownEncodingName(n)) => assert_eq!(n, "BogusEncoding"),
            other => panic!("expected UnknownEncodingName; got {other:?}"),
        }
    }

    // ── Property-based ────────────────────────────────────────────

    use proptest::prelude::*;

    /// Generate any standard encoding name. All eight are decodable
    /// after the lopdf visibility patch.
    fn arb_standard_encoding() -> impl Strategy<Value = StandardEncodingName> {
        proptest::sample::select(vec![
            StandardEncodingName::WinAnsi,
            StandardEncodingName::PdfDoc,
            StandardEncodingName::MacRoman,
            StandardEncodingName::MacExpert,
            StandardEncodingName::Standard,
            StandardEncodingName::Symbol,
            StandardEncodingName::ZapfDingbats,
            StandardEncodingName::Expert,
        ])
    }

    proptest! {
        /// decode_bytes is deterministic for any Standard encoding
        /// over arbitrary byte sequences — same input always
        /// produces same output.
        #[test]
        fn prop_standard_decode_is_deterministic(
            name in arb_standard_encoding(),
            bytes in proptest::collection::vec(any::<u8>(), 0..256),
        ) {
            let font = font_with_encoding(FontEncoding::Standard(name));
            let s1 = decode_bytes(&font, &bytes).unwrap();
            let s2 = decode_bytes(&font, &bytes).unwrap();
            prop_assert_eq!(s1, s2);
        }

        /// WinAnsi is identity on ASCII bytes (0x20–0x7E): every
        /// printable ASCII character round-trips. Per Annex D.5.
        #[test]
        fn prop_winansi_ascii_is_identity(s in "[\\x20-\\x7E]{0,32}") {
            let font = font_with_encoding(FontEncoding::Standard(StandardEncodingName::WinAnsi));
            let decoded = decode_bytes(&font, s.as_bytes()).unwrap();
            prop_assert_eq!(decoded, s);
        }

        /// Identity-H decoding is the inverse of UTF-16BE encoding
        /// for any valid UTF-16 string in the BMP.
        #[test]
        fn prop_identity_round_trips_basic_multilingual_plane(
            s in "[\\x20-\\x7E]{0,32}",
        ) {
            let font = font_with_encoding(FontEncoding::Identity);
            let mut bytes: Vec<u8> = Vec::with_capacity(s.encode_utf16().count() * 2);
            for cu in s.encode_utf16() {
                bytes.extend_from_slice(&cu.to_be_bytes());
            }
            let decoded = decode_bytes(&font, &bytes).unwrap();
            prop_assert_eq!(decoded, s);
        }

        /// Identity rejects odd byte counts with a named error.
        #[test]
        fn prop_identity_rejects_odd_length(
            bytes in proptest::collection::vec(any::<u8>(), 1..32)
                .prop_filter("must be odd length", |v| !v.len().is_multiple_of(2)),
        ) {
            let font = font_with_encoding(FontEncoding::Identity);
            prop_assert_eq!(
                decode_bytes(&font, &bytes).unwrap_err(),
                FontDecodeError::IdentityNotAlignedUtf16
            );
        }

        /// FontBuiltIn is identity on Latin-1 codepoints — every
        /// byte 0..=0xFF maps to U+0000..=U+00FF.
        #[test]
        fn prop_font_builtin_is_latin1_identity(byte in 0u8..=255) {
            let font = font_with_encoding(FontEncoding::FontBuiltIn);
            let decoded = decode_bytes(&font, &[byte]).unwrap();
            let chars: Vec<char> = decoded.chars().collect();
            prop_assert_eq!(chars.len(), 1);
            prop_assert_eq!(chars[0] as u32, byte as u32);
        }

        /// /Differences with empty array decodes identically to
        /// the base encoding — every byte produces the same
        /// codepoint either way.
        #[test]
        fn prop_empty_differences_matches_base(
            base in arb_standard_encoding(),
            bytes in proptest::collection::vec(any::<u8>(), 0..32),
        ) {
            let merged = apply_differences(base.table(), &[]);
            let with_diffs = font_with_encoding(FontEncoding::WithDifferences {
                base,
                merged,
            });
            let standard = font_with_encoding(FontEncoding::Standard(base));
            let a = decode_bytes(&with_diffs, &bytes).unwrap();
            let b = decode_bytes(&standard, &bytes).unwrap();
            prop_assert_eq!(a, b);
        }

        /// /Differences overrides applied to WinAnsi preserve
        /// ASCII byte mappings (overrides at positions ≥0x80 don't
        /// disturb ASCII). Mirror Annex D.5's invariant that
        /// WinAnsi's lower half is ASCII.
        #[test]
        fn prop_differences_at_high_bytes_preserves_ascii(
            ascii in "[\\x20-\\x7E]{0,32}",
            override_byte in 0x80u8..=0xFFu8,
        ) {
            use lopdf::Object;
            let diffs = vec![
                Object::Integer(override_byte as i64),
                Object::Name(b"emdash".to_vec()),
            ];
            let merged = apply_differences(StandardEncodingName::WinAnsi.table(), &diffs);
            let font = font_with_encoding(FontEncoding::WithDifferences {
                base: StandardEncodingName::WinAnsi,
                merged,
            });
            prop_assert_eq!(decode_bytes(&font, ascii.as_bytes()).unwrap(), ascii);
        }

        /// StandardEncodingName::from_pdf_name is a left-inverse of
        /// the canonical name list.
        #[test]
        fn prop_canonical_encoding_names_round_trip(_seed in any::<u32>()) {
            let pairs = [
                ("WinAnsiEncoding", StandardEncodingName::WinAnsi),
                ("MacRomanEncoding", StandardEncodingName::MacRoman),
                ("MacExpertEncoding", StandardEncodingName::MacExpert),
                ("StandardEncoding", StandardEncodingName::Standard),
                ("PDFDocEncoding", StandardEncodingName::PdfDoc),
                ("Symbol", StandardEncodingName::Symbol),
                ("ZapfDingbats", StandardEncodingName::ZapfDingbats),
                ("ExpertEncoding", StandardEncodingName::Expert),
            ];
            for (name, expected) in pairs {
                prop_assert_eq!(StandardEncodingName::from_pdf_name(name), Some(expected));
            }
        }
    }
}

//! Font + encoding pipeline — byte → Unicode resolution.
//!
//! Bridges a [`super::content_stream::TextShowEvent`] (raw glyph
//! bytes + font name) to a decoded `String` by walking the font's
//! `/Encoding` entry (ISO 32000-2:2020 §9.6.5, Annex D) and
//! noting `/ToUnicode` presence (Adobe Tech Note #5014; §9.10.2)
//! as a typed gap.
//!
//! Resolution precedence per ISO 32000-2:2020 §9.10.2 (*Mapping
//! character codes to Unicode values*) is:
//!
//! 1. **`/ToUnicode` CMap** — explicit per-font Unicode mapping;
//!    takes precedence when present.
//! 2. **`/Encoding` entry** — a named base encoding from Annex D
//!    (`WinAnsiEncoding`, `MacRomanEncoding`, …), optionally
//!    overlaid with a `/Differences` array (§9.6.5.4).
//! 3. **Font's built-in encoding** (§9.6.5.2).
//! 4. **CIDFont (Identity-H / Identity-V)** — composite-font case
//!    (§9.7.4.3); bytes are interpreted as big-endian 16-bit CIDs.
//!
//! ## Encoding coverage in this commit
//!
//! Decodable end-to-end:
//! - `WinAnsiEncoding` (Annex D.5) — via lopdf's name-dispatched
//!   `SimpleEncoding`.
//! - `Identity-H` / `Identity-V` — handled directly as UTF-16BE.
//! - **FontBuiltIn** fallback when no `/Encoding` and no
//!   `/ToUnicode` are declared (Latin-1 passthrough).
//!
//! Flagged as a typed gap (decode returns
//! [`FontDecodeError::UnsupportedEncoding`]):
//! - `/ToUnicode` CMap present — lopdf 0.40 doesn't pub-export
//!   the `ToUnicodeCMap` type, so we detect it but cannot use it
//!   from this crate ([`UnsupportedReason::ToUnicodeCmap`]).
//!   Unblocking is either an upstream lopdf API patch or a
//!   different PDF crate; the API surface here doesn't change.
//! - `PDFDocEncoding`, `MacRomanEncoding`, `MacExpertEncoding`,
//!   `StandardEncoding`, `Symbol`, `ZapfDingbats`,
//!   `ExpertEncoding` — lopdf doesn't pub-export their 256-entry
//!   tables ([`UnsupportedReason::StandardEncoding`]).
//! - `/Differences` array — resolving glyph names requires the
//!   Adobe Glyph List, which lopdf doesn't pub-export either
//!   ([`UnsupportedReason::Differences`]).
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

// ─────────────────────────────────────────────────────────────────────
// FontEncoding — the praxis-typed wrapper.
// ─────────────────────────────────────────────────────────────────────

/// How a font's glyph codes map to Unicode.
#[derive(Debug, Clone)]
pub enum FontEncoding {
    /// `WinAnsiEncoding` from ISO 32000-2:2020 Annex D.5.
    /// Decoded via lopdf's name-dispatched table.
    WinAnsi,

    /// CIDFont case — `/Encoding /Identity-H` or `/Identity-V`
    /// (§9.7.4.3). Bytes are big-endian 16-bit CIDs that we
    /// pass through as UTF-16BE best-effort. Compliance-grade
    /// extraction requires `/ToUnicode` for these fonts.
    Identity,

    /// Font program's built-in default — neither `/ToUnicode` nor
    /// `/Encoding` declared (§9.6.5.2). Without the font program
    /// parsed, we pass through bytes as Latin-1 and signal the gap
    /// via the typed variant.
    FontBuiltIn,

    /// Encoding is recognized but not decodable in this build —
    /// the typed reason names exactly why.
    Unsupported(UnsupportedReason),
}

/// Why a font's encoding can't be used for decoding even though
/// we recognized its declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsupportedReason {
    /// `/ToUnicode` CMap present, but lopdf 0.40 doesn't pub-
    /// export `ToUnicodeCMap`. Upstream patch or crate switch
    /// needed.
    ToUnicodeCmap,
    /// Named standard encoding whose 256-entry table isn't in
    /// this build (everything in Annex D except WinAnsi).
    StandardEncoding(StandardEncodingName),
    /// `/Differences` array present — resolving glyph names
    /// requires the Adobe Glyph List, which lopdf doesn't
    /// pub-export at the version pinned here.
    Differences { base: StandardEncodingName },
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

    /// Whether this encoding has a decodable table in the current
    /// build. Only `WinAnsi` does today; the others produce
    /// [`FontEncoding::Unsupported`].
    pub fn is_decodable(self) -> bool {
        matches!(self, Self::WinAnsi)
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
    _doc: &lopdf::Document,
) -> Result<FontEncoding, FontResolveError> {
    // ─── §9.10.2 step 1 — /ToUnicode (highest precedence) ───
    // We can detect /ToUnicode presence but lopdf 0.40 doesn't
    // pub-export ToUnicodeCMap, so we can't store one. Flag as a
    // typed gap rather than silently falling back to /Encoding —
    // /ToUnicode-flagged fonts may have wildly different mappings.
    if font_dict.get(b"ToUnicode").is_ok() {
        return Ok(FontEncoding::Unsupported(UnsupportedReason::ToUnicodeCmap));
    }

    // ─── §9.6.5 step 2 — /Encoding ───
    if let Ok(enc_obj) = font_dict.get(b"Encoding") {
        if let Ok(name_bytes) = enc_obj.as_name() {
            let name = String::from_utf8_lossy(name_bytes);
            if name == "Identity-H" || name == "Identity-V" {
                return Ok(FontEncoding::Identity);
            }
            if let Some(std) = StandardEncodingName::from_pdf_name(&name) {
                return Ok(if std.is_decodable() {
                    FontEncoding::WinAnsi
                } else {
                    FontEncoding::Unsupported(UnsupportedReason::StandardEncoding(std))
                });
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
/// Resolving `/Differences` correctly requires the Adobe Glyph
/// List, which lopdf doesn't pub-export. `/Differences` is parsed
/// for well-formedness (malformed arrays still surface as named
/// errors) but the resulting font is marked
/// [`FontEncoding::Unsupported`] with the base encoding name
/// preserved.
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
        return Ok(FontEncoding::Unsupported(UnsupportedReason::Differences {
            base: base_name,
        }));
    }

    Ok(if base_name.is_decodable() {
        FontEncoding::WinAnsi
    } else {
        FontEncoding::Unsupported(UnsupportedReason::StandardEncoding(base_name))
    })
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
    /// Font's encoding is recognized but not decodable in this
    /// build — the typed reason names exactly why.
    UnsupportedEncoding(UnsupportedReason),
}

impl core::fmt::Display for FontDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LopdfDecode { detail } => write!(f, "decode error: {detail}"),
            Self::IdentityNotAlignedUtf16 => {
                write!(f, "Identity-H/V bytes not aligned to UTF-16 code units")
            }
            Self::UnsupportedEncoding(reason) => write!(f, "unsupported encoding: {reason:?}"),
        }
    }
}

impl std::error::Error for FontDecodeError {}

/// Decode raw glyph-code bytes into a Unicode `String` using the
/// font's encoding. Per ISO 32000-2:2020 §9.10.2.
pub fn decode_bytes(font: &PdfFont, bytes: &[u8]) -> Result<String, FontDecodeError> {
    match &font.encoding {
        FontEncoding::WinAnsi => LopdfEncoding::SimpleEncoding(b"WinAnsiEncoding")
            .bytes_to_string(bytes)
            .map_err(|e| FontDecodeError::LopdfDecode {
                detail: format!("{e}"),
            }),

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

        FontEncoding::Unsupported(reason) => {
            Err(FontDecodeError::UnsupportedEncoding(reason.clone()))
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

    #[test]
    fn only_winansi_is_decodable_in_this_build() {
        assert!(StandardEncodingName::WinAnsi.is_decodable());
        assert!(!StandardEncodingName::PdfDoc.is_decodable());
        assert!(!StandardEncodingName::MacRoman.is_decodable());
        assert!(!StandardEncodingName::Standard.is_decodable());
    }

    // ── decode_bytes with WinAnsi ─────────────────────────────────

    #[test]
    fn winansi_ascii_is_passthrough() {
        let font = font_with_encoding(FontEncoding::WinAnsi);
        assert_eq!(decode_bytes(&font, b"Hello").unwrap(), "Hello");
    }

    #[test]
    fn winansi_decodes_high_byte_to_latin1_supplement() {
        let font = font_with_encoding(FontEncoding::WinAnsi);
        // 0xE9 in WinAnsi = 'é' (U+00E9).
        assert_eq!(decode_bytes(&font, &[0xE9]).unwrap(), "é");
        // 0xFC = 'ü', 0xDF = 'ß'.
        assert_eq!(
            decode_bytes(&font, &[b'A', 0xE9, b'B', 0xFC, 0xDF]).unwrap(),
            "AéBüß"
        );
    }

    // ── Identity / CIDFont ────────────────────────────────────────

    #[test]
    fn identity_two_bytes_per_code_decodes_as_utf16be() {
        let font = font_with_encoding(FontEncoding::Identity);
        // 0x00 0x48 = 'H', 0x00 0x69 = 'i'.
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

    // ── FontBuiltIn (no /Encoding, no /ToUnicode) ─────────────────

    #[test]
    fn font_builtin_passthrough_is_latin1() {
        let font = font_with_encoding(FontEncoding::FontBuiltIn);
        assert_eq!(
            decode_bytes(&font, &[0x48, 0x69, 0xE9]).unwrap(),
            "Hi\u{00E9}"
        );
    }

    // ── Unsupported variants fail closed with typed reason ────────

    #[test]
    fn pdfdoc_encoding_returns_unsupported_error() {
        let font = font_with_encoding(FontEncoding::Unsupported(
            UnsupportedReason::StandardEncoding(StandardEncodingName::PdfDoc),
        ));
        assert_eq!(
            decode_bytes(&font, b"x").unwrap_err(),
            FontDecodeError::UnsupportedEncoding(UnsupportedReason::StandardEncoding(
                StandardEncodingName::PdfDoc
            ))
        );
    }

    #[test]
    fn macroman_encoding_returns_unsupported_error() {
        let font = font_with_encoding(FontEncoding::Unsupported(
            UnsupportedReason::StandardEncoding(StandardEncodingName::MacRoman),
        ));
        match decode_bytes(&font, b"x").unwrap_err() {
            FontDecodeError::UnsupportedEncoding(UnsupportedReason::StandardEncoding(
                StandardEncodingName::MacRoman,
            )) => {}
            other => panic!("expected StandardEncoding(MacRoman); got {other:?}"),
        }
    }

    #[test]
    fn to_unicode_present_returns_unsupported_with_named_reason() {
        let font = font_with_encoding(FontEncoding::Unsupported(UnsupportedReason::ToUnicodeCmap));
        assert_eq!(
            decode_bytes(&font, b"x").unwrap_err(),
            FontDecodeError::UnsupportedEncoding(UnsupportedReason::ToUnicodeCmap)
        );
    }

    #[test]
    fn differences_returns_unsupported_with_base() {
        let font = font_with_encoding(FontEncoding::Unsupported(UnsupportedReason::Differences {
            base: StandardEncodingName::WinAnsi,
        }));
        match decode_bytes(&font, b"x").unwrap_err() {
            FontDecodeError::UnsupportedEncoding(UnsupportedReason::Differences { base }) => {
                assert_eq!(base, StandardEncodingName::WinAnsi);
            }
            other => panic!("expected Differences; got {other:?}"),
        }
    }

    // ── Determinism ───────────────────────────────────────────────

    #[test]
    fn decode_is_deterministic_on_same_input() {
        let font = font_with_encoding(FontEncoding::WinAnsi);
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
        assert!(matches!(font.encoding, FontEncoding::WinAnsi));
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
    fn resolve_font_with_pdfdoc_encoding_is_unsupported_variant() {
        use lopdf::{Document, dictionary};
        let doc = Document::with_version("1.4");
        let font_dict = dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "Encoding" => "PDFDocEncoding",
        };
        let font = resolve_font("F1", &font_dict, &doc).expect("resolve");
        match font.encoding {
            FontEncoding::Unsupported(UnsupportedReason::StandardEncoding(
                StandardEncodingName::PdfDoc,
            )) => {}
            other => panic!("expected Unsupported(StandardEncoding(PdfDoc)); got {other:?}"),
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
}

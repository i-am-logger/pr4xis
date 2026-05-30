//! W3C XML 1.0 §F + §4.3.3 character-encoding label families.
//!
//! The XML 1.0 parser's `parser::grammar` consistency check at the
//! XML-declaration boundary asks: *given the encoding label declared
//! inside `<?xml encoding="…"?>`, is the label a 16-bit-Unicode label?*
//! That predicate must come from the published spec, not from a
//! hand-coded `lower == "utf-16" || "utf-16le" || …` list.
//!
//! This module loads the canonical encoding-label vocabulary from the
//! bundled W3C XML 1.0 Fifth Edition spec — the same bytes the parser
//! grammar predicates derive from (`xml_1_0_fifth_edition@2008` in
//! `praxis.toml`, sha256-pinned in `praxis.lock`). The §F BOM-detection
//! table cells and the §4.3.3 SHOULD-name list are walked at first
//! call and the resulting label sets cached via `OnceLock`.
//!
//! Per `feedback_bottom_up_loaded_not_encoded`: the canonical names
//! (UTF-16, UTF-16BE, UTF-16LE, ISO-10646-UCS-2) come from the loaded
//! spec source — never from a hand-curated Rust slice. A corpus-wide
//! audit (`feedback_corpus_wide_audit_on_load`) at test time fails
//! closed if any expected name is missing from the spec or appears
//! outside §F / §4.3.3, so spec-source drift surfaces immediately.
//!
//! ## Citation
//!
//! - **Bray, T., Paoli, J., Sperberg-McQueen, C. M., Maler, E. &
//!   Yergeau, F.** (eds.) (2008) *Extensible Markup Language (XML)
//!   1.0 (Fifth Edition)*, W3C Recommendation 26 November 2008.
//!   - **§F (Autodetection of Character Encodings)** — the
//!     BOM-detection table pairing byte patterns with encoding-name
//!     labels (UTF-16 big/little-endian; UTF-16BE / UTF-16LE or
//!     ISO-10646-UCS-2 without BOM).
//!   - **§4.3.3 (Character Encoding in Entities)** — the normative
//!     SHOULD-list of Unicode encoding names: UTF-8, UTF-16,
//!     ISO-10646-UCS-2, ISO-10646-UCS-4. Case-insensitive matching
//!     is RECOMMENDED.
//!   - **Erratum E05** — clarifies that "UTF-16" in this spec does
//!     not apply to "related character encodings, including but not
//!     limited to UTF-16BE, UTF-16LE, or CESU-8".

use std::sync::OnceLock;

/// W3C XML 1.0 §F + §4.3.3 encoding-label families. Each family is
/// the set of encoding-name labels the loaded spec source declares
/// for one byte-pattern class.
///
/// Per §4.3.3, encoding-name matching is case-insensitive — the
/// predicates [`Self::is_utf16_family`], [`Self::is_utf8_family`],
/// and [`Self::is_ucs4_family`] are case-folding membership tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlEncodingFamilies {
    /// Labels for the UTF-8 encoding family. §F BOM-detection:
    /// `EF BB BF` → "UTF-8". §4.3.3 normative SHOULD-name: "UTF-8".
    utf8: Vec<String>,
    /// Labels for the 16-bit Unicode family — UTF-16 plus the
    /// endianness-explicit aliases UTF-16BE / UTF-16LE (erratum E05),
    /// plus the §4.3.3 SHOULD-name "ISO-10646-UCS-2". §F
    /// BOM-detection: `FE FF` / `FF FE` → "UTF-16, big/little-endian";
    /// `00 3C 00 3F` / `3C 00 3F 00` → "UTF-16BE or big-endian
    /// ISO-10646-UCS-2" / "UTF-16LE or little-endian ISO-10646-UCS-2".
    utf16: Vec<String>,
    /// Labels for the 32-bit Unicode family — ISO-10646-UCS-4
    /// (§4.3.3 SHOULD-name; §F BOM-less detection table).
    ucs4: Vec<String>,
}

impl XmlEncodingFamilies {
    /// The UTF-8 family labels in source order.
    #[must_use]
    pub fn utf8_labels(&self) -> &[String] {
        &self.utf8
    }

    /// The 16-bit Unicode family labels in source order.
    #[must_use]
    pub fn utf16_labels(&self) -> &[String] {
        &self.utf16
    }

    /// The 32-bit Unicode family labels in source order.
    #[must_use]
    pub fn ucs4_labels(&self) -> &[String] {
        &self.ucs4
    }

    /// True iff `name` matches a W3C-declared 16-bit Unicode
    /// encoding label, case-insensitively per §4.3.3.
    ///
    /// Returns `false` for labels not literally in §F or §4.3.3 —
    /// notably the bare short form `UCS-2` (the spec uses only the
    /// long `ISO-10646-UCS-2` form), and IANA-only aliases. Per §4.3.3
    /// such labels are "to be treated as unknown" by an XML processor
    /// that does not implement the IANA registry.
    #[must_use]
    pub fn is_utf16_family(&self, name: &str) -> bool {
        self.utf16.iter().any(|s| s.eq_ignore_ascii_case(name))
    }

    /// True iff `name` matches a W3C-declared UTF-8 label,
    /// case-insensitively per §4.3.3.
    #[must_use]
    pub fn is_utf8_family(&self, name: &str) -> bool {
        self.utf8.iter().any(|s| s.eq_ignore_ascii_case(name))
    }

    /// True iff `name` matches a W3C-declared 32-bit Unicode label,
    /// case-insensitively per §4.3.3.
    #[must_use]
    pub fn is_ucs4_family(&self, name: &str) -> bool {
        self.ucs4.iter().any(|s| s.eq_ignore_ascii_case(name))
    }
}

/// The loaded W3C XML 1.0 encoding-label families — parsed from the
/// bundled spec bytes on first call, cached thereafter.
///
/// Per `feedback_bottom_up_loaded_not_encoded`: every parser site
/// that needs to classify an encoding-declaration label MUST query
/// this function rather than hand-coding the alias list.
///
/// Panics if the spec source has drifted such that any of the
/// §4.3.3 + §F + erratum-E05 canonical names (`UTF-16`, `UTF-16BE`,
/// `UTF-16LE`, `ISO-10646-UCS-2`, `UTF-8`, `ISO-10646-UCS-4`) is
/// missing from the corresponding spec section. A panic here means
/// the bundled `xml_1_0_fifth_edition@2008` bytes no longer match
/// the W3C-published source the praxis.lock hash certifies.
#[must_use]
pub fn loaded_xml_encoding_families() -> &'static XmlEncodingFamilies {
    static FAMILIES: OnceLock<XmlEncodingFamilies> = OnceLock::new();
    FAMILIES.get_or_init(|| {
        extract_encoding_families(super::spec::XML_1_0_FIFTH_EDITION).expect(
            "W3C XML 1.0 §F + §4.3.3 must yield the canonical \
             encoding-label families from the loaded spec source",
        )
    })
}

/// Errors raised while extracting encoding-label families from the
/// bundled spec source. Each variant identifies a structural
/// invariant the W3C spec must continue to satisfy.
#[derive(Debug, PartialEq, Eq)]
pub enum EncodingLabelExtractionError {
    /// The §F (`<inform-div1 id="sec-guessing">`) section was not
    /// located in the spec source.
    FAnchorNotFound,
    /// The §4.3.3 (`<div3 id="charencoding">`) section was not
    /// located in the spec source.
    Section4_3_3AnchorNotFound,
    /// A canonical encoding-name label expected per §F + §4.3.3 +
    /// erratum E05 was not found in any of the spec's
    /// encoding-name carriers.
    MissingCanonicalLabel(&'static str),
}

impl std::fmt::Display for EncodingLabelExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FAnchorNotFound => {
                write!(
                    f,
                    "W3C XML 1.0 §F (sec-guessing) anchor not found in spec source"
                )
            }
            Self::Section4_3_3AnchorNotFound => write!(
                f,
                "W3C XML 1.0 §4.3.3 (charencoding) anchor not found in spec source"
            ),
            Self::MissingCanonicalLabel(name) => write!(
                f,
                "W3C XML 1.0 canonical encoding label {name:?} not found in §F or §4.3.3"
            ),
        }
    }
}

impl std::error::Error for EncodingLabelExtractionError {}

/// Extract the encoding-label families from the bundled W3C XML 1.0
/// spec bytes.
///
/// Walks §F's `<inform-div1 id="sec-guessing">` table cells and
/// §4.3.3's `<div3 id="charencoding">` `<code>` blocks. Tokenises
/// each cell into EncName candidates per W3C XML 1.0 §4.3.4 \[81\]
/// `EncName ::= [A-Za-z] ([A-Za-z0-9._] | '-')*`, then filters and
/// classifies by W3C-defined name shape.
///
/// Returns `Ok(XmlEncodingFamilies)` on success. Returns
/// `Err(EncodingLabelExtractionError)` if any required canonical
/// label is absent — spec-source drift fails closed.
pub(crate) fn extract_encoding_families(
    spec: &str,
) -> Result<XmlEncodingFamilies, EncodingLabelExtractionError> {
    let f_text = locate_section(spec, "sec-guessing", "</inform-div1>")
        .ok_or(EncodingLabelExtractionError::FAnchorNotFound)?;
    let charenc_text = locate_section(spec, "charencoding", "</div3>")
        .ok_or(EncodingLabelExtractionError::Section4_3_3AnchorNotFound)?;

    // §F: collect labels from every <td>...</td> cell. The §F table
    // pairs byte-patterns with encoding-name text in plain-prose
    // cells (e.g. "UTF-16, big-endian"), so we walk each cell as a
    // distinct context for tokenisation.
    let mut all_labels: Vec<String> = Vec::new();
    collect_labels_from_tags(f_text, "<td>", "</td>", &mut all_labels);
    // §4.3.3: collect labels from each <code>...</code> element —
    // the SHOULD-list ("UTF-8", "UTF-16", "ISO-10646-UCS-2", …) is
    // marked up structurally.
    collect_labels_from_tags(charenc_text, "<code>", "</code>", &mut all_labels);

    // Deduplicate preserving first-seen order; case-insensitive.
    let mut deduped: Vec<String> = Vec::new();
    for label in all_labels {
        if !deduped
            .iter()
            .any(|s: &String| s.eq_ignore_ascii_case(&label))
        {
            deduped.push(label);
        }
    }

    let mut utf8: Vec<String> = Vec::new();
    let mut utf16: Vec<String> = Vec::new();
    let mut ucs4: Vec<String> = Vec::new();

    for label in &deduped {
        // Classify by W3C-defined name shape per §F + §4.3.3 + E05.
        let upper = label.to_ascii_uppercase();
        if upper == "UTF-8" {
            utf8.push(label.clone());
        } else if upper.starts_with("UTF-16") {
            // Catches UTF-16, UTF-16BE, UTF-16LE per E05.
            utf16.push(label.clone());
        } else if upper == "ISO-10646-UCS-2" {
            utf16.push(label.clone());
        } else if upper == "ISO-10646-UCS-4" {
            ucs4.push(label.clone());
        }
        // Other 8-bit / 16-bit encodings (ISO-8859-N, EUC-JP,
        // Shift_JIS) are documented in §4.3.3 but not part of the
        // Unicode-family check the parser site needs. They are
        // intentionally omitted.
    }

    // Corpus-wide audit (`feedback_corpus_wide_audit_on_load`):
    // every canonical label per §F + §4.3.3 + E05 MUST be present.
    // Fail closed on drift.
    require_label(&utf8, "UTF-8")?;
    require_label(&utf16, "UTF-16")?;
    require_label(&utf16, "UTF-16BE")?;
    require_label(&utf16, "UTF-16LE")?;
    require_label(&utf16, "ISO-10646-UCS-2")?;
    require_label(&ucs4, "ISO-10646-UCS-4")?;

    Ok(XmlEncodingFamilies { utf8, utf16, ucs4 })
}

fn require_label(
    labels: &[String],
    canonical: &'static str,
) -> Result<(), EncodingLabelExtractionError> {
    if labels.iter().any(|s| s.eq_ignore_ascii_case(canonical)) {
        Ok(())
    } else {
        Err(EncodingLabelExtractionError::MissingCanonicalLabel(
            canonical,
        ))
    }
}

/// Find the section anchored by `id="<id>"` in `spec` and return
/// the substring from the anchor up to the next occurrence of
/// `close_marker` after it. Returns `None` if the anchor is missing.
fn locate_section<'a>(spec: &'a str, id: &str, close_marker: &str) -> Option<&'a str> {
    let needle = format!("id=\"{id}\"");
    let start = spec.find(&needle)?;
    let after = &spec[start..];
    let end_rel = after.find(close_marker)?;
    Some(&after[..end_rel + close_marker.len()])
}

/// Walk every `<open_tag>…</close_tag>` pair in `text`, tokenise
/// each cell's body into EncName candidates, and append to `out`.
fn collect_labels_from_tags(text: &str, open_tag: &str, close_tag: &str, out: &mut Vec<String>) {
    let mut cursor = 0usize;
    while let Some(open_rel) = text[cursor..].find(open_tag) {
        let body_start = cursor + open_rel + open_tag.len();
        let Some(close_rel) = text[body_start..].find(close_tag) else {
            break;
        };
        let cell = &text[body_start..body_start + close_rel];
        extract_encname_candidates(cell, out);
        cursor = body_start + close_rel + close_tag.len();
    }
}

/// Tokenise `text` for EncName candidates and append them to `out`.
///
/// Per W3C XML 1.0 §4.3.4 \[81\] `EncName ::= [A-Za-z]
/// ([A-Za-z0-9._] | '-')*`. We further filter to:
/// - **starts with an uppercase letter** — encoding labels in §F
///   and §4.3.3 are conventionally capitalised; prose words like
///   "big-endian", "and", "or" are lowercase and rejected.
/// - **contains at least one digit, hyphen, or underscore** —
///   excludes pure-letter words like "ASCII" (irrelevant for the
///   Unicode-family check) and English prose. The structured
///   encoding labels we want (UTF-8, UTF-16, UTF-16BE, UTF-16LE,
///   ISO-10646-UCS-2, ISO-10646-UCS-4, EUC-JP, Shift_JIS, ISO-2022-JP,
///   ISO-8859-N) all satisfy this.
fn extract_encname_candidates(text: &str, out: &mut Vec<String>) {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Look for an uppercase-letter start.
        if !bytes[i].is_ascii_uppercase() {
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        while i < bytes.len() {
            let c = bytes[i];
            if c.is_ascii_alphanumeric() || c == b'.' || c == b'_' || c == b'-' {
                i += 1;
            } else {
                break;
            }
        }
        let token = &text[start..i];
        let has_special = token
            .bytes()
            .any(|c| c.is_ascii_digit() || c == b'-' || c == b'_' || c == b'.');
        // Encoding labels in this spec are at least 3 characters long
        // (the shortest is "UTF-8" at 5). Reject short uppercase
        // initialisms that happen to start prose tokens (e.g.
        // "ASCII", "JIS").
        if has_special && token.len() >= 4 {
            out.push(token.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loaded_families_present() {
        // The cached accessor must populate without panicking,
        // which (per `extract_encoding_families`) implies every
        // canonical §F + §4.3.3 + E05 label is present in the
        // loaded spec source.
        let families = loaded_xml_encoding_families();
        assert!(
            !families.utf8_labels().is_empty(),
            "UTF-8 family must be non-empty"
        );
        assert!(
            !families.utf16_labels().is_empty(),
            "UTF-16 family must be non-empty"
        );
        assert!(
            !families.ucs4_labels().is_empty(),
            "UCS-4 family must be non-empty"
        );
    }

    #[test]
    fn utf16_family_contains_all_canonical_w3c_names() {
        // The 16-bit Unicode label set the parser's
        // encoding-consistency check at §4.3.3 + §F operates over.
        let families = loaded_xml_encoding_families();
        for canonical in ["UTF-16", "UTF-16BE", "UTF-16LE", "ISO-10646-UCS-2"] {
            assert!(
                families.is_utf16_family(canonical),
                "is_utf16_family must return true for §F/§4.3.3 canonical name {canonical}"
            );
        }
    }

    #[test]
    fn utf16_family_predicate_is_case_insensitive() {
        // §4.3.3 RECOMMENDS case-insensitive matching of encoding
        // names. Verify the predicate honours that.
        let families = loaded_xml_encoding_families();
        assert!(families.is_utf16_family("utf-16"));
        assert!(families.is_utf16_family("UTF-16"));
        assert!(families.is_utf16_family("Utf-16"));
        assert!(families.is_utf16_family("utf-16be"));
        assert!(families.is_utf16_family("UTF-16BE"));
        assert!(families.is_utf16_family("utf-16le"));
        assert!(families.is_utf16_family("iso-10646-ucs-2"));
        assert!(families.is_utf16_family("ISO-10646-UCS-2"));
    }

    #[test]
    fn utf16_family_predicate_rejects_iana_only_short_alias() {
        // The bare "UCS-2" short form is an IANA alias of
        // ISO-10646-UCS-2 but does NOT appear normatively in the
        // W3C XML 1.0 spec. Per §4.3.3 "treat it as unknown"
        // semantics, the W3C-grounded predicate returns false.
        // A future praxis source registering IANA Character Sets
        // would be the natural place to recognise "UCS-2" via
        // the alias chain.
        let families = loaded_xml_encoding_families();
        assert!(!families.is_utf16_family("UCS-2"));
        assert!(!families.is_utf16_family("ucs-2"));
    }

    #[test]
    fn utf8_family_contains_utf_8() {
        let families = loaded_xml_encoding_families();
        assert!(families.is_utf8_family("UTF-8"));
        assert!(families.is_utf8_family("utf-8"));
        // UTF-8 is the only label in the UTF-8 family per W3C §4.3.3.
        assert!(!families.is_utf8_family("UTF-16"));
    }

    #[test]
    fn ucs4_family_contains_iso_10646_ucs_4() {
        let families = loaded_xml_encoding_families();
        assert!(families.is_ucs4_family("ISO-10646-UCS-4"));
        assert!(families.is_ucs4_family("iso-10646-ucs-4"));
        // The UCS-4 family must NOT spill into the UTF-16 family.
        assert!(!families.is_utf16_family("ISO-10646-UCS-4"));
    }

    #[test]
    fn extractor_rejects_prose_tokens() {
        // The §F table cells embed natural-language prose like
        // "UTF-16, big-endian" or "big-endian machine (1234 order)".
        // The tokeniser MUST reject "big-endian" (lowercase) but
        // accept "UTF-16" (uppercase + digit).
        let mut out = Vec::new();
        extract_encname_candidates("UTF-16, big-endian", &mut out);
        assert_eq!(out, vec!["UTF-16".to_string()]);
    }

    #[test]
    fn extractor_handles_iso_10646_ucs_2_as_single_token() {
        // Per EncName [81], hyphenated compound names are single
        // tokens. "ISO-10646-UCS-2" must NOT decompose into
        // {"ISO", "10646", "UCS", "2"}.
        let mut out = Vec::new();
        extract_encname_candidates(
            "UTF-16BE or big-endian ISO-10646-UCS-2 or other encoding",
            &mut out,
        );
        // Expected: UTF-16BE and ISO-10646-UCS-2.
        assert!(out.contains(&"UTF-16BE".to_string()), "got {out:?}");
        assert!(out.contains(&"ISO-10646-UCS-2".to_string()), "got {out:?}");
    }

    #[test]
    fn locate_section_finds_charencoding() {
        // Spot-check the section locator on a synthetic fragment so
        // the real-spec test below can rely on locate_section being
        // sound.
        let spec = r#"foo <div3 id="charencoding"><p>UTF-8</p></div3> bar"#;
        let section = locate_section(spec, "charencoding", "</div3>").unwrap();
        assert!(section.contains("UTF-8"));
        assert!(section.contains("</div3>"));
    }

    #[test]
    fn extractor_fails_closed_on_missing_canonical_label() {
        // Per `feedback_corpus_wide_audit_on_load`: removing a
        // canonical label from the input must cause extraction to
        // fail rather than silently produce a subset.
        let spec_without_utf16le = "<inform-div1 id=\"sec-guessing\"><td>UTF-16</td>\
            <td>UTF-16BE</td><td>ISO-10646-UCS-2</td><td>ISO-10646-UCS-4</td>\
            <td>UTF-8</td></inform-div1>\
            <div3 id=\"charencoding\"><code>UTF-8</code></div3>";
        let result = extract_encoding_families(spec_without_utf16le);
        assert_eq!(
            result,
            Err(EncodingLabelExtractionError::MissingCanonicalLabel(
                "UTF-16LE"
            ))
        );
    }

    #[test]
    fn extractor_fails_closed_on_missing_section_f() {
        let result = extract_encoding_families("<div3 id=\"charencoding\"></div3>");
        assert_eq!(result, Err(EncodingLabelExtractionError::FAnchorNotFound));
    }

    #[test]
    fn extractor_fails_closed_on_missing_section_4_3_3() {
        let result = extract_encoding_families("<inform-div1 id=\"sec-guessing\"></inform-div1>");
        assert_eq!(
            result,
            Err(EncodingLabelExtractionError::Section4_3_3AnchorNotFound)
        );
    }
}

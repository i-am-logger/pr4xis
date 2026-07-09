//! Adobe Glyph List decoder.
//!
//! Parses the published Adobe Glyph List (AGL; Adobe Systems Inc.,
//! 2002–2019; BSD-3-Clause) into an in-memory `name → Unicode
//! codepoint` table. The AGL is cited by ISO 32000-2:2020
//! §9.6.5.4 + Adobe Tech Note #5014 as the canonical resolver for
//! PDF `/Differences` glyph names.
//!
//! ## File format (per the AGL preamble)
//!
//! Lines beginning with `#` are comments and ignored. Data lines
//! are `<glyphname>;<hex codepoint>[ <hex codepoint>]*`. Multi-
//! codepoint mappings (e.g. `ffi;0066 0066 0069`) flatten to the
//! first codepoint — body-text PDFs use single-codepoint glyph
//! names exclusively.
//!
//! ## Relationship to `pdf::agl`
//!
//! The PDF text-extraction pipeline at
//! `crates/domains/src/social/software/binary/pdf/agl.rs`
//! `include_str!`s the same data file with the same parser
//! semantics. The pipeline's `OnceLock` cache is what runtime PDF
//! decoders consult per (`/Differences` array name) lookup; this
//! decoder is the registry-side entry point that the data-
//! provisioning machinery uses to verify "every registered source
//! has a runtime decoder" (the
//! [`super::super::ontology::DecoderTotalityPerKind`] axiom).

use std::collections::HashMap;

/// Parse AGL bytes into a name → Unicode codepoint map.
///
/// Returns the first Unicode codepoint per glyph name. Unknown
/// or malformed lines are skipped — the AGL itself is well-
/// formed by publisher policy, so malformed lines on disk indicate
/// file corruption rather than a parser deficiency.
/// The [`ContentType`](crate::applied::data_provisioning::ontology::ContentType)
/// this module realizes -- the single declaration of which content type
/// this file decodes, read by `super::has_decoder_for` (audit 2026-06-12 D-22).
pub const DECODES: crate::applied::data_provisioning::ontology::ContentType =
    crate::applied::data_provisioning::ontology::ContentType::AdobeGlyphList;

pub fn parse(bytes: &str) -> HashMap<String, u16> {
    let mut map = HashMap::with_capacity(4500);
    for line in bytes.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, rest)) = line.split_once(';') else {
            continue;
        };
        let first_hex = rest.split_ascii_whitespace().next().unwrap_or("");
        let Ok(codepoint) = u32::from_str_radix(first_hex, 16) else {
            continue;
        };
        if codepoint <= 0xFFFF {
            map.insert(name.to_string(), codepoint as u16);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parses_canonical_names() {
        let input = "\
            # comment line\n\
            A;0041\n\
            emdash;2014\n\
            endash;2013\n\
            quoteleft;2018\n\
            quoteright;2019\n";
        let map = parse(input);
        assert_eq!(map.get("A"), Some(&0x0041));
        assert_eq!(map.get("emdash"), Some(&0x2014));
        assert_eq!(map.get("endash"), Some(&0x2013));
        assert_eq!(map.get("quoteleft"), Some(&0x2018));
        assert_eq!(map.get("quoteright"), Some(&0x2019));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn comments_and_empty_lines_skipped() {
        let input = "\n# this is a comment\n\nA;0041\n";
        let map = parse(input);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("A"), Some(&0x0041));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn multi_codepoint_uses_first() {
        let input = "ffi;0066 0066 0069\n";
        let map = parse(input);
        assert_eq!(map.get("ffi"), Some(&0x0066));
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn malformed_lines_skipped() {
        let input = "no_semicolon\nbad_hex;ZZZZ\nA;0041\n";
        let map = parse(input);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("A"), Some(&0x0041));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parse_real_agl_population() {
        // Sanity check the actual embedded glyph list parses to
        // the expected scale (~4300 entries). The raw `.txt` is fetch-only
        // (`pr4xis update`) and ships in NO crate; the bytes are materialized
        // from the committed `.prx` through the fail-closed
        // `[compact_archive_signatures]` gate — the SAME load path the runtime
        // `pdf::agl::glyph_list_bytes()` uses, so a clean checkout (no
        // `pr4xis update`) still compiles + runs this test.
        use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
        const GLYPH_LIST_PRX: &[u8] = include_bytes!("../../../../data/adobe/glyphlist.prx");
        let bytes = raw_source_text_embedded("adobe_glyph_list", "2019", GLYPH_LIST_PRX);
        let map = parse(&bytes);
        assert!(
            map.len() > 4000,
            "expected >4000 AGL entries, got {}",
            map.len()
        );
        // Spot-check the GPO-PDF-required names.
        assert_eq!(map.get("emdash"), Some(&0x2014));
        assert_eq!(map.get("endash"), Some(&0x2013));
        assert_eq!(map.get("quoteleft"), Some(&0x2018));
        assert_eq!(map.get("quoteright"), Some(&0x2019));
    }
}

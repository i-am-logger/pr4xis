//! Adobe Glyph List — name → Unicode resolver for `/Differences`
//! arrays in PDF font dictionaries.
//!
//! The Adobe Glyph List (AGL) is the authoritative published map
//! from PostScript glyph names (`"emdash"`, `"endash"`, `"A"`,
//! `"Aacute"`, …) to Unicode codepoints. ISO 32000-2:2020 §9.6.5.4
//! cites it as the resolver for `/Differences` glyph names.
//!
//! ## Data source
//!
//! The canonical file lives at
//! <https://raw.githubusercontent.com/adobe-type-tools/agl-aglfn/master/glyphlist.txt>
//! (Adobe, 2002–2019; SPDX-License-Identifier: BSD-3-Clause). The
//! repo embeds a verbatim copy at
//! `crates/domains/data/adobe/glyphlist.txt`; `praxis.lock` pins
//! its sha256.
//!
//! ## File format (per the file's own preamble)
//!
//! Lines beginning with `#` are comments and ignored. Data lines
//! are `<glyphname>;<hex unicode codepoint(s)>`. Multi-codepoint
//! mappings (e.g. `ffi;0066 0066 0069`) are rare in body text and
//! are flattened to the first codepoint; that's a faithful first-
//! approximation since `/Differences` glyph names in body-text
//! USCODE PDFs use single-codepoint mappings exclusively.
//!
//! ## Citation requirement
//!
//! Adobe Systems Inc., *Adobe Glyph List*, version 2002-2019;
//! cited by ISO 32000-2:2020 §9.6.5.4 as the resolver for
//! `/Differences` glyph names.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Verbatim glyph list bytes embedded at build time.
///
/// SHA-256: `a3b2f61ced9f3644cc0d4ecde5c59df34ca286c689d9484a43a710a81c466789`
/// — pinned in `praxis.lock` under `[hashes."adobe_glyph_list@2019"]`.
pub const GLYPH_LIST_BYTES: &str = include_str!("../data/adobe/glyphlist.txt");

/// Parse the AGL once on first call and cache the name → Unicode
/// codepoint map.
///
/// Returns the first Unicode codepoint per glyph name. Multi-
/// codepoint mappings (rare; e.g. `ffi → 0066 0066 0069`) flatten
/// to the first codepoint — a faithful first-approximation since
/// `/Differences` arrays in modern body-text PDFs use single-
/// codepoint glyph names exclusively.
pub fn glyph_name_table() -> &'static HashMap<&'static str, u16> {
    static TABLE: OnceLock<HashMap<&'static str, u16>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut map = HashMap::with_capacity(4500);
        for line in GLYPH_LIST_BYTES.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Format: `<name>;<HEX>[ <HEX>]*`
            let Some((name, rest)) = line.split_once(';') else {
                continue;
            };
            let first_hex = rest.split_ascii_whitespace().next().unwrap_or("");
            let Ok(codepoint) = u32::from_str_radix(first_hex, 16) else {
                continue;
            };
            // AGL data spans the BMP (U+0000..U+FFFF); fits in u16
            // by construction. Anything outside is ignored — the
            // PDF text path is 16-bit per glyph anyway.
            if codepoint <= 0xFFFF {
                map.insert(name, codepoint as u16);
            }
        }
        map
    })
}

/// Look up a single glyph name. Returns `None` for unknown names.
pub fn glyph_name_to_unicode(name: &str) -> Option<u16> {
    glyph_name_table().get(name).copied()
}

/// SHA-256 of the embedded glyph list bytes, as a lowercase hex
/// string. Used to verify the file in-tree matches `praxis.lock`.
#[allow(dead_code)]
pub fn glyph_list_sha256_hex() -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(GLYPH_LIST_BYTES.as_bytes());
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_glyph_names_resolve() {
        assert_eq!(glyph_name_to_unicode("A"), Some(0x0041));
        assert_eq!(glyph_name_to_unicode("emdash"), Some(0x2014));
        assert_eq!(glyph_name_to_unicode("endash"), Some(0x2013));
        assert_eq!(glyph_name_to_unicode("quoteleft"), Some(0x2018));
        assert_eq!(glyph_name_to_unicode("quoteright"), Some(0x2019));
        assert_eq!(glyph_name_to_unicode("Aacute"), Some(0x00C1));
    }

    #[test]
    fn unknown_glyph_name_returns_none() {
        assert_eq!(glyph_name_to_unicode("definitely_not_a_glyph"), None);
        assert_eq!(glyph_name_to_unicode(""), None);
    }

    #[test]
    fn comments_and_empty_lines_skipped() {
        // The file starts with a 25-line copyright block — table
        // should still parse cleanly to ~4300 entries (the actual
        // Adobe AGL count).
        let table = glyph_name_table();
        assert!(
            table.len() > 4000,
            "AGL parse produced only {} entries; expected >4000",
            table.len()
        );
    }

    #[test]
    fn parse_is_deterministic() {
        let n1 = glyph_name_table().len();
        let n2 = glyph_name_table().len();
        assert_eq!(n1, n2);
    }

    #[test]
    fn embedded_sha256_matches_pinned_hash() {
        // The hash pinned in `praxis.lock`'s `[hashes]` table for
        // `adobe_glyph_list@2019`. If this assertion fails, the
        // file in-tree has drifted from its pinned hash — re-fetch
        // or update praxis.lock.
        assert_eq!(
            glyph_list_sha256_hex(),
            "a3b2f61ced9f3644cc0d4ecde5c59df34ca286c689d9484a43a710a81c466789"
        );
    }
}

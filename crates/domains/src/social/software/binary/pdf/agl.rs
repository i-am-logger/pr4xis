//! Adobe Glyph List — name → Unicode resolver for `/Differences`
//! arrays in PDF font dictionaries (runtime path).
//!
//! The glyph table is materialized from the committed
//! `glyphlist.prx` (`include_bytes!`) through the generalized
//! fail-closed `[compact_archive_signatures]` content gate — the
//! raw `glyphlist.txt` is fetch-only (`pr4xis update`) and ships
//! in NO crate. See [`glyph_list_bytes`].
//!
//! ## Data source
//!
//! Adobe Systems Inc., *Adobe Glyph List*, 2002–2019,
//! SPDX-License-Identifier: BSD-3-Clause. The canonical file
//! lives at
//! <https://raw.githubusercontent.com/adobe-type-tools/agl-aglfn/master/glyphlist.txt>;
//! the repo fetches a verbatim copy to
//! `crates/domains/data/adobe/glyphlist.txt` (fetch-only) and ships
//! only the committed content-addressed `glyphlist.prx`. ISO
//! 32000-2:2020 §9.6.5.4 cites the AGL as the resolver for
//! `/Differences` glyph names.
//!
//! ## File format
//!
//! Lines beginning with `#` are comments. Data lines are
//! `<glyphname>;<hex unicode codepoint(s)>`. Multi-codepoint
//! mappings (e.g. `ffi;0066 0066 0069`) flatten to the first
//! codepoint — body-text PDFs use single-codepoint glyph names
//! exclusively.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use std::collections::HashMap;
use std::sync::OnceLock;

/// Content address (BLAKE3) of the loaded glyph-list bytes, pinned by an axiom
/// in `mod.rs` and verified at test time. Updating the file requires recomputing
/// this address.
pub const PINNED_ADDRESS: &str = "5a200d1e890dce2c1ce30e8063f241eea96c4ffadab39eb01259cb927bb1b67f";

/// The committed Adobe Glyph List `.prx` — the content-addressed envelope
/// carrying the `name;HEX[ HEX]*` table bytes. The raw `.txt` is fetch-only
/// (`pr4xis update`) and ships in NO crate; only this `.prx` is committed +
/// embedded. Loaded through the generalized raw-source gate (phase 2).
const GLYPH_LIST_PRX: &[u8] = include_bytes!("../../../../../data/adobe/glyphlist.prx");

/// The loaded Adobe Glyph List bytes, materialized from the committed `.prx`
/// through the fail-closed `[compact_archive_signatures]` content gate, cached
/// for the process behind a `OnceLock`. The raw `.txt` is no longer embedded —
/// only the gated `.prx` is. The function-form successor of the former
/// `GLYPH_LIST_BYTES` const.
#[must_use]
pub fn glyph_list_bytes() -> &'static str {
    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    static BYTES: OnceLock<&'static str> = OnceLock::new();
    BYTES.get_or_init(|| raw_source_text_embedded("adobe_glyph_list", "2019", GLYPH_LIST_PRX))
}

/// Parse the AGL once on first call and cache the name → Unicode
/// codepoint map. Multi-codepoint mappings flatten to the first
/// codepoint per the module-level rationale.
pub fn glyph_name_table() -> &'static HashMap<&'static str, u16> {
    static TABLE: OnceLock<HashMap<&'static str, u16>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut map = HashMap::with_capacity(4500);
        for line in glyph_list_bytes().lines() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis_runtime::address::ContentAddress;

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
    fn table_has_full_agl_population() {
        let table = glyph_name_table();
        assert!(
            table.len() > 4000,
            "AGL parse produced only {} entries; expected >4000",
            table.len()
        );
    }

    #[test]
    fn embedded_address_matches_pinned_hash() {
        let hex = ContentAddress::of(glyph_list_bytes().as_bytes()).to_hex();
        assert_eq!(hex, PINNED_ADDRESS);
    }
}

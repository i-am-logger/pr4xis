//! Bundled MODS 3.8 XSD bytes + cached accessor.
//!
//! Mirrors `social::software::markup::xml::lmf::dtd` — the bundled
//! schema bytes for the `mods_3_8@2018` source registered in
//! `praxis.toml`; the pinned digest in `praxis.lock` certifies these
//! bytes. Downstream code (case-law structural-extraction) reads the
//! XSD via [`loaded_mods_3_8`] and parses it via the existing
//! [`XsdSchemaLens`](crate::formal::meta::xsd::lens::XsdSchemaLens)
//! to project GovInfo MODS metadata XML into typed
//! [`XsdOntologyInstance`](crate::formal::meta::xsd::XsdOntologyInstance)
//! values.
//!
//! ## Citation
//!
//! - **Library of Congress, Network Development and MARC Standards
//!   Office** (2018) *MODS XML Schema Version 3.8*, the published XSD
//!   at <https://www.loc.gov/standards/mods/v3/mods-3-8.xsd>. The
//!   bundled file is a byte-for-byte copy; the `mods_3_8@2018` hash
//!   in `praxis.lock` is the content digest of those bytes.

/// The committed MODS 3.8 `.prx` — the content-addressed envelope carrying the
/// LC-published schema bytes. The raw `.xsd` is fetch-only (`pr4xis update`) and
/// ships in NO crate; only this `.prx` is committed + embedded. Loaded through
/// the generalized raw-source gate (phase 2), the byte-stream sibling of OLiA's
/// embedded committed `.prx.gz`.
const MODS_3_8_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/markup-schemas/mods/mods_3_8-2018.prx"
));

/// The loaded MODS 3.8 XSD — the schema GovInfo packages
/// (USREP / SCOTUS-slip / USCOURTS) declare against in their
/// per-package `mods.xml`. Downstream code queries this to anchor
/// MODS concept identity in the published XSD rather than in
/// hand-coded runtime types (`feedback_bottom_up_loaded_not_encoded`).
///
/// The bytes are materialized from the committed `.prx` through the fail-closed
/// `[compact_archive_signatures]` content gate
/// ([`raw_source_text_embedded`](crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded)),
/// cached for the process behind a `OnceLock`. The raw `.xsd` is no longer
/// embedded — only the gated `.prx` is.
#[must_use]
pub fn loaded_mods_3_8() -> &'static str {
    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    use std::sync::OnceLock;
    // The accessor returns a `Cow` (owned when the payload rides DEFLATE); the
    // `OnceLock` caches the one materialization for the process.
    static XSD: OnceLock<alloc::borrow::Cow<'static, str>> = OnceLock::new();
    XSD.get_or_init(|| raw_source_text_embedded("mods_3_8", "2018", MODS_3_8_PRX))
        .as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn loaded_mods_returns_published_xsd_root() {
        let bytes = loaded_mods_3_8();
        // The LC-published XSD opens with the XML declaration followed
        // by the LC editor comment block; assert the schema root is
        // present rather than parsing the whole document (parsing is
        // exercised by the case-law pipeline tests downstream).
        assert!(
            bytes.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
            "MODS 3.8 XSD must begin with the W3C XML 1.0 §2.8 declaration"
        );
        assert!(
            bytes.contains("targetNamespace=\"http://www.loc.gov/mods/v3\""),
            "MODS 3.8 XSD must declare the LC v3 target namespace"
        );
        assert!(
            bytes.contains("<xs:schema"),
            "MODS 3.8 XSD must have an <xs:schema> root element"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mods_bytes_match_lock_hash() {
        use pr4xis_runtime::address::ContentAddress;
        let bytes = loaded_mods_3_8();
        let hex = ContentAddress::of(bytes.as_bytes()).to_hex();
        // `mods_3_8@2018` pinned digest from praxis.lock.
        assert_eq!(
            hex, "f10d8db0297932a3b1ab37318aa3113b0bcc5c72dfc8b379e6c2e27854d21497",
            "loaded MODS 3.8 XSD bytes must match the praxis.lock pinned digest"
        );
    }
}

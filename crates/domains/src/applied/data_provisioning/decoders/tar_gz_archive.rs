//! Gzip-compressed tar archive decoder.
//!
//! Identifies a byte stream as a gzip-compressed tar archive by
//! checking the gzip magic number (RFC 1952 §2.3.1: every gzip member
//! begins with the two-byte magic `1f 8b`). Praxis intentionally does
//! not pull `flate2` / `tar` into `pr4xis-domains`' runtime
//! dependency surface; decompression + per-entry walks are done by
//! the consumer (e.g. [`crate::formal::meta::xsd::xsts_audit`] reads
//! the pre-extracted tree).
//!
//! This minimal decoder is what the `DecoderTotalityPerKind` axiom
//! needs: every leaf [`crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept`]
//! shipping as [`super::super::ontology::ContentType::TarGzArchive`]
//! has a registered byte-level identifier.
//!
//! ## Citation
//!
//! - **Deutsch, P.** (1996) *GZIP file format specification version
//!   4.3*, IETF RFC 1952. §2.3.1 defines the `1f 8b` magic.

/// True iff `bytes` begin with the RFC 1952 gzip magic number
/// (`1f 8b`).
#[must_use]
pub fn is_gzip(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_gzip_magic() {
        assert!(is_gzip(&[0x1f, 0x8b, 0x08, 0x00]));
    }

    #[test]
    fn rejects_non_gzip() {
        assert!(!is_gzip(b"<xml/>"));
        assert!(!is_gzip(&[]));
        assert!(!is_gzip(&[0x1f]));
    }
}

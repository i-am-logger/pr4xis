//! PKZIP archive decoder.
//!
//! Identifies a byte stream as a PKZIP archive by checking the
//! local-file-header signature `0x04034b50` (PKWARE APPNOTE.TXT
//! 6.3.10 §4.3.7, 2022). Same role as the gzip / DTD identifiers
//! in this `decoders/` tree — magic-prefix recognition.
//!
//! Single-member extraction lives in the `fetch` feature's
//! downloader (`fetch::unzip_single_xml`): it reads the central
//! directory (§4.3.12) for authoritative sizes and inflates DEFLATE
//! members (RFC 1951) with `flate2` — the same compression crate the
//! gzip path uses, so no dedicated ZIP-decompressor crate enters the
//! dependency surface. That covers single-document archives such as
//! the USC release-point title `.zip`s. OOXML schema archives
//! (ECMA-376 5th ed. §11.3) ship as ZIP too; their multi-file
//! per-XSD extraction remains a consumer concern.
//!
//! ## Citation
//!
//! - **PKWARE Inc.** (2022) *APPNOTE.TXT — .ZIP File Format
//!   Specification*, version 6.3.10. §4.3.7 defines the local-file
//!   header signature as the little-endian 32-bit value
//!   `0x04034b50`.
//! - **ISO/IEC 21320-1:2015** *Document container file — Part 1:
//!   Core* — the OOXML / EPUB / OPC subset of PKZIP.

/// The [`ContentType`](crate::applied::data_provisioning::ontology::ContentType)
/// this module realizes -- the single declaration of which content type
/// this file decodes, read by `super::has_decoder_for` (audit 2026-06-12 D-22).
pub const DECODES: crate::applied::data_provisioning::ontology::ContentType =
    crate::applied::data_provisioning::ontology::ContentType::ZipArchive;

/// True iff `bytes` begin with the PKWARE APPNOTE.TXT §4.3.7
/// local-file-header signature `0x04034b50` (little-endian:
/// bytes `50 4B 03 04`).
#[must_use]
pub fn is_zip(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[0] == 0x50 && bytes[1] == 0x4B && bytes[2] == 0x03 && bytes[3] == 0x04
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_pkzip_magic() {
        assert!(is_zip(&[0x50, 0x4B, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00]));
    }

    #[test]
    fn rejects_non_zip() {
        assert!(!is_zip(b"<xml/>"));
        assert!(!is_zip(&[0x1f, 0x8b])); // gzip magic, not zip
        assert!(!is_zip(&[]));
        assert!(!is_zip(&[0x50, 0x4B, 0x03])); // truncated
    }

    #[test]
    fn recognises_bundled_ooxml_schema_archive() {
        // The bundled OOXML schema archive is a real PKZIP file; the magic-prefix
        // check returns true on its first bytes. The raw `.zip` is fetch-only and
        // ships in NO crate — the bytes are materialized from the committed `.prx`
        // through the fail-closed `[compact_archive_signatures]` gate (phase 2d).
        use crate::applied::data_provisioning::raw_source_prx::raw_source_bytes_embedded;
        const OOXML_PRX: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/markup-schemas/ooxml/ooxml_schema_strict-2016.prx"
        ));
        let bytes = raw_source_bytes_embedded("ooxml_schema_strict", "2016", OOXML_PRX);
        assert!(
            is_zip(&bytes),
            "bundled OOXML schema archive must be a valid PKZIP file"
        );
    }
}

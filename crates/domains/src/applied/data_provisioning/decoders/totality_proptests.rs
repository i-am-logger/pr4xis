//! Honest, made structural at the byte boundary.
//!
//! Each decoder is the `RawBytes → DomainOntology` entry point for untrusted
//! external input. The Honest guarantee at this boundary is **totality**: for
//! *any* bytes whatsoever the decoder returns `Ok` or `Err`, but never panics.
//! A panic on malformed input is confabulation-via-crash — the system failing
//! to refuse cleanly.
//!
//! These are the universal (∀ bytes) refutation of that failure mode: proptest
//! feeds each decoder thousands of arbitrary byte strings; if any input makes a
//! decoder panic (an `unwrap`/`expect`/slice-out-of-bounds on adversarial
//! bytes), proptest shrinks it to the minimal witness and fails. This is the
//! Honest value's by-construction backing for decoding — not "we tested a few
//! malformed cases", but "the decoder is total over the whole input space".
//!
//! Tagged via `register_praxis_value!` because the `proptest!` macro generates
//! the test functions (the `#[praxis_value]` attribute cannot wrap them).

#![cfg(test)]

use proptest::prelude::*;

use super::super::{raw_source_prx, registry_prx};
use super::{
    adobe_glyph_list, owl, plaintext_tsv, tar_gz_archive, theme_collection, xhtml, xml_dtd,
    xml_lmf, xml_xsd, zip_archive,
};

proptest! {
    #[test]
    fn prop_owl_decode_is_total(bytes in any::<Vec<u8>>()) {
        let _ = owl::decode(&bytes);
    }

    #[test]
    fn prop_plaintext_tsv_decode_is_total(bytes in any::<Vec<u8>>()) {
        let _ = plaintext_tsv::decode(&bytes);
    }

    #[test]
    fn prop_theme_collection_decode_is_total(bytes in any::<Vec<u8>>()) {
        let _ = theme_collection::decode(&bytes);
    }

    #[test]
    fn prop_xhtml_decode_is_total(bytes in any::<Vec<u8>>()) {
        let _ = xhtml::decode(&bytes);
    }

    #[test]
    fn prop_xml_lmf_decode_is_total(bytes in any::<Vec<u8>>()) {
        let _ = xml_lmf::decode(&bytes);
    }

    #[test]
    fn prop_xml_xsd_decode_is_total(bytes in any::<Vec<u8>>()) {
        let _ = xml_xsd::decode(&bytes);
    }

    // Content-type recognizers are byte boundaries too: ∀ bytes they must
    // return a bool, never panic (a recognizer that crashes on garbage instead
    // of declining to match is an Honest violation).
    #[test]
    fn prop_is_gzip_is_total(bytes in any::<Vec<u8>>()) {
        let _ = tar_gz_archive::is_gzip(&bytes);
    }

    #[test]
    fn prop_is_zip_is_total(bytes in any::<Vec<u8>>()) {
        let _ = zip_archive::is_zip(&bytes);
    }

    #[test]
    fn prop_is_dtd_is_total(bytes in any::<Vec<u8>>()) {
        let _ = xml_dtd::is_dtd(&bytes);
    }

    // Lenient line-parsers (the UTF-8 boundary is upstream): ∀ string they must
    // return a value, never panic — they drop malformed lines, not crash.
    #[test]
    fn prop_adobe_glyph_parse_is_total(s in any::<String>()) {
        let _ = adobe_glyph_list::parse(&s);
    }

    #[test]
    fn prop_plaintext_tsv_parse_is_total(s in any::<String>()) {
        let _ = plaintext_tsv::parse(&s);
    }

    // The `.prx` succinct envelope decoders (varint-length-prefixed blobs) are
    // untrusted-input boundaries too — a forged length prefix must be refused,
    // never slice past the buffer or over-allocate.
    #[test]
    fn prop_raw_source_prx_decode_is_total(bytes in any::<Vec<u8>>()) {
        let _ = raw_source_prx::decode_raw_source(&bytes);
    }

    // COMPRESSED-envelope totality: arbitrary corruption of a REAL DEFLATE
    // raw-source envelope (a mutated byte anywhere — frame, declared length,
    // or the RFC 1951 stream itself) must decode to Ok/Err, never panic and
    // never a runaway allocation (the declared-length bomb guard bounds the
    // inflater). The payload is a LOW-ENTROPY repeated pattern: uniform-random
    // bytes are incompressible, so the encoder's store-if-smaller would
    // downgrade them to Identity and the Deflate arm would never actually run
    // (the plain ∀-bytes property above almost never reaches a valid DEFLATE
    // payload by chance either). The size assertion pins that the envelope
    // genuinely carries a compressed payload — an Identity envelope can never
    // be smaller than the blob it wraps.
    #[test]
    fn prop_raw_source_prx_decode_is_total_over_corrupted_deflate(
        pattern in proptest::collection::vec(any::<u8>(), 1..16),
        reps in 32usize..128,
        byte_idx in any::<prop::sample::Index>(),
        xor in any::<u8>(),
    ) {
        let blob: Vec<u8> = pattern
            .iter()
            .cycle()
            .take(pattern.len() * reps)
            .copied()
            .collect();
        let prx = raw_source_prx::encode_raw_source(
            "widget",
            "1",
            &blob,
            raw_source_prx::PayloadEncoding::Deflate,
        );
        // The Deflate arm must actually be live for this property to test it:
        // a repeated pattern compresses far below its own length, while an
        // Identity envelope is always ≥ payload + header.
        prop_assert!(
            prx.len() < blob.len(),
            "envelope not compressed ({} >= {}): the Deflate arm was downgraded",
            prx.len(),
            blob.len()
        );
        let mut bytes = prx;
        let i = byte_idx.index(bytes.len());
        bytes[i] ^= xor; // xor may be 0: the uncorrupted envelope must be Ok
        let _ = raw_source_prx::decode_raw_source(&bytes);
    }

    #[test]
    fn prop_registry_prx_decode_is_total(bytes in any::<Vec<u8>>()) {
        let _ = registry_prx::decode_registry(&bytes);
    }
}

pr4xis::register_praxis_value!(prop_owl_decode_is_total, Honest);
pr4xis::register_praxis_value!(prop_plaintext_tsv_decode_is_total, Honest);
pr4xis::register_praxis_value!(prop_theme_collection_decode_is_total, Honest);
pr4xis::register_praxis_value!(prop_xhtml_decode_is_total, Honest);
pr4xis::register_praxis_value!(prop_xml_lmf_decode_is_total, Honest);
pr4xis::register_praxis_value!(prop_xml_xsd_decode_is_total, Honest);
pr4xis::register_praxis_value!(prop_is_gzip_is_total, Honest);
pr4xis::register_praxis_value!(prop_is_zip_is_total, Honest);
pr4xis::register_praxis_value!(prop_is_dtd_is_total, Honest);
pr4xis::register_praxis_value!(prop_adobe_glyph_parse_is_total, Honest);
pr4xis::register_praxis_value!(prop_plaintext_tsv_parse_is_total, Honest);
pr4xis::register_praxis_value!(prop_raw_source_prx_decode_is_total, Honest);
pr4xis::register_praxis_value!(
    prop_raw_source_prx_decode_is_total_over_corrupted_deflate,
    Honest
);
pr4xis::register_praxis_value!(prop_registry_prx_decode_is_total, Honest);

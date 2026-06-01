//! Runnable axioms of the [`OntologyArchiveStorage`](super::ontology)
//! ontology — each `verify()` is a *predicate that runs*, exercising the
//! real `.prx` realisation (`xml::owl::prx`) rather than asserting a
//! doc-comment (M4.ι.0 / task #175; `feedback_praxis_as_compiler_self_describing`).
//!
//! Gated on `feature = "prx"` because the realisation these predicates
//! verify lives there. When USC (#271) becomes a second consumer the
//! same axioms verify against it too.
//!
//! Deferred, deliberately NOT declared here (no machinery yet — a
//! passing `verify()` would be a stub / over-claim): `AttestationChainVerifiable`
//! (TUF / in-toto / SLSA) and `IntegrityClaimVerifiable` (W3C SRI). They
//! are concepts in the ontology; their axioms land with the supply-chain
//! attestation work.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use crate::social::software::markup::xml::owl::prx::{
    OwnedCodegenData, PrxEnvelope, PrxMetadata, PrxMode, RawSource, envelope_from_bytes,
    envelope_to_bytes, gunzip, gzip, load_prx_gz, reconstruct_source, source_content_hash,
};

/// Distinct witness byte-strings the byte-level axioms run over —
/// empty, ASCII, binary (incl. 0x00 / 0xFF), and a longer repeated run.
fn witness_sources() -> Vec<Vec<u8>> {
    alloc::vec![
        Vec::new(),
        b"praxis ontology archive".to_vec(),
        alloc::vec![0u8, 1, 2, 255, 254, 0, 13, 10],
        b"the quick brown fox".repeat(64),
    ]
}

/// A minimal `BytesPlusView` envelope carrying `source` content-addressed
/// — built directly (no `build_envelope`, which is codegen-gated) so the
/// axioms run under `feature = "prx"` alone.
fn witness_envelope(name: &str, source: &[u8]) -> PrxEnvelope {
    let hash = source_content_hash(source);
    PrxEnvelope {
        metadata: PrxMetadata {
            name: name.to_string(),
            version: "1".to_string(),
            ontology_uri: String::new(),
            source_url: String::new(),
            source_sha256: hash.clone(),
            number_of_classes: 0,
            number_of_properties: 0,
        },
        data: OwnedCodegenData {
            entity_count: 0,
            entity_ids: Vec::new(),
            entity_kind: Vec::new(),
            entity_labels: Vec::new(),
            entity_defs: Vec::new(),
            word_index: Vec::new(),
            taxonomy: Vec::new(),
            mereology: Vec::new(),
            opposition: Vec::new(),
            equivalence: Vec::new(),
            causation: Vec::new(),
            references: Vec::new(),
        },
        mode: PrxMode::BytesPlusView,
        raw: Some(RawSource {
            content_address: hash,
            blob: source.to_vec(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Merkle layer.
// ---------------------------------------------------------------------------

/// The content address of a node is the SHA-256 of its bytes — a
/// deterministic, spec-defined function. Checked by known-answer test
/// against the NIST FIPS 180-4 SHA-256 example vectors: a degenerate,
/// truncating, or drifted hash is falsified here. Merkle (1987); NIST
/// FIPS 180-4 §6.2.
pub struct MerkleHashDeterministic;

impl Axiom for MerkleHashDeterministic {
    fn verify(&self) -> Verdict {
        // Known-answer test (NIST FIPS 180-4 SHA-256 example vectors): the
        // content address must equal the exact, spec-defined SHA-256 hex of
        // the bytes. A constant/degenerate digest, a hex-formatting bug, or
        // a non-SHA-256 hash is falsified here (this is not `h(x) == h(x)`).
        let vectors: &[(&[u8], &str)] = &[
            (
                b"",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                b"abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
        ];
        for (input, expected) in vectors {
            if source_content_hash(input) != *expected {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            }
        }
        Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MerkleHashDeterministic",
        "the content address is the spec-defined SHA-256 of the node bytes (known-answer)",
        "Merkle (1987) A Digital Signature Based on a Conventional Encryption Function, CRYPTO '87; NIST (2015) FIPS 180-4 §6.2"
    );
}

pr4xis::register_axiom!(
    MerkleHashDeterministic,
    "Merkle (1987) CRYPTO '87; NIST (2015) FIPS 180-4 §6.2"
);

/// Content-addressed dedup is correct: two nodes share an address iff
/// they share their bytes — identical content dedups, distinct content
/// does not collide (over the witness set). Benet (2014) IPFS/IPLD; Git
/// object dedup.
pub struct MerkleDedupCorrect;

impl Axiom for MerkleDedupCorrect {
    fn verify(&self) -> Verdict {
        let sources = witness_sources();
        for a in &sources {
            for b in &sources {
                let same_address = source_content_hash(a) == source_content_hash(b);
                let same_bytes = a == b;
                if same_address != same_bytes {
                    return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                        self.meta(),
                    )));
                }
            }
        }
        Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MerkleDedupCorrect",
        "nodes share a content address iff they share their bytes (dedup is sound)",
        "Benet (2014) IPFS: Content-Addressed, Versioned, P2P File System"
    );
}

pr4xis::register_axiom!(
    MerkleDedupCorrect,
    "Benet (2014) IPFS: Content-Addressed, Versioned, P2P File System"
);

// ---------------------------------------------------------------------------
// Envelope layer.
// ---------------------------------------------------------------------------

/// The compressed form is lossless: `gunzip(gzip(x)) == x`. Deutsch
/// (1996) RFC 1952.
pub struct CompressionRoundTrip;

impl Axiom for CompressionRoundTrip {
    fn verify(&self) -> Verdict {
        for source in &witness_sources() {
            let Ok(compressed) = gzip(source) else {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            };
            match gunzip(&compressed) {
                Ok(restored) if &restored == source => {}
                _ => {
                    return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                        self.meta(),
                    )));
                }
            }
        }
        Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "CompressionRoundTrip",
        "gzip is lossless: gunzip(gzip(x)) == x",
        "Deutsch (1996) GZIP file format specification version 4.3, RFC 1952"
    );
}

pr4xis::register_axiom!(
    CompressionRoundTrip,
    "Deutsch (1996) GZIP file format specification version 4.3, RFC 1952"
);

/// The rkyv serialization is value-deterministic: two independently built
/// EQUAL envelopes serialize to equal bytes (so the blob hash is a stable
/// content address), and two DIFFERENT envelopes serialize to different
/// bytes (so the serialization distinguishes content). A serializer that
/// leaked allocation order/addresses, or collapsed distinct values, is
/// falsified here. Hill rkyv v0.8.
pub struct RkyvDeterminism;

impl Axiom for RkyvDeterminism {
    fn verify(&self) -> Verdict {
        // Two INDEPENDENTLY constructed equal envelopes must serialize to
        // equal bytes (not the same object serialized twice), and a
        // DIFFERENT envelope must serialize to different bytes.
        let one = witness_envelope("rkyv-determinism", b"praxis archive bytes");
        let one_again = witness_envelope("rkyv-determinism", b"praxis archive bytes");
        let other = witness_envelope("rkyv-determinism", b"a different source");
        let (Ok(a), Ok(b), Ok(c)) = (
            envelope_to_bytes(&one),
            envelope_to_bytes(&one_again),
            envelope_to_bytes(&other),
        ) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        if a == b && a != c {
            Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )))
        }
    }

    pr4xis::axiom_meta!(
        "RkyvDeterminism",
        "equal envelopes serialize to equal bytes and distinct envelopes to distinct bytes",
        "Hill, D. — rkyv: zero-copy deserialization framework for Rust, v0.8"
    );
}

pr4xis::register_axiom!(
    RkyvDeterminism,
    "Hill, D. — rkyv: zero-copy deserialization framework for Rust, v0.8"
);

/// Emit/load is a well-behaved lens: deserializing a serialized envelope
/// reproduces it exactly (the GetPut leg, through the rkyv + gzip layers).
/// Foster, Greenwald, Moore, Pierce & Schmitt (2007) §2.2.
pub struct EmitLoadWellBehaved;

impl Axiom for EmitLoadWellBehaved {
    fn verify(&self) -> Verdict {
        let envelope = witness_envelope("emit-load", b"well-behaved lens witness");
        // rkyv leg
        let Ok(bytes) = envelope_to_bytes(&envelope) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        // gzip leg: gunzip(gzip(bytes)) must feed back the same rkyv bytes
        let Ok(gz) = gzip(&bytes) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        match gunzip(&gz) {
            Ok(back_bytes) if back_bytes == bytes => {}
            _ => {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            }
        }
        match envelope_from_bytes(&bytes) {
            Ok(back) if back == envelope => {
                Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta())))
            }
            _ => Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            ))),
        }
    }

    pr4xis::axiom_meta!(
        "EmitLoadWellBehaved",
        "deserialize(serialize(envelope)) == envelope through the rkyv + gzip layers",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(
    EmitLoadWellBehaved,
    "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2"
);

/// The source pin is faithful: reconstructing a `BytesPlusView`
/// envelope's source returns the exact bytes, and their hash equals the
/// recorded pin (the operator's "…→ .prx → xml, same byte hash"
/// invariant). NIST FIPS 180-4 §6.2; Dolstra (2006).
pub struct SourceHashFaithfulness;

impl Axiom for SourceHashFaithfulness {
    fn verify(&self) -> Verdict {
        for source in &witness_sources() {
            let envelope = witness_envelope("source-hash", source);
            let Ok(reconstructed) = reconstruct_source(&envelope) else {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            };
            if &reconstructed != source
                || source_content_hash(&reconstructed) != envelope.metadata.source_sha256
            {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            }
        }
        Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SourceHashFaithfulness",
        "reconstruct_source returns the exact source bytes whose hash equals the recorded pin",
        "NIST (2015) FIPS 180-4 §6.2; Dolstra (2006) The Purely Functional Software Deployment Model"
    );
}

pr4xis::register_axiom!(
    SourceHashFaithfulness,
    "NIST (2015) FIPS 180-4 §6.2; Dolstra (2006) The Purely Functional Software Deployment Model"
);

/// The load gate fails closed: an envelope whose source pin does not
/// match the trusted pin is rejected and nothing is installed.
pub struct LoadGateFailsClosed;

impl Axiom for LoadGateFailsClosed {
    fn verify(&self) -> Verdict {
        let envelope = witness_envelope("load-gate", b"gated source bytes");
        let Ok(rkyv_bytes) = envelope_to_bytes(&envelope) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        let Ok(prx_gz) = gzip(&rkyv_bytes) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        // A pin that is NOT the envelope's source hash must be rejected.
        let wrong_pin = "0".repeat(64);
        match load_prx_gz(&prx_gz, &wrong_pin) {
            Err(_) => Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta()))),
            Ok(_) => Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            ))),
        }
    }

    pr4xis::axiom_meta!(
        "LoadGateFailsClosed",
        "an archive whose source pin mismatches the trusted pin is rejected (nothing installed)",
        "Samuel et al. (2010) Survivable Key Compromise in Software Update Systems (TUF), CCS '10"
    );
}

pr4xis::register_axiom!(
    LoadGateFailsClosed,
    "Samuel et al. (2010) Survivable Key Compromise in Software Update Systems (TUF), CCS '10"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_archive_axioms_hold() {
        assert!(MerkleHashDeterministic.verify().is_ok());
        assert!(MerkleDedupCorrect.verify().is_ok());
        assert!(CompressionRoundTrip.verify().is_ok());
        assert!(RkyvDeterminism.verify().is_ok());
        assert!(EmitLoadWellBehaved.verify().is_ok());
        assert!(SourceHashFaithfulness.verify().is_ok());
        assert!(LoadGateFailsClosed.verify().is_ok());
    }

    #[test]
    fn source_hash_faithfulness_rejects_a_lying_envelope() {
        // A counterexample envelope whose raw blob disagrees with its pin
        // must make the faithfulness predicate fail — the axiom has teeth.
        let mut envelope = witness_envelope("liar", b"honest source");
        if let Some(raw) = envelope.raw.as_mut() {
            raw.blob.push(b'!'); // blob no longer hashes to content_address
        }
        assert!(
            reconstruct_source(&envelope).is_err(),
            "a tampered raw blob must be rejected (fail-closed)"
        );
    }
}

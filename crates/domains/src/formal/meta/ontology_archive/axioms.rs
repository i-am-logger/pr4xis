//! Runnable axioms of the [`OntologyArchiveStorage`](super::ontology)
//! ontology — each `verify()` is a *predicate that runs*, exercising the
//! real `.prx` realisation (`xml::owl::prx`) rather than asserting a
//! doc-comment (M4.ι.0 / task #175; `feedback_praxis_as_compiler_self_describing`).
//!
//! Gated on `feature = "prx"` because the realisation these predicates
//! verify lives there. When USC (#271) becomes a second consumer the
//! same axioms verify against it too.
//!
//! `IntegrityClaimVerifiable` (W3C SRI) IS realised below, over the
//! multi-algorithm `HashAlgorithm` claim (SHA-256/512, BLAKE3). Still
//! deferred, deliberately NOT declared here (no machinery yet — a passing
//! `verify()` would be a stub / over-claim): `AttestationChainVerifiable`
//! (TUF / in-toto / SLSA). It is a concept in the ontology; its axiom lands
//! with the supply-chain attestation work (which also needs an
//! authenticated, signed pin in place of the current unsigned `praxis.lock`).

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;
use pr4xis_runtime::address::ContentAddress;

use crate::applied::data_provisioning::registry::LockDigest;
use crate::formal::meta::well_behaved_lens::{RoundTripFidelity, WellBehavedLens};
use crate::social::software::markup::xml::owl::lens::OwlLens;
use crate::social::software::markup::xml::owl::prx::{
    OwnedCodegenData, PrxEnvelope, PrxError, PrxMetadata, RawSource, envelope_from_bytes,
    envelope_to_bytes, gunzip, gzip, load_prx_gz, reconstruct_source,
};
// The USC second consumer (#271): the SAME archive axioms verify against its
// aux-carrying envelope too. Imported up-layer here exactly as the OWL `prx`
// types are above — these are realisation witnesses, not a parallel axiom set.
use crate::social::software::markup::xml::uslm::corpus::prx::{
    OwnedUscSectionAux, OwnedUscSubdivision, UsCodePrxEnvelope, UscPrxMetadata, load_usc_prx_gz,
    usc_envelope_from_bytes, usc_envelope_to_bytes, usc_reconstruct_source,
};
// The WordNet / English consumer (Stage E0): the SAME archive axioms verify
// against its FLAT, word-index-carrying envelope too — same realisation
// pattern as the USC import above, no parallel axiom set.
use crate::social::software::markup::xml::lmf::prx::{
    WnPrxMetadata, WordNetPrxEnvelope, load_wordnet_prx_gz, wn_reconstruct_source,
    wordnet_envelope_from_bytes, wordnet_envelope_to_bytes,
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
    let hash = ContentAddress::of(source).to_hex();
    PrxEnvelope {
        metadata: PrxMetadata {
            name: name.to_string(),
            version: "1".to_string(),
            ontology_uri: String::new(),
            source_url: String::new(),
            source_address: hash.clone(),
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
        mode: RoundTripFidelity::RawBytesComplementFloor,
        graph: None,
        raw: Some(RawSource {
            content_address: hash,
            blob: source.to_vec(),
        }),
    }
}

/// A minimal USC `RawBytesComplementFloor` envelope carrying `source`
/// content-addressed, with a NON-TRIVIAL nested subdivision tree (`/a → /a/1 →
/// /a/1/A`) so the widened axioms genuinely exercise the aux-carrying path: a
/// dropped, reordered, or poisoned [`OwnedUscSubdivision`] is falsified.
///
/// Built directly — no `read_uslm_title` — so the axioms run under
/// `feature = "prx"` alone, exactly like [`witness_envelope`]. The USC second
/// consumer realises the SAME `OntologyArchiveStorage` axioms; this fixture is
/// the witness they run against.
fn witness_usc_envelope(name: &str, source: &[u8]) -> UsCodePrxEnvelope {
    let hash = ContentAddress::of(source).to_hex();
    let sub_a1a = OwnedUscSubdivision {
        urn: "/us/usc/t18/s1514A/a/1/A".to_string(),
        kind: "subparagraph".to_string(),
        num: "A".to_string(),
        heading: None,
        chapeau: None,
        content: Some("by reason of lawful acts".to_string()),
        children: Vec::new(),
    };
    let sub_a1 = OwnedUscSubdivision {
        urn: "/us/usc/t18/s1514A/a/1".to_string(),
        kind: "paragraph".to_string(),
        num: "1".to_string(),
        heading: None,
        chapeau: None,
        content: Some("No company may discriminate.".to_string()),
        children: alloc::vec![sub_a1a],
    };
    let sub_a = OwnedUscSubdivision {
        urn: "/us/usc/t18/s1514A/a".to_string(),
        kind: "subsection".to_string(),
        num: "a".to_string(),
        heading: None,
        chapeau: Some("In general—".to_string()),
        content: None,
        children: alloc::vec![sub_a1],
    };
    let aux = alloc::vec![OwnedUscSectionAux {
        urn: "/us/usc/t18/s1514A".to_string(),
        subdivisions: alloc::vec![sub_a],
        relations: alloc::vec![
            (
                "/us/usc/t18/s1514A/a".to_string(),
                "/us/usc/t18/s1514A".to_string()
            ),
            (
                "/us/usc/t18/s1514A/a/1".to_string(),
                "/us/usc/t18/s1514A/a".to_string()
            ),
            (
                "/us/usc/t18/s1514A/a/1/A".to_string(),
                "/us/usc/t18/s1514A/a/1".to_string()
            ),
        ],
    }];
    UsCodePrxEnvelope {
        metadata: UscPrxMetadata {
            name: name.to_string(),
            version: "1".to_string(),
            corpus_uri: String::new(),
            source_url: String::new(),
            source_address: hash.clone(),
            number_of_sections: 1,
            number_of_subdivisions: 3,
        },
        data: OwnedCodegenData {
            entity_count: 1,
            entity_ids: alloc::vec!["/us/usc/t18/s1514A".to_string()],
            entity_kind: alloc::vec!["section".to_string()],
            entity_labels: alloc::vec![
                "Civil action to protect against retaliation in fraud cases".to_string()
            ],
            entity_defs: alloc::vec![
                "In general— No company may discriminate. by reason of lawful acts".to_string()
            ],
            word_index: Vec::new(),
            taxonomy: Vec::new(),
            mereology: Vec::new(),
            opposition: Vec::new(),
            equivalence: Vec::new(),
            causation: Vec::new(),
            references: Vec::new(),
        },
        aux,
        mode: RoundTripFidelity::RawBytesComplementFloor,
        // This witness rides the universal floor (the raw blob); the
        // graph-faithful payload is absent in this tier.
        graph: None,
        raw: Some(RawSource {
            content_address: hash,
            blob: source.to_vec(),
        }),
        grounded: Vec::new(),
    }
}

/// A minimal WordNet `RawBytesComplementFloor` envelope carrying `source`
/// content-addressed, with a NON-TRIVIAL POPULATED `word_index` (two words,
/// one multi-sense) so the widened axioms genuinely exercise the
/// word-index-carrying path: a dropped, reordered, or poisoned `word_index`
/// entry is falsified.
///
/// Built directly — no `read_wordnet` — so the axioms run under
/// `feature = "prx"` alone, exactly like [`witness_envelope`] and
/// [`witness_usc_envelope`]. The WordNet consumer realises the SAME
/// `OntologyArchiveStorage` axioms; this fixture is the witness they run
/// against. The `word_index` is sorted (`"cat"` < `"dog"`) — the
/// `binary_search` + reproducible-`MerkleRoot` invariant the real
/// `wn_builder_to_owned` upholds.
fn witness_wordnet_envelope(name: &str, source: &[u8]) -> WordNetPrxEnvelope {
    let hash = ContentAddress::of(source).to_hex();
    WordNetPrxEnvelope {
        metadata: WnPrxMetadata {
            name: name.to_string(),
            version: "1".to_string(),
            lexicon_uri: String::new(),
            source_url: String::new(),
            source_address: hash.clone(),
            number_of_synsets: 2,
            number_of_senses: 2,
        },
        data: OwnedCodegenData {
            entity_count: 2,
            entity_ids: alloc::vec!["s-cat".to_string(), "s-dog".to_string()],
            entity_kind: alloc::vec!["n".to_string(), "n".to_string()],
            entity_labels: alloc::vec!["s-cat".to_string(), "s-dog".to_string()],
            entity_defs: alloc::vec![
                "a small feline".to_string(),
                "a domesticated canine".to_string()
            ],
            // Sorted by word; "cat" → synset 0, "dog" → synset 1.
            word_index: alloc::vec![
                ("cat".to_string(), alloc::vec![0u64]),
                ("dog".to_string(), alloc::vec![1u64]),
            ],
            taxonomy: Vec::new(),
            mereology: Vec::new(),
            opposition: Vec::new(),
            equivalence: Vec::new(),
            causation: Vec::new(),
            references: Vec::new(),
        },
        mode: RoundTripFidelity::RawBytesComplementFloor,
        // This witness exercises the SHARED archive axioms over the WordNet
        // envelope SHAPE at the universal FLOOR (stored raw complement), the
        // tier OWL + USC still ride — so `graph` is `None` and `raw` carries the
        // content-addressed source. SLICE 3b's emit path uses the graph-faithful
        // tier (`graph` populated, `raw = None`); both are valid envelope
        // configurations and the shared axioms hold over either.
        graph: None,
        raw: Some(RawSource {
            content_address: hash,
            blob: source.to_vec(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Merkle layer.
// ---------------------------------------------------------------------------

/// The content address of a node is the BLAKE3 hash of its bytes — a
/// deterministic, spec-defined function. Checked by known-answer test
/// against published BLAKE3 vectors: a degenerate, truncating, or
/// drifted hash is falsified here. Merkle (1987); Aumasson, O'Connor,
/// Neves & Wilcox-O'Hearn (2020).
pub struct MerkleHashDeterministic;

impl Axiom for MerkleHashDeterministic {
    fn verify(&self) -> Verdict {
        // Known-answer test: the content address must equal the exact,
        // spec-defined BLAKE3 hex of the bytes. A constant/degenerate
        // digest, a hex-formatting bug, or a non-BLAKE3 hash is falsified
        // here (this is not `h(x) == h(x)`). Empty-input vector from the
        // BLAKE3 team's published test_vectors.json (input_len = 0); the
        // "abc" vector cross-derived with the reference `b3sum` binary.
        let vectors: &[(&[u8], &str)] = &[
            (
                b"",
                "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
            ),
            (
                b"abc",
                "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85",
            ),
        ];
        for (input, expected) in vectors {
            if ContentAddress::of(input).to_hex() != *expected {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            }
        }
        Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MerkleHashDeterministic",
        "the content address is the spec-defined BLAKE3 hash of the node bytes (known-answer)",
        "Merkle (1987) A Digital Signature Based on a Conventional Encryption Function, CRYPTO '87; Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020) BLAKE3: one function, fast everywhere"
    );
}

pr4xis::register_axiom!(
    MerkleHashDeterministic,
    "Merkle (1987) CRYPTO '87; Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020) BLAKE3"
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
                let same_address = ContentAddress::of(a) == ContentAddress::of(b);
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
            Ok(back) if back == envelope => {}
            _ => {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            }
        }
        // USC second consumer (#271): the same GetPut lens law over the
        // aux-carrying envelope. The round-tripped envelope must equal the
        // original INCLUDING the full recursive aux tree — so a dropped or
        // reordered `OwnedUscSubdivision` is falsified here.
        let usc = witness_usc_envelope("emit-load-usc", b"usc well-behaved lens witness");
        let Ok(usc_bytes) = usc_envelope_to_bytes(&usc) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        let Ok(usc_gz) = gzip(&usc_bytes) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        match gunzip(&usc_gz) {
            Ok(b) if b == usc_bytes => {}
            _ => {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            }
        }
        match usc_envelope_from_bytes(&usc_bytes) {
            Ok(back) if back == usc => {}
            _ => {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            }
        }
        // WordNet consumer (Stage E0): the same GetPut lens law over the FLAT,
        // word-index-carrying envelope. The round-tripped envelope must equal
        // the original INCLUDING the full `word_index` — so a dropped or
        // reordered entry is falsified here.
        let wn = witness_wordnet_envelope("emit-load-wn", b"wordnet well-behaved lens witness");
        let Ok(wn_bytes) = wordnet_envelope_to_bytes(&wn) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        let Ok(wn_gz) = gzip(&wn_bytes) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        match gunzip(&wn_gz) {
            Ok(b) if b == wn_bytes => {}
            _ => {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            }
        }
        match wordnet_envelope_from_bytes(&wn_bytes) {
            Ok(back) if back == wn => Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta()))),
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
/// invariant). Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020);
/// Dolstra (2006).
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
                || ContentAddress::of(&reconstructed).to_hex() != envelope.metadata.source_address
            {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            }
        }
        // USC second consumer (#271): the same source-faithfulness over the
        // aux-carrying envelope — `usc_reconstruct_source` returns the exact
        // bytes whose hash equals the recorded pin.
        for source in &witness_sources() {
            let envelope = witness_usc_envelope("source-hash-usc", source);
            let Ok(reconstructed) = usc_reconstruct_source(&envelope) else {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            };
            if &reconstructed != source
                || ContentAddress::of(&reconstructed).to_hex() != envelope.metadata.source_address
            {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            }
        }
        // WordNet consumer (Stage E0): the same source-faithfulness over the
        // word-index-carrying envelope — `wn_reconstruct_source` returns the
        // exact bytes whose hash equals the recorded pin.
        for source in &witness_sources() {
            let envelope = witness_wordnet_envelope("source-hash-wn", source);
            let Ok(reconstructed) = wn_reconstruct_source(&envelope) else {
                return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                    self.meta(),
                )));
            };
            if &reconstructed != source
                || ContentAddress::of(&reconstructed).to_hex() != envelope.metadata.source_address
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
        "Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020) BLAKE3: one function, fast everywhere; Dolstra (2006) The Purely Functional Software Deployment Model"
    );
}

pr4xis::register_axiom!(
    SourceHashFaithfulness,
    "Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020) BLAKE3; Dolstra (2006) The Purely Functional Software Deployment Model"
);

/// The load gate is a fail-closed content-address verification: it admits
/// an archive only when the `MerkleRoot` re-derived from the node's own
/// bytes equals the trusted pin. It refuses (a) a wrong pin and (b) a
/// poisoned `data` column carried under a genuine source label — the latter
/// only fails because the gate binds the *installed node*, not a label. The
/// accept-on-match leg is proven too, so a degenerate reject-everything gate
/// is also falsified. Samuel et al. (2010) TUF; W3C (2016) SRI.
pub struct LoadGateFailsClosed;

impl Axiom for LoadGateFailsClosed {
    fn verify(&self) -> Verdict {
        // A valid RDF/XML witness source: the OWL load gate's third leg
        // (RDFC-1.0 graph identity) re-derives the canonical N-Quads from
        // the carried source, so it must parse. A minimal one-class graph
        // suffices.
        let gated_source = br#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns="http://ex.org/gate#">
  <owl:Class rdf:about="http://ex.org/gate#Gated"/>
</rdf:RDF>
"#;
        let envelope = witness_envelope("load-gate", gated_source);
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
        // The genuine MerkleRoot (re-derived from the node's own bytes), the
        // genuine SourcePin, and the genuine RDFC-1.0 graph-identity pin.
        let archive_pin = LockDigest::address(ContentAddress::of(&rkyv_bytes).to_hex());
        let source_pin = LockDigest::address(envelope.metadata.source_address.clone());
        let Ok(canonical_sig) = OwlLens::signature(gated_source) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        let canonical_pin = LockDigest::address(
            canonical_sig
                .iter()
                .map(|b| alloc::format!("{b:02x}"))
                .collect::<String>(),
        );

        // Accept-on-match: genuine pins admit the archive. Without this leg a
        // degenerate gate that rejects EVERYTHING would satisfy the axiom.
        if load_prx_gz(&prx_gz, &archive_pin, &source_pin, &canonical_pin).is_err() {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        }
        // Reject a wrong MerkleRoot pin: a label that does not match the
        // content address re-derived from the bytes is refused (specifically
        // by the MerkleRoot leg — a HashMismatch, not just any error).
        let wrong_pin = LockDigest::address("0".repeat(64));
        if !matches!(
            load_prx_gz(&prx_gz, &wrong_pin, &source_pin, &canonical_pin),
            Err(PrxError::HashMismatch { .. })
        ) {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        }
        // Reject a wrong RDFC-1.0 canonical pin: a graph-identity label that
        // does not match the canonical N-Quads of the loaded graph is refused
        // by the new third leg (also a HashMismatch).
        if !matches!(
            load_prx_gz(&prx_gz, &archive_pin, &source_pin, &wrong_pin),
            Err(PrxError::HashMismatch { .. })
        ) {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        }
        // Teeth — poisoned `data` under an honest source label and raw leaf:
        // the MerkleRoot binds the installed node, so a poisoned column
        // changes the content address and is refused even with the genuine
        // source pin (the attack a label-only gate would admit).
        let mut poisoned = witness_envelope("load-gate", gated_source);
        poisoned.data.entity_count += 1;
        let Ok(poisoned_bytes) = envelope_to_bytes(&poisoned) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        let Ok(poisoned_gz) = gzip(&poisoned_bytes) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        if !matches!(
            load_prx_gz(&poisoned_gz, &archive_pin, &source_pin, &canonical_pin),
            Err(PrxError::HashMismatch { .. })
        ) {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        }
        // USC second consumer (#271): the same fail-closed gate over the
        // aux-carrying envelope, with the teeth on the RECURSIVE aux tree —
        // poisoning one `OwnedUscSubdivision.num` under a genuine source label
        // changes the MerkleRoot and is refused.
        let usc = witness_usc_envelope("load-gate-usc", b"usc gated source bytes");
        let Ok(usc_bytes) = usc_envelope_to_bytes(&usc) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        let Ok(usc_gz) = gzip(&usc_bytes) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        let usc_archive_pin = LockDigest::address(ContentAddress::of(&usc_bytes).to_hex());
        let usc_source_pin = LockDigest::address(usc.metadata.source_address.clone());
        // Accept-on-match.
        if load_usc_prx_gz(&usc_gz, &usc_archive_pin, &usc_source_pin).is_err() {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        }
        // Wrong MerkleRoot pin → HashMismatch.
        if !matches!(
            load_usc_prx_gz(&usc_gz, &wrong_pin, &usc_source_pin),
            Err(PrxError::HashMismatch { .. })
        ) {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        }
        // Teeth: poison one subdivision `num` under the honest source label +
        // raw leaf — the MerkleRoot binds the recursive aux tree, so it is
        // refused even with the genuine source pin.
        let mut usc_poisoned = witness_usc_envelope("load-gate-usc", b"usc gated source bytes");
        usc_poisoned.aux[0].subdivisions[0].num = "POISON".to_string();
        let Ok(usc_poisoned_bytes) = usc_envelope_to_bytes(&usc_poisoned) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        let Ok(usc_poisoned_gz) = gzip(&usc_poisoned_bytes) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        if !matches!(
            load_usc_prx_gz(&usc_poisoned_gz, &usc_archive_pin, &usc_source_pin),
            Err(PrxError::HashMismatch { .. })
        ) {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        }
        // WordNet consumer (Stage E0): the same fail-closed gate over the FLAT,
        // word-index-carrying envelope, with the teeth on the WORD INDEX —
        // poisoning one `word_index` entry's word under a genuine source label
        // changes the MerkleRoot and is refused even with the genuine source
        // pin (the attack a label-only gate would admit).
        let wn = witness_wordnet_envelope("load-gate-wn", b"wn gated source bytes");
        let Ok(wn_bytes) = wordnet_envelope_to_bytes(&wn) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        let Ok(wn_gz) = gzip(&wn_bytes) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        let wn_archive_pin = LockDigest::address(ContentAddress::of(&wn_bytes).to_hex());
        let wn_source_pin = LockDigest::address(wn.metadata.source_address.clone());
        // Accept-on-match.
        if load_wordnet_prx_gz(&wn_gz, &wn_archive_pin, &wn_source_pin).is_err() {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        }
        // Wrong MerkleRoot pin → HashMismatch.
        if !matches!(
            load_wordnet_prx_gz(&wn_gz, &wrong_pin, &wn_source_pin),
            Err(PrxError::HashMismatch { .. })
        ) {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        }
        // Teeth: poison one `word_index` entry's word under the honest source
        // label + raw leaf — the MerkleRoot binds the whole installed node
        // (including the word index), so it is refused even with the genuine
        // source pin.
        let mut wn_poisoned = witness_wordnet_envelope("load-gate-wn", b"wn gated source bytes");
        wn_poisoned.data.word_index[0].0 = "POISON".to_string();
        let Ok(wn_poisoned_bytes) = wordnet_envelope_to_bytes(&wn_poisoned) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        let Ok(wn_poisoned_gz) = gzip(&wn_poisoned_bytes) else {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        };
        if !matches!(
            load_wordnet_prx_gz(&wn_poisoned_gz, &wn_archive_pin, &wn_source_pin),
            Err(PrxError::HashMismatch { .. })
        ) {
            return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )));
        }
        Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "LoadGateFailsClosed",
        "the load gate admits an archive iff the MerkleRoot re-derived from its bytes matches the trusted pin; a wrong pin and a poisoned-data/honest-label archive are both refused",
        "Samuel et al. (2010) Survivable Key Compromise in Software Update Systems (TUF), CCS '10; W3C (2016) Subresource Integrity"
    );
}

pr4xis::register_axiom!(
    LoadGateFailsClosed,
    "Samuel et al. (2010) Survivable Key Compromise in Software Update Systems (TUF), CCS '10"
);

/// W3C Subresource Integrity: an `IntegrityClaim` binds a resource to its
/// expected content digest under a *named* hash algorithm, and a verifier
/// admits the resource only when the recomputed digest matches. Realised
/// over praxis's multi-algorithm `HashAlgorithm` claim — SHA-256 / SHA-512
/// (FIPS 180-4) and BLAKE3 (Aumasson et al. 2020). (SRI itself registers
/// only SHA-256/384/512; BLAKE3 extends the same expectation-bound-integrity
/// *pattern* to a stronger function — it is not an SRI-registered digest.)
/// Weak functions (MD5 / SHA-1) are *unrepresentable* in the enum, so
/// "refuse weak algorithms" is a type invariant rather than a runtime branch. Falsifiable + non-vacuous:
/// for every algorithm and every witness source the TRUE digest verifies,
/// while tampered content AND a corrupted expected-digest are both rejected
/// (never silently `Verified`).
pub struct IntegrityClaimVerifiable;

impl Axiom for IntegrityClaimVerifiable {
    fn verify(&self) -> Verdict {
        use crate::formal::meta::artifact_identity::ontology::{
            ClaimData, HashAlgorithm, IdentityClaim, IdentityConcept, VerificationResult,
        };
        use crate::formal::meta::artifact_identity::schemes::raw_hash;

        for source in &witness_sources() {
            for algorithm in [
                HashAlgorithm::Sha256,
                HashAlgorithm::Sha512,
                HashAlgorithm::Blake3,
            ] {
                let expected = raw_hash::hash_hex(algorithm, source);
                let claim = IdentityClaim {
                    concept: IdentityConcept::RawHash,
                    data: ClaimData::HashAlgorithm {
                        algorithm,
                        digest_hex: expected.clone(),
                    },
                };
                // POSITIVE — the true digest of the very bytes verifies.
                if !matches!(
                    raw_hash::verify(&claim, source),
                    VerificationResult::Verified(_)
                ) {
                    return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                        self.meta(),
                    )));
                }
                // NON-VACUITY #1 — appending a byte changes the content, so the
                // claim must NOT verify against the tampered bytes.
                let mut tampered = source.clone();
                tampered.push(0xAB);
                if matches!(
                    raw_hash::verify(&claim, &tampered),
                    VerificationResult::Verified(_)
                ) {
                    return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                        self.meta(),
                    )));
                }
                // NON-VACUITY #2 — a corrupted expected digest must not verify
                // against the genuine bytes (flip the first hex nibble).
                let mut corrupted: Vec<char> = expected.chars().collect();
                if let Some(c) = corrupted.first_mut() {
                    *c = if *c == '0' { '1' } else { '0' };
                }
                let wrong_claim = IdentityClaim {
                    concept: IdentityConcept::RawHash,
                    data: ClaimData::HashAlgorithm {
                        algorithm,
                        digest_hex: corrupted.into_iter().collect(),
                    },
                };
                if matches!(
                    raw_hash::verify(&wrong_claim, source),
                    VerificationResult::Verified(_)
                ) {
                    return Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                        self.meta(),
                    )));
                }
            }
        }
        Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "IntegrityClaimVerifiable",
        "a named-algorithm integrity claim (SHA-256/512, BLAKE3; weak functions unrepresentable) verifies the true digest and rejects tampered content or a corrupted digest",
        "W3C (2016) Subresource Integrity; NIST (2015) FIPS 180-4 §6.2; Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020) BLAKE3"
    );
}

pr4xis::register_axiom!(IntegrityClaimVerifiable, "W3C (2016) Subresource Integrity");

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
        assert!(IntegrityClaimVerifiable.verify().is_ok());
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

    /// The USC leg of `SourceHashFaithfulness` has teeth too: a USC envelope
    /// whose raw blob disagrees with its pin is rejected by
    /// `usc_reconstruct_source` (so the widened axiom can FAIL on a USC defect,
    /// not just pass).
    #[test]
    fn usc_reconstruct_source_rejects_a_lying_envelope() {
        let mut envelope = witness_usc_envelope("usc-liar", b"honest usc source");
        if let Some(raw) = envelope.raw.as_mut() {
            raw.blob.push(b'!'); // blob no longer hashes to content_address
        }
        assert!(
            usc_reconstruct_source(&envelope).is_err(),
            "a tampered USC raw blob must be rejected (fail-closed)"
        );
    }

    /// The WordNet leg of `SourceHashFaithfulness` has teeth too: a WordNet
    /// envelope whose raw blob disagrees with its pin is rejected by
    /// `wn_reconstruct_source` (so the widened axiom can FAIL on a WordNet
    /// defect, not just pass).
    #[test]
    fn wn_reconstruct_source_rejects_a_lying_envelope() {
        let mut envelope = witness_wordnet_envelope("wn-liar", b"honest wn source");
        if let Some(raw) = envelope.raw.as_mut() {
            raw.blob.push(b'!'); // blob no longer hashes to content_address
        }
        assert!(
            wn_reconstruct_source(&envelope).is_err(),
            "a tampered WordNet raw blob must be rejected (fail-closed)"
        );
    }
}

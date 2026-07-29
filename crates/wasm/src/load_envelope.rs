//! Load-envelope ontology — the browser runtime's ONE typed load path
//! ([`LoadRequest`](crate::Pr4xis) → `Encoding` →
//! `TrustAnchor` → `decode_and_project`) declared as a
//! first-class praxis ontology, with its fail-closed guarantee as a runnable,
//! registered axiom.
//!
//! Before this module, `Encoding` and `TrustAnchor` were plain Rust enums —
//! correct, but OPAQUE to praxis's own reasoning: the sibling OWL tier is
//! self-describing (`succinct_codec/ontology.rs` declares the raw-source
//! envelope and its DEFLATE transport as concepts with the
//! `RawSourceDeflateTransport` axiom; the Lens ontology holds the lens laws),
//! while the load envelope that admits every runtime ontology answered no
//! registry query. This module gives the load path the same citizenship: its
//! concepts are declared, its edges are kinded, and its one guarantee — EVERY
//! `Encoding` arm verifies its `TrustAnchor` BEFORE decoding a byte, and every
//! gate refuses fail-closed — is [`LoadEnvelopeFailClosed`], whose `verify()`
//! drives the REAL `decode_and_project` arms
//! (never a restatement of the enum).
//!
//! # Literature
//!
//! - **Dolstra, E. (2006)** *The Purely Functional Software Deployment Model*,
//!   ch. 6 — content-addressing by cryptographic hash: the Merkle-root anchor
//!   the content-addressed `.prx` arm re-derives and refuses on mismatch.
//! - **W3C (2016)** *Subresource Integrity* — verify the integrity metadata
//!   BEFORE using fetched bytes, fail-closed: the trust-anchor-before-decode
//!   ordering the axiom pins.
//! - **1 U.S.C. §204** (USLM XML titles) and **W3C OWL 2** (RDF/XML) — the two
//!   transport-trust source encodings, already cited on the `ContentType`
//!   provisioning ontology this envelope grounds on.
//! - **Deutsch, P. & Gailly, J.-L. (1996)** *RFC 1952: GZIP file format* — the
//!   OWL `.prx.gz` distribution envelope's wrapper, validated by the triple
//!   praxis.lock pin (the praxis.lock trust-anchor design).
//! - **Smith et al. (2005)** *Relations in biomedical ontologies* (OBO
//!   Relation Ontology), *Genome Biology* 6:R46 — the `part of` / `depends on`
//!   kinds the edges carry.

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::definition::Definition;
use pr4xis_runtime::lens::archive_lens::ArchiveLens;

use crate::{Encoding, LoadError, TrustAnchor, decode_and_project, embedded_demo};

/// A minimal, well-formed USLM Title (Title 18 §1) — the valid-payload fixture
/// the fail-closed axiom AND the native acceptance tests share, so the axiom
/// exercises the same arm the load-a-statute demo drives.
pub(crate) const SAMPLE_USLM_TITLE: &str = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><num value="18">Title 18—</num><heading>CRIMES AND CRIMINAL PROCEDURE</heading><section identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>First section</heading><content>Body text.</content></section></title>"##;

/// A minimal, real `rkyv`-encoded [`Archive`] — the [`Encoding::RkyvArchive`]
/// leg's valid-payload fixture. Built here (not read from a staged file: the
/// axiom must hold even in a checkout with no fetched USC corpus) via the
/// SAME `ArchiveLens::put_aligned` build.rs uses to stage a real title, so
/// the axiom exercises the real encoder, not a hand-rolled byte literal.
fn rkyv_probe_archive_bytes() -> (rkyv::util::AlignedVec<16>, ContentAddress) {
    let archive = Archive {
        nodes: vec![Definition {
            kind: "Probe".to_string(),
            name: "load-envelope-rkyv-probe".to_string(),
            edges: Vec::new(),
            axioms: Vec::new(),
            lexical: None,
        }],
        connections: Vec::new(),
    };
    let root = archive
        .root()
        .expect("a single-node Archive has a derivable Merkle root");
    (ArchiveLens::put_aligned(&archive), root)
}

pr4xis::ontology! {
    name: "LoadEnvelope",
    source: "Dolstra (2006) The Purely Functional Software Deployment Model ch. 6 — content addressing; W3C (2016) Subresource Integrity — verify-before-use, fail-closed; 1 U.S.C. §204 (USLM titles) and W3C OWL 2 (RDF/XML) — the transport-trust source encodings; Deutsch & Gailly (1996) RFC 1952 GZIP — the .prx.gz distribution wrapper under the triple praxis.lock pin; Smith et al. (2005) Relations in biomedical ontologies (OBO Relation Ontology), Genome Biology 6:R46",

    concepts: [
        LoadRequest,
        LoadEncoding,
        LoadTrustAnchor,
        UslmTitleEncoding,
        OwlSourceEncoding,
        OwlPrxGzEncoding,
        RkyvArchiveEncoding,
        TransportAnchor,
        LockPinnedAnchor,
        MerkleRootAnchor,
    ],

    labels: {
        LoadRequest: ("en", "Load request",
            "The ONE typed entry for loading knowledge into the runtime: WHAT is loaded (a typed encoding, never a byte-sniff), the payload bytes, and the trust anchor that makes the load fail-closed — decoded once at the JS↔wasm boundary."),
        LoadEncoding: ("en", "Load encoding",
            "WHAT a load request carries — the typed selector that resolves the (decoder, projection functor) pair by a single typed match, grounding on the cited ContentType provisioning ontology."),
        LoadTrustAnchor: ("en", "Load trust anchor",
            "HOW a load is made fail-closed (W3C Subresource Integrity 2016: verify before use) — one typed anchor, verified in its encoding's arm BEFORE any byte is decoded."),
        UslmTitleEncoding: ("en", "USLM title encoding",
            "A USLM XML title (1 U.S.C. §204), decoded by read_uslm_title and projected by the USC bridge; its bytes carry no embedded hash, so its trust is the transport anchor."),
        OwlSourceEncoding: ("en", "OWL source encoding",
            "A W3C OWL 2 RDF/XML source, decoded by read_owl and projected by the OWL bridge; its bytes carry no embedded hash, so its trust is the transport anchor."),
        OwlPrxGzEncoding: ("en", "OWL .prx.gz encoding",
            "The OWL .prx.gz distribution envelope (RFC 1952 wrapper), decoded by the three-pin gated load_prx_gz; its trust is the lock-pinned anchor (archive signature + source hash + RDFC-1.0 canonical id, looked up by name@version)."),
        RkyvArchiveEncoding: ("en", "rkyv archive encoding",
            "A pre-projected rkyv local-cache .prx archive (tasks #21/#29) — the same-toolchain, same-Cargo.lock zero-copy form every build-baked or on-demand-fetched ontology in this crate uses, admitted by ontology::materialize_bytes then re-deriving the Merkle root from the decoded content and refusing on mismatch (Dolstra 2006); its trust is the Merkle-root anchor."),
        TransportAnchor: ("en", "Transport anchor",
            "Integrity rests on the host having fetched the bytes from the registry-pinned URL — the honest floor for source encodings whose bytes embed no hash."),
        LockPinnedAnchor: ("en", "Lock-pinned anchor",
            "The praxis.lock triple pin keyed by name@version — archive signature, source hash, and RDFC-1.0 canonical graph id — each of which must verify or the load refuses."),
        MerkleRootAnchor: ("en", "Merkle-root anchor",
            "A trusted content-address root supplied from OUTSIDE the bytes (a peer's claim, a baked manifest); the gate re-derives the root from the decoded content and refuses on mismatch (Dolstra 2006)."),
    },

    // The four encodings and three anchors are KINDS of the two selector
    // concepts; a load request HAS an encoding and a trust anchor.
    is_a: [
        (UslmTitleEncoding, LoadEncoding),
        (OwlSourceEncoding, LoadEncoding),
        (OwlPrxGzEncoding, LoadEncoding),
        (RkyvArchiveEncoding, LoadEncoding),
        (TransportAnchor, LoadTrustAnchor),
        (LockPinnedAnchor, LoadTrustAnchor),
        (MerkleRootAnchor, LoadTrustAnchor),
    ],
    has_a: [
        (LoadRequest, LoadEncoding),
        (LoadRequest, LoadTrustAnchor),
    ],

    // Each encoding DEPENDS ON exactly one trust-anchor kind (`depends on`,
    // RO:0002502): its arm demands that anchor and refuses any other — the
    // pairing LoadEnvelopeFailClosed proves against the real arms.
    edges: [
        (UslmTitleEncoding, TransportAnchor, Dependency),
        (OwlSourceEncoding, TransportAnchor, Dependency),
        (OwlPrxGzEncoding, LockPinnedAnchor, Dependency),
        (RkyvArchiveEncoding, MerkleRootAnchor, Dependency),
    ],
}

/// Quality: a short symbolic description of each load-envelope concept,
/// matching the citation column in the ontology header.
#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = LoadEnvelopeConcept;
    type Value = &'static str;

    fn get(&self, c: &LoadEnvelopeConcept) -> Option<&'static str> {
        use LoadEnvelopeConcept as C;
        Some(match c {
            C::LoadRequest => "the one typed load entry: name + encoding + payload + trust anchor",
            C::LoadEncoding => "WHAT is loaded — the typed (decoder, functor) selector",
            C::LoadTrustAnchor => {
                "HOW the load is fail-closed — verified before decode (W3C SRI 2016)"
            }
            C::UslmTitleEncoding => "USLM XML title (1 U.S.C. §204) under transport trust",
            C::OwlSourceEncoding => "OWL 2 RDF/XML source (W3C OWL 2) under transport trust",
            C::OwlPrxGzEncoding => {
                ".prx.gz distribution envelope (RFC 1952) under the triple lock pin"
            }
            C::RkyvArchiveEncoding => {
                "pre-projected rkyv archive, same-toolchain zero-copy, under the re-derived Merkle root"
            }
            C::TransportAnchor => "registry-pinned-URL transport trust (bytes embed no hash)",
            C::LockPinnedAnchor => "praxis.lock triple pin by name@version",
            C::MerkleRootAnchor => "externally-supplied trusted root, re-derived from content",
        })
    }
}

/// THE fail-closed load-envelope axiom: every `Encoding` arm of the REAL
/// `decode_and_project` verifies its `TrustAnchor` BEFORE decoding — a
/// mismatched anchor is a typed `TrustMismatch` even when the payload is
/// arbitrary garbage (were trust checked after decode, garbage would surface
/// as a parse error instead) — and every gate refuses garbage / tampered /
/// mis-rooted bytes with a typed error, while the true fixtures still load
/// (the refusals are not vacuous).
///
/// This is runnable, not a tautology: each leg calls the same arm the browser
/// load takes, with the same fixtures the native acceptance tests use (the
/// embedded demo `.prx` + its baked root; the sample USLM title).
pub struct LoadEnvelopeFailClosed;

impl LoadEnvelopeFailClosed {
    /// The predicate — every leg against the real arms. Returns `false` on the
    /// first violated leg.
    fn holds() -> bool {
        let garbage: &[u8] = b"\xffnot any of the five formats\x00\x01\x02";
        let demo = embedded_demo();
        let true_root = match ContentAddress::from_hex(demo.root_hex) {
            Some(root) => root,
            None => return false,
        };
        let wrong_root = ContentAddress::of(b"not the demo root");
        let (rkyv_probe_buf, rkyv_true_root) = rkyv_probe_archive_bytes();
        let rkyv_probe_bytes = rkyv_probe_buf.as_slice();

        // ── Leg 1: anchor verified BEFORE decode, on every arm. A GARBAGE
        // payload under a mismatched anchor must be TrustMismatch — a parse
        // error here would mean the arm decoded first.
        let mismatched: [(Encoding, TrustAnchor); 4] = [
            (Encoding::UslmTitle, TrustAnchor::MerkleRoot(wrong_root)),
            (
                Encoding::OwlSource,
                TrustAnchor::LockPinned {
                    version: String::from("0"),
                },
            ),
            (Encoding::OwlPrxGz, TrustAnchor::Transport),
            (Encoding::RkyvArchive, TrustAnchor::Transport),
        ];
        for (encoding, trust) in &mismatched {
            if !matches!(
                decode_and_project("probe", *encoding, trust, garbage),
                Err(LoadError::TrustMismatch { .. })
            ) {
                return false;
            }
        }

        // ── Leg 2: with the RIGHT anchor kind, each gate refuses bad bytes
        // with its arm's typed verdict.
        if !matches!(
            decode_and_project(
                "probe",
                Encoding::UslmTitle,
                &TrustAnchor::Transport,
                garbage
            ),
            Err(LoadError::UslmParse(_))
        ) {
            return false;
        }
        if !matches!(
            decode_and_project(
                "probe",
                Encoding::OwlSource,
                &TrustAnchor::Transport,
                garbage
            ),
            Err(LoadError::OwlParse(_))
        ) {
            return false;
        }
        // An unregistered name@version has no praxis.lock pins: the three-pin
        // lookup itself refuses before any gunzip.
        if !matches!(
            decode_and_project(
                "no-such-vocabulary",
                Encoding::OwlPrxGz,
                &TrustAnchor::LockPinned {
                    version: String::from("0.0.0"),
                },
                garbage,
            ),
            Err(LoadError::MissingLockPin(_))
        ) {
            return false;
        }
        // The rkyv gate, over the REAL embedded demo bytes: the TRUE bytes
        // under a WRONG root are a typed root-mismatch refusal…
        if !matches!(
            decode_and_project(
                demo.name,
                Encoding::RkyvArchive,
                &TrustAnchor::MerkleRoot(wrong_root),
                demo.bytes,
            ),
            Err(LoadError::RkyvRootMismatch { .. })
        ) {
            return false;
        }
        // …and TAMPERED bytes under the TRUE root are refused too.
        let mut tampered = demo.bytes.to_vec();
        if let Some(last) = tampered.last_mut() {
            *last ^= 0xff;
        }
        if !matches!(
            decode_and_project(
                demo.name,
                Encoding::RkyvArchive,
                &TrustAnchor::MerkleRoot(true_root),
                &tampered,
            ),
            Err(LoadError::Materialize(_)) | Err(LoadError::RkyvRootMismatch { .. })
        ) {
            return false;
        }
        // The rkyv gate over a SEPARATE minimal synthetic archive: garbage
        // bytes under the RIGHT anchor kind fail
        // bytecheck validation inside `materialize_bytes` — a typed
        // Materialize refusal, never a silent admit.
        if !matches!(
            decode_and_project(
                "rkyv-probe",
                Encoding::RkyvArchive,
                &TrustAnchor::MerkleRoot(rkyv_true_root),
                garbage,
            ),
            Err(LoadError::Materialize(_))
        ) {
            return false;
        }
        // The TRUE rkyv bytes under a WRONG root are a typed root-mismatch
        // refusal (materialize_bytes takes no root of its own to check
        // against — the comparison this axiom exercises is the one
        // decode_and_project's RkyvArchive arm adds).
        if !matches!(
            decode_and_project(
                "rkyv-probe",
                Encoding::RkyvArchive,
                &TrustAnchor::MerkleRoot(wrong_root),
                rkyv_probe_bytes,
            ),
            Err(LoadError::RkyvRootMismatch { .. })
        ) {
            return false;
        }
        // …and TAMPERED rkyv bytes under the TRUE root are refused too
        // (either bytecheck rejects the corrupted buffer, or a corruption
        // bytecheck admits still re-derives a different root — both are
        // typed refusals, never a silent admit).
        let mut rkyv_tampered = rkyv_probe_bytes.to_vec();
        if let Some(last) = rkyv_tampered.last_mut() {
            *last ^= 0xff;
        }
        if decode_and_project(
            "rkyv-probe",
            Encoding::RkyvArchive,
            &TrustAnchor::MerkleRoot(rkyv_true_root),
            &rkyv_tampered,
        )
        .is_ok()
        {
            return false;
        }

        // ── Leg 3: positive controls — the refusals above are not vacuous.
        // The true demo bytes under the true root load; the sample USLM title
        // parses and projects under transport trust; the true rkyv probe
        // bytes under their own true root load too.
        decode_and_project(
            demo.name,
            Encoding::RkyvArchive,
            &TrustAnchor::MerkleRoot(true_root),
            demo.bytes,
        )
        .is_ok()
            && decode_and_project(
                "Title 18 (axiom fixture)",
                Encoding::UslmTitle,
                &TrustAnchor::Transport,
                SAMPLE_USLM_TITLE.as_bytes(),
            )
            .is_ok()
            && decode_and_project(
                "rkyv-probe",
                Encoding::RkyvArchive,
                &TrustAnchor::MerkleRoot(rkyv_true_root),
                rkyv_probe_bytes,
            )
            .is_ok()
    }
}

impl Axiom for LoadEnvelopeFailClosed {
    fn verify(&self) -> Verdict {
        if Self::holds() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LoadEnvelopeFailClosed",
        "every Encoding arm of the real decode_and_project verifies its TrustAnchor before decoding (a mismatched anchor on garbage is TrustMismatch, never a parse error), every gate refuses garbage/tampered/mis-rooted bytes with a typed error, and the true fixtures still load",
        "Dolstra (2006) The Purely Functional Software Deployment Model ch. 6 — content addressing; W3C (2016) Subresource Integrity — verify-before-use, fail-closed"
    );
}

pr4xis::register_axiom!(LoadEnvelopeFailClosed, constructor);

impl Ontology for LoadEnvelopeOntology {
    type Cat = LoadEnvelopeCategory;
    type Qual = ConceptDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(LoadEnvelopeFailClosed));
        axioms
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::ontology::registry::{axiom_by_name, describe_knowledge_base};

    #[test]
    fn category_laws() {
        assert_category_laws::<LoadEnvelopeCategory>();
    }

    /// Runs the category laws AND the fail-closed axiom's `verify()` against
    /// the real `decode_and_project` arms + the embedded demo fixtures.
    #[test]
    fn ontology_validates() {
        LoadEnvelopeOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    /// Every declared concept carries a non-empty, DISTINCT description.
    #[test]
    fn every_concept_carries_a_distinct_description() {
        let q = ConceptDescription;
        let mut seen = std::collections::BTreeSet::new();
        for c in LoadEnvelopeConcept::variants() {
            let d = q.get(&c).expect("every declared concept is described");
            assert!(!d.is_empty(), "{c:?} has an empty description");
            assert!(
                seen.insert(d),
                "{c:?} repeats another concept's description"
            );
        }
        assert!(
            !seen.is_empty(),
            "the ontology declares at least one concept"
        );
    }

    /// The load envelope is reasoned about through the SAME registry as any
    /// statute: the ontology is discoverable in the knowledge base and the
    /// fail-closed axiom re-binds by name (the load-time rebind gate). This is
    /// exactly the citizenship the plain `Encoding`/`TrustAnchor` enums lacked.
    #[test]
    fn discoverable_via_self_model() {
        assert!(
            describe_knowledge_base()
                .iter()
                .any(|v| v.name() == "LoadEnvelopeOntology"),
            "LoadEnvelope must be discoverable in the ontology registry"
        );
        assert!(
            axiom_by_name("LoadEnvelopeFailClosed").is_some(),
            "LoadEnvelopeFailClosed must re-bind through the registry (axiom_by_name)"
        );
    }
}

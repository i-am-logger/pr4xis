//! Ontology-archive storage ontology — content-addressed Merkle-DAG
//! storage for praxis ontologies, with a fail-closed source-hash load
//! gate and (future) supply-chain attestation (M4.ι.0 / task #175).
//!
//! This is the *first-class praxis ontology* the `.prx` machinery
//! realises: rather than describe the archive store in doc-comments,
//! praxis declares its concepts here and proves its guarantees as
//! runnable axioms (see `super::axioms`, whose `verify()` predicates
//! exercise the real `xml::owl::prx` realisation). The `.prx.gz`
//! envelope is *one* realisation; USC (#271) and other consumers become
//! thin realisations of the same ontology.
//!
//! # Literature
//!
//! - **Merkle (1987)** "A Digital Signature Based on a Conventional
//!   Encryption Function", CRYPTO '87 — the content-addressed hash tree.
//! - **Benet (2014)** "IPFS: Content-Addressed, Versioned, P2P File
//!   System" — the Merkle-DAG with cross-node dedup (IPLD).
//! - **Hamano & Torvalds** — Git's content-addressed DAG of blob/tree
//!   objects (identical content → identical object id → dedup).
//! - **Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020)** *BLAKE3: one
//!   function, fast everywhere* — the content
//!   address.
//! - **Deutsch (1996)** *GZIP file format* RFC 1952 — the compressed form.
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** ACM TOPLAS
//!   29(3) §2.2 — the emit/load well-behaved lens.
//! - **Hill** *rkyv* v0.8 — the deterministic zero-copy archive.
//! - **Samuel et al. (2010)** TUF, CCS '10; **Torres-Arias et al.
//!   (2019)** in-toto, USENIX Security '19; **OpenSSF** SLSA — the
//!   supply-chain attestation chain (future, `super::axioms` deferred).
//! - **W3C (2016)** *Subresource Integrity* — the integrity claim.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "OntologyArchiveStorage",
    source: "Merkle (1987) A Digital Signature Based on a Conventional Encryption Function, CRYPTO '87; Benet (2014) IPFS: Content-Addressed, Versioned, P2P File System; Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020) BLAKE3; Deutsch (1996) GZIP file format, RFC 1952; Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2; Samuel et al. (2010) TUF, CCS '10; Torres-Arias et al. (2019) in-toto, USENIX Security '19; W3C (2016) Subresource Integrity",

    concepts: [
        ContentAddressableNode,
        MerkleEdge,
        MerkleDag,
        MerkleRoot,
        BinaryEnvelope,
        CompressedForm,
        SourcePin,
        LoadGate,
        Attestation,
        IntegrityClaim,
        AttestationChain,
        SupplyChainStep,
    ],

    labels: {
        ContentAddressableNode: ("en", "Content-addressable node",
            "Merkle (1987); Benet (2014) IPFS: a stored node named by the cryptographic hash of its bytes — identical content yields the identical address."),
        MerkleEdge: ("en", "Merkle edge",
            "Benet (2014) IPLD: a link from one node to another by the target's content address; the edge set makes the store a directed acyclic graph."),
        MerkleDag: ("en", "Merkle DAG",
            "Merkle (1987); Benet (2014); Git object graph: the content-addressed directed acyclic graph of nodes and edges, with cross-node dedup."),
        MerkleRoot: ("en", "Merkle root",
            "Merkle (1987): the top content address that transitively fixes every reachable node — verifying the root verifies the whole sub-DAG."),
        BinaryEnvelope: ("en", "Binary envelope",
            "Hill rkyv v0.8: the deterministic zero-copy binary container for an ontology's archived data plus its self-describing metadata; itself a content-addressable node."),
        CompressedForm: ("en", "Compressed form",
            "Deutsch (1996) RFC 1952: the gzip-wrapped serialization of a binary envelope; gunzip(gzip(x)) == x (lossless)."),
        SourcePin: ("en", "Source pin",
            "Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020) BLAKE3; Dolstra (2006): the recorded content address of the authoritative source bytes an archive is derived from."),
        LoadGate: ("en", "Load gate",
            "Dolstra (2006); W3C (2016) SRI; Samuel et al. (2010) TUF: the fail-closed admission check that discharges an IntegrityClaim over the node it installs — it RE-DERIVES the content address from the node's own bytes and materializes only when that re-derived address equals the externally trusted recorded pin. It never trusts an embedded self-asserted label. On mismatch, an unverifiable claim, or an absent pin, nothing is installed."),
        Attestation: ("en", "Attestation",
            "Samuel et al. (2010) TUF; Torres-Arias et al. (2019) in-toto: a signed statement about how an archive was produced (future supply-chain work)."),
        IntegrityClaim: ("en", "Integrity claim",
            "W3C (2016) Subresource Integrity: a verifiable claim binding a resource to its expected content hash."),
        AttestationChain: ("en", "Attestation chain",
            "OpenSSF SLSA; in-toto: the ordered sequence of attestations covering each supply-chain step from source to published archive (future)."),
        SupplyChainStep: ("en", "Supply-chain step",
            "OpenSSF SLSA: one producer step (fetch, parse, project, archive) whose inputs and outputs an attestation records (future)."),
    },

    is_a: [
        // A Merkle root is itself a content-addressable node.
        (MerkleRoot, ContentAddressableNode),
        // A binary envelope is named by its content address — a node.
        (BinaryEnvelope, ContentAddressableNode),
        // A source pin is an integrity claim over the source bytes (SRI).
        (SourcePin, IntegrityClaim),
        // An attestation asserts integrity claims about a step.
        (Attestation, IntegrityClaim),
    ],

    has_a: [
        // A Merkle DAG is made of nodes, edges, and a root.
        (MerkleDag, ContentAddressableNode),
        (MerkleDag, MerkleEdge),
        (MerkleDag, MerkleRoot),
        // An envelope has a compressed form and pins its source.
        (BinaryEnvelope, CompressedForm),
        (BinaryEnvelope, SourcePin),
        // The load gate checks the source pin.
        (LoadGate, SourcePin),
        // An attestation chain is a sequence of steps and attestations.
        (AttestationChain, SupplyChainStep),
        (AttestationChain, Attestation),
    ],
}

/// Quality: a short symbolic description of each archive-storage
/// concept, matching the citation column in the ontology header.
#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = OntologyArchiveStorageConcept;
    type Value = &'static str;

    fn get(&self, c: &OntologyArchiveStorageConcept) -> Option<&'static str> {
        use OntologyArchiveStorageConcept as C;
        Some(match c {
            C::ContentAddressableNode => "node named by the BLAKE3 hash of its bytes (Merkle 1987)",
            C::MerkleEdge => "link to a node by its content address (Benet 2014 IPLD)",
            C::MerkleDag => "content-addressed DAG with cross-node dedup",
            C::MerkleRoot => "top address that fixes the whole sub-DAG (Merkle 1987)",
            C::BinaryEnvelope => "deterministic rkyv container for archived data + metadata",
            C::CompressedForm => "gzip wrapper, gunzip(gzip(x)) == x (RFC 1952)",
            C::SourcePin => "recorded content digest of the authoritative source bytes",
            C::LoadGate => {
                "fail-closed admission: re-derive the node's content address and admit iff it equals the trusted pin"
            }
            C::Attestation => "signed statement about how an archive was produced (future)",
            C::IntegrityClaim => "claim binding a resource to its expected hash (W3C SRI)",
            C::AttestationChain => "ordered attestations over each supply-chain step (future)",
            C::SupplyChainStep => "one producer step whose I/O an attestation records (future)",
        })
    }
}

impl Ontology for OntologyArchiveStorageOntology {
    type Cat = OntologyArchiveStorageCategory;
    type Qual = ConceptDescription;

    fn axioms() -> alloc::vec::Vec<alloc::boxed::Box<dyn Axiom>> {
        // `mut` is used only under `feature = "prx"` (the realisation pushes
        // below); without it the vector is returned unchanged.
        #[cfg_attr(not(feature = "prx"), allow(unused_mut))]
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        // The domain axioms are runnable only where a realisation exists.
        // The `.prx` realisation lives behind `feature = "prx"`; under it,
        // each axiom's `verify()` exercises the real machinery. When USC
        // (#271) lands as a second consumer, these generalise.
        #[cfg(feature = "prx")]
        {
            use super::axioms::{
                CompressionRoundTrip, EmitLoadWellBehaved, IntegrityClaimVerifiable,
                LoadGateFailsClosed, MerkleDedupCorrect, MerkleHashDeterministic, RkyvDeterminism,
                SourceHashFaithfulness,
            };
            axioms.push(alloc::boxed::Box::new(MerkleHashDeterministic));
            axioms.push(alloc::boxed::Box::new(MerkleDedupCorrect));
            axioms.push(alloc::boxed::Box::new(CompressionRoundTrip));
            axioms.push(alloc::boxed::Box::new(RkyvDeterminism));
            axioms.push(alloc::boxed::Box::new(EmitLoadWellBehaved));
            axioms.push(alloc::boxed::Box::new(SourceHashFaithfulness));
            axioms.push(alloc::boxed::Box::new(LoadGateFailsClosed));
            axioms.push(alloc::boxed::Box::new(IntegrityClaimVerifiable));
        }
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<OntologyArchiveStorageCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        // Under `feature = "prx"` this runs the eight realisation axioms
        // (each exercising the real `.prx` machinery); without it, the
        // structural axioms only. Both must hold.
        OntologyArchiveStorageOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn twelve_concepts() {
        assert_eq!(OntologyArchiveStorageConcept::variants().len(), 12);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn concept_description_total() {
        let q = ConceptDescription;
        for c in OntologyArchiveStorageConcept::variants() {
            assert!(q.get(&c).is_some(), "{c:?} missing description");
        }
    }
}

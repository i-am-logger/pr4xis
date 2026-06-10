//! Praxis knowledge-graph ontology (#272) — the concepts behind praxis's
//! **whole-graph wire protocol** (the p2p substrate that carries an instance's
//! ontologies + the functors / lenses over them to another); its shipped step
//! is a content-addressed graph **SLICE**.
//!
//! The unit of persistence is a SELECTED subgraph (a `RootSet` under an
//! `EdgeKindFilter` — the whole graph by default, but typically a minimal
//! slice): its concepts, axioms, lenses, and edges — content-addressed,
//! Merkle-dedup'd, rkyv-serialized as a binary that rehydrates into a live
//! graph whose behavioural nodes **re-bind** to the running binary's registered
//! handler tables. A reference LEAVING the slice is an explicit
//! `UnboundReference` (fail-closed) — the missing-piece manifest a higher,
//! deferred request/response layer would use to fetch what a peer lacks.
//!
//! Every persisted node ships in the *pair-ontology* shape
//! `(structural data, binding identity)`: the structural part is portable,
//! the behaviour lives in the praxis binary on each side, and the binding
//! identity is the stable wire-name the load step matches against the
//! receiver's registries (axioms via
//! [`pr4xis::ontology::axiom_by_name`], plus the functor / adjunction /
//! lens slices).
//!
//! This is the *whole-graph* generalisation of
//! [`OntologyArchiveStorage`](super::super::ontology_archive): the archive
//! is its content-addressed storage substratum, mapped in by the
//! fully-faithful [`ArchiveIntoGraph`](super::functor::ArchiveIntoGraph)
//! functor — created *alongside*, never a rewrite
//! (`feedback_evolution_via_functor`).
//!
//! # Literature
//!
//! - **Merkle (1987)**; **Benet (2014)** IPFS/IPLD; Git content-addressed
//!   DAG — the content-addressed node store.
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** ACM TOPLAS
//!   29(3) §2.2 (lenses); **Mac Lane (1971)** CWM Ch. I §3 (functor laws),
//!   Ch. IV §1 (adjunction triangles) — the behavioural nodes' laws.
//! - **Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020)** BLAKE3 (the
//!   content-address hash); **W3C (2016)** Subresource
//!   Integrity; **Samuel et al. (2010)** TUF; **Torres-Arias et al.
//!   (2019)** in-toto; **OpenSSF** SLSA — integrity + supply-chain
//!   attestation (the latter declared as concepts, deferred as axioms).
//! - **Hill** rkyv v0.8; **Deutsch (1996)** RFC 1952 — the binary wire form.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "PraxisKnowledgeGraph",
    source: "Merkle (1987) A Digital Signature Based on a Conventional Encryption Function, CRYPTO '87; Benet (2014) IPFS: Content-Addressed, Versioned, P2P File System; Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020) BLAKE3; Deutsch (1996) GZIP file format, RFC 1952; Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2; Mac Lane (1971) Categories for the Working Mathematician Ch. I §3 + Ch. IV §1; Samuel et al. (2010) TUF, CCS '10; Torres-Arias et al. (2019) in-toto, USENIX Security '19; W3C (2016) Subresource Integrity; Hill rkyv v0.8",

    concepts: [
        // === Structural-knowledge nodes (the kinds of node a graph holds) ===
        ConceptNode,
        AxiomNode,
        RuleNode,
        FunctorNode,
        AdjunctionNode,
        LensNode,
        CompositionNode,
        RelationEdge,
        LiteratureCitation,
        // === Pair-ontology bindings (the behavioural half of each node) ===
        AxiomBinding,
        RuleBinding,
        FunctorBinding,
        AdjunctionBinding,
        LensBinding,
        // === Selection / slicing (choosing a subgraph to emit) ===
        RootSet,
        EdgeKindFilter,
        ReachableSubgraph,
        UnboundReference,
        // === Storage / wire envelope (the content-addressed substratum) ===
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
        GraphVersion,
        GraphSnapshot,
    ],

    labels: {
        ConceptNode: ("en", "Concept node", "A concept (object) of some ontology, persisted as a content-addressed graph node."),
        AxiomNode: ("en", "Axiom node", "A runnable axiom (logic::Axiom) as a graph node; its predicate is supplied by an AxiomBinding."),
        RuleNode: ("en", "Rule node", "A structural rule (the relation-kind → licensed structural-axiom catalog) as a graph node."),
        FunctorNode: ("en", "Functor node", "A category::Functor as a graph node; its apply is supplied by a FunctorBinding."),
        AdjunctionNode: ("en", "Adjunction node", "A category::Adjunction (F ⊣ G) as a graph node; its unit/counit are supplied by an AdjunctionBinding."),
        LensNode: ("en", "Lens node", "A well-behaved lens (Foster 2007) as a graph node; its get/put are supplied by a LensBinding."),
        CompositionNode: ("en", "Composition node", "A composite of functors/lenses — a derived behavioural node referencing its components."),
        RelationEdge: ("en", "Relation edge", "A kinded morphism between two nodes (category::Arrow), persisted as part of the graph."),
        LiteratureCitation: ("en", "Literature citation", "A published source a node cites — the literature grounding required of every axiom and concept."),
        AxiomBinding: ("en", "Axiom binding", "The stable wire-name an AxiomNode re-binds to a registered axiom constructor by, on load (pr4xis::ontology::axiom_by_name)."),
        RuleBinding: ("en", "Rule binding", "The canonical relation-kind name a RuleNode re-binds to its structural-axiom constructor by."),
        FunctorBinding: ("en", "Functor binding", "The stable name (Provenance.name in the FUNCTORS slice) a FunctorNode re-binds its apply to."),
        AdjunctionBinding: ("en", "Adjunction binding", "The stable name (in the ADJUNCTIONS slice) an AdjunctionNode re-binds its unit/counit to."),
        LensBinding: ("en", "Lens binding", "The registration key (\"name@version\" in the lens registry) a LensNode re-binds its get/put to."),
        RootSet: ("en", "Root set", "The set of nodes chosen as roots of a selection — the starting points of a subgraph slice."),
        EdgeKindFilter: ("en", "Edge-kind filter", "Which morphism kinds a selection follows when computing reachability."),
        ReachableSubgraph: ("en", "Reachable subgraph", "The subgraph reachable from a RootSet through an EdgeKindFilter — the computed slice to emit."),
        UnboundReference: ("en", "Unbound reference", "A reference leaving the chosen slice (or a binding the receiver cannot resolve) — fail-closed on load."),
        ContentAddressableNode: ("en", "Content-addressable node", "Merkle (1987); Benet (2014) IPFS: a stored node named by the cryptographic hash of its bytes — identical content yields the identical address."),
        MerkleEdge: ("en", "Merkle edge", "Benet (2014) IPLD: a link to a node by its content address; the edge set makes the store a DAG."),
        MerkleDag: ("en", "Merkle DAG", "Merkle (1987); Benet (2014); Git object graph: the content-addressed DAG with cross-node dedup."),
        MerkleRoot: ("en", "Merkle root", "Merkle (1987): the top content address that transitively fixes every reachable node."),
        BinaryEnvelope: ("en", "Binary envelope", "Hill rkyv v0.8: the deterministic zero-copy binary container for archived data plus metadata; itself a content-addressable node."),
        CompressedForm: ("en", "Compressed form", "Deutsch (1996) RFC 1952: the gzip-wrapped serialization; gunzip(gzip(x)) == x."),
        SourcePin: ("en", "Source pin", "Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020) BLAKE3; Dolstra (2006): the recorded content address of the authoritative source bytes."),
        LoadGate: ("en", "Load gate", "Dolstra (2006); W3C (2016) SRI; Samuel et al. (2010) TUF: the fail-closed admission check that re-derives a node's content address from its own bytes and admits only on a match to the trusted pin."),
        Attestation: ("en", "Attestation", "Samuel et al. (2010) TUF; Torres-Arias et al. (2019) in-toto: a signed statement about how a node was produced (concept declared; axioms deferred)."),
        IntegrityClaim: ("en", "Integrity claim", "W3C (2016) Subresource Integrity: a verifiable claim binding a resource to its expected content hash."),
        AttestationChain: ("en", "Attestation chain", "OpenSSF SLSA; in-toto: the ordered attestations covering each supply-chain step (deferred)."),
        SupplyChainStep: ("en", "Supply-chain step", "OpenSSF SLSA: one producer step whose inputs and outputs an attestation records (deferred)."),
        GraphVersion: ("en", "Graph version", "A version label of the whole knowledge graph — the content address of a GraphSnapshot identifies it."),
        GraphSnapshot: ("en", "Graph snapshot", "The whole knowledge graph at a version, content-addressed as a Merkle DAG — the emit/load unit praxis's whole-graph wire protocol carries between instances (the teach-a-peer request/response negotiation is the deferred chat layer; realisation #271)."),
    },

    is_a: [
        // Storage subset — re-declared verbatim from OntologyArchiveStorage so
        // ArchiveIntoGraph maps each by name (these four are the only is_a
        // edges among the twelve archive concepts).
        (MerkleRoot, ContentAddressableNode),
        (BinaryEnvelope, ContentAddressableNode),
        (SourcePin, IntegrityClaim),
        (Attestation, IntegrityClaim),
        // Every persisted node is content-addressable. Each source below is
        // OUTSIDE the twelve-concept archive image (or is GraphSnapshot), so
        // none adds a Subsumption morphism BETWEEN two archive objects —
        // FunctorFullOnImageLaw stays green.
        (GraphSnapshot, ContentAddressableNode),
        (ConceptNode, ContentAddressableNode),
        (AxiomNode, ContentAddressableNode),
        (RuleNode, ContentAddressableNode),
        (FunctorNode, ContentAddressableNode),
        (AdjunctionNode, ContentAddressableNode),
        (LensNode, ContentAddressableNode),
        (CompositionNode, ContentAddressableNode),
        (RelationEdge, ContentAddressableNode),
        (LiteratureCitation, ContentAddressableNode),
    ],

    has_a: [
        // Storage subset — re-declared verbatim from OntologyArchiveStorage
        // (the only has_a edges among the twelve archive concepts).
        (MerkleDag, ContentAddressableNode),
        (MerkleDag, MerkleEdge),
        (MerkleDag, MerkleRoot),
        (BinaryEnvelope, CompressedForm),
        (BinaryEnvelope, SourcePin),
        (LoadGate, SourcePin),
        (AttestationChain, SupplyChainStep),
        (AttestationChain, Attestation),
        // Whole-graph snapshot structure.
        (GraphSnapshot, MerkleDag),
        (GraphSnapshot, GraphVersion),
        // The pair-ontology spine: each behavioural node carries its binding.
        (AxiomNode, AxiomBinding),
        (RuleNode, RuleBinding),
        (FunctorNode, FunctorBinding),
        (AdjunctionNode, AdjunctionBinding),
        (LensNode, LensBinding),
        // Selection structure.
        (ReachableSubgraph, RootSet),
        (ReachableSubgraph, EdgeKindFilter),
        (ReachableSubgraph, UnboundReference),
    ],
}

/// Quality: a short symbolic description of each knowledge-graph concept.
#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = PraxisKnowledgeGraphConcept;
    type Value = &'static str;

    fn get(&self, c: &PraxisKnowledgeGraphConcept) -> Option<&'static str> {
        use PraxisKnowledgeGraphConcept as C;
        Some(match c {
            C::ConceptNode => "an ontology concept as a content-addressed node",
            C::AxiomNode => "a runnable axiom node (predicate via AxiomBinding)",
            C::RuleNode => "a structural-rule node (catalog via RuleBinding)",
            C::FunctorNode => "a functor node (apply via FunctorBinding)",
            C::AdjunctionNode => "an adjunction node (unit/counit via AdjunctionBinding)",
            C::LensNode => "a well-behaved-lens node (get/put via LensBinding)",
            C::CompositionNode => "a composite of behavioural nodes",
            C::RelationEdge => "a kinded morphism between two nodes",
            C::LiteratureCitation => "a published source a node cites",
            C::AxiomBinding => "stable name an AxiomNode re-binds by (axiom_by_name)",
            C::RuleBinding => "relation-kind name a RuleNode re-binds by",
            C::FunctorBinding => "stable name a FunctorNode re-binds by",
            C::AdjunctionBinding => "stable name an AdjunctionNode re-binds by",
            C::LensBinding => "name@version a LensNode re-binds by",
            C::RootSet => "the roots of a selection",
            C::EdgeKindFilter => "which morphism kinds a selection follows",
            C::ReachableSubgraph => "the computed slice from roots through the filter",
            C::UnboundReference => "a reference leaving the slice — fail-closed on load",
            C::ContentAddressableNode => "node named by the BLAKE3 hash of its bytes (Merkle 1987)",
            C::MerkleEdge => "link to a node by its content address (Benet 2014 IPLD)",
            C::MerkleDag => "content-addressed DAG with cross-node dedup",
            C::MerkleRoot => "top address that fixes the whole sub-DAG (Merkle 1987)",
            C::BinaryEnvelope => "deterministic rkyv container for archived data + metadata",
            C::CompressedForm => "gzip wrapper, gunzip(gzip(x)) == x (RFC 1952)",
            C::SourcePin => "recorded content digest of the authoritative source bytes",
            C::LoadGate => {
                "fail-closed admission: re-derive the address and admit iff it equals the trusted pin"
            }
            C::Attestation => "signed statement about how a node was produced (deferred)",
            C::IntegrityClaim => "claim binding a resource to its expected hash (W3C SRI)",
            C::AttestationChain => "ordered attestations over each supply-chain step (deferred)",
            C::SupplyChainStep => "one producer step whose I/O an attestation records (deferred)",
            C::GraphVersion => "a version label of the whole graph",
            C::GraphSnapshot => "the whole graph at a version, content-addressed (#271)",
        })
    }
}

impl Ontology for PraxisKnowledgeGraphOntology {
    type Cat = PraxisKnowledgeGraphCategory;
    type Qual = ConceptDescription;

    fn axioms() -> alloc::vec::Vec<alloc::boxed::Box<dyn Axiom>> {
        // Structural axioms always run. The domain axioms (the runnable
        // ones — the fully-faithful ArchiveIntoGraph functor laws, the eight
        // re-exported archive axioms, the two re-bind axioms, and the lens /
        // selection / pair-round-trip axioms) live behind `feature = "prx"`,
        // where the `.prx` realisation they exercise exists; see
        // [`super::axioms`] (snapshot / attestation axioms deferred there).
        #[cfg_attr(not(feature = "prx"), allow(unused_mut))]
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        #[cfg(feature = "prx")]
        {
            axioms.extend(super::axioms::domain_axioms());
        }
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<PraxisKnowledgeGraphCategory>();
    }

    /// The ontology validates: every category law and every domain axiom
    /// discharges.
    ///
    /// `Ontology::validate()` would re-run the whole axiom set, including
    /// `LensLawPreservation` — which delegates to the round-trip lens harness over
    /// every registered lens (parsing every small on-disk source). That identical
    /// pass is already run, on every push, by its dedicated owner
    /// `well_behaved_lens::harness::tests::ci_gate_passes`, so re-running it inside
    /// this validation is pure duplication. This test therefore validates the SAME
    /// claim test-only — the category laws (already independently discharged by the
    /// sibling `category_laws` test) plus every axiom EXCEPT that one lens leg —
    /// while still asserting the lens leg is PRESENT in the axiom set (the cheap
    /// non-vacuity check). Production `validate()` is unchanged; the costly pass
    /// runs exactly once, by its owner.
    #[test]
    fn ontology_validates() {
        use pr4xis::category::laws::category_law_axioms;

        // The category-law leg of validate(). (Also discharged structurally by the
        // sibling `category_laws` test; run here so this test covers the same
        // surface validate() does.)
        for law in category_law_axioms::<PraxisKnowledgeGraphCategory>() {
            law.verify()
                .unwrap_or_else(|c| panic!("category law failed: {}", c.meta().name.as_str()));
        }

        // The axiom leg of validate(). Under `feature = "prx"` the domain axioms
        // include the heavy lens harness leg (`LensLawPreservation`) owned by
        // ci_gate_passes; assert that leg is PRESENT (non-vacuity), then skip it.
        // Without `prx`, the set is the structural axioms only (no lens leg).
        let axioms = PraxisKnowledgeGraphOntology::axioms();
        #[cfg(feature = "prx")]
        {
            let lens_law = super::super::axioms::LensLawPreservation.name();
            assert!(
                axioms.iter().any(|ax| ax.name() == lens_law),
                "LensLawPreservation must remain in the ontology's axiom set — its presence \
                 (not a re-run of its harness pass) is what this validation asserts for that \
                 leg; ci_gate_passes runs the pass"
            );
            for ax in &axioms {
                if ax.name() == lens_law {
                    continue; // owned by ci_gate_passes — see the doc comment above
                }
                ax.verify()
                    .unwrap_or_else(|c| panic!("axiom failed: {}", c.meta().name.as_str()));
            }
        }
        #[cfg(not(feature = "prx"))]
        for ax in &axioms {
            ax.verify()
                .unwrap_or_else(|c| panic!("axiom failed: {}", c.meta().name.as_str()));
        }
    }

    #[test]
    fn thirty_two_concepts() {
        assert_eq!(PraxisKnowledgeGraphConcept::variants().len(), 32);
    }

    #[test]
    fn concept_description_total() {
        let q = ConceptDescription;
        for c in PraxisKnowledgeGraphConcept::variants() {
            assert!(q.get(&c).is_some(), "{c:?} missing description");
        }
    }
}

//! Doctrine Discovery — the synthesis layer of the M4.κ track.
//!
//! Composes the three preceding pieces — FCA (M4.κ.2), Rule Algebra
//! (M4.κ.3), and the Causation → Derivation abductive functor
//! (M4.κ.1) — into a single pipeline: given a corpus of objects and
//! a pluggable attribute extractor, emit the canonical doctrine
//! basis, the doctrine-cluster lattice over the Classification
//! ontology, and the subsumption order on the discovered
//! implications.
//!
//! See [`ontology`] for the type-level concept inventory and
//! literature, and [`engine`] for the runtime entry point
//! [`engine::discover`].

pub mod engine;
pub mod ontology;

pub use engine::{
    AttributeExtractor, CanonicalBasisIsSubsumptionMinimal, DiscoveredClustersMatchLattice,
    DiscoveryIsDeterministic, DoctrineDiscovery, EveryImplicationIsContextValid, discover,
};
pub use ontology::{
    DoctrineDiscoveryCategory, DoctrineDiscoveryConcept, DoctrineDiscoveryLineage,
    DoctrineDiscoveryOntology, DoctrineDiscoveryRelation, DoctrineDiscoveryRelationKind,
    OutputsClassifyAsDiscoveryOutput, PipelineIsLinearChain,
};

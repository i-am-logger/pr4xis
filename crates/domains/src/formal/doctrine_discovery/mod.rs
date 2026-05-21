//! Doctrine Discovery — the synthesis layer that composes FCA, Rule
//! Algebra, and the Causation → Derivation abductive functor into a
//! single pipeline: given a corpus of objects and a pluggable
//! attribute extractor, emit the canonical doctrine basis, the
//! doctrine-cluster lattice over the Classification ontology, and
//! the subsumption order on the discovered implications.
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

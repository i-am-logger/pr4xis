//! Functor Synthesizer — closes the doctrine-discovery loop.
//!
//! Distils a [`crate::formal::doctrine_discovery::DoctrineDiscovery`]
//! into a runtime [`synthesizer::SynthesizedFunctor`] (object → cluster
//! index), and feeds the result back into the discovery pipeline as
//! an additional attribute extractor for the next cycle.
//!
//! See [`ontology`] for the type-level concept inventory and
//! literature, and [`synthesizer`] for the runtime entry point
//! [`synthesizer::synthesize`].

pub mod ontology;
pub mod synthesizer;

pub use ontology::{
    FunctorLawHasBothAxioms, FunctorSynthesizerCategory, FunctorSynthesizerConcept,
    FunctorSynthesizerLineage, FunctorSynthesizerOntology, FunctorSynthesizerRelation,
    FunctorSynthesizerRelationKind, IdentityAndCompositionAreComplementary,
    PipelineReachesConvergence,
};
pub use synthesizer::{
    ClusterAssignmentIsTightestFit, SynthesizedFunctor, SynthesizedFunctorPreservesComposition,
    SynthesizedFunctorPreservesIdentity, SynthesizerIsDeterministic, synthesize,
};

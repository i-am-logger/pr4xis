//! Counting / Cardinality (Frege 1884; Gelman & Gallistel 1978) --
//! the turing-benchmark A4 sibling to MereologyTheory.

pub mod ontology;

pub use ontology::{
    CountingCategory, CountingConcept, CountingKind, CountingOntology, CountingRelation,
    CountingRelationKind, cardinality,
};

//! MereologyTheory — formal parthood (issue #152).
//!
//! Leśniewski / Leonard-Goodman / Simons / Casati-Varzi lineage.
//! Richer vocabulary behind domain ontologies' `has_a:` clause.

pub mod counting;
pub mod ontology;
pub mod wordnet_grounding;

pub use ontology::{
    MereologyKind, MereologyTheoryCategory, MereologyTheoryConcept, MereologyTheoryOntology,
    MereologyTheoryRelation, MereologyTheoryRelationKind,
};
pub use wordnet_grounding::{ProperPartAndWholeAreGroundedInWordNet, wordnet_concept_of_mereology};

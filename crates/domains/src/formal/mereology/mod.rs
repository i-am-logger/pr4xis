//! MereologyTheory — formal parthood (issue #152).
//!
//! Leśniewski / Leonard-Goodman / Simons / Casati-Varzi lineage.
//! Richer vocabulary behind domain ontologies' `has_a:` clause.

pub mod ontology;

pub use ontology::{
    MereologyKind, MereologyTheoryCategory, MereologyTheoryConcept, MereologyTheoryOntology,
    MereologyTheoryRelation, MereologyTheoryRelationKind,
};

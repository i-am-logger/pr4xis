//! Classification — kinds, taxa, and ranks (issue #152).
//!
//! Guarino OntoClean + Linnaean seven ranks + Aristotle-Porphyry
//! differentia + Ghiselin-Hull species-as-individuals. Richer
//! vocabulary behind domain ontologies' `is_a:` clause.

pub mod ontology;

pub use ontology::{
    ClassificationCategory, ClassificationConcept, ClassificationLineage, ClassificationOntology,
    ClassificationRelation, ClassificationRelationKind,
};

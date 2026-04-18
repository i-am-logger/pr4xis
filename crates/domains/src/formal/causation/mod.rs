//! Causation — Lewis 1973 counterfactual theory, Pearl 2000 structural
//! equations, Reichenbach 1956 common-cause, Woodward 2003 interventionism,
//! Hall 2004 two concepts, Mackie 1974 INUS.
//!
//! The rich vocabulary that domain ontologies reach for when the bare
//! `causes:` clause isn't enough.

pub mod ontology;

pub use ontology::{
    CausationCategory, CausationConcept, CausationOntology, CausationRelation,
    CausationRelationKind, CauseRole,
};

//! WordNet ontology — Miller (1995) lexical database of synsets linked
//! by hypernym/hyponym/meronym/antonym relations.

pub mod ontology;

pub use ontology::{WordNetCategory, WordNetConcept, WordNetOntology, WordNetRelation};

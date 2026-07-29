//! VerbNet (Kipper, Korhonen, Ryant & Palmer 2008) — the loaded
//! syntactic-semantic verb-class hierarchy, an independent (not
//! WordNet-derived) corroboration signal for the two-entity relation path.
//!
//! Mirrors the shape of [`crate::cognitive::linguistics::wordnet`] (a small
//! typed vocabulary) plus [`crate::cognitive::linguistics::english`] (the
//! real loaded instance data): [`ontology`] carries the typed class/member
//! shape, [`reader`] populates it from the raw archived XML, [`store`] is
//! the indexed, cached, query-ready runtime view.

pub mod ontology;
pub mod reader;
pub mod store;

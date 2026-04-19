//! Category theory — the substrate ontology grounding `category/arrow.rs`,
//! `category/functor.rs`, `category/adjunction.rs`, `category/transformation.rs`.
//! Names the concepts (Morphism, Arrow, Functor, NaturalTransformation,
//! Adjunction, Identity, Composition, …) that core's Rust traits and structs
//! realise, per Mac Lane (1971), Awodey (2010), Bénabou (1967), Leinster (2004).

pub mod ontology;

pub use ontology::{CategoryTheoryCategory, CategoryTheoryConcept, CategoryTheoryOntology};

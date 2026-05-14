//! Proof theory — the substrate ontology grounding `logic/axiom.rs` and
//! `logic/proof.rs`. Names the concepts (Proof, Counterexample, Axiom,
//! Theorem, Cut, Normalisation, …) that core's Rust traits realise, per
//! Gentzen (1935), Prawitz (1965), Troelstra & Schwichtenberg (2000),
//! Girard-Lafont-Taylor (1989).

pub mod ontology;

pub use ontology::{
    ProofTheoryCategory, ProofTheoryConcept, ProofTheoryOntology, ProofTheoryTradition,
};

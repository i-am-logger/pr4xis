//! Formal logic ontologies — reasoning structures that are timeless /
//! substrate-independent.

pub mod derivation;
pub mod dialectics;
pub mod inference_rules;
pub mod kripke;
pub mod model_theory;
pub mod trace_theory;

// proof_theory moved to core: `pr4xis::logic::proof_theory`.
// Re-import from core; downstream domain ontologies that functored
// over ProofTheoryConcept will be re-wired during the domains
// migration.

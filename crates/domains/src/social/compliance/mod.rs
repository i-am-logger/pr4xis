//! The compliance ontology — IFF classification, escalation of force,
//! LOAC engagement rules, Geneva Convention axioms, plus US legal
//! statutes generated from `praxis.lock`.
pub mod case_law;
pub mod classification;
pub mod compositions;
pub mod escalation;
pub mod law;
pub mod ontology;
pub mod proof_standards;
pub mod statutes;

#[cfg(test)]
mod tests;

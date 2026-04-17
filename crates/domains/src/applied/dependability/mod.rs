//! Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) taxonomy.
//!
//! Defines what an Error IS — the foundation for Resilience (#123) and the
//! typed-error replacement for `Result<(), Vec<String>>` in `Ontology::validate()`.

pub mod ontology;
// NOTE: a Dependability → Diagnostics functor is desired. The previous
// diagnosis ("dense-to-kinded many-to-one collapse; needs lax functors") is
// superseded by the #98 research doc at docs/research/kinded-functor-
// failures.md: the real issue is directional — Dependability runs cause→
// observation (Fault→Error→Failure) while Diagnostics runs observation→
// cause (Symptom→Hypothesis→Diagnosis→FaultMode, Reiter 1987). The right
// construction is F: Dependability^op → Diagnostics, which needs an Op-
// category helper in pr4xis::category. Tracked as a follow-up to #98.

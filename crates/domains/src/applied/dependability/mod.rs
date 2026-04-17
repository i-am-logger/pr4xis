//! Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) taxonomy.
//!
//! Defines what an Error IS — the foundation for Resilience (#123) and the
//! typed-error replacement for `Result<(), Vec<String>>` in `Ontology::validate()`.

pub mod ontology;
// NOTE: a Dependability → Diagnostics functor is desired (#122) but the
// strict-Functor laws fail at this scale: Dependability is dense (no kinds)
// while Diagnostics is kinded, and the many-to-one collapse from 44 to 10
// concepts breaks `F(g ∘ f) = F(g) ∘ F(f)`. The right structure is a lax
// functor / natural transformation. Tracked as follow-up.

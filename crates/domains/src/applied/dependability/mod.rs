//! Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) taxonomy.
//!
//! Defines what an Error IS — the foundation for Resilience (#123) and the
//! typed-error replacement for `Result<(), Vec<String>>` in `Ontology::validate()`.

pub mod ontology;
// NOTE: the Dependability → Diagnostics functor remains deferred. The #98
// research doc's case-2 recommendation — "use Op<Dependability> → Diagnostics
// to invert causation" — turns out to require a second fix beyond Op<C>.
// Dependability is a dense category (every pair has a morphism, no kinds),
// while Diagnostics is kinded. Dense categories identify Identity and Composed
// self-loops; kinded ones distinguish them. Any functor into a kinded target
// therefore can't satisfy the composition law for self-loop results of
// non-identity compositions — the failure is empirically `F(g∘f) has kind
// Identity while F(g)∘F(f) has kind Composed`. See the expanded case-2 note
// in docs/research/kinded-functor-failures.md. Proper fix requires either a
// kinded Dependability variant or sub-category framework support.

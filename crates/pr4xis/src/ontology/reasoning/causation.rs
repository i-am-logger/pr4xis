//! Causation module — vestigial after #169.
//!
//! The `CausalDef` trait, `Causes<E>` morphism struct, `CausalCategory<T>`
//! wrapper, and per-def query helpers (`effects_of`, `causes_of`) were
//! deleted. Causal structure is now expressed as kinded morphisms in
//! an ontology's `Category` — Causation-kinded morphisms ARE the causal
//! graph.
//!
//! **Queries**: filter `C::morphisms()` by `Kind::Causation`.
//! **Structural axioms**: inherit via
//! [`crate::ontology::reasoning::structural_axioms_for`] — Causation kind
//! gets `AsymmetricOnKind` + `IrreflexiveOnKind` automatically per
//! Lewis (1973); Reichenbach (1956); Tarski (1941).

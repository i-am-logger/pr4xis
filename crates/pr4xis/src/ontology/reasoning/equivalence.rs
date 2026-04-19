//! Equivalence module — vestigial after #169.
//!
//! The `EquivalenceDef` trait, `Equivalent<E>` morphism struct,
//! `EquivalenceCategory<T>` wrapper, and per-def query helpers
//! (`equivalent_to`, `equivalence_class`, `all_classes`,
//! `are_equivalent`) were deleted. Equivalence is now expressed as
//! kinded morphisms in an ontology's `Category` — Equivalence-kinded
//! morphisms ARE the equivalence relation.
//!
//! **Queries**: filter `C::morphisms()` by `Kind::Equivalence`.
//! **Structural axioms**: inherit via
//! [`crate::ontology::reasoning::structural_axioms_for`] when an
//! Equivalence kind catalog entry is added; canonical properties per
//! Tarski (1941) Calculus of Relations — reflexive + symmetric +
//! transitive.

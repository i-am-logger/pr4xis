//! Opposition module — vestigial after #169.
//!
//! The `OppositionDef` trait and per-def query helpers (`opposites`,
//! `are_opposed`) were deleted. Opposition is now expressed as kinded
//! morphisms in an ontology's `Category` — Opposition-kinded morphisms
//! ARE the opposition relation.
//!
//! **Queries**: filter `C::morphisms()` by `Kind::Opposition`.
//! **Structural axioms**: inherit via
//! [`crate::ontology::reasoning::structural_axioms_for`] — Opposition
//! kind gets `SymmetricOnKind` + `IrreflexiveOnKind` automatically per
//! Aristotle *Peri Hermeneias* (Square of Opposition); Saussure (1916);
//! Cruse (1986); Tarski (1941).

//! Taxonomy module — vestigial after #169.
//!
//! The `TaxonomyDef` trait, `IsA<E>` morphism struct, `TaxonomyCategory<T>`
//! wrapper, and per-def query helpers (`is_a`, `ancestors`, `descendants`,
//! `inherit_quality`) were deleted. Taxonomy is now expressed as kinded
//! morphisms in an ontology's `Category` — every morphism carries a
//! `Kind` tag, and Subsumption-kinded morphisms ARE the taxonomy.
//!
//! **Queries**: filter `C::morphisms()` by `Kind::Subsumption`.
//! **Structural axioms**: inherit via
//! [`crate::ontology::reasoning::structural_axioms_for`] from the catalog
//! — Subsumption kind gets `NoCyclesOnKind` + `AntisymmetricOnKind`
//! automatically per OBO-RO (Smith et al. 2005).
//!
//! This module is kept as a placeholder so `use crate::ontology::reasoning::taxonomy`
//! paths in transitional code don't break outright. Domain migration
//! will rewrite callers to use kinded-morphism filtering; this file
//! deletes when that lands.

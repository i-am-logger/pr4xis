//! Mereology module — vestigial after #169.
//!
//! The `MereologyDef` trait, `HasA<E>` morphism struct, `MereologyCategory<T>`
//! wrapper, and per-def query helpers (`parts_of`, `whole_of`) were
//! deleted. Part-whole structure is now expressed as kinded morphisms
//! in an ontology's `Category` — Parthood-kinded morphisms ARE the
//! mereology.
//!
//! **Queries**: filter `C::morphisms()` by `Kind::Parthood`.
//! **Structural axioms**: inherit via
//! [`crate::ontology::reasoning::structural_axioms_for`] — Parthood kind
//! gets `NoCyclesOnKind` automatically per OBO-RO (Smith et al. 2005);
//! Casati & Varzi (1999) WeakSupplementation belongs as a domain axiom
//! in mereology-specific ontologies.

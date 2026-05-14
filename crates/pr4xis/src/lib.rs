//! # pr4xis — Axiomatic Intelligence
//!
//! Substrate crate for the pr4xis runtime. Defines the core traits that
//! every domain ontology composes against:
//!
//! - [`category::Arrow`] — directed structure between concepts, carrying
//!   a relation-kind tag and per-instance provenance.
//!   Grounded in Mac Lane (1971) *Categories for the Working Mathematician*
//!   Ch. I §1 and Awodey (2010) *Category Theory* 2nd ed.
//! - [`category::Concept`] — closed-world enum of named objects in a
//!   category. Grounded in Guarino (2009) *What is an Ontology?*
//! - [`logic::Axiom`] — verifiable claim returning a typed
//!   [`logic::proof::Verdict`] (`Ok` = `Proof` witness, `Err` =
//!   `Counterexample` refutation, per Martin-Löf 1984). Required
//!   companion: [`logic::Axiom::citation`] — every axiom traces to
//!   published literature.
//! - [`ontology::Ontology`] — `type Cat`, `type Qual`, and `fn axioms()`.
//!   Structural axioms come from
//!   [`ontology::reasoning::structural_axioms_for`] (the catalog).
//!
//! Authoring shortcut: the [`ontology!`] proc macro takes a declarative
//! shape (`name`, `source`, `concepts`, `labels`, `is_a:` / `has_a:` /
//! `causes:` / `opposes:` sugar clauses, optional inline `axioms:` block)
//! and emits the full impl chain at compile time.
//!
//! See `docs/understand/architecture.md` for the layered design and
//! `docs/learn/get-started.md` for a guided walk-through.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod category;
#[cfg(feature = "codegen")]
pub mod codegen;
pub mod codegen_data;
pub mod engine;
pub mod logic;
pub mod ontology;

pub use pr4xis_derive::ontology;

// Re-export linkme/paste so downstream macros can refer to them
// without requiring crates to add these dependencies directly.
#[doc(hidden)]
pub use linkme;
#[doc(hidden)]
pub use paste;

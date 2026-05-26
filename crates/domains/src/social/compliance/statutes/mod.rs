//! US legal statutes — auto-generated ontologies from `praxis.lock`
//! structural data.
//!
//! Each statute lives in its own sub-module containing a single
//! `include!()` line that pulls in the build-time codegen output
//! (`$OUT_DIR/<name>_codegen.rs`). The structural source of truth is
//! the matching `[structural."<name>@<version>"]` block in
//! `praxis.lock`; regenerating the lock regenerates these modules
//! automatically on the next build.
//!
//! The codegen produces a `<Name>Id` concept enum plus static arrays of
//! entity labels and relation tuples. Downstream code consumes the type
//! via `pr4xis::category::Concept` like any other ontology.
//!
//! # Why under `social::compliance`
//!
//! `social::compliance::law` already names LOAC / Geneva Convention
//! axioms; that domain is *international* humanitarian law. US statutes
//! are a parallel sibling — same Hart (1961) primary-rule status, but
//! a distinct corpus. Keeping them in `statutes/` avoids overloading
//! the `law` module name while still grouping them with the rest of
//! the compliance ontology.

pub mod air21_42121;
pub mod from_uslm;
pub mod lens;
pub mod sox_1514a;
pub mod statute;

pub use from_uslm::{derive_structural, from_uslm_section};
pub use statute::{Statute, StatuteConstructError};

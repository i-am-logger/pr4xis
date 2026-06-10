//! `Staging` — how an ontology *arrived* in the running system.
//!
//! This is the self-observable provenance of an ontology's instantiation: was
//! it frozen into the binary at build time, downloaded at runtime, memory-mapped
//! on demand, or produced by runtime composition? The self-model catalog
//! ([`crate`]'s knowledge-boundary model, in the `domains` crate) tags every
//! loaded source with its `Staging` so a meta-aware system can report not just
//! *what* it knows but *how that knowledge came to be present*.
//!
//! # The Futamura correspondence
//!
//! The staging quality maps directly onto the Futamura projections (Futamura
//! 1971, *Partial Evaluation of Computation Process — An Approach to a
//! Compiler-Compiler*):
//!
//! - [`Staging::Embedded`] = `StaticInput` — the ontology is specialized into
//!   the binary at build time (the first Futamura projection: an interpreter
//!   specialized to a fixed program becomes a compiled artifact). It is present
//!   with zero runtime cost.
//! - [`Staging::Async`] = `DynamicInput` — loaded from network or disk while the
//!   system runs.
//! - [`Staging::Mmap`] = `DynamicInput` — a memory-mapped file, demand-paged by
//!   the OS rather than eagerly read.
//! - [`Staging::Composed`] = `DynamicInput` — produced by runtime composition of
//!   already-present ontologies.
//!
//! `Staging` is a self-description *data type*: it carries no behaviour and no
//! composition logic — it is the observable label the meta-level reads. The
//! lowercase wire label for each variant lives next to its catalog consumer.

/// How an ontology arrived in the system — observable staging quality.
///
/// Maps directly to the Futamura projections (Futamura 1971):
///   * `Embedded` = StaticInput (first Futamura projection, frozen at build time)
///   * `Async` = DynamicInput (loaded from network/disk at runtime)
///   * `Mmap` = DynamicInput (memory-mapped file, demand-paged by the OS)
///   * `Composed` = DynamicInput (runtime composition of present ontologies)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Staging {
    /// Specialized into the binary at build time — present at zero runtime cost.
    Embedded,
    /// Loaded from network or disk while the system runs.
    Async,
    /// A memory-mapped file, demand-paged by the OS.
    Mmap,
    /// Produced by runtime composition of already-present ontologies.
    Composed,
}

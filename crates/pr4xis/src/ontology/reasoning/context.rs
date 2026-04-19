//! Context module — vestigial after #169.
//!
//! The `ContextDef` trait and per-def query helpers (`resolve`,
//! `interpretations`, `ambiguous_entities`) were deleted. Context
//! resolution belongs in the cognitive/linguistics domain ontologies
//! that need it, not as a core substrate concern.
//!
//! **Literature that lived here:**
//! - Carnap (1947) *Meaning and Necessity* — intension + context → extension
//! - Pustejovsky (1995) *The Generative Lexicon* — context-dependent
//!   semantics
//!
//! These invariants should be re-expressed as domain axioms in the
//! linguistics ontologies that use them.

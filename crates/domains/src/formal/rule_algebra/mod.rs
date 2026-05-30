//! Rule Algebra — subsumption / normalization / conflict detection
//! over conditional rules.
//!
//! Composes existing praxis pieces:
//!
//! - [`crate::social::judicial::ontology::RelationType`] supplies the
//!   13 legal relation kinds (Requires, Implies, Contradicts, …).
//! - [`crate::social::judicial::rule`] supplies the operational fact-
//!   driven legal rule (the evaluator).
//! - [`crate::formal::analytical_methods::fca`] supplies the FCA
//!   implication basis that the algebra normalises against.
//! - Deontic operators via von Wright (1951) Mind 60:1-15 — codified
//!   here in [`implication::DeonticOperator`].
//!
//! This module adds the *algebraic operations* — what's done with
//! rules viewed as logical objects — declared in [`ontology`] and
//! implemented in [`implication`] (single-rule) and [`rule_set`]
//! (collection level).

pub mod implication;
pub mod ontology;
pub mod rule_set;

pub use implication::{DeonticOperator, Implication};
pub use ontology::{
    DeonticSquareOpposes, ImplicationIsRuleShape, RuleAlgebraCategory, RuleAlgebraConcept,
    RuleAlgebraLineage, RuleAlgebraOntology, RuleAlgebraRelation, RuleAlgebraRelationKind,
    StrictDefeasibleOppose,
};
pub use rule_set::{
    CanonicalBasisIsSubset, ConflictSymmetric, DeonticConflictDetected, NormalizationIdempotent,
    RuleSet, SubsumptionReflexive, SubsumptionTransitive,
};

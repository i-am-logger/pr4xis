//! Proof and Counterexample — typed ontological results of `Axiom::verify()`.
//!
//! # Two typed outcomes, zero booleans
//!
//! Under Martin-Löf (1984) *Intuitionistic Type Theory*, a proof IS a term
//! inhabiting the type of its claim — the inhabitant is the proof.
//! Falsity is a DIFFERENT type: a term of type `P → ⊥` (Curry & Feys 1958).
//! A boolean verdict flattens these two structurally-distinct kinds of
//! evidence into one — which is exactly the primitive-leak pr4xis rejects.
//!
//! `Axiom::verify()` therefore returns
//! `Result<Box<dyn Proof>, Box<dyn Counterexample>>`: the `Ok` branch
//! carries a term witnessing the claim, the `Err` branch carries a term
//! witnessing the claim's negation. No `is_valid() -> bool` anywhere in
//! core.
//!
//! # Literature
//!
//! - Martin-Löf (1984) *Intuitionistic Type Theory* — proof is term,
//!   term-of-`⊥` is refutation
//! - Curry & Feys (1958) *Combinatory Logic* — propositions-as-types
//! - Prawitz (1965) *Natural Deduction* — proofs as structured derivations
//! - Joyal, Street & Verity (1996) — traced monoidal categories
//! - Lambek (1968) "Deductive Systems and Categories" — proofs as morphisms
//!
//! Issue #160.

use core::fmt::Debug;

use crate::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

/// Outcome of verifying a claim — a typed `Proof` if the claim holds, a
/// typed `Counterexample` if it fails. Zero booleans (#160).
pub type Verdict = Result<Box<dyn Proof>, Box<dyn Counterexample>>;

/// A witnessed claim — the `Ok` branch of [`Verdict`].
///
/// Under Curry-Howard, the inhabitant IS the proof: its existence witnesses
/// the claim. Concrete implementations can carry arbitrary internal
/// structure (Witness, Hypothesis, Consequence, sub-derivations) — consumers
/// downcast when they need richer access.
///
/// Object-safe so `Box<dyn Proof>` is valid.
pub trait Proof: Debug {
    /// Structured metadata — name, description, citation, module path.
    fn meta(&self) -> Provenance;
}

/// A refutation — the `Err` branch of [`Verdict`]. A term witnessing the
/// claim's negation under Martin-Löf's `P → ⊥` discipline.
///
/// Object-safe so `Box<dyn Counterexample>` is valid.
pub trait Counterexample: Debug {
    /// Structured metadata — name, description, citation, module path.
    fn meta(&self) -> Provenance;
}

// ---------------------------------------------------------------------------
// Concrete canonical proofs and counterexamples
// ---------------------------------------------------------------------------

/// Minimal concrete [`Proof`] — carries a metadata record only.
///
/// Used by axioms whose witnessed claim has no internal derivation
/// structure. Axioms with richer proofs (sub-derivations, hypotheses,
/// witnesses) implement their own [`Proof`] — typically reusing the
/// Derivation ontology's concept vocabulary.
#[derive(Debug, Clone)]
pub struct SimpleProof {
    meta: Provenance,
}

impl SimpleProof {
    pub fn new(meta: Provenance) -> Self {
        Self { meta }
    }
}

impl Proof for SimpleProof {
    fn meta(&self) -> Provenance {
        self.meta.clone()
    }
}

/// Minimal concrete [`Counterexample`] — carries a metadata record only.
///
/// The refutation counterpart to [`SimpleProof`]: the axiom's claim fails
/// but no richer counter-derivation structure is attached.
#[derive(Debug, Clone)]
pub struct SimpleCounterexample {
    meta: Provenance,
}

impl SimpleCounterexample {
    pub fn new(meta: Provenance) -> Self {
        Self { meta }
    }
}

impl Counterexample for SimpleCounterexample {
    fn meta(&self) -> Provenance {
        self.meta.clone()
    }
}

// Note: no `from_bool` / `verdict_from_bool` helper is provided. Core
// public API never accepts or returns `bool` — see `feedback_core_no_bool_api`.
// Axiom implementations construct `Ok(Box::new(SimpleProof::new(meta)))` or
// `Err(Box::new(SimpleCounterexample::new(meta)))` directly from their own
// domain-specific check expression.

// ---------------------------------------------------------------------------
// Composite proofs and counterexamples
// ---------------------------------------------------------------------------

/// Aggregates sub-verdicts — the composite witnesses the claim "every
/// sub-claim holds". If any sub-verdict is a counterexample, the composite
/// itself fails (see [`combine_verdicts`]).
///
/// Pattern from Prawitz (1965): a compound derivation is the concatenation
/// of its sub-derivations; the conclusion holds iff every sub-derivation
/// discharges.
#[derive(Debug)]
pub struct CompositeProof {
    meta: Provenance,
    subproofs: Vec<Box<dyn Proof>>,
}

impl CompositeProof {
    pub fn new(meta: Provenance, subproofs: Vec<Box<dyn Proof>>) -> Self {
        Self { meta, subproofs }
    }

    pub fn subproofs(&self) -> &[Box<dyn Proof>] {
        &self.subproofs
    }
}

impl Proof for CompositeProof {
    fn meta(&self) -> Provenance {
        self.meta.clone()
    }
}

/// Aggregates sub-counterexamples — the composite refutes the claim
/// "every sub-claim holds" by carrying the specific failing sub-refutations
/// (plus the passing sub-proofs for context).
#[derive(Debug)]
pub struct CompositeCounterexample {
    meta: Provenance,
    passed: Vec<Box<dyn Proof>>,
    failed: Vec<Box<dyn Counterexample>>,
}

impl CompositeCounterexample {
    pub fn new(
        meta: Provenance,
        passed: Vec<Box<dyn Proof>>,
        failed: Vec<Box<dyn Counterexample>>,
    ) -> Self {
        Self {
            meta,
            passed,
            failed,
        }
    }

    pub fn passed(&self) -> &[Box<dyn Proof>] {
        &self.passed
    }

    pub fn failed(&self) -> &[Box<dyn Counterexample>] {
        &self.failed
    }
}

impl Counterexample for CompositeCounterexample {
    fn meta(&self) -> Provenance {
        self.meta.clone()
    }
}

/// Combine a sequence of sub-verdicts into one aggregate verdict.
///
/// `Ok` iff every sub-verdict is `Ok`; otherwise `Err` carrying a
/// [`CompositeCounterexample`] that preserves both the failing refutations
/// and the passing proofs for granular reporting.
pub fn combine_verdicts(meta: Provenance, subverdicts: Vec<Verdict>) -> Verdict {
    let mut passed: Vec<Box<dyn Proof>> = Vec::new();
    let mut failed: Vec<Box<dyn Counterexample>> = Vec::new();
    for v in subverdicts {
        match v {
            Ok(p) => passed.push(p),
            Err(c) => failed.push(c),
        }
    }
    if failed.is_empty() {
        Ok(Box::new(CompositeProof::new(meta, passed)))
    } else {
        Err(Box::new(CompositeCounterexample::new(meta, passed, failed)))
    }
}

// ---------------------------------------------------------------------------
// Meta helpers for convenience construction
// ---------------------------------------------------------------------------

/// Build a minimal [`Provenance`] for quick inline construction.
pub fn proof_meta(name: &'static str, citation: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(name),
        citation: Citation::parse_static(citation),
        module_path: ModulePath::new_static(module_path!()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Constructing a `SimpleProof` directly — no bool helper, per
    /// `feedback_core_no_bool_api`.
    #[crate::praxis_value(Explainable)]
    #[test]
    fn simple_proof_carries_meta() {
        let p = SimpleProof::new(proof_meta("TestClaim", "Tarski (1941)"));
        assert_eq!(p.meta().name.as_str(), "TestClaim");
    }

    #[crate::praxis_value(Explainable)]
    #[test]
    fn simple_counterexample_carries_meta() {
        let c = SimpleCounterexample::new(proof_meta("FailedClaim", "Lewis (1973)"));
        assert_eq!(c.meta().name.as_str(), "FailedClaim");
    }

    #[crate::praxis_value(Verifiable)]
    #[test]
    fn combine_all_ok_yields_ok_composite() {
        let subs: Vec<Verdict> = vec![
            Ok(Box::new(SimpleProof::new(proof_meta("A", "X")))),
            Ok(Box::new(SimpleProof::new(proof_meta("B", "X")))),
        ];
        match combine_verdicts(proof_meta("Composite", "X"), subs) {
            Ok(_) => {}
            Err(_) => panic!("expected composite proof"),
        }
    }

    #[crate::praxis_value(Honest, Verifiable)]
    #[test]
    fn combine_with_any_err_yields_err_composite() {
        let subs: Vec<Verdict> = vec![
            Ok(Box::new(SimpleProof::new(proof_meta("A", "X")))),
            Err(Box::new(SimpleCounterexample::new(proof_meta("B", "X")))),
        ];
        match combine_verdicts(proof_meta("Composite", "X"), subs) {
            Ok(_) => panic!("expected composite counterexample"),
            Err(c) => assert!(!c.meta().name.as_str().is_empty()),
        }
    }

    #[crate::praxis_value(Verifiable)]
    #[test]
    fn proof_is_dyn_safe() {
        let _p: Box<dyn Proof> = Box::new(SimpleProof::new(proof_meta("X", "Y")));
    }

    #[crate::praxis_value(Verifiable)]
    #[test]
    fn counterexample_is_dyn_safe() {
        let _c: Box<dyn Counterexample> = Box::new(SimpleCounterexample::new(proof_meta("X", "Y")));
    }
}

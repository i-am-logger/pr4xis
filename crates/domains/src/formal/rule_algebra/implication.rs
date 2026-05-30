//! Runtime types for rule algebra: [`Implication`], [`DeonticOperator`],
//! plus the three core operations subsumption / normalization /
//! conflict detection.
//!
//! See [`super::ontology`] for the type-level concept inventory and
//! literature. This module is the executable counterpart.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use core::cmp::Ordering;

// =============================================================================
// Deontic flavour — von Wright (1951).
// =============================================================================

/// The deontic flavour of an implication's consequent. von Wright
/// (1951) "Deontic Logic", *Mind* 60: 1–15 — Obligation, Permission,
/// Prohibition, plus the plain assertoric (no modal) form for
/// classical rule reasoning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeonticOperator {
    /// `Op` — "p is required".
    Obligation,
    /// `Pp` — "p is allowed".
    Permission,
    /// `Fp` — "p is forbidden".
    Prohibition,
    /// Plain "p holds" — no deontic interpretation.
    Assertoric,
}

impl DeonticOperator {
    /// Two deontic operators *conflict* on a common target when one is
    /// Obligation and the other is Prohibition (von Wright 1951
    /// deontic square: `Op ∧ Fp` is the canonical contradiction).
    /// Permission and Assertoric never conflict on the operator
    /// dimension (though their consequents may still contradict at
    /// the concept level — checked separately by
    /// [`Implication::conflicts_with`]).
    #[must_use]
    pub fn conflicts_with(self, other: Self) -> bool {
        matches!(
            (self, other),
            (DeonticOperator::Obligation, DeonticOperator::Prohibition)
                | (DeonticOperator::Prohibition, DeonticOperator::Obligation)
        )
    }
}

// =============================================================================
// Implication — Robinson (1965), Plotkin (1970).
// =============================================================================

/// A propositional implication `antecedent ⇒ consequent` over a
/// concept space `C`, tagged with a [`DeonticOperator`].
///
/// **Invariants** (preserved by [`Implication::new`] and
/// [`Implication::normalize`]):
///
/// - `antecedent` and `consequent` are sorted ascending and deduped
///   (so structurally-equal rules compare `==`).
/// - Empty antecedent means "axiom" — the rule fires unconditionally.
///
/// Citation: Robinson (1965) JACM 12:23–41; Plotkin (1970) Machine
/// Intelligence 5:153–163.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Implication<C> {
    antecedent: Vec<C>,
    consequent: Vec<C>,
    deontic: DeonticOperator,
}

impl<C> Implication<C> {
    /// Borrow the antecedent (sorted, deduped).
    #[must_use]
    pub fn antecedent(&self) -> &[C] {
        &self.antecedent
    }

    /// Borrow the consequent (sorted, deduped).
    #[must_use]
    pub fn consequent(&self) -> &[C] {
        &self.consequent
    }

    /// The deontic operator tagging this implication's consequent.
    #[must_use]
    pub fn deontic(&self) -> DeonticOperator {
        self.deontic
    }
}

impl<C: Ord> Implication<C> {
    /// Construct a normalized implication. Antecedent and consequent
    /// are sorted and deduplicated.
    #[must_use]
    pub fn new(antecedent: Vec<C>, consequent: Vec<C>, deontic: DeonticOperator) -> Self {
        Self {
            antecedent: dedup_sorted(antecedent),
            consequent: dedup_sorted(consequent),
            deontic,
        }
    }

    /// Construct an Assertoric (deontic-flavour-free) implication.
    #[must_use]
    pub fn assertoric(antecedent: Vec<C>, consequent: Vec<C>) -> Self {
        Self::new(antecedent, consequent, DeonticOperator::Assertoric)
    }

    /// Construct an Obligation implication (von Wright `Op`).
    #[must_use]
    pub fn obligation(antecedent: Vec<C>, consequent: Vec<C>) -> Self {
        Self::new(antecedent, consequent, DeonticOperator::Obligation)
    }

    /// Construct a Prohibition implication (von Wright `Fp`).
    #[must_use]
    pub fn prohibition(antecedent: Vec<C>, consequent: Vec<C>) -> Self {
        Self::new(antecedent, consequent, DeonticOperator::Prohibition)
    }

    /// Construct a Permission implication (von Wright `Pp`).
    #[must_use]
    pub fn permission(antecedent: Vec<C>, consequent: Vec<C>) -> Self {
        Self::new(antecedent, consequent, DeonticOperator::Permission)
    }

    /// **Subsumption** (Plotkin 1970 θ-subsumption, restricted to set-
    /// form clauses): `self ≼ other` iff `self.antecedent ⊆
    /// other.antecedent` and `self.consequent ⊇ other.consequent` and
    /// `self.deontic == other.deontic`.
    ///
    /// Reading: `self` is at least as general as `other` — it fires on
    /// fewer conditions and concludes at least as much. The deontic
    /// dimension is exact: an Obligation rule subsumes only other
    /// Obligation rules (mixed-modality subsumption would require a
    /// stronger modal logic — see ASPIC+ Modgil-Prakken 2014).
    #[must_use]
    pub fn subsumes(&self, other: &Self) -> bool {
        self.deontic == other.deontic
            && is_subset_sorted(&self.antecedent, &other.antecedent)
            && is_subset_sorted(&other.consequent, &self.consequent)
    }

    /// **Normalization** — return the canonical form of this
    /// implication. With the [`Implication::new`] invariants this is
    /// the identity (antecedent and consequent are already sorted +
    /// deduped); the function is provided so the operation is
    /// callable as named in the `super::ontology::Normalization`
    /// concept. Idempotent.
    #[must_use]
    pub fn normalize(self) -> Self {
        // Already canonical by construction; this exposes the named
        // operation and acts as a defensive idempotent wrapper.
        Self::new(self.antecedent, self.consequent, self.deontic)
    }

    /// **Conflict detection** (Prakken & Sartor 1997 / Modgil-Prakken
    /// 2014, restricted to the deontic-flavour conflict).
    ///
    /// `self` and `other` are *in conflict* iff:
    ///
    /// 1. Their deontic operators conflict
    ///    ([`DeonticOperator::conflicts_with`] — i.e. Obligation vs
    ///    Prohibition).
    /// 2. They share at least one consequent concept — the **target**
    ///    of the deontic conflict.
    /// 3. Their antecedents are *jointly satisfiable*: in the
    ///    propositional set-form, this means neither antecedent
    ///    contradicts the other (here, the antecedents need only be
    ///    compatible, i.e. not literal negations — for set-form
    ///    rules without explicit negation, this reduces to "both
    ///    could fire", which is always true). We use the stronger
    ///    *antecedent-overlap* test: at least one shared antecedent
    ///    concept, OR one antecedent is empty (an unconditional rule
    ///    always fires under any state of the world).
    #[must_use]
    pub fn conflicts_with(&self, other: &Self) -> bool {
        if !self.deontic.conflicts_with(other.deontic) {
            return false;
        }
        // Shared target — at least one consequent concept in common.
        let targets_overlap = self
            .consequent
            .iter()
            .any(|c| binary_search_sorted(&other.consequent, c).is_some());
        if !targets_overlap {
            return false;
        }
        // Antecedent-overlap or unconditional firing.
        if self.antecedent.is_empty() || other.antecedent.is_empty() {
            return true;
        }
        self.antecedent
            .iter()
            .any(|c| binary_search_sorted(&other.antecedent, c).is_some())
    }
}

// =============================================================================
// Sorted-slice helpers. Kept private — Implication's invariants mean
// every public Vec<C> field is already sorted+deduped, so subset and
// search use O(min) algorithms over the sorted runs.
// =============================================================================

/// Sort + dedup a vector. Stable: equal elements preserve their
/// relative order (Vec::sort is stable; dedup removes consecutive
/// equals after sort).
fn dedup_sorted<C: Ord>(mut xs: Vec<C>) -> Vec<C> {
    xs.sort();
    xs.dedup();
    xs
}

/// `is_subset_sorted(a, b)` returns true iff every element of `a` is
/// in `b`, both sorted. O(|a| + |b|) merge walk.
fn is_subset_sorted<C: Ord>(a: &[C], b: &[C]) -> bool {
    let mut bi = 0usize;
    for x in a {
        // Advance b while it's smaller than x.
        while bi < b.len() {
            match b[bi].cmp(x) {
                Ordering::Less => bi += 1,
                Ordering::Equal => break,
                Ordering::Greater => return false,
            }
        }
        if bi >= b.len() {
            return false;
        }
        if b[bi] != *x {
            return false;
        }
        // Don't advance bi past equal — the next x in `a` could also
        // equal `b[bi]` (deduped antecedents make this rare but the
        // algorithm should be robust to non-deduped slices anyway).
    }
    true
}

/// Binary search a sorted slice. Returns the index if present.
fn binary_search_sorted<C: Ord>(xs: &[C], target: &C) -> Option<usize> {
    xs.binary_search(target).ok()
}

// =============================================================================
// Tests of the private helpers — public API tested via tests.rs.
// =============================================================================

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn dedup_sorted_works() {
        assert_eq!(dedup_sorted(vec![3, 1, 2, 1, 3]), vec![1, 2, 3]);
        assert_eq!(dedup_sorted::<i32>(vec![]), Vec::<i32>::new());
        assert_eq!(dedup_sorted(vec![5]), vec![5]);
    }

    #[test]
    fn is_subset_sorted_works() {
        assert!(is_subset_sorted(&[1, 2], &[1, 2, 3]));
        assert!(is_subset_sorted::<i32>(&[], &[1, 2, 3]));
        assert!(is_subset_sorted(&[2], &[1, 2, 3]));
        assert!(!is_subset_sorted(&[1, 4], &[1, 2, 3]));
        assert!(!is_subset_sorted(&[1, 2, 3], &[1, 2]));
    }

    #[test]
    fn binary_search_sorted_works() {
        assert_eq!(binary_search_sorted(&[1, 2, 3, 5], &3), Some(2));
        assert_eq!(binary_search_sorted(&[1, 2, 3, 5], &4), None);
        assert_eq!(binary_search_sorted::<i32>(&[], &1), None);
    }
}

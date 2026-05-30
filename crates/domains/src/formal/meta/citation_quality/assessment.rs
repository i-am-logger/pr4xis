//! Runtime citation-quality assessment — per-dimension outcomes and
//! their composition into a single verdict.
//!
//! The [`super::ontology`] module names the *dimensions* a citation is
//! judged on and their severity. This module is the runtime layer on
//! top: for a given citation you record a [`DimensionStatus`] per
//! dimension, and [`assess`] folds those into a [`CitationVerdict`].
//!
//! The fold is a **meet over a three-element bounded chain**
//! `Valid > ValidWithIssues > Invalid`: the overall verdict is the
//! worst per-dimension outcome. A defect or unconfirmed status on a
//! *blocking* dimension (existence / claim-support — see
//! [`super::ontology::is_sound_gate`]) drives the verdict to `Invalid`;
//! on a non-blocking dimension it only drives it to `ValidWithIssues`.
//! This is what lets Praxis say "valid, with these issues" instead of
//! collapsing every imperfection to pass/fail.
//!
//! # Literature
//!
//! - **Davey, B. A. & Priestley, H. A. (2002)** *Introduction to
//!   Lattices and Order*, 2nd ed., Cambridge University Press —
//!   §1–§2: meet-semilattices and bounded lattices. The verdict fold
//!   is the meet of a bounded chain with top `Valid`.
//! - **Daubert v. Merrell Dow Pharmaceuticals, Inc.**, 509 U.S. 579
//!   (1993) — evidentiary reliability factors (testability /
//!   reproducibility / known error rate). Grounds the
//!   [`VerificationMethod`] strength order: a machine-checked result
//!   is reproducible by anyone re-running it (highest reliability), a
//!   human attestation depends on the attester (lower), an unverified
//!   claim has no reliability basis (lowest).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Monoid;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::ontology::{CitationQualityConcept, is_sound_gate};

// ---------------------------------------------------------------------------
// VerificationMethod — how a dimension's status was established.
// ---------------------------------------------------------------------------

/// How the status of a citation-quality dimension was established. The
/// ordering reflects evidentiary reliability (Daubert 1993): a
/// machine-checked result is reproducible by re-running it; a human
/// attestation depends on the attester; an unverified claim has no
/// basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationMethod {
    /// No verification has been performed.
    Unverified,
    /// A person inspected the cited work and attested to the status.
    HumanAttested,
    /// A machine check (e.g. byte-level quote match against the loaded
    /// source) established the status, reproducibly.
    MachineChecked,
}

impl VerificationMethod {
    /// Reliability strength, ascending. `Unverified` = 0.
    pub fn strength(self) -> u8 {
        match self {
            VerificationMethod::Unverified => 0,
            VerificationMethod::HumanAttested => 1,
            VerificationMethod::MachineChecked => 2,
        }
    }
}

// ---------------------------------------------------------------------------
// DimensionStatus — the per-dimension outcome.
// ---------------------------------------------------------------------------

/// The outcome recorded for a single citation-quality dimension.
///
/// Kept deliberately small: a dimension is either confirmed good or
/// not yet confirmed. Typed defects (wrong section, misquote,
/// reference error, style break) — which would distinguish "confirmed
/// flawed" from "not yet checked" — layer on later; until then an
/// unconfirmed dimension is the only non-`Verified` state, and the
/// verdict treats it conservatively (a blocking dimension that is not
/// `Verified` makes the citation `Invalid`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionStatus {
    /// The dimension was checked and is sound.
    Verified,
    /// The dimension has not been confirmed.
    Unverified,
}

// ---------------------------------------------------------------------------
// CitationVerdict — the composed overall outcome (bounded chain).
// ---------------------------------------------------------------------------

/// The overall verdict for a citation, composed by meet over its
/// dimensions. A bounded chain with top `Valid` and bottom `Invalid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CitationVerdict {
    /// Every dimension is `Verified`.
    Valid,
    /// Every blocking dimension is `Verified`, but at least one
    /// non-blocking dimension is unconfirmed. The citation can be
    /// relied on; the issues are recorded.
    ValidWithIssues,
    /// At least one blocking (sound-gate) dimension is not `Verified`.
    /// The citation cannot be relied on.
    Invalid,
}

impl CitationVerdict {
    /// Rank in the chain: `Valid` = 2 (top), `Invalid` = 0 (bottom).
    fn rank(self) -> u8 {
        match self {
            CitationVerdict::Valid => 2,
            CitationVerdict::ValidWithIssues => 1,
            CitationVerdict::Invalid => 0,
        }
    }

    fn from_rank(r: u8) -> CitationVerdict {
        match r {
            2 => CitationVerdict::Valid,
            1 => CitationVerdict::ValidWithIssues,
            _ => CitationVerdict::Invalid,
        }
    }

    /// The meet (greatest lower bound) of two verdicts: the worse of
    /// the two. `Valid` is the identity (top). Davey & Priestley
    /// (2002) §2 — meet of a bounded chain.
    pub fn meet(self, other: CitationVerdict) -> CitationVerdict {
        CitationVerdict::from_rank(self.rank().min(other.rank()))
    }

    /// The top of the chain — the meet identity.
    pub const TOP: CitationVerdict = CitationVerdict::Valid;
}

/// `CitationVerdict` is an **idempotent commutative monoid** under
/// `meet`, with identity `Valid` (the chain top). Composing the
/// per-dimension verdicts is therefore a monoid reduction — the same
/// algebraic shape Praxis uses for trace accumulation and morphism
/// composition (Mac Lane 1971, Ch. VII). Idempotence and commutativity
/// (which strengthen it from a monoid to a bounded meet-semilattice)
/// are checked separately by [`VerdictMeetIsBoundedSemilattice`].
impl Monoid for CitationVerdict {
    fn empty() -> Self {
        CitationVerdict::TOP
    }

    fn combine(&self, other: &Self) -> Self {
        self.meet(*other)
    }
}

/// The verdict contributed by one dimension given its status. A
/// blocking dimension that is not `Verified` contributes `Invalid`; a
/// non-blocking one contributes `ValidWithIssues`; a `Verified`
/// dimension contributes `Valid` (the identity).
pub fn dimension_verdict(dim: CitationQualityConcept, status: DimensionStatus) -> CitationVerdict {
    match status {
        DimensionStatus::Verified => CitationVerdict::Valid,
        DimensionStatus::Unverified => {
            if is_sound_gate(dim) {
                CitationVerdict::Invalid
            } else {
                CitationVerdict::ValidWithIssues
            }
        }
    }
}

/// Compose per-dimension statuses into an overall verdict — the monoid
/// reduction of the per-dimension verdicts under `combine` (meet). An
/// empty assessment yields `Valid` (the monoid identity / chain top).
pub fn assess(statuses: &[(CitationQualityConcept, DimensionStatus)]) -> CitationVerdict {
    statuses
        .iter()
        .map(|&(dim, status)| dimension_verdict(dim, status))
        .fold(CitationVerdict::empty(), |acc, v| acc.combine(&v))
}

// ---------------------------------------------------------------------------
// Axiom — the verdict fold is a bounded meet-semilattice.
// ---------------------------------------------------------------------------

/// Axiom: `CitationVerdict::meet` is a bounded meet-semilattice —
/// idempotent, commutative, associative, with `Valid` as identity
/// (top). Verified exhaustively over the three-element carrier.
pub struct VerdictMeetIsBoundedSemilattice;

impl Axiom for VerdictMeetIsBoundedSemilattice {
    fn verify(&self) -> Verdict {
        use CitationVerdict as V;
        let all = [V::Valid, V::ValidWithIssues, V::Invalid];

        for a in all {
            // Idempotent.
            if a.meet(a) != a {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            // Identity (Valid is top).
            if V::TOP.meet(a) != a || a.meet(V::TOP) != a {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            for b in all {
                // Commutative.
                if a.meet(b) != b.meet(a) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
                for c in all {
                    // Associative.
                    if a.meet(b).meet(c) != a.meet(b.meet(c)) {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "VerdictMeetIsBoundedSemilattice",
        "CitationVerdict::meet is idempotent, commutative, associative, with Valid as identity (top)",
        "Davey & Priestley (2002) Introduction to Lattices and Order, 2nd ed., CUP, §2 (meet-semilattices)"
    );
}

pr4xis::register_axiom!(
    VerdictMeetIsBoundedSemilattice,
    "Davey & Priestley (2002) Introduction to Lattices and Order, 2nd ed., CUP"
);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use CitationQualityConcept as C;

    #[test]
    fn semilattice_axiom_holds() {
        assert!(VerdictMeetIsBoundedSemilattice.verify().is_ok());
    }

    #[test]
    fn verification_method_strength_is_ordered() {
        assert!(
            VerificationMethod::MachineChecked.strength()
                > VerificationMethod::HumanAttested.strength()
        );
        assert!(
            VerificationMethod::HumanAttested.strength()
                > VerificationMethod::Unverified.strength()
        );
        // Derived Ord agrees with the strength order.
        assert!(VerificationMethod::MachineChecked > VerificationMethod::HumanAttested);
        assert!(VerificationMethod::HumanAttested > VerificationMethod::Unverified);
    }

    #[test]
    fn all_verified_is_valid() {
        let statuses = [
            (C::Existence, DimensionStatus::Verified),
            (C::ClaimSupport, DimensionStatus::Verified),
            (C::LocatorAccuracy, DimensionStatus::Verified),
            (C::BibliographicAccuracy, DimensionStatus::Verified),
            (C::FormatConformance, DimensionStatus::Verified),
        ];
        assert_eq!(assess(&statuses), CitationVerdict::Valid);
    }

    #[test]
    fn unconfirmed_blocking_dimension_is_invalid() {
        // Claim support unconfirmed → Invalid, regardless of the rest.
        let statuses = [
            (C::Existence, DimensionStatus::Verified),
            (C::ClaimSupport, DimensionStatus::Unverified),
            (C::LocatorAccuracy, DimensionStatus::Verified),
        ];
        assert_eq!(assess(&statuses), CitationVerdict::Invalid);
    }

    #[test]
    fn unconfirmed_nonblocking_dimension_is_valid_with_issues() {
        // Sound gate intact, locator unconfirmed → ValidWithIssues.
        let statuses = [
            (C::Existence, DimensionStatus::Verified),
            (C::ClaimSupport, DimensionStatus::Verified),
            (C::LocatorAccuracy, DimensionStatus::Unverified),
            (C::BibliographicAccuracy, DimensionStatus::Unverified),
        ];
        assert_eq!(assess(&statuses), CitationVerdict::ValidWithIssues);
    }

    #[test]
    fn blocking_failure_dominates_nonblocking() {
        // Both a blocking and a non-blocking gap → the blocking one wins.
        let statuses = [
            (C::Existence, DimensionStatus::Unverified),
            (C::LocatorAccuracy, DimensionStatus::Unverified),
        ];
        assert_eq!(assess(&statuses), CitationVerdict::Invalid);
    }

    #[test]
    fn empty_assessment_is_top() {
        assert_eq!(assess(&[]), CitationVerdict::Valid);
    }

    #[test]
    fn verdict_is_a_monoid() {
        use CitationVerdict as V;
        let all = [V::Valid, V::ValidWithIssues, V::Invalid];
        for a in all {
            // Identity (Valid is the monoid empty / chain top).
            assert_eq!(V::empty().combine(&a), a);
            assert_eq!(a.combine(&V::empty()), a);
            for b in all {
                for c in all {
                    // Associativity.
                    assert_eq!(a.combine(&b).combine(&c), a.combine(&b.combine(&c)));
                }
            }
        }
    }

    #[test]
    fn meet_is_worst_of_two() {
        assert_eq!(
            CitationVerdict::Valid.meet(CitationVerdict::ValidWithIssues),
            CitationVerdict::ValidWithIssues
        );
        assert_eq!(
            CitationVerdict::ValidWithIssues.meet(CitationVerdict::Invalid),
            CitationVerdict::Invalid
        );
        assert_eq!(
            CitationVerdict::Valid.meet(CitationVerdict::Invalid),
            CitationVerdict::Invalid
        );
    }

    // ── Property-based laws ────────────────────────────────────────
    use super::super::ontology::dimensions;
    use proptest::prelude::*;

    fn arb_verdict() -> impl Strategy<Value = CitationVerdict> {
        prop_oneof![
            Just(CitationVerdict::Valid),
            Just(CitationVerdict::ValidWithIssues),
            Just(CitationVerdict::Invalid),
        ]
    }

    fn arb_method() -> impl Strategy<Value = VerificationMethod> {
        prop_oneof![
            Just(VerificationMethod::Unverified),
            Just(VerificationMethod::HumanAttested),
            Just(VerificationMethod::MachineChecked),
        ]
    }

    fn arb_status() -> impl Strategy<Value = DimensionStatus> {
        prop_oneof![
            Just(DimensionStatus::Verified),
            Just(DimensionStatus::Unverified),
        ]
    }

    fn arb_dim_status() -> impl Strategy<Value = (CitationQualityConcept, DimensionStatus)> {
        (
            proptest::sample::select(dimensions().to_vec()),
            arb_status(),
        )
    }

    proptest! {
        /// Meet is commutative (bounded meet-semilattice; Davey &
        /// Priestley 2002 §2).
        #[test]
        fn prop_meet_commutative(a in arb_verdict(), b in arb_verdict()) {
            prop_assert_eq!(a.meet(b), b.meet(a));
        }

        /// Meet is associative.
        #[test]
        fn prop_meet_associative(a in arb_verdict(), b in arb_verdict(), c in arb_verdict()) {
            prop_assert_eq!(a.meet(b).meet(c), a.meet(b.meet(c)));
        }

        /// Idempotence + the monoid identity law (Valid is the top).
        #[test]
        fn prop_meet_idempotent_and_identity(a in arb_verdict()) {
            prop_assert_eq!(a.meet(a), a);
            prop_assert_eq!(CitationVerdict::empty().combine(&a), a);
            prop_assert_eq!(a.combine(&CitationVerdict::empty()), a);
        }

        /// The defining gate characterization: a verdict is Invalid iff
        /// some blocking (sound-gate) dimension is unconfirmed; else
        /// ValidWithIssues iff some non-blocking dimension is unconfirmed;
        /// else Valid.
        #[test]
        fn prop_assess_characterization(
            statuses in proptest::collection::vec(arb_dim_status(), 0..12)
        ) {
            let blocking_gap = statuses
                .iter()
                .any(|(d, s)| is_sound_gate(*d) && *s == DimensionStatus::Unverified);
            let nonblocking_gap = statuses
                .iter()
                .any(|(d, s)| !is_sound_gate(*d) && *s == DimensionStatus::Unverified);
            let expected = if blocking_gap {
                CitationVerdict::Invalid
            } else if nonblocking_gap {
                CitationVerdict::ValidWithIssues
            } else {
                CitationVerdict::Valid
            };
            prop_assert_eq!(assess(&statuses), expected);
        }

        /// The verification-method strength order agrees with the derived
        /// `Ord` (Daubert reliability ordering).
        #[test]
        fn prop_method_strength_matches_ord(a in arb_method(), b in arb_method()) {
            prop_assert_eq!(a.strength().cmp(&b.strength()), a.cmp(&b));
        }
    }
}

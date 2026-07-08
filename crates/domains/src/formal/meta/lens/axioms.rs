//! Runnable axioms of the [`Lens`](super::ontology) ontology — each `verify()`
//! is a *predicate that runs*, exercising the real general-lens machinery
//! ([`crate::formal::meta::lens_composition`]) rather than asserting a
//! doc-comment (North Star W3 slice 4;
//! `feedback_praxis_as_compiler_self_describing`).
//!
//! These run in EVERY build: the general [`Lens`] trait and its law checkers
//! ([`get_put_holds`], [`put_get_holds`], [`put_put_holds`]) are unconditional
//! domains code (contrast [`super::super::ontology_archive::axioms`], gated on
//! `feature = "prx"`).
//!
//! Each axiom verifies a genuinely-uncovered lens law over REAL lens values
//! and has teeth — a wrong lens fails it:
//!
//! - The three well-behaved-lens laws (Foster et al. 2007 §2.2) —
//!   [`LensGetPutLaw`], [`LensPutGetLaw`], [`LensPutPutLaw`] — hold over the
//!   well-behaved witnesses AND reject a deliberately mis-paired [`Broken`]
//!   lens; [`LensPutPutLaw`] additionally proves the well-behaved /
//!   very-well-behaved boundary by falsifying [`StashingLens`], a lens that
//!   obeys GetPut + PutGet yet violates PutPut.
//! - The three category laws (Foster et al. 2007 §3, lenses form a category) —
//!   [`LensCompositionWellBehaved`] (composition preserves well-behavedness,
//!   with a broken composite rejected), [`LensCompositionAssociative`], and
//!   [`LensIdentityUnit`].
//!
//! The byte-anchored [`WellBehavedLens`](crate::formal::meta::well_behaved_lens::WellBehavedLens)
//! PutGet law is deliberately NOT restated here: it is owned by, and verified
//! against the real on-disk sources in,
//! [`RoundTripHarnessAllVerified`](crate::formal::meta::well_behaved_lens::RoundTripHarnessAllVerified),
//! and the archive emit/load GetPut leg by
//! `EmitLoadWellBehaved`. The Lens ontology's discoverability test resolves
//! those through the same registry rather than re-running them.
//!
//! # Literature
//!
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators for
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2 (the lens
//!   laws; well-behaved vs very-well-behaved) and §3 (composition, identity,
//!   the lens category).

use alloc::boxed::Box;
use core::convert::Infallible;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use crate::formal::meta::lens_composition::{
    Compose, IdentityLens, Lens, get_put_holds, put_get_holds, put_put_holds,
};

// =============================================================================
// Witness lenses — REAL `Lens` impls the axioms verify over.
// =============================================================================

/// A two-field record — the source of the [`Fst`] / [`StashingLens`] witnesses.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Pair {
    a: i32,
    b: i32,
}

/// `fst : Pair ⇆ i32` — focuses the `a` field. A very-well-behaved lens
/// (satisfies GetPut, PutGet, and PutPut).
struct Fst;
impl Lens for Fst {
    type Source = Pair;
    type View = i32;
    type Error = Infallible;
    fn get(&self, s: &Pair) -> Result<i32, Infallible> {
        Ok(s.a)
    }
    fn put(&self, v: &i32, s: &Pair) -> Result<Pair, Infallible> {
        Ok(Pair { a: *v, b: s.b })
    }
}

/// `affine : i32 ⇆ i32` — view is `x + 1`; put recovers `v - 1`. Very
/// well-behaved.
struct PlusOne;
impl Lens for PlusOne {
    type Source = i32;
    type View = i32;
    type Error = Infallible;
    fn get(&self, s: &i32) -> Result<i32, Infallible> {
        Ok(s + 1)
    }
    fn put(&self, v: &i32, _s: &i32) -> Result<i32, Infallible> {
        Ok(v - 1)
    }
}

/// A WELL-BEHAVED but NOT very-well-behaved lens `Pair ⇆ i32` (Foster et al.
/// 2007 §2.2): `get` focuses `a`; `put` writes the new view into `a` and, ONLY
/// when the view actually changes, stashes the OLD view into `b` (a constant
/// complement, Bancilhon & Spyratos 1981).
///
/// - GetPut holds: putting the current view back (`v == s.a`) is a no-op.
/// - PutGet holds: `get(put(v, s)) == v` in both branches.
/// - PutPut FAILS: `put(v', put(v, s)) != put(v', s)` once `v` differs from
///   `s.a`, because the intermediate `put` moved the old view into `b`, which a
///   later `put` then stashes forward. This is the witness that proves the
///   [`LensPutPutLaw`] axiom distinguishes well-behaved from very-well-behaved.
struct StashingLens;
impl Lens for StashingLens {
    type Source = Pair;
    type View = i32;
    type Error = Infallible;
    fn get(&self, s: &Pair) -> Result<i32, Infallible> {
        Ok(s.a)
    }
    fn put(&self, v: &i32, s: &Pair) -> Result<Pair, Infallible> {
        if *v == s.a {
            Ok(s.clone())
        } else {
            Ok(Pair { a: *v, b: s.a })
        }
    }
}

/// A deliberately MIS-PAIRED lens `i32 ⇆ i32` — `get` is the identity but `put`
/// adds a constant, so `get`/`put` are not inverse and BOTH GetPut and PutGet
/// fail. The teeth: every well-behaved-lens-law axiom rejects it, so the axioms
/// can FAIL on a real defect, not just pass.
struct Broken;
impl Lens for Broken {
    type Source = i32;
    type View = i32;
    type Error = Infallible;
    fn get(&self, s: &i32) -> Result<i32, Infallible> {
        Ok(*s)
    }
    fn put(&self, v: &i32, _s: &i32) -> Result<i32, Infallible> {
        Ok(v + 100)
    }
}

/// The canonical witness source shared by the axioms.
fn pair() -> Pair {
    Pair { a: 3, b: 7 }
}

// =============================================================================
// Well-behaved-lens laws (Foster et al. 2007 §2.2).
// =============================================================================

/// GetPut (Foster et al. 2007 §2.2): `put(get(s), s) == s`. Holds over the
/// well-behaved witnesses (identity, `Fst`, `PlusOne`, `StashingLens`, and a
/// composite) AND is FALSIFIED by the mis-paired [`Broken`] lens — so the law
/// is non-vacuous.
pub struct LensGetPutLaw;

impl Axiom for LensGetPutLaw {
    fn verify(&self) -> Verdict {
        let s = pair();
        let holds = get_put_holds(&IdentityLens::<Pair>::new(), &s)
            && get_put_holds(&Fst, &s)
            && get_put_holds(&PlusOne, &5)
            && get_put_holds(&StashingLens, &s)
            && get_put_holds(&Compose::new(Fst, PlusOne), &s);
        // Teeth: a mis-paired lens must be rejected, not silently pass.
        let broken_rejected = !get_put_holds(&Broken, &5);
        if holds && broken_rejected {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LensGetPutLaw",
        "put(get(s), s) == s over the well-behaved witness lenses, and a mis-paired lens is rejected",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(LensGetPutLaw, constructor);

/// PutGet (Foster et al. 2007 §2.2): `get(put(v, s)) == v`. Holds over the
/// well-behaved witnesses AND is FALSIFIED by the mis-paired [`Broken`] lens.
pub struct LensPutGetLaw;

impl Axiom for LensPutGetLaw {
    fn verify(&self) -> Verdict {
        let s = pair();
        let holds = put_get_holds(&IdentityLens::<Pair>::new(), &Pair { a: 9, b: 9 }, &s)
            && put_get_holds(&Fst, &10, &s)
            && put_get_holds(&PlusOne, &9, &5)
            && put_get_holds(&StashingLens, &10, &s)
            && put_get_holds(&Compose::new(Fst, PlusOne), &11, &s);
        let broken_rejected = !put_get_holds(&Broken, &9, &5);
        if holds && broken_rejected {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LensPutGetLaw",
        "get(put(v, s)) == v over the well-behaved witness lenses, and a mis-paired lens is rejected",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(LensPutGetLaw, constructor);

/// PutPut (Foster et al. 2007 §2.2): `put(v', put(v, s)) == put(v', s)` — the
/// extra law of a VERY well-behaved lens. Holds over the very-well-behaved
/// witnesses (identity, `Fst`, `PlusOne`, and a composite) AND is FALSIFIED by
/// [`StashingLens`], which obeys GetPut + PutGet yet not PutPut — so the axiom
/// genuinely distinguishes well-behaved from very-well-behaved (Foster's own
/// distinction), not a tautology.
pub struct LensPutPutLaw;

impl Axiom for LensPutPutLaw {
    fn verify(&self) -> Verdict {
        let s = pair();
        let very_well_behaved = put_put_holds(
            &IdentityLens::<Pair>::new(),
            &Pair { a: 9, b: 9 },
            &Pair { a: 8, b: 8 },
            &s,
        ) && put_put_holds(&Fst, &10, &20, &s)
            && put_put_holds(&PlusOne, &9, &12, &5)
            && put_put_holds(&Compose::new(Fst, PlusOne), &11, &21, &s);
        // Teeth + the very-well-behaved boundary: a well-behaved-but-not-very-
        // well-behaved lens must FAIL PutPut (v1 = 5 ≠ s.a = 3, v2 = 9).
        let stashing_not_very_well_behaved = !put_put_holds(&StashingLens, &5, &9, &s);
        if very_well_behaved && stashing_not_very_well_behaved {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LensPutPutLaw",
        "put(v', put(v, s)) == put(v', s) over the very-well-behaved witnesses, and a well-behaved-but-not-very-well-behaved lens is falsified",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(LensPutPutLaw, constructor);

// =============================================================================
// The lens category (Foster et al. 2007 §3).
// =============================================================================

/// Composition preserves well-behavedness (Foster et al. 2007 §3): the
/// composite `Fst ; PlusOne` of two well-behaved lenses is itself well-behaved
/// (satisfies GetPut and PutGet). Non-vacuous: composing with the mis-paired
/// [`Broken`] lens yields a composite that is REJECTED, so the axiom is not the
/// trivial "every composite is fine".
pub struct LensCompositionWellBehaved;

impl Axiom for LensCompositionWellBehaved {
    fn verify(&self) -> Verdict {
        let s = pair();
        let composite = Compose::new(Fst, PlusOne);
        let preserves = get_put_holds(&composite, &s) && put_get_holds(&composite, &11, &s);
        // Teeth: a composite through a broken lens is not well-behaved.
        let broken_composite = Compose::new(Fst, Broken);
        let broken_detected = !get_put_holds(&broken_composite, &s);
        if preserves && broken_detected {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LensCompositionWellBehaved",
        "the sequential composite of two well-behaved lenses is well-behaved; a composite through a mis-paired lens is not",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3"
    );
}

pr4xis::register_axiom!(LensCompositionWellBehaved, constructor);

/// Sequential composition is associative (Foster et al. 2007 §3, lenses form a
/// category): `(Fst ; PlusOne) ; PlusOne` and `Fst ; (PlusOne ; PlusOne)` agree
/// on both `get` and `put` over the witness source. A `Compose` whose `put`
/// combinator was wired wrong would disagree here — this exercises the real
/// composition, not `f == f`.
pub struct LensCompositionAssociative;

impl Axiom for LensCompositionAssociative {
    fn verify(&self) -> Verdict {
        let s = pair();
        let left = Compose::new(Compose::new(Fst, PlusOne), PlusOne);
        let right = Compose::new(Fst, Compose::new(PlusOne, PlusOne));
        // Both composites' `Error` is uninhabited (a `ComposeError` over
        // `Infallible` legs), so these `Ok` patterns are irrefutable.
        let (Ok(lg), Ok(rg)) = (left.get(&s), right.get(&s));
        let (Ok(lp), Ok(rp)) = (left.put(&12, &s), right.put(&12, &s));
        if lg == rg && lp == rp {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LensCompositionAssociative",
        "(l ; k) ; m and l ; (k ; m) agree on get and put — sequential composition is associative",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3"
    );
}

pr4xis::register_axiom!(LensCompositionAssociative, constructor);

/// The identity lens is the unit of composition (Foster et al. 2007 §3):
/// `id ; Fst` and `Fst ; id` both behave as `Fst` on `get` and `put`.
pub struct LensIdentityUnit;

impl Axiom for LensIdentityUnit {
    fn verify(&self) -> Verdict {
        let s = pair();
        let left = Compose::new(IdentityLens::<Pair>::new(), Fst);
        let right = Compose::new(Fst, IdentityLens::<i32>::new());
        // Every leg is `Infallible`, so these `Ok` patterns are irrefutable.
        let (Ok(bare_g), Ok(lg), Ok(rg)) = (Fst.get(&s), left.get(&s), right.get(&s));
        let (Ok(bare_p), Ok(lp), Ok(rp)) =
            (Fst.put(&10, &s), left.put(&10, &s), right.put(&10, &s));
        if lg == bare_g && rg == bare_g && lp == bare_p && rp == bare_p {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LensIdentityUnit",
        "id ; l and l ; id both behave as l — the identity lens is the composition unit",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3"
    );
}

pr4xis::register_axiom!(LensIdentityUnit, constructor);

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn all_lens_axioms_hold() {
        assert!(LensGetPutLaw.verify().is_ok());
        assert!(LensPutGetLaw.verify().is_ok());
        assert!(LensPutPutLaw.verify().is_ok());
        assert!(LensCompositionWellBehaved.verify().is_ok());
        assert!(LensCompositionAssociative.verify().is_ok());
        assert!(LensIdentityUnit.verify().is_ok());
    }

    /// Teeth for the well-behaved-lens laws: the mis-paired [`Broken`] lens
    /// violates both GetPut and PutGet, so the law checkers reject it — the
    /// axioms can FAIL on a real defect, not just pass.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn broken_lens_violates_get_put_and_put_get() {
        assert!(!get_put_holds(&Broken, &5), "Broken must fail GetPut");
        assert!(!put_get_holds(&Broken, &9, &5), "Broken must fail PutGet");
    }

    /// Teeth for the very-well-behaved boundary: [`StashingLens`] is
    /// well-behaved (GetPut + PutGet) yet NOT very-well-behaved (PutPut fails),
    /// so [`LensPutPutLaw`] genuinely separates the two classes.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn stashing_lens_is_well_behaved_but_not_very_well_behaved() {
        let s = pair();
        assert!(
            get_put_holds(&StashingLens, &s),
            "StashingLens obeys GetPut"
        );
        assert!(
            put_get_holds(&StashingLens, &10, &s),
            "StashingLens obeys PutGet"
        );
        assert!(
            !put_put_holds(&StashingLens, &5, &9, &s),
            "StashingLens must violate PutPut (well-behaved, not very-well-behaved)"
        );
    }
}

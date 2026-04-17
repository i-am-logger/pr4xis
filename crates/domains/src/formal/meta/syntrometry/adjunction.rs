//! Informal adjunction: Syntrometry ⊣ Pr4xisSubstrate.
//!
//! A strict [`pr4xis::category::Adjunction`] impl is not reachable under
//! the current category structures — the reverse direction would require
//! a dense-source → kinded-target strict `Functor`, which the #98 research
//! note (`docs/research/kinded-functor-failures.md`) established is
//! impossible. What IS well-defined, and what paper 02's gap-detection
//! methodology actually needs, are the **object-level unit and counit
//! round-trips**:
//!
//! - `unit(A)`  = `A → G(F(A))` as a pair of syntrometric concepts
//! - `counit(B)` = `F(G(B)) → B` as a pair of substrate concepts
//!
//! Both are computed from the forward functor's object map
//! ([`super::lineage_functor::SyntrometryToPr4xisSubstrate::map_object`])
//! and the reverse object map
//! ([`super::substrate_functor::map_substrate`]). The gap analysis
//! in [`crate::formal::meta::gap_analysis::analyze_syntrometry_substrate`]
//! uses these directly.
//!
//! Concepts that *don't* round-trip to themselves are genuine missing
//! distinctions in the pr4xis substrate relative to Heim's vocabulary.
//! Expected collapses, by construction of the forward map:
//!
//! - `SyntrixLevel` → `SubEntity` → `Predicate` (level-of-Syntrix vs predicate-as-atom collapsed)
//! - `Part` → `SubMorphism` → `Koordination` (mereology-as-morphism vs ordering collapsed)
//! - `Dialektik`, `Aspekt` → `SubCategory` → `Syntrix` (opposition / product / leveled all collapsed to category)

use pr4xis::category::Functor;

use super::lineage_functor::SyntrometryToPr4xisSubstrate;
use super::ontology::SyntrometryConcept;
use super::substrate::Pr4xisSubstrateConcept;
use super::substrate_functor::map_substrate;

/// The unit round-trip for `A`: `(A, G(F(A)))`.
pub fn unit_pair(obj: &SyntrometryConcept) -> (SyntrometryConcept, SyntrometryConcept) {
    let round_trip = map_substrate(&SyntrometryToPr4xisSubstrate::map_object(obj));
    (*obj, round_trip)
}

/// The counit round-trip for `B`: `(F(G(B)), B)`.
pub fn counit_pair(
    obj: &Pr4xisSubstrateConcept,
) -> (Pr4xisSubstrateConcept, Pr4xisSubstrateConcept) {
    let round_trip = SyntrometryToPr4xisSubstrate::map_object(&map_substrate(obj));
    (round_trip, *obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;

    /// Every counit round-trip lands exactly at its target. (The substrate
    /// set is closed under the object mapping, so every substrate primitive
    /// is in the image of the forward functor.)
    #[test]
    fn counit_is_identity_on_substrate() {
        for obj in Pr4xisSubstrateConcept::variants() {
            let (rt, target) = counit_pair(&obj);
            assert_eq!(target, obj);
            assert_eq!(
                rt, obj,
                "every substrate primitive should round-trip back to itself (pr4xis is at least as expressive as its own substrate)"
            );
        }
    }

    /// Exactly four syntrometric concepts intentionally collapse in the
    /// primary substrate functor:
    /// - `Dialektik` → opposition-structure lives in Dialectics.
    /// - `SequencePermutation`, `OrientationPermutation` → both are
    ///   endomorphisms; substrate collapses them with Synkolator.
    /// - `Aspektivsystem` → a predicate-system-of-aspects; substrate
    ///   collapses it with Predikatrix.
    /// Richer distinctions live in the Syntrometry ontology itself and
    /// in the Dialectics cross-functor.
    #[test]
    fn unit_collapses_are_intentional() {
        use SyntrometryConcept as S;
        let collapses: Vec<_> = SyntrometryConcept::variants()
            .into_iter()
            .filter(|obj| {
                let (source, rt) = unit_pair(obj);
                source != rt
            })
            .collect();
        let expected = [
            S::Dialektik,
            S::SequencePermutation,
            S::OrientationPermutation,
            S::Aspektivsystem,
        ];
        assert_eq!(
            collapses.len(),
            expected.len(),
            "unexpected collapses: {:?}",
            collapses
        );
        for e in &expected {
            assert!(
                collapses.contains(e),
                "expected {:?} in collapses; got {:?}",
                e,
                collapses
            );
        }
    }

    /// 14 of the 18 concepts round-trip as fixed points.
    #[test]
    fn unit_preserves_fourteen_concepts() {
        let preserved: Vec<_> = SyntrometryConcept::variants()
            .into_iter()
            .filter(|obj| {
                let (source, rt) = unit_pair(obj);
                source == rt
            })
            .collect();
        assert_eq!(preserved.len(), 14);
    }
}

//! Tests for [`super::CausationToDerivation`]. Three layers per
//! `feedback_high_test_coverage`:
//!
//! 1. **Functor-law tests** — Mac Lane (1971) §I.3 identity and
//!    composition preservation, via `assert_functor_laws`.
//! 2. **Axiom tests** — every axiom registered in
//!    `derivation_functor.rs` is verified directly.
//! 3. **Property-based tests** — proptest-driven invariants over the
//!    full Causation concept inventory (object map total + Derivation
//!    component-class respected) and the relation-kind dispatch.

use proptest::prelude::*;

use super::super::ontology::{CausationConcept, CausationRelation, CausationRelationKind};
use super::{
    CausalGraphIsComposedAbduction, CausationToDerivation, CauseIsAbductiveConclusion,
    CounterfactualDependenceGroundsConclusion, EffectIsAbductivePremise,
    InterventionIsInferenceRule,
};
use crate::formal::derivation::ontology::{DerivationConcept, DerivationRelationKind};
use pr4xis::category::{Functor, kinds::FunctorKind, laws::assert_functor_laws};
use pr4xis::ontology::Axiom;

// =============================================================================
// Layer 1 — structural functor laws (Mac Lane §I.3).
// =============================================================================

#[test]
fn functor_laws() {
    assert_functor_laws::<CausationToDerivation>();
}

#[test]
fn kind_is_forgetful() {
    // Five `Cause` subtypes (Sufficient/Necessary/Proximate/Distal/
    // Common) collapse to the same `Conclusion` — classic forgetful
    // shape (Mac Lane §I.4).
    assert_eq!(CausationToDerivation::KIND, FunctorKind::Forgetful);
}

#[test]
fn meta_has_literature_citation() {
    let meta = CausationToDerivation::meta();
    let citation = meta.citation.as_str();
    // Smoke-check: every cited author the doc-string promises actually
    // appears in the citation field. `feedback_literature_or_remove`.
    for needle in ["Peirce", "Lewis", "Pearl", "Schurz", "Mac Lane"] {
        assert!(
            citation.contains(needle),
            "expected citation to mention {needle}; got: {citation}"
        );
    }
}

// =============================================================================
// Layer 2 — axiom verification. Each domain axiom in the functor's
// source file is checked directly.
// =============================================================================

#[test]
fn axiom_effect_is_abductive_premise() {
    assert!(EffectIsAbductivePremise.verify().is_ok());
}

#[test]
fn axiom_cause_is_abductive_conclusion() {
    assert!(CauseIsAbductiveConclusion.verify().is_ok());
}

#[test]
fn axiom_counterfactual_dependence_grounds_conclusion() {
    assert!(CounterfactualDependenceGroundsConclusion.verify().is_ok());
}

#[test]
fn axiom_intervention_is_inference_rule() {
    assert!(InterventionIsInferenceRule.verify().is_ok());
}

#[test]
fn axiom_causal_graph_is_composed_abduction() {
    assert!(CausalGraphIsComposedAbduction.verify().is_ok());
}

// =============================================================================
// Layer 3 — pinpoint object-map cases. Direct table-test of every
// projection promised by the module-level docs.
// =============================================================================

#[test]
fn effect_projects_to_premise() {
    assert_eq!(
        CausationToDerivation::map_object(&CausationConcept::Effect),
        DerivationConcept::Premise
    );
}

#[test]
fn all_cause_subtypes_collapse_to_conclusion() {
    use CausationConcept as C;
    for cause_kind in [
        C::Cause,
        C::SufficientCause,
        C::NecessaryCause,
        C::ProximateCause,
        C::DistalCause,
        C::CommonCause,
    ] {
        assert_eq!(
            CausationToDerivation::map_object(&cause_kind),
            DerivationConcept::Conclusion,
            "{cause_kind:?} should project to Conclusion (forgetful collapse)"
        );
    }
}

#[test]
fn counterfactual_pair_projects_to_justification() {
    use CausationConcept as C;
    for c in [C::Counterfactual, C::CounterfactualDependence] {
        assert_eq!(
            CausationToDerivation::map_object(&c),
            DerivationConcept::Justification,
            "{c:?} should project to Justification (Lewis 1973 warrant role)"
        );
    }
}

#[test]
fn intervention_projects_to_inference_rule() {
    assert_eq!(
        CausationToDerivation::map_object(&CausationConcept::Intervention),
        DerivationConcept::InferenceRule
    );
}

#[test]
fn preemption_and_overdetermination_project_to_proof_step() {
    use CausationConcept as C;
    for c in [C::Preemption, C::Overdetermination, C::CausalChain] {
        assert_eq!(
            CausationToDerivation::map_object(&c),
            DerivationConcept::ProofStep,
            "{c:?} should project to ProofStep"
        );
    }
}

#[test]
fn causal_graph_projects_to_composition() {
    assert_eq!(
        CausationToDerivation::map_object(&CausationConcept::CausalGraph),
        DerivationConcept::Composition
    );
}

// =============================================================================
// Layer 3 — relation-kind dispatch.
// =============================================================================

#[test]
fn canonical_kinds_preserved() {
    use CausationRelationKind as S;
    use DerivationRelationKind as T;
    let cases = [
        (S::Identity, T::Identity),
        (S::Subsumption, T::Subsumption),
        (S::Parthood, T::Parthood),
        (S::Causation, T::Causation),
        (S::Opposition, T::Opposition),
    ];
    for (src, expected) in cases {
        let m = CausationRelation {
            from: CausationConcept::Cause,
            to: CausationConcept::Effect,
            kind: src,
        };
        assert_eq!(
            CausationToDerivation::map_morphism(&m).kind,
            expected,
            "canonical kind {src:?} should preserve to {expected:?}"
        );
    }
}

#[test]
fn custom_kinds_project_to_canonical() {
    use CausationRelationKind as S;
    use DerivationRelationKind as T;
    let cases = [
        // The forward causal arrow becomes the backward abductive arrow
        // (kept as Causation kind).
        (S::Produces, T::Causation),
        // Pearl's `do(X)`-action: InferenceRule acts on Conclusion causally.
        (S::ActsOn, T::Causation),
        // Lewis's counterfactual grounding warrants the conclusion.
        (S::Grounds, T::Causation),
        // A cause/effect participating in a chain ↦ part-of the proof step.
        (S::ParticipatesIn, T::Parthood),
        // A chain embedded in a graph ↦ step embedded in composition.
        (S::EmbedsIn, T::Parthood),
        // A pattern (Preemption / Overdetermination) involves a cause
        // ↦ proof step involves the conclusion causally.
        (S::Involves, T::Causation),
    ];
    for (src, expected) in cases {
        let m = CausationRelation {
            from: CausationConcept::Cause,
            to: CausationConcept::Effect,
            kind: src,
        };
        assert_eq!(
            CausationToDerivation::map_morphism(&m).kind,
            expected,
            "custom kind {src:?} should collapse to {expected:?}"
        );
    }
}

// =============================================================================
// Layer 3 — the abductive schema applied to the canonical
// (Cause, Effect, Produces) edge: end-to-end check.
// =============================================================================

#[test]
fn produces_edge_realises_abductive_schema() {
    // Peirce (1903) Lecture VII:
    //   1. observed: C (Effect)
    //   2. rule:     A → C  (Produces edge from Cause)
    //   3. conclude: A (Cause)
    let edge = CausationRelation {
        from: CausationConcept::Cause,
        to: CausationConcept::Effect,
        kind: CausationRelationKind::Produces,
    };
    let projected = CausationToDerivation::map_morphism(&edge);
    assert_eq!(projected.from, DerivationConcept::Conclusion);
    assert_eq!(projected.to, DerivationConcept::Premise);
    assert_eq!(projected.kind, DerivationRelationKind::Causation);
}

#[test]
fn intervention_acts_on_cause_realises_pearl_do() {
    // Pearl (2000) §3.4: do(X) acts on a cause to read off effects.
    // After projection: InferenceRule --Causation--> Conclusion.
    let edge = CausationRelation {
        from: CausationConcept::Intervention,
        to: CausationConcept::Cause,
        kind: CausationRelationKind::ActsOn,
    };
    let projected = CausationToDerivation::map_morphism(&edge);
    assert_eq!(projected.from, DerivationConcept::InferenceRule);
    assert_eq!(projected.to, DerivationConcept::Conclusion);
    assert_eq!(projected.kind, DerivationRelationKind::Causation);
}

#[test]
fn counterfactual_grounds_cause_realises_lewis() {
    // Lewis (1973): the counterfactual grounds the causal claim.
    // After projection: Justification --Causation--> Conclusion.
    let edge = CausationRelation {
        from: CausationConcept::CounterfactualDependence,
        to: CausationConcept::Cause,
        kind: CausationRelationKind::Grounds,
    };
    let projected = CausationToDerivation::map_morphism(&edge);
    assert_eq!(projected.from, DerivationConcept::Justification);
    assert_eq!(projected.to, DerivationConcept::Conclusion);
    assert_eq!(projected.kind, DerivationRelationKind::Causation);
}

// =============================================================================
// Layer 3 — property tests.
// =============================================================================

fn arb_causation_concept() -> impl Strategy<Value = CausationConcept> {
    use CausationConcept as C;
    prop_oneof![
        Just(C::Cause),
        Just(C::Effect),
        Just(C::SufficientCause),
        Just(C::NecessaryCause),
        Just(C::ProximateCause),
        Just(C::DistalCause),
        Just(C::CommonCause),
        Just(C::Counterfactual),
        Just(C::CounterfactualDependence),
        Just(C::Preemption),
        Just(C::Overdetermination),
        Just(C::Intervention),
        Just(C::CausalChain),
        Just(C::CausalGraph),
    ]
}

fn arb_causation_relation_kind() -> impl Strategy<Value = CausationRelationKind> {
    use CausationRelationKind as K;
    prop_oneof![
        Just(K::Identity),
        Just(K::Subsumption),
        Just(K::Parthood),
        Just(K::Causation),
        Just(K::Opposition),
        Just(K::Produces),
        Just(K::ActsOn),
        Just(K::Grounds),
        Just(K::ParticipatesIn),
        Just(K::EmbedsIn),
        Just(K::Involves),
    ]
}

/// Codomain-class invariant: every Causation concept maps to a
/// Derivation component, type, or pipeline-stage concept — never to
/// a logical property (Soundness / Completeness / Validity /
/// Decidability), which are *meta* claims that have no abductive
/// surface meaning.
fn derivation_concept_is_admitted_target(c: DerivationConcept) -> bool {
    use DerivationConcept as D;
    matches!(
        c,
        D::Premise
            | D::Conclusion
            | D::Justification
            | D::Evidence
            | D::InferenceRule
            | D::ProofStep
            | D::Composition
            | D::Abduction
            | D::DerivationComponent
            | D::DerivationType
    )
}

proptest! {
    /// Every causation concept maps to a derivation concept admitted
    /// as a target by the abductive-projection codomain.
    #[test]
    fn property_object_map_total_and_codomain_restricted(c in arb_causation_concept()) {
        let projected = CausationToDerivation::map_object(&c);
        prop_assert!(
            derivation_concept_is_admitted_target(projected),
            "{c:?} mapped to {projected:?}, which is not in the abductive-projection codomain"
        );
    }

    /// Every relation-kind maps to a canonical derivation kind
    /// (Identity / Subsumption / Parthood / Causation / Opposition).
    /// The Derivation ontology declares no custom kinds, so this
    /// closure property is a hard invariant of the functor.
    #[test]
    fn property_kind_map_into_canonical(k in arb_causation_relation_kind()) {
        let m = CausationRelation {
            from: CausationConcept::Cause,
            to: CausationConcept::Effect,
            kind: k,
        };
        let projected_kind = CausationToDerivation::map_morphism(&m).kind;
        prop_assert!(
            matches!(
                projected_kind,
                DerivationRelationKind::Identity
                    | DerivationRelationKind::Subsumption
                    | DerivationRelationKind::Parthood
                    | DerivationRelationKind::Causation
                    | DerivationRelationKind::Opposition
            ),
            "kind {k:?} mapped to {projected_kind:?}, which is not canonical"
        );
    }

    /// Identity preservation (Mac Lane §I.3 functor law, instance form):
    /// for every object A, F(id_A) is the identity morphism on F(A).
    #[test]
    fn property_identity_preservation(c in arb_causation_concept()) {
        let id_source = CausationRelation {
            from: c,
            to: c,
            kind: CausationRelationKind::Identity,
        };
        let projected = CausationToDerivation::map_morphism(&id_source);
        let expected_target = CausationToDerivation::map_object(&c);
        prop_assert_eq!(projected.from, expected_target);
        prop_assert_eq!(projected.to, expected_target);
        prop_assert_eq!(projected.kind, DerivationRelationKind::Identity);
    }

    /// Object map is deterministic — calling twice yields the same
    /// result. (Trivial for a pure function; serves as a regression
    /// guard if anyone ever adds internal state.)
    #[test]
    fn property_object_map_deterministic(c in arb_causation_concept()) {
        let a = CausationToDerivation::map_object(&c);
        let b = CausationToDerivation::map_object(&c);
        prop_assert_eq!(a, b);
    }
}

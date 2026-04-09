use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::validate::check_category_laws;

use super::distinction::*;
use super::epistemics::*;
use super::metacognition::*;

// =============================================================================
// Distinction tests
// =============================================================================

#[test]
fn distinction_category_laws() {
    check_category_laws::<DistinctionCategory>().unwrap();
}

#[test]
fn distinction_has_6_elements() {
    assert_eq!(DistinctionElement::variants().len(), 6);
}

#[test]
fn mark_creates_boundary() {
    let m = DistinctionCategory::morphisms();
    assert!(m.iter().any(|r| r.from == DistinctionElement::Mark
        && r.to == DistinctionElement::Boundary
        && r.kind == DistinctionRelationKind::Creates));
}

#[test]
fn void_precedes_mark() {
    let m = DistinctionCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.from == DistinctionElement::Void && r.to == DistinctionElement::Mark)
    );
}

#[test]
fn reentry_is_self_reference() {
    let m = DistinctionCategory::morphisms();
    assert!(m.iter().any(|r| r.from == DistinctionElement::ReEntry
        && r.to == DistinctionElement::Mark
        && r.kind == DistinctionRelationKind::AppliesTo));
}

#[test]
fn draw_distinction_works() {
    let (marked, unmarked) = draw_distinction("this", "that");
    assert_eq!(marked, "this");
    assert_eq!(unmarked, "that");
}

#[test]
#[should_panic]
fn draw_distinction_requires_difference() {
    draw_distinction("same", "same");
}

// =============================================================================
// Epistemics tests
// =============================================================================

#[test]
fn epistemic_category_laws() {
    check_category_laws::<EpistemicCategory>().unwrap();
}

#[test]
fn epistemic_has_4_states() {
    assert_eq!(EpistemicState::variants().len(), 4);
}

#[test]
fn observation_detects_gap() {
    let m = EpistemicCategory::morphisms();
    assert!(m.iter().any(|r| r.from == EpistemicState::UnknownUnknown
        && r.to == EpistemicState::KnownUnknown
        && r.kind == TransitionKind::Observation));
}

#[test]
fn learning_fills_gap() {
    let m = EpistemicCategory::morphisms();
    assert!(m.iter().any(|r| r.from == EpistemicState::KnownUnknown
        && r.to == EpistemicState::KnownKnown
        && r.kind == TransitionKind::Learning));
}

#[test]
fn repair_fixes_access() {
    let m = EpistemicCategory::morphisms();
    assert!(m.iter().any(|r| r.from == EpistemicState::UnknownKnown
        && r.to == EpistemicState::KnownKnown
        && r.kind == TransitionKind::Repair));
}

#[test]
fn classify_known_known() {
    let state = classify_result(true, true, Some("dog is a mammal"));
    assert_eq!(state, EpistemicState::KnownKnown);
}

#[test]
fn classify_known_unknown() {
    let state = classify_result::<&str>(true, false, None);
    assert_eq!(state, EpistemicState::KnownUnknown);
}

#[test]
fn classify_unknown_known() {
    let state = classify_result::<&str>(false, true, None);
    assert_eq!(state, EpistemicState::UnknownKnown);
}

#[test]
fn classify_unknown_unknown() {
    let state = classify_result::<&str>(false, false, None);
    assert_eq!(state, EpistemicState::UnknownUnknown);
}

// =============================================================================
// Metacognition tests
// =============================================================================

#[test]
fn metacognition_category_laws() {
    check_category_laws::<MetaCognitionCategory>().unwrap();
}

#[test]
fn metacognition_has_10_concepts() {
    assert_eq!(MetaConcept::variants().len(), 10);
}

#[test]
fn meta_observes_object() {
    let m = MetaCognitionCategory::morphisms();
    assert!(m.iter().any(|r| r.from == MetaConcept::MetaLevel
        && r.to == MetaConcept::ObjectLevel
        && r.kind == MetaRelationKind::Observes));
}

#[test]
fn evaluation_detects_gap() {
    let m = MetaCognitionCategory::morphisms();
    assert!(m.iter().any(|r| r.from == MetaConcept::Evaluation
        && r.to == MetaConcept::Gap
        && r.kind == MetaRelationKind::Detects));
}

#[test]
fn gap_triggers_repair_or_clarification() {
    let m = MetaCognitionCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.from == MetaConcept::Gap && r.to == MetaConcept::Repair)
    );
    assert!(
        m.iter()
            .any(|r| r.from == MetaConcept::Gap && r.to == MetaConcept::Clarification)
    );
}

#[test]
fn meta_reaches_clarification() {
    // The full loop: MetaLevel → ... → Clarification
    let m = MetaCognitionCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.from == MetaConcept::MetaLevel && r.to == MetaConcept::Clarification)
    );
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_epistemic() -> impl Strategy<Value = EpistemicState> {
        prop_oneof![
            Just(EpistemicState::KnownKnown),
            Just(EpistemicState::KnownUnknown),
            Just(EpistemicState::UnknownKnown),
            Just(EpistemicState::UnknownUnknown),
        ]
    }

    fn arb_meta() -> impl Strategy<Value = MetaConcept> {
        prop_oneof![
            Just(MetaConcept::ObjectLevel),
            Just(MetaConcept::MetaLevel),
            Just(MetaConcept::Monitoring),
            Just(MetaConcept::Evaluation),
            Just(MetaConcept::Control),
            Just(MetaConcept::Trace),
            Just(MetaConcept::Gap),
            Just(MetaConcept::Repair),
            Just(MetaConcept::Clarification),
            Just(MetaConcept::EpistemicAssessment),
        ]
    }

    proptest! {
        #[test]
        fn prop_epistemic_identity(s in arb_epistemic()) {
            let id = EpistemicCategory::identity(&s);
            prop_assert_eq!(EpistemicCategory::compose(&id, &id), Some(id));
        }

        #[test]
        fn prop_meta_identity(c in arb_meta()) {
            let id = MetaCognitionCategory::identity(&c);
            prop_assert_eq!(MetaCognitionCategory::compose(&id, &id), Some(id));
        }

        /// The goal state is always KnownKnown — every other state has a path to it.
        #[test]
        fn prop_known_known_reachable(s in arb_epistemic()) {
            let m = EpistemicCategory::morphisms();
            let reaches = m.iter().any(|r| r.from == s && r.to == EpistemicState::KnownKnown);
            prop_assert!(reaches, "{:?} should be able to reach KnownKnown", s);
        }

        /// MetaLevel can reach all concepts (it observes everything).
        #[test]
        fn prop_meta_reaches_all(c in arb_meta()) {
            let m = MetaCognitionCategory::morphisms();
            let reaches = m.iter().any(|r| r.from == MetaConcept::MetaLevel && r.to == c);
            prop_assert!(reaches, "MetaLevel should reach {:?}", c);
        }
    }
}

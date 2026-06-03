use pr4xis::category::Category;
use pr4xis::category::entity::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;

use super::distinction::*;
use super::epistemics::*;
use super::metacognition::*;

// -----------------------------------------------------------------------------
// Reachability helper — BFS over the category's directed morphism graph.
//
// Per #166 the macro emits only same-kind transitive closure as direct
// morphisms; heterogeneous-kind paths exist in the graph but are not
// materialized as single edges. Tests that assert "concept A can reach
// concept B" via a heterogeneous chain must walk the graph rather than
// query `morphisms()` for a direct edge.
// -----------------------------------------------------------------------------

fn reaches<C: Category>(from: C::Object, to: C::Object) -> bool
where
    C::Object: Eq + Clone + std::hash::Hash,
{
    use pr4xis::category::Arrow;
    use std::collections::{HashSet, VecDeque};
    let ms = C::morphisms();
    let mut visited: HashSet<C::Object> = HashSet::new();
    let mut queue: VecDeque<C::Object> = VecDeque::new();
    queue.push_back(from);
    while let Some(node) = queue.pop_front() {
        if node == to {
            return true;
        }
        if !visited.insert(node.clone()) {
            continue;
        }
        for m in ms.iter().filter(|m| m.source() == node) {
            queue.push_back(m.target());
        }
    }
    false
}

// =============================================================================
// Distinction tests
// =============================================================================

#[test]
fn distinction_category_laws() {
    assert_category_laws::<DistinctionCategory>();
}

#[test]
fn distinction_has_6_elements() {
    assert_eq!(DistinctionConcept::variants().len(), 6);
}

#[test]
fn mark_creates_boundary() {
    let m = DistinctionCategory::morphisms();
    assert!(m.iter().any(|r| r.from == DistinctionConcept::Mark
        && r.to == DistinctionConcept::Boundary
        && r.kind == DistinctionRelationKind::Creates));
}

#[test]
fn void_precedes_mark() {
    let m = DistinctionCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.from == DistinctionConcept::Void && r.to == DistinctionConcept::Mark)
    );
}

#[test]
fn reentry_is_self_reference() {
    let m = DistinctionCategory::morphisms();
    assert!(m.iter().any(|r| r.from == DistinctionConcept::ReEntry
        && r.to == DistinctionConcept::Mark
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
    assert_category_laws::<EpistemicCategory>();
}

#[test]
fn epistemic_has_4_states() {
    assert_eq!(EpistemicConcept::variants().len(), 4);
}

#[test]
fn observation_detects_gap() {
    let m = EpistemicCategory::morphisms();
    assert!(m.iter().any(|r| r.from == EpistemicConcept::UnknownUnknown
        && r.to == EpistemicConcept::KnownUnknown
        && r.kind == EpistemicRelationKind::Observation));
}

#[test]
fn learning_fills_gap() {
    let m = EpistemicCategory::morphisms();
    assert!(m.iter().any(|r| r.from == EpistemicConcept::KnownUnknown
        && r.to == EpistemicConcept::KnownKnown
        && r.kind == EpistemicRelationKind::Learning));
}

#[test]
fn repair_fixes_access() {
    let m = EpistemicCategory::morphisms();
    assert!(m.iter().any(|r| r.from == EpistemicConcept::UnknownKnown
        && r.to == EpistemicConcept::KnownKnown
        && r.kind == EpistemicRelationKind::Repair));
}

#[test]
fn classify_known_known() {
    let state = classify_result(true, true, Some("dog is a mammal"));
    assert_eq!(state, EpistemicConcept::KnownKnown);
}

#[test]
fn classify_known_unknown() {
    let state = classify_result::<&str>(true, false, None);
    assert_eq!(state, EpistemicConcept::KnownUnknown);
}

#[test]
fn classify_unknown_known() {
    let state = classify_result::<&str>(false, true, None);
    assert_eq!(state, EpistemicConcept::UnknownKnown);
}

#[test]
fn classify_unknown_unknown() {
    let state = classify_result::<&str>(false, false, None);
    assert_eq!(state, EpistemicConcept::UnknownUnknown);
}

// =============================================================================
// Metacognition tests
// =============================================================================

#[test]
fn metacognition_category_laws() {
    assert_category_laws::<MetaCognitionCategory>();
}

#[test]
fn metacognition_has_10_concepts() {
    assert_eq!(MetaCognitionConcept::variants().len(), 10);
}

#[test]
fn meta_observes_object() {
    let m = MetaCognitionCategory::morphisms();
    assert!(m.iter().any(|r| r.from == MetaCognitionConcept::MetaLevel
        && r.to == MetaCognitionConcept::ObjectLevel
        && r.kind == MetaCognitionRelationKind::Observes));
}

#[test]
fn evaluation_detects_gap() {
    let m = MetaCognitionCategory::morphisms();
    assert!(m.iter().any(|r| r.from == MetaCognitionConcept::Evaluation
        && r.to == MetaCognitionConcept::Gap
        && r.kind == MetaCognitionRelationKind::Detects));
}

#[test]
fn gap_triggers_repair_or_clarification() {
    let m = MetaCognitionCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.from == MetaCognitionConcept::Gap && r.to == MetaCognitionConcept::Repair)
    );
    assert!(m.iter().any(
        |r| r.from == MetaCognitionConcept::Gap && r.to == MetaCognitionConcept::Clarification
    ));
}

#[test]
fn meta_reaches_clarification() {
    // The full loop: MetaLevel → ... → Clarification. Per #166 the
    // heterogeneous-kind chain (Orchestrates ∘ Decides) is not emitted as a
    // direct morphism — walk the graph.
    assert!(reaches::<MetaCognitionCategory>(
        MetaCognitionConcept::MetaLevel,
        MetaCognitionConcept::Clarification
    ));
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_epistemic() -> impl Strategy<Value = EpistemicConcept> {
        prop_oneof![
            Just(EpistemicConcept::KnownKnown),
            Just(EpistemicConcept::KnownUnknown),
            Just(EpistemicConcept::UnknownKnown),
            Just(EpistemicConcept::UnknownUnknown),
        ]
    }

    fn arb_meta() -> impl Strategy<Value = MetaCognitionConcept> {
        prop_oneof![
            Just(MetaCognitionConcept::ObjectLevel),
            Just(MetaCognitionConcept::MetaLevel),
            Just(MetaCognitionConcept::Monitoring),
            Just(MetaCognitionConcept::Evaluation),
            Just(MetaCognitionConcept::Control),
            Just(MetaCognitionConcept::Trace),
            Just(MetaCognitionConcept::Gap),
            Just(MetaCognitionConcept::Repair),
            Just(MetaCognitionConcept::Clarification),
            Just(MetaCognitionConcept::EpistemicAssessment),
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
            // Self-trivially reaches; the test exercises non-identity paths.
            let r = s == EpistemicConcept::KnownKnown
                || super::reaches::<EpistemicCategory>(s, EpistemicConcept::KnownKnown);
            prop_assert!(r, "{:?} should be able to reach KnownKnown", s);
        }

        /// MetaLevel can reach all concepts (it observes everything).
        #[test]
        fn prop_meta_reaches_all(c in arb_meta()) {
            let r = c == MetaCognitionConcept::MetaLevel
                || super::reaches::<MetaCognitionCategory>(MetaCognitionConcept::MetaLevel, c);
            prop_assert!(r, "MetaLevel should reach {:?}", c);
        }

        // ---- Distinction property tests ----

        /// Distinction identity is idempotent.
        #[test]
        fn prop_distinction_identity(e in arb_distinction()) {
            let id = DistinctionCategory::identity(&e);
            prop_assert_eq!(DistinctionCategory::compose(&id, &id), Some(id));
        }

        /// Boundary always separates into MarkedSpace AND UnmarkedSpace.
        #[test]
        fn prop_boundary_separates_both(_dummy in 0..1i32) {
            let m = DistinctionCategory::morphisms();
            let to_marked = m.iter().any(|r|
                r.from == DistinctionConcept::Boundary
                && r.to == DistinctionConcept::MarkedSpace);
            let to_unmarked = m.iter().any(|r|
                r.from == DistinctionConcept::Boundary
                && r.to == DistinctionConcept::UnmarkedSpace);
            prop_assert!(to_marked, "Boundary must separate to MarkedSpace");
            prop_assert!(to_unmarked, "Boundary must separate to UnmarkedSpace");
        }

        /// Void reaches Mark (distinction can emerge from nothing).
        #[test]
        fn prop_void_reaches_mark(_dummy in 0..1i32) {
            let m = DistinctionCategory::morphisms();
            let reaches = m.iter().any(|r|
                r.from == DistinctionConcept::Void
                && r.to == DistinctionConcept::Mark);
            prop_assert!(reaches);
        }

        /// ReEntry reaches both spaces (self-reference sees both sides).
        /// Per #166 the heterogeneous chain (AppliesTo ∘ Creates ∘ Separates)
        /// isn't a direct morphism — walk the graph.
        #[test]
        fn prop_reentry_reaches_both_spaces(_dummy in 0..1i32) {
            let to_marked = super::reaches::<DistinctionCategory>(
                DistinctionConcept::ReEntry,
                DistinctionConcept::MarkedSpace,
            );
            let to_unmarked = super::reaches::<DistinctionCategory>(
                DistinctionConcept::ReEntry,
                DistinctionConcept::UnmarkedSpace,
            );
            prop_assert!(to_marked);
            prop_assert!(to_unmarked);
        }

        // ---- Epistemic property tests ----

        /// Observation then Learning gives UU → KK (composed transition).
        /// Per #166 heterogeneous-kind chain isn't materialized as a direct
        /// morphism — walk the graph.
        #[test]
        fn prop_observe_then_learn(_dummy in 0..1i32) {
            let uu_to_kk = super::reaches::<EpistemicCategory>(
                EpistemicConcept::UnknownUnknown,
                EpistemicConcept::KnownKnown,
            );
            prop_assert!(uu_to_kk, "UU should reach KK via observation+learning");
        }

        /// Forgetting is recoverable: KK → UK → KK via repair.
        #[test]
        fn prop_forgetting_recoverable(_dummy in 0..1i32) {
            let m = EpistemicCategory::morphisms();
            let forgets = m.iter().any(|r|
                r.from == EpistemicConcept::KnownKnown
                && r.to == EpistemicConcept::UnknownKnown
                && r.kind == EpistemicRelationKind::Forgetting);
            let repairs = m.iter().any(|r|
                r.from == EpistemicConcept::UnknownKnown
                && r.to == EpistemicConcept::KnownKnown
                && r.kind == EpistemicRelationKind::Repair);
            prop_assert!(forgets, "KK should be able to forget to UK");
            prop_assert!(repairs, "UK should be repairable back to KK");
        }

        /// classify_result is exhaustive: every combination maps to a state.
        #[test]
        fn prop_classify_exhaustive(parsed in proptest::bool::ANY, exists in proptest::bool::ANY) {
            let result: Option<&str> = if parsed && exists { Some("value") } else { None };
            let _state = classify_result(parsed, exists, result);
            // Should not panic — all combinations handled
        }

        // ---- Metacognition property tests ----

        /// Gap always leads to either Repair or Clarification (never stuck).
        #[test]
        fn prop_gap_never_stuck(_dummy in 0..1i32) {
            let m = MetaCognitionCategory::morphisms();
            let to_repair = m.iter().any(|r|
                r.from == MetaCognitionConcept::Gap && r.to == MetaCognitionConcept::Repair);
            let to_clarification = m.iter().any(|r|
                r.from == MetaCognitionConcept::Gap && r.to == MetaCognitionConcept::Clarification);
            prop_assert!(to_repair || to_clarification,
                "Gap must lead to Repair or Clarification");
            prop_assert!(to_repair, "Gap must be able to trigger Repair");
            prop_assert!(to_clarification, "Gap must be able to trigger Clarification");
        }

        /// Monitoring → Evaluation chain exists (you can't evaluate without monitoring first).
        #[test]
        fn prop_monitoring_before_evaluation(_dummy in 0..1i32) {
            let m = MetaCognitionCategory::morphisms();
            let chain = m.iter().any(|r|
                r.from == MetaCognitionConcept::Monitoring
                && r.to == MetaCognitionConcept::Evaluation);
            prop_assert!(chain);
        }

        /// Evaluation → Control chain exists (evaluation informs control decisions).
        #[test]
        fn prop_evaluation_informs_control(_dummy in 0..1i32) {
            let m = MetaCognitionCategory::morphisms();
            let chain = m.iter().any(|r|
                r.from == MetaCognitionConcept::Evaluation
                && r.to == MetaCognitionConcept::Control);
            prop_assert!(chain);
        }
    }

    fn arb_distinction() -> impl Strategy<Value = DistinctionConcept> {
        prop_oneof![
            Just(DistinctionConcept::Void),
            Just(DistinctionConcept::Mark),
            Just(DistinctionConcept::Boundary),
            Just(DistinctionConcept::MarkedSpace),
            Just(DistinctionConcept::UnmarkedSpace),
            Just(DistinctionConcept::ReEntry),
        ]
    }
}

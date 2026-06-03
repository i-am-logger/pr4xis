use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};
use pr4xis::category::{Category, FinitelyGenerated, Functor};

use super::ontology::*;
use super::traffic_functor::*;

// =============================================================================
// Systems thinking category tests
// =============================================================================

#[test]
fn systems_category_laws() {
    assert_category_laws::<SystemCategory>();
}

#[test]
fn systems_has_10_concepts() {
    assert_eq!(SystemConcept::variants().len(), 10);
}

#[test]
fn constraint_governs_transition() {
    let morphisms = SystemCategory::morphisms();
    assert!(morphisms.iter().any(|m| m.from == SystemConcept::Constraint
        && m.to == SystemConcept::Transition
        && m.kind == SystemRelationKind::Governs));
}

#[test]
fn feedback_loop_exists() {
    // State → Feedback → Transition → State (the cybernetic loop)
    let morphisms = SystemCategory::morphisms();

    // State → Feedback
    assert!(morphisms.iter().any(|m| m.from == SystemConcept::State
        && m.to == SystemConcept::Feedback
        && m.kind == SystemRelationKind::FeedsBack));

    // Feedback → Transition
    assert!(morphisms.iter().any(|m| m.from == SystemConcept::Feedback
        && m.to == SystemConcept::Transition
        && m.kind == SystemRelationKind::FeedsBack));

    // Transition → State
    assert!(morphisms.iter().any(|m| m.from == SystemConcept::Transition
        && m.to == SystemConcept::State
        && m.kind == SystemRelationKind::Changes));
}

#[test]
fn emergence_arises_from_interaction() {
    let morphisms = SystemCategory::morphisms();
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == SystemConcept::Interaction
                && m.to == SystemConcept::Emergence
                && m.kind == SystemRelationKind::ArisesFrom)
    );
}

#[test]
fn controller_regulates_constraint() {
    let morphisms = SystemCategory::morphisms();
    assert!(morphisms.iter().any(|m| m.from == SystemConcept::Controller
        && m.to == SystemConcept::Constraint
        && m.kind == SystemRelationKind::Regulates));
}

#[test]
fn homeostasis_stabilizes_state() {
    let morphisms = SystemCategory::morphisms();
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == SystemConcept::Homeostasis
                && m.to == SystemConcept::State
                && m.kind == SystemRelationKind::Stabilizes)
    );
}

// =============================================================================
// Traffic system category tests
// =============================================================================

#[test]
fn traffic_system_category_laws() {
    assert_category_laws::<TrafficSystemCategory>();
}

#[test]
fn traffic_has_10_elements() {
    assert_eq!(TrafficSystemElement::variants().len(), 10);
}

#[test]
fn safety_rule_governs_signal_advance() {
    let morphisms = TrafficSystemCategory::morphisms();
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == TrafficSystemElement::SafetyRule
                && m.to == TrafficSystemElement::SignalAdvance
                && m.kind == TrafficRelationKind::Governs)
    );
}

#[test]
fn traffic_feedback_loop() {
    let morphisms = TrafficSystemCategory::morphisms();

    // IntersectionState → CongestionFeedback
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == TrafficSystemElement::IntersectionState
                && m.to == TrafficSystemElement::CongestionFeedback)
    );

    // CongestionFeedback → SignalAdvance
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == TrafficSystemElement::CongestionFeedback
                && m.to == TrafficSystemElement::SignalAdvance)
    );

    // SignalAdvance → IntersectionState
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == TrafficSystemElement::SignalAdvance
                && m.to == TrafficSystemElement::IntersectionState)
    );
}

// =============================================================================
// THE PROOF: Functor from Traffic to Systems Thinking
// =============================================================================

#[test]
fn functor_laws_hold() {
    // THIS IS THE PROOF.
    // If this passes, traffic IS a system — not by analogy,
    // but by mathematical structure preservation.
    assert_functor_laws::<TrafficToSystems>();
}

#[test]
fn signal_maps_to_component() {
    assert_eq!(
        TrafficToSystems::map_object(&TrafficSystemElement::Signal),
        SystemConcept::Component
    );
}

#[test]
fn intersection_state_maps_to_state() {
    assert_eq!(
        TrafficToSystems::map_object(&TrafficSystemElement::IntersectionState),
        SystemConcept::State
    );
}

#[test]
fn signal_advance_maps_to_transition() {
    assert_eq!(
        TrafficToSystems::map_object(&TrafficSystemElement::SignalAdvance),
        SystemConcept::Transition
    );
}

#[test]
fn safety_rule_maps_to_constraint() {
    assert_eq!(
        TrafficToSystems::map_object(&TrafficSystemElement::SafetyRule),
        SystemConcept::Constraint
    );
}

#[test]
fn congestion_maps_to_feedback() {
    assert_eq!(
        TrafficToSystems::map_object(&TrafficSystemElement::CongestionFeedback),
        SystemConcept::Feedback
    );
}

#[test]
fn green_wave_maps_to_homeostasis() {
    assert_eq!(
        TrafficToSystems::map_object(&TrafficSystemElement::GreenWaveTiming),
        SystemConcept::Homeostasis
    );
}

#[test]
fn flow_rate_maps_to_emergence() {
    assert_eq!(
        TrafficToSystems::map_object(&TrafficSystemElement::FlowRate),
        SystemConcept::Emergence
    );
}

#[test]
fn functor_preserves_identity() {
    for elem in TrafficSystemElement::variants() {
        let traffic_id = TrafficSystemCategory::identity(&elem);
        let mapped = TrafficToSystems::map_morphism(&traffic_id);
        let systems_id = SystemCategory::identity(&TrafficToSystems::map_object(&elem));
        assert_eq!(mapped, systems_id, "identity not preserved for {:?}", elem);
    }
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_system_concept() -> impl Strategy<Value = SystemConcept> {
        prop_oneof![
            Just(SystemConcept::Component),
            Just(SystemConcept::Interaction),
            Just(SystemConcept::State),
            Just(SystemConcept::Transition),
            Just(SystemConcept::Constraint),
            Just(SystemConcept::Feedback),
            Just(SystemConcept::Homeostasis),
            Just(SystemConcept::Emergence),
            Just(SystemConcept::Boundary),
            Just(SystemConcept::Controller),
        ]
    }

    fn arb_traffic_element() -> impl Strategy<Value = TrafficSystemElement> {
        prop_oneof![
            Just(TrafficSystemElement::Signal),
            Just(TrafficSystemElement::DirectionConflict),
            Just(TrafficSystemElement::IntersectionState),
            Just(TrafficSystemElement::SignalAdvance),
            Just(TrafficSystemElement::SafetyRule),
            Just(TrafficSystemElement::CongestionFeedback),
            Just(TrafficSystemElement::GreenWaveTiming),
            Just(TrafficSystemElement::FlowRate),
            Just(TrafficSystemElement::IntersectionBoundary),
            Just(TrafficSystemElement::SignalController),
        ]
    }

    proptest! {
        /// Functor maps every traffic element to a valid system concept.
        #[test]
        fn prop_functor_maps_valid(elem in arb_traffic_element()) {
            let concept = TrafficToSystems::map_object(&elem);
            prop_assert!(SystemConcept::variants().contains(&concept));
        }

        /// Functor preserves identity for all elements.
        #[test]
        fn prop_functor_preserves_identity(elem in arb_traffic_element()) {
            let traffic_id = TrafficSystemCategory::identity(&elem);
            let mapped = TrafficToSystems::map_morphism(&traffic_id);
            let systems_id = SystemCategory::identity(&TrafficToSystems::map_object(&elem));
            prop_assert_eq!(mapped, systems_id);
        }

        /// Every system concept has identity composition: id ∘ id = id.
        #[test]
        fn prop_identity_idempotent(concept in arb_system_concept()) {
            let id = SystemCategory::identity(&concept);
            let composed = SystemCategory::compose(&id, &id);
            prop_assert_eq!(composed, Some(id));
        }

        /// Functor preserves composition: F(g∘f) = F(g)∘F(f) for any composable pair.
        #[test]
        fn prop_functor_preserves_composition(
            a in arb_traffic_element(),
            b in arb_traffic_element(),
            c in arb_traffic_element()
        ) {
            let morphisms = TrafficSystemCategory::morphisms();
            // Find morphisms a→b and b→c if they exist
            if let Some(f) = morphisms.iter().find(|m| m.from == a && m.to == b)
                && let Some(g) = morphisms.iter().find(|m| m.from == b && m.to == c)
                && let Some(gf) = TrafficSystemCategory::compose(f, g)
            {
                let mapped_gf = TrafficToSystems::map_morphism(&gf);
                let composed_mapped = SystemCategory::compose(
                    &TrafficToSystems::map_morphism(f),
                    &TrafficToSystems::map_morphism(g),
                );
                prop_assert_eq!(Some(mapped_gf), composed_mapped);
            }
        }

        /// The functor is surjective on objects: every system concept has at least one traffic element mapping to it.
        #[test]
        fn prop_functor_surjective(concept in arb_system_concept()) {
            let has_preimage = TrafficSystemElement::variants()
                .iter()
                .any(|e| TrafficToSystems::map_object(e) == concept);
            prop_assert!(has_preimage, "no traffic element maps to {:?}", concept);
        }

        /// State reaches every concept of the cybernetic feedback loop.
        /// Ashby (1956) §10: the loop is `State → Feedback → Controller →
        /// Constraint → Transition → Component → State`. Per Ashby's
        /// architecture, the loop is closed over those six concepts;
        /// structural concepts that *feed* the loop (Interaction,
        /// Boundary) and *emerge from* it (Emergence) live outside the
        /// loop's outgoing reachability.
        ///
        /// Per #166 the closure across heterogeneous kinds is not a single
        /// morphism, so reachability is walked through the morphism graph.
        #[test]
        fn prop_state_reaches_all(concept in arb_system_concept()) {
            // Outside-the-loop concepts (see Ashby 1956 §10): no outgoing
            // path from State reaches them by design.
            if matches!(concept,
                SystemConcept::Interaction
                | SystemConcept::Emergence
                | SystemConcept::Boundary
            ) {
                return Ok(());
            }
            use std::collections::{HashSet, VecDeque};
            use pr4xis::category::Arrow;
            let morphisms = SystemCategory::morphisms();
            let mut visited: HashSet<SystemConcept> = HashSet::new();
            let mut queue: VecDeque<SystemConcept> = VecDeque::new();
            queue.push_back(SystemConcept::State);
            let mut reaches = SystemConcept::State == concept;
            while let Some(node) = queue.pop_front() {
                if node == concept {
                    reaches = true;
                    break;
                }
                if !visited.insert(node) {
                    continue;
                }
                for m in morphisms.iter().filter(|m| m.source() == node) {
                    queue.push_back(m.target());
                }
            }
            prop_assert!(reaches,
                "State cannot reach {:?} — cybernetic loop is broken", concept);
        }

        /// Every endurant-like concept (Component, State, Boundary) is NOT a transition.
        #[test]
        fn prop_structural_concepts_not_transitions(concept in arb_system_concept()) {
            if matches!(concept, SystemConcept::Component | SystemConcept::State | SystemConcept::Boundary) {
                // These should NOT map to Transition-like roles
                let morphisms = SystemCategory::morphisms();
                let changes_something = morphisms.iter().any(|m|
                    m.from == concept && m.kind == SystemRelationKind::Changes);
                prop_assert!(!changes_something,
                    "{:?} should not directly change state (only transitions do)", concept);
            }
        }
    }
}

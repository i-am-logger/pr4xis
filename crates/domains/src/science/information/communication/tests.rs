use super::ontology::*;
use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;

#[test]
fn category_laws() {
    check_category_laws::<CommunicationCategory>().unwrap();
}

#[test]
fn eight_concepts() {
    assert_eq!(CommunicationConcept::variants().len(), 8);
}

#[test]
fn sender_produces_message() {
    let morphisms = CommunicationCategory::morphisms();
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == CommunicationConcept::Sender
                && m.to == CommunicationConcept::Message
                && m.kind == CommunicationRelationKind::Produces)
    );
}

#[test]
fn noise_corrupts_channel() {
    let morphisms = CommunicationCategory::morphisms();
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == CommunicationConcept::Noise
                && m.to == CommunicationConcept::Channel
                && m.kind == CommunicationRelationKind::Corrupts)
    );
}

#[test]
fn feedback_is_cybernetic() {
    let morphisms = CommunicationCategory::morphisms();
    // Receiver → Feedback → Sender (the cybernetic loop)
    assert!(morphisms.iter().any(
        |m| m.from == CommunicationConcept::Receiver && m.to == CommunicationConcept::Feedback
    ));
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == CommunicationConcept::Feedback
                && m.to == CommunicationConcept::Sender)
    );
}

#[test]
fn jakobson_six_functions() {
    assert_eq!(JakobsonFunction::variants().len(), 6);
}

#[test]
fn phatic_focuses_on_channel() {
    // "Hello" is phatic — focuses on maintaining the channel
    assert_eq!(
        JakobsonFunction::Phatic.focused_component(),
        CommunicationConcept::Channel
    );
}

#[test]
fn metalingual_focuses_on_code() {
    // "What does X mean?" is metalingual — about the code itself
    assert_eq!(
        JakobsonFunction::Metalingual.focused_component(),
        CommunicationConcept::Code
    );
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_communication() -> impl Strategy<Value = CommunicationConcept> {
        prop_oneof![
            Just(CommunicationConcept::Sender),
            Just(CommunicationConcept::Receiver),
            Just(CommunicationConcept::Message),
            Just(CommunicationConcept::Channel),
            Just(CommunicationConcept::Code),
            Just(CommunicationConcept::Noise),
            Just(CommunicationConcept::Feedback),
            Just(CommunicationConcept::Context),
        ]
    }

    fn arb_jakobson() -> impl Strategy<Value = JakobsonFunction> {
        prop_oneof![
            Just(JakobsonFunction::Referential),
            Just(JakobsonFunction::Emotive),
            Just(JakobsonFunction::Conative),
            Just(JakobsonFunction::Phatic),
            Just(JakobsonFunction::Metalingual),
            Just(JakobsonFunction::Poetic),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_communication()) {
            let id = CommunicationCategory::identity(&c);
            prop_assert_eq!(CommunicationCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has both Identity and Composed self-morphisms.
        #[test]
        fn prop_self_morphisms(c in arb_communication()) {
            let m = CommunicationCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c && r.kind == CommunicationRelationKind::Identity);
            let has_composed = m.iter().any(|r| r.from == c && r.to == c && r.kind == CommunicationRelationKind::Composed);
            prop_assert!(has_identity);
            prop_assert!(has_composed);
        }

        /// Jakobson: every function focuses on exactly one communication component.
        #[test]
        fn prop_jakobson_focuses_valid(f in arb_jakobson()) {
            let component = f.focused_component();
            prop_assert!(CommunicationConcept::variants().contains(&component));
        }

        /// Jakobson bijection: no two functions focus on the same component.
        #[test]
        fn prop_jakobson_injective(f1 in arb_jakobson(), f2 in arb_jakobson()) {
            if f1 != f2 {
                prop_assert_ne!(f1.focused_component(), f2.focused_component());
            }
        }

        /// Shannon's chain: Sender → Message → Channel exists.
        #[test]
        fn prop_shannon_chain(_dummy in 0..1i32) {
            let m = CommunicationCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.from == CommunicationConcept::Sender && r.to == CommunicationConcept::Message));
            prop_assert!(m.iter().any(|r| r.from == CommunicationConcept::Message && r.to == CommunicationConcept::Channel));
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_communication()) {
            let m = CommunicationCategory::morphisms();
            let id = CommunicationCategory::identity(&c);
            for morph in m.iter().filter(|r| r.from == c) {
                let composed = CommunicationCategory::compose(&id, morph);
                prop_assert_eq!(composed.as_ref().map(|r| (r.from, r.to)), Some((morph.from, morph.to)));
            }
        }
    }
}

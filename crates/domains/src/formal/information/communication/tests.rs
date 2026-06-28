use super::ontology::*;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn category_laws() {
    assert_category_laws::<CommunicationCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn eight_concepts() {
    assert_eq!(CommunicationConcept::variants().len(), 8);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn sender_produces_message() {
    let m = CommunicationCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == CommunicationConcept::Sender
        && r.target() == CommunicationConcept::Message
        && r.kind() == CommunicationRelationKind::Produces));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn noise_corrupts_channel() {
    let m = CommunicationCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == CommunicationConcept::Noise
        && r.target() == CommunicationConcept::Channel
        && r.kind() == CommunicationRelationKind::Corrupts));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn feedback_is_cybernetic() {
    let m = CommunicationCategory::morphisms();
    // Wiener (1948): Receiver → Feedback → Sender (the cybernetic loop).
    assert!(
        m.iter()
            .any(|r| r.source() == CommunicationConcept::Receiver
                && r.target() == CommunicationConcept::Feedback)
    );
    assert!(
        m.iter()
            .any(|r| r.source() == CommunicationConcept::Feedback
                && r.target() == CommunicationConcept::Sender)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn jakobson_six_functions() {
    assert_eq!(JakobsonFunction::variants().len(), 6);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn phatic_focuses_on_channel() {
    assert_eq!(
        JakobsonFunction::Phatic.focused_component(),
        CommunicationConcept::Channel
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn metalingual_focuses_on_code() {
    assert_eq!(
        JakobsonFunction::Metalingual.focused_component(),
        CommunicationConcept::Code
    );
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_communication() -> impl Strategy<Value = CommunicationConcept> {
        proptest::sample::select(CommunicationConcept::variants())
    }

    fn arb_jakobson() -> impl Strategy<Value = JakobsonFunction> {
        proptest::sample::select(JakobsonFunction::variants())
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_communication()) {
            let id = CommunicationCategory::identity(&c);
            prop_assert_eq!(CommunicationCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has an Identity self-morphism. The legacy
        /// `Composed` self-morphism check was removed when the proc macro
        /// dropped its dense `Composed` kind (#166).
        #[test]
        fn prop_self_identity(c in arb_communication()) {
            let m = CommunicationCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == c
                && r.target() == c
                && r.kind() == CommunicationRelationKind::Identity));
        }

        #[test]
        fn prop_jakobson_focuses_valid(f in arb_jakobson()) {
            prop_assert!(CommunicationConcept::variants().contains(&f.focused_component()));
        }

        #[test]
        fn prop_jakobson_injective(f1 in arb_jakobson(), f2 in arb_jakobson()) {
            if f1 != f2 {
                prop_assert_ne!(f1.focused_component(), f2.focused_component());
            }
        }

        #[test]
        fn prop_shannon_chain(_dummy in 0..1i32) {
            let m = CommunicationCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == CommunicationConcept::Sender
                && r.target() == CommunicationConcept::Message));
            prop_assert!(m.iter().any(|r| r.source() == CommunicationConcept::Message
                && r.target() == CommunicationConcept::Channel));
        }

        #[test]
        fn prop_left_identity(c in arb_communication()) {
            let m = CommunicationCategory::morphisms();
            let id = CommunicationCategory::identity(&c);
            for morph in m.iter().filter(|r| r.source() == c) {
                let composed = CommunicationCategory::compose(&id, morph);
                prop_assert_eq!(
                    composed.as_ref().map(|r| (r.source(), r.target())),
                    Some((morph.source(), morph.target()))
                );
            }
        }
    }

    pr4xis::register_praxis_value!(prop_identity_idempotent, Deterministic);
    pr4xis::register_praxis_value!(prop_self_identity, Deterministic);
    pr4xis::register_praxis_value!(prop_jakobson_focuses_valid, Verifiable);
    pr4xis::register_praxis_value!(prop_jakobson_injective, Verifiable);
    pr4xis::register_praxis_value!(prop_shannon_chain, Verifiable);
    pr4xis::register_praxis_value!(prop_left_identity, Deterministic);
}

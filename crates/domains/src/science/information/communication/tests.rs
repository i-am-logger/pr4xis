use super::ontology::*;
use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::validate::check_category_laws;

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

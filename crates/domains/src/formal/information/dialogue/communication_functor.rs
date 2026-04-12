use pr4xis::category::Functor;

use super::ontology::*;
use crate::formal::information::communication::ontology::*;

// Dialogue → Communication functor.
//
// Proof that dialogue IS communication.
// Every dialogue concept maps to a communication concept.
// Every dialogue relationship maps to a communication relationship.
// Identity and composition are preserved.
//
// The mapping (Wiener 1948, Jakobson 1960):
//   Participant  → Sender / Receiver (dual role)
//   Utterance    → Message
//   DialogueAct  → Message (the act IS the message content)
//   DialogueState → Context (shared understanding)
//   Topic        → Context (what's being communicated about)
//   History      → Channel (the medium carries history)
//   Understanding → Code (shared interpretation system)
//   Generation   → Code (shared production system)
//   TurnManagement → Feedback (regulates the conversation)
//   Grounding    → Feedback (mutual confirmation of understanding)

pub struct DialogueToCommunication;

impl Functor for DialogueToCommunication {
    type Source = DialogueCategory;
    type Target = CommunicationCategory;

    fn map_object(obj: &DialogueConcept) -> CommunicationConcept {
        match obj {
            // Participants are both sender and receiver (they alternate)
            DialogueConcept::Participant => CommunicationConcept::Sender,
            // Utterance IS the message
            DialogueConcept::Utterance => CommunicationConcept::Message,
            // DialogueAct is what the message DOES — still a message
            DialogueConcept::DialogueAct => CommunicationConcept::Message,
            // DialogueState is shared context
            DialogueConcept::DialogueState => CommunicationConcept::Context,
            // Topic is what we're communicating about — context
            DialogueConcept::Topic => CommunicationConcept::Context,
            // History is the channel — it carries all previous communication
            DialogueConcept::History => CommunicationConcept::Channel,
            // Understanding is the shared code for interpretation
            DialogueConcept::Understanding => CommunicationConcept::Code,
            // Generation is the shared code for production
            DialogueConcept::Generation => CommunicationConcept::Code,
            // TurnManagement is feedback — regulates who speaks
            DialogueConcept::TurnManagement => CommunicationConcept::Feedback,
            // Grounding is feedback — confirms mutual understanding
            DialogueConcept::Grounding => CommunicationConcept::Feedback,
            // QUD is context — the questions drive interpretation
            DialogueConcept::QUD => CommunicationConcept::Context,
            // CommonGround is context — shared beliefs
            DialogueConcept::CommonGround => CommunicationConcept::Context,
            // Intention is the message before it's encoded
            DialogueConcept::Intention => CommunicationConcept::Message,
            // GroundingAct is feedback — confirming understanding
            DialogueConcept::GroundingAct => CommunicationConcept::Feedback,
            // Repair is feedback — fixing misunderstanding
            DialogueConcept::Repair => CommunicationConcept::Feedback,
        }
    }

    fn map_morphism(m: &DialogueRelation) -> CommunicationRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            DialogueRelationKind::Identity => CommunicationRelationKind::Identity,
            DialogueRelationKind::Produces => CommunicationRelationKind::Produces,
            DialogueRelationKind::Expresses => CommunicationRelationKind::Produces,
            DialogueRelationKind::Updates => CommunicationRelationKind::Grounds,
            DialogueRelationKind::AppendedTo => CommunicationRelationKind::TransmittedThrough,
            DialogueRelationKind::Interprets => CommunicationRelationKind::EncodesDecodes,
            DialogueRelationKind::Creates => CommunicationRelationKind::EncodesDecodes,
            DialogueRelationKind::Controls => CommunicationRelationKind::FlowsBack,
            DialogueRelationKind::ArisesFrom => CommunicationRelationKind::FlowsBack,
            DialogueRelationKind::Addresses => CommunicationRelationKind::Grounds,
            DialogueRelationKind::RaisesOrResolves => CommunicationRelationKind::Grounds,
            DialogueRelationKind::EstablishesIn => CommunicationRelationKind::Grounds,
            DialogueRelationKind::Drives => CommunicationRelationKind::Produces,
            DialogueRelationKind::Achieves => CommunicationRelationKind::FlowsBack,
            DialogueRelationKind::Restores => CommunicationRelationKind::FlowsBack,
            DialogueRelationKind::FormedFrom => CommunicationRelationKind::Grounds,
            DialogueRelationKind::Composed => CommunicationRelationKind::Composed,
        };
        CommunicationRelation { from, to, kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::entity::Entity;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<DialogueToCommunication>().unwrap();
    }

    #[test]
    fn participant_maps_to_sender() {
        assert_eq!(
            DialogueToCommunication::map_object(&DialogueConcept::Participant),
            CommunicationConcept::Sender
        );
    }

    #[test]
    fn utterance_maps_to_message() {
        assert_eq!(
            DialogueToCommunication::map_object(&DialogueConcept::Utterance),
            CommunicationConcept::Message
        );
    }

    #[test]
    fn grounding_maps_to_feedback() {
        assert_eq!(
            DialogueToCommunication::map_object(&DialogueConcept::Grounding),
            CommunicationConcept::Feedback
        );
    }

    #[test]
    fn all_dialogue_concepts_map() {
        // Every dialogue concept must map to a valid communication concept
        for concept in DialogueConcept::variants() {
            let mapped = DialogueToCommunication::map_object(&concept);
            assert!(
                CommunicationConcept::variants().contains(&mapped),
                "{:?} mapped to {:?} which is not a valid communication concept",
                concept,
                mapped
            );
        }
    }
}

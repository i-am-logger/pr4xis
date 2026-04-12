use pr4xis::category::Functor;

use super::channel::*;
use crate::formal::information::communication::ontology::*;

// NoisyChannel → Communication functor.
//
// Proof that spelling correction IS communication (Shannon 1948).
// The noisy channel model IS Shannon's communication model:
//   Writer → Channel(with noise) → Reader → Correction
//
// The mapping:
//   Word          → Message (what was intended)
//   Observation   → Message (what was received — possibly corrupted)
//   ErrorModel    → Noise (the corruption model)
//   LanguageModel → Code (the shared knowledge of the language)
//   Correction    → Feedback (the repair signal)
//   ConfusionMatrix → Noise (detailed corruption parameters)

pub struct ChannelToCommunication;

impl Functor for ChannelToCommunication {
    type Source = ChannelCategory;
    type Target = CommunicationCategory;

    fn map_object(obj: &ChannelConcept) -> CommunicationConcept {
        match obj {
            ChannelConcept::Word => CommunicationConcept::Message,
            ChannelConcept::Observation => CommunicationConcept::Message,
            ChannelConcept::ErrorModel => CommunicationConcept::Noise,
            ChannelConcept::LanguageModel => CommunicationConcept::Code,
            ChannelConcept::Correction => CommunicationConcept::Feedback,
            ChannelConcept::ConfusionMatrix => CommunicationConcept::Noise,
        }
    }

    fn map_morphism(m: &ChannelRelation) -> CommunicationRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            ChannelRelationKind::Identity => CommunicationRelationKind::Identity,
            ChannelRelationKind::Corrupts => CommunicationRelationKind::Corrupts,
            ChannelRelationKind::Corrects => CommunicationRelationKind::FlowsBack,
            ChannelRelationKind::Parameterizes => CommunicationRelationKind::Corrupts,
            ChannelRelationKind::Weights => CommunicationRelationKind::EncodesDecodes,
            ChannelRelationKind::Provides => CommunicationRelationKind::Corrupts,
            ChannelRelationKind::Uses => CommunicationRelationKind::EncodesDecodes,
            ChannelRelationKind::Composed => CommunicationRelationKind::Composed,
        };
        CommunicationRelation { from, to, kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<ChannelToCommunication>().unwrap();
    }

    #[test]
    fn word_is_message() {
        assert_eq!(
            ChannelToCommunication::map_object(&ChannelConcept::Word),
            CommunicationConcept::Message
        );
    }

    #[test]
    fn correction_is_feedback() {
        assert_eq!(
            ChannelToCommunication::map_object(&ChannelConcept::Correction),
            CommunicationConcept::Feedback
        );
    }
}

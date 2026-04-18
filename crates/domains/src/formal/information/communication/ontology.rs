#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept; // trait + derive macro
use pr4xis::define_ontology;
use pr4xis::ontology::{Ontology, Quality};

// Communication ontology — the science of information transfer.
//
// Two foundational models:
//   Shannon (1948): Source → Encoder → Channel → Decoder → Destination + Noise
//   Jakobson (1960): Sender → Message → Receiver + Context/Code/Channel functions
//
// References:
// - Shannon, A Mathematical Theory of Communication (1948)
// - Jakobson, Linguistics and Poetics (1960)
// - Lasswell, The Structure and Function of Communication in Society (1948)
// - Wiener, Cybernetics (1948) — feedback in communication

/// Core concepts of communication.
/// Unified from Shannon (1948) and Jakobson (1960).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum CommunicationConcept {
    /// The agent producing the message (Shannon: source; Jakobson: addresser).
    Sender,
    /// The agent interpreting the message (Shannon: destination; Jakobson: addressee).
    Receiver,
    /// The information being communicated (Shannon: signal; Jakobson: message).
    Message,
    /// The medium through which the message travels (Shannon: channel; Jakobson: contact).
    Channel,
    /// The shared system for encoding/decoding (Shannon: encoder/decoder; Jakobson: code).
    Code,
    /// Interference that corrupts the message (Shannon: noise source).
    Noise,
    /// The receiver's response back to the sender (Wiener: cybernetic feedback).
    Feedback,
    /// The shared referential frame (Jakobson: context).
    Context,
}

define_ontology! {
    /// Communication category — Shannon (1948) + Jakobson (1960).
    pub CommunicationOntology for CommunicationCategory {
        concepts: CommunicationConcept,
        relation: CommunicationRelation,
        kind: CommunicationRelationKind,
        kinds: [
            /// Sender produces Message.
            Produces,
            /// Message is transmitted through Channel.
            TransmittedThrough,
            /// Receiver interprets Message.
            Interprets,
            /// Code encodes/decodes Message.
            EncodesDecodes,
            /// Noise corrupts Message in Channel.
            Corrupts,
            /// Feedback flows from Receiver to Sender.
            FlowsBack,
            /// Context grounds Message interpretation.
            Grounds,
            /// Sender and Receiver share Code.
            Shares,
        ],
        edges: [
            // Shannon's chain: Sender → Message → Channel → Receiver
            (Sender, Message, Produces),
            (Message, Channel, TransmittedThrough),
            (Receiver, Message, Interprets),
            // Code encodes/decodes
            (Code, Message, EncodesDecodes),
            // Noise corrupts
            (Noise, Channel, Corrupts),
            // Wiener's cybernetic feedback loop
            (Feedback, Sender, FlowsBack),
            (Receiver, Feedback, Produces),
            // Context grounds interpretation
            (Context, Message, Grounds),
            // Shared code
            (Sender, Code, Shares),
            (Receiver, Code, Shares),
        ],
        composed: [
            (Sender, Channel),
            (Sender, Receiver),
            (Noise, Message),
            (Receiver, Sender),
        ],
        being: AbstractObject,
        source: "Shannon (1948); Jakobson (1960)",
    }
}

/// Jakobson function mapping — which language function does a concept serve?
#[derive(Debug, Clone)]
pub struct CommunicationFunctionQuality;

impl Quality for CommunicationFunctionQuality {
    type Individual = CommunicationConcept;
    type Value = JakobsonFunction;

    fn get(&self, individual: &CommunicationConcept) -> Option<JakobsonFunction> {
        match individual {
            CommunicationConcept::Context => Some(JakobsonFunction::Referential),
            CommunicationConcept::Sender => Some(JakobsonFunction::Emotive),
            CommunicationConcept::Receiver => Some(JakobsonFunction::Conative),
            CommunicationConcept::Channel => Some(JakobsonFunction::Phatic),
            CommunicationConcept::Code => Some(JakobsonFunction::Metalingual),
            CommunicationConcept::Message => Some(JakobsonFunction::Poetic),
            _ => None,
        }
    }
}

impl Ontology for CommunicationOntology {
    type Cat = CommunicationCategory;
    type Qual = CommunicationFunctionQuality;

    fn structural_axioms() -> Vec<Box<dyn pr4xis::ontology::Axiom>> {
        Self::generated_structural_axioms()
    }
}

/// Jakobson's six language functions (1960).
/// Each communication component has a corresponding function when
/// the communicative act focuses on that component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum JakobsonFunction {
    /// Focus on Context → referential (informative).
    Referential,
    /// Focus on Sender → emotive/expressive.
    Emotive,
    /// Focus on Receiver → conative (persuasive, imperative).
    Conative,
    /// Focus on Channel → phatic (maintaining contact: "hello", "how are you?").
    Phatic,
    /// Focus on Code → metalingual (about the code itself: "what does X mean?").
    Metalingual,
    /// Focus on Message → poetic (the form of the message itself).
    Poetic,
}

impl JakobsonFunction {
    /// Which communication component does this function focus on?
    pub fn focused_component(&self) -> CommunicationConcept {
        match self {
            Self::Referential => CommunicationConcept::Context,
            Self::Emotive => CommunicationConcept::Sender,
            Self::Conative => CommunicationConcept::Receiver,
            Self::Phatic => CommunicationConcept::Channel,
            Self::Metalingual => CommunicationConcept::Code,
            Self::Poetic => CommunicationConcept::Message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<CommunicationCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        CommunicationOntology::validate().unwrap();
    }
}

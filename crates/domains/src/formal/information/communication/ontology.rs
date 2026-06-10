//! Communication — Shannon + Jakobson + Wiener model of information
//! transfer.
//!
//! The eight `CommunicationConcept` objects form Shannon's source-channel-
//! destination chain extended with Jakobson's Context/Code and Wiener's
//! cybernetic Feedback loop.
//!
//! # Literature
//!
//! - **Shannon (1948)** "A Mathematical Theory of Communication", *Bell
//!   System Technical Journal* 27 — source / encoder / channel / decoder /
//!   destination + noise.
//! - **Jakobson (1960)** "Linguistics and Poetics", in *Style in Language*
//!   (MIT Press) — six communication functions:
//!   referential / emotive / conative / phatic / metalingual / poetic.
//! - **Lasswell (1948)** "The Structure and Function of Communication in
//!   Society" — who says what to whom through what channel with what effect.
//! - **Wiener (1948)** *Cybernetics: Or Control and Communication in the
//!   Animal and the Machine*, MIT Press — feedback as the defining feature
//!   of cybernetic communication.

use pr4xis::category::Concept;
use pr4xis::ontology::{Ontology, Quality};

pr4xis::ontology! {
    name: "Communication",
    source: "Shannon (1948) A Mathematical Theory of Communication, Bell System Technical Journal 27; Jakobson (1960) Linguistics and Poetics; Lasswell (1948) The Structure and Function of Communication in Society; Wiener (1948) Cybernetics: Or Control and Communication in the Animal and the Machine",

    concepts: [
        Sender,
        Receiver,
        Message,
        Channel,
        Code,
        Noise,
        Feedback,
        Context,
    ],

    labels: {
        Sender: ("en", "Sender",
            "Shannon (1948) §1 source; Jakobson (1960) addresser - the agent producing the message."),
        Receiver: ("en", "Receiver",
            "Shannon (1948) §1 destination; Jakobson (1960) addressee - the agent interpreting the message."),
        Message: ("en", "Message",
            "Shannon (1948) §1 signal; Jakobson (1960) message - the information being communicated."),
        Channel: ("en", "Channel",
            "Shannon (1948) §1 channel; Jakobson (1960) contact - the medium through which the message travels."),
        Code: ("en", "Code",
            "Shannon (1948) §1 encoder/decoder; Jakobson (1960) code - the shared system for encoding and decoding the message."),
        Noise: ("en", "Noise",
            "Shannon (1948) §1 noise source - interference that corrupts the message in the channel."),
        Feedback: ("en", "Feedback",
            "Wiener (1948) Ch. 4 - the receiver's response back to the sender that closes the cybernetic loop."),
        Context: ("en", "Context",
            "Jakobson (1960) - the shared referential frame against which the message is interpreted."),
    },

    edges: [
        // Shannon's chain (1948) §1.
        (Sender, Message, Produces),
        (Message, Channel, TransmittedThrough),
        (Receiver, Message, Interprets),
        // The encoder/decoder pair operates on the message (Shannon 1948).
        (Code, Message, EncodesDecodes),
        // Noise corrupts the channel (Shannon 1948 §1.5).
        (Noise, Channel, Corrupts),
        // Wiener (1948) feedback loop: receiver produces feedback that
        // flows back to the sender.
        (Receiver, Feedback, Produces),
        (Feedback, Sender, FlowsBack),
        // Jakobson (1960): the context grounds interpretation.
        (Context, Message, Grounds),
        // Both ends share the code (Jakobson 1960; Shannon 1948).
        (Sender, Code, Shares),
        (Receiver, Code, Shares),
    ],
}

/// Jakobson's six language functions (1960).
///
/// Each communication component has a corresponding function when the
/// communicative act focuses on that component. Kept as a sibling rich
/// type — the function is a quality-valued descriptor, not a structural
/// object in the communication category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum JakobsonFunction {
    /// Focus on Context → referential (informative).
    Referential,
    /// Focus on Sender → emotive / expressive.
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

/// Quality: the Jakobson function whose focus rests on this concept.
/// Bijective across the six functional components — Jakobson (1960). Noise
/// and Feedback have no Jakobson function (they are Shannon/Wiener
/// constructs outside Jakobson's six-function inventory).
#[derive(Debug, Clone)]
pub struct CommunicationFunctionQuality;

impl Quality for CommunicationFunctionQuality {
    type Individual = CommunicationConcept;
    type Value = JakobsonFunction;

    fn get(&self, individual: &CommunicationConcept) -> Option<JakobsonFunction> {
        use CommunicationConcept as C;
        match individual {
            C::Context => Some(JakobsonFunction::Referential),
            C::Sender => Some(JakobsonFunction::Emotive),
            C::Receiver => Some(JakobsonFunction::Conative),
            C::Channel => Some(JakobsonFunction::Phatic),
            C::Code => Some(JakobsonFunction::Metalingual),
            C::Message => Some(JakobsonFunction::Poetic),
            _ => None,
        }
    }
}

impl Ontology for CommunicationOntology {
    type Cat = CommunicationCategory;
    type Qual = CommunicationFunctionQuality;

    fn axioms() -> Vec<Box<dyn pr4xis::ontology::Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<CommunicationCategory>();
    }

    #[test]
    fn ontology_validates() {
        CommunicationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn eight_concepts() {
        assert_eq!(CommunicationConcept::variants().len(), 8);
    }

    #[test]
    fn sender_produces_message() {
        assert!(CommunicationCategory::morphisms().iter().any(|m| {
            m.source() == CommunicationConcept::Sender
                && m.target() == CommunicationConcept::Message
                && m.kind() == CommunicationRelationKind::Produces
        }));
    }

    #[test]
    fn noise_corrupts_channel() {
        assert!(CommunicationCategory::morphisms().iter().any(|m| {
            m.source() == CommunicationConcept::Noise
                && m.target() == CommunicationConcept::Channel
                && m.kind() == CommunicationRelationKind::Corrupts
        }));
    }

    #[test]
    fn feedback_loop_present() {
        let m = CommunicationCategory::morphisms();
        // Wiener (1948): Receiver → Feedback → Sender.
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

    #[test]
    fn jakobson_six_functions() {
        assert_eq!(JakobsonFunction::variants().len(), 6);
    }

    #[test]
    fn phatic_focuses_on_channel() {
        assert_eq!(
            JakobsonFunction::Phatic.focused_component(),
            CommunicationConcept::Channel
        );
    }

    #[test]
    fn metalingual_focuses_on_code() {
        assert_eq!(
            JakobsonFunction::Metalingual.focused_component(),
            CommunicationConcept::Code
        );
    }

    fn arb_communication() -> impl Strategy<Value = CommunicationConcept> {
        proptest::sample::select(CommunicationConcept::variants())
    }

    fn arb_jakobson() -> impl Strategy<Value = JakobsonFunction> {
        proptest::sample::select(JakobsonFunction::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in CommunicationCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in CommunicationOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_identity_self_morphism(c in arb_communication()) {
            let m = CommunicationCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == c && r.target() == c
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
        fn prop_shannon_chain(_seed in any::<u32>()) {
            // Shannon (1948): Sender → Message → Channel exists.
            let m = CommunicationCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == CommunicationConcept::Sender
                && r.target() == CommunicationConcept::Message));
            prop_assert!(m.iter().any(|r| r.source() == CommunicationConcept::Message
                && r.target() == CommunicationConcept::Channel));
        }
    }
}

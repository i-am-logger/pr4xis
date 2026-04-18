//! Dialogue ontology — the science of conversation.
//!
//! A dialogue IS:
//! - An event-driven system (utterances are events)
//! - Concurrent (two+ agents, turn-taking)
//! - A system (feedback: listen → understand → respond)
//! - Linguistic (uses grammar, semantics, pragmatics)
//!
//! References:
//! - Austin, How to Do Things with Words (1962) — speech acts
//! - Searle, Speech Acts (1969) — illocutionary force
//! - Jurafsky & Martin, Speech and Language Processing — dialogue acts
//! - Traum, A Computational Theory of Grounding (1994)
//! - Ginzburg, The Interactive Stance (2012) — KoS, QUD
//! - Clark, Using Language (1996) — grounding, common ground
//! - Stalnaker, Common Ground (2002) — context set, assertion
//! - Levelt, Speaking (1989) — speech production model
//! - Grice, Logic and Conversation (1975) — cooperative principle

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Ontology, Quality};

pr4xis::ontology! {
    name: "Dialogue",
    source: "Austin (1962); Traum (1994); Clark (1996); Ginzburg (2012); Stalnaker (2002); Levelt (1989)",
    being: Process,

    concepts: [
        Utterance,
        Participant,
        DialogueAct,
        DialogueState,
        Topic,
        History,
        Understanding,
        Generation,
        TurnManagement,
        Grounding,
        QUD,
        CommonGround,
        Intention,
        GroundingAct,
        Repair,
    ],

    labels: {
        Utterance: ("en", "Utterance", "A single utterance from a participant."),
        Participant: ("en", "Participant", "The speaker/listener — an agent in the conversation."),
        DialogueAct: ("en", "Dialogue act", "What the speaker intends to achieve with an utterance (inform, question, request, confirm, deny)."),
        DialogueState: ("en", "Dialogue state", "The shared knowledge between participants at a point in time."),
        Topic: ("en", "Topic", "The topic currently being discussed."),
        History: ("en", "History", "The conversational context — everything said so far."),
        Understanding: ("en", "Understanding", "Parsing, meaning extraction, intent recognition."),
        Generation: ("en", "Generation", "Selecting content, constructing utterance."),
        TurnManagement: ("en", "Turn management", "The mechanism controlling who speaks when."),
        Grounding: ("en", "Grounding", "A successful exchange — both parties understand each other. Traum (1994): Initiate → Acknowledge → Ground."),
        QUD: ("en", "Question Under Discussion", "Ginzburg (2012): priority queue of open questions. Every utterance raises or resolves one."),
        CommonGround: ("en", "Common ground", "Stalnaker (2002): propositions both participants accept as true. Assertion = proposal to add to common ground."),
        Intention: ("en", "Intention", "Levelt (1989): the preverbal message from the Conceptualizer. Contains topic, focus, mood, speech act type, propositional content."),
        GroundingAct: ("en", "Grounding act", "Traum (1994): Initiate, Continue, Acknowledge, Repair, ReqRepair. Clark (1996): presentation + acceptance."),
        Repair: ("en", "Repair", "Schegloff et al. (1977): self-repair, other-repair, other-initiated self-repair."),
    },

    edges: [
        (Participant, Utterance, Produces),
        (Utterance, DialogueAct, Expresses),
        (Utterance, DialogueState, Updates),
        (Utterance, History, AppendedTo),
        (Understanding, Utterance, Interprets),
        (Generation, Utterance, Creates),
        (TurnManagement, Participant, Controls),
        (Understanding, Grounding, ArisesFrom),
        (DialogueAct, Topic, Addresses),
        (Utterance, QUD, RaisesOrResolves),
        (Understanding, CommonGround, EstablishesIn),
        (Intention, Generation, Drives),
        (GroundingAct, Grounding, Achieves),
        (Repair, Understanding, Restores),
        (DialogueState, Intention, FormedFrom),
        (QUD, Intention, FormedFrom),
    ],
}

/// Whether a dialogue concept is agent-facing (produced/consumed by participants).
#[derive(Debug, Clone)]
pub struct IsAgentFacing;

impl Quality for IsAgentFacing {
    type Individual = DialogueConcept;
    type Value = bool;

    fn get(&self, individual: &DialogueConcept) -> Option<bool> {
        match individual {
            DialogueConcept::Participant => Some(true),
            DialogueConcept::Utterance => Some(true),
            DialogueConcept::DialogueAct => Some(true),
            DialogueConcept::Intention => Some(true),
            _ => Some(false),
        }
    }
}

impl Ontology for DialogueOntology {
    type Cat = DialogueCategory;
    type Qual = IsAgentFacing;

    fn structural_axioms() -> Vec<Box<dyn pr4xis::ontology::Axiom>> {
        Self::generated_structural_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<DialogueCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        DialogueOntology::validate().unwrap();
    }
}

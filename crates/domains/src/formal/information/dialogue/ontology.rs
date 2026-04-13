use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Ontology, Quality};

// Dialogue ontology — the science of conversation.
//
// A dialogue IS:
// - An event-driven system (utterances are events)
// - Concurrent (two+ agents, turn-taking)
// - A system (feedback: listen → understand → respond)
// - Linguistic (uses grammar, semantics, pragmatics)
//
// References:
// - Austin, How to Do Things with Words (1962) — speech acts
// - Searle, Speech Acts (1969) — illocutionary force
// - Jurafsky & Martin, Speech and Language Processing — dialogue acts
// - Traum, A Computational Theory of Grounding (1994)
// - Ginzburg, The Interactive Stance (2012) — KoS, QUD
// - Clark, Using Language (1996) — grounding, common ground
// - Stalnaker, Common Ground (2002) — context set, assertion
// - Levelt, Speaking (1989) — speech production model
// - Grice, Logic and Conversation (1975) — cooperative principle

/// Core concepts of a dialogue system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum DialogueConcept {
    /// A single utterance from a participant.
    Utterance,
    /// The speaker/listener — an agent in the conversation.
    Participant,
    /// What the speaker intends to achieve with an utterance.
    /// Inform, question, request, confirm, deny, etc.
    DialogueAct,
    /// The shared knowledge between participants at a point in time.
    /// What's been said, what's understood, what's expected.
    DialogueState,
    /// The topic currently being discussed.
    Topic,
    /// The conversational context — everything said so far.
    History,
    /// Understanding an utterance — parsing, meaning extraction, intent recognition.
    Understanding,
    /// Generating a response — selecting content, constructing utterance.
    Generation,
    /// The mechanism controlling who speaks when.
    TurnManagement,
    /// A successful exchange — both parties understand each other.
    /// Traum (1994): Initiate → Acknowledge → Ground.
    Grounding,

    // === New concepts from literature ===
    /// Questions Under Discussion — the stack that drives interpretation.
    /// Ginzburg (2012): QUD is a priority queue of open questions.
    /// Every utterance either raises a question or resolves one.
    QUD,
    /// Shared beliefs between participants (Stalnaker 2002).
    /// The context set — propositions both participants accept as true.
    /// Assertion = proposal to add to common ground.
    CommonGround,
    /// What the speaker wants to achieve BEFORE formulating words.
    /// Levelt (1989): the preverbal message from the Conceptualizer.
    /// Contains: topic, focus, mood, speech act type, propositional content.
    Intention,
    /// The act of establishing mutual understanding.
    /// Traum (1994): Initiate, Continue, Acknowledge, Repair, ReqRepair.
    /// Clark (1996): presentation + acceptance.
    GroundingAct,
    /// When understanding fails and needs correction.
    /// Schegloff et al. (1977): self-repair, other-repair, other-initiated self-repair.
    Repair,
}

define_ontology! {
    pub DialogueOntology for DialogueCategory {
        concepts: DialogueConcept,
        relation: DialogueRelation,
        kind: DialogueRelationKind,
        kinds: [
            /// Participant produces Utterance.
            Produces,
            /// Utterance expresses DialogueAct.
            Expresses,
            /// Utterance updates DialogueState.
            Updates,
            /// Utterance appended to History.
            AppendedTo,
            /// Understanding interprets Utterance.
            Interprets,
            /// Generation creates Utterance.
            Creates,
            /// TurnManagement controls Participant.
            Controls,
            /// Grounding arises from shared Understanding.
            ArisesFrom,
            /// DialogueAct addresses Topic.
            Addresses,
            /// Utterance raises/resolves QUD (Ginzburg).
            RaisesOrResolves,
            /// Understanding updates CommonGround (Stalnaker).
            EstablishesIn,
            /// Intention drives Generation (Levelt).
            Drives,
            /// GroundingAct achieves Grounding (Traum).
            Achieves,
            /// Repair restores Understanding (Schegloff).
            Restores,
            /// Intention formed from DialogueState + QUD (what to say next).
            FormedFrom,
        ],
        edges: [
            // Participant produces Utterance
            (Participant, Utterance, Produces),
            // Utterance expresses DialogueAct
            (Utterance, DialogueAct, Expresses),
            // Utterance updates DialogueState
            (Utterance, DialogueState, Updates),
            // Utterance appended to History
            (Utterance, History, AppendedTo),
            // Understanding interprets Utterance
            (Understanding, Utterance, Interprets),
            // Generation creates Utterance
            (Generation, Utterance, Creates),
            // TurnManagement controls Participant
            (TurnManagement, Participant, Controls),
            // Grounding arises from Understanding
            (Understanding, Grounding, ArisesFrom),
            // DialogueAct addresses Topic
            (DialogueAct, Topic, Addresses),
            // QUD: utterances raise or resolve questions (Ginzburg 2012)
            (Utterance, QUD, RaisesOrResolves),
            // Understanding establishes facts in CommonGround (Stalnaker 2002)
            (Understanding, CommonGround, EstablishesIn),
            // Intention drives Generation (Levelt 1989)
            (Intention, Generation, Drives),
            // GroundingAct achieves Grounding (Traum 1994)
            (GroundingAct, Grounding, Achieves),
            // Repair restores Understanding (Schegloff 1977)
            (Repair, Understanding, Restores),
            // Intention formed from DialogueState + QUD
            (DialogueState, Intention, FormedFrom),
            (QUD, Intention, FormedFrom),
        ],
        composed: [
            (Participant, DialogueAct),
            (Participant, DialogueState),
            (Participant, History),
            (TurnManagement, Utterance),
            (Understanding, DialogueAct),
            (Generation, DialogueAct),
            // Intention → Utterance (through Generation)
            (Intention, Utterance),
            // DialogueState → Generation (through Intention)
            (DialogueState, Generation),
            // Repair → Grounding (through Understanding → GroundingAct)
            (Repair, Grounding),
        ],
    }
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

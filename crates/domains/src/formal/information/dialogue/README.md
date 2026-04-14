# Dialogue -- Conversation as event-driven concurrent system

Models conversation as a fifteen-concept category combining speech-act theory, dialogue-state tracking, common ground, Questions Under Discussion, and repair. A dialogue is simultaneously event-driven (utterances are events), concurrent (two agents, turn-taking), cybernetic (listen → understand → respond feedback), and linguistic (grammar, semantics, pragmatics). The `engine` and `grounding` modules realize this category at runtime.

Key references:
- Austin 1962: *How to Do Things with Words* (speech acts)
- Searle 1969: *Speech Acts* (illocutionary force)
- Traum 1994: *A Computational Theory of Grounding*
- Clark 1996: *Using Language* (grounding, common ground)
- Stalnaker 2002: *Common Ground* (context set, assertion)
- Ginzburg 2012: *The Interactive Stance* (KoS, Questions Under Discussion)
- Levelt 1989: *Speaking* (Conceptualizer → preverbal message)
- Grice 1975: *Logic and Conversation* (cooperative principle)
- Schegloff, Jefferson & Sacks 1977: *The Preference for Self-Correction* (repair)
- Jurafsky & Martin: *Speech and Language Processing* (dialogue acts)

## Entities (15)

| Category | Entities |
|---|---|
| Utterance and speaker (3) | Utterance, Participant, DialogueAct |
| State and context (4) | DialogueState, Topic, History, QUD |
| Pragmatics (2) | CommonGround, Intention |
| Process (3) | Understanding, Generation, TurnManagement |
| Grounding and repair (3) | Grounding, GroundingAct, Repair |

## Category

Morphisms encode the conversational pipeline: Participant produces Utterance → expresses DialogueAct, updates DialogueState, appends to History, raises/resolves QUD. Understanding interprets Utterance and establishes facts in CommonGround. Intention (formed from DialogueState + QUD) drives Generation, which creates the next Utterance. GroundingAct achieves Grounding; Repair restores Understanding. Composition closes paths like `Intention → Utterance` and `Repair → Grounding`.

## Qualities

| Quality | Type | Description |
|---|---|---|
| IsAgentFacing | bool | Participant, Utterance, DialogueAct, Intention = true; all others = false |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the dialogue kinded relation graph | auto-generated |

## Functors

**Outgoing (1):**

| Functor | Target | File |
|---|---|---|
| DialogueToCommunication | communication | `communication_functor.rs` |

**Incoming (1):**

| Functor | Source | File |
|---|---|---|
| DrtDialogueFunctor | linguistics/pragmatics (DRT) | `../../../cognitive/linguistics/pragmatics/drt_dialogue_functor.rs` |

## Files

- `ontology.rs` -- `DialogueConcept`, dialogue category, IsAgentFacing quality, tests
- `engine.rs` -- Runtime dialogue engine that walks the category
- `grounding.rs` -- Traum/Clark grounding-act and repair implementation
- `communication_functor.rs` -- Dialogue → communication functor
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations

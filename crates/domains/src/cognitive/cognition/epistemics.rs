use pr4xis::category::Entity;
use pr4xis::define_ontology;

// Epistemics — what you know about what you know.
//
// The Rumsfeld matrix formalized as an ontology:
//   Known Known    — knowledge exists AND is accessible
//   Known Unknown  — absence of knowledge is detected
//   Unknown Known  — knowledge exists but is NOT accessible (ontology gap)
//   Unknown Unknown — absence is not even detectable
//
// Second-order cybernetics moves things between these states:
//   Self-observation turns Unknown Unknowns → Known Unknowns
//   Ontology repair turns Unknown Knowns → Known Knowns
//   Learning turns Known Unknowns → Known Knowns
//
// Open World Assumption:
//   Failure to find ≠ falsity. "I don't know" is a valid epistemic state.

/// Epistemic states — what the system knows about its own knowledge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum EpistemicState {
    /// I know it and can access it. "Dogs are mammals" — in taxonomy, query works.
    KnownKnown,
    /// I know I don't know. "What is a quark?" — not in my ontology, I detect the gap.
    KnownUnknown,
    /// I know it but can't access it. "Is a dog a mammal?" — answer exists but grammar can't parse the question.
    UnknownKnown,
    /// I don't know that I don't know. Not even aware of the gap.
    UnknownUnknown,
}

define_ontology! {
    /// Epistemics — second-order knowledge states (von Foerster 1981).
    pub EpistemicOntology for EpistemicCategory {
        concepts: EpistemicState,
        relation: EpistemicTransition,
        kind: TransitionKind,
        kinds: [
            /// Self-observation: detecting that something is missing.
            /// UnknownUnknown → KnownUnknown (I now know I don't know).
            Observation,
            /// Learning: acquiring new knowledge.
            /// KnownUnknown → KnownKnown (I learned the answer).
            Learning,
            /// Repair: fixing an ontology gap that blocked access.
            /// UnknownKnown → KnownKnown (the grammar now parses the question).
            Repair,
            /// Discovery: finding knowledge you didn't know you had.
            /// UnknownKnown → KnownKnown (realized I could answer this all along).
            Discovery,
            /// Forgetting: knowledge becomes inaccessible.
            /// KnownKnown → UnknownKnown (cache expired, index stale).
            Forgetting,
        ],
        edges: [
            // Self-observation: UU → KU (now I know I don't know)
            (UnknownUnknown, KnownUnknown, Observation),
            // Learning: KU → KK (acquired knowledge)
            (KnownUnknown, KnownKnown, Learning),
            // Repair: UK → KK (fixed the access gap)
            (UnknownKnown, KnownKnown, Repair),
            // Discovery: UK → KK (realized I had the answer)
            (UnknownKnown, KnownKnown, Discovery),
            // Forgetting: KK → UK (lost access)
            (KnownKnown, UnknownKnown, Forgetting),
        ],
        composed: [
            // Transitive: UU → KU → KK (observe then learn)
            (UnknownUnknown, KnownKnown),
        ],

        being: MentalObject,
        source: "von Foerster (1981)",
    }
}

/// Classify the epistemic state of a query result.
pub fn classify_result<T>(
    query_parsed: bool,
    knowledge_exists: bool,
    result: Option<T>,
) -> EpistemicState {
    match (query_parsed, knowledge_exists, result.is_some()) {
        (true, true, true) => EpistemicState::KnownKnown,
        (true, false, false) => EpistemicState::KnownUnknown,
        (false, true, false) => EpistemicState::UnknownKnown,
        (false, false, false) => EpistemicState::UnknownUnknown,
        // Edge cases
        (true, true, false) => EpistemicState::UnknownKnown, // parsed but couldn't retrieve
        (true, false, true) => EpistemicState::KnownKnown, // shouldn't happen but handle gracefully
        (false, true, true) => EpistemicState::KnownKnown, // got result despite parse failure
        (false, false, true) => EpistemicState::KnownKnown, // shouldn't happen
    }
}

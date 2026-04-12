use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for EpistemicState {
    fn variants() -> Vec<Self> {
        vec![
            Self::KnownKnown,
            Self::KnownUnknown,
            Self::UnknownKnown,
            Self::UnknownUnknown,
        ]
    }
}

/// Transitions between epistemic states — how knowledge awareness changes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EpistemicTransition {
    pub from: EpistemicState,
    pub to: EpistemicState,
    pub kind: TransitionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitionKind {
    Identity,
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
    Composed,
}

impl Relationship for EpistemicTransition {
    type Object = EpistemicState;
    fn source(&self) -> EpistemicState {
        self.from
    }
    fn target(&self) -> EpistemicState {
        self.to
    }
}

pub struct EpistemicCategory;

impl Category for EpistemicCategory {
    type Object = EpistemicState;
    type Morphism = EpistemicTransition;

    fn identity(obj: &EpistemicState) -> EpistemicTransition {
        EpistemicTransition {
            from: *obj,
            to: *obj,
            kind: TransitionKind::Identity,
        }
    }

    fn compose(f: &EpistemicTransition, g: &EpistemicTransition) -> Option<EpistemicTransition> {
        if f.to != g.from {
            return None;
        }
        if f.kind == TransitionKind::Identity {
            return Some(g.clone());
        }
        if g.kind == TransitionKind::Identity {
            return Some(f.clone());
        }
        Some(EpistemicTransition {
            from: f.from,
            to: g.to,
            kind: TransitionKind::Composed,
        })
    }

    fn morphisms() -> Vec<EpistemicTransition> {
        use EpistemicState::*;
        use TransitionKind::*;

        let mut m = Vec::new();

        for s in EpistemicState::variants() {
            m.push(EpistemicTransition {
                from: s,
                to: s,
                kind: Identity,
            });
        }

        // Self-observation: UU → KU (now I know I don't know)
        m.push(EpistemicTransition {
            from: UnknownUnknown,
            to: KnownUnknown,
            kind: Observation,
        });
        // Learning: KU → KK (acquired knowledge)
        m.push(EpistemicTransition {
            from: KnownUnknown,
            to: KnownKnown,
            kind: Learning,
        });
        // Repair: UK → KK (fixed the access gap)
        m.push(EpistemicTransition {
            from: UnknownKnown,
            to: KnownKnown,
            kind: Repair,
        });
        // Discovery: UK → KK (realized I had the answer)
        m.push(EpistemicTransition {
            from: UnknownKnown,
            to: KnownKnown,
            kind: Discovery,
        });
        // Forgetting: KK → UK (lost access)
        m.push(EpistemicTransition {
            from: KnownKnown,
            to: UnknownKnown,
            kind: Forgetting,
        });

        // Transitive: UU → KU → KK (observe then learn)
        m.push(EpistemicTransition {
            from: UnknownUnknown,
            to: KnownKnown,
            kind: Composed,
        });

        // Self-composed
        for s in EpistemicState::variants() {
            m.push(EpistemicTransition {
                from: s,
                to: s,
                kind: Composed,
            });
        }

        m
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

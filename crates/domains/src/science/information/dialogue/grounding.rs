use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

// Grounding — the process of establishing mutual understanding.
//
// A finite state machine category: objects are grounding states,
// morphisms are grounding acts that transition between states.
//
// The "clean path" is: S → Initiated → Acknowledged → Grounded
// Repair: Initiated → RepairRequested → Initiated → ... → Grounded
// Failure: any → Dead (abandoned)
//
// References:
// - Traum, "A Computational Theory of Grounding in Natural Language
//   Conversation" (1994), TR 545, U. Rochester
// - Clark & Schaefer, "Collaborating on Contributions to Conversations" (1987)
// - Clark, "Using Language" (1996), Ch. 8 — contribution = presentation + acceptance

/// States of a discourse unit's grounding lifecycle.
///
/// Every contribution passes through these states. The DU starts
/// at S and must reach Grounded (F) for the content to enter
/// common ground. If it reaches Dead (D), the content is abandoned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroundingState {
    /// Initial state — no discourse unit initiated yet.
    Start,
    /// Content has been presented by one participant.
    /// Awaiting acknowledgment from the other.
    Initiated,
    /// The other participant has requested clarification/repair.
    /// Awaiting the presenter's correction.
    RepairRequested,
    /// The other participant has acknowledged understanding.
    /// Grounding criteria being established.
    Acknowledged,
    /// Fully grounded — both participants mutually believe
    /// that they understand the content sufficiently for current purposes.
    /// Terminal success state.
    Grounded,
    /// Abandoned — the discourse unit was cancelled without grounding.
    /// Terminal failure state.
    Dead,
}

impl Entity for GroundingState {
    fn variants() -> Vec<Self> {
        vec![
            Self::Start,
            Self::Initiated,
            Self::RepairRequested,
            Self::Acknowledged,
            Self::Grounded,
            Self::Dead,
        ]
    }
}

/// Grounding acts — actions that move a discourse unit through its lifecycle.
///
/// Each act transitions the DU from one state to another.
/// The taxonomy is from Traum (1994).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroundingTransition {
    pub from: GroundingState,
    pub to: GroundingState,
    pub act: GroundingAct,
}

/// The seven grounding acts from Traum (1994).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroundingAct {
    /// Identity — no action.
    Identity,
    /// Start a new discourse unit (first utterance of a contribution).
    Initiate,
    /// Extend the current DU from the same speaker (continuation, parenthetical).
    Continue,
    /// Signal understanding of the DU content. Moves toward grounded.
    /// Evidence types (Clark 1996): verbatim > paraphrase > relevant next > ack token > attention.
    Acknowledge,
    /// Correct or replace content of the DU (self-repair or other-repair).
    Repair,
    /// Request the other party to repair (signal non-understanding).
    /// "What do you mean?" / "Could you repeat that?"
    RequestRepair,
    /// Request explicit acknowledgment. "Do you follow me?"
    RequestAck,
    /// Abort the DU without grounding. Topic abandoned.
    Cancel,
    /// Composed transition (multiple acts).
    Composed,
}

impl Relationship for GroundingTransition {
    type Object = GroundingState;
    fn source(&self) -> GroundingState {
        self.from
    }
    fn target(&self) -> GroundingState {
        self.to
    }
}

pub struct GroundingCategory;

impl Category for GroundingCategory {
    type Object = GroundingState;
    type Morphism = GroundingTransition;

    fn identity(obj: &GroundingState) -> GroundingTransition {
        GroundingTransition {
            from: *obj,
            to: *obj,
            act: GroundingAct::Identity,
        }
    }

    fn compose(f: &GroundingTransition, g: &GroundingTransition) -> Option<GroundingTransition> {
        if f.to != g.from {
            return None;
        }
        if f.act == GroundingAct::Identity {
            return Some(g.clone());
        }
        if g.act == GroundingAct::Identity {
            return Some(f.clone());
        }
        Some(GroundingTransition {
            from: f.from,
            to: g.to,
            act: GroundingAct::Composed,
        })
    }

    fn morphisms() -> Vec<GroundingTransition> {
        use GroundingAct as A;
        use GroundingState as S;

        let mut m = Vec::new();

        // Identities
        for s in GroundingState::variants() {
            m.push(GroundingTransition {
                from: s,
                to: s,
                act: A::Identity,
            });
        }

        // S → Initiated (start a new contribution)
        m.push(GroundingTransition {
            from: S::Start,
            to: S::Initiated,
            act: A::Initiate,
        });

        // Initiated → Initiated (extend from same speaker)
        m.push(GroundingTransition {
            from: S::Initiated,
            to: S::Initiated,
            act: A::Continue,
        });

        // Initiated → Acknowledged (addressee signals understanding)
        m.push(GroundingTransition {
            from: S::Initiated,
            to: S::Acknowledged,
            act: A::Acknowledge,
        });

        // Initiated → RepairRequested (addressee signals non-understanding)
        m.push(GroundingTransition {
            from: S::Initiated,
            to: S::RepairRequested,
            act: A::RequestRepair,
        });

        // Initiated → Initiated (self-repair by presenter)
        m.push(GroundingTransition {
            from: S::Initiated,
            to: S::Initiated,
            act: A::Repair,
        });

        // Initiated → Dead (cancel without grounding)
        m.push(GroundingTransition {
            from: S::Initiated,
            to: S::Dead,
            act: A::Cancel,
        });

        // RepairRequested → Initiated (presenter repairs)
        m.push(GroundingTransition {
            from: S::RepairRequested,
            to: S::Initiated,
            act: A::Repair,
        });

        // RepairRequested → Dead (give up)
        m.push(GroundingTransition {
            from: S::RepairRequested,
            to: S::Dead,
            act: A::Cancel,
        });

        // Acknowledged → Grounded (mutual belief established)
        m.push(GroundingTransition {
            from: S::Acknowledged,
            to: S::Grounded,
            act: A::Acknowledge,
        });

        // Acknowledged → Initiated (re-opened for correction)
        m.push(GroundingTransition {
            from: S::Acknowledged,
            to: S::Initiated,
            act: A::Repair,
        });

        // Initiated → Initiated (request ack is a continuation)
        m.push(GroundingTransition {
            from: S::Initiated,
            to: S::Initiated,
            act: A::RequestAck,
        });

        // Composed transitions (transitive paths)
        // Start → Grounded (clean path: Initiate then Acknowledge then Acknowledge)
        m.push(GroundingTransition {
            from: S::Start,
            to: S::Grounded,
            act: A::Composed,
        });
        // Start → Dead (initiate then cancel)
        m.push(GroundingTransition {
            from: S::Start,
            to: S::Dead,
            act: A::Composed,
        });
        // Start → Acknowledged (initiate then ack)
        m.push(GroundingTransition {
            from: S::Start,
            to: S::Acknowledged,
            act: A::Composed,
        });
        // RepairRequested → Grounded (repair then ack)
        m.push(GroundingTransition {
            from: S::RepairRequested,
            to: S::Grounded,
            act: A::Composed,
        });

        // Self-composed
        for s in GroundingState::variants() {
            m.push(GroundingTransition {
                from: s,
                to: s,
                act: A::Composed,
            });
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::Category;
    use praxis::category::entity::Entity;

    #[test]
    fn category_identity_law() {
        for s in GroundingState::variants() {
            let id = GroundingCategory::identity(&s);
            assert_eq!(id.from, s);
            assert_eq!(id.to, s);
            assert_eq!(id.act, GroundingAct::Identity);
        }
    }

    #[test]
    fn category_composition_with_identity() {
        for m in &GroundingCategory::morphisms() {
            let left =
                GroundingCategory::compose(&GroundingCategory::identity(&m.from), m).unwrap();
            assert_eq!(left.from, m.from);
            assert_eq!(left.to, m.to);
        }
    }

    #[test]
    fn has_six_states() {
        assert_eq!(GroundingState::variants().len(), 6);
    }

    #[test]
    fn clean_path_exists() {
        // The happy path: Start → Initiated → Acknowledged → Grounded
        let morphisms = GroundingCategory::morphisms();
        assert!(morphisms.iter().any(|m| m.from == GroundingState::Start
            && m.to == GroundingState::Initiated
            && m.act == GroundingAct::Initiate));
        assert!(morphisms.iter().any(|m| m.from == GroundingState::Initiated
            && m.to == GroundingState::Acknowledged
            && m.act == GroundingAct::Acknowledge));
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == GroundingState::Acknowledged
                    && m.to == GroundingState::Grounded
                    && m.act == GroundingAct::Acknowledge)
        );
    }

    #[test]
    fn clean_path_composes() {
        // Start → Initiated → Acknowledged composes
        let initiate = GroundingTransition {
            from: GroundingState::Start,
            to: GroundingState::Initiated,
            act: GroundingAct::Initiate,
        };
        let acknowledge = GroundingTransition {
            from: GroundingState::Initiated,
            to: GroundingState::Acknowledged,
            act: GroundingAct::Acknowledge,
        };
        let composed = GroundingCategory::compose(&initiate, &acknowledge).unwrap();
        assert_eq!(composed.from, GroundingState::Start);
        assert_eq!(composed.to, GroundingState::Acknowledged);
    }

    #[test]
    fn repair_path_exists() {
        // Initiated → RepairRequested → Initiated (repair loop)
        let morphisms = GroundingCategory::morphisms();
        assert!(morphisms.iter().any(|m| m.from == GroundingState::Initiated
            && m.to == GroundingState::RepairRequested
            && m.act == GroundingAct::RequestRepair));
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == GroundingState::RepairRequested
                    && m.to == GroundingState::Initiated
                    && m.act == GroundingAct::Repair)
        );
    }

    #[test]
    fn grounded_is_terminal() {
        // No non-identity morphisms FROM Grounded
        let morphisms = GroundingCategory::morphisms();
        let exits: Vec<_> = morphisms
            .iter()
            .filter(|m| {
                m.from == GroundingState::Grounded
                    && m.to != GroundingState::Grounded
                    && m.act != GroundingAct::Identity
                    && m.act != GroundingAct::Composed
            })
            .collect();
        assert!(
            exits.is_empty(),
            "Grounded should be terminal but has exits: {:?}",
            exits
        );
    }

    #[test]
    fn dead_is_terminal() {
        // No non-identity morphisms FROM Dead
        let morphisms = GroundingCategory::morphisms();
        let exits: Vec<_> = morphisms
            .iter()
            .filter(|m| {
                m.from == GroundingState::Dead
                    && m.to != GroundingState::Dead
                    && m.act != GroundingAct::Identity
                    && m.act != GroundingAct::Composed
            })
            .collect();
        assert!(
            exits.is_empty(),
            "Dead should be terminal but has exits: {:?}",
            exits
        );
    }

    #[test]
    fn cancel_always_leads_to_dead() {
        let morphisms = GroundingCategory::morphisms();
        let cancels: Vec<_> = morphisms
            .iter()
            .filter(|m| m.act == GroundingAct::Cancel)
            .collect();
        for c in &cancels {
            assert_eq!(
                c.to,
                GroundingState::Dead,
                "Cancel from {:?} should lead to Dead, not {:?}",
                c.from,
                c.to
            );
        }
    }
}

use pr4xis::category::Entity;
use pr4xis::define_category;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
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

define_category! {
    pub GroundingCategory {
        entity: GroundingState,
        relation: GroundingTransition,
        kind: GroundingTransitionKind,
        kinds: [
            /// Start a new discourse unit.
            Initiate,
            /// Extend the current DU from the same speaker.
            Continue,
            /// Signal understanding of the DU content.
            Acknowledge,
            /// Correct or replace content of the DU.
            Repair,
            /// Request the other party to repair.
            RequestRepair,
            /// Request explicit acknowledgment.
            RequestAck,
            /// Abort the DU without grounding.
            Cancel,
        ],
        edges: [
            // S → Initiated (start a new contribution)
            (Start, Initiated, Initiate),
            // Initiated → Initiated (extend from same speaker)
            (Initiated, Initiated, Continue),
            // Initiated → Acknowledged (addressee signals understanding)
            (Initiated, Acknowledged, Acknowledge),
            // Initiated → RepairRequested (addressee signals non-understanding)
            (Initiated, RepairRequested, RequestRepair),
            // Initiated → Initiated (self-repair by presenter)
            (Initiated, Initiated, Repair),
            // Initiated → Dead (cancel without grounding)
            (Initiated, Dead, Cancel),
            // RepairRequested → Initiated (presenter repairs)
            (RepairRequested, Initiated, Repair),
            // RepairRequested → Dead (give up)
            (RepairRequested, Dead, Cancel),
            // Acknowledged → Grounded (mutual belief established)
            (Acknowledged, Grounded, Acknowledge),
            // Acknowledged → Initiated (re-opened for correction)
            (Acknowledged, Initiated, Repair),
            // Initiated → Initiated (request ack is a continuation)
            (Initiated, Initiated, RequestAck),
        ],
        composed: [
            // Start → Grounded (clean path: Initiate then Acknowledge then Acknowledge)
            (Start, Grounded),
            // Start → Dead (initiate then cancel)
            (Start, Dead),
            // Start → Acknowledged (initiate then ack)
            (Start, Acknowledged),
            // RepairRequested → Grounded (repair then ack)
            (RepairRequested, Grounded),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::Entity;

    #[test]
    fn category_identity_law() {
        for s in GroundingState::variants() {
            let id = GroundingCategory::identity(&s);
            assert_eq!(id.from, s);
            assert_eq!(id.to, s);
            assert_eq!(id.kind, GroundingTransitionKind::Identity);
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
            && m.kind == GroundingTransitionKind::Initiate));
        assert!(morphisms.iter().any(|m| m.from == GroundingState::Initiated
            && m.to == GroundingState::Acknowledged
            && m.kind == GroundingTransitionKind::Acknowledge));
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == GroundingState::Acknowledged
                    && m.to == GroundingState::Grounded
                    && m.kind == GroundingTransitionKind::Acknowledge)
        );
    }

    #[test]
    fn clean_path_composes() {
        // Start → Initiated → Acknowledged composes
        let initiate = GroundingTransition {
            from: GroundingState::Start,
            to: GroundingState::Initiated,
            kind: GroundingTransitionKind::Initiate,
        };
        let acknowledge = GroundingTransition {
            from: GroundingState::Initiated,
            to: GroundingState::Acknowledged,
            kind: GroundingTransitionKind::Acknowledge,
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
            && m.kind == GroundingTransitionKind::RequestRepair));
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == GroundingState::RepairRequested
                    && m.to == GroundingState::Initiated
                    && m.kind == GroundingTransitionKind::Repair)
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
                    && m.kind != GroundingTransitionKind::Identity
                    && m.kind != GroundingTransitionKind::Composed
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
                    && m.kind != GroundingTransitionKind::Identity
                    && m.kind != GroundingTransitionKind::Composed
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
            .filter(|m| m.kind == GroundingTransitionKind::Cancel)
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

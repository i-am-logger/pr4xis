use pr4xis::ontology::Axiom;

use crate::social::compliance::classification::*;
use crate::social::compliance::escalation::*;

/// Geneva Convention principle of distinction (Protocol I, Art. 48).
///
/// "The Parties to the conflict shall at all times distinguish between
/// the civilian population and combatants and between civilian objects
/// and military objectives and accordingly shall direct their operations
/// only against military objectives."
pub struct DistinctionPrinciple;

impl Axiom for DistinctionPrinciple {
    fn description(&self) -> &str {
        "LOAC distinction: engagement requires MilitaryObjective classification (Protocol I, Art. 48)"
    }

    fn holds(&self) -> bool {
        // A protected entity cannot reach Engage
        let protected = ClassifiedEntity {
            entity_type: EntityType::Person,
            iff: IffClassification::Unknown,
            protected_status: ProtectedStatus::Protected,
            confidence: Confidence::High,
        };
        can_transition(
            EscalationLevel::WarningAction,
            EscalationLevel::Engage,
            &protected,
            Authorization::CommanderAuthorized,
        )
        .is_err()
    }
}
pr4xis::register_axiom!(DistinctionPrinciple);

/// Presumption of civilian status (Protocol I, Art. 50(1)).
///
/// "In case of doubt whether a person is a civilian, that person
/// shall be considered to be a civilian."
pub struct CivilianPresumption;

impl Axiom for CivilianPresumption {
    fn description(&self) -> &str {
        "LOAC presumption: unknown persons are assumed civilian (Protocol I, Art. 50(1))"
    }

    fn holds(&self) -> bool {
        let unknown_person = ClassifiedEntity {
            entity_type: EntityType::Person,
            iff: IffClassification::Unknown,
            protected_status: ProtectedStatus::Undetermined,
            confidence: Confidence::None,
        };
        // Unknown person must NOT meet engagement threshold
        !unknown_person.meets_engagement_threshold()
    }
}
pr4xis::register_axiom!(CivilianPresumption);

/// Persons require human positive identification for engagement.
///
/// DoD Directive 3000.09: autonomous weapon systems require
/// "appropriate levels of human judgment over the use of force."
/// For persons, this means human PositiveId, not sensor classification.
pub struct HumanInTheLoop;

impl Axiom for HumanInTheLoop {
    fn description(&self) -> &str {
        "persons require human PositiveId for engagement (DoD Directive 3000.09)"
    }

    fn holds(&self) -> bool {
        // Even a hostile-classified person with High (but not PositiveId) confidence cannot be engaged
        let hostile_person = ClassifiedEntity {
            entity_type: EntityType::Person,
            iff: IffClassification::Hostile,
            protected_status: ProtectedStatus::MilitaryObjective,
            confidence: Confidence::High, // not PositiveId
        };
        can_transition(
            EscalationLevel::WarningAction,
            EscalationLevel::Engage,
            &hostile_person,
            Authorization::CommanderAuthorized,
        )
        .is_err()
    }
}
pr4xis::register_axiom!(HumanInTheLoop);

/// Escalation is sequential — cannot skip force levels.
///
/// Protocol I, Art. 57(2)(a)(ii): "take all feasible precautions
/// in the choice of means and methods of attack with a view to
/// avoiding, and in any event to minimizing, incidental loss."
pub struct SequentialEscalation;

impl Axiom for SequentialEscalation {
    fn description(&self) -> &str {
        "escalation must be sequential — no skipping levels (Protocol I, Art. 57)"
    }

    fn holds(&self) -> bool {
        let target = ClassifiedEntity {
            entity_type: EntityType::GroundVehicle,
            iff: IffClassification::Hostile,
            protected_status: ProtectedStatus::MilitaryObjective,
            confidence: Confidence::High,
        };
        // Cannot jump from Observe directly to Engage
        can_transition(
            EscalationLevel::Observe,
            EscalationLevel::Engage,
            &target,
            Authorization::CommanderAuthorized,
        )
        .is_err()
    }
}
pr4xis::register_axiom!(SequentialEscalation);

/// Advance warning requirement (Protocol I, Art. 57(2)(c)).
///
/// "effective advance warning shall be given of attacks which may
/// affect the civilian population, unless circumstances do not permit."
pub struct AdvanceWarning;

impl Axiom for AdvanceWarning {
    fn description(&self) -> &str {
        "advance warning before engagement when feasible (Protocol I, Art. 57(2)(c))"
    }

    fn holds(&self) -> bool {
        let target = ClassifiedEntity {
            entity_type: EntityType::GroundVehicle,
            iff: IffClassification::Hostile,
            protected_status: ProtectedStatus::MilitaryObjective,
            confidence: Confidence::High,
        };
        // Cannot go from Alert directly to Engage (must pass through Warn, ShowForce, etc.)
        can_transition(
            EscalationLevel::Alert,
            EscalationLevel::Engage,
            &target,
            Authorization::CommanderAuthorized,
        )
        .is_err()
    }
}
pr4xis::register_axiom!(AdvanceWarning);

/// Abort is always available from any state.
///
/// A system must always be able to abort. There is no state
/// from which de-escalation is impossible.
pub struct AbortAlwaysAvailable;

impl Axiom for AbortAlwaysAvailable {
    fn description(&self) -> &str {
        "abort is always available from any escalation level"
    }

    fn holds(&self) -> bool {
        let entity = ClassifiedEntity::unknown();
        let levels = [
            EscalationLevel::Observe,
            EscalationLevel::Identify,
            EscalationLevel::Classify,
            EscalationLevel::Alert,
            EscalationLevel::Warn,
            EscalationLevel::ShowForce,
            EscalationLevel::NonLethal,
            EscalationLevel::WarningAction,
            EscalationLevel::Engage,
        ];
        for &level in &levels {
            if can_transition(
                level,
                EscalationLevel::Abort,
                &entity,
                Authorization::Autonomous,
            )
            .is_err()
            {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(AbortAlwaysAvailable);

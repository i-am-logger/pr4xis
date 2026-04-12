use crate::governance::compliance::classification::{ClassifiedEntity, Confidence};
use pr4xis::category::Entity;

/// Escalation of Force (EOF) levels.
///
/// Each level is a state in the engagement Engine. The system enforces
/// sequential progression — you CANNOT skip levels.
///
/// Based on US military EOF doctrine and NATO ROE framework.
///
/// Sources:
///   - US Army FM 5-19 "Composite Risk Management"
///   - NATO MC 362/1 "Rules of Engagement"
///   - DoD Directive 3000.09 "Autonomy in Weapon Systems" (2023)
///   - LOAC Protocol I, Art. 57 "Precautions in attack"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum EscalationLevel {
    /// Sensor observation only. No interaction with the entity.
    /// Always permitted. No authorization required.
    Observe,

    /// Entity has been detected and recognized (type classified).
    /// IFF process initiated.
    Identify,

    /// Entity classified (IFF + protected status determined).
    /// Human operator alerted if entity is Person or ProtectedObject.
    Classify,

    /// Alert: notify operator/chain of command.
    /// Required before any further escalation.
    Alert,

    /// Verbal/visual warning to the entity (if possible).
    /// "Shout" — attempt to communicate intent.
    /// Protocol I Art. 57(2)(c): "effective advance warning shall be given."
    Warn,

    /// Show of force: demonstrate capability without causing harm.
    /// Deterrent action.
    ShowForce,

    /// Non-lethal measures: barriers, deterrents, disabling.
    NonLethal,

    /// Warning action (e.g., warning shot).
    /// Requires explicit human authorization.
    WarningAction,

    /// Engagement: use of force. LAST RESORT.
    /// Requires: human authorization + PositiveId + MilitaryObjective + proportionality.
    Engage,

    /// De-escalation: threat has ceased or entity surrendered.
    /// System returns to Observe.
    Deescalate,

    /// Abort: engagement cancelled at any point.
    /// Always available. Immediately returns to Observe.
    Abort,
}

/// Authorization level required for each escalation level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Authorization {
    /// System can perform autonomously (Observe, Identify).
    Autonomous,
    /// Operator must be notified (Classify, Alert).
    OperatorNotified,
    /// Operator must explicitly approve (Warn, ShowForce, NonLethal).
    OperatorApproved,
    /// Commander authorization required (WarningAction, Engage).
    CommanderAuthorized,
}

impl Authorization {
    /// Explicit numeric authority level for reliable comparison.
    pub fn level(&self) -> u8 {
        match self {
            Self::Autonomous => 0,
            Self::OperatorNotified => 1,
            Self::OperatorApproved => 2,
            Self::CommanderAuthorized => 3,
        }
    }
}

/// What authorization level does this escalation level require?
///
/// DoD Directive 3000.09: autonomous systems shall be designed to allow
/// "appropriate levels of human judgment over the use of force."
pub fn required_authorization(level: EscalationLevel) -> Authorization {
    match level {
        EscalationLevel::Observe | EscalationLevel::Identify => Authorization::Autonomous,
        EscalationLevel::Classify | EscalationLevel::Alert => Authorization::OperatorNotified,
        EscalationLevel::Warn | EscalationLevel::ShowForce | EscalationLevel::NonLethal => {
            Authorization::OperatorApproved
        }
        EscalationLevel::WarningAction | EscalationLevel::Engage => {
            Authorization::CommanderAuthorized
        }
        EscalationLevel::Deescalate | EscalationLevel::Abort => Authorization::Autonomous,
    }
}

/// Can we transition from one escalation level to the next?
///
/// The escalation ladder is strictly sequential upward.
/// De-escalation and abort are always available from any level.
pub fn can_transition(
    from: EscalationLevel,
    to: EscalationLevel,
    entity: &ClassifiedEntity,
    authorization: Authorization,
) -> Result<(), EscalationDenial> {
    // Abort and de-escalate always permitted
    if to == EscalationLevel::Abort || to == EscalationLevel::Deescalate {
        return Ok(());
    }

    // Check authorization level
    let required = required_authorization(to);
    if authorization.level() < required.level() {
        return Err(EscalationDenial::InsufficientAuthorization {
            required,
            provided: authorization,
        });
    }

    // Person-specific rules: engagement of persons ALWAYS requires PositiveId
    if entity.is_person()
        && to == EscalationLevel::Engage
        && entity.confidence < Confidence::PositiveId
    {
        return Err(EscalationDenial::PersonRequiresPositiveId);
    }

    // Cannot engage protected entities
    if entity.is_protected() && to == EscalationLevel::Engage {
        return Err(EscalationDenial::ProtectedEntity);
    }

    // Must be classified as MilitaryObjective + Hostile for engagement
    if to == EscalationLevel::Engage && !entity.meets_engagement_threshold() {
        if entity.is_person() {
            return Err(EscalationDenial::PersonRequiresPositiveId);
        }
        return Err(EscalationDenial::ClassificationInsufficient);
    }

    // Sequential escalation (cannot skip levels going up)
    let from_ord = escalation_order(from);
    let to_ord = escalation_order(to);
    if to_ord > from_ord + 1 {
        return Err(EscalationDenial::SkippedLevel { from, to });
    }

    Ok(())
}

/// Denial reasons — why escalation was blocked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscalationDenial {
    /// Authorization level too low.
    InsufficientAuthorization {
        required: Authorization,
        provided: Authorization,
    },
    /// Person detected — requires human PositiveId, not sensor classification.
    PersonRequiresPositiveId,
    /// Entity has protected status under LOAC.
    ProtectedEntity,
    /// Classification does not meet engagement threshold.
    ClassificationInsufficient,
    /// Attempted to skip escalation levels.
    SkippedLevel {
        from: EscalationLevel,
        to: EscalationLevel,
    },
}

fn escalation_order(level: EscalationLevel) -> u8 {
    match level {
        EscalationLevel::Observe => 0,
        EscalationLevel::Identify => 1,
        EscalationLevel::Classify => 2,
        EscalationLevel::Alert => 3,
        EscalationLevel::Warn => 4,
        EscalationLevel::ShowForce => 5,
        EscalationLevel::NonLethal => 6,
        EscalationLevel::WarningAction => 7,
        EscalationLevel::Engage => 8,
        EscalationLevel::Deescalate => 0,
        EscalationLevel::Abort => 0,
    }
}

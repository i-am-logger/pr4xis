#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::lifecycle::{Case, CaseAction, PhaseTag};
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

impl Situation for Case {
    fn describe(&self) -> String {
        format!(
            "{} | {:?} | {} motions | {} events",
            self.caption,
            self.phase.tag(),
            self.motions.len(),
            self.events.len()
        )
    }

    fn is_terminal(&self) -> bool {
        self.phase.is_terminal()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegalAction(pub CaseAction);

impl Action for LegalAction {
    type Sit = Case;

    fn describe(&self) -> String {
        match &self.0 {
            CaseAction::File { date, .. } => format!("file case ({})", date),
            CaseAction::BeginDiscovery { date } => format!("begin discovery ({})", date),
            CaseAction::FileMotion { .. } => "file motion".into(),
            CaseAction::RuleOnMotion { motion_index, .. } => {
                format!("rule on motion {}", motion_index)
            }
            CaseAction::SetForTrial { date } => format!("set trial ({})", date),
            CaseAction::BeginTrial { date } => format!("begin trial ({})", date),
            CaseAction::Verdict { outcome, .. } => format!("verdict: {}", outcome),
            CaseAction::Appeal { .. } => "appeal".into(),
            CaseAction::Settle { terms, .. } => format!("settle: {}", terms),
            CaseAction::Dismiss { reason, .. } => format!("dismiss: {}", reason),
        }
    }
}

/// Validates that the action is valid for the current case phase.
pub struct PhaseTransition;

impl PhaseTransition {
    fn required_phase(action: &CaseAction) -> Option<PhaseTag> {
        match action {
            CaseAction::File { .. } => Some(PhaseTag::PreFiling),
            CaseAction::BeginTrial { .. } => Some(PhaseTag::PreTrial),
            CaseAction::Verdict { .. } => Some(PhaseTag::Trial),
            CaseAction::Appeal { .. } => Some(PhaseTag::PostTrial),
            _ => None, // multi-phase actions validated by valid_transitions
        }
    }

    fn target_phase(action: &CaseAction) -> Option<PhaseTag> {
        match action {
            CaseAction::File { .. } => Some(PhaseTag::Filed),
            CaseAction::BeginDiscovery { .. } => Some(PhaseTag::Discovery),
            CaseAction::SetForTrial { .. } => Some(PhaseTag::PreTrial),
            CaseAction::BeginTrial { .. } => Some(PhaseTag::Trial),
            CaseAction::Verdict { .. } => Some(PhaseTag::PostTrial),
            CaseAction::Appeal { .. } => Some(PhaseTag::Appeal),
            CaseAction::Settle { .. } | CaseAction::Dismiss { .. } => Some(PhaseTag::Closed),
            CaseAction::FileMotion { .. } | CaseAction::RuleOnMotion { .. } => None,
        }
    }
}

impl Precondition<LegalAction> for PhaseTransition {
    fn check(&self, case: &Case, action: &LegalAction) -> PreconditionResult {
        let current = case.phase.tag();

        if case.phase.is_terminal() {
            return PreconditionResult::violated(
                "phase_transition",
                "case is closed — no further actions allowed",
                &case.describe(),
                &action.describe(),
            );
        }

        // Check strict phase requirement
        if let Some(required) = Self::required_phase(&action.0)
            && current != required
        {
            return PreconditionResult::violated(
                "phase_transition",
                &format!("requires {:?} phase but case is in {:?}", required, current),
                &case.describe(),
                &action.describe(),
            );
        }

        // Check valid transition target
        if let Some(target) = Self::target_phase(&action.0)
            && target != PhaseTag::Closed
            && !current.valid_transitions().contains(&target)
        {
            return PreconditionResult::violated(
                "phase_transition",
                &format!("cannot transition from {:?} to {:?}", current, target),
                &case.describe(),
                &action.describe(),
            );
        }

        // Motion-specific: filing requires Filed/Discovery/Motions
        if let CaseAction::FileMotion { .. } = &action.0
            && !matches!(
                current,
                PhaseTag::Filed | PhaseTag::Discovery | PhaseTag::Motions
            )
        {
            return PreconditionResult::violated(
                "phase_transition",
                &format!("cannot file motion in {:?} phase", current),
                &case.describe(),
                &action.describe(),
            );
        }

        PreconditionResult::satisfied(
            "phase_transition",
            &format!(
                "{:?} action valid in {:?} phase",
                action.describe(),
                current
            ),
        )
    }

    fn describe(&self) -> &str {
        "action must be valid for the current case phase"
    }
}

fn apply_legal(case: &Case, action: &LegalAction) -> Result<Case, String> {
    let mut next = case.clone();
    next.act(action.0.clone());
    Ok(next)
}

pub type LegalEngine = Engine<LegalAction>;

pub fn new_case(caption: &str) -> LegalEngine {
    Engine::new(
        Case::new(caption),
        vec![Box::new(PhaseTransition)],
        apply_legal,
    )
}

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::lifecycle::{Case, CaseAction, PhaseTag};
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

fn axiom_meta(name: &'static str, description: &'static str, citation: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(citation),
        module_path: ModulePath::new_static(module_path!()),
    }
}

impl Situation for Case {}

#[derive(Debug, Clone, PartialEq)]
pub struct LegalAction(pub CaseAction);

impl Action for LegalAction {
    type Sit = Case;
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
    fn check(&self, case: &Case, action: &LegalAction) -> Verdict {
        let meta = axiom_meta(
            "phase_transition",
            "action must be valid for the current case phase",
            "Federal Rules of Civil Procedure (FRCP, as amended); Federal Rules of Appellate Procedure (FRAP); Wright & Miller, Federal Practice and Procedure",
        );
        let current = case.phase.tag();

        if case.phase.is_terminal() {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }

        if let Some(required) = Self::required_phase(&action.0)
            && current != required
        {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }

        if let Some(target) = Self::target_phase(&action.0)
            && target != PhaseTag::Closed
            && !current.valid_transitions().contains(&target)
        {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }

        if let CaseAction::FileMotion { .. } = &action.0
            && !matches!(
                current,
                PhaseTag::Filed | PhaseTag::Discovery | PhaseTag::Motions
            )
        {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }

        Ok(Box::new(SimpleProof::new(meta)))
    }
}

fn apply_legal(case: &Case, action: &LegalAction) -> Result<Case, Box<dyn Counterexample>> {
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

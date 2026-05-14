#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::intersection::Intersection;
use super::signal::SignalAction;
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

impl Situation for Intersection {}

#[derive(Debug, Clone, PartialEq)]
pub enum TrafficAction {
    AdvanceSignal { direction: usize },
    Tick,
    Malfunction { direction: usize },
    Recover { direction: usize },
}

impl Action for TrafficAction {
    type Sit = Intersection;
}

pub struct SafetyCheck;

impl Precondition<TrafficAction> for SafetyCheck {
    fn check(&self, intersection: &Intersection, action: &TrafficAction) -> Verdict {
        let meta = axiom_meta(
            "safety_check",
            "conflicting directions cannot both be green",
            "Manual on Uniform Traffic Control Devices (MUTCD 2009) Part 4D; Webster (1958) Traffic Signal Settings, Road Research Tech. Paper 39",
        );
        if let TrafficAction::AdvanceSignal { direction } = action {
            match intersection.advance_signal(*direction) {
                Ok(_) => Ok(Box::new(SimpleProof::new(meta))),
                Err(_) => Err(Box::new(SimpleCounterexample::new(meta))),
            }
        } else {
            Ok(Box::new(SimpleProof::new(meta)))
        }
    }
}

fn apply_traffic(
    intersection: &Intersection,
    action: &TrafficAction,
) -> Result<Intersection, Box<dyn Counterexample>> {
    let conflict_meta = axiom_meta(
        "safety_check",
        "conflicting directions cannot both be green",
        "Manual on Uniform Traffic Control Devices (MUTCD 2009) Part 4D",
    );
    let mut next = intersection.clone();
    match action {
        TrafficAction::AdvanceSignal { direction } => {
            return intersection.advance_signal(*direction).map_err(|_| {
                Box::new(SimpleCounterexample::new(conflict_meta)) as Box<dyn Counterexample>
            });
        }
        TrafficAction::Tick => return Ok(intersection.tick()),
        TrafficAction::Malfunction { direction } => {
            if *direction < next.signals.len()
                && let Ok(s) = next.signals[*direction].apply(SignalAction::Malfunction)
            {
                next.signals[*direction] = s;
            }
        }
        TrafficAction::Recover { direction } => {
            if *direction < next.signals.len()
                && let Ok(s) = next.signals[*direction].apply(SignalAction::Recover)
            {
                next.signals[*direction] = s;
            }
        }
    }
    Ok(next)
}

pub type TrafficEngine = Engine<TrafficAction>;

pub fn new_intersection(green: u32, yellow: u32, red: u32) -> TrafficEngine {
    Engine::new(
        Intersection::four_way(green, yellow, red),
        vec![Box::new(SafetyCheck)],
        apply_traffic,
    )
}

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::building::Building;
use super::request::Request;
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

impl Situation for Building {}

#[derive(Debug, Clone, PartialEq)]
pub enum ElevatorAction {
    Request(Request),
    Dispatch,
    Step,
    RunToCompletion { max_steps: usize },
}

impl Action for ElevatorAction {
    type Sit = Building;
}

pub struct ValidRequest;

impl Precondition<ElevatorAction> for ValidRequest {
    fn check(&self, building: &Building, action: &ElevatorAction) -> Verdict {
        let meta = axiom_meta(
            "valid_request",
            "requests must have valid floors and different origin/destination",
            "Strakosch & Caporale (2010) The Vertical Transportation Handbook §6; ASME A17.1 Safety Code for Elevators and Escalators",
        );
        if let ElevatorAction::Request(req) = action
            && (req.floor >= building.num_floors
                || req.destination >= building.num_floors
                || req.floor == req.destination)
        {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

fn apply_elevator(
    building: &Building,
    action: &ElevatorAction,
) -> Result<Building, Box<dyn Counterexample>> {
    let mut next = building.clone();
    match action {
        ElevatorAction::Request(req) => {
            let _ = next.request(*req);
        }
        ElevatorAction::Dispatch => {
            next.dispatch();
        }
        ElevatorAction::Step => {
            next.step();
        }
        ElevatorAction::RunToCompletion { max_steps } => {
            next.run_to_completion(*max_steps);
        }
    }
    Ok(next)
}

pub type ElevatorEngine = Engine<ElevatorAction>;

pub fn new_building(floors: usize, elevators: usize, capacity: u32) -> ElevatorEngine {
    Engine::new(
        Building::new(floors, elevators, capacity),
        vec![Box::new(ValidRequest)],
        apply_elevator,
    )
}

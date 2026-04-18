#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::building::Building;
use super::elevator::DoorState;
use super::request::Request;
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

impl Situation for Building {
    fn describe(&self) -> String {
        let floors: Vec<String> = self
            .elevators
            .iter()
            .map(|e| {
                format!(
                    "E{}@F{}{}",
                    e.id,
                    e.floor,
                    if e.door == DoorState::Open {
                        "(open)"
                    } else {
                        ""
                    }
                )
            })
            .collect();
        format!(
            "floors={} pending={} [{}]",
            self.num_floors,
            self.pending_requests.len(),
            floors.join(", ")
        )
    }

    fn is_terminal(&self) -> bool {
        self.pending_requests.is_empty()
            && self
                .elevators
                .iter()
                .all(|e| e.is_idle() && e.door == DoorState::Closed)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElevatorAction {
    Request(Request),
    Dispatch,
    Step,
    RunToCompletion { max_steps: usize },
}

impl Action for ElevatorAction {
    type Sit = Building;

    fn describe(&self) -> String {
        match self {
            ElevatorAction::Request(r) => format!("request floor {} → {}", r.floor, r.destination),
            ElevatorAction::Dispatch => "dispatch".into(),
            ElevatorAction::Step => "step".into(),
            ElevatorAction::RunToCompletion { max_steps } => format!("run (max {})", max_steps),
        }
    }
}

pub struct ValidRequest;

impl Precondition<ElevatorAction> for ValidRequest {
    fn check(&self, building: &Building, action: &ElevatorAction) -> PreconditionResult {
        if let ElevatorAction::Request(req) = action {
            if req.floor >= building.num_floors {
                return PreconditionResult::violated(
                    "valid_request",
                    "floor out of range",
                    &building.describe(),
                    &action.describe(),
                );
            }
            if req.destination >= building.num_floors {
                return PreconditionResult::violated(
                    "valid_request",
                    "destination out of range",
                    &building.describe(),
                    &action.describe(),
                );
            }
            if req.floor == req.destination {
                return PreconditionResult::violated(
                    "valid_request",
                    "same floor",
                    &building.describe(),
                    &action.describe(),
                );
            }
        }
        PreconditionResult::satisfied("valid_request", "request is valid")
    }

    fn describe(&self) -> &str {
        "requests must have valid floors and different origin/destination"
    }
}

fn apply_elevator(building: &Building, action: &ElevatorAction) -> Result<Building, String> {
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

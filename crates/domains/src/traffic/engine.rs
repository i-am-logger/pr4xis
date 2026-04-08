use super::intersection::Intersection;
use super::signal::SignalAction;
use praxis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

impl Situation for Intersection {
    fn describe(&self) -> String {
        let states: Vec<String> = self
            .signals
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{}:{:?}({}t)", i, s.state, s.ticks_in_state))
            .collect();
        format!("[{}]", states.join(", "))
    }

    fn is_terminal(&self) -> bool {
        false // traffic never stops
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrafficAction {
    AdvanceSignal { direction: usize },
    Tick,
    Malfunction { direction: usize },
    Recover { direction: usize },
}

impl Action for TrafficAction {
    type Sit = Intersection;

    fn describe(&self) -> String {
        match self {
            TrafficAction::AdvanceSignal { direction } => format!("advance signal {}", direction),
            TrafficAction::Tick => "tick".into(),
            TrafficAction::Malfunction { direction } => format!("malfunction signal {}", direction),
            TrafficAction::Recover { direction } => format!("recover signal {}", direction),
        }
    }
}

pub struct SafetyCheck;

impl Precondition<TrafficAction> for SafetyCheck {
    fn check(&self, intersection: &Intersection, action: &TrafficAction) -> PreconditionResult {
        if let TrafficAction::AdvanceSignal { direction } = action {
            // Check if advancing would create a conflict
            match intersection.advance_signal(*direction) {
                Ok(_) => PreconditionResult::satisfied("safety_check", "no conflict"),
                Err(msg) => PreconditionResult::violated(
                    "safety_check",
                    msg,
                    &intersection.describe(),
                    &action.describe(),
                ),
            }
        } else {
            PreconditionResult::satisfied("safety_check", "no conflict risk")
        }
    }

    fn describe(&self) -> &str {
        "conflicting directions cannot both be green"
    }
}

fn apply_traffic(intersection: &Intersection, action: &TrafficAction) -> Intersection {
    let mut next = intersection.clone();
    match action {
        TrafficAction::AdvanceSignal { direction } => {
            return intersection
                .advance_signal(*direction)
                .expect("precondition should have caught invalid advance");
        }
        TrafficAction::Tick => return intersection.tick(),
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
    next
}

pub type TrafficEngine = Engine<TrafficAction>;

pub fn new_intersection(green: u32, yellow: u32, red: u32) -> TrafficEngine {
    Engine::new(
        Intersection::four_way(green, yellow, red),
        vec![Box::new(SafetyCheck)],
        apply_traffic,
    )
}

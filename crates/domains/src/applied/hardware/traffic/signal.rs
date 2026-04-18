#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// State of a single traffic signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalState {
    Red,
    Yellow,
    Green,
    LeftArrow,      // protected left turn
    BlinkingYellow, // malfunction/caution
    BlinkingRed,    // treat as stop sign
    Off,
}

/// Actions that can be applied to a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalAction {
    /// Normal cycle advance
    Advance,
    /// Enter malfunction mode
    Malfunction,
    /// Recovery from malfunction (always goes to Red)
    Recover,
    /// Turn off completely
    TurnOff,
    /// Turn on (starts at Red)
    TurnOn,
}

/// A single traffic signal with timing enforcement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    pub state: SignalState,
    pub ticks_in_state: u32,
    pub green_duration: u32,
    pub yellow_duration: u32,
    pub red_duration: u32,
    pub left_arrow_duration: u32,
    pub min_red_duration: u32, // minimum red before going green
}

impl Signal {
    pub fn new(green: u32, yellow: u32, red: u32) -> Self {
        Self {
            state: SignalState::Red,
            ticks_in_state: 0,
            green_duration: green,
            yellow_duration: yellow,
            red_duration: red,
            left_arrow_duration: 0,
            min_red_duration: yellow, // at least yellow duration
        }
    }

    pub fn with_left_arrow(mut self, duration: u32) -> Self {
        self.left_arrow_duration = duration;
        self
    }

    /// Valid transitions from current state.
    fn valid_transitions(&self) -> Vec<SignalState> {
        match self.state {
            SignalState::Red => {
                let mut next = vec![];
                if self.ticks_in_state >= self.min_red_duration {
                    if self.left_arrow_duration > 0 {
                        next.push(SignalState::LeftArrow);
                    }
                    next.push(SignalState::Green);
                }
                next
            }
            SignalState::Green => {
                if self.ticks_in_state >= self.green_duration {
                    vec![SignalState::Yellow]
                } else {
                    vec![]
                }
            }
            SignalState::Yellow => {
                if self.ticks_in_state >= self.yellow_duration {
                    vec![SignalState::Red]
                } else {
                    vec![]
                }
            }
            SignalState::LeftArrow => {
                if self.ticks_in_state >= self.left_arrow_duration {
                    vec![SignalState::Green]
                } else {
                    vec![]
                }
            }
            SignalState::BlinkingYellow | SignalState::BlinkingRed => {
                vec![SignalState::Red] // recovery always goes to red
            }
            SignalState::Off => vec![SignalState::Red], // turn on → red
        }
    }

    /// Can the signal advance right now?
    pub fn can_advance(&self) -> bool {
        !self.valid_transitions().is_empty()
    }

    /// Apply an action. Returns Err if the action violates rules.
    pub fn apply(&self, action: SignalAction) -> Result<Signal, &'static str> {
        let mut next = self.clone();

        match action {
            SignalAction::Advance => {
                let transitions = self.valid_transitions();
                if transitions.is_empty() {
                    return Err("cannot advance: minimum time not reached");
                }
                next.state = transitions[0];
                next.ticks_in_state = 0;
            }
            SignalAction::Malfunction => {
                // Any state can enter malfunction
                next.state = SignalState::BlinkingYellow;
                next.ticks_in_state = 0;
            }
            SignalAction::Recover => match self.state {
                SignalState::BlinkingYellow | SignalState::BlinkingRed => {
                    next.state = SignalState::Red;
                    next.ticks_in_state = 0;
                }
                _ => return Err("can only recover from malfunction state"),
            },
            SignalAction::TurnOff => {
                next.state = SignalState::Off;
                next.ticks_in_state = 0;
            }
            SignalAction::TurnOn => {
                if self.state != SignalState::Off {
                    return Err("signal is already on");
                }
                next.state = SignalState::Red;
                next.ticks_in_state = 0;
            }
        }

        Ok(next)
    }

    /// Advance one tick (time passes, no state change).
    pub fn tick(&self) -> Signal {
        let mut next = self.clone();
        next.ticks_in_state += 1;
        next
    }
}

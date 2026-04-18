#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::signal::{Signal, SignalState};

/// Result of intersection validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntersectionResult {
    Safe,
    Conflict {
        direction_a: usize,
        direction_b: usize,
    },
}

/// An intersection with multiple signals (one per direction).
/// The ontology enforces: conflicting directions can't both be green.
#[derive(Debug, Clone, PartialEq)]
pub struct Intersection {
    pub signals: Vec<Signal>,
    /// Conflict matrix: conflicts[i] = list of directions that conflict with i.
    /// If signal i is green/leftarrow, all conflicting signals must be red.
    pub conflicts: Vec<Vec<usize>>,
}

impl Intersection {
    /// Create a standard 4-way intersection (NS and EW conflict).
    pub fn four_way(green: u32, yellow: u32, red: u32) -> Self {
        let signals = vec![
            Signal::new(green, yellow, red), // 0: North
            Signal::new(green, yellow, red), // 1: South
            Signal::new(green, yellow, red), // 2: East
            Signal::new(green, yellow, red), // 3: West
        ];
        // NS conflicts with EW
        let conflicts = vec![
            vec![2, 3], // North conflicts with East, West
            vec![2, 3], // South conflicts with East, West
            vec![0, 1], // East conflicts with North, South
            vec![0, 1], // West conflicts with North, South
        ];
        Self { signals, conflicts }
    }

    /// Check if the intersection is safe (no conflicting greens).
    pub fn validate(&self) -> IntersectionResult {
        for (i, signal) in self.signals.iter().enumerate() {
            if !is_go_state(signal.state) {
                continue;
            }
            for &j in &self.conflicts[i] {
                if is_go_state(self.signals[j].state) {
                    return IntersectionResult::Conflict {
                        direction_a: i,
                        direction_b: j,
                    };
                }
            }
        }
        IntersectionResult::Safe
    }

    /// Is the intersection currently safe?
    pub fn is_safe(&self) -> bool {
        self.validate() == IntersectionResult::Safe
    }

    /// Advance a specific signal. Returns Err if it would create a conflict.
    pub fn advance_signal(&self, direction: usize) -> Result<Intersection, &'static str> {
        if direction >= self.signals.len() {
            return Err("direction out of range");
        }

        let mut next = self.clone();
        let new_signal = self.signals[direction].apply(super::signal::SignalAction::Advance)?;

        // Check: if the new state is a go state, all conflicting must be non-go
        if is_go_state(new_signal.state) {
            for &j in &self.conflicts[direction] {
                if is_go_state(self.signals[j].state) {
                    return Err("would create conflicting green signals");
                }
            }
        }

        next.signals[direction] = new_signal;
        Ok(next)
    }

    /// Tick all signals (advance time by one unit).
    pub fn tick(&self) -> Intersection {
        let mut next = self.clone();
        for signal in &mut next.signals {
            *signal = signal.tick();
        }
        next
    }
}

fn is_go_state(state: SignalState) -> bool {
    matches!(state, SignalState::Green | SignalState::LeftArrow)
}

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::elevator::Direction;

/// A hall call: someone at a floor wants to go in a direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Request {
    pub floor: usize,
    pub direction: Direction,
    pub passenger_weight: u32,
    pub destination: usize,
}

impl Request {
    pub fn new(floor: usize, destination: usize, weight: u32) -> Self {
        let direction = if destination > floor {
            Direction::Up
        } else if destination < floor {
            Direction::Down
        } else {
            Direction::Idle
        };
        Self {
            floor,
            direction,
            passenger_weight: weight,
            destination,
        }
    }
}

use super::elevator::{Direction, Elevator};
use super::request::Request;

/// Dispatch strategy for assigning requests to elevators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchStrategy {
    /// Assign to the nearest idle or same-direction elevator.
    NearestCar,
    /// Zone-based: each elevator serves a range of floors.
    Zone,
}

pub struct Dispatch;

impl Dispatch {
    /// Assign requests to elevators based on the strategy.
    pub fn assign(
        strategy: &DispatchStrategy,
        elevators: &[Elevator],
        requests: &[Request],
    ) -> Vec<(usize, Request)> {
        match strategy {
            DispatchStrategy::NearestCar => Self::nearest_car(elevators, requests),
            DispatchStrategy::Zone => Self::zone_based(elevators, requests),
        }
    }

    /// Nearest car: assign each request to the best-scoring elevator.
    /// Score considers: distance, direction alignment, load.
    fn nearest_car(elevators: &[Elevator], requests: &[Request]) -> Vec<(usize, Request)> {
        let mut assignments = Vec::new();

        for &req in requests {
            let best = elevators
                .iter()
                .enumerate()
                .filter(|(_, e)| e.can_accept(req.passenger_weight))
                .min_by_key(|(_, e)| Self::score(e, &req));

            if let Some((id, _)) = best {
                assignments.push((id, req));
            }
        }

        assignments
    }

    /// Zone-based: divide floors evenly among elevators.
    fn zone_based(elevators: &[Elevator], requests: &[Request]) -> Vec<(usize, Request)> {
        if elevators.is_empty() {
            return Vec::new();
        }

        let num_elevators = elevators.len();
        let mut assignments = Vec::new();

        for &req in requests {
            // Simple zone: floor / (total_floors / num_elevators)
            // Find which elevator's zone this floor falls in
            let zone_id = req.floor * num_elevators / (req.floor.max(1) + num_elevators);
            let zone_id = zone_id.min(num_elevators - 1);

            // If the assigned elevator can't accept, find nearest that can
            if elevators[zone_id].can_accept(req.passenger_weight) {
                assignments.push((zone_id, req));
            } else {
                // Fallback to nearest car
                let fallback = elevators
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.can_accept(req.passenger_weight))
                    .min_by_key(|(_, e)| e.distance_to(req.floor));

                if let Some((id, _)) = fallback {
                    assignments.push((id, req));
                }
            }
        }

        assignments
    }

    /// Score an elevator for a request. Lower = better.
    fn score(elevator: &Elevator, request: &Request) -> usize {
        let distance = elevator.distance_to(request.floor);

        // Bonus for same direction (divide distance by 2)
        let direction_bonus = match elevator.direction {
            Direction::Up
                if request.direction == Direction::Up && request.floor >= elevator.floor =>
            {
                true
            }
            Direction::Down
                if request.direction == Direction::Down && request.floor <= elevator.floor =>
            {
                true
            }
            Direction::Idle => true,
            _ => false,
        };

        if direction_bonus {
            distance
        } else {
            distance * 3 // Penalize wrong direction
        }
    }
}

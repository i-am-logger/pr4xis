use super::dispatch::{Dispatch, DispatchStrategy};
use super::elevator::{DoorState, Elevator};
use super::request::Request;

/// A building with N floors and M elevators.
/// The ontology: only valid operations can be performed.
#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    pub num_floors: usize,
    pub elevators: Vec<Elevator>,
    pub pending_requests: Vec<Request>,
    pub strategy: DispatchStrategy,
}

impl Building {
    pub fn new(num_floors: usize, num_elevators: usize, capacity: u32) -> Self {
        let elevators = (0..num_elevators)
            .map(|id| Elevator::new(id, capacity, num_floors))
            .collect();
        Self {
            num_floors,
            elevators,
            pending_requests: Vec::new(),
            strategy: DispatchStrategy::NearestCar,
        }
    }

    /// Submit a request. Returns Err if the request is invalid.
    pub fn request(&mut self, req: Request) -> Result<(), &'static str> {
        if req.floor >= self.num_floors {
            return Err("floor out of range");
        }
        if req.destination >= self.num_floors {
            return Err("destination out of range");
        }
        if req.floor == req.destination {
            return Err("origin and destination are the same floor");
        }
        self.pending_requests.push(req);
        Ok(())
    }

    /// Dispatch pending requests to elevators using the current strategy.
    /// Returns the assignments: (elevator_id, request).
    pub fn dispatch(&mut self) -> Vec<(usize, Request)> {
        let assignments = Dispatch::assign(&self.strategy, &self.elevators, &self.pending_requests);

        // Apply assignments: add pickup and destination stops
        for &(eid, ref req) in &assignments {
            let elevator = &mut self.elevators[eid];
            if elevator.floor != req.floor {
                elevator.add_stop(req.floor);
            }
            // Always add destination — direction will be updated after pickup
            elevator.stops.push(req.destination);
            elevator.stops.sort();
            elevator.stops.dedup();
            // Set initial direction if idle
            if elevator.direction == super::elevator::Direction::Idle {
                if req.floor > elevator.floor
                    || (req.floor == elevator.floor && req.destination > elevator.floor)
                {
                    elevator.direction = super::elevator::Direction::Up;
                } else {
                    elevator.direction = super::elevator::Direction::Down;
                }
            }
        }

        // Remove dispatched requests from pending
        let dispatched: Vec<Request> = assignments.iter().map(|(_, r)| *r).collect();
        self.pending_requests.retain(|r| !dispatched.contains(r));

        assignments
    }

    /// Simulate one time step: each elevator moves one floor (if needed).
    /// Returns events that occurred.
    pub fn step(&mut self) -> Vec<Event> {
        let mut events = Vec::new();

        for elevator in &mut self.elevators {
            // Always close open doors first, even if idle
            if elevator.door == DoorState::Open {
                let _ = elevator.close_doors();
                events.push(Event::DoorsClosed {
                    elevator: elevator.id,
                    floor: elevator.floor,
                });
                continue;
            }

            if elevator.is_idle() {
                continue;
            }

            // If at a stop, open doors
            if elevator.should_stop_here() && elevator.door == DoorState::Closed {
                let _ = elevator.open_doors();
                events.push(Event::DoorsOpened {
                    elevator: elevator.id,
                    floor: elevator.floor,
                });
                continue;
            }

            // Move one floor
            if elevator.move_one().is_ok() {
                events.push(Event::Moved {
                    elevator: elevator.id,
                    floor: elevator.floor,
                });
            }
        }

        events
    }

    /// Run the simulation until all elevators are idle and no pending requests.
    /// Returns all events. Caps at max_steps to prevent infinite loops.
    pub fn run_to_completion(&mut self, max_steps: usize) -> Vec<Event> {
        let mut all_events = Vec::new();

        for _ in 0..max_steps {
            // Dispatch any pending requests
            if !self.pending_requests.is_empty() {
                self.dispatch();
            }

            // Check if done: all idle AND all doors closed
            if self.pending_requests.is_empty()
                && self
                    .elevators
                    .iter()
                    .all(|e| e.is_idle() && e.door == DoorState::Closed)
            {
                break;
            }

            let events = self.step();
            all_events.extend(events);
        }

        all_events
    }
}

/// Events produced during simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    DoorsOpened { elevator: usize, floor: usize },
    DoorsClosed { elevator: usize, floor: usize },
    Moved { elevator: usize, floor: usize },
}

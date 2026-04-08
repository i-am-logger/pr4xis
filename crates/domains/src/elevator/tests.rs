use super::*;
use proptest::prelude::*;

// =============================================================================
// Proptest strategies
// =============================================================================

fn arb_request(num_floors: usize) -> impl Strategy<Value = Request> {
    (0..num_floors, 0..num_floors, 30..120u32)
        .prop_filter("origin != destination", |(o, d, _)| o != d)
        .prop_map(|(origin, dest, weight)| Request::new(origin, dest, weight))
}

fn arb_requests(num_floors: usize, max_requests: usize) -> impl Strategy<Value = Vec<Request>> {
    proptest::collection::vec(arb_request(num_floors), 1..=max_requests)
}

// =============================================================================
// Basic setup tests
// =============================================================================

#[test]
fn test_building_creation() {
    let building = Building::new(10, 3, 1000);
    assert_eq!(building.num_floors, 10);
    assert_eq!(building.elevators.len(), 3);
    assert!(building.pending_requests.is_empty());
}

#[test]
fn test_elevator_starts_at_ground() {
    let building = Building::new(10, 1, 1000);
    assert_eq!(building.elevators[0].floor, 0);
    assert_eq!(building.elevators[0].direction, Direction::Idle);
    assert_eq!(building.elevators[0].door, DoorState::Closed);
}

#[test]
fn test_invalid_request_rejected() {
    let mut building = Building::new(10, 1, 1000);
    assert!(building.request(Request::new(0, 0, 80)).is_err()); // same floor
    assert!(building.request(Request::new(15, 0, 80)).is_err()); // floor out of range
    assert!(building.request(Request::new(0, 15, 80)).is_err()); // destination out of range
}

#[test]
fn test_valid_request_accepted() {
    let mut building = Building::new(10, 1, 1000);
    assert!(building.request(Request::new(0, 5, 80)).is_ok());
    assert_eq!(building.pending_requests.len(), 1);
}

// =============================================================================
// Elevator enforcement tests
// =============================================================================

#[test]
fn test_cannot_move_with_doors_open() {
    let mut elevator = Elevator::new(0, 1000, 10);
    elevator.add_stop(5);
    elevator.direction = Direction::Up;
    elevator.door = DoorState::Open;
    assert!(elevator.move_one(10).is_err());
}

#[test]
fn test_cannot_board_with_doors_closed() {
    let mut elevator = Elevator::new(0, 1000, 10);
    assert!(elevator.board(80).is_err());
}

#[test]
fn test_cannot_exceed_capacity() {
    let mut elevator = Elevator::new(0, 200, 10);
    elevator.door = DoorState::Open;
    assert!(elevator.board(150).is_ok());
    assert!(elevator.board(100).is_err()); // 150 + 100 > 200
}

#[test]
fn test_direction_commitment_up() {
    let mut elevator = Elevator::new(0, 1000, 10);
    elevator.floor = 5;
    elevator.direction = Direction::Up;
    assert!(elevator.add_stop(8)); // above — ok
    assert!(!elevator.add_stop(2)); // below — rejected
}

#[test]
fn test_direction_commitment_down() {
    let mut elevator = Elevator::new(0, 1000, 10);
    elevator.floor = 5;
    elevator.direction = Direction::Down;
    assert!(elevator.add_stop(2)); // below — ok
    assert!(!elevator.add_stop(8)); // above — rejected
}

#[test]
fn test_idle_accepts_any_stop() {
    let mut elevator = Elevator::new(0, 1000, 10);
    elevator.floor = 5;
    elevator.direction = Direction::Idle;
    assert!(elevator.add_stop(2));
    assert!(elevator.add_stop(8));
}

// =============================================================================
// Dispatch tests
// =============================================================================

#[test]
fn test_nearest_car_dispatch() {
    let mut building = Building::new(10, 2, 1000);
    building.elevators[0].floor = 0;
    building.elevators[1].floor = 8;
    building.request(Request::new(7, 0, 80)).unwrap();
    let assignments = building.dispatch();
    // Elevator 1 (at floor 8) should get the request for floor 7
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].0, 1);
}

#[test]
fn test_dispatch_respects_capacity() {
    let mut building = Building::new(10, 2, 100);
    building.elevators[0].load = 90; // near capacity
    building.elevators[1].floor = 9; // far away but has capacity
    building.request(Request::new(0, 5, 80)).unwrap();
    let assignments = building.dispatch();
    // Elevator 0 can't accept (90+80 > 100), so elevator 1 gets it
    assert_eq!(assignments[0].0, 1);
}

// =============================================================================
// Simulation tests
// =============================================================================

#[test]
fn test_single_request_completes() {
    let mut building = Building::new(10, 1, 1000);
    building.request(Request::new(0, 5, 80)).unwrap();
    let events = building.run_to_completion(100);
    assert!(!events.is_empty());
    assert!(building.pending_requests.is_empty());
    assert!(building.elevators[0].is_idle());
}

#[test]
fn test_multiple_requests_complete() {
    let mut building = Building::new(10, 2, 1000);
    building.request(Request::new(0, 5, 80)).unwrap();
    building.request(Request::new(3, 9, 80)).unwrap();
    building.request(Request::new(7, 1, 80)).unwrap();
    let _events = building.run_to_completion(200);
    assert!(building.pending_requests.is_empty());
}

// =============================================================================
// Property-based tests — ontology enforcement
// =============================================================================

proptest! {
    /// Invalid requests (same floor) are always rejected
    #[test]
    fn prop_same_floor_rejected(floor in 0..10usize) {
        let mut building = Building::new(10, 1, 1000);
        prop_assert!(building.request(Request::new(floor, floor, 80)).is_err());
    }

    /// Out-of-range floors are always rejected
    #[test]
    fn prop_out_of_range_rejected(floor in 10..100usize) {
        let mut building = Building::new(10, 1, 1000);
        prop_assert!(building.request(Request::new(floor, 0, 80)).is_err());
        prop_assert!(building.request(Request::new(0, floor, 80)).is_err());
    }

    /// Valid requests are always accepted
    #[test]
    fn prop_valid_request_accepted(req in arb_request(10)) {
        let mut building = Building::new(10, 1, 1000);
        prop_assert!(building.request(req).is_ok());
    }

    /// Elevator never exceeds capacity
    #[test]
    fn prop_never_exceeds_capacity(loads in proptest::collection::vec(30..100u32, 1..5)) {
        let mut elevator = Elevator::new(0, 200, 10);
        elevator.door = DoorState::Open;
        for weight in &loads {
            let _ = elevator.board(*weight); // may fail, that's fine
        }
        prop_assert!(elevator.load <= elevator.capacity);
    }

    /// Elevator floor is always within building bounds during simulation
    #[test]
    fn prop_floor_in_bounds(requests in arb_requests(10, 5)) {
        let mut building = Building::new(10, 2, 1000);
        for req in requests {
            let _ = building.request(req);
        }
        let _events = building.run_to_completion(200);
        for elevator in &building.elevators {
            prop_assert!(elevator.floor < building.num_floors,
                "elevator {} at floor {} but building has {} floors",
                elevator.id, elevator.floor, building.num_floors);
        }
    }

    /// Doors are always closed when elevator is between floors (after completion)
    #[test]
    fn prop_doors_closed_when_idle(requests in arb_requests(10, 3)) {
        let mut building = Building::new(10, 2, 1000);
        for req in requests {
            let _ = building.request(req);
        }
        let _events = building.run_to_completion(200);
        for elevator in &building.elevators {
            if elevator.is_idle() {
                prop_assert_eq!(elevator.door, DoorState::Closed,
                    "elevator {} is idle but doors are open", elevator.id);
            }
        }
    }

    /// All requests are eventually served (no starvation)
    #[test]
    fn prop_no_starvation(requests in arb_requests(10, 5)) {
        let mut building = Building::new(10, 2, 1000);
        for req in requests {
            let _ = building.request(req);
        }
        let _events = building.run_to_completion(500);
        prop_assert!(building.pending_requests.is_empty(),
            "{} requests still pending", building.pending_requests.len());
    }

    /// Direction commitment: elevator going up doesn't accept stops below
    #[test]
    fn prop_direction_commitment_up(current in 1..9usize, below in 0..1usize) {
        let mut elevator = Elevator::new(0, 1000, 10);
        elevator.floor = current;
        elevator.direction = Direction::Up;
        prop_assert!(!elevator.add_stop(below),
            "elevator at {} going up should reject stop at {}", current, below);
    }

    /// Direction commitment: elevator going down doesn't accept stops above
    #[test]
    fn prop_direction_commitment_down(current in 0..8usize, above in 9..10usize) {
        let mut elevator = Elevator::new(0, 1000, 10);
        elevator.floor = current;
        elevator.direction = Direction::Down;
        prop_assert!(!elevator.add_stop(above),
            "elevator at {} going down should reject stop at {}", current, above);
    }

    /// Dispatch assigns every request to some elevator (if capacity allows)
    #[test]
    fn prop_dispatch_assigns_all(requests in arb_requests(10, 5)) {
        let mut building = Building::new(10, 3, 2000); // big capacity
        for req in &requests {
            let _ = building.request(*req);
        }
        let pending_count = building.pending_requests.len();
        let assignments = building.dispatch();
        prop_assert_eq!(assignments.len(), pending_count,
            "not all requests were dispatched");
    }

    /// Dispatch never assigns to an over-capacity elevator
    #[test]
    fn prop_dispatch_respects_capacity(requests in arb_requests(10, 3)) {
        let mut building = Building::new(10, 2, 200);
        for req in &requests {
            let _ = building.request(*req);
        }
        let assignments = building.dispatch();
        for (eid, req) in &assignments {
            prop_assert!(building.elevators[*eid].can_accept(req.passenger_weight),
                "elevator {} can't accept weight {}", eid, req.passenger_weight);
        }
    }

    /// Nearest car dispatch picks the closest elevator
    #[test]
    fn prop_nearest_is_actually_nearest(floor in 0..10usize, dest in 0..10usize) {
        prop_assume!(floor != dest);
        let mut building = Building::new(10, 3, 1000);
        // Place elevators at different floors
        building.elevators[0].floor = 0;
        building.elevators[1].floor = 5;
        building.elevators[2].floor = 9;
        building.request(Request::new(floor, dest, 80)).unwrap();
        let assignments = building.dispatch();
        if let Some(&(eid, _)) = assignments.first() {
            let assigned_dist = building.elevators[eid].distance_to(floor);
            // The assigned elevator should be within 3x of the actual nearest
            // (because direction penalty is 3x)
            let min_dist = building.elevators.iter()
                .map(|e| e.distance_to(floor))
                .min().unwrap();
            prop_assert!(assigned_dist <= min_dist * 3 + 1,
                "assigned elevator {} (dist={}) but nearest is dist={}",
                eid, assigned_dist, min_dist);
        }
    }
}

// =============================================================================
// Engine tests — Situation/Action/Precondition/Trace
// =============================================================================

use praxis::engine::*;

#[test]
fn engine_request_dispatch_run() {
    let e = new_building(10, 2, 4);
    let e = e
        .try_next(ElevatorAction::Request(Request::new(0, 5, 70)))
        .unwrap();
    let e = e.try_next(ElevatorAction::Dispatch).unwrap();
    let e = e
        .try_next(ElevatorAction::RunToCompletion { max_steps: 200 })
        .unwrap();
    // After dispatch + run, the elevator should have serviced the request
    assert_eq!(e.step(), 3);
    assert!(e.trace().entries.iter().all(|entry| entry.success));
}

#[test]
fn engine_invalid_request_rejected() {
    let e = new_building(5, 1, 2);
    // Same floor request
    let result = e.try_next(ElevatorAction::Request(Request::new(3, 3, 70)));
    assert!(result.is_err());
}

#[test]
fn engine_out_of_range_rejected() {
    let e = new_building(5, 1, 2);
    let result = e.try_next(ElevatorAction::Request(Request::new(0, 99, 70)));
    assert!(result.is_err());
}

#[test]
fn engine_back_forward() {
    let e = new_building(10, 1, 4);
    let e = e
        .try_next(ElevatorAction::Request(Request::new(0, 3, 70)))
        .unwrap();
    let e = e.try_next(ElevatorAction::Dispatch).unwrap();
    assert_eq!(e.step(), 2);
    let e = e.back().unwrap();
    assert_eq!(e.step(), 1);
    let e = e.forward().unwrap();
    assert_eq!(e.step(), 2);
}

#[test]
fn engine_trace_records_all() {
    let e = new_building(5, 1, 2);
    let e = e
        .try_next(ElevatorAction::Request(Request::new(0, 4, 70)))
        .unwrap();
    let e = e.try_next(ElevatorAction::Dispatch).unwrap();
    let e = e.try_next(ElevatorAction::Step).unwrap();
    assert_eq!(e.trace().entries.len(), 3);
}

use crate::*;
use proptest::prelude::*;

fn arb_signal_action() -> impl Strategy<Value = SignalAction> {
    prop_oneof![
        Just(SignalAction::Advance),
        Just(SignalAction::Malfunction),
        Just(SignalAction::Recover),
        Just(SignalAction::TurnOff),
        Just(SignalAction::TurnOn),
    ]
}

fn arb_direction() -> impl Strategy<Value = usize> {
    0..4usize
}

// =============================================================================
// Signal enforcement tests
// =============================================================================

#[test]
fn test_signal_starts_red() {
    let signal = Signal::new(30, 5, 30);
    assert_eq!(signal.state, SignalState::Red);
}

#[test]
fn test_cant_advance_before_min_time() {
    let signal = Signal::new(30, 5, 30);
    assert!(signal.apply(SignalAction::Advance).is_err());
}

#[test]
fn test_advance_after_min_red() {
    let mut signal = Signal::new(30, 5, 30);
    for _ in 0..5 {
        signal = signal.tick();
    } // tick past min_red (=yellow=5)
    let next = signal.apply(SignalAction::Advance).unwrap();
    assert_eq!(next.state, SignalState::Green);
}

#[test]
fn test_green_to_yellow() {
    let mut signal = Signal::new(3, 2, 5);
    for _ in 0..5 {
        signal = signal.tick();
    } // past min_red
    signal = signal.apply(SignalAction::Advance).unwrap(); // → Green
    for _ in 0..3 {
        signal = signal.tick();
    } // past green_duration
    let next = signal.apply(SignalAction::Advance).unwrap();
    assert_eq!(next.state, SignalState::Yellow);
}

#[test]
fn test_yellow_to_red() {
    let mut signal = Signal::new(3, 2, 5);
    for _ in 0..5 {
        signal = signal.tick();
    }
    signal = signal.apply(SignalAction::Advance).unwrap(); // → Green
    for _ in 0..3 {
        signal = signal.tick();
    }
    signal = signal.apply(SignalAction::Advance).unwrap(); // → Yellow
    for _ in 0..2 {
        signal = signal.tick();
    }
    let next = signal.apply(SignalAction::Advance).unwrap();
    assert_eq!(next.state, SignalState::Red);
}

#[test]
fn test_full_cycle() {
    let mut signal = Signal::new(3, 2, 5);
    // Red (wait 5) → Green (wait 3) → Yellow (wait 2) → Red
    for _ in 0..5 {
        signal = signal.tick();
    }
    signal = signal.apply(SignalAction::Advance).unwrap();
    assert_eq!(signal.state, SignalState::Green);
    for _ in 0..3 {
        signal = signal.tick();
    }
    signal = signal.apply(SignalAction::Advance).unwrap();
    assert_eq!(signal.state, SignalState::Yellow);
    for _ in 0..2 {
        signal = signal.tick();
    }
    signal = signal.apply(SignalAction::Advance).unwrap();
    assert_eq!(signal.state, SignalState::Red);
}

#[test]
fn test_malfunction_from_any_state() {
    let signal = Signal::new(30, 5, 30);
    let mal = signal.apply(SignalAction::Malfunction).unwrap();
    assert_eq!(mal.state, SignalState::BlinkingYellow);
}

#[test]
fn test_recover_goes_to_red() {
    let signal = Signal::new(30, 5, 30);
    let mal = signal.apply(SignalAction::Malfunction).unwrap();
    let recovered = mal.apply(SignalAction::Recover).unwrap();
    assert_eq!(recovered.state, SignalState::Red);
}

#[test]
fn test_cant_recover_from_normal() {
    let signal = Signal::new(30, 5, 30);
    assert!(signal.apply(SignalAction::Recover).is_err());
}

#[test]
fn test_left_arrow_phase() {
    let mut signal = Signal::new(30, 5, 30).with_left_arrow(10);
    for _ in 0..5 {
        signal = signal.tick();
    }
    let next = signal.apply(SignalAction::Advance).unwrap();
    assert_eq!(next.state, SignalState::LeftArrow);
}

#[test]
fn test_off_and_on() {
    let signal = Signal::new(30, 5, 30);
    let off = signal.apply(SignalAction::TurnOff).unwrap();
    assert_eq!(off.state, SignalState::Off);
    let on = off.apply(SignalAction::TurnOn).unwrap();
    assert_eq!(on.state, SignalState::Red);
}

#[test]
fn test_cant_turn_on_when_already_on() {
    let signal = Signal::new(30, 5, 30);
    assert!(signal.apply(SignalAction::TurnOn).is_err());
}

// =============================================================================
// Intersection enforcement tests
// =============================================================================

#[test]
fn test_intersection_starts_safe() {
    let intersection = Intersection::four_way(30, 5, 30);
    assert!(intersection.is_safe());
}

#[test]
fn test_cant_create_conflicting_greens() {
    let mut intersection = Intersection::four_way(3, 2, 5);
    // Advance North to green
    for _ in 0..5 {
        intersection = intersection.tick();
    }
    intersection = intersection.advance_signal(0).unwrap(); // North → Green
    // Try to advance East to green — should fail (conflicts with North)
    for _ in 0..5 {
        intersection = intersection.tick();
    }
    let result = intersection.advance_signal(2); // East
    assert!(result.is_err());
}

#[test]
fn test_non_conflicting_can_both_be_green() {
    let mut intersection = Intersection::four_way(3, 2, 5);
    for _ in 0..5 {
        intersection = intersection.tick();
    }
    intersection = intersection.advance_signal(0).unwrap(); // North → Green
    // South doesn't conflict with North — should succeed
    let result = intersection.advance_signal(1);
    assert!(result.is_ok());
}

// =============================================================================
// Property-based tests
// =============================================================================

proptest! {
    /// Signal always starts at Red
    #[test]
    fn prop_starts_red(green in 1..100u32, yellow in 1..100u32, red in 1..100u32) {
        let signal = Signal::new(green, yellow, red);
        prop_assert_eq!(signal.state, SignalState::Red);
    }

    /// Can't advance without enough ticks
    #[test]
    fn prop_cant_advance_early(green in 1..100u32, yellow in 1..100u32, red in 1..100u32) {
        let signal = Signal::new(green, yellow, red);
        prop_assert!(signal.apply(SignalAction::Advance).is_err());
    }

    /// Malfunction is always allowed
    #[test]
    fn prop_malfunction_always_allowed(green in 1..50u32, yellow in 1..10u32, red in 1..50u32, ticks in 0..100u32) {
        let mut signal = Signal::new(green, yellow, red);
        for _ in 0..ticks { signal = signal.tick(); }
        // Try random advances (may fail, that's fine)
        let _ = signal.apply(SignalAction::Advance);
        let result = signal.apply(SignalAction::Malfunction);
        prop_assert!(result.is_ok());
    }

    /// Recovery only from malfunction states
    #[test]
    fn prop_recover_only_from_malfunction(green in 1..50u32, yellow in 1..10u32, red in 1..50u32) {
        let signal = Signal::new(green, yellow, red);
        // Red is not a malfunction state
        prop_assert!(signal.apply(SignalAction::Recover).is_err());
        // BlinkingYellow is
        let mal = signal.apply(SignalAction::Malfunction).unwrap();
        prop_assert!(mal.apply(SignalAction::Recover).is_ok());
    }

    /// Recovery always goes to Red
    #[test]
    fn prop_recovery_is_red(green in 1..50u32, yellow in 1..10u32, red in 1..50u32) {
        let signal = Signal::new(green, yellow, red);
        let recovered = signal.apply(SignalAction::Malfunction).unwrap()
            .apply(SignalAction::Recover).unwrap();
        prop_assert_eq!(recovered.state, SignalState::Red);
    }

    /// Ticks never change state (only actions change state)
    #[test]
    fn prop_tick_preserves_state(green in 1..50u32, yellow in 1..10u32, red in 1..50u32, ticks in 1..200u32) {
        let signal = Signal::new(green, yellow, red);
        let mut ticked = signal.clone();
        for _ in 0..ticks { ticked = ticked.tick(); }
        prop_assert_eq!(signal.state, ticked.state);
    }

    /// Tick increments ticks_in_state
    #[test]
    fn prop_tick_increments(green in 1..50u32, yellow in 1..10u32, red in 1..50u32) {
        let signal = Signal::new(green, yellow, red);
        let ticked = signal.tick();
        prop_assert_eq!(ticked.ticks_in_state, signal.ticks_in_state + 1);
    }

    /// Green → Yellow is the only valid transition from Green
    #[test]
    fn prop_green_only_to_yellow(green in 1..10u32, yellow in 1..5u32, red in 1..10u32) {
        let mut signal = Signal::new(green, yellow, red);
        for _ in 0..red.max(yellow) { signal = signal.tick(); }
        let _ = signal.apply(SignalAction::Advance); // → Green (might fail)
        if signal.state != SignalState::Green {
            signal = Signal::new(green, yellow, red);
            for _ in 0..yellow { signal = signal.tick(); }
            if let Ok(s) = signal.apply(SignalAction::Advance) { signal = s; }
        }
        if signal.state == SignalState::Green {
            for _ in 0..green { signal = signal.tick(); }
            let next = signal.apply(SignalAction::Advance).unwrap();
            prop_assert_eq!(next.state, SignalState::Yellow);
        }
    }

    /// Yellow → Red is the only valid transition from Yellow
    #[test]
    fn prop_yellow_only_to_red(green in 1..10u32, yellow in 1..5u32, red in 1..10u32) {
        let mut signal = Signal::new(green, yellow, red);
        // Get to Yellow state
        for _ in 0..yellow { signal = signal.tick(); }
        if let Ok(s) = signal.apply(SignalAction::Advance) { signal = s; } // → Green
        if signal.state == SignalState::Green {
            for _ in 0..green { signal = signal.tick(); }
            if let Ok(s) = signal.apply(SignalAction::Advance) { signal = s; } // → Yellow
            if signal.state == SignalState::Yellow {
                for _ in 0..yellow { signal = signal.tick(); }
                let next = signal.apply(SignalAction::Advance).unwrap();
                prop_assert_eq!(next.state, SignalState::Red);
            }
        }
    }

    /// Intersection is always safe at start
    #[test]
    fn prop_intersection_starts_safe(green in 1..50u32, yellow in 1..10u32, red in 1..50u32) {
        let intersection = Intersection::four_way(green, yellow, red);
        prop_assert!(intersection.is_safe());
    }

    /// Ticking an intersection preserves safety
    #[test]
    fn prop_tick_preserves_safety(green in 1..50u32, yellow in 1..10u32, red in 1..50u32, ticks in 1..100u32) {
        let mut intersection = Intersection::four_way(green, yellow, red);
        for _ in 0..ticks {
            intersection = intersection.tick();
        }
        prop_assert!(intersection.is_safe());
    }

    /// advance_signal never creates conflicts
    #[test]
    fn prop_advance_never_conflicts(
        green in 1..10u32, yellow in 1..5u32, red in 1..10u32,
        dir in arb_direction(), ticks in 0..50u32
    ) {
        let mut intersection = Intersection::four_way(green, yellow, red);
        for _ in 0..ticks { intersection = intersection.tick(); }
        if let Ok(next) = intersection.advance_signal(dir) {
            prop_assert!(next.is_safe());
        }
        // If advance_signal returns Err, it prevented a conflict — also correct
    }

    /// Out of range direction is rejected
    #[test]
    fn prop_invalid_direction_rejected(dir in 4..100usize) {
        let intersection = Intersection::four_way(30, 5, 30);
        prop_assert!(intersection.advance_signal(dir).is_err());
    }

    /// Off → TurnOn → Red
    #[test]
    fn prop_turn_on_is_red(green in 1..50u32, yellow in 1..10u32, red in 1..50u32) {
        let signal = Signal::new(green, yellow, red);
        let off = signal.apply(SignalAction::TurnOff).unwrap();
        let on = off.apply(SignalAction::TurnOn).unwrap();
        prop_assert_eq!(on.state, SignalState::Red);
    }

    /// Advance resets ticks_in_state to 0
    #[test]
    fn prop_advance_resets_ticks(green in 1..10u32, yellow in 1..5u32, red in 1..10u32) {
        let mut signal = Signal::new(green, yellow, red);
        for _ in 0..yellow { signal = signal.tick(); }
        if let Ok(next) = signal.apply(SignalAction::Advance) {
            prop_assert_eq!(next.ticks_in_state, 0);
        }
    }
}

// =============================================================================
// Engine tests — Situation/Action/Precondition/Trace
// =============================================================================

use crate::engine::*;

#[test]
fn engine_tick_sequence() {
    let e = new_intersection(3, 1, 3);
    let e = e.try_next(TrafficAction::Tick).unwrap();
    let e = e.try_next(TrafficAction::Tick).unwrap();
    assert_eq!(e.step(), 2);
    assert!(!e.is_terminal());
}

#[test]
fn engine_back_forward() {
    let e = new_intersection(3, 1, 3);
    let e = e.try_next(TrafficAction::Tick).unwrap();
    let e = e.try_next(TrafficAction::Tick).unwrap();
    assert_eq!(e.step(), 2);
    let e = e.back().unwrap();
    assert_eq!(e.step(), 1);
    let e = e.forward().unwrap();
    assert_eq!(e.step(), 2);
}

#[test]
fn engine_advance_and_tick() {
    let e = new_intersection(3, 1, 3);
    // Tick through green duration
    let mut e = e;
    for _ in 0..3 {
        e = e.try_next(TrafficAction::Tick).unwrap();
    }
    // Advance signal 0
    let e = e
        .try_next(TrafficAction::AdvanceSignal { direction: 0 })
        .unwrap();
    assert!(e.step() > 0);
}

#[test]
fn engine_trace_records_actions() {
    let e = new_intersection(3, 1, 3);
    let e = e.try_next(TrafficAction::Tick).unwrap();
    let e = e.try_next(TrafficAction::Tick).unwrap();
    assert_eq!(e.trace().entries.len(), 2);
    assert!(e.trace().entries.iter().all(|entry| entry.success));
}

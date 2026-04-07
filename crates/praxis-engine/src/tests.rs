use crate::*;
use proptest::prelude::*;

// =============================================================================
// Example: Counter with bounds enforcement
//
// Situation: a number 0..=max
// Actions: Increment, Decrement
// Preconditions: can't go below 0, can't go above max
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
struct Counter {
    value: i32,
    max: i32,
}

impl Situation for Counter {
    fn describe(&self) -> String {
        format!("counter={} (max={})", self.value, self.max)
    }

    fn is_terminal(&self) -> bool {
        false // counter is never terminal
    }
}

#[derive(Debug, Clone, PartialEq)]
enum CounterAction {
    Increment { by: i32 },
    Decrement { by: i32 },
    Reset,
}

impl Action for CounterAction {
    type Sit = Counter;

    fn describe(&self) -> String {
        match self {
            CounterAction::Increment { by } => format!("increment by {}", by),
            CounterAction::Decrement { by } => format!("decrement by {}", by),
            CounterAction::Reset => "reset to 0".to_string(),
        }
    }
}

struct NotBelowZero;

impl Precondition<CounterAction> for NotBelowZero {
    fn check(&self, situation: &Counter, action: &CounterAction) -> PreconditionResult {
        if let CounterAction::Decrement { by } = action {
            if situation.value - by < 0 {
                return PreconditionResult::violated(
                    "not_below_zero",
                    &format!("{} - {} would be negative", situation.value, by),
                    &situation.describe(),
                    &action.describe(),
                );
            }
        }
        PreconditionResult::satisfied("not_below_zero", "value stays >= 0")
    }

    fn describe(&self) -> &str {
        "counter must not go below zero"
    }
}

struct NotAboveMax;

impl Precondition<CounterAction> for NotAboveMax {
    fn check(&self, situation: &Counter, action: &CounterAction) -> PreconditionResult {
        if let CounterAction::Increment { by } = action {
            if situation.value + by > situation.max {
                return PreconditionResult::violated(
                    "not_above_max",
                    &format!(
                        "{} + {} would exceed max {}",
                        situation.value, by, situation.max
                    ),
                    &situation.describe(),
                    &action.describe(),
                );
            }
        }
        PreconditionResult::satisfied("not_above_max", "value stays <= max")
    }

    fn describe(&self) -> &str {
        "counter must not exceed maximum"
    }
}

fn counter_apply(situation: &Counter, action: &CounterAction) -> Counter {
    let mut next = situation.clone();
    match action {
        CounterAction::Increment { by } => next.value += by,
        CounterAction::Decrement { by } => next.value -= by,
        CounterAction::Reset => next.value = 0,
    }
    next
}

fn make_engine(value: i32, max: i32) -> Engine<CounterAction> {
    Engine::new(
        Counter { value, max },
        vec![Box::new(NotBelowZero), Box::new(NotAboveMax)],
        counter_apply,
    )
}

// =============================================================================
// Basic tests
// =============================================================================

#[test]
fn test_increment() {
    let engine = make_engine(0, 10);
    let engine = engine.next(CounterAction::Increment { by: 5 }).unwrap();
    assert_eq!(engine.situation().value, 5);
}

#[test]
fn test_decrement() {
    let engine = make_engine(5, 10);
    let engine = engine.next(CounterAction::Decrement { by: 3 }).unwrap();
    assert_eq!(engine.situation().value, 2);
}

#[test]
fn test_reset() {
    let engine = make_engine(7, 10);
    let engine = engine.next(CounterAction::Reset).unwrap();
    assert_eq!(engine.situation().value, 0);
}

#[test]
fn test_chain() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 3 })
        .unwrap()
        .next(CounterAction::Increment { by: 4 })
        .unwrap()
        .next(CounterAction::Decrement { by: 2 })
        .unwrap();
    assert_eq!(engine.situation().value, 5);
}

#[test]
fn test_below_zero_blocked() {
    let engine = make_engine(2, 10);
    let result = engine.next(CounterAction::Decrement { by: 5 });
    assert!(result.is_err());
}

#[test]
fn test_above_max_blocked() {
    let engine = make_engine(8, 10);
    let result = engine.next(CounterAction::Increment { by: 5 });
    assert!(result.is_err());
}

#[test]
fn test_trace_records_success() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 3 })
        .unwrap()
        .next(CounterAction::Increment { by: 2 })
        .unwrap();
    assert_eq!(engine.trace().successful_steps(), 2);
    assert_eq!(engine.trace().violations(), 0);
}

#[test]
fn test_trace_records_violations() {
    let engine = make_engine(0, 10);
    let (engine, violations) = engine.next(CounterAction::Decrement { by: 5 }).unwrap_err();
    assert_eq!(violations.len(), 1);
    assert_eq!(engine.trace().violations(), 1);
}

#[test]
fn test_violation_carries_context() {
    let engine = make_engine(2, 10);
    let (_, violations) = engine.next(CounterAction::Decrement { by: 5 }).unwrap_err();
    if let PreconditionResult::Violated { rule, reason, .. } = &violations[0] {
        assert_eq!(rule, "not_below_zero");
        assert!(reason.contains("2 - 5"));
    } else {
        panic!("expected violation");
    }
}

#[test]
fn test_try_next() {
    let engine = make_engine(0, 10);
    let result = engine.try_next(CounterAction::Decrement { by: 5 });
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors[0].contains("not_below_zero"));
}

#[test]
fn test_trace_dump() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 3 })
        .unwrap();
    let dump = engine.trace().dump();
    assert!(dump.contains("OK"));
    assert!(dump.contains("increment by 3"));
}

#[test]
fn test_satisfied_carries_context() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 1 })
        .unwrap();
    let entry = engine.trace().last().unwrap();
    let satisfied: Vec<_> = entry
        .precondition_results
        .iter()
        .filter(|r| r.is_satisfied())
        .collect();
    assert_eq!(satisfied.len(), 2); // both rules satisfied
    assert_eq!(satisfied[0].rule(), "not_below_zero");
    assert_eq!(satisfied[1].rule(), "not_above_max");
}

// =============================================================================
// Back/Forward tests
// =============================================================================

#[test]
fn test_back_restores_previous() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 5 })
        .unwrap();
    assert_eq!(engine.situation().value, 5);
    let engine = engine.back().unwrap();
    assert_eq!(engine.situation().value, 0);
}

#[test]
fn test_forward_after_back() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 5 })
        .unwrap()
        .back()
        .unwrap();
    assert_eq!(engine.situation().value, 0);
    let engine = engine.forward().unwrap();
    assert_eq!(engine.situation().value, 5);
}

#[test]
fn test_back_forward_roundtrip() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 3 })
        .unwrap()
        .next(CounterAction::Increment { by: 4 })
        .unwrap()
        .next(CounterAction::Increment { by: 2 })
        .unwrap();
    assert_eq!(engine.situation().value, 9);
    let engine = engine.back().unwrap().back().unwrap();
    assert_eq!(engine.situation().value, 3);
    let engine = engine.forward().unwrap();
    assert_eq!(engine.situation().value, 7);
    let engine = engine.forward().unwrap();
    assert_eq!(engine.situation().value, 9);
}

#[test]
fn test_back_on_initial_fails() {
    let engine = make_engine(0, 10);
    assert!(engine.back().is_err());
}

#[test]
fn test_forward_without_back_fails() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 5 })
        .unwrap();
    assert!(engine.forward().is_err());
}

#[test]
fn test_next_after_back_clears_future() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 5 })
        .unwrap()
        .next(CounterAction::Increment { by: 3 })
        .unwrap();
    assert_eq!(engine.forward_depth(), 0);
    let engine = engine.back().unwrap();
    assert_eq!(engine.forward_depth(), 1);
    // New action clears future
    let engine = engine.next(CounterAction::Increment { by: 1 }).unwrap();
    assert_eq!(engine.forward_depth(), 0);
    assert_eq!(engine.situation().value, 6); // 5 + 1, not 5 + 3
}

#[test]
fn test_back_depth() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 1 })
        .unwrap()
        .next(CounterAction::Increment { by: 1 })
        .unwrap()
        .next(CounterAction::Increment { by: 1 })
        .unwrap();
    assert_eq!(engine.back_depth(), 3);
    assert_eq!(engine.forward_depth(), 0);
    let engine = engine.back().unwrap();
    assert_eq!(engine.back_depth(), 2);
    assert_eq!(engine.forward_depth(), 1);
}

#[test]
fn test_step_derived_from_history() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 1 })
        .unwrap()
        .next(CounterAction::Increment { by: 1 })
        .unwrap()
        .next(CounterAction::Increment { by: 1 })
        .unwrap();
    assert_eq!(engine.step(), 3);
    let engine = engine.back().unwrap();
    assert_eq!(engine.step(), 2);
    let engine = engine.back().unwrap();
    assert_eq!(engine.step(), 1);
    let engine = engine.forward().unwrap();
    assert_eq!(engine.step(), 2);
}

// =============================================================================
// Property-based tests
// =============================================================================

proptest! {
    /// Back then forward = original situation
    #[test]
    fn prop_back_forward_identity(n in 1..10usize) {
        let mut engine = make_engine(0, 1000);
        for i in 0..n {
            engine = engine.next(CounterAction::Increment { by: i as i32 + 1 }).unwrap();
        }
        let expected = engine.situation().value;
        let engine = engine.back().unwrap().forward().unwrap();
        prop_assert_eq!(engine.situation().value, expected);
    }

    /// Full undo restores initial state
    #[test]
    fn prop_full_undo(n in 1..10usize) {
        let mut engine = make_engine(0, 1000);
        for i in 0..n {
            engine = engine.next(CounterAction::Increment { by: i as i32 + 1 }).unwrap();
        }
        for _ in 0..n {
            engine = engine.back().unwrap();
        }
        prop_assert_eq!(engine.situation().value, 0);
        prop_assert_eq!(engine.back_depth(), 0);
    }

    /// Full undo then full redo = original final state
    #[test]
    fn prop_full_undo_redo(n in 1..10usize) {
        let mut engine = make_engine(0, 1000);
        for i in 0..n {
            engine = engine.next(CounterAction::Increment { by: i as i32 + 1 }).unwrap();
        }
        let expected = engine.situation().value;
        for _ in 0..n {
            engine = engine.back().unwrap();
        }
        for _ in 0..n {
            engine = engine.forward().unwrap();
        }
        prop_assert_eq!(engine.situation().value, expected);
    }

    /// back_depth + forward_depth = total steps after any back/forward sequence
    #[test]
    fn prop_depth_invariant(steps in 1..5usize, backs in 0..5usize) {
        let mut engine = make_engine(0, 1000);
        for i in 0..steps {
            engine = engine.next(CounterAction::Increment { by: i as i32 + 1 }).unwrap();
        }
        let actual_backs = backs.min(steps);
        for _ in 0..actual_backs {
            engine = engine.back().unwrap();
        }
        prop_assert_eq!(engine.back_depth() + engine.forward_depth(), steps);
    }

    /// step() always equals back_depth()
    #[test]
    fn prop_step_equals_back_depth(steps in 1..5usize, backs in 0..5usize) {
        let mut engine = make_engine(0, 1000);
        for i in 0..steps {
            engine = engine.next(CounterAction::Increment { by: i as i32 + 1 }).unwrap();
        }
        let actual_backs = backs.min(steps);
        for _ in 0..actual_backs {
            engine = engine.back().unwrap();
        }
        prop_assert_eq!(engine.step(), engine.back_depth());
    }

    /// Increment by positive amount increases value
    #[test]
    fn prop_increment_increases(start in 0..50i32, by in 1..10i32) {
        let engine = make_engine(start, 100);
        let engine = engine.next(CounterAction::Increment { by }).unwrap();
        prop_assert_eq!(engine.situation().value, start + by);
    }

    /// Decrement by positive amount decreases value
    #[test]
    fn prop_decrement_decreases(start in 10..50i32, by in 1..10i32) {
        let engine = make_engine(start, 100);
        let engine = engine.next(CounterAction::Decrement { by }).unwrap();
        prop_assert_eq!(engine.situation().value, start - by);
    }

    /// Reset always produces 0
    #[test]
    fn prop_reset_to_zero(start in 0..100i32) {
        let engine = make_engine(start, 100);
        let engine = engine.next(CounterAction::Reset).unwrap();
        prop_assert_eq!(engine.situation().value, 0);
    }

    /// Below zero is always blocked
    #[test]
    fn prop_below_zero_blocked(value in 0..10i32, by in 11..100i32) {
        let engine = make_engine(value, 100);
        let result = engine.next(CounterAction::Decrement { by });
        prop_assert!(result.is_err());
    }

    /// Above max is always blocked
    #[test]
    fn prop_above_max_blocked(value in 90..100i32, by in 11..100i32) {
        let engine = make_engine(value, 100);
        let result = engine.next(CounterAction::Increment { by });
        prop_assert!(result.is_err());
    }

    /// Successful action increases trace count
    #[test]
    fn prop_trace_grows_on_success(n in 1..10usize) {
        let mut engine = make_engine(0, 1000);
        for _ in 0..n {
            engine = engine.next(CounterAction::Increment { by: 1 }).unwrap();
        }
        prop_assert_eq!(engine.trace().successful_steps(), n);
    }

    /// Failed action increases violation count
    #[test]
    fn prop_trace_records_violations(value in 0..5i32) {
        let engine = make_engine(value, 10);
        let (engine, _) = engine.next(CounterAction::Decrement { by: value + 1 }).unwrap_err();
        prop_assert_eq!(engine.trace().violations(), 1);
    }

    /// Violation result always carries rule name
    #[test]
    fn prop_violation_has_rule(value in 0..5i32) {
        let engine = make_engine(value, 10);
        let (_, violations) = engine.next(CounterAction::Decrement { by: value + 1 }).unwrap_err();
        for v in &violations {
            prop_assert!(!v.rule().is_empty());
            prop_assert!(!v.reason().is_empty());
        }
    }

    /// Satisfied result always carries rule name
    #[test]
    fn prop_satisfied_has_rule(value in 0..50i32) {
        let engine = make_engine(value, 100)
            .next(CounterAction::Increment { by: 1 }).unwrap();
        let entry = engine.trace().last().unwrap();
        for r in &entry.precondition_results {
            prop_assert!(!r.rule().is_empty());
        }
    }

    /// Engine situation matches expected value after chain
    #[test]
    fn prop_chain_value(increments in proptest::collection::vec(1..5i32, 1..10)) {
        let mut engine = make_engine(0, 10000);
        let mut expected = 0;
        for by in &increments {
            engine = engine.next(CounterAction::Increment { by: *by }).unwrap();
            expected += by;
        }
        prop_assert_eq!(engine.situation().value, expected);
    }

    /// Trace step count matches action count
    #[test]
    fn prop_trace_step_count(n in 1..20usize) {
        let mut engine = make_engine(50, 1000);
        for _ in 0..n {
            engine = engine.next(CounterAction::Increment { by: 1 }).unwrap();
        }
        prop_assert_eq!(engine.trace().entries.len(), n);
    }

    /// try_next Ok matches next Ok
    #[test]
    fn prop_try_next_consistent(value in 0..50i32, by in 1..10i32) {
        let engine1 = make_engine(value, 100);
        let engine2 = make_engine(value, 100);
        let r1 = engine1.next(CounterAction::Increment { by: by }).map(|e| e.situation().value);
        let r2 = engine2.try_next(CounterAction::Increment { by: by }).map(|e| e.situation().value);
        prop_assert_eq!(r1.is_ok(), r2.is_ok());
        if let (Ok(v1), Ok(v2)) = (r1, r2) {
            prop_assert_eq!(v1, v2);
        }
    }

    /// Trace dump is non-empty after any action
    #[test]
    fn prop_dump_non_empty(value in 0..50i32) {
        let engine = make_engine(value, 100)
            .next(CounterAction::Increment { by: 1 }).unwrap();
        prop_assert!(!engine.trace().dump().is_empty());
    }
}

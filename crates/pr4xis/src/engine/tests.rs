use super::*;
use crate::logic::proof::{Counterexample, Proof, SimpleCounterexample, SimpleProof, Verdict};
use crate::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
use proptest::prelude::*;

// =============================================================================
// Example: Counter with bounds enforcement — typed API (#161)
//
// - Situation: Counter struct (Debug via derive, no describe() primitive-leak)
// - Action: CounterAction enum (Debug via derive, no describe() primitive-leak)
// - Preconditions: return typed Verdict (Proof / Counterexample), not String-Result
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
struct Counter {
    value: i32,
    max: i32,
}

impl Situation for Counter {}

#[derive(Debug, Clone, PartialEq)]
enum CounterAction {
    Increment { by: i32 },
    Decrement { by: i32 },
    Reset,
}

impl Action for CounterAction {
    type Sit = Counter;
}

/// Helper: typed meta for a precondition axiom.
fn axiom_meta(name: &'static str, description: &'static str, citation: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(citation),
        module_path: ModulePath::new_static(module_path!()),
    }
}

struct NotBelowZero;

impl Precondition<CounterAction> for NotBelowZero {
    fn check(&self, situation: &Counter, action: &CounterAction) -> Verdict {
        let meta = axiom_meta("NotBelowZero", "counter must not go below zero", "");
        if let CounterAction::Decrement { by } = action
            && situation.value - by < 0
        {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

struct NotAboveMax;

impl Precondition<CounterAction> for NotAboveMax {
    fn check(&self, situation: &Counter, action: &CounterAction) -> Verdict {
        let meta = axiom_meta("NotAboveMax", "counter must not exceed maximum", "");
        if let CounterAction::Increment { by } = action
            && situation.value + by > situation.max
        {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

fn counter_apply(
    situation: &Counter,
    action: &CounterAction,
) -> Result<Counter, Box<dyn Counterexample>> {
    let mut next = situation.clone();
    match action {
        CounterAction::Increment { by } => next.value += by,
        CounterAction::Decrement { by } => next.value -= by,
        CounterAction::Reset => next.value = 0,
    }
    Ok(next)
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

#[crate::praxis_value(Verifiable)]
#[test]
fn test_increment() {
    let engine = make_engine(0, 10);
    let engine = engine.next(CounterAction::Increment { by: 5 }).unwrap();
    assert_eq!(engine.situation().value, 5);
}

#[crate::praxis_value(Verifiable)]
#[test]
fn test_decrement() {
    let engine = make_engine(5, 10);
    let engine = engine.next(CounterAction::Decrement { by: 3 }).unwrap();
    assert_eq!(engine.situation().value, 2);
}

#[crate::praxis_value(Verifiable)]
#[test]
fn test_reset() {
    let engine = make_engine(7, 10);
    let engine = engine.next(CounterAction::Reset).unwrap();
    assert_eq!(engine.situation().value, 0);
}

#[crate::praxis_value(Verifiable)]
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

#[crate::praxis_value(Honest)]
#[test]
fn test_below_zero_blocked() {
    let engine = make_engine(2, 10);
    let result = engine.next(CounterAction::Decrement { by: 5 });
    assert!(result.is_err());
}

#[crate::praxis_value(Honest)]
#[test]
fn test_above_max_blocked() {
    let engine = make_engine(8, 10);
    let result = engine.next(CounterAction::Increment { by: 5 });
    assert!(result.is_err());
}

#[crate::praxis_value(Verifiable)]
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

#[crate::praxis_value(Verifiable)]
#[test]
fn test_trace_records_violations() {
    let engine = make_engine(0, 10);
    let EngineError::Violated { engine, violations } =
        engine.next(CounterAction::Decrement { by: 5 }).unwrap_err()
    else {
        panic!("expected Violated")
    };
    assert_eq!(violations.len(), 1);
    assert_eq!(engine.trace().violations(), 1);
}

#[crate::praxis_value(Explainable, Verifiable)]
#[test]
fn test_violation_carries_typed_counterexample() {
    let engine = make_engine(2, 10);
    let EngineError::Violated { violations, .. } =
        engine.next(CounterAction::Decrement { by: 5 }).unwrap_err()
    else {
        panic!("expected Violated")
    };
    // Verify the counterexample is typed — carries axiom meta, not String fields.
    assert_eq!(violations[0].meta().name.as_str(), "NotBelowZero");
    assert!(
        violations[0]
            .meta()
            .description
            .as_str()
            .contains("below zero")
    );
}

#[crate::praxis_value(Explainable, Verifiable)]
#[test]
fn test_satisfied_precondition_is_typed_proof() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 1 })
        .unwrap();
    let entry = engine.trace().last().unwrap();
    // Both preconditions succeed — each verdict is Ok carrying a Proof.
    let proofs: Vec<&Box<dyn Proof>> = entry
        .precondition_verdicts
        .iter()
        .filter_map(|v| v.as_ref().ok())
        .collect();
    assert_eq!(proofs.len(), 2);
    let names: Vec<String> = proofs
        .iter()
        .map(|p| p.meta().name.as_str().to_string())
        .collect();
    assert!(names.iter().any(|n| n == "NotBelowZero"));
    assert!(names.iter().any(|n| n == "NotAboveMax"));
}

// =============================================================================
// Back/Forward tests
// =============================================================================

#[crate::praxis_value(Verifiable)]
#[test]
fn test_back_restores_previous() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 5 })
        .unwrap();
    assert_eq!(engine.situation().value, 5);
    let engine = engine.back().unwrap();
    assert_eq!(engine.situation().value, 0);
}

#[crate::praxis_value(Verifiable)]
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

#[crate::praxis_value(Verifiable)]
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

#[crate::praxis_value(Honest)]
#[test]
fn test_back_on_initial_fails() {
    let engine = make_engine(0, 10);
    assert!(engine.back().is_err());
}

#[crate::praxis_value(Honest)]
#[test]
fn test_forward_without_back_fails() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 5 })
        .unwrap();
    assert!(engine.forward().is_err());
}

#[crate::praxis_value(Verifiable)]
#[test]
fn test_next_after_back_clears_future() {
    let engine = make_engine(0, 10)
        .next(CounterAction::Increment { by: 5 })
        .unwrap()
        .next(CounterAction::Increment { by: 3 })
        .unwrap()
        .back()
        .unwrap();
    // Taking a new action after back() clears the redo-future.
    let engine = engine.next(CounterAction::Decrement { by: 2 }).unwrap();
    assert_eq!(engine.situation().value, 3);
    assert_eq!(engine.forward_depth(), 0);
}

// =============================================================================
// Proptest — typed preconditions, typed verdicts
// =============================================================================

proptest! {
    #[test]
    fn prop_increment_within_bounds_always_succeeds(
        start in 0i32..50,
        by in 1i32..50,
        max in 50i32..100,
    ) {
        prop_assume!(start + by <= max);
        let engine = make_engine(start, max);
        let result = engine.next(CounterAction::Increment { by });
        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap().situation().value, start + by);
    }

    #[test]
    fn prop_decrement_below_zero_always_fails(
        start in 0i32..10,
        by in 11i32..20,
    ) {
        prop_assume!(start - by < 0);
        let engine = make_engine(start, 100);
        let result = engine.next(CounterAction::Decrement { by });
        prop_assert!(result.is_err());
    }

    #[test]
    fn prop_back_forward_restores_value(
        start in 0i32..10,
        by in 1i32..10,
    ) {
        let engine = make_engine(start, 100);
        let engine = engine.next(CounterAction::Increment { by }).unwrap();
        let after_incr = engine.situation().value;
        let engine = engine.back().unwrap();
        prop_assert_eq!(engine.situation().value, start);
        let engine = engine.forward().unwrap();
        prop_assert_eq!(engine.situation().value, after_incr);
    }
}
crate::register_praxis_value!(prop_increment_within_bounds_always_succeeds, Verifiable);
crate::register_praxis_value!(prop_decrement_below_zero_always_fails, Honest);
crate::register_praxis_value!(prop_back_forward_restores_value, Verifiable);

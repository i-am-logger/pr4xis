//! Property-based tests for the Parallelism ontology, engine, and its
//! three cross-functors.

#![cfg(test)]

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use proptest::prelude::*;

use super::engine::{
    evaluate, evaluate_along, fib_dag, fibonacci, greedy_processor_counts, greedy_schedule, span,
};
use super::ontology::{
    CostCarrier, IsDeterministicByDefault, MachineClass, ParallelismCategory, ParallelismConcept,
    ParallelismOntology,
};
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Ontology, Quality};

fn arb_concept() -> impl Strategy<Value = ParallelismConcept> {
    proptest::sample::select(ParallelismConcept::variants())
}

proptest! {
    /// `MachineClass` is defined exactly on the four Flynn stream classes
    /// (Flynn's own generative 2x2 product) and nowhere else — including
    /// not on the abstract `MachineOrganization` parent.
    #[test]
    fn prop_machine_class_exactly_on_stream_classes(c in arb_concept()) {
        use ParallelismConcept as C;
        let is_stream_class = matches!(c, C::SISD | C::SIMD | C::MISD | C::MIMD);
        prop_assert_eq!(MachineClass.get(&c).is_some(), is_stream_class);
    }

    /// `CostCarrier` is defined exactly on the two counted cost measures
    /// (CLRS Ch. 27), and its value is always dimensionless.
    #[test]
    fn prop_cost_carrier_exactly_on_cost_measures(c in arb_concept()) {
        use ParallelismConcept as C;
        let is_cost = matches!(c, C::Work | C::Span);
        let value = CostCarrier.get(&c);
        prop_assert_eq!(value.is_some(), is_cost);
        if let Some(q) = value {
            prop_assert!(q.is_dimensionless());
        }
    }

    /// `IsDeterministicByDefault` is defined exactly on the two forms of
    /// parallelism it distinguishes (Bocchino et al. 2009; Lee 2006).
    #[test]
    fn prop_determinism_by_default_exactly_on_two_forms(c in arb_concept()) {
        use ParallelismConcept as C;
        let is_classified = matches!(c, C::DataParallelism | C::TaskParallelism);
        prop_assert_eq!(IsDeterministicByDefault.get(&c).is_some(), is_classified);
    }

    /// Every arrow of the category carries a non-empty name.
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in ParallelismCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    /// Structural + domain axioms all discharge, regardless of the
    /// sampling that drives the test.
    #[test]
    fn prop_all_axioms_hold(_seed in 0..16u32) {
        for axiom in ParallelismOntology::axioms() {
            if let Err(c) = axiom.verify() {
                prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
            }
        }
    }

    /// The greedy bound holds over a family of P-FIB DAGs, not just the
    /// fib(4) fixture: for random small `n` and every probed processor
    /// count, `max(ceil(T1/p), Tinf) <= T_p <= floor(T1/p) + Tinf`
    /// (Graham 1966; 1969; Brent 1974).
    #[test]
    fn prop_greedy_bound_over_small_dags(n in 2u64..7) {
        let dag = fib_dag(n);
        let t1 = dag.work();
        let t_inf = span(&dag);
        for p in greedy_processor_counts(t_inf) {
            let t_p = greedy_schedule(&dag, p).makespan();
            prop_assert!(t_p >= t_inf, "T_p >= Tinf");
            prop_assert!(p * t_p >= t1, "p*T_p >= T1");
            prop_assert!(t_p <= t1 / p + t_inf, "T_p <= floor(T1/p) + Tinf");
        }
    }

    /// Determinism of the DAG's result: every greedy schedule, over every
    /// probed `p`, computes the same value as the sequential elaboration
    /// (Bocchino et al. 2009) — for a family of P-FIB DAGs.
    #[test]
    fn prop_result_is_schedule_independent(n in 2u64..7) {
        let dag = fib_dag(n);
        let sequential = fibonacci(n);
        prop_assert_eq!(evaluate(&dag), sequential);
        for p in greedy_processor_counts(span(&dag)) {
            let schedule = greedy_schedule(&dag, p);
            prop_assert_eq!(evaluate_along(&dag, &schedule.flatten()), sequential);
        }
    }
}

pr4xis::register_praxis_value!(prop_machine_class_exactly_on_stream_classes, Verifiable);
pr4xis::register_praxis_value!(prop_cost_carrier_exactly_on_cost_measures, Verifiable);
pr4xis::register_praxis_value!(prop_determinism_by_default_exactly_on_two_forms, Verifiable);
pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
pr4xis::register_praxis_value!(prop_all_axioms_hold, Verifiable);
pr4xis::register_praxis_value!(prop_greedy_bound_over_small_dags, Verifiable);
pr4xis::register_praxis_value!(prop_result_is_schedule_independent, Deterministic);

//! Property-based tests for the Scheduler ontology, engine, and its
//! three cross-functors.

#![cfg(test)]

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use proptest::prelude::*;

use super::engine::{
    EDF_UTILIZATION_BOUND, PolicyOrder, TaskId, base_model_task, ll_increased_task_set, rm_admits,
    simulate_periodic, utilization,
};
use super::ontology::{
    IsPreemptive, PolicyPriorityAssignment, SchedulerCategory, SchedulerConcept, SchedulerOntology,
    TimingAttribute,
};
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Ontology, Quality};

fn arb_concept() -> impl Strategy<Value = SchedulerConcept> {
    proptest::sample::select(SchedulerConcept::variants())
}

/// Periods drawn from small divisors-friendly values so the hyperperiod
/// stays tractable: lcm of any subset of {2, 3, 4, 5, 6, 8, 10, 12} is
/// at most 120 slots.
fn arb_task_set() -> impl Strategy<Value = Vec<super::engine::PeriodicTask>> {
    let period = proptest::sample::select(vec![2u64, 3, 4, 5, 6, 8, 10, 12]);
    let task = (period, 1u64..=3).prop_map(|(p, c)| (c.min(p), p));
    proptest::collection::vec(task, 2..=3).prop_map(|specs| {
        specs
            .into_iter()
            .enumerate()
            .map(|(i, (c, p))| base_model_task(TaskId(i), c as f64, p as f64))
            .collect()
    })
}

proptest! {
    /// `TimingAttribute` is total exactly over the five timing concepts
    /// of the task model (Liu & Layland 1973 §2, §4-5) and nowhere
    /// else; times carry the second, ratios the unitless unit.
    #[test]
    fn prop_timing_attribute_totality(c in arb_concept()) {
        use SchedulerConcept as S;
        let is_timing = matches!(
            c,
            S::Period | S::Deadline | S::Wcet | S::Utilization | S::UtilizationBound
        );
        let value = TimingAttribute.get(&c);
        prop_assert_eq!(value.is_some(), is_timing);
        if let Some(u) = value {
            let dimensionless = matches!(c, S::Utilization | S::UtilizationBound);
            prop_assert_eq!(u.dimension.is_dimensionless(), dimensionless);
        }
    }

    /// `PolicyPriorityAssignment` is defined exactly on the concrete
    /// policy taxonomy (Liu & Layland 1973 §3/§7; Corbató et al. 1962;
    /// Molnar 2007) — the abstract parent and non-policies carry none.
    #[test]
    fn prop_policy_priority_assignment_totality(c in arb_concept()) {
        use SchedulerConcept as S;
        let is_classified_policy = matches!(
            c,
            S::FixedPriority
                | S::RateMonotonic
                | S::DeadlineMonotonic
                | S::EarliestDeadlineFirst
                | S::RoundRobin
                | S::MultilevelFeedbackQueue
                | S::FairShare
                | S::CompletelyFairScheduler
        );
        prop_assert_eq!(PolicyPriorityAssignment.get(&c).is_some(), is_classified_policy);
    }

    /// `IsPreemptive` is defined exactly on the six preemptive
    /// dispatch disciplines (Liu & Layland 1973; Leung & Whitehead
    /// 1982; Corbató et al. 1962).
    #[test]
    fn prop_is_preemptive_exactly_on_preemptive_policies(c in arb_concept()) {
        use SchedulerConcept as S;
        let is_preemptive_policy = matches!(
            c,
            S::RateMonotonic
                | S::DeadlineMonotonic
                | S::EarliestDeadlineFirst
                | S::FixedPriority
                | S::RoundRobin
                | S::MultilevelFeedbackQueue
        );
        prop_assert_eq!(IsPreemptive.get(&c).is_some(), is_preemptive_policy);
    }

    /// Every arrow of the category carries a non-empty name.
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in SchedulerCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    /// Structural + domain axioms all discharge, regardless of the
    /// sampling that drives the test.
    #[test]
    fn prop_all_axioms_hold(_seed in 0..16u32) {
        for axiom in SchedulerOntology::axioms() {
            if let Err(c) = axiom.verify() {
                prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
            }
        }
    }

    /// Liu & Layland (1973) Theorem 5, swept: any random task set the
    /// rate-monotonic admission test admits (U ≤ n(2^(1/n)-1)) meets
    /// every deadline in rate-monotonic simulation over its
    /// hyperperiod.
    #[test]
    fn prop_rm_admission_implies_deadlines_met(tasks in arb_task_set()) {
        if rm_admits(&tasks) {
            let trace = simulate_periodic(&tasks, PolicyOrder::RateMonotonic);
            prop_assert!(
                trace.met_all_deadlines(),
                "admitted set must be schedulable: U = {}",
                utilization(&tasks)
            );
        }
    }

    /// Liu & Layland (1973) Theorem 7, swept: any random task set with
    /// U ≤ 1 meets every deadline under the deadline-driven
    /// (earliest-deadline-first) order over its hyperperiod.
    #[test]
    fn prop_edf_feasible_up_to_full_utilization(tasks in arb_task_set()) {
        if utilization(&tasks) <= EDF_UTILIZATION_BOUND {
            let trace = simulate_periodic(&tasks, PolicyOrder::EarliestDeadlineFirst);
            prop_assert!(
                trace.met_all_deadlines(),
                "U <= 1 set must be EDF-schedulable: U = {}",
                utilization(&tasks)
            );
        }
    }
}

pr4xis::register_praxis_value!(prop_timing_attribute_totality, Verifiable);
pr4xis::register_praxis_value!(prop_policy_priority_assignment_totality, Verifiable);
pr4xis::register_praxis_value!(
    prop_is_preemptive_exactly_on_preemptive_policies,
    Verifiable
);
pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
pr4xis::register_praxis_value!(prop_all_axioms_hold, Verifiable);
pr4xis::register_praxis_value!(prop_rm_admission_implies_deadlines_met, Verifiable);
pr4xis::register_praxis_value!(prop_edf_feasible_up_to_full_utilization, Verifiable);

/// The Theorem 5 bound is sufficient, not necessary — Liu & Layland's
/// own §3 continuation (`C2` increased to 2): the engine's admission
/// test honestly refuses the set (U = 0.9 exceeds U_2 ≈ 0.828), yet
/// rate-monotonic simulation schedules it — and its trace exercises the
/// `Preempt` action (τ1's slot-2 release preempts τ2).
#[pr4xis::praxis_value(Honest, Verifiable)]
#[test]
fn rm_bound_is_sufficient_not_necessary() {
    let tasks = ll_increased_task_set();
    assert!(
        !rm_admits(&tasks),
        "U = 0.9 exceeds the two-task bound, so the sufficient test refuses"
    );
    let trace = simulate_periodic(&tasks, PolicyOrder::RateMonotonic);
    assert!(
        trace.met_all_deadlines(),
        "yet the set is rate-monotonic-schedulable (Liu & Layland 1973 sec 3)"
    );
    assert!(
        trace.preemption_count() > 0,
        "the trace realises preemption (the preemptive model in action)"
    );
}

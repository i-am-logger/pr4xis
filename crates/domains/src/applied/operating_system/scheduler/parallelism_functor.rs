//! Functor: Scheduler → Parallelism.
//!
//! Uniprocessor priority scheduling read as multiprocessor list
//! scheduling: Graham's list scheduler (Graham 1966; 1969) *is*
//! priority scheduling — a priority list orders the ready tasks and a
//! greedy dispatcher never idles a processor while the list is
//! non-empty. This functor is the *forgetful* direction into that
//! coarser theory:
//!
//! - every scheduling policy (and the priority/preemption machinery it
//!   dispatches with) collapses onto parallelism's one
//!   `GreedyScheduler` — the parallel theory keeps the greedy dispatch
//!   rule and forgets *which* priority list ordered it;
//! - `Wcet` maps to `Work` (a job's processor demand is its operation
//!   count), while `Deadline` and `Period` map — forgetfully — to
//!   `Span`: both are *time bounds* on the computation's completion,
//!   read as critical-path analogues; the deadline's hard-real-time
//!   semantics is forgotten;
//! - `Utilization` and its bound map to `Efficiency` (utilization is
//!   per-processor useful-work fraction — Liu & Layland §4's factor is
//!   exactly the parallel-efficiency reading for one processor);
//! - the blocking phenomena (`PriorityInversion`) and their protocol
//!   (`PriorityInheritance`) have no image in the deterministic
//!   work-span theory; they collapse onto `GreedyScheduler` as
//!   scheduler-protocol internals (documented collapse).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    SchedulerCategory, SchedulerConcept, SchedulerRelation, SchedulerRelationKind,
};
use crate::formal::systems::parallelism::ontology::{
    ParallelismCategory, ParallelismConcept, ParallelismRelation, ParallelismRelationKind,
};

/// The forgetful functor reading each scheduler concept as its coarser
/// parallel-scheduling image (Graham 1966; 1969; Liu & Layland 1973).
pub struct SchedulerToParallelism;

impl Functor for SchedulerToParallelism {
    type Source = SchedulerCategory;
    type Target = ParallelismCategory;

    fn map_object(obj: &SchedulerConcept) -> ParallelismConcept {
        use SchedulerConcept as S;
        match obj {
            // Graham list scheduling IS priority scheduling: every
            // policy is a priority list feeding the greedy dispatcher,
            // and the rank/preemption machinery — plus the blocking
            // phenomena and protocols — are its internals (documented
            // collapse; the work-span theory cannot see them).
            S::SchedulingPolicy
            | S::FixedPriority
            | S::RateMonotonic
            | S::DeadlineMonotonic
            | S::EarliestDeadlineFirst
            | S::RoundRobin
            | S::MultilevelFeedbackQueue
            | S::FairShare
            | S::CompletelyFairScheduler
            | S::Priority
            | S::Preemption
            | S::PriorityInversion
            | S::PriorityInheritance => ParallelismConcept::GreedyScheduler,
            // The behavioural units of decomposition.
            S::Task | S::Job => ParallelismConcept::ParallelTask,
            // Per-processor useful-work fraction.
            S::Utilization | S::UtilizationBound => ParallelismConcept::Efficiency,
            // A job's processor demand is its operation count.
            S::Wcet => ParallelismConcept::Work,
            // Forgetful time-bound reading: deadline and period are
            // completion-time bounds, read as critical-path analogues.
            S::Deadline | S::Period => ParallelismConcept::Span,
        }
    }

    fn map_morphism(m: &SchedulerRelation) -> ParallelismRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            SchedulerRelationKind::Identity => return ParallelismCategory::identity(&from),
            // Dispatching a task is the task-on-machine pairing, read
            // from the scheduler's side.
            SchedulerRelationKind::Schedules => ParallelismRelationKind::ExecutesOn,
            // A bound on admissible utilization stays a bound.
            SchedulerRelationKind::Bounds => ParallelismRelationKind::Bounds,
            // Dominance is feasibility-region inclusion (Liu & Layland
            // §7) — read as Subsumption.
            SchedulerRelationKind::Dominates => ParallelismRelationKind::Subsumption,
            // The protocol achieving its blocking bound reads as the
            // scheduler achieving a bound (the Graham/Brent reading).
            SchedulerRelationKind::Mitigates => ParallelismRelationKind::Achieves,
            // Preemptive dispatch is how the policy achieves its bound
            // — work-conserving dispatch is what makes the greedy
            // bound attainable.
            SchedulerRelationKind::Employs => ParallelismRelationKind::Achieves,
            // The four canonical Relations-ontology kinds map to their
            // namesakes (Smith 2005 OBO-RO).
            SchedulerRelationKind::Subsumption => ParallelismRelationKind::Subsumption,
            SchedulerRelationKind::Parthood => ParallelismRelationKind::Parthood,
            SchedulerRelationKind::Causation => ParallelismRelationKind::Causation,
            SchedulerRelationKind::Opposition => ParallelismRelationKind::Opposition,
        };
        ParallelismRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    SchedulerToParallelism,
    "Graham (1966) BSTJ 45(9); Graham (1969) SIAM J. Appl. Math. 17(2); Liu & Layland (1973) JACM 20(1)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<SchedulerToParallelism>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn every_policy_is_the_greedy_scheduler() {
        // Graham list scheduling IS priority scheduling: the policies
        // collapse onto the one greedy dispatcher.
        for c in [
            SchedulerConcept::SchedulingPolicy,
            SchedulerConcept::RateMonotonic,
            SchedulerConcept::EarliestDeadlineFirst,
            SchedulerConcept::RoundRobin,
            SchedulerConcept::CompletelyFairScheduler,
        ] {
            assert_eq!(
                SchedulerToParallelism::map_object(&c),
                ParallelismConcept::GreedyScheduler,
                "{c:?} is a priority list for the greedy dispatcher"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn cost_measures_land_on_work_span_efficiency() {
        assert_eq!(
            SchedulerToParallelism::map_object(&SchedulerConcept::Wcet),
            ParallelismConcept::Work
        );
        for c in [SchedulerConcept::Deadline, SchedulerConcept::Period] {
            assert_eq!(
                SchedulerToParallelism::map_object(&c),
                ParallelismConcept::Span,
                "{c:?} is a completion-time bound (forgetful reading)"
            );
        }
        for c in [
            SchedulerConcept::Utilization,
            SchedulerConcept::UtilizationBound,
        ] {
            assert_eq!(
                SchedulerToParallelism::map_object(&c),
                ParallelismConcept::Efficiency,
                "{c:?} is per-processor useful-work fraction"
            );
        }
    }
}

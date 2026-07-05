//! Functor: Scheduler → MAPE-K.
//!
//! A scheduler is an autonomic control loop in the sense of Kephart &
//! Chess (2003): it *monitors* the ready set of tasks and jobs,
//! *analyzes* feasibility via utilization against the schedulability
//! bound, *plans* the dispatch decision through its policy and priority
//! ranks, and *executes* the decision by preemption — all over the
//! *knowledge* of the task model it consults (periods, deadlines,
//! execution times, and the blocking phenomena and protocols that
//! constrain dispatch).
//!
//! # The mapping
//!
//! | Scheduler concept | MAPE-K phase | Why |
//! |---|---|---|
//! | `Task`, `Job` | `Monitor` | the observed ready set |
//! | `Utilization`, `UtilizationBound` | `Analyze` | feasibility analysis (Liu & Layland Theorem 5) |
//! | `SchedulingPolicy` + every policy child, `Priority` | `Plan` | the dispatch decision and its rank vocabulary |
//! | `Preemption` | `Execute` | the enacted context switch |
//! | `Period`, `Deadline`, `Wcet`, `PriorityInversion`, `PriorityInheritance` | `Knowledge` | the task-model knowledge the loop consults — the declared timing parameters and the blocking phenomenon/protocol facts that inform every dispatch decision |

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    SchedulerCategory, SchedulerConcept, SchedulerRelation, SchedulerRelationKind,
};
use crate::formal::systems::mape_k::ontology::{
    MapeKCategory, MapeKConcept, MapeKRelation, MapeKRelationKind,
};

/// Maps each scheduler concept to the MAPE-K phase it realises
/// (Kephart & Chess 2003; Liu & Layland 1973).
pub struct SchedulerToMapeK;

impl Functor for SchedulerToMapeK {
    type Source = SchedulerCategory;
    type Target = MapeKCategory;

    fn map_object(obj: &SchedulerConcept) -> MapeKConcept {
        use SchedulerConcept as S;
        match obj {
            // The observed ready set: what the loop senses.
            S::Task | S::Job => MapeKConcept::Monitor,
            // Feasibility analysis: utilization against the bound.
            S::Utilization | S::UtilizationBound => MapeKConcept::Analyze,
            // The dispatch decision: the policy family and the ordinal
            // rank vocabulary it decides with.
            S::SchedulingPolicy
            | S::FixedPriority
            | S::RateMonotonic
            | S::DeadlineMonotonic
            | S::EarliestDeadlineFirst
            | S::RoundRobin
            | S::MultilevelFeedbackQueue
            | S::FairShare
            | S::CompletelyFairScheduler
            | S::Priority => MapeKConcept::Plan,
            // The enacted decision: the context switch.
            S::Preemption => MapeKConcept::Execute,
            // The task-model knowledge the loop consults: declared
            // timing parameters and the blocking phenomenon/protocol
            // facts (documented in the module table).
            S::Period | S::Deadline | S::Wcet | S::PriorityInversion | S::PriorityInheritance => {
                MapeKConcept::Knowledge
            }
        }
    }

    fn map_morphism(m: &SchedulerRelation) -> MapeKRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            SchedulerRelationKind::Identity => return MapeKCategory::identity(&from),
            // The policy's dispatch decision handed to the monitored
            // ready set — the loop's actuation direction.
            SchedulerRelationKind::Schedules => MapeKRelationKind::HandsOffTo,
            // The feasibility analysis hands its admission verdict to
            // the planner — the loop's A → P leg.
            SchedulerRelationKind::Bounds => MapeKRelationKind::HandsOffTo,
            // Dominance is feasibility-region inclusion (Liu & Layland
            // §7: the deadline-driven algorithm schedules any set the
            // fixed-priority one can) — read as Subsumption.
            SchedulerRelationKind::Dominates => MapeKRelationKind::Subsumption,
            // Inheritance causally bounds inversion (Sha et al. 1990) —
            // a causal intervention inside the knowledge base.
            SchedulerRelationKind::Mitigates => MapeKRelationKind::Causation,
            // The plan hands off to execution — the loop's P → E leg
            // (a preemptive policy actuates through preemption).
            SchedulerRelationKind::Employs => MapeKRelationKind::HandsOffTo,
            // The four canonical Relations-ontology kinds map to their
            // namesakes (Smith 2005 OBO-RO).
            SchedulerRelationKind::Subsumption => MapeKRelationKind::Subsumption,
            SchedulerRelationKind::Parthood => MapeKRelationKind::Parthood,
            SchedulerRelationKind::Causation => MapeKRelationKind::Causation,
            SchedulerRelationKind::Opposition => MapeKRelationKind::Opposition,
        };
        MapeKRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    SchedulerToMapeK,
    "Kephart & Chess (2003) IEEE Computer 36(1); Liu & Layland (1973) JACM 20(1)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<SchedulerToMapeK>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn every_policy_plans() {
        for c in [
            SchedulerConcept::SchedulingPolicy,
            SchedulerConcept::FixedPriority,
            SchedulerConcept::RateMonotonic,
            SchedulerConcept::DeadlineMonotonic,
            SchedulerConcept::EarliestDeadlineFirst,
            SchedulerConcept::RoundRobin,
            SchedulerConcept::MultilevelFeedbackQueue,
            SchedulerConcept::FairShare,
            SchedulerConcept::CompletelyFairScheduler,
        ] {
            assert_eq!(
                SchedulerToMapeK::map_object(&c),
                MapeKConcept::Plan,
                "{c:?} is the dispatch decision"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn loop_phases_are_covered() {
        // The scheduler realises all four operative phases plus the
        // knowledge substrate — a full Kephart & Chess loop.
        assert_eq!(
            SchedulerToMapeK::map_object(&SchedulerConcept::Job),
            MapeKConcept::Monitor
        );
        assert_eq!(
            SchedulerToMapeK::map_object(&SchedulerConcept::Utilization),
            MapeKConcept::Analyze
        );
        assert_eq!(
            SchedulerToMapeK::map_object(&SchedulerConcept::Priority),
            MapeKConcept::Plan
        );
        assert_eq!(
            SchedulerToMapeK::map_object(&SchedulerConcept::Preemption),
            MapeKConcept::Execute
        );
        assert_eq!(
            SchedulerToMapeK::map_object(&SchedulerConcept::Wcet),
            MapeKConcept::Knowledge
        );
    }
}

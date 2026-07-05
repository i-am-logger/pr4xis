//! Functor: Scheduler → System.
//!
//! A scheduled processor is a regulated system in the sense of Ashby
//! (1956): the scheduling policies (and the priority-inheritance
//! protocol) are its controllers, tasks and jobs its components, the
//! timing parameters and ordinal ranks its constraints, preemption its
//! state transition, and priority inversion the unwanted return path —
//! the feedback pathology the inheritance controller regulates away.
//!
//! # Literature
//!
//! - **Ashby (1956)** *An Introduction to Cybernetics* — regulation,
//!   constraint, and the controller.
//! - **Liu & Layland (1973)** JACM 20(1) — the scheduling concepts this
//!   functor reads cybernetically.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    SchedulerCategory, SchedulerConcept, SchedulerRelation, SchedulerRelationKind,
};
use crate::formal::systems::ontology::{
    SystemCategory, SystemConcept, SystemRelation, SystemRelationKind,
};

/// Maps each scheduler concept to the systems-thinking role it plays
/// (Ashby 1956; Liu & Layland 1973).
pub struct SchedulerToSystem;

impl Functor for SchedulerToSystem {
    type Source = SchedulerCategory;
    type Target = SystemCategory;

    fn map_object(obj: &SchedulerConcept) -> SystemConcept {
        use SchedulerConcept as S;
        match obj {
            // The regulators: every dispatch policy, and the
            // priority-inheritance protocol that regulates blocking
            // (Ashby 1956 §11's regulator).
            S::SchedulingPolicy
            | S::FixedPriority
            | S::RateMonotonic
            | S::DeadlineMonotonic
            | S::EarliestDeadlineFirst
            | S::RoundRobin
            | S::MultilevelFeedbackQueue
            | S::FairShare
            | S::CompletelyFairScheduler
            | S::PriorityInheritance => SystemConcept::Controller,
            // The regulated elements.
            S::Task | S::Job => SystemConcept::Component,
            // The context switch is the system's state change.
            S::Preemption => SystemConcept::Transition,
            // Timing parameters, the schedulability bound, and the
            // ordinal rank restrict which behaviours are admissible
            // (Ashby 1956 §7's constraint).
            S::Period
            | S::Deadline
            | S::Wcet
            | S::Utilization
            | S::UtilizationBound
            | S::Priority => SystemConcept::Constraint,
            // Priority inversion is an unwanted return path: the low
            // component's state feeding back into the high one's
            // progress (Wiener's feedback, read pathologically).
            S::PriorityInversion => SystemConcept::Feedback,
        }
    }

    fn map_morphism(m: &SchedulerRelation) -> SystemRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            SchedulerRelationKind::Identity => return SystemCategory::identity(&from),
            // The controller regulating the components' execution.
            SchedulerRelationKind::Schedules => SystemRelationKind::Regulates,
            // A bound governs which configurations are admissible
            // (Ashby 1956 §7).
            SchedulerRelationKind::Bounds => SystemRelationKind::Governs,
            // Dominance is feasibility-region inclusion (Liu & Layland
            // §7) — read as Subsumption.
            SchedulerRelationKind::Dominates => SystemRelationKind::Subsumption,
            // The inheritance controller regulates the inversion
            // feedback pathway (Sha et al. 1990).
            SchedulerRelationKind::Mitigates => SystemRelationKind::Regulates,
            // A preemptive policy governs which preemption transitions
            // occur — the rule restricting valid transitions.
            SchedulerRelationKind::Employs => SystemRelationKind::Governs,
            // The four canonical Relations-ontology kinds map to their
            // namesakes (Smith 2005 OBO-RO).
            SchedulerRelationKind::Subsumption => SystemRelationKind::Subsumption,
            SchedulerRelationKind::Parthood => SystemRelationKind::Parthood,
            SchedulerRelationKind::Causation => SystemRelationKind::Causation,
            SchedulerRelationKind::Opposition => SystemRelationKind::Opposition,
        };
        SystemRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    SchedulerToSystem,
    "Ashby (1956) An Introduction to Cybernetics; Liu & Layland (1973) JACM 20(1)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<SchedulerToSystem>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn policies_and_inheritance_are_controllers() {
        for c in [
            SchedulerConcept::SchedulingPolicy,
            SchedulerConcept::RateMonotonic,
            SchedulerConcept::EarliestDeadlineFirst,
            SchedulerConcept::PriorityInheritance,
        ] {
            assert_eq!(
                SchedulerToSystem::map_object(&c),
                SystemConcept::Controller,
                "{c:?} regulates the system"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn inversion_is_the_feedback_pathology() {
        assert_eq!(
            SchedulerToSystem::map_object(&SchedulerConcept::PriorityInversion),
            SystemConcept::Feedback
        );
        // …and the timing parameters are the constraints the regulator
        // works against.
        for c in [
            SchedulerConcept::Period,
            SchedulerConcept::Deadline,
            SchedulerConcept::Wcet,
            SchedulerConcept::Priority,
        ] {
            assert_eq!(
                SchedulerToSystem::map_object(&c),
                SystemConcept::Constraint,
                "{c:?} constrains admissible behaviour"
            );
        }
    }
}

//! Functor: Parallelism → System.
//!
//! A parallel computation is a system in the sense of von Bertalanffy
//! (1968): processing elements and tasks are its components, the forms of
//! parallel execution its interactions, cost measures its state, whole-
//! computation ratios (speedup, efficiency) its emergent properties, cost
//! models its constraints, and the greedy scheduler its controller. This
//! is also `ProcessingElement`'s **faithful** home — the hardware
//! endurant maps to `Component` here, where the concurrency functor could
//! only collapse it.
//!
//! # Literature
//!
//! - **von Bertalanffy (1968)** *General System Theory* — components,
//!   interactions, emergence.
//! - **Flynn (1966)** — the machine organizations that are the system's
//!   components.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    ParallelismCategory, ParallelismConcept, ParallelismRelation, ParallelismRelationKind,
};
use crate::formal::systems::ontology::{
    SystemCategory, SystemConcept, SystemRelation, SystemRelationKind,
};

/// Maps each parallelism concept to the systems-thinking role it plays
/// (von Bertalanffy 1968; Flynn 1966).
pub struct ParallelismToSystem;

impl Functor for ParallelismToSystem {
    type Source = ParallelismCategory;
    type Target = SystemCategory;

    fn map_object(obj: &ParallelismConcept) -> SystemConcept {
        use ParallelismConcept as P;
        match obj {
            // Hardware endurants and behavioural units are the system's
            // elements — ProcessingElement's faithful home (Flynn 1966).
            P::ProcessingElement
            | P::ParallelTask
            | P::MachineOrganization
            | P::SISD
            | P::SIMD
            | P::MISD
            | P::MIMD => SystemConcept::Component,
            // Execution and its forms are the interactions that make the
            // components a system.
            P::ParallelExecution
            | P::DataParallelism
            | P::TaskParallelism
            | P::PipelineParallelism => SystemConcept::Interaction,
            // Quantitative cost attributes are configurations of the
            // system at a point in time.
            P::Work | P::Span | P::SequentialFraction => SystemConcept::State,
            // Whole-computation ratios are emergent properties the parts
            // do not individually possess.
            P::Speedup | P::Efficiency | P::ScaledSpeedup => SystemConcept::Emergence,
            // Cost models and the determinism guarantee restrict which
            // behaviours are admissible.
            P::CostModel | P::PRAM | P::BSP | P::LogP | P::DeterministicParallelism => {
                SystemConcept::Constraint
            }
            // The scheduler is the regulator keeping the system within
            // its bounds (Ashby 1956 §10).
            P::GreedyScheduler => SystemConcept::Controller,
        }
    }

    fn map_morphism(m: &ParallelismRelation) -> SystemRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            ParallelismRelationKind::Identity => return SystemCategory::identity(&from),
            // A bound / a cost model governs which configurations are
            // admissible.
            ParallelismRelationKind::Bounds | ParallelismRelationKind::Models => {
                SystemRelationKind::Governs
            }
            // The scheduler achieving a bound is a regulator acting.
            ParallelismRelationKind::Achieves => SystemRelationKind::Regulates,
            // A task executing on an element composes the system.
            ParallelismRelationKind::ExecutesOn => SystemRelationKind::ComposesInto,
            // A form of parallelism exhibiting determinism is an emergent
            // property arising from the interaction.
            ParallelismRelationKind::Exhibits => SystemRelationKind::ArisesFrom,
            // The four canonical Relations-ontology kinds map to their
            // namesakes (Smith 2005 OBO-RO).
            ParallelismRelationKind::Subsumption => SystemRelationKind::Subsumption,
            ParallelismRelationKind::Parthood => SystemRelationKind::Parthood,
            ParallelismRelationKind::Causation => SystemRelationKind::Causation,
            ParallelismRelationKind::Opposition => SystemRelationKind::Opposition,
        };
        SystemRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    ParallelismToSystem,
    "von Bertalanffy (1968) General System Theory; Flynn (1966) Proc. IEEE 54(12)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<ParallelismToSystem>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn processing_element_is_a_component() {
        // ProcessingElement's faithful home: the hardware endurant is a
        // system component here (unlike the concurrency functor, which
        // must collapse it).
        assert_eq!(
            ParallelismToSystem::map_object(&ParallelismConcept::ProcessingElement),
            SystemConcept::Component
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn speedup_is_emergent() {
        for c in [
            ParallelismConcept::Speedup,
            ParallelismConcept::Efficiency,
            ParallelismConcept::ScaledSpeedup,
        ] {
            assert_eq!(
                ParallelismToSystem::map_object(&c),
                SystemConcept::Emergence,
                "{c:?} should be emergent"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn scheduler_is_the_controller() {
        assert_eq!(
            ParallelismToSystem::map_object(&ParallelismConcept::GreedyScheduler),
            SystemConcept::Controller
        );
    }
}

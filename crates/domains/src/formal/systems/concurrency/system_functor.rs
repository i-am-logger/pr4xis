//! Functor: Concurrency → System.
//!
//! Every concurrent program is a system in the sense of von Bertalanffy
//! (1968): processes are its components, channels and parallel
//! composition its interactions, temporal properties its constraints,
//! synchronization mechanisms its controllers, and event orderings its
//! transitions (Lamport 1978). The functor makes that reading a
//! verified structure-preserving map instead of an analogy.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    ConcurrencyCategory, ConcurrencyConcept, ConcurrencyRelation, ConcurrencyRelationKind,
};
use crate::formal::systems::ontology::{
    SystemCategory, SystemConcept, SystemRelation, SystemRelationKind,
};

/// Maps each concurrency concept to the systems-thinking role it plays
/// (von Bertalanffy 1968; Ashby 1956 for controller/constraint).
pub struct ConcurrencyToSystem;

impl Functor for ConcurrencyToSystem {
    type Source = ConcurrencyCategory;
    type Target = SystemCategory;

    fn map_object(obj: &ConcurrencyConcept) -> SystemConcept {
        use ConcurrencyConcept as C;
        match obj {
            // The unit of concurrent composition is the system element.
            C::Process => SystemConcept::Component,
            // Communication media and the composition operator are the
            // relational glue between components.
            C::Channel | C::ParallelComposition => SystemConcept::Interaction,
            // Occupancy of the protected region, both progress
            // failures, and the clock's counter value are
            // configurations of the system at a point in time.
            C::CriticalSection | C::Deadlock | C::Livelock | C::LogicalClock => {
                SystemConcept::State
            }
            // Event orderings and interleaved merges are the system's
            // dynamics through its state space.
            C::HappensBefore | C::Interleaving => SystemConcept::Transition,
            // Temporal properties and the Coffman conditions restrict
            // which behaviours are admissible.
            C::MutualExclusion
            | C::SafetyProperty
            | C::LivenessProperty
            | C::HoldAndWait
            | C::NoPreemption
            | C::CircularWait => SystemConcept::Constraint,
            // Coordination mechanisms are the regulators that keep the
            // system within its constraints (Ashby 1956 §10).
            C::Synchronization | C::Semaphore | C::Monitor | C::Lock => SystemConcept::Controller,
        }
    }

    fn map_morphism(m: &ConcurrencyRelation) -> SystemRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            ConcurrencyRelationKind::Identity => return SystemCategory::identity(&from),
            // Enforces: a mechanism (Controller) upholding mutual
            // exclusion (Constraint) is exactly Ashby (1956) §10's
            // regulator acting on a constraint.
            ConcurrencyRelationKind::Enforces => SystemRelationKind::Regulates,
            // NecessaryFor: the Coffman conditions (Constraints) govern
            // whether the deadlock state is reachable.
            ConcurrencyRelationKind::NecessaryFor => SystemRelationKind::Governs,
            // CommunicatesVia: components and their interactions
            // compose the system (von Bertalanffy 1968 Ch. 1).
            ConcurrencyRelationKind::CommunicatesVia => SystemRelationKind::ComposesInto,
            // Respects: the clock value (State) feeds back into the
            // event ordering (Transition) — the cybernetic return path.
            ConcurrencyRelationKind::Respects => SystemRelationKind::FeedsBack,
            // ExpandsTo: the interleaved behaviour arises from the
            // parallel interaction (von Bertalanffy 1968 Ch. 3).
            ConcurrencyRelationKind::ExpandsTo => SystemRelationKind::ArisesFrom,
            // Violates: a progress failure stands in opposition to the
            // correctness constraint it breaks (Deadlock a safety
            // property, Livelock a liveness one) — canonical Opposition.
            ConcurrencyRelationKind::Violates => SystemRelationKind::Opposition,
            // The four canonical Relations-ontology kinds map to their
            // target namesakes (Smith 2005 OBO-RO).
            ConcurrencyRelationKind::Subsumption => SystemRelationKind::Subsumption,
            ConcurrencyRelationKind::Parthood => SystemRelationKind::Parthood,
            ConcurrencyRelationKind::Causation => SystemRelationKind::Causation,
            ConcurrencyRelationKind::Opposition => SystemRelationKind::Opposition,
        };
        SystemRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    ConcurrencyToSystem,
    "von Bertalanffy (1968) General System Theory; Lamport (1978) CACM 21(7)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<ConcurrencyToSystem>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn mechanisms_map_to_controller() {
        for c in [
            ConcurrencyConcept::Synchronization,
            ConcurrencyConcept::Semaphore,
            ConcurrencyConcept::Monitor,
            ConcurrencyConcept::Lock,
        ] {
            assert_eq!(
                ConcurrencyToSystem::map_object(&c),
                SystemConcept::Controller,
                "{c:?} should be a Controller"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn properties_map_to_constraint() {
        for c in [
            ConcurrencyConcept::MutualExclusion,
            ConcurrencyConcept::SafetyProperty,
            ConcurrencyConcept::LivenessProperty,
            ConcurrencyConcept::HoldAndWait,
            ConcurrencyConcept::NoPreemption,
            ConcurrencyConcept::CircularWait,
        ] {
            assert_eq!(
                ConcurrencyToSystem::map_object(&c),
                SystemConcept::Constraint,
                "{c:?} should be a Constraint"
            );
        }
    }
}

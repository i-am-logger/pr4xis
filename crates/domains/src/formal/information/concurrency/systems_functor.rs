use pr4xis::category::Functor;

use super::ontology::*;
use crate::formal::systems::ontology::*;

/// Functor: SystemsThinking → Concurrency.
///
/// Proves that every system IS concurrent — multiple interacting
/// components sharing state with coordination mechanisms.
///
/// The mapping:
/// - Component → Agent (components act)
/// - Interaction → SharedResource (interactions are shared state changes)
/// - Transition → Action (transitions are concurrent actions)
/// - Constraint → Protocol (constraints govern what's allowed)
/// - State → State (direct correspondence)
/// - Feedback → Synchronization (feedback IS coordination between agents)
/// - Controller → Synchronization (controller IS the synchronizer)
/// - Homeostasis → Deadlock (inverted: homeostasis = stable = no deadlock)
///   Actually: Homeostasis → State (stability is a state property)
/// - Emergence → RaceCondition (emergence depends on interaction order)
/// - Boundary → SharedResource (boundary defines what's shared)
pub struct SystemsToConcurrency;

impl Functor for SystemsToConcurrency {
    type Source = SystemCategory;
    type Target = ConcurrencyCategory;

    fn map_object(obj: &SystemConcept) -> ConcurrencyConcept {
        match obj {
            SystemConcept::Component => ConcurrencyConcept::Agent,
            SystemConcept::Interaction => ConcurrencyConcept::SharedResource,
            SystemConcept::State => ConcurrencyConcept::State,
            SystemConcept::Transition => ConcurrencyConcept::Action,
            SystemConcept::Constraint => ConcurrencyConcept::Protocol,
            SystemConcept::Feedback => ConcurrencyConcept::Synchronization,
            SystemConcept::Homeostasis => ConcurrencyConcept::State,
            SystemConcept::Emergence => ConcurrencyConcept::RaceCondition,
            SystemConcept::Boundary => ConcurrencyConcept::SharedResource,
            SystemConcept::Controller => ConcurrencyConcept::Synchronization,
        }
    }

    fn map_morphism(m: &SystemRelation) -> ConcurrencyRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        // Per #166 the proc-macro `ontology!` no longer emits a `Composed`
        // kind — composition of typed morphisms is partial (OBO-RO
        // transitive_over). For functor purposes, identity maps to
        // identity; the remaining direct-edge kinds are projected by
        // (source, target) pair.
        let kind = if m.kind == SystemRelationKind::Identity {
            ConcurrencyRelationKind::Identity
        } else {
            match (from, to) {
                (ConcurrencyConcept::Agent, ConcurrencyConcept::SharedResource) => {
                    ConcurrencyRelationKind::ActsOn
                }
                (ConcurrencyConcept::Synchronization, ConcurrencyConcept::Agent) => {
                    ConcurrencyRelationKind::Controls
                }
                (ConcurrencyConcept::Protocol, ConcurrencyConcept::Action) => {
                    ConcurrencyRelationKind::Governs
                }
                (ConcurrencyConcept::Action, ConcurrencyConcept::State) => {
                    ConcurrencyRelationKind::Changes
                }
                (ConcurrencyConcept::SharedResource, ConcurrencyConcept::RaceCondition) => {
                    ConcurrencyRelationKind::UnsynchronizedAccess
                }
                (ConcurrencyConcept::Synchronization, ConcurrencyConcept::Deadlock) => {
                    ConcurrencyRelationKind::ArisesFrom
                }
                // Off-pattern pairs project to Subsumption (canonical
                // OBO-RO kind, Smith 2005) — Identity is wrong when
                // from ≠ to because target's identity-aware compose then
                // breaks FunctorCompositionLaw.
                _ => ConcurrencyRelationKind::Subsumption,
            }
        };
        ConcurrencyRelation { from, to, kind }
    }
}
pr4xis::register_functor!(SystemsToConcurrency);

//! Functor: Concurrency → Parallelism.
//!
//! Concurrency (composing interacting activities, Milner 1980) and
//! parallelism (speeding up one computation with more hardware, Brent
//! 1974) are distinct theories (Pike 2012; Harper 2011). This functor is
//! the *forgetful* direction: it reads a concurrent structure as the
//! coarser parallel one, and in doing so **collapses the interleaving /
//! true-parallel-composition distinction** — Milner's `Interleaving`
//! (a nondeterministic single-thread merge) and his `ParallelComposition`
//! both land on parallelism's one `ParallelExecution`. That collapse is
//! the content of the adjunction gap in
//! [`super::super::parallelism::concurrency_functor`].
//!
//! # Literature
//!
//! - **Milner (1980)** *A Calculus of Communicating Systems*, LNCS 92 —
//!   parallel composition and its interleaving expansion.
//! - **Marlow (2012)** *Parallel and Concurrent Programming in Haskell*,
//!   CEFP 2011, LNCS 7241, §1.2 — the parallel/concurrent distinction.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    ConcurrencyCategory, ConcurrencyConcept, ConcurrencyRelation, ConcurrencyRelationKind,
};
use crate::formal::systems::parallelism::ontology::{
    ParallelismCategory, ParallelismConcept, ParallelismRelation, ParallelismRelationKind,
};

/// The forgetful functor reading each concurrency concept as its coarser
/// parallelism image (Milner 1980; Marlow 2012).
pub struct ConcurrencyToParallelism;

impl Functor for ConcurrencyToParallelism {
    type Source = ConcurrencyCategory;
    type Target = ParallelismCategory;

    fn map_object(obj: &ConcurrencyConcept) -> ParallelismConcept {
        use ConcurrencyConcept as C;
        use ParallelismConcept as P;
        match obj {
            // The unit of concurrent activity is the parallel strand
            // (behaviour to behaviour — the one faithful pairing).
            C::Process => P::ParallelTask,
            // Milner's parallel composition IS parallel execution …
            C::ParallelComposition => P::ParallelExecution,
            // … and so, forgetfully, is interleaving: this collapse is
            // the gap — parallelism cannot see the single-thread merge.
            C::Interleaving => P::ParallelExecution,
            // Communication media and critical regions have no parallel
            // image; they collapse onto the execution umbrella.
            C::Channel | C::CriticalSection => P::ParallelExecution,
            // Coordination mechanisms read as parallelism's scheduler.
            C::Synchronization | C::Semaphore | C::Monitor | C::Lock => P::GreedyScheduler,
            // Temporal correctness properties read as determinism —
            // parallelism's one correctness notion.
            C::MutualExclusion | C::SafetyProperty | C::LivenessProperty => {
                P::DeterministicParallelism
            }
            // Progress failures are concurrency phenomena with no
            // parallel image; they collapse onto the execution umbrella.
            C::Deadlock | C::Livelock | C::HoldAndWait | C::NoPreemption | C::CircularWait => {
                P::ParallelExecution
            }
            // Event ordering and the logical clock read as the
            // critical-path depth (Span is the longest happens-before
            // chain).
            C::HappensBefore | C::LogicalClock => P::Span,
        }
    }

    fn map_morphism(m: &ConcurrencyRelation) -> ParallelismRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            ConcurrencyRelationKind::Identity => return ParallelismCategory::identity(&from),
            // A mechanism enforcing a property reads as the scheduler
            // achieving a bound.
            ConcurrencyRelationKind::Enforces => ParallelismRelationKind::Achieves,
            // A necessary condition reads as a bound on an outcome.
            ConcurrencyRelationKind::NecessaryFor => ParallelismRelationKind::Bounds,
            // Communication over a channel reads as the executes-on
            // relation.
            ConcurrencyRelationKind::CommunicatesVia => ParallelismRelationKind::ExecutesOn,
            // The clock respecting the order reads as an ordering bound.
            ConcurrencyRelationKind::Respects => ParallelismRelationKind::Bounds,
            // Composition expanding to interleaving reads as a model
            // relation (the coarser view models the finer).
            ConcurrencyRelationKind::ExpandsTo => ParallelismRelationKind::Models,
            // A progress failure violating liveness is canonical
            // opposition.
            ConcurrencyRelationKind::Violates => ParallelismRelationKind::Opposition,
            // The four canonical Relations-ontology kinds map to their
            // namesakes (Smith 2005 OBO-RO).
            ConcurrencyRelationKind::Subsumption => ParallelismRelationKind::Subsumption,
            ConcurrencyRelationKind::Parthood => ParallelismRelationKind::Parthood,
            ConcurrencyRelationKind::Causation => ParallelismRelationKind::Causation,
            ConcurrencyRelationKind::Opposition => ParallelismRelationKind::Opposition,
        };
        ParallelismRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    ConcurrencyToParallelism,
    "Marlow (2012) LNCS 7241 §1.2; Milner (1980) A Calculus of Communicating Systems, LNCS 92"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<ConcurrencyToParallelism>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn interleaving_and_composition_collapse_together() {
        // The heart of the gap: two distinct concurrency concepts land
        // on one parallelism concept.
        assert_eq!(
            ConcurrencyToParallelism::map_object(&ConcurrencyConcept::Interleaving),
            ParallelismConcept::ParallelExecution
        );
        assert_eq!(
            ConcurrencyToParallelism::map_object(&ConcurrencyConcept::ParallelComposition),
            ParallelismConcept::ParallelExecution
        );
    }
}

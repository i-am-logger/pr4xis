//! Concurrency — the abstract theory of composing independent
//! sequential processes.
//!
//! The five source traditions this ontology draws on:
//!
//! - **Dijkstra (1968)** *Cooperating Sequential Processes* (EWD-123,
//!   written 1965), in Genuys (ed.) *Programming Languages*, Academic
//!   Press — critical sections, mutual exclusion, the P/V semaphore.
//! - **Hoare (1974)** *Monitors: An Operating System Structuring
//!   Concept*, CACM 17(10) — the monitor; **Hoare (1978)**
//!   *Communicating Sequential Processes*, CACM 21(8) — processes and
//!   channels as the units of concurrent composition.
//! - **Milner (1980)** *A Calculus of Communicating Systems*, Springer
//!   LNCS 92 — parallel composition and its interleaving expansion.
//! - **Lamport (1977)** *Proving the Correctness of Multiprocess
//!   Programs*, IEEE TSE SE-3(2) — safety vs. liveness; **Lamport
//!   (1978)** *Time, Clocks, and the Ordering of Events in a
//!   Distributed System*, CACM 21(7) — happens-before and logical
//!   clocks.
//! - **Coffman, Elphick & Shoshani (1971)** *System Deadlocks*, ACM
//!   Computing Surveys 3(2) — the four jointly necessary deadlock
//!   conditions.
//!
//! The five domain axioms are discharged against the small verified
//! fixtures in [`super::engine`]: a bounded exhaustive interleaving
//! explorer, a Lamport-clock event fixture, and a resource-allocation
//! graph.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use super::engine::{
    CoffmanCondition, EXPANSION_FIXTURE_ACTION_COUNT, ProcessId, coffman_fixture, explore,
    happens_before, lamport_fixture, logical_clocks, maximal_interleavings, mutex_initial,
    state_space_bound,
};

pr4xis::ontology! {
    name: "Concurrency",
    source: "Dijkstra (1968); Hoare (1974, 1978); Milner (1980); Lamport (1977, 1978); Coffman, Elphick & Shoshani (1971)",

    concepts: [
        // === Composition (Hoare 1978; Milner 1980) ===
        Process,
        Channel,
        ParallelComposition,
        Interleaving,

        // === Shared state and exclusion (Dijkstra 1968; Hoare 1974) ===
        CriticalSection,
        MutualExclusion,
        Synchronization,
        Semaphore,
        Monitor,
        Lock,

        // === Temporal properties (Lamport 1977) ===
        SafetyProperty,
        LivenessProperty,

        // === Progress failures and the Coffman conditions ===
        Deadlock,
        Livelock,
        HoldAndWait,
        NoPreemption,
        CircularWait,

        // === Event ordering (Lamport 1978) ===
        HappensBefore,
        LogicalClock,
    ],

    labels: {
        Process: ("en", "Process", "Hoare (1978) CACM 21(8): an independent sequential activity; the unit of concurrent composition."),
        Channel: ("en", "Channel", "Hoare (1978) CACM 21(8): a medium over which processes communicate by message passing."),
        ParallelComposition: ("en", "Parallel composition", "Milner (1980) CCS, LNCS 92; Hoare (1978): the operator P|Q composing processes."),
        Interleaving: ("en", "Interleaving", "Milner (1980) CCS, LNCS 92: the semantics in which concurrency is modeled as nondeterministic sequential merge."),
        CriticalSection: ("en", "Critical section", "Dijkstra (1968) EWD-123: a code region accessing shared state that must not be executed by two processes at once."),
        MutualExclusion: ("en", "Mutual exclusion", "Dijkstra (1968): the safety property that at most one process occupies a critical section; Coffman et al. (1971) condition 1."),
        Synchronization: ("en", "Synchronization", "Dijkstra (1968): the abstract parent of coordination mechanisms."),
        Semaphore: ("en", "Semaphore", "Dijkstra (1968) 'Cooperating Sequential Processes' (EWD-123, written 1965; in Genuys (ed.) Programming Languages, Academic Press): the P/V synchronization primitive."),
        Monitor: ("en", "Monitor", "Hoare (1974) CACM 17(10): a module encapsulating shared state with implicit mutual exclusion."),
        Lock: ("en", "Lock", "Dijkstra (1968): an acquire/release exclusion primitive - the generalization of the binary semaphore."),
        SafetyProperty: ("en", "Safety property", "Lamport (1977) IEEE TSE SE-3(2); Alpern & Schneider (1985) IPL 21(4): 'nothing bad happens' - a property violated by a finite prefix."),
        LivenessProperty: ("en", "Liveness property", "Lamport (1977); Alpern & Schneider (1985): 'something good eventually happens'."),
        Deadlock: ("en", "Deadlock", "Coffman, Elphick & Shoshani (1971) ACM Computing Surveys 3(2): a cycle of processes each holding resources the next needs; no progress."),
        Livelock: ("en", "Livelock", "Kwong (1979) 'On the Absence of Livelocks in Parallel Programs', LNCS 70: the term for a liveness failure (Lamport 1977) in which processes act forever without making progress."),
        HoldAndWait: ("en", "Hold and wait", "Coffman et al. (1971) condition 2: a process holds resources while waiting for more."),
        NoPreemption: ("en", "No preemption", "Coffman et al. (1971) condition 3: resources cannot be forcibly taken from their holders."),
        CircularWait: ("en", "Circular wait", "Coffman et al. (1971) condition 4: a circular chain of processes, each waiting for a resource the next holds."),
        HappensBefore: ("en", "Happens-before", "Lamport (1978) CACM 21(7), 'Time, Clocks, and the Ordering of Events in a Distributed System': the irreflexive partial order on events."),
        LogicalClock: ("en", "Logical clock", "Lamport (1978) CACM 21(7): a counter assigning timestamps consistent with happens-before."),
    },

    is_a: [
        (Semaphore, Synchronization),
        (Monitor, Synchronization),
        (Lock, Synchronization),
        (MutualExclusion, SafetyProperty),
    ],

    edges: [
        // === Coffman et al. (1971): the four jointly necessary
        // deadlock conditions (condition 1 = mutual exclusion). ===
        (MutualExclusion, Deadlock, NecessaryFor),
        (HoldAndWait, Deadlock, NecessaryFor),
        (NoPreemption, Deadlock, NecessaryFor),
        (CircularWait, Deadlock, NecessaryFor),

        // === What the mechanisms guarantee ===
        // Dijkstra (1968): P/V around the critical section.
        (Semaphore, MutualExclusion, Enforces),
        // Hoare (1974): the monitor's implicit exclusion.
        (Monitor, MutualExclusion, Enforces),

        // Hoare (1978): processes communicate by message passing.
        (Process, Channel, CommunicatesVia),

        // Lamport (1978) clock condition: C respects happens-before.
        (LogicalClock, HappensBefore, Respects),

        // Milner (1980) expansion law: P|Q expands to interleavings.
        (ParallelComposition, Interleaving, ExpandsTo),

        // Alpern & Schneider (1985): a deadlocked state is a discrete,
        // finite-prefix-observable "bad thing", so a deadlock Violates a
        // SAFETY property. Livelock — perpetual activity without progress
        // — is a genuine LIVENESS violation (Lamport 1977).
        (Deadlock, SafetyProperty, Violates),
        (Livelock, LivenessProperty, Violates),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// The Alpern & Schneider (1985) IPL 21(4) dichotomy: every temporal
/// property is the intersection of a safety property and a liveness
/// property, so the kind space is exactly {Safety, Liveness}.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalPropertyKind {
    /// "Nothing bad happens" — violated by a finite prefix.
    Safety,
    /// "Something good eventually happens" — never violated by a
    /// finite prefix alone.
    Liveness,
}

/// Which side of the Alpern & Schneider (1985) safety/liveness
/// dichotomy a concept lives on, *derived from the category's edges*
/// rather than hand-matched — so the edge and the Quality can never
/// disagree. A concept takes a pole's kind when it is that pole, is
/// subsumed under it (`MutualExclusion` is_a `SafetyProperty`), or
/// `Violates` it. Thus `Deadlock` (violates safety — a deadlocked state
/// is a discrete "bad thing", Alpern & Schneider 1985) is Safety and
/// `Livelock` (violates liveness — Lamport 1977) is Liveness. `None` for
/// concepts that are not temporal properties.
#[derive(Debug, Clone)]
pub struct PropertyKind;

impl Quality for PropertyKind {
    type Individual = ConcurrencyConcept;
    type Value = TemporalPropertyKind;

    fn get(&self, c: &ConcurrencyConcept) -> Option<TemporalPropertyKind> {
        use ConcurrencyConcept as C;
        use ConcurrencyRelationKind as R;
        // Each pole classifies itself, whatever is subsumed under it,
        // and whatever Violates it — read straight off the morphisms.
        for (pole, kind) in [
            (C::SafetyProperty, TemporalPropertyKind::Safety),
            (C::LivenessProperty, TemporalPropertyKind::Liveness),
        ] {
            if *c == pole
                || kinded_edge_exists(*c, pole, R::Subsumption)
                || kinded_edge_exists(*c, pole, R::Violates)
            {
                return Some(kind);
            }
        }
        None
    }
}

/// Whether a synchronization mechanism blocks the caller until it can
/// proceed: the semaphore's `P` suspends (Dijkstra 1968), the monitor's
/// entry and condition queues suspend (Hoare 1974), and the lock's
/// acquire is the binary-semaphore case (Dijkstra 1968). Its domain is
/// *derived from the `Synchronization` taxonomy* — the concrete
/// mechanisms are exactly its direct children — so the Quality is
/// coherent-by-construction and cannot drift when a mechanism is added.
/// `None` for concepts that are not concrete mechanisms (including the
/// abstract `Synchronization` parent).
#[derive(Debug, Clone)]
pub struct IsBlockingPrimitive;

impl Quality for IsBlockingPrimitive {
    type Individual = ConcurrencyConcept;
    type Value = bool;

    fn get(&self, c: &ConcurrencyConcept) -> Option<bool> {
        if direct_children_of(ConcurrencyConcept::Synchronization).contains(c) {
            Some(true)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn direct_children_of(parent: ConcurrencyConcept) -> Vec<ConcurrencyConcept> {
    use pr4xis::category::{Arrow, Category};
    ConcurrencyCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == ConcurrencyRelationKind::Subsumption && m.target() == parent)
        .map(|m| m.source())
        .collect()
}

fn kinded_edge_exists(
    from: ConcurrencyConcept,
    to: ConcurrencyConcept,
    kind: ConcurrencyRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    ConcurrencyCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Lamport (1978): happens-before is a strict partial order —
/// irreflexive, asymmetric, transitive — verified on the engine's
/// three-process event fixture.
pub struct HappensBeforeStrictPartialOrder;

impl Axiom for HappensBeforeStrictPartialOrder {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let events = lamport_fixture();
        let hb = happens_before(&events);
        let irreflexive = hb.iter().all(|(a, b)| a != b);
        let asymmetric = hb.iter().all(|(a, b)| !hb.contains(&(*b, *a)));
        let transitive = hb.iter().all(|(a, b)| {
            hb.iter()
                .filter(|(b2, _)| b2 == b)
                .all(|(_, c)| hb.contains(&(*a, *c)))
        });
        // Non-vacuity: the fixture actually orders some events.
        let ok = !hb.is_empty() && irreflexive && asymmetric && transitive;
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "HappensBeforeStrictPartialOrder",
        "on the event fixture, happens-before is irreflexive, asymmetric, and transitive",
        "Lamport (1978) CACM 21(7) \u{00a7}Logical Clocks"
    );
}
pr4xis::register_axiom!(
    HappensBeforeStrictPartialOrder,
    "Lamport (1978) CACM 21(7) \u{00a7}Logical Clocks"
);

/// Lamport (1978) clock condition: for all events a, b — if a
/// happens-before b then C(a) < C(b) — verified on the engine's event
/// fixture; the category carries the matching `Respects` edge.
pub struct ClockCondition;

impl Axiom for ClockCondition {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let events = lamport_fixture();
        let hb = happens_before(&events);
        let Ok(clocks) = logical_clocks(&events) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let condition = !hb.is_empty() && hb.iter().all(|(a, b)| clocks[*a] < clocks[*b]);
        let edge = kinded_edge_exists(
            ConcurrencyConcept::LogicalClock,
            ConcurrencyConcept::HappensBefore,
            ConcurrencyRelationKind::Respects,
        );
        if condition && edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ClockCondition",
        "for all fixture events a, b: a happens-before b implies C(a) < C(b)",
        "Lamport (1978) CACM 21(7), Time, Clocks, and the Ordering of Events in a Distributed System"
    );
}
pr4xis::register_axiom!(
    ClockCondition,
    "Lamport (1978) CACM 21(7), Time, Clocks, and the Ordering of Events in a Distributed System"
);

/// Coffman, Elphick & Shoshani (1971): the four conditions are jointly
/// necessary for deadlock. The category carries all four `NecessaryFor`
/// edges, and on the resource-allocation-graph fixture the deadlock
/// cycle exists with all four conditions holding and disappears when
/// any single one is denied.
pub struct CoffmanConditionsNecessary;

impl Axiom for CoffmanConditionsNecessary {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let necessary_for = |from: ConcurrencyConcept| {
            kinded_edge_exists(
                from,
                ConcurrencyConcept::Deadlock,
                ConcurrencyRelationKind::NecessaryFor,
            )
        };
        let edges_present = necessary_for(ConcurrencyConcept::MutualExclusion)
            && necessary_for(ConcurrencyConcept::HoldAndWait)
            && necessary_for(ConcurrencyConcept::NoPreemption)
            && necessary_for(ConcurrencyConcept::CircularWait);

        let graph = coffman_fixture();
        let all_conditions_hold = CoffmanCondition::ALL
            .iter()
            .all(|c| graph.condition_holds(*c));
        let deadlocked = graph.has_deadlock_cycle();
        let each_denial_breaks_it = CoffmanCondition::ALL.iter().all(|c| {
            let denied = graph.deny(*c);
            !denied.condition_holds(*c) && !denied.has_deadlock_cycle()
        });

        if edges_present && all_conditions_hold && deadlocked && each_denial_breaks_it {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CoffmanConditionsNecessary",
        "all four NecessaryFor edges are present; on the RAG fixture the deadlock cycle exists with all four conditions and disappears when any one is denied",
        "Coffman, Elphick & Shoshani (1971) ACM Computing Surveys 3(2) \u{00a7}2"
    );
}
pr4xis::register_axiom!(
    CoffmanConditionsNecessary,
    "Coffman, Elphick & Shoshani (1971) ACM Computing Surveys 3(2) \u{00a7}2"
);

/// Dijkstra (1968): a binary semaphore around the critical section
/// enforces mutual exclusion. Bounded exhaustive interleaving of the
/// two-process fixture reaches no state with two processes inside the
/// protected region; the category carries the matching `Enforces` edge.
pub struct SemaphoreEnforcesMutualExclusion;

impl Axiom for SemaphoreEnforcesMutualExclusion {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let initial = mutex_initial();
        let states = explore(&initial);
        let bounded = states.len() <= state_space_bound(initial.processes.len());
        let exclusion = states.iter().all(|s| !s.violates_mutual_exclusion());
        // Non-vacuity: some reachable state actually occupies the
        // critical section, so the exclusion check has bite.
        let occupied = states.iter().any(|s| s.critical_occupancy() > 0);
        let edge = kinded_edge_exists(
            ConcurrencyConcept::Semaphore,
            ConcurrencyConcept::MutualExclusion,
            ConcurrencyRelationKind::Enforces,
        );
        if bounded && exclusion && occupied && edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SemaphoreEnforcesMutualExclusion",
        "bounded exhaustive interleaving of two P/critical/V processes under a binary semaphore reaches no state with both in the critical section",
        "Dijkstra (1968) Cooperating Sequential Processes (EWD-123), in Genuys (ed.) Programming Languages, Academic Press"
    );
}
pr4xis::register_axiom!(
    SemaphoreEnforcesMutualExclusion,
    "Dijkstra (1968) Cooperating Sequential Processes (EWD-123), in Genuys (ed.) Programming Languages, Academic Press"
);

/// Milner (1980) expansion law: for two atomic actions a and b, the
/// maximal traces of a|b are exactly {ab, ba} — computed by the
/// engine's exhaustive interleaver; the category carries the matching
/// `ExpandsTo` edge.
pub struct ExpansionLaw;

impl Axiom for ExpansionLaw {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let traces = maximal_interleavings(EXPANSION_FIXTURE_ACTION_COUNT);
        // Milner (1980): a.NIL | b.NIL = a.b.NIL + b.a.NIL — exactly
        // the two orders of the two atomic actions.
        let expected = [
            vec![ProcessId(0), ProcessId(1)],
            vec![ProcessId(1), ProcessId(0)],
        ];
        let set_equal = traces.len() == expected.len()
            && expected.iter().all(|t| traces.contains(t))
            && traces
                .iter()
                .enumerate()
                .all(|(i, t)| traces.iter().skip(i + 1).all(|u| t != u));
        let edge = kinded_edge_exists(
            ConcurrencyConcept::ParallelComposition,
            ConcurrencyConcept::Interleaving,
            ConcurrencyRelationKind::ExpandsTo,
        );
        if set_equal && edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ExpansionLaw",
        "the set of maximal traces of a|b for two atomic actions equals {ab, ba}",
        "Milner (1980) A Calculus of Communicating Systems, LNCS 92, expansion law"
    );
}
pr4xis::register_axiom!(
    ExpansionLaw,
    "Milner (1980) A Calculus of Communicating Systems, LNCS 92, expansion law"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for ConcurrencyOntology {
    type Cat = ConcurrencyCategory;
    type Qual = PropertyKind;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(HappensBeforeStrictPartialOrder));
        axioms.push(Box::new(ClockCondition));
        axioms.push(Box::new(CoffmanConditionsNecessary));
        axioms.push(Box::new(SemaphoreEnforcesMutualExclusion));
        axioms.push(Box::new(ExpansionLaw));
        axioms
    }
}

/// Direct children of `Synchronization` — used by tests; grounded in
/// the Subsumption edges declared above (Dijkstra 1968; Hoare 1974).
pub fn synchronization_mechanisms() -> Vec<ConcurrencyConcept> {
    direct_children_of(ConcurrencyConcept::Synchronization)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ConcurrencyCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ConcurrencyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn happens_before_strict_partial_order_holds() {
        assert!(HappensBeforeStrictPartialOrder.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn clock_condition_holds() {
        assert!(ClockCondition.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn coffman_conditions_necessary_holds() {
        assert!(CoffmanConditionsNecessary.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn semaphore_enforces_mutual_exclusion_holds() {
        assert!(SemaphoreEnforcesMutualExclusion.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn expansion_law_holds() {
        assert!(ExpansionLaw.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn safety_liveness_classification() {
        let q = PropertyKind;
        // Safety: the safety pole, MutualExclusion (is_a SafetyProperty),
        // and Deadlock — a deadlocked state is a discrete, finite-prefix
        // "bad thing" (Alpern & Schneider 1985).
        for c in [
            ConcurrencyConcept::SafetyProperty,
            ConcurrencyConcept::MutualExclusion,
            ConcurrencyConcept::Deadlock,
        ] {
            assert_eq!(q.get(&c), Some(TemporalPropertyKind::Safety), "{c:?}");
        }
        // Liveness: the liveness pole and Livelock — perpetual activity
        // without progress, a genuine liveness violation (Lamport 1977).
        for c in [
            ConcurrencyConcept::LivenessProperty,
            ConcurrencyConcept::Livelock,
        ] {
            assert_eq!(q.get(&c), Some(TemporalPropertyKind::Liveness), "{c:?}");
        }
        assert_eq!(q.get(&ConcurrencyConcept::Process), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn synchronization_taxonomy() {
        let mechanisms = synchronization_mechanisms();
        let expected = [
            ConcurrencyConcept::Semaphore,
            ConcurrencyConcept::Monitor,
            ConcurrencyConcept::Lock,
        ];
        assert_eq!(mechanisms.len(), expected.len());
        for c in expected {
            assert!(mechanisms.contains(&c), "{c:?} should be a mechanism");
            assert_eq!(IsBlockingPrimitive.get(&c), Some(true), "{c:?}");
        }
        assert_eq!(
            IsBlockingPrimitive.get(&ConcurrencyConcept::Synchronization),
            None,
            "abstract parent has no blocking discipline of its own"
        );
    }
}

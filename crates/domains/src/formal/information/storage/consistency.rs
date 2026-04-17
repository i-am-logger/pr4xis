//! Consistency ontology — the lattice of consistency models.
//!
//! Consistency models define what ordering guarantees a system provides
//! for concurrent operations. They form a bounded lattice: stronger models
//! are subsets of weaker models.
//!
//! References:
//! - Viotti & Vukolic, "Consistency in Non-Transactional Distributed Storage
//!   Systems" (2016, ACM Computing Surveys)
//! - Herlihy & Wing, "Linearizability" (1990, ACM TOPLAS)
//! - Lamport (1979) sequential consistency; Lamport (1978) happens-before
//! - Vogels, "Eventually Consistent" (2009, CACM)
//! - Papadimitriou, "The Serializability of Concurrent Database Updates" (1979, JACM)
//! - Terry et al., "Session Guarantees for Weakly Consistent Replicated Data" (1994, ACM TOCS)

pr4xis::ontology! {
    name: "Consistency",
    source: "Viotti & Vukolic (2016); Herlihy & Wing (1990)",
    being: AbstractObject,

    concepts: [
        Linearizable,
        SequentiallyConsistent,
        Serializable,
        Causal,
        Pram,
        MonotonicReads,
        ReadYourWrites,
        EventuallyConsistent,
    ],

    labels: {
        Linearizable: ("en", "Linearizable", "Operations appear atomic, ordered consistently with real-time. Herlihy & Wing (1990). Strongest achievable model."),
        SequentiallyConsistent: ("en", "Sequentially consistent", "Operations appear in some total order consistent with program order. Lamport (1979). Drops real-time constraint."),
        Serializable: ("en", "Serializable", "Transactions appear to execute in some serial order. Papadimitriou (1979). Incomparable with linearizability."),
        Causal: ("en", "Causal", "Causally related operations seen in same order everywhere. Concurrent operations may differ across observers. Lamport (1978) happens-before; Ahamad et al. (1995)."),
        Pram: ("en", "PRAM", "Each process's writes seen in program order by all. No cross-process ordering guarantee. Lipton & Sandberg (1988)."),
        MonotonicReads: ("en", "Monotonic reads", "Successive reads by a process return same or newer values. Terry et al. (1994) session guarantee."),
        ReadYourWrites: ("en", "Read-your-writes", "A process always sees its own prior writes. Terry et al. (1994) session guarantee."),
        EventuallyConsistent: ("en", "Eventually consistent", "If no new updates, all replicas eventually converge. Vogels (2009). Among the weakest useful guarantees."),
    },

    edges: [
        (Linearizable, SequentiallyConsistent, Weakens),
        (SequentiallyConsistent, Causal, Weakens),
        (Serializable, Causal, Weakens),
        (Causal, Pram, Weakens),
        (Causal, MonotonicReads, Weakens),
        (Causal, ReadYourWrites, Weakens),
        (Pram, EventuallyConsistent, Weakens),
        (MonotonicReads, EventuallyConsistent, Weakens),
        (ReadYourWrites, EventuallyConsistent, Weakens),
    ],
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<ConsistencyCategory>().unwrap();
    }

    #[test]
    fn has_eight_models() {
        assert_eq!(ConsistencyConcept::variants().len(), 8);
    }

    #[test]
    fn linearizable_weakens_to_sequential() {
        let m = ConsistencyCategory::morphisms();
        assert!(m.iter().any(|r| r.from == ConsistencyConcept::Linearizable
            && r.to == ConsistencyConcept::SequentiallyConsistent
            && r.kind == ConsistencyRelationKind::Weakens));
    }

    #[test]
    fn sequential_weakens_to_causal() {
        let m = ConsistencyCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == ConsistencyConcept::SequentiallyConsistent
                    && r.to == ConsistencyConcept::Causal
                    && r.kind == ConsistencyRelationKind::Weakens)
        );
    }

    #[test]
    fn linearizable_and_serializable_incomparable() {
        let m = ConsistencyCategory::morphisms();
        assert!(!m.iter().any(|r| r.from == ConsistencyConcept::Linearizable
            && r.to == ConsistencyConcept::Serializable
            && r.kind == ConsistencyRelationKind::Weakens));
        assert!(!m.iter().any(|r| r.from == ConsistencyConcept::Serializable
            && r.to == ConsistencyConcept::Linearizable
            && r.kind == ConsistencyRelationKind::Weakens));
    }

    #[test]
    fn eventual_is_weakest() {
        let m = ConsistencyCategory::morphisms();
        for model in ConsistencyConcept::variants() {
            if model == ConsistencyConcept::EventuallyConsistent {
                continue;
            }
            assert!(
                m.iter()
                    .any(|r| r.from == model && r.to == ConsistencyConcept::EventuallyConsistent),
                "{model:?} should reach EventuallyConsistent"
            );
        }
    }

    #[test]
    fn session_guarantees_weaken_to_eventual() {
        let m = ConsistencyCategory::morphisms();
        for session in [
            ConsistencyConcept::MonotonicReads,
            ConsistencyConcept::ReadYourWrites,
        ] {
            assert!(m.iter().any(|r| r.from == session
                && r.to == ConsistencyConcept::EventuallyConsistent
                && r.kind == ConsistencyRelationKind::Weakens));
        }
    }

    #[test]
    fn causal_implies_session_guarantees() {
        let m = ConsistencyCategory::morphisms();
        assert!(m.iter().any(|r| r.from == ConsistencyConcept::Causal
            && r.to == ConsistencyConcept::MonotonicReads
            && r.kind == ConsistencyRelationKind::Weakens));
        assert!(m.iter().any(|r| r.from == ConsistencyConcept::Causal
            && r.to == ConsistencyConcept::ReadYourWrites
            && r.kind == ConsistencyRelationKind::Weakens));
    }

    #[test]
    fn linearizable_reaches_eventual_transitively() {
        let m = ConsistencyCategory::morphisms();
        assert!(m.iter().any(|r| r.from == ConsistencyConcept::Linearizable
            && r.to == ConsistencyConcept::EventuallyConsistent));
    }
}

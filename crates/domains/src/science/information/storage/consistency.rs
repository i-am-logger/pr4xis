use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

// Consistency ontology — the lattice of consistency models.
//
// Consistency models define what ordering guarantees a system provides
// for concurrent operations. They form a bounded lattice: stronger models
// are subsets of weaker models (every linearizable execution is also
// sequentially consistent, etc.).
//
// References:
// - Viotti & Vukolic, "Consistency in Non-Transactional Distributed Storage
//   Systems" (2016, ACM Computing Surveys) — THE comprehensive lattice
// - Herlihy & Wing, "Linearizability: A Correctness Condition for Concurrent
//   Objects" (1990, ACM TOPLAS) — gold standard definition
// - Lamport, "How to Make a Multiprocessor Computer That Correctly Executes
//   Multiprocess Programs" (1979, IEEE Trans. Comp.) — sequential consistency
// - Lamport, "Time, Clocks, and the Ordering of Events in a Distributed
//   System" (1978, CACM) — happens-before, causal consistency
// - Vogels, "Eventually Consistent" (2009, CACM)
// - Papadimitriou, "The Serializability of Concurrent Database Updates"
//   (1979, JACM) — serializability
// - Terry et al., "Session Guarantees for Weakly Consistent Replicated Data"
//   (1994, ACM TOCS) — monotonic reads, read-your-writes

/// Consistency models — objects in the consistency lattice.
///
/// Ordered from strongest to weakest. The morphisms are weakening maps:
/// if model A is stronger than B, there exists a morphism A → B
/// (every A-consistent history is also B-consistent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsistencyModel {
    /// Operations appear atomic, ordered consistently with real-time.
    /// Herlihy & Wing (1990). Strongest achievable model.
    Linearizable,

    /// Operations appear in some total order consistent with program order.
    /// Lamport (1979). Drops real-time constraint.
    SequentiallyConsistent,

    /// Transactions appear to execute in some serial order.
    /// Papadimitriou (1979). Incomparable with linearizability.
    Serializable,

    /// Causally related operations seen in same order everywhere.
    /// Concurrent operations may differ across observers.
    /// Lamport (1978) happens-before. Ahamad et al. (1995).
    Causal,

    /// Each process's writes seen in program order by all.
    /// No cross-process ordering guarantee.
    /// Lipton & Sandberg (1988).
    Pram,

    /// Successive reads by a process return same or newer values.
    /// Terry et al. (1994) session guarantee.
    MonotonicReads,

    /// A process always sees its own prior writes.
    /// Terry et al. (1994) session guarantee.
    ReadYourWrites,

    /// If no new updates, all replicas eventually converge.
    /// Vogels (2009). Among the weakest useful guarantees.
    EventuallyConsistent,
}

impl Entity for ConsistencyModel {
    fn variants() -> Vec<Self> {
        vec![
            Self::Linearizable,
            Self::SequentiallyConsistent,
            Self::Serializable,
            Self::Causal,
            Self::Pram,
            Self::MonotonicReads,
            Self::ReadYourWrites,
            Self::EventuallyConsistent,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConsistencyRelation {
    pub from: ConsistencyModel,
    pub to: ConsistencyModel,
    pub kind: ConsistencyRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsistencyRelationKind {
    Identity,
    /// Weakening: from is strictly stronger than to.
    /// Every history allowed under `from` is also allowed under `to`.
    Weakens,
    Composed,
}

impl Relationship for ConsistencyRelation {
    type Object = ConsistencyModel;
    fn source(&self) -> ConsistencyModel {
        self.from
    }
    fn target(&self) -> ConsistencyModel {
        self.to
    }
}

pub struct ConsistencyCategory;

impl Category for ConsistencyCategory {
    type Object = ConsistencyModel;
    type Morphism = ConsistencyRelation;

    fn identity(obj: &ConsistencyModel) -> ConsistencyRelation {
        ConsistencyRelation {
            from: *obj,
            to: *obj,
            kind: ConsistencyRelationKind::Identity,
        }
    }

    fn compose(f: &ConsistencyRelation, g: &ConsistencyRelation) -> Option<ConsistencyRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == ConsistencyRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == ConsistencyRelationKind::Identity {
            return Some(f.clone());
        }
        Some(ConsistencyRelation {
            from: f.from,
            to: g.to,
            kind: ConsistencyRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<ConsistencyRelation> {
        use ConsistencyModel as C;
        use ConsistencyRelationKind as R;
        let mut m = Vec::new();

        for c in ConsistencyModel::variants() {
            m.push(ConsistencyRelation {
                from: c,
                to: c,
                kind: R::Identity,
            });
        }

        // The Viotti & Vukolic (2016) lattice — direct weakening edges:
        //
        //   Linearizable → SequentiallyConsistent → Causal → Pram → Eventual
        //                                              ↓
        //                                      MonotonicReads → Eventual
        //                                      ReadYourWrites → Eventual
        //
        //   Serializable → Causal (in single-object case)
        //
        // Note: Linearizable and Serializable are INCOMPARABLE in general.
        // Linearizable applies to single objects; Serializable to transactions.

        // Linearizable → SequentiallyConsistent (drop real-time)
        m.push(ConsistencyRelation {
            from: C::Linearizable,
            to: C::SequentiallyConsistent,
            kind: R::Weakens,
        });

        // SequentiallyConsistent → Causal (drop total order)
        m.push(ConsistencyRelation {
            from: C::SequentiallyConsistent,
            to: C::Causal,
            kind: R::Weakens,
        });

        // Serializable → Causal (in the single-object projection)
        m.push(ConsistencyRelation {
            from: C::Serializable,
            to: C::Causal,
            kind: R::Weakens,
        });

        // Causal → PRAM (causal implies PRAM — Ahamad et al.)
        m.push(ConsistencyRelation {
            from: C::Causal,
            to: C::Pram,
            kind: R::Weakens,
        });

        // Causal → MonotonicReads (causal implies monotonic reads)
        m.push(ConsistencyRelation {
            from: C::Causal,
            to: C::MonotonicReads,
            kind: R::Weakens,
        });

        // Causal → ReadYourWrites (causal implies read-your-writes)
        m.push(ConsistencyRelation {
            from: C::Causal,
            to: C::ReadYourWrites,
            kind: R::Weakens,
        });

        // PRAM → Eventual
        m.push(ConsistencyRelation {
            from: C::Pram,
            to: C::EventuallyConsistent,
            kind: R::Weakens,
        });

        // MonotonicReads → Eventual
        m.push(ConsistencyRelation {
            from: C::MonotonicReads,
            to: C::EventuallyConsistent,
            kind: R::Weakens,
        });

        // ReadYourWrites → Eventual
        m.push(ConsistencyRelation {
            from: C::ReadYourWrites,
            to: C::EventuallyConsistent,
            kind: R::Weakens,
        });

        // Transitive compositions (closure of the lattice)
        // Linearizable → everything weaker
        for target in [
            C::Causal,
            C::Pram,
            C::MonotonicReads,
            C::ReadYourWrites,
            C::EventuallyConsistent,
        ] {
            m.push(ConsistencyRelation {
                from: C::Linearizable,
                to: target,
                kind: R::Composed,
            });
        }
        // SequentiallyConsistent → everything below Causal
        for target in [
            C::Pram,
            C::MonotonicReads,
            C::ReadYourWrites,
            C::EventuallyConsistent,
        ] {
            m.push(ConsistencyRelation {
                from: C::SequentiallyConsistent,
                to: target,
                kind: R::Composed,
            });
        }
        // Serializable → below Causal
        for target in [
            C::Pram,
            C::MonotonicReads,
            C::ReadYourWrites,
            C::EventuallyConsistent,
        ] {
            m.push(ConsistencyRelation {
                from: C::Serializable,
                to: target,
                kind: R::Composed,
            });
        }
        // Causal → Eventual
        m.push(ConsistencyRelation {
            from: C::Causal,
            to: C::EventuallyConsistent,
            kind: R::Composed,
        });

        for c in ConsistencyModel::variants() {
            m.push(ConsistencyRelation {
                from: c,
                to: c,
                kind: R::Composed,
            });
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<ConsistencyCategory>().unwrap();
    }

    #[test]
    fn has_eight_models() {
        assert_eq!(ConsistencyModel::variants().len(), 8);
    }

    // --- Herlihy & Wing (1990): Linearizable is the strongest ---

    #[test]
    fn linearizable_weakens_to_sequential() {
        let m = ConsistencyCategory::morphisms();
        assert!(m.iter().any(|r| r.from == ConsistencyModel::Linearizable
            && r.to == ConsistencyModel::SequentiallyConsistent
            && r.kind == ConsistencyRelationKind::Weakens));
    }

    // --- Lamport (1979): Sequential → Causal ---

    #[test]
    fn sequential_weakens_to_causal() {
        let m = ConsistencyCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == ConsistencyModel::SequentiallyConsistent
                    && r.to == ConsistencyModel::Causal
                    && r.kind == ConsistencyRelationKind::Weakens)
        );
    }

    // --- Viotti & Vukolic (2016): Linearizable and Serializable are INCOMPARABLE ---

    #[test]
    fn linearizable_and_serializable_incomparable() {
        let m = ConsistencyCategory::morphisms();
        // No direct weakening from Linearizable → Serializable
        assert!(!m.iter().any(|r| r.from == ConsistencyModel::Linearizable
            && r.to == ConsistencyModel::Serializable
            && r.kind == ConsistencyRelationKind::Weakens));
        // No direct weakening from Serializable → Linearizable
        assert!(!m.iter().any(|r| r.from == ConsistencyModel::Serializable
            && r.to == ConsistencyModel::Linearizable
            && r.kind == ConsistencyRelationKind::Weakens));
    }

    // --- Viotti & Vukolic: Eventual is the weakest (terminal in lattice) ---

    #[test]
    fn eventual_is_weakest() {
        let m = ConsistencyCategory::morphisms();
        // Every model has a path to Eventual
        for model in ConsistencyModel::variants() {
            if model == ConsistencyModel::EventuallyConsistent {
                continue;
            }
            assert!(
                m.iter()
                    .any(|r| r.from == model && r.to == ConsistencyModel::EventuallyConsistent),
                "{model:?} should reach EventuallyConsistent"
            );
        }
    }

    // --- Terry et al. (1994): Session guarantees weaken to Eventual ---

    #[test]
    fn session_guarantees_weaken_to_eventual() {
        let m = ConsistencyCategory::morphisms();
        for session in [
            ConsistencyModel::MonotonicReads,
            ConsistencyModel::ReadYourWrites,
        ] {
            assert!(m.iter().any(|r| r.from == session
                && r.to == ConsistencyModel::EventuallyConsistent
                && r.kind == ConsistencyRelationKind::Weakens));
        }
    }

    // --- Causal implies all session guarantees (Ahamad et al. 1995) ---

    #[test]
    fn causal_implies_session_guarantees() {
        let m = ConsistencyCategory::morphisms();
        assert!(m.iter().any(|r| r.from == ConsistencyModel::Causal
            && r.to == ConsistencyModel::MonotonicReads
            && r.kind == ConsistencyRelationKind::Weakens));
        assert!(m.iter().any(|r| r.from == ConsistencyModel::Causal
            && r.to == ConsistencyModel::ReadYourWrites
            && r.kind == ConsistencyRelationKind::Weakens));
    }

    // --- Lattice: transitivity holds ---

    #[test]
    fn linearizable_reaches_eventual_transitively() {
        let m = ConsistencyCategory::morphisms();
        assert!(m.iter().any(|r| r.from == ConsistencyModel::Linearizable
            && r.to == ConsistencyModel::EventuallyConsistent));
    }
}

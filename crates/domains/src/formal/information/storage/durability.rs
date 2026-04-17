//! Durability ontology — data lifecycle and persistence guarantees.
//!
//! Durability classifies how long data survives and what failures it
//! withstands. Forms a total order (chain): each level subsumes all
//! weaker guarantees.
//!
//! Also models the Haerder & Reuter (1983) buffer management taxonomy:
//! Force/No-Force × Steal/No-Steal, which determines what recovery
//! mechanisms are needed (redo, undo, both, neither).
//!
//! References:
//! - Haerder & Reuter, "Principles of Transaction-Oriented Database Recovery"
//!   (1983, ACM Computing Surveys)
//! - Mohan et al., "ARIES" (1992, ACM TODS)
//! - Pillai et al., "All File Systems Are Not Created Equal on Crash
//!   Consistency" (2014, OSDI)
//! - Rosenblum & Ousterhout, "Log-Structured File System" (1991, SOSP)
//! - SNIA, "Information Lifecycle Management" (2004)

pr4xis::ontology! {
    name: "Durability",
    source: "Haerder & Reuter (1983); Pillai et al. (2014); Pelley et al. (2014)",
    being: AbstractObject,

    concepts: [Ephemeral, Transient, Persistent, Durable, Replicated, Archived],

    labels: {
        Ephemeral: ("en", "Ephemeral", "Data exists only for the duration of a computation. CPU registers, stack frames. Lost on function return."),
        Transient: ("en", "Transient", "Data exists for the lifetime of the process. Heap-allocated, in-memory caches. Lost on process exit."),
        Persistent: ("en", "Persistent", "Data survives process exit but not system crash. Written to filesystem but not fsync'd. Pillai et al. (2014)."),
        Durable: ("en", "Durable", "Data survives system crash — committed to stable storage. The D in ACID (Haerder & Reuter 1983). Requires fsync/FUA."),
        Replicated: ("en", "Replicated", "Data survives node failure — replicated across failure domains. Requires distributed consensus."),
        Archived: ("en", "Archived", "Data survives organizational change — immutable, long-term retention. SNIA ILM (2004): cold/frozen tier."),
    },

    edges: [
        (Ephemeral, Transient, Strengthens),
        (Transient, Persistent, Strengthens),
        (Persistent, Durable, Strengthens),
        (Durable, Replicated, Strengthens),
        (Replicated, Archived, Strengthens),
    ],
}

/// Buffer management strategy — Haerder & Reuter (1983) taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferPolicy {
    /// Dirty pages MUST be flushed at commit time.
    Force,
    /// Dirty pages may remain in buffer after commit.
    NoForce,
    /// Uncommitted pages may be flushed to disk.
    Steal,
    /// Uncommitted pages may NOT be flushed to disk.
    NoSteal,
}

/// Crash consistency strategy — how data structures survive crashes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrashConsistency {
    /// No crash protection.
    None,
    /// Write-ahead logging (Mohan ARIES 1992).
    WriteAheadLog,
    /// Copy-on-write (ZFS, Btrfs).
    CopyOnWrite,
    /// Sequential log (Rosenblum & Ousterhout 1991).
    LogStructured,
    /// Journal, then apply (ext3/ext4).
    Journaling,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<DurabilityCategory>().unwrap();
    }

    #[test]
    fn has_six_levels() {
        assert_eq!(DurabilityConcept::variants().len(), 6);
    }

    #[test]
    fn durable_is_stronger_than_persistent() {
        let m = DurabilityCategory::morphisms();
        assert!(m.iter().any(|r| r.from == DurabilityConcept::Persistent
            && r.to == DurabilityConcept::Durable
            && r.kind == DurabilityRelationKind::Strengthens));
    }

    #[test]
    fn chain_is_total_order() {
        let m = DurabilityCategory::morphisms();
        let chain = [
            DurabilityConcept::Ephemeral,
            DurabilityConcept::Transient,
            DurabilityConcept::Persistent,
            DurabilityConcept::Durable,
            DurabilityConcept::Replicated,
            DurabilityConcept::Archived,
        ];
        for i in 0..chain.len() - 1 {
            assert!(
                m.iter().any(|r| r.from == chain[i]
                    && r.to == chain[i + 1]
                    && r.kind == DurabilityRelationKind::Strengthens),
                "{:?} should strengthen to {:?}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    #[test]
    fn ephemeral_reaches_archived() {
        let m = DurabilityCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == DurabilityConcept::Ephemeral
                    && r.to == DurabilityConcept::Archived)
        );
    }

    #[test]
    fn persistent_is_not_durable() {
        assert_ne!(DurabilityConcept::Persistent, DurabilityConcept::Durable);
        let m = DurabilityCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == DurabilityConcept::Persistent
                    && r.to == DurabilityConcept::Durable)
        );
        assert!(!m.iter().any(|r| r.from == DurabilityConcept::Durable
            && r.to == DurabilityConcept::Persistent
            && r.kind == DurabilityRelationKind::Strengthens));
    }

    #[test]
    fn buffer_policies_exist() {
        let _force = BufferPolicy::Force;
        let _no_force = BufferPolicy::NoForce;
        let _steal = BufferPolicy::Steal;
        let _no_steal = BufferPolicy::NoSteal;
    }

    #[test]
    fn steal_no_force_needs_aries() {
        assert_ne!(BufferPolicy::Steal, BufferPolicy::NoForce);
    }

    #[test]
    fn crash_consistency_strategies_exist() {
        let strategies = [
            CrashConsistency::None,
            CrashConsistency::WriteAheadLog,
            CrashConsistency::CopyOnWrite,
            CrashConsistency::LogStructured,
            CrashConsistency::Journaling,
        ];
        assert_eq!(strategies.len(), 5);
    }

    #[test]
    fn archived_is_strongest() {
        let m = DurabilityCategory::morphisms();
        assert!(!m.iter().any(|r| r.from == DurabilityConcept::Archived
            && r.kind == DurabilityRelationKind::Strengthens));
    }

    #[test]
    fn ephemeral_is_weakest() {
        let m = DurabilityCategory::morphisms();
        assert!(!m.iter().any(|r| r.to == DurabilityConcept::Ephemeral
            && r.kind == DurabilityRelationKind::Strengthens));
    }
}

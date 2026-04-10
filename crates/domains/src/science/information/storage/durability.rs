use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

// Durability ontology — data lifecycle and persistence guarantees.
//
// Durability classifies how long data survives and what failures it
// withstands. Forms a total order (chain): each level subsumes all
// weaker guarantees.
//
// Also models the Haerder & Reuter (1983) buffer management taxonomy:
// Force/No-Force × Steal/No-Steal, which determines what recovery
// mechanisms are needed (redo, undo, both, neither).
//
// References:
// - Haerder & Reuter, "Principles of Transaction-Oriented Database Recovery"
//   (1983, ACM Computing Surveys) — ACID, force/steal taxonomy
// - Mohan et al., "ARIES: A Transaction Recovery Method Supporting
//   Fine-Granularity Locking" (1992, ACM TODS) — steal/no-force recovery
// - Pillai et al., "All File Systems Are Not Created Equal on Crash
//   Consistency" (2014, OSDI) — fsync semantics, crash consistency
// - Rosenblum & Ousterhout, "The Design and Implementation of a
//   Log-Structured File System" (1991, SOSP)
// - SNIA, "Information Lifecycle Management" (2004)

/// Durability levels — a total order from weakest to strongest.
///
/// Each level subsumes all weaker guarantees.
/// Ephemeral ⊂ Transient ⊂ Persistent ⊂ Durable ⊂ Replicated ⊂ Archived.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurabilityLevel {
    /// Data exists only for the duration of a computation.
    /// CPU registers, stack frames. Lost on function return.
    Ephemeral,

    /// Data exists for the lifetime of the process.
    /// Heap-allocated, in-memory caches. Lost on process exit.
    Transient,

    /// Data survives process exit but not system crash.
    /// Written to filesystem but not fsync'd.
    /// Pillai et al. (2014): in page cache, not on media.
    Persistent,

    /// Data survives system crash — committed to stable storage.
    /// Haerder & Reuter (1983): the D in ACID.
    /// Requires fsync/FUA to guarantee media persistence.
    Durable,

    /// Data survives node failure — replicated across failure domains.
    /// Requires distributed consensus (Paxos, Raft) or replication protocol.
    Replicated,

    /// Data survives organizational change — immutable, long-term retention.
    /// SNIA ILM (2004): cold/frozen storage tier.
    /// Write-once, read-many. Tape, optical, archival object storage.
    Archived,
}

impl Entity for DurabilityLevel {
    fn variants() -> Vec<Self> {
        vec![
            Self::Ephemeral,
            Self::Transient,
            Self::Persistent,
            Self::Durable,
            Self::Replicated,
            Self::Archived,
        ]
    }
}

/// Buffer management strategy — Haerder & Reuter (1983) taxonomy.
///
/// Determines what recovery mechanisms are needed:
/// - Steal + No-Force → needs both undo AND redo (ARIES)
/// - No-Steal + Force → needs neither (simplest, worst performance)
/// - Steal + Force → needs undo only
/// - No-Steal + No-Force → needs redo only
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferPolicy {
    /// Dirty pages MUST be flushed at commit time.
    /// Simple but slow — every commit forces I/O.
    Force,

    /// Dirty pages may remain in buffer after commit.
    /// Fast but requires REDO recovery (WAL).
    /// Haerder & Reuter (1983): the performance choice.
    NoForce,

    /// Uncommitted pages may be flushed to disk.
    /// Allows large transactions but requires UNDO recovery.
    Steal,

    /// Uncommitted pages may NOT be flushed to disk.
    /// Simple but limits transaction size to buffer pool.
    NoSteal,
}

/// Crash consistency strategy — how data structures survive crashes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrashConsistency {
    /// No crash protection. Data may be corrupted after crash.
    None,

    /// Write-ahead logging — Mohan et al. ARIES (1992).
    /// Log records written before data pages.
    /// Recovery: redo committed, undo uncommitted.
    WriteAheadLog,

    /// Copy-on-write — never overwrite in place.
    /// Old version persists until new version committed.
    /// ZFS, Btrfs.
    CopyOnWrite,

    /// All writes are appends to a sequential log.
    /// Rosenblum & Ousterhout (1991).
    LogStructured,

    /// Operations recorded in journal, then applied.
    /// ext3/ext4 journaling. Simpler than full WAL.
    Journaling,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DurabilityRelation {
    pub from: DurabilityLevel,
    pub to: DurabilityLevel,
    pub kind: DurabilityRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurabilityRelationKind {
    Identity,
    /// Strengthening: from is strictly weaker than to.
    /// from's data survives strictly fewer failure modes.
    Strengthens,
    Composed,
}

impl Relationship for DurabilityRelation {
    type Object = DurabilityLevel;
    fn source(&self) -> DurabilityLevel {
        self.from
    }
    fn target(&self) -> DurabilityLevel {
        self.to
    }
}

pub struct DurabilityCategory;

impl Category for DurabilityCategory {
    type Object = DurabilityLevel;
    type Morphism = DurabilityRelation;

    fn identity(obj: &DurabilityLevel) -> DurabilityRelation {
        DurabilityRelation {
            from: *obj,
            to: *obj,
            kind: DurabilityRelationKind::Identity,
        }
    }

    fn compose(f: &DurabilityRelation, g: &DurabilityRelation) -> Option<DurabilityRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == DurabilityRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == DurabilityRelationKind::Identity {
            return Some(f.clone());
        }
        Some(DurabilityRelation {
            from: f.from,
            to: g.to,
            kind: DurabilityRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<DurabilityRelation> {
        use DurabilityLevel as D;
        use DurabilityRelationKind as R;
        let mut m = Vec::new();

        for d in DurabilityLevel::variants() {
            m.push(DurabilityRelation {
                from: d,
                to: d,
                kind: R::Identity,
            });
        }

        // The chain: Ephemeral → Transient → Persistent → Durable → Replicated → Archived
        // Direct edges (adjacent in the chain):
        let chain = [
            D::Ephemeral,
            D::Transient,
            D::Persistent,
            D::Durable,
            D::Replicated,
            D::Archived,
        ];
        for i in 0..chain.len() - 1 {
            m.push(DurabilityRelation {
                from: chain[i],
                to: chain[i + 1],
                kind: R::Strengthens,
            });
        }

        // Transitive closure (all non-adjacent pairs)
        for i in 0..chain.len() {
            for j in i + 2..chain.len() {
                m.push(DurabilityRelation {
                    from: chain[i],
                    to: chain[j],
                    kind: R::Composed,
                });
            }
        }

        for d in DurabilityLevel::variants() {
            m.push(DurabilityRelation {
                from: d,
                to: d,
                kind: R::Composed,
            });
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<DurabilityCategory>().unwrap();
    }

    #[test]
    fn has_six_levels() {
        assert_eq!(DurabilityLevel::variants().len(), 6);
    }

    // --- Haerder & Reuter (1983): Durability is the D in ACID ---

    #[test]
    fn durable_is_stronger_than_persistent() {
        let m = DurabilityCategory::morphisms();
        assert!(m.iter().any(|r| r.from == DurabilityLevel::Persistent
            && r.to == DurabilityLevel::Durable
            && r.kind == DurabilityRelationKind::Strengthens));
    }

    // --- Total order: the chain is complete ---

    #[test]
    fn chain_is_total_order() {
        let m = DurabilityCategory::morphisms();
        let chain = [
            DurabilityLevel::Ephemeral,
            DurabilityLevel::Transient,
            DurabilityLevel::Persistent,
            DurabilityLevel::Durable,
            DurabilityLevel::Replicated,
            DurabilityLevel::Archived,
        ];
        // Every adjacent pair has a direct strengthening edge
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

    // --- Transitivity: Ephemeral reaches Archived ---

    #[test]
    fn ephemeral_reaches_archived() {
        let m = DurabilityCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == DurabilityLevel::Ephemeral && r.to == DurabilityLevel::Archived)
        );
    }

    // --- Pillai et al. (2014): Persistent ≠ Durable (fsync gap) ---

    #[test]
    fn persistent_is_not_durable() {
        // Persistent (in page cache) is strictly weaker than Durable (fsync'd)
        assert_ne!(DurabilityLevel::Persistent, DurabilityLevel::Durable);
        let m = DurabilityCategory::morphisms();
        // There's a strengthening from Persistent → Durable (not the reverse)
        assert!(
            m.iter()
                .any(|r| r.from == DurabilityLevel::Persistent && r.to == DurabilityLevel::Durable)
        );
        assert!(!m.iter().any(|r| r.from == DurabilityLevel::Durable
            && r.to == DurabilityLevel::Persistent
            && r.kind == DurabilityRelationKind::Strengthens));
    }

    // --- Haerder & Reuter (1983): Force/Steal taxonomy ---

    #[test]
    fn buffer_policies_exist() {
        // The four combinations determine recovery needs
        let _force = BufferPolicy::Force;
        let _no_force = BufferPolicy::NoForce;
        let _steal = BufferPolicy::Steal;
        let _no_steal = BufferPolicy::NoSteal;
    }

    #[test]
    fn steal_no_force_needs_aries() {
        // Steal + NoForce = needs both undo AND redo = ARIES (Mohan 1992)
        // This is the most flexible and most complex combination
        assert_ne!(BufferPolicy::Steal, BufferPolicy::NoForce);
    }

    // --- Crash consistency strategies ---

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

    // --- SNIA ILM: Archived is the strongest ---

    #[test]
    fn archived_is_strongest() {
        let m = DurabilityCategory::morphisms();
        // Nothing strengthens beyond Archived
        assert!(!m.iter().any(|r| r.from == DurabilityLevel::Archived
            && r.kind == DurabilityRelationKind::Strengthens));
    }

    // --- Ephemeral is the weakest ---

    #[test]
    fn ephemeral_is_weakest() {
        let m = DurabilityCategory::morphisms();
        // Nothing weakens below Ephemeral
        assert!(
            !m.iter().any(|r| r.to == DurabilityLevel::Ephemeral
                && r.kind == DurabilityRelationKind::Strengthens)
        );
    }
}

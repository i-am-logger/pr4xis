use pr4xis::category::Entity;
use pr4xis::define_category;

// Volatility ontology — storage media hierarchy and persistence domains.
//
// Models the physical storage hierarchy from CPU registers to tape,
// classified by volatility (loses data on power loss?) and access latency.
// The hierarchy forms a preorder: "can serve as backing store for."
//
// References:
// - SNIA, "NVM Programming Model v1.2" (2017) — persistent memory taxonomy
// - IEEE Std 1005 — volatile/non-volatile memory classification
// - Pelley et al., "Memory Persistency" (2014, ISCA) — persistence domains,
//   separating volatile consistency from persistent ordering
// - Intel Software Developer Manual — CLFLUSH, CLWB, SFENCE instructions

/// Storage media types — objects in the volatility hierarchy.
///
/// Ordered by latency (fastest first) and partitioned into
/// volatile (loses data on power loss) and non-volatile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum StorageMedia {
    /// CPU register — ~0.3ns, volatile.
    /// Fastest, smallest, most ephemeral.
    Register,

    /// CPU cache (L1/L2/L3) — ~1-10ns, volatile.
    /// Hardware-managed, transparent to software.
    Cache,

    /// Dynamic RAM — ~100ns, volatile.
    /// Main system memory. Lost on power failure.
    Dram,

    /// Persistent memory — ~300ns, non-volatile.
    /// Byte-addressable via load/store on the memory bus.
    /// SNIA NVM Model (2017). Intel Optane (defunct), CXL.
    /// The boundary between volatile and non-volatile.
    PersistentMemory,

    /// Flash / NVMe SSD — ~10us, non-volatile.
    /// Block-addressable. Write endurance limited.
    Flash,

    /// Hard disk drive — ~10ms, non-volatile.
    /// Spinning platters. Sequential access fast, random slow.
    Disk,

    /// Magnetic tape — seconds to minutes, non-volatile.
    /// Sequential only. Highest density, lowest cost per byte.
    /// Archival / cold storage.
    Tape,
}

/// Is this storage media volatile?
///
/// IEEE Std 1005: volatile = loses contents when power removed.
/// Pelley et al. (2014): the persistence domain boundary.
impl StorageMedia {
    pub fn is_volatile(&self) -> bool {
        matches!(self, Self::Register | Self::Cache | Self::Dram)
    }

    pub fn is_non_volatile(&self) -> bool {
        !self.is_volatile()
    }
}

define_category! {
    pub VolatilityCategory {
        entity: StorageMedia,
        relation: VolatilityRelation,
        kind: VolatilityRelationKind,
        kinds: [
            /// from is faster (lower latency) than to — can be cached by to.
            /// This is "can serve as cache for" in the hierarchy.
            FasterThan,
            /// from is in the same volatility class as to.
            SameVolatility,
        ],
        edges: [
            // The latency hierarchy (direct edges):
            // Register → Cache → DRAM → PersistentMemory → Flash → Disk → Tape
            (Register, Cache, FasterThan),
            (Cache, Dram, FasterThan),
            (Dram, PersistentMemory, FasterThan),
            (PersistentMemory, Flash, FasterThan),
            (Flash, Disk, FasterThan),
            (Disk, Tape, FasterThan),
        ],
        composed: [
            // Transitive closure
            (Register, Dram),
            (Register, PersistentMemory),
            (Register, Flash),
            (Register, Disk),
            (Register, Tape),
            (Cache, PersistentMemory),
            (Cache, Flash),
            (Cache, Disk),
            (Cache, Tape),
            (Dram, Flash),
            (Dram, Disk),
            (Dram, Tape),
            (PersistentMemory, Disk),
            (PersistentMemory, Tape),
            (Flash, Tape),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<VolatilityCategory>().unwrap();
    }

    #[test]
    fn has_seven_media_types() {
        assert_eq!(StorageMedia::variants().len(), 7);
    }

    // --- IEEE Std 1005: volatile/non-volatile partition ---

    #[test]
    fn volatile_partition() {
        assert!(StorageMedia::Register.is_volatile());
        assert!(StorageMedia::Cache.is_volatile());
        assert!(StorageMedia::Dram.is_volatile());
        assert!(!StorageMedia::PersistentMemory.is_volatile());
        assert!(!StorageMedia::Flash.is_volatile());
        assert!(!StorageMedia::Disk.is_volatile());
        assert!(!StorageMedia::Tape.is_volatile());
    }

    #[test]
    fn volatile_non_volatile_exhaustive() {
        // Every media type is exactly one of volatile or non-volatile
        for media in StorageMedia::variants() {
            assert_ne!(
                media.is_volatile(),
                media.is_non_volatile(),
                "{media:?} must be exactly volatile XOR non-volatile"
            );
        }
    }

    // --- Pelley et al. (2014): PersistentMemory is the boundary ---

    #[test]
    fn persistent_memory_is_boundary() {
        // PM is the first non-volatile tier
        assert!(StorageMedia::Dram.is_volatile());
        assert!(StorageMedia::PersistentMemory.is_non_volatile());
    }

    // --- SNIA NVM Model: latency hierarchy ---

    #[test]
    fn register_is_fastest() {
        let m = VolatilityCategory::morphisms();
        // Register is faster than everything
        for media in StorageMedia::variants() {
            if media == StorageMedia::Register {
                continue;
            }
            assert!(
                m.iter()
                    .any(|r| r.from == StorageMedia::Register && r.to == media),
                "Register should be faster than {media:?}"
            );
        }
    }

    #[test]
    fn tape_is_slowest() {
        let m = VolatilityCategory::morphisms();
        // Nothing is slower than tape
        assert!(
            !m.iter()
                .any(|r| r.from == StorageMedia::Tape
                    && r.kind == VolatilityRelationKind::FasterThan)
        );
    }

    #[test]
    fn hierarchy_is_total_order() {
        let m = VolatilityCategory::morphisms();
        let hierarchy = [
            StorageMedia::Register,
            StorageMedia::Cache,
            StorageMedia::Dram,
            StorageMedia::PersistentMemory,
            StorageMedia::Flash,
            StorageMedia::Disk,
            StorageMedia::Tape,
        ];
        for i in 0..hierarchy.len() - 1 {
            assert!(
                m.iter().any(|r| r.from == hierarchy[i]
                    && r.to == hierarchy[i + 1]
                    && r.kind == VolatilityRelationKind::FasterThan),
                "{:?} should be faster than {:?}",
                hierarchy[i],
                hierarchy[i + 1]
            );
        }
    }

    // --- Three volatile, four non-volatile ---

    #[test]
    fn three_volatile_four_non_volatile() {
        let volatile_count = StorageMedia::variants()
            .iter()
            .filter(|m| m.is_volatile())
            .count();
        let non_volatile_count = StorageMedia::variants()
            .iter()
            .filter(|m| m.is_non_volatile())
            .count();
        assert_eq!(volatile_count, 3);
        assert_eq!(non_volatile_count, 4);
    }
}

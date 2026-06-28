//! Volatility — storage-media latency hierarchy and persistence domains.
//!
//! Models the physical storage hierarchy from CPU registers down to
//! magnetic tape, partitioned by volatility (loses data on power loss?)
//! and ordered by access latency. The "faster-than" relation forms a
//! total order.
//!
//! # Literature
//!
//! - **SNIA (2017)** *NVM Programming Model v1.2*, Storage Networking
//!   Industry Association — persistent-memory taxonomy (NVM.PM.FILE,
//!   DAX mode).
//! - **IEEE Std 1005** — volatile vs non-volatile memory
//!   classification.
//! - **Pelley, Chen & Wenisch (2014)** "Memory Persistency",
//!   *ISCA 2014* — persistence domains; separating volatile
//!   consistency from persistent ordering.
//! - **Intel Software Developer Manual** — CLFLUSH, CLWB, SFENCE
//!   instructions for persistent-memory ordering.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Volatility",
    source: "SNIA (2017) NVM Programming Model v1.2; IEEE Std 1005 (volatile/non-volatile memory); Pelley, Chen & Wenisch (2014) Memory Persistency, ISCA; Intel Software Developer Manual (CLFLUSH/CLWB/SFENCE)",

    concepts: [
        Register,
        Cache,
        Dram,
        PersistentMemory,
        Flash,
        Disk,
        Tape,
    ],

    labels: {
        Register: ("en", "Register",
            "CPU register - ~0.3ns, volatile (IEEE Std 1005). Fastest, smallest, most ephemeral."),
        Cache: ("en", "Cache",
            "CPU cache (L1/L2/L3) - ~1-10ns, volatile. Hardware-managed."),
        Dram: ("en", "DRAM",
            "Dynamic RAM - ~100ns, volatile. Main system memory; lost on power failure."),
        PersistentMemory: ("en", "Persistent memory",
            "SNIA (2017) NVM Programming Model: ~300ns, non-volatile. Byte-addressable via load/store on the memory bus. Pelley et al. (2014) boundary between volatile and non-volatile."),
        Flash: ("en", "Flash",
            "Flash / NVMe SSD - ~10us, non-volatile. Block-addressable; write endurance limited."),
        Disk: ("en", "Disk",
            "Hard disk drive - ~10ms, non-volatile. Sequential access fast, random slow."),
        Tape: ("en", "Tape",
            "Magnetic tape - seconds to minutes, non-volatile. Sequential only; highest density, lowest cost per byte; archival."),
    },

    edges: [
        // Pelley et al. (2014) / SNIA latency hierarchy.
        (Register, Cache, FasterThan),
        (Cache, Dram, FasterThan),
        (Dram, PersistentMemory, FasterThan),
        (PersistentMemory, Flash, FasterThan),
        (Flash, Disk, FasterThan),
        (Disk, Tape, FasterThan),
    ],
}

/// Legacy alias — earlier code called the concept enum `StorageMedia`.
pub type StorageMedia = VolatilityConcept;

impl VolatilityConcept {
    /// IEEE Std 1005: volatile = loses contents when power removed.
    /// Pelley et al. (2014): the persistence-domain boundary.
    pub fn is_volatile(&self) -> bool {
        matches!(self, Self::Register | Self::Cache | Self::Dram)
    }

    pub fn is_non_volatile(&self) -> bool {
        !self.is_volatile()
    }
}

/// Quality: is this medium volatile? IEEE Std 1005 binary partition.
#[derive(Debug, Clone)]
pub struct IsVolatile;

impl Quality for IsVolatile {
    type Individual = VolatilityConcept;
    type Value = bool;

    fn get(&self, c: &VolatilityConcept) -> Option<bool> {
        Some(c.is_volatile())
    }
}

impl Ontology for VolatilityOntology {
    type Cat = VolatilityCategory;
    type Qual = IsVolatile;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<VolatilityCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        VolatilityOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn seven_media_types() {
        assert_eq!(VolatilityConcept::variants().len(), 7);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn volatile_partition() {
        // IEEE Std 1005.
        assert!(VolatilityConcept::Register.is_volatile());
        assert!(VolatilityConcept::Cache.is_volatile());
        assert!(VolatilityConcept::Dram.is_volatile());
        assert!(!VolatilityConcept::PersistentMemory.is_volatile());
        assert!(!VolatilityConcept::Flash.is_volatile());
        assert!(!VolatilityConcept::Disk.is_volatile());
        assert!(!VolatilityConcept::Tape.is_volatile());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn volatile_non_volatile_exhaustive() {
        for media in VolatilityConcept::variants() {
            assert_ne!(media.is_volatile(), media.is_non_volatile());
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn persistent_memory_is_boundary() {
        // Pelley et al. (2014).
        assert!(VolatilityConcept::Dram.is_volatile());
        assert!(VolatilityConcept::PersistentMemory.is_non_volatile());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hierarchy_direct_edges() {
        let m = VolatilityCategory::morphisms();
        let hierarchy = [
            VolatilityConcept::Register,
            VolatilityConcept::Cache,
            VolatilityConcept::Dram,
            VolatilityConcept::PersistentMemory,
            VolatilityConcept::Flash,
            VolatilityConcept::Disk,
            VolatilityConcept::Tape,
        ];
        for i in 0..hierarchy.len() - 1 {
            assert!(m.iter().any(|r| r.source() == hierarchy[i]
                && r.target() == hierarchy[i + 1]
                && r.kind() == VolatilityRelationKind::FasterThan));
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn three_volatile_four_non_volatile() {
        let volatile_count = VolatilityConcept::variants()
            .iter()
            .filter(|m| m.is_volatile())
            .count();
        assert_eq!(volatile_count, 3);
        assert_eq!(VolatilityConcept::variants().len() - volatile_count, 4);
    }

    fn arb_concept() -> impl Strategy<Value = VolatilityConcept> {
        proptest::sample::select(VolatilityConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in VolatilityCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in VolatilityOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_volatility_total(c in arb_concept()) {
            prop_assert!(IsVolatile.get(&c).is_some());
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_volatility_total, Verifiable);
}

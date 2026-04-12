use pr4xis::category::Entity;
use pr4xis::ontology::Quality;

/// Fusion architecture taxonomy.
///
/// Classifies the structural pattern by which multiple sensors'
/// measurements are combined. The choice of architecture determines
/// bandwidth, latency, fault tolerance, and optimality.
///
/// Source: Liggins et al. (2008), *Handbook of Multisensor Data Fusion*, Chapter 2.
///         Castanedo (2013), "A Review of Data Fusion Techniques."
///         Khaleghi et al. (2013), "Multisensor data fusion: A review."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FusionArchitecture {
    /// All raw measurements sent to a single central filter.
    /// Optimal (if filter is correct), but high bandwidth and single point of failure.
    Centralized,
    /// Each sensor runs its own filter; results fused at a higher level.
    /// Fault-tolerant, low bandwidth, but suboptimal (correlation lost).
    Distributed,
    /// Like distributed, but with feedback from the master to local filters.
    /// Attempts to recover optimality while keeping fault tolerance.
    /// Source: Carlson (1990), "Federated square root filter."
    Federated,
    /// Multi-level fusion: local filters feed into regional filters,
    /// which feed into a global filter. Common in military C4ISR.
    Hierarchical,
    /// Sequential processing: output of one filter feeds the next.
    /// Example: INS/GNSS loosely-coupled integration.
    Cascaded,
}

impl Entity for FusionArchitecture {
    fn variants() -> Vec<Self> {
        vec![
            Self::Centralized,
            Self::Distributed,
            Self::Federated,
            Self::Hierarchical,
            Self::Cascaded,
        ]
    }
}

/// Quality: description of each architecture's key property.
#[derive(Debug, Clone)]
pub struct ArchitectureDescription;

impl Quality for ArchitectureDescription {
    type Individual = FusionArchitecture;
    type Value = &'static str;

    fn get(&self, arch: &FusionArchitecture) -> Option<&'static str> {
        Some(match arch {
            FusionArchitecture::Centralized => {
                "optimal but fragile: all data to one filter, single point of failure"
            }
            FusionArchitecture::Distributed => {
                "fault-tolerant but suboptimal: independent local filters, results merged"
            }
            FusionArchitecture::Federated => {
                "compromise: local filters with master feedback, near-optimal with redundancy"
            }
            FusionArchitecture::Hierarchical => {
                "multi-level: local -> regional -> global, scalable for large sensor networks"
            }
            FusionArchitecture::Cascaded => {
                "sequential: output of one filter feeds next, simple but error propagation risk"
            }
        })
    }
}

/// Quality: whether the architecture preserves cross-sensor correlations.
#[derive(Debug, Clone)]
pub struct PreservesCorrelations;

impl Quality for PreservesCorrelations {
    type Individual = FusionArchitecture;
    type Value = bool;

    fn get(&self, arch: &FusionArchitecture) -> Option<bool> {
        Some(match arch {
            FusionArchitecture::Centralized => true, // full correlation maintained
            FusionArchitecture::Distributed => false, // correlations lost
            FusionArchitecture::Federated => true,   // recovered via feedback
            FusionArchitecture::Hierarchical => false, // partially lost at each level
            FusionArchitecture::Cascaded => false,   // sequential ignores cross-correlation
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_architectures_have_descriptions() {
        let desc = ArchitectureDescription;
        for arch in FusionArchitecture::variants() {
            assert!(desc.get(&arch).is_some());
        }
    }

    #[test]
    fn centralized_preserves_correlations() {
        let q = PreservesCorrelations;
        assert_eq!(q.get(&FusionArchitecture::Centralized), Some(true));
    }

    #[test]
    fn distributed_loses_correlations() {
        let q = PreservesCorrelations;
        assert_eq!(q.get(&FusionArchitecture::Distributed), Some(false));
    }

    #[test]
    fn five_architecture_variants() {
        assert_eq!(FusionArchitecture::variants().len(), 5);
    }
}

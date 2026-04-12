use pr4xis::category::Entity;
use pr4xis::ontology::Quality;

/// Filter composition strategies for combining estimates from multiple filters.
///
/// When multiple local filters produce independent estimates of the same state,
/// these strategies determine how to fuse them into a single best estimate.
///
/// Source: Julier & Uhlmann (1997), "A Non-divergent Estimation Algorithm in the
///         Presence of Unknown Correlations."
///         Mutambara (1998), *Decentralized Estimation and Control for
///         Multisensor Systems*, Chapter 4.
///         Carlson (1990), "Federated square root filter for decentralized
///         parallel processes."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompositionStrategy {
    /// Covariance intersection (CI): fuse two estimates without knowing
    /// their cross-correlation. Conservative but guaranteed consistent.
    ///
    /// P_fused^{-1} = omega * P1^{-1} + (1-omega) * P2^{-1}
    ///
    /// where omega in [0,1] is optimized to minimize trace(P_fused).
    ///
    /// Source: Julier & Uhlmann (1997).
    CovarianceIntersection,
    /// Information fusion: sum information matrices and information vectors.
    ///
    /// I_fused = I_1 + I_2, i_fused = i_1 + i_2
    /// where I = P^{-1} (information matrix), i = P^{-1} * x (information vector)
    ///
    /// Optimal when estimates are INDEPENDENT (zero cross-correlation).
    /// Over-confident if correlations exist.
    ///
    /// Source: Mutambara (1998), Chapter 3.
    InformationFusion,
    /// Federated fusion: divide information equally among local filters,
    /// then recombine at the master.
    ///
    /// Each local filter gets P_local = n * P_global (inflated covariance).
    /// Master fuses: P_fused^{-1} = sum(P_local_i^{-1}) - (n-1)*P_prior^{-1}
    ///
    /// Source: Carlson (1990).
    FederatedFusion,
}

impl Entity for CompositionStrategy {
    fn variants() -> Vec<Self> {
        vec![
            Self::CovarianceIntersection,
            Self::InformationFusion,
            Self::FederatedFusion,
        ]
    }
}

/// Quality: whether the strategy is consistent (never over-confident)
/// when cross-correlations are unknown.
#[derive(Debug, Clone)]
pub struct ConsistentUnderUnknownCorrelation;

impl Quality for ConsistentUnderUnknownCorrelation {
    type Individual = CompositionStrategy;
    type Value = bool;

    fn get(&self, strategy: &CompositionStrategy) -> Option<bool> {
        Some(match strategy {
            // CI is guaranteed consistent regardless of correlation
            CompositionStrategy::CovarianceIntersection => true,
            // Information fusion assumes independence — over-confident if not
            CompositionStrategy::InformationFusion => false,
            // Federated fusion handles correlations via the master
            CompositionStrategy::FederatedFusion => true,
        })
    }
}

/// Quality: description of each strategy's key tradeoff.
#[derive(Debug, Clone)]
pub struct CompositionDescription;

impl Quality for CompositionDescription {
    type Individual = CompositionStrategy;
    type Value = &'static str;

    fn get(&self, strategy: &CompositionStrategy) -> Option<&'static str> {
        Some(match strategy {
            CompositionStrategy::CovarianceIntersection => {
                "conservative fusion: consistent without knowing correlations, but suboptimal"
            }
            CompositionStrategy::InformationFusion => {
                "optimal when independent: additive information, but over-confident if correlated"
            }
            CompositionStrategy::FederatedFusion => {
                "federated: local filters with master recombination, balances optimality and fault tolerance"
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_strategies_have_descriptions() {
        let desc = CompositionDescription;
        for strategy in CompositionStrategy::variants() {
            assert!(desc.get(&strategy).is_some());
        }
    }

    #[test]
    fn ci_consistent_under_unknown_correlation() {
        let q = ConsistentUnderUnknownCorrelation;
        assert_eq!(
            q.get(&CompositionStrategy::CovarianceIntersection),
            Some(true)
        );
    }

    #[test]
    fn information_fusion_not_consistent_under_unknown_correlation() {
        let q = ConsistentUnderUnknownCorrelation;
        assert_eq!(q.get(&CompositionStrategy::InformationFusion), Some(false));
    }

    #[test]
    fn three_composition_variants() {
        assert_eq!(CompositionStrategy::variants().len(), 3);
    }
}

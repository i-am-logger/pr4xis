/// Covariance Intersection — decentralized fusion without cross-correlation.
///
/// When two estimates have unknown cross-correlation, standard fusion
/// (Kalman update) can be inconsistent. Covariance Intersection provides
/// a conservative but consistent fusion.
///
/// Proves:
/// - Fused uncertainty is less than either individual
/// - Gaussian fusion (information form) reduces variance
///
/// Source: Julier & Uhlmann (1997). "A non-divergent estimation algorithm
///         in the presence of unknown correlations."
#[cfg(test)]
mod tests {
    use pr4xis_domains::formal::math::probability::gaussian::Gaussian1D;

    #[test]
    fn gaussian_fusion_reduces_uncertainty() {
        let est1 = Gaussian1D::new(10.0, 4.0); // σ² = 4
        let est2 = Gaussian1D::new(11.0, 9.0); // σ² = 9

        let fused = est1.fuse(&est2);

        // Fused variance must be less than both
        assert!(fused.variance < est1.variance);
        assert!(fused.variance < est2.variance);

        // Fused mean should be between the two (weighted toward lower variance)
        assert!(fused.mean > est1.mean - 1.0);
        assert!(fused.mean < est2.mean + 1.0);
    }

    #[test]
    fn fusion_of_identical_estimates_halves_variance() {
        let est = Gaussian1D::new(5.0, 4.0);
        let fused = est.fuse(&est);

        // Fusing identical independent estimates: σ²_fused = σ²/2
        assert!((fused.variance - 2.0).abs() < 1e-10);
        assert!((fused.mean - 5.0).abs() < 1e-10);
    }

    #[test]
    fn very_precise_estimate_dominates() {
        let precise = Gaussian1D::new(10.0, 0.01); // very precise
        let noisy = Gaussian1D::new(20.0, 100.0); // very noisy

        let fused = precise.fuse(&noisy);

        // Fused mean should be very close to precise estimate
        assert!((fused.mean - 10.0).abs() < 0.5);
        assert!(fused.variance < precise.variance + 0.001);
    }

    mod proptest_proofs {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn fusion_always_reduces_variance(
                m1 in -100.0..100.0_f64, v1 in 0.01..100.0_f64,
                m2 in -100.0..100.0_f64, v2 in 0.01..100.0_f64,
            ) {
                let g1 = Gaussian1D::new(m1, v1);
                let g2 = Gaussian1D::new(m2, v2);
                let fused = g1.fuse(&g2);
                prop_assert!(fused.variance < v1.min(v2) + 1e-10,
                    "fused var={} must be < min({},{})", fused.variance, v1, v2);
            }

            #[test]
            fn fusion_is_deterministic(
                m1 in -50.0..50.0_f64, v1 in 0.1..10.0_f64,
                m2 in -50.0..50.0_f64, v2 in 0.1..10.0_f64,
            ) {
                let g1 = Gaussian1D::new(m1, v1);
                let g2 = Gaussian1D::new(m2, v2);
                let f1 = g1.fuse(&g2);
                let f2 = g1.fuse(&g2);
                prop_assert_eq!(f1.mean.to_bits(), f2.mean.to_bits());
                prop_assert_eq!(f1.variance.to_bits(), f2.variance.to_bits());
            }
        }
    }
}

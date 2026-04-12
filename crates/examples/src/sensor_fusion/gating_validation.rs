/// Mahalanobis Gating — validating measurements before fusion.
///
/// A measurement that is too far from the predicted state (in a
/// statistical sense) should be rejected. This prevents outliers
/// from corrupting the state estimate.
///
/// Proves:
/// - Valid measurements within gate pass
/// - Outliers outside gate are rejected
/// - Gate threshold follows chi-squared distribution
///
/// Source: Bar-Shalom et al. (2001), Chapter 2 (gating).
///         Mahalanobis (1936).
#[cfg(test)]
mod tests {
    use pr4xis_domains::formal::math::linear_algebra::matrix::Matrix;
    use pr4xis_domains::formal::math::linear_algebra::vector_space::Vector;
    use pr4xis_domains::formal::math::probability::mahalanobis;

    #[test]
    fn measurement_at_mean_always_passes_gate() {
        let mean = Vector::new(vec![10.0, 20.0]);
        let cov = Matrix::new(2, 2, vec![4.0, 0.0, 0.0, 9.0]);
        let threshold = mahalanobis::chi_squared_threshold(2, 0.95);

        let result = mahalanobis::within_gate(&mean, &mean, &cov, threshold);
        assert_eq!(result, Some(true));
    }

    #[test]
    fn close_measurement_passes_gate() {
        let mean = Vector::new(vec![10.0, 20.0]);
        let cov = Matrix::new(2, 2, vec![4.0, 0.0, 0.0, 9.0]);
        let threshold = mahalanobis::chi_squared_threshold(2, 0.95);

        // 1 sigma away
        let z = Vector::new(vec![12.0, 23.0]);
        let result = mahalanobis::within_gate(&z, &mean, &cov, threshold);
        assert_eq!(result, Some(true));
    }

    #[test]
    fn far_measurement_rejected_by_gate() {
        let mean = Vector::new(vec![10.0, 20.0]);
        let cov = Matrix::new(2, 2, vec![4.0, 0.0, 0.0, 9.0]);
        let threshold = mahalanobis::chi_squared_threshold(2, 0.95);

        // 10 sigma away — way outside gate
        let z = Vector::new(vec![30.0, 50.0]);
        let result = mahalanobis::within_gate(&z, &mean, &cov, threshold);
        assert_eq!(result, Some(false));
    }

    #[test]
    fn mahalanobis_distance_is_zero_at_mean() {
        let mean = Vector::new(vec![5.0, 10.0, 15.0]);
        let cov = Matrix::diagonal(&[1.0, 2.0, 3.0]);

        let d2 = mahalanobis::mahalanobis_squared(&mean, &mean, &cov).unwrap();
        assert!(d2 < 1e-10);
    }

    mod proptest_proofs {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn distance_at_mean_is_always_zero(
                m1 in -100.0..100.0_f64,
                m2 in -100.0..100.0_f64,
            ) {
                let mean = Vector::new(vec![m1, m2]);
                let cov = Matrix::new(2, 2, vec![2.0, 0.5, 0.5, 3.0]);
                let d2 = mahalanobis::mahalanobis_squared(&mean, &mean, &cov).unwrap();
                prop_assert!(d2 < 1e-10);
            }

            #[test]
            fn distance_is_non_negative(
                x1 in -50.0..50.0_f64, x2 in -50.0..50.0_f64,
                m1 in -50.0..50.0_f64, m2 in -50.0..50.0_f64,
            ) {
                let x = Vector::new(vec![x1, x2]);
                let mean = Vector::new(vec![m1, m2]);
                let cov = Matrix::new(2, 2, vec![2.0, 0.5, 0.5, 3.0]);
                let d2 = mahalanobis::mahalanobis_squared(&x, &mean, &cov).unwrap();
                prop_assert!(d2 >= -1e-10);
            }

            #[test]
            fn gating_is_deterministic(
                x1 in -50.0..50.0_f64, x2 in -50.0..50.0_f64,
            ) {
                let x = Vector::new(vec![x1, x2]);
                let mean = Vector::new(vec![0.0, 0.0]);
                let cov = Matrix::identity(2);
                let threshold = mahalanobis::chi_squared_threshold(2, 0.95);
                let r1 = mahalanobis::within_gate(&x, &mean, &cov, threshold);
                let r2 = mahalanobis::within_gate(&x, &mean, &cov, threshold);
                prop_assert_eq!(r1, r2);
            }
        }
    }
}

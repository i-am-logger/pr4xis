use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::science::math::signal_processing::ontology::*;

#[test]
fn signal_category_laws() {
    check_category_laws::<SignalCategory>().unwrap();
}

#[test]
fn signal_processing_ontology_validates() {
    SignalProcessingOntology::validate().unwrap();
}

#[test]
fn nyquist_theorem_holds() {
    assert!(NyquistTheorem.holds());
}

#[test]
fn aliasing_below_nyquist_holds() {
    assert!(AliasingOccursBelowNyquist.holds());
}

#[test]
fn bandwidth_positive_holds() {
    assert!(BandwidthPositive.holds());
}

#[cfg(test)]
mod proptest_proofs {
    use crate::science::math::signal_processing::filter::FirstOrderLowPass;
    use crate::science::math::signal_processing::sampling;
    use crate::science::math::signal_processing::spectrum::{self, SpectralBand};
    use proptest::prelude::*;

    proptest! {
        /// Nyquist rate is always exactly 2x bandwidth.
        #[test]
        fn nyquist_rate_is_2x_bandwidth(bw in 0.001..1e8_f64) {
            let nyquist = sampling::nyquist_rate(bw);
            prop_assert!((nyquist - 2.0 * bw).abs() < 1e-10);
        }

        /// Alias frequency always folds into [0, f_s/2].
        #[test]
        fn alias_frequency_folds_correctly(
            f in 0.1..1e6_f64,
            f_s in 1.0..1e6_f64,
        ) {
            let alias = sampling::alias_frequency(f, f_s);
            prop_assert!(alias >= -1e-10, "alias frequency must be non-negative, got {}", alias);
            prop_assert!(alias <= f_s / 2.0 + 1e-10, "alias must be <= f_s/2, got {} > {}", alias, f_s / 2.0);
        }

        /// Low-pass filter output is bounded by the input range.
        #[test]
        fn low_pass_output_bounded(
            alpha in 0.01..1.0_f64,
            inputs in proptest::collection::vec(-100.0..100.0_f64, 1..50),
        ) {
            let mut filter = FirstOrderLowPass::from_alpha(alpha);
            let outputs = filter.filter(&inputs);

            // After the first sample, the output must be within the range of all inputs seen
            let min_input = inputs.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_input = inputs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            // Output is a weighted average: bounded between 0 (initial state) and input extremes
            let bound = min_input.min(0.0);
            let upper = max_input.max(0.0);
            for &y in &outputs {
                prop_assert!(y >= bound - 1e-10, "output {} below bound {}", y, bound);
                prop_assert!(y <= upper + 1e-10, "output {} above bound {}", y, upper);
            }
        }

        /// Frequency resolution decreases (improves) with more samples.
        #[test]
        fn more_samples_better_resolution(
            f_s in 1.0..1e6_f64,
            n1 in 2_usize..1000,
            extra in 1_usize..1000,
        ) {
            let n2 = n1 + extra;
            let res1 = spectrum::frequency_resolution(f_s, n1);
            let res2 = spectrum::frequency_resolution(f_s, n2);
            prop_assert!(res2 <= res1 + 1e-10, "more samples should give finer resolution");
        }

        /// SpectralBand bandwidth is always positive for valid bands.
        #[test]
        fn spectral_band_bandwidth_positive(
            f_low in 0.0..1e5_f64,
            width in 0.01..1e5_f64,
        ) {
            let band = SpectralBand::new(f_low, f_low + width).unwrap();
            prop_assert!(band.bandwidth() > 0.0);
        }
    }
}

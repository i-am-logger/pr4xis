use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::formal::math::signal_processing::ontology::*;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn signal_category_laws() {
    assert_category_laws::<SignalProcessingCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn signal_processing_ontology_validates() {
    SignalProcessingOntology::validate().unwrap();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn nyquist_theorem_holds() {
    assert!(NyquistTheorem.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn aliasing_below_nyquist_holds() {
    assert!(AliasingOccursBelowNyquist.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn bandwidth_positive_holds() {
    assert!(BandwidthPositive.verify().is_ok());
}

#[cfg(test)]
mod proptest_proofs {
    use crate::formal::math::signal_processing::filter::FirstOrderLowPass;
    use crate::formal::math::signal_processing::sampling;
    use crate::formal::math::signal_processing::spectrum::{self, SpectralBand};
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

    pr4xis::register_praxis_value!(nyquist_rate_is_2x_bandwidth, Verifiable);
    pr4xis::register_praxis_value!(alias_frequency_folds_correctly, Verifiable);
    pr4xis::register_praxis_value!(low_pass_output_bounded, Verifiable);
    pr4xis::register_praxis_value!(more_samples_better_resolution, Verifiable);
    pr4xis::register_praxis_value!(spectral_band_bandwidth_positive, Verifiable);
}

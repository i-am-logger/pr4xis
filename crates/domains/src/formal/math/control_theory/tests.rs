use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::formal::math::control_theory::ontology::*;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn control_category_laws() {
    assert_category_laws::<ControlTheoryCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn control_theory_ontology_validates() {
    ControlTheoryOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn negative_feedback_stabilizes_holds() {
    assert!(NegativeFeedbackStabilizes.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn error_converges_to_zero_holds() {
    assert!(ErrorConvergesToZero.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn bibo_stability_definition_holds() {
    assert!(BIBOStabilityDefinition.verify().is_ok());
}

#[cfg(test)]
mod proptest_proofs {
    use crate::formal::math::control_theory::feedback;
    use crate::formal::math::control_theory::pid::{PidController, PidGains};
    use crate::formal::math::control_theory::stability;
    use crate::formal::math::control_theory::transfer_function::{self, TransferFunction};
    use crate::formal::math::quantity::unit;
    use crate::formal::math::quantity::value::Quantity;
    use proptest::prelude::*;

    proptest! {
        /// Negative feedback always reduces gain for positive G and H.
        #[test]
        fn negative_feedback_reduces_gain(
            g in 0.1..100.0_f64,
            h in 0.1..100.0_f64,
        ) {
            let cl = feedback::closed_loop_gain(g, h).value;
            prop_assert!(cl < g + 1e-10, "closed-loop gain {} should be < open-loop gain {}", cl, g);
            prop_assert!(cl > 0.0, "closed-loop gain should be positive for positive G and H");
        }

        /// Error signal is zero when reference equals measured.
        #[test]
        fn error_zero_at_setpoint(value in -1000.0..1000.0_f64) {
            let e = feedback::error_signal(value, value).value;
            prop_assert!(e.abs() < 1e-15, "error should be zero at setpoint, got {}", e);
        }

        /// PID output is bounded by saturation limits.
        #[test]
        fn pid_output_bounded(
            kp in 0.1..10.0_f64,
            ki in 0.0..5.0_f64,
            kd in 0.0..5.0_f64,
            error in -100.0..100.0_f64,
        ) {
            let gains = PidGains::new(
                Quantity::dimensionless(kp),
                Quantity::dimensionless(ki),
                Quantity::dimensionless(kd),
            );
            let mut pid = PidController::new(gains, Quantity::from_unit(0.01, &unit::SECOND))
                .with_limits(Quantity::dimensionless(-50.0), Quantity::dimensionless(50.0));
            let output = pid.update(Quantity::dimensionless(error)).value;
            prop_assert!(output >= -50.0 - 1e-10, "output {} below min", output);
            prop_assert!(output <= 50.0 + 1e-10, "output {} above max", output);
        }

        /// All negative poles imply BIBO stability.
        #[test]
        fn negative_poles_are_stable(
            poles in proptest::collection::vec(-100.0..-0.01_f64, 1..5),
        ) {
            prop_assert!(stability::is_bibo_stable(&poles));
            prop_assert_eq!(
                stability::classify_stability(&poles),
                stability::StabilityClass::AsymptoticallyStable
            );
        }

        /// System order equals denominator degree.
        #[test]
        fn system_order_is_denominator_degree(
            num_len in 1_usize..5,
            den_len in 1_usize..6,
        ) {
            let num = vec![1.0; num_len];
            let den = vec![1.0; den_len];
            let tf = TransferFunction::new(num, den);
            prop_assert_eq!(tf.order(), den_len - 1);
        }

        /// First-order system has order 1.
        #[test]
        fn first_order_system_has_order_1(
            gain in 0.1..10.0_f64,
            tau in 0.01..10.0_f64,
        ) {
            let tf = transfer_function::first_order_system(gain, tau);
            prop_assert_eq!(tf.order(), 1);
            prop_assert!(tf.is_proper());
            // DC gain should be gain (G(0) = K / 1 = K)
            let dc = tf.dc_gain().unwrap();
            prop_assert!((dc - gain).abs() < 1e-10, "DC gain {} should be {}", dc, gain);
        }
    }

    pr4xis::register_praxis_value!(negative_feedback_reduces_gain, Verifiable);
    pr4xis::register_praxis_value!(error_zero_at_setpoint, Verifiable);
    pr4xis::register_praxis_value!(pid_output_bounded, Verifiable);
    pr4xis::register_praxis_value!(negative_poles_are_stable, Verifiable);
    pr4xis::register_praxis_value!(system_order_is_denominator_degree, Verifiable);
    pr4xis::register_praxis_value!(first_order_system_has_order_1, Verifiable);
}

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::{Quantity, QuantityRange};

/// PID controller with anti-windup.
///
/// Åström & Murray (2008). *Feedback Systems*. Princeton University Press.
/// Ogata (2010). *Modern Control Engineering* (5th ed.).
///
/// PID control law: u(t) = Kp*e(t) + Ki*∫e(τ)dτ + Kd*de(t)/dt
///
/// Discrete-time approximation:
///   `u[n] = Kp*e[n] + Ki*Σe[k]*dt + Kd*(e[n] - e[n-1])/dt`
///
/// Every gain, signal, and accumulator here is a [`Quantity`] (typed at
/// every public boundary), never a bare `f64` — a gain a caller cannot
/// query/explain/cite is exactly the primitive leak this crate's typed-
/// quantity discipline exists to close. Gains, error, and the integral/
/// derivative terms are treated as DIMENSIONLESS (this is a generic PID
/// law over abstract signal values with no inherent physical unit at
/// this layer — see [`PidController::update`]'s own doc); the sample
/// period `dt` is the one genuinely time-dimensioned quantity here.
/// PID controller gains.
#[derive(Debug, Clone, PartialEq)]
pub struct PidGains {
    /// Proportional gain.
    pub kp: Quantity,
    /// Integral gain.
    pub ki: Quantity,
    /// Derivative gain.
    pub kd: Quantity,
}

impl PidGains {
    pub fn new(kp: Quantity, ki: Quantity, kd: Quantity) -> Self {
        Self { kp, ki, kd }
    }

    /// P-only controller.
    pub fn proportional(kp: Quantity) -> Self {
        Self::new(
            kp,
            Quantity::dimensionless(0.0),
            Quantity::dimensionless(0.0),
        )
    }

    /// PI controller (no derivative).
    pub fn pi(kp: Quantity, ki: Quantity) -> Self {
        Self::new(kp, ki, Quantity::dimensionless(0.0))
    }

    /// PD controller (no integral).
    pub fn pd(kp: Quantity, kd: Quantity) -> Self {
        Self::new(kp, Quantity::dimensionless(0.0), kd)
    }
}

/// Discrete PID controller with anti-windup.
#[derive(Debug, Clone, PartialEq)]
pub struct PidController {
    /// PID gains.
    pub gains: PidGains,
    /// Sample period.
    pub dt: Quantity,
    /// Accumulated integral of error.
    pub integral: Quantity,
    /// Previous error (for derivative term).
    pub prev_error: Quantity,
    /// Output saturation range for anti-windup — the typed replacement
    /// for a bare `(output_min, output_max)` pair (`QuantityRange`'s own
    /// stated purpose).
    pub output_range: QuantityRange,
}

impl PidController {
    /// Create a new PID controller. Output is unsaturated
    /// (`[-inf, inf]`) until [`PidController::with_limits`] narrows it.
    pub fn new(gains: PidGains, dt: Quantity) -> Self {
        let integral = Quantity::dimensionless(0.0);
        let prev_error = Quantity::dimensionless(0.0);
        let output_range = QuantityRange::new(
            Quantity::dimensionless(f64::NEG_INFINITY),
            Quantity::dimensionless(f64::INFINITY),
        )
        .expect("[-inf, inf] is a well-formed dimensionless range");
        Self {
            gains,
            dt,
            integral,
            prev_error,
            output_range,
        }
    }

    /// Set output saturation limits for anti-windup.
    pub fn with_limits(mut self, min: Quantity, max: Quantity) -> Self {
        self.output_range =
            QuantityRange::new(min, max).expect("output limits must share a dimension, min <= max");
        self
    }

    /// Compute the control output for the given error.
    ///
    /// u = Kp*e + Ki*integral(e) + Kd*de/dt
    ///
    /// Anti-windup: integral is clamped when output saturates.
    ///
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), never a bare
    /// `f64` — this is a generic PID control law (Åström & Murray 2008;
    /// Ogata 2010) over abstract signal values with no inherent physical
    /// unit at this layer, the same treatment as
    /// `control_theory::feedback::error_signal`.
    pub fn update(&mut self, error: Quantity) -> Quantity {
        let error = error.value;
        let dt = self.dt.value;

        // Proportional term
        let p_term = self.gains.kp.value * error;

        // Integral term (trapezoidal integration)
        let mut integral = self.integral.value;
        integral += error * dt;

        let i_term = self.gains.ki.value * integral;

        // Derivative term
        let prev_error = self.prev_error.value;
        let derivative = if dt > 0.0 {
            (error - prev_error) / dt
        } else {
            0.0
        };
        let d_term = self.gains.kd.value * derivative;

        self.prev_error = Quantity::dimensionless(error);

        // Compute raw output
        let output = p_term + i_term + d_term;

        // Anti-windup: clamp output and back-calculate integral
        let clamped = output.clamp(self.output_range.min.value, self.output_range.max.value);
        if (clamped - output).abs() > 1e-15 {
            // Output is saturated — undo the integral accumulation
            integral -= error * dt;
        }
        self.integral = Quantity::dimensionless(integral);

        Quantity::from_unit(clamped, &unit::UNITLESS)
    }

    /// Reset the controller state.
    pub fn reset(&mut self) {
        self.integral = Quantity::dimensionless(0.0);
        self.prev_error = Quantity::dimensionless(0.0);
    }
}

/// Ziegler-Nichols tuning: given the ultimate gain Ku and ultimate period Tu,
/// compute PID gains.
///
/// Classic Ziegler-Nichols (1942) tuning rules:
/// - P:   Kp = 0.5 * Ku
/// - PI:  Kp = 0.45 * Ku,  Ki = 1.2 * Kp / Tu
/// - PID: Kp = 0.6 * Ku,   Ki = 2 * Kp / Tu,   Kd = Kp * Tu / 8
pub fn ziegler_nichols_pid(ku: Quantity, tu: Quantity) -> PidGains {
    let ku = ku.value;
    let tu = tu.value;
    let kp = 0.6 * ku;
    let ki = 2.0 * kp / tu;
    let kd = kp * tu / 8.0;
    PidGains::new(
        Quantity::dimensionless(kp),
        Quantity::dimensionless(ki),
        Quantity::dimensionless(kd),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn gains(kp: f64, ki: f64, kd: f64) -> PidGains {
        PidGains::new(
            Quantity::dimensionless(kp),
            Quantity::dimensionless(ki),
            Quantity::dimensionless(kd),
        )
    }

    fn dt_seconds(seconds: f64) -> Quantity {
        Quantity::from_unit(seconds, &unit::SECOND)
    }

    fn dimensionless(value: f64) -> Quantity {
        Quantity::dimensionless(value)
    }

    /// Zero error with no accumulated history yields zero output — the
    /// P/I/D terms are all proportional to error (or its history), so
    /// a controller that has never seen a nonzero error must be silent.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn zero_error_zero_history_yields_zero_output() {
        let mut pid = PidController::new(gains(0.4, 0.05, 0.1), dt_seconds(1.0));
        let output = pid.update(dimensionless(0.0));
        assert!(output.value.abs() < 1e-9, "got {}", output.value);
    }

    /// Anti-windup back-calculation (Åström & Hägglund 2006 §3.5) must
    /// hold regardless of how long or how far the error persists: the
    /// output is ALWAYS within [output_min, output_max], never merely
    /// "usually."
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn output_is_always_within_saturation() {
        let mut pid = PidController::new(gains(0.4, 0.05, 0.1), dt_seconds(1.0))
            .with_limits(dimensionless(-1.0), dimensionless(1.0));
        for _ in 0..100 {
            let output = pid.update(dimensionless(5.0));
            assert!(
                output.value >= pid.output_range.min.value
                    && output.value <= pid.output_range.max.value,
                "output {} escaped [{}, {}]",
                output.value,
                pid.output_range.min.value,
                pid.output_range.max.value
            );
        }
    }

    /// Integral action: a persistent nonzero error must drive the
    /// output magnitude UP over time (until saturation), never down —
    /// the whole point of the I term (Åström & Murray 2008 §11.1).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn integral_term_drives_output_up_over_time() {
        let mut pid = PidController::new(gains(0.4, 0.05, 0.1), dt_seconds(1.0))
            .with_limits(dimensionless(-100.0), dimensionless(100.0));
        let first = pid.update(dimensionless(5.0)).value;
        let mut last = first;
        for _ in 0..49 {
            last = pid.update(dimensionless(5.0)).value;
        }
        assert!(
            last >= first,
            "output should grow as the integral accumulates: first={first}, last={last}"
        );
    }

    /// Anti-windup actually engages: once saturated, the accumulated
    /// integral must be LESS than the naive (unclamped) trapezoidal sum
    /// — the back-calculation in `update` undoes the excess. Without
    /// this the "anti-windup" claim in the module doc would be untested.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn anti_windup_clamps_the_integral_once_saturated() {
        let mut pid = PidController::new(gains(0.4, 0.05, 0.1), dt_seconds(1.0))
            .with_limits(dimensionless(-1.0), dimensionless(1.0));
        for _ in 0..200 {
            pid.update(dimensionless(50.0));
        }
        let naive_unclamped_integral = 50.0 * 200.0;
        assert!(
            pid.integral.value < naive_unclamped_integral,
            "anti-windup must prevent the integral from winding up unbounded: \
             got {}, naive would be {naive_unclamped_integral}",
            pid.integral.value
        );
    }

    proptest! {
        /// Axiom: with positive Kp and Ki = Kd = 0 (P-only), the sign of
        /// the output matches the sign of the error — the controller
        /// pushes in the direction that reduces the error, never away
        /// from it. Åström & Murray (2008) Ch. 1.
        #[test]
        fn proportional_only_output_sign_matches_error_sign(
            kp in 0.01f64..10.0,
            error in -50.0f64..50.0,
        ) {
            let mut pid = PidController::new(
                PidGains::proportional(Quantity::dimensionless(kp)),
                dt_seconds(1.0),
            )
            .with_limits(dimensionless(-1e6), dimensionless(1e6));
            let output = pid.update(dimensionless(error)).value;
            if error.abs() < 1e-9 {
                prop_assert!(output.abs() < 1e-9);
            } else {
                prop_assert_eq!(output.signum(), error.signum());
            }
        }

        /// Axiom: output is always saturated to [output_min, output_max],
        /// for ANY sequence of errors and ANY gains — the anti-windup
        /// contract holds universally, not just on the hand-picked cases
        /// above.
        #[test]
        fn output_always_saturated(
            kp in -10.0f64..10.0,
            ki in -10.0f64..10.0,
            kd in -10.0f64..10.0,
            errors in proptest::collection::vec(-100.0f64..100.0, 1..50),
        ) {
            let mut pid = PidController::new(gains(kp, ki, kd), dt_seconds(1.0))
                .with_limits(dimensionless(-1.0), dimensionless(1.0));
            for error in errors {
                let output = pid.update(dimensionless(error)).value;
                prop_assert!(output >= pid.output_range.min.value && output <= pid.output_range.max.value);
            }
        }
    }

    pr4xis::register_praxis_value!(proportional_only_output_sign_matches_error_sign, Verifiable);
    pr4xis::register_praxis_value!(output_always_saturated, Verifiable);
}

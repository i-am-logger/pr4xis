#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

use crate::applied::navigation::ins_gnss::coupling::{
    CouplingMode, PosVelCoupling, coasting_position_error, scalar_kalman_gain,
};
use crate::applied::navigation::ins_gnss::ontology::{CouplingLevel, InsGnssState};
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::duration::Duration;
use crate::natural::physics::kinematics::acceleration::Acceleration;

/// INS/GNSS integration situation.
#[derive(Debug, Clone, PartialEq)]
pub struct InsGnssSituation {
    /// Current system state.
    pub state: InsGnssState,
    /// Active coupling mode.
    pub coupling: CouplingLevel,
    /// Position error estimate (1-sigma).
    pub position_error: Quantity,
    /// Velocity error estimate (1-sigma).
    pub velocity_error: Quantity,
    /// Time since last GNSS update.
    pub time_since_gnss: Duration,
    /// Accelerometer bias estimate.
    pub accel_bias: Acceleration,
    /// Step counter.
    pub step: usize,
}

impl Situation for InsGnssSituation {}

/// INS/GNSS integration action.
#[derive(Debug, Clone)]
pub enum InsGnssAction {
    /// INS mechanization step (propagate state).
    InsPropagation {
        /// Time step.
        dt: Duration,
    },
    /// GNSS measurement update.
    GnssUpdate {
        /// GNSS position measurement noise (1-sigma, meters).
        measurement_noise: f64,
        /// Number of visible satellites.
        num_satellites: usize,
    },
    /// GNSS signal lost.
    GnssOutage,
    /// GNSS signal reacquired.
    GnssReacquisition {
        /// GNSS position measurement noise (1-sigma, meters).
        measurement_noise: f64,
    },
}

impl Action for InsGnssAction {
    type Sit = InsGnssSituation;
}

/// Apply an INS/GNSS action to the current situation.
pub fn apply_ins_gnss(
    situation: &InsGnssSituation,
    action: &InsGnssAction,
) -> Result<InsGnssSituation, String> {
    match action {
        InsGnssAction::InsPropagation { dt } => {
            if dt.is_negative() {
                return Err("dt must be non-negative".into());
            }
            let bias_magnitude = situation.accel_bias.magnitude();
            // During coasting, position error grows quadratically due to accel bias
            let additional_error = coasting_position_error(bias_magnitude.value, dt.seconds());
            Ok(InsGnssSituation {
                state: situation.state,
                coupling: situation.coupling,
                position_error: Quantity::from_unit(
                    situation.position_error.value + additional_error.value,
                    &unit::METER,
                ),
                velocity_error: Quantity::from_unit(
                    situation.velocity_error.value + bias_magnitude.value * dt.seconds(),
                    &unit::METER_PER_SECOND,
                ),
                time_since_gnss: situation.time_since_gnss.add(dt),
                accel_bias: situation.accel_bias.clone(),
                step: situation.step + 1,
            })
        }
        InsGnssAction::GnssUpdate {
            measurement_noise,
            num_satellites,
        } => {
            let mode = CouplingMode::for_level(situation.coupling);
            if !mode.can_operate(*num_satellites) {
                return Err(format!(
                    "{:?} requires >= {} satellites, have {}",
                    situation.coupling, mode.min_satellites, num_satellites
                ));
            }
            // Scalar Kalman update on position variance; the velocity error is
            // corrected through the pos–vel coupling by the SAME gain.
            let prior_var = situation.position_error.value * situation.position_error.value;
            let meas_var = measurement_noise * measurement_noise;
            let k = scalar_kalman_gain(prior_var, meas_var);
            let post_var = (1.0 - k.value) * prior_var;
            Ok(InsGnssSituation {
                state: InsGnssState::NavigationMode,
                coupling: situation.coupling,
                position_error: Quantity::from_unit(post_var.sqrt(), &unit::METER),
                velocity_error: PosVelCoupling::nominal()
                    .velocity_error_after_fix(situation.velocity_error.value, k.value),
                time_since_gnss: Duration::zero(),
                accel_bias: situation.accel_bias.clone(),
                step: situation.step + 1,
            })
        }
        InsGnssAction::GnssOutage => Ok(InsGnssSituation {
            state: InsGnssState::Coasting,
            coupling: situation.coupling,
            position_error: situation.position_error.clone(),
            velocity_error: situation.velocity_error.clone(),
            time_since_gnss: situation.time_since_gnss.clone(),
            accel_bias: situation.accel_bias.clone(),
            step: situation.step + 1,
        }),
        InsGnssAction::GnssReacquisition { measurement_noise } => {
            let prior_var = situation.position_error.value * situation.position_error.value;
            let meas_var = measurement_noise * measurement_noise;
            let k = scalar_kalman_gain(prior_var, meas_var);
            let post_var = (1.0 - k.value) * prior_var;
            Ok(InsGnssSituation {
                state: InsGnssState::GnssReacquired,
                coupling: situation.coupling,
                position_error: Quantity::from_unit(post_var.sqrt(), &unit::METER),
                velocity_error: PosVelCoupling::reacquisition()
                    .velocity_error_after_fix(situation.velocity_error.value, k.value),
                time_since_gnss: Duration::zero(),
                accel_bias: situation.accel_bias.clone(),
                step: situation.step + 1,
            })
        }
    }
}

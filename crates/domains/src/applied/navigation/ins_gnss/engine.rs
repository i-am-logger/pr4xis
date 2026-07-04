#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

use crate::applied::navigation::ins_gnss::coupling::{
    CouplingMode, PosVelCoupling, coasting_position_error, scalar_kalman_gain,
};
use crate::applied::navigation::ins_gnss::ontology::{CouplingLevel, InsGnssState};

/// INS/GNSS integration situation.
#[derive(Debug, Clone, PartialEq)]
pub struct InsGnssSituation {
    /// Current system state.
    pub state: InsGnssState,
    /// Active coupling mode.
    pub coupling: CouplingLevel,
    /// Position error estimate (1-sigma, meters).
    pub position_error: f64,
    /// Velocity error estimate (1-sigma, m/s).
    pub velocity_error: f64,
    /// Time since last GNSS update (seconds).
    pub time_since_gnss: f64,
    /// Accelerometer bias estimate (m/s^2).
    pub accel_bias: f64,
    /// Step counter.
    pub step: usize,
}

impl Situation for InsGnssSituation {}

/// INS/GNSS integration action.
#[derive(Debug, Clone)]
pub enum InsGnssAction {
    /// INS mechanization step (propagate state).
    InsPropagation {
        /// Time step (seconds).
        dt: f64,
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
            if *dt < 0.0 {
                return Err("dt must be non-negative".into());
            }
            // During coasting, position error grows quadratically due to accel bias
            let additional_error = coasting_position_error(situation.accel_bias, *dt);
            Ok(InsGnssSituation {
                state: situation.state,
                coupling: situation.coupling,
                position_error: situation.position_error + additional_error,
                velocity_error: situation.velocity_error + situation.accel_bias.abs() * dt,
                time_since_gnss: situation.time_since_gnss + dt,
                accel_bias: situation.accel_bias,
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
            let prior_var = situation.position_error * situation.position_error;
            let meas_var = measurement_noise * measurement_noise;
            let k = scalar_kalman_gain(prior_var, meas_var);
            let post_var = (1.0 - k) * prior_var;
            Ok(InsGnssSituation {
                state: InsGnssState::NavigationMode,
                coupling: situation.coupling,
                position_error: post_var.sqrt(),
                velocity_error: PosVelCoupling::nominal()
                    .velocity_error_after_fix(situation.velocity_error, k),
                time_since_gnss: 0.0,
                accel_bias: situation.accel_bias,
                step: situation.step + 1,
            })
        }
        InsGnssAction::GnssOutage => Ok(InsGnssSituation {
            state: InsGnssState::Coasting,
            coupling: situation.coupling,
            position_error: situation.position_error,
            velocity_error: situation.velocity_error,
            time_since_gnss: situation.time_since_gnss,
            accel_bias: situation.accel_bias,
            step: situation.step + 1,
        }),
        InsGnssAction::GnssReacquisition { measurement_noise } => {
            let prior_var = situation.position_error * situation.position_error;
            let meas_var = measurement_noise * measurement_noise;
            let k = scalar_kalman_gain(prior_var, meas_var);
            let post_var = (1.0 - k) * prior_var;
            Ok(InsGnssSituation {
                state: InsGnssState::GnssReacquired,
                coupling: situation.coupling,
                position_error: post_var.sqrt(),
                velocity_error: PosVelCoupling::reacquisition()
                    .velocity_error_after_fix(situation.velocity_error, k),
                time_since_gnss: 0.0,
                accel_bias: situation.accel_bias,
                step: situation.step + 1,
            })
        }
    }
}

use pr4xis::engine::{Action, Situation};

use crate::applied::navigation::ins_gnss::coupling::{
    CouplingMode, coasting_position_error, scalar_kalman_update,
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

impl Situation for InsGnssSituation {
    fn describe(&self) -> String {
        format!(
            "INS/GNSS step={}, state={:?}, coupling={:?}, pos_err={:.2}m, t_gnss={:.1}s",
            self.step, self.state, self.coupling, self.position_error, self.time_since_gnss
        )
    }

    fn is_terminal(&self) -> bool {
        false
    }
}

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

    fn describe(&self) -> String {
        match self {
            InsGnssAction::InsPropagation { dt } => format!("INS propagation dt={:.4}s", dt),
            InsGnssAction::GnssUpdate {
                num_satellites,
                measurement_noise,
            } => format!(
                "GNSS update ({} sats, noise={:.1}m)",
                num_satellites, measurement_noise
            ),
            InsGnssAction::GnssOutage => "GNSS outage".to_string(),
            InsGnssAction::GnssReacquisition { measurement_noise } => {
                format!("GNSS reacquisition (noise={:.1}m)", measurement_noise)
            }
        }
    }
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
            // Scalar Kalman update on position variance
            let prior_var = situation.position_error * situation.position_error;
            let meas_var = measurement_noise * measurement_noise;
            let post_var = scalar_kalman_update(prior_var, meas_var);
            Ok(InsGnssSituation {
                state: InsGnssState::NavigationMode,
                coupling: situation.coupling,
                position_error: post_var.sqrt(),
                velocity_error: situation.velocity_error * 0.8, // GNSS also helps velocity
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
            let post_var = scalar_kalman_update(prior_var, meas_var);
            Ok(InsGnssSituation {
                state: InsGnssState::GnssReacquired,
                coupling: situation.coupling,
                position_error: post_var.sqrt(),
                velocity_error: situation.velocity_error * 0.5,
                time_since_gnss: 0.0,
                accel_bias: situation.accel_bias,
                step: situation.step + 1,
            })
        }
    }
}

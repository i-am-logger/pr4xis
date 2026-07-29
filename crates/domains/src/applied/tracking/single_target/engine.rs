#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::sensor_fusion::fusion::engine::{FusionAction, new_fusion_engine};
use crate::applied::sensor_fusion::state::estimate::StateEstimate;
use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::duration::Duration;
use crate::formal::math::temporal::instant::Instant;
use crate::formal::math::temporal::time_system::TimeSystem;

use crate::applied::tracking::single_target::motion_model;

/// Create a 1D constant velocity tracker.
///
/// State: [position, velocity].
/// Observation: position only (H = [1, 0]).
///
/// Source: Bar-Shalom (2001), Example 6.4.
pub fn new_cv_tracker_1d(
    initial_pos: Quantity,
    initial_vel: Quantity,
    initial_uncertainty: f64,
    _process_noise_intensity: f64,
    _measurement_noise: f64,
) -> pr4xis::engine::Engine<FusionAction> {
    let state = StateEstimate::new(
        Vector::new(vec![initial_pos.value, initial_vel.value]),
        Matrix::diagonal(&[initial_uncertainty, initial_uncertainty]),
        Instant::new(0.0, TimeSystem::GPS),
    );
    new_fusion_engine(state)
}

/// Predict step for constant velocity tracker.
pub fn cv_predict_1d(dt: Duration, q: f64) -> FusionAction {
    let (f, process_noise) = motion_model::constant_velocity_1d(dt.clone(), q);
    FusionAction::Predict {
        dt,
        transition: f,
        process_noise,
    }
}

/// Update step with position-only measurement.
pub fn cv_update_position_1d(measurement: f64, noise: f64) -> FusionAction {
    FusionAction::Update {
        observation_matrix: Matrix::new(1, 2, vec![1.0, 0.0]),
        measurement: Vector::new(vec![measurement]),
        measurement_noise: Matrix::new(1, 1, vec![noise]),
    }
}

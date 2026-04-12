/// Process control data structures and algorithms.
///
/// Source: Ogunnaike & Ray (1994), *Process Dynamics, Modeling, and Control*
///
/// The PID controller delegates to `crate::formal::math::control_theory::pid`,
/// which provides the canonical implementation with anti-windup. This module
/// wraps it with a process-control-specific API (setpoint + measured value).
use crate::formal::math::control_theory::pid as ct_pid;

/// A process sensor reading with validation.
#[derive(Debug, Clone)]
pub struct SensorReading {
    /// Measured value in physical units.
    pub value: f64,
    /// Sensor identifier.
    pub sensor_id: usize,
    /// Timestamp in seconds.
    pub timestamp: f64,
    /// Whether the reading passed validation.
    pub valid: bool,
}

/// PID controller for process control.
///
/// Thin wrapper around `control_theory::pid::PidController` that accepts
/// (setpoint, measured, dt) instead of raw error. This is the standard
/// process-control interface (Ogunnaike & Ray 1994).
#[derive(Debug, Clone)]
pub struct PidController {
    /// Inner PID from the control theory ontology.
    inner: ct_pid::PidController,
    /// Output limits (also stored in inner, exposed for tests).
    pub output_min: f64,
    pub output_max: f64,
}

impl PidController {
    pub fn new(kp: f64, ki: f64, kd: f64, output_min: f64, output_max: f64) -> Self {
        // dt=1.0 as placeholder; actual dt is supplied per update call.
        let gains = ct_pid::PidGains::new(kp, ki, kd);
        let inner = ct_pid::PidController::new(gains, 1.0).with_limits(output_min, output_max);
        Self {
            inner,
            output_min,
            output_max,
        }
    }

    /// Compute PID control output.
    ///
    /// setpoint: desired value
    /// measured: current measured value
    /// dt: time step in seconds
    pub fn update(&mut self, setpoint: f64, measured: f64, dt: f64) -> f64 {
        let error = setpoint - measured;
        // Update the inner controller's dt for this step
        self.inner.dt = dt;
        self.inner.update(error)
    }

    /// Reset the controller state.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Access the integral accumulator (for test inspection).
    pub fn integral(&self) -> f64 {
        self.inner.integral
    }

    /// Access the previous error (for test inspection).
    pub fn prev_error(&self) -> f64 {
        self.inner.prev_error
    }
}

/// Convert Celsius to Kelvin.
pub fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

/// Convert Kelvin to Celsius.
pub fn kelvin_to_celsius(kelvin: f64) -> f64 {
    kelvin - 273.15
}

/// Validate a temperature reading (must be above absolute zero in Kelvin).
pub fn validate_temperature_k(value: f64) -> bool {
    value >= 0.0
}

/// Validate a pressure reading (absolute pressure must be non-negative).
pub fn validate_pressure(value: f64) -> bool {
    value >= 0.0
}

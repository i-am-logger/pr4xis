#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Process control data structures and algorithms.
///
/// Source: Ogunnaike & Ray (1994), *Process Dynamics, Modeling, and Control*
///
/// The PID controller delegates to `crate::formal::math::control_theory::pid`,
/// which provides the canonical implementation with anti-windup. This module
/// wraps it with a process-control-specific API (setpoint + measured value).
use crate::formal::math::control_theory::pid as ct_pid;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

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
        // dt=1.0 (dimensionless-seconds placeholder); actual dt is
        // supplied per update call.
        let gains = ct_pid::PidGains::new(
            Quantity::dimensionless(kp),
            Quantity::dimensionless(ki),
            Quantity::dimensionless(kd),
        );
        let inner = ct_pid::PidController::new(gains, Quantity::from_unit(1.0, &unit::SECOND))
            .with_limits(
                Quantity::dimensionless(output_min),
                Quantity::dimensionless(output_max),
            );
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
    ///
    /// Returns the dimensionless [`Quantity`] (`unit::UNITLESS`) that
    /// `ct_pid::PidController::update` already produces — a generic PID
    /// control law over abstract signal values with no inherent physical
    /// unit at this layer (Ogunnaike & Ray 1994; Åström & Murray 2008).
    pub fn update(&mut self, setpoint: f64, measured: f64, dt: f64) -> Quantity {
        let error = setpoint - measured;
        // Update the inner controller's dt for this step
        self.inner.dt = Quantity::from_unit(dt, &unit::SECOND);
        self.inner.update(Quantity::dimensionless(error))
    }

    /// Reset the controller state.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Access the integral accumulator (for test inspection).
    ///
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), matching
    /// [`PidController::update`].
    pub fn integral(&self) -> Quantity {
        self.inner.integral.clone()
    }

    /// Access the previous error (for test inspection).
    ///
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), matching
    /// [`PidController::update`].
    pub fn prev_error(&self) -> Quantity {
        self.inner.prev_error.clone()
    }
}

/// Convert Celsius to Kelvin.
///
/// Returns a [`Quantity`] tagged `Dimension::TEMPERATURE`. Kelvin is the SI
/// base unit for temperature, so this is the same canonical representation
/// [`kelvin_to_celsius`] would produce for the corresponding Kelvin reading.
pub fn celsius_to_kelvin(celsius: f64) -> Quantity {
    Quantity::from_unit(celsius, &unit::CELSIUS)
}

/// Convert Kelvin to Celsius.
///
/// Returns a [`Quantity`] tagged `Dimension::TEMPERATURE`; call
/// `.in_unit(&unit::CELSIUS)` on the result to read off the Celsius-scaled
/// number.
pub fn kelvin_to_celsius(kelvin: f64) -> Quantity {
    Quantity::from_unit(kelvin, &unit::KELVIN)
}

/// Validate a temperature reading (must be above absolute zero in Kelvin).
pub fn validate_temperature_k(value: f64) -> bool {
    value >= 0.0
}

/// Validate a pressure reading (absolute pressure must be non-negative).
pub fn validate_pressure(value: f64) -> bool {
    value >= 0.0
}

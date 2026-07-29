#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::angle::Angle;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::duration::Duration;

/// EW geolocation algorithms.
///
/// Source: Poisel (2012), *Electronic Warfare Target Location Methods*
/// Wrap an angle to [-pi, pi].
///
/// Returns a [`Quantity`] in [`unit::RADIAN`] (`Dimension::ANGLE`, which is
/// dimensionless per BIPM SI Brochure (2019) §2.3.3 — radian is a ratio of
/// two lengths) rather than a bare `f64`, so the wrapped bearing stays a
/// citable ontological quantity through the geolocation pipeline.
pub fn wrap_angle(angle: &Quantity) -> Quantity {
    let mut a = angle.value % (2.0 * core::f64::consts::PI);
    if a > core::f64::consts::PI {
        a -= 2.0 * core::f64::consts::PI;
    } else if a < -core::f64::consts::PI {
        a += 2.0 * core::f64::consts::PI;
    }
    Quantity::from_unit(a, &unit::RADIAN)
}

/// AOA measurement from a single sensor.
#[derive(Debug, Clone)]
pub struct AoaMeasurement {
    /// Sensor position as a 2D `(x, y)` vector in the local ground-plane
    /// (sensor-network) Cartesian frame.
    pub sensor_pos: Vector,
    /// Measured bearing as an [`Angle`] (from north, clockwise).
    pub bearing: Angle,
    /// Measurement uncertainty (1-sigma), a [`Quantity`] in [`unit::RADIAN`].
    pub sigma: Quantity,
}

/// Determinant magnitude below which two lines of bearing (LOBs) are treated
/// as parallel, i.e. an ill-conditioned intersection with no reliable solution.
///
/// The determinant `sin1*cos2 - sin2*cos1` equals `sin(bearing1 - bearing2)`,
/// so it is the sine of the angular separation between the two LOBs: a
/// dimensionless quantity in `[-1, 1]` that vanishes when the bearings are
/// (anti)parallel. The triangulation geometry follows Poisel (2012),
/// *Electronic Warfare Target Location Methods*; the specific tolerance below
/// is not from that text but a numerical-conditioning guard: `1e-12` sits just
/// above the IEEE-754 double-precision noise floor for this normalized
/// determinant, so a smaller magnitude means the crossing angle is
/// numerically indistinguishable from zero and the intersection `t = ... / det`
/// would blow up. Returning `None` makes that failure explicit rather than
/// producing a wildly extrapolated position.
const PARALLEL_LOB_DET_EPS: f64 = 1e-12;

/// Compute AOA intersection of two lines of bearing (triangulation).
///
/// Returns the estimated emitter position as a 2D `(x, y)` vector in the local
/// ground-plane frame, or None if lines are parallel.
pub fn aoa_triangulation(m1: &AoaMeasurement, m2: &AoaMeasurement) -> Option<Vector> {
    let sin1 = m1.bearing.sin();
    let cos1 = m1.bearing.cos();
    let sin2 = m2.bearing.sin();
    let cos2 = m2.bearing.cos();

    let det = sin1 * cos2 - sin2 * cos1;
    if det.abs() < PARALLEL_LOB_DET_EPS {
        return None; // parallel lines
    }

    let dx = m2.sensor_pos.get(0) - m1.sensor_pos.get(0);
    let dy = m2.sensor_pos.get(1) - m1.sensor_pos.get(1);

    let t = (dx * cos2 - dy * sin2) / det;

    Some(Vector::new(vec![
        m1.sensor_pos.get(0) + t * sin1,
        m1.sensor_pos.get(1) + t * cos1,
    ]))
}

/// TDOA measurement between a sensor pair.
#[derive(Debug, Clone)]
pub struct TdoaMeasurement {
    /// Position of sensor A as a 2D `(x, y)` vector in the local ground-plane
    /// (sensor-network) Cartesian frame.
    pub sensor_a: Vector,
    /// Position of sensor B as a 2D `(x, y)` vector in the local ground-plane
    /// (sensor-network) Cartesian frame.
    pub sensor_b: Vector,
    /// Time difference of arrival: t_B - t_A, a [`Duration`] (may be negative).
    pub tdoa: Duration,
    /// Speed of signal propagation, a [`Quantity`] in [`unit::METER_PER_SECOND`].
    pub signal_speed: Quantity,
}

impl TdoaMeasurement {
    /// Compute the range difference.
    ///
    /// `tdoa` (seconds) times `signal_speed` (m/s) is a real physical range
    /// in meters, so this returns a [`Quantity`] in [`unit::METER`] rather
    /// than a bare `f64`.
    pub fn range_difference(&self) -> Quantity {
        Quantity::from_unit(self.tdoa.seconds() * self.signal_speed.value, &unit::METER)
    }
}

/// Compute distance between two 2D points.
///
/// The sensor and emitter positions are real ground-plane coordinates in
/// meters (see [`AoaMeasurement::sensor_pos`] / [`TdoaMeasurement`]), so the
/// Euclidean distance between them is a real physical length: a [`Quantity`]
/// in [`unit::METER`], not a bare `f64`.
pub fn distance_2d(a: &Vector, b: &Vector) -> Quantity {
    let dx = b.get(0) - a.get(0);
    let dy = b.get(1) - a.get(1);
    Quantity::from_unit((dx * dx + dy * dy).sqrt(), &unit::METER)
}

/// Compute TDOA residual for a candidate emitter position.
///
/// Both the predicted range difference (from candidate-emitter geometry) and
/// the measured range difference (from `tdoa * signal_speed`) are lengths in
/// meters, so their difference is itself a [`Quantity`] in [`unit::METER`]
/// (a range residual), not a bare `f64`.
pub fn tdoa_residual(measurement: &TdoaMeasurement, emitter: &Vector) -> Quantity {
    let r_a = distance_2d(&measurement.sensor_a, emitter);
    let r_b = distance_2d(&measurement.sensor_b, emitter);
    let predicted_range_diff = r_b
        .sub(&r_a)
        .expect("both r_a and r_b are METER-dimensioned distances");
    predicted_range_diff
        .sub(&measurement.range_difference())
        .expect("predicted range diff and measured range diff are both METER-dimensioned")
}

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::angle::Angle;
use crate::formal::math::linear_algebra::vector_space::Vector;

/// EW geolocation algorithms.
///
/// Source: Poisel (2012), *Electronic Warfare Target Location Methods*
/// Wrap an angle to [-pi, pi].
pub fn wrap_angle(angle: f64) -> f64 {
    let mut a = angle % (2.0 * core::f64::consts::PI);
    if a > core::f64::consts::PI {
        a -= 2.0 * core::f64::consts::PI;
    } else if a < -core::f64::consts::PI {
        a += 2.0 * core::f64::consts::PI;
    }
    a
}

/// AOA measurement from a single sensor.
#[derive(Debug, Clone)]
pub struct AoaMeasurement {
    /// Sensor position as a 2D `(x, y)` vector in the local ground-plane
    /// (sensor-network) Cartesian frame.
    pub sensor_pos: Vector,
    /// Measured bearing as an [`Angle`] (from north, clockwise).
    pub bearing: Angle,
    /// Measurement uncertainty (radians, 1-sigma).
    pub sigma: f64,
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
    /// Time difference of arrival (seconds): t_B - t_A.
    pub tdoa: f64,
    /// Speed of signal propagation (m/s).
    pub signal_speed: f64,
}

impl TdoaMeasurement {
    /// Compute the range difference.
    pub fn range_difference(&self) -> f64 {
        self.tdoa * self.signal_speed
    }
}

/// Compute distance between two 2D points.
pub fn distance_2d(a: &Vector, b: &Vector) -> f64 {
    let dx = b.get(0) - a.get(0);
    let dy = b.get(1) - a.get(1);
    (dx * dx + dy * dy).sqrt()
}

/// Compute TDOA residual for a candidate emitter position.
pub fn tdoa_residual(measurement: &TdoaMeasurement, emitter: &Vector) -> f64 {
    let r_a = distance_2d(&measurement.sensor_a, emitter);
    let r_b = distance_2d(&measurement.sensor_b, emitter);
    let predicted_range_diff = r_b - r_a;
    predicted_range_diff - measurement.range_difference()
}

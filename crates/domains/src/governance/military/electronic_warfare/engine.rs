/// EW geolocation algorithms.
///
/// Source: Poisel (2012), *Electronic Warfare Target Location Methods*
/// Wrap an angle to [-pi, pi].
pub fn wrap_angle(angle: f64) -> f64 {
    let mut a = angle % (2.0 * std::f64::consts::PI);
    if a > std::f64::consts::PI {
        a -= 2.0 * std::f64::consts::PI;
    } else if a < -std::f64::consts::PI {
        a += 2.0 * std::f64::consts::PI;
    }
    a
}

/// AOA measurement from a single sensor.
#[derive(Debug, Clone)]
pub struct AoaMeasurement {
    /// Sensor position [x, y].
    pub sensor_pos: [f64; 2],
    /// Measured bearing (radians, from north CW).
    pub bearing: f64,
    /// Measurement uncertainty (radians, 1-sigma).
    pub sigma: f64,
}

/// Compute AOA intersection of two lines of bearing (triangulation).
///
/// Returns the estimated emitter position, or None if lines are parallel.
pub fn aoa_triangulation(m1: &AoaMeasurement, m2: &AoaMeasurement) -> Option<[f64; 2]> {
    let sin1 = m1.bearing.sin();
    let cos1 = m1.bearing.cos();
    let sin2 = m2.bearing.sin();
    let cos2 = m2.bearing.cos();

    let det = sin1 * cos2 - sin2 * cos1;
    if det.abs() < 1e-12 {
        return None; // parallel lines
    }

    let dx = m2.sensor_pos[0] - m1.sensor_pos[0];
    let dy = m2.sensor_pos[1] - m1.sensor_pos[1];

    let t = (dx * cos2 - dy * sin2) / det;

    Some([m1.sensor_pos[0] + t * sin1, m1.sensor_pos[1] + t * cos1])
}

/// TDOA measurement between a sensor pair.
#[derive(Debug, Clone)]
pub struct TdoaMeasurement {
    /// Position of sensor A [x, y].
    pub sensor_a: [f64; 2],
    /// Position of sensor B [x, y].
    pub sensor_b: [f64; 2],
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
pub fn distance_2d(a: &[f64; 2], b: &[f64; 2]) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    (dx * dx + dy * dy).sqrt()
}

/// Compute TDOA residual for a candidate emitter position.
pub fn tdoa_residual(measurement: &TdoaMeasurement, emitter: &[f64; 2]) -> f64 {
    let r_a = distance_2d(&measurement.sensor_a, emitter);
    let r_b = distance_2d(&measurement.sensor_b, emitter);
    let predicted_range_diff = r_b - r_a;
    predicted_range_diff - measurement.range_difference()
}

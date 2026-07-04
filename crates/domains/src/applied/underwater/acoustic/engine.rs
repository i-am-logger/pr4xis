#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::angle::Angle;
use crate::formal::math::linear_algebra::vector_space::Vector;

/// Sound speed profile computation.
///
/// Source: Mackenzie (1981), "Nine-term equation for sound speed in the oceans"
/// Compute sound speed in seawater using Mackenzie's equation.
///
/// temperature: water temperature in degrees Celsius
/// salinity: salinity in parts per thousand (PSU)
/// depth: depth in meters
///
/// Returns sound speed in m/s.
pub fn mackenzie_sound_speed(temperature: f64, salinity: f64, depth: f64) -> f64 {
    let t = temperature;
    let s = salinity;
    let d = depth;

    1448.96 + 4.591 * t - 0.05304 * t * t
        + 0.000237_1 * t * t * t
        + 1.340 * (s - 35.0)
        + 0.01630 * d
        + 1.675e-7 * d * d
        - 0.01025 * t * (s - 35.0)
        - 7.139e-13 * t * d * d * d
}

/// Compute range from two-way travel time and sound speed.
pub fn range_from_travel_time(travel_time: f64, sound_speed: f64) -> f64 {
    sound_speed * travel_time / 2.0
}

/// USBL angle measurement (simplified).
#[derive(Debug, Clone)]
pub struct UsblFix {
    /// Slant range in meters.
    pub range: f64,
    /// Bearing angle (an element of the circle group S¹).
    pub bearing: Angle,
    /// Depression angle (an element of the circle group S¹).
    pub depression: Angle,
}

impl UsblFix {
    /// Convert to a Cartesian position in the transceiver-local frame
    /// (x forward, y starboard, z down so positive depth is downward).
    pub fn to_cartesian(&self) -> Vector {
        let cos_dep = self.depression.cos();
        Vector::new(vec![
            self.range * cos_dep * self.bearing.cos(),
            self.range * cos_dep * self.bearing.sin(),
            -self.range * self.depression.sin(), // positive depth is downward
        ])
    }
}

/// LBL position fix from range measurements to multiple transponders.
///
/// transponders: transponder positions, each a 3-D point in a common local frame
/// ranges: measured ranges to each transponder
///
/// Returns the estimated position (same local frame) using trilateration
/// (simplified least-squares).
pub fn lbl_trilateration(transponders: &[Vector], ranges: &[f64]) -> Option<Vector> {
    if transponders.len() < 3 || transponders.len() != ranges.len() {
        return None;
    }
    // Simplified: use centroid weighted by inverse range as approximation
    let mut wx = 0.0;
    let mut wy = 0.0;
    let mut wz = 0.0;
    let mut w_total = 0.0;
    for (tp, &r) in transponders.iter().zip(ranges.iter()) {
        if r <= 0.0 {
            continue;
        }
        let w = 1.0 / r;
        wx += w * tp.get(0);
        wy += w * tp.get(1);
        wz += w * tp.get(2);
        w_total += w;
    }
    if w_total > 0.0 {
        Some(Vector::new(vec![wx / w_total, wy / w_total, wz / w_total]))
    } else {
        None
    }
}

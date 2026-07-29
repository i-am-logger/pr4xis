#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::angle::Angle;
use crate::formal::math::geometry::point::Point3;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::duration::Duration;

/// Sound speed profile computation.
///
/// Source: Mackenzie (1981), "Nine-term equation for sound speed in the oceans"
/// Compute sound speed in seawater using Mackenzie's equation.
///
/// temperature: water temperature, `Dimension::TEMPERATURE` (read via `unit::CELSIUS`)
/// salinity: practical salinity, `Dimension::SALINITY` (read via `unit::PSU`) —
///   UNESCO (1981), "Background papers and supporting data on the Practical
///   Salinity Scale 1978," UNESCO Technical Papers in Marine Science No. 37
/// depth: depth, `Dimension::LENGTH` (read via `unit::METER`)
///
/// Returns a [`Quantity`] tagged `Dimension::VELOCITY` (`unit::METER_PER_SECOND`).
pub fn mackenzie_sound_speed(
    temperature: Quantity,
    salinity: Quantity,
    depth: Quantity,
) -> Quantity {
    let t = temperature
        .in_unit(&unit::CELSIUS)
        .expect("TEMPERATURE quantity must convert to CELSIUS");
    let s = salinity
        .in_unit(&unit::PSU)
        .expect("SALINITY quantity must convert to PSU");
    let d = depth
        .in_unit(&unit::METER)
        .expect("LENGTH quantity must convert to METER");

    let speed = 1448.96 + 4.591 * t - 0.05304 * t * t
        + 0.000237_1 * t * t * t
        + 1.340 * (s - 35.0)
        + 0.01630 * d
        + 1.675e-7 * d * d
        - 0.01025 * t * (s - 35.0)
        - 7.139e-13 * t * d * d * d;
    Quantity::from_unit(speed, &unit::METER_PER_SECOND)
}

/// Compute range from two-way travel time and sound speed.
///
/// travel_time: two-way acoustic travel time, a genuine measured [`Duration`].
/// sound_speed: a [`Quantity`] tagged `Dimension::VELOCITY`, typically the
///   output of [`mackenzie_sound_speed`].
///
/// Returns a [`Quantity`] tagged `Dimension::LENGTH` (`unit::METER`).
pub fn range_from_travel_time(travel_time: Duration, sound_speed: Quantity) -> Quantity {
    Quantity::from_unit(
        sound_speed.value * travel_time.seconds() / 2.0,
        &unit::METER,
    )
}

/// USBL angle measurement (simplified).
#[derive(Debug, Clone)]
pub struct UsblFix {
    /// Slant range, tagged `Dimension::LENGTH` (`unit::METER`).
    pub range: Quantity,
    /// Bearing angle (an element of the circle group S¹).
    pub bearing: Angle,
    /// Depression angle (an element of the circle group S¹).
    pub depression: Angle,
}

impl UsblFix {
    /// Convert to a Cartesian position in the transceiver-local frame
    /// (x forward, y starboard, z down so positive depth is downward).
    pub fn to_cartesian(&self) -> Point3 {
        let cos_dep = self.depression.cos();
        let range = self.range.value;
        Point3::new(
            range * cos_dep * self.bearing.cos(),
            range * cos_dep * self.bearing.sin(),
            -range * self.depression.sin(), // positive depth is downward
        )
    }
}

/// LBL position fix from range measurements to multiple transponders.
///
/// transponders: transponder positions, each a 3-D point in a common local frame
/// ranges: measured ranges to each transponder, each a [`Quantity`] tagged
///   `Dimension::LENGTH`
///
/// Returns the estimated position (same local frame) using trilateration
/// (simplified least-squares).
pub fn lbl_trilateration(transponders: &[Point3], ranges: &[Quantity]) -> Option<Point3> {
    if transponders.len() < 3 || transponders.len() != ranges.len() {
        return None;
    }
    // Simplified: use centroid weighted by inverse range as approximation
    let mut wx = 0.0;
    let mut wy = 0.0;
    let mut wz = 0.0;
    let mut w_total = 0.0;
    for (tp, r) in transponders.iter().zip(ranges.iter()) {
        let r = r.value;
        if r <= 0.0 {
            continue;
        }
        let w = 1.0 / r;
        wx += w * tp.x;
        wy += w * tp.y;
        wz += w * tp.z;
        w_total += w;
    }
    if w_total > 0.0 {
        Some(Point3::new(wx / w_total, wy / w_total, wz / w_total))
    } else {
        None
    }
}

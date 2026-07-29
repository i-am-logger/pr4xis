#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::angle::Angle;
use crate::formal::math::geometry::point::Point3;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::duration::Duration;
use crate::natural::physics::kinematics::velocity::Velocity;

/// AUV navigation state and dead reckoning.
///
/// Source: Kinsey et al. (2006), "A Survey of Underwater Vehicle Navigation"
/// AUV navigation state (2D + depth).
#[derive(Debug, Clone)]
pub struct AuvState {
    /// Position: `x` = north (m), `y` = east (m), `z` = depth (m, positive
    /// downward) — the underwater analogue of
    /// [`crate::applied::navigation::imu::strapdown::NavState::position`].
    pub position: Point3,
    /// Heading (angle, radians from north clockwise).
    pub heading: Angle,
}

/// DVL velocity measurement in body frame.
#[derive(Debug, Clone)]
pub struct DvlMeasurement {
    /// Body-frame velocity: `vx` = forward, `vy` = starboard, `vz` = downward
    /// (m/s).
    pub velocity: Velocity,
    /// Whether bottom lock is achieved.
    pub bottom_lock: bool,
}

/// Depth sensor measurement.
#[derive(Debug, Clone)]
pub struct DepthMeasurement {
    /// Measured depth. Tagged `Dimension::LENGTH` (`unit::METER`).
    pub depth: Quantity,
}

/// Dead reckoning: propagate AUV state using DVL and compass.
pub fn dead_reckon(
    state: &AuvState,
    dvl: &DvlMeasurement,
    heading: Angle,
    dt: Duration,
) -> AuvState {
    let cos_h = heading.cos();
    let sin_h = heading.sin();
    let dt_s = dt.seconds();
    // Transform body-frame velocity to world frame
    let v_north = dvl.velocity.vx * cos_h - dvl.velocity.vy * sin_h;
    let v_east = dvl.velocity.vx * sin_h + dvl.velocity.vy * cos_h;

    AuvState {
        position: Point3::new(
            state.position.x + v_north * dt_s,
            state.position.y + v_east * dt_s,
            state.position.z + dvl.velocity.vz * dt_s,
        ),
        heading,
    }
}

/// Compute distance traveled between two states.
///
/// Returns a [`Quantity`] tagged `Dimension::LENGTH` (`unit::METER`).
pub fn distance_2d(a: &AuvState, b: &AuvState) -> Quantity {
    let dn = b.position.x - a.position.x;
    let de = b.position.y - a.position.y;
    Quantity::from_unit((dn * dn + de * de).sqrt(), &unit::METER)
}

/// Compute 3D distance between two states.
///
/// Returns a [`Quantity`] tagged `Dimension::LENGTH` (`unit::METER`).
pub fn distance_3d(a: &AuvState, b: &AuvState) -> Quantity {
    let dn = b.position.x - a.position.x;
    let de = b.position.y - a.position.y;
    let dd = b.position.z - a.position.z;
    Quantity::from_unit((dn * dn + de * de + dd * dd).sqrt(), &unit::METER)
}

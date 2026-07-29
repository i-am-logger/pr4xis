#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::vector_space::Vector;

/// Quaternion attitude kinematics.
///
/// Source: Wertz (1978), Chapter 16; Markley & Crassidis (2014),
///         *Fundamentals of Spacecraft Attitude Determination and Control*
///
/// The attitude representation itself is the shared SO(3) unit quaternion
/// (`formal::math::rotation::quaternion::Quaternion`) — the same type
/// `navigation/imu/strapdown.rs` uses for `NavState.attitude` — rather than
/// a second, hand-reinvented, bare-`f64`-field quaternion type. Attitude
/// kinematics in *this* module is the derivative/propagation machinery
/// (`propagate_attitude`) layered on top of that shared representation.
pub use crate::formal::math::rotation::quaternion::Quaternion;

/// Propagate quaternion forward in time by dt, given a body-frame angular
/// velocity `omega` (rad/s, length-3 [wx, wy, wz]).
///
/// Builds the incremental rotation `Δq = exp(ω dt / 2)` as a proper
/// axis-angle unit quaternion (`Quaternion::from_axis_angle`) and composes
/// it with the current attitude (`q * Δq`), then renormalizes — the same
/// pattern `navigation/imu/strapdown.rs::mechanize` uses for its attitude
/// update. This avoids ever constructing a non-unit "rate" quaternion,
/// which the shared `Quaternion` type's private fields (by design) do not
/// allow — only proper rotations are representable.
///
/// Source: Markley & Crassidis (2014) §2.9.1, quaternion propagation via
///         the truncated exponential map.
pub fn propagate_attitude(q: &Quaternion, omega: &Vector, dt: f64) -> Quaternion {
    let wx = omega.get(0);
    let wy = omega.get(1);
    let wz = omega.get(2);
    let angle = (wx * wx + wy * wy + wz * wz).sqrt() * dt;
    let dq = if angle > 1e-12 {
        let axis_norm = angle / dt;
        let axis = Vector::new(vec![wx / axis_norm, wy / axis_norm, wz / axis_norm]);
        Quaternion::from_axis_angle(&axis, angle)
    } else {
        Quaternion::identity()
    };
    q.multiply(&dq).normalize()
}

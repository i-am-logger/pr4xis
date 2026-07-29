#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::space::attitude::kinematics::{Quaternion, propagate_attitude};
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::natural::physics::kinematics::angular_velocity::AngularVelocity;

/// Attitude determination using TRIAD method.
///
/// Given two vector observations (v1, v2) in body frame and their known
/// directions (r1, r2) in reference frame, compute the attitude quaternion.
///
/// Source: Shuster & Oh (1981), "Three-Axis Attitude Determination from Vector Observations"
/// Normalize a 3D vector.
fn normalize(v: &Vector) -> Vector {
    let n = (v.get(0) * v.get(0) + v.get(1) * v.get(1) + v.get(2) * v.get(2)).sqrt();
    if n > 0.0 {
        Vector::new(vec![v.get(0) / n, v.get(1) / n, v.get(2) / n])
    } else {
        Vector::new(vec![0.0, 0.0, 0.0])
    }
}

/// Dot product of two 3D vectors.
fn dot(a: &Vector, b: &Vector) -> f64 {
    a.get(0) * b.get(0) + a.get(1) * b.get(1) + a.get(2) * b.get(2)
}

/// Simple attitude propagation state.
///
/// `angular_velocity` is the body-frame angular rate ω (rad/s).
#[derive(Debug, Clone)]
pub struct AttitudeState {
    pub quaternion: Quaternion,
    pub angular_velocity: AngularVelocity,
}

impl AttitudeState {
    /// Propagate attitude forward by dt seconds (constant angular velocity).
    pub fn propagate(&self, dt: f64) -> Self {
        // `propagate_attitude` composes the incremental rotation as an
        // axis-angle quaternion built from a length-3 `Vector` (body-frame
        // rate axes); convert the typed `AngularVelocity` into that form
        // at the boundary rather than leaking `Vector` into the state.
        let omega = Vector::new(vec![
            self.angular_velocity.x,
            self.angular_velocity.y,
            self.angular_velocity.z,
        ]);
        Self {
            quaternion: propagate_attitude(&self.quaternion, &omega, dt),
            angular_velocity: self.angular_velocity.clone(),
        }
    }
}

/// Compute the angle between two unit vectors.
///
/// Returns a [`Quantity`] tagged [`crate::formal::math::quantity::dimension::Dimension::ANGLE`]
/// (`unit::RADIAN`), never a bare `f64`.
pub fn angle_between(a: &Vector, b: &Vector) -> Quantity {
    let a = normalize(a);
    let b = normalize(b);
    let d = dot(&a, &b).clamp(-1.0, 1.0);
    Quantity::from_unit(d.acos(), &unit::RADIAN)
}

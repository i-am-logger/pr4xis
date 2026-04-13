use crate::applied::space::attitude::kinematics::{Quaternion, propagate_attitude};

/// Attitude determination using TRIAD method.
///
/// Given two vector observations (v1, v2) in body frame and their known
/// directions (r1, r2) in reference frame, compute the attitude quaternion.
///
/// Source: Shuster & Oh (1981), "Three-Axis Attitude Determination from Vector Observations"
/// Normalize a 3D vector.
fn normalize(v: &[f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if n > 0.0 {
        [v[0] / n, v[1] / n, v[2] / n]
    } else {
        [0.0, 0.0, 0.0]
    }
}

/// Dot product of two 3D vectors.
fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Simple attitude propagation state.
#[derive(Debug, Clone)]
pub struct AttitudeState {
    pub quaternion: Quaternion,
    pub angular_velocity: [f64; 3],
}

impl AttitudeState {
    /// Propagate attitude forward by dt seconds (constant angular velocity).
    pub fn propagate(&self, dt: f64) -> Self {
        Self {
            quaternion: propagate_attitude(&self.quaternion, &self.angular_velocity, dt),
            angular_velocity: self.angular_velocity,
        }
    }
}

/// Compute the angle between two unit vectors (rad).
pub fn angle_between(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let a = normalize(a);
    let b = normalize(b);
    let d = dot(&a, &b).clamp(-1.0, 1.0);
    d.acos()
}

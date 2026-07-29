#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::quantity::dimension::Dimension;
use crate::formal::math::quantity::value::Quantity;

/// Angular velocity vector: the rotational analog of [`super::velocity::Velocity`].
///
/// ω = dθ/dt (radians per second), a body-frame vector whose direction is
/// the instantaneous rotation axis and whose magnitude is the rotation
/// rate — the same Euler-vector representation used to build the
/// incremental attitude quaternion in
/// `applied::space::attitude::kinematics::propagate_attitude`.
///
/// Source: Goldstein, *Classical Mechanics* (2002), Chapter 4 (rotational
///         kinematics of a rigid body; ω is treated as the rotational
///         counterpart of the linear velocity of Chapter 1).
#[derive(Debug, Clone, PartialEq)]
pub struct AngularVelocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl AngularVelocity {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Rotation rate: |ω| = √(x² + y² + z²).
    pub fn magnitude(&self) -> Quantity {
        Quantity::new(
            (self.x * self.x + self.y * self.y + self.z * self.z).sqrt(),
            Dimension::ANGULAR_VELOCITY,
        )
    }

    /// Add two angular velocities (linear superposition of body rates).
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    /// Scale angular velocity.
    pub fn scale(&self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}

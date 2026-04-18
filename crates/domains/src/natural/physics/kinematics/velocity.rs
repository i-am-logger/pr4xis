#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Velocity vector: the first derivative of position with respect to time.
///
/// v = dx/dt (meters per second).
///
/// Velocity is a tangent vector to the trajectory curve in R³.
/// It lives in the tangent space at the current position.
///
/// Source: Goldstein, *Classical Mechanics* (2002), Chapter 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

impl Velocity {
    pub fn new(vx: f64, vy: f64, vz: f64) -> Self {
        Self { vx, vy, vz }
    }

    pub fn zero() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
        }
    }

    /// Speed: |v| = √(vx² + vy² + vz²).
    pub fn speed(&self) -> f64 {
        (self.vx * self.vx + self.vy * self.vy + self.vz * self.vz).sqrt()
    }

    /// Position change over duration dt: Δx = v * dt.
    /// First-order integration (Euler).
    pub fn displace(&self, dt: f64) -> crate::formal::math::geometry::vector::Vec3 {
        crate::formal::math::geometry::vector::Vec3::new(self.vx * dt, self.vy * dt, self.vz * dt)
    }

    /// Add two velocities (Galilean, non-relativistic).
    pub fn add(&self, other: &Self) -> Self {
        Self {
            vx: self.vx + other.vx,
            vy: self.vy + other.vy,
            vz: self.vz + other.vz,
        }
    }

    /// Scale velocity.
    pub fn scale(&self, s: f64) -> Self {
        Self {
            vx: self.vx * s,
            vy: self.vy * s,
            vz: self.vz * s,
        }
    }

    /// Estimate acceleration from two velocities and time interval.
    pub fn acceleration_to(
        &self,
        other: &Self,
        dt: f64,
    ) -> Option<crate::natural::physics::kinematics::acceleration::Acceleration> {
        if dt.abs() < 1e-15 {
            return None;
        }
        Some(
            crate::natural::physics::kinematics::acceleration::Acceleration {
                ax: (other.vx - self.vx) / dt,
                ay: (other.vy - self.vy) / dt,
                az: (other.vz - self.vz) / dt,
            },
        )
    }
}

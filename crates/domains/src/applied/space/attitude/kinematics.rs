#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Quaternion attitude kinematics.
///
/// Source: Wertz (1978), Chapter 16; Markley & Crassidis (2014),
///         *Fundamentals of Spacecraft Attitude Determination and Control*
/// A unit quaternion representing spacecraft attitude.
///
/// Convention: q = [q0, q1, q2, q3] where q0 is the scalar part.
#[derive(Debug, Clone)]
pub struct Quaternion {
    pub q0: f64, // scalar
    pub q1: f64, // vector x
    pub q2: f64, // vector y
    pub q3: f64, // vector z
}

impl Quaternion {
    /// Create a new quaternion and normalize it.
    pub fn new(q0: f64, q1: f64, q2: f64, q3: f64) -> Self {
        let mut q = Self { q0, q1, q2, q3 };
        q.normalize();
        q
    }

    /// Identity quaternion (no rotation).
    pub fn identity() -> Self {
        Self {
            q0: 1.0,
            q1: 0.0,
            q2: 0.0,
            q3: 0.0,
        }
    }

    /// Compute the norm.
    pub fn norm(&self) -> f64 {
        (self.q0 * self.q0 + self.q1 * self.q1 + self.q2 * self.q2 + self.q3 * self.q3).sqrt()
    }

    /// Normalize to unit quaternion.
    pub fn normalize(&mut self) {
        let n = self.norm();
        if n > 0.0 {
            self.q0 /= n;
            self.q1 /= n;
            self.q2 /= n;
            self.q3 /= n;
        }
    }

    /// Quaternion multiplication: self * other.
    pub fn multiply(&self, other: &Quaternion) -> Quaternion {
        Quaternion {
            q0: self.q0 * other.q0 - self.q1 * other.q1 - self.q2 * other.q2 - self.q3 * other.q3,
            q1: self.q0 * other.q1 + self.q1 * other.q0 + self.q2 * other.q3 - self.q3 * other.q2,
            q2: self.q0 * other.q2 - self.q1 * other.q3 + self.q2 * other.q0 + self.q3 * other.q1,
            q3: self.q0 * other.q3 + self.q1 * other.q2 - self.q2 * other.q1 + self.q3 * other.q0,
        }
    }

    /// Conjugate (inverse for unit quaternions).
    pub fn conjugate(&self) -> Quaternion {
        Quaternion {
            q0: self.q0,
            q1: -self.q1,
            q2: -self.q2,
            q3: -self.q3,
        }
    }
}

/// Quaternion rate equation: dq/dt = 0.5 * Omega(omega) * q
///
/// omega: angular velocity vector [wx, wy, wz] in rad/s (body frame)
/// Returns the quaternion time derivative.
pub fn quaternion_rate(q: &Quaternion, omega: &[f64; 3]) -> Quaternion {
    let wx = omega[0];
    let wy = omega[1];
    let wz = omega[2];
    Quaternion {
        q0: 0.5 * (-wx * q.q1 - wy * q.q2 - wz * q.q3),
        q1: 0.5 * (wx * q.q0 + wz * q.q2 - wy * q.q3),
        q2: 0.5 * (wy * q.q0 - wz * q.q1 + wx * q.q3),
        q3: 0.5 * (wz * q.q0 + wy * q.q1 - wx * q.q2),
    }
}

/// Propagate quaternion forward in time using first-order integration.
///
/// dt: time step in seconds
/// omega: angular velocity vector [wx, wy, wz] in rad/s
pub fn propagate_attitude(q: &Quaternion, omega: &[f64; 3], dt: f64) -> Quaternion {
    let dq = quaternion_rate(q, omega);
    let mut q_new = Quaternion {
        q0: q.q0 + dq.q0 * dt,
        q1: q.q1 + dq.q1 * dt,
        q2: q.q2 + dq.q2 * dt,
        q3: q.q3 + dq.q3 * dt,
    };
    q_new.normalize();
    q_new
}

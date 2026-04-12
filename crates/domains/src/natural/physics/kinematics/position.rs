use crate::formal::math::geometry::point::Point3;
use crate::formal::math::temporal::instant::Instant;

/// A position stamped with time — the fundamental kinematic entity.
///
/// Position is geometry (where) + time (when).
/// Velocity is the derivative of position with respect to time: v = dx/dt.
///
/// Source: Goldstein, *Classical Mechanics* (2002), Chapter 1.
#[derive(Debug, Clone, PartialEq)]
pub struct TimedPosition {
    pub position: Point3,
    pub time: Instant,
}

impl TimedPosition {
    pub fn new(position: Point3, time: Instant) -> Self {
        Self { position, time }
    }

    /// Estimate velocity from two timed positions: v ≈ Δx / Δt.
    /// Returns None if same timestamp.
    pub fn velocity_to(
        &self,
        other: &Self,
    ) -> Option<crate::natural::physics::kinematics::velocity::Velocity> {
        let dt = self.time.duration_to(&other.time)?;
        let dt_s = dt.seconds();
        if dt_s.abs() < 1e-15 {
            return None;
        }
        let dx = self.position.vector_to(&other.position);
        Some(crate::natural::physics::kinematics::velocity::Velocity {
            vx: dx.x / dt_s,
            vy: dx.y / dt_s,
            vz: dx.z / dt_s,
        })
    }
}

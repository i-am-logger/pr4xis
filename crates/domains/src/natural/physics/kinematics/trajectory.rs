use crate::formal::math::geometry::point::Point3;

use crate::natural::physics::kinematics::acceleration::Acceleration;
use crate::natural::physics::kinematics::position::TimedPosition;
use crate::natural::physics::kinematics::velocity::Velocity;

/// A kinematic state: position + velocity + acceleration at an instant.
///
/// This is the complete first-order description of a particle's motion.
/// Higher derivatives (jerk, snap) exist but are rarely needed in
/// sensor fusion except for high-dynamics tracking (maneuvering targets).
#[derive(Debug, Clone, PartialEq)]
pub struct KinematicState {
    pub position: Point3,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
}

impl KinematicState {
    pub fn stationary(position: Point3) -> Self {
        Self {
            position,
            velocity: Velocity::zero(),
            acceleration: Acceleration::zero(),
        }
    }

    /// Propagate state forward by dt using constant acceleration model.
    ///
    /// x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
    /// v(t+dt) = v(t) + a(t)*dt
    /// a(t+dt) = a(t)  (constant acceleration assumption)
    ///
    /// This is the kinematic equation from classical mechanics.
    pub fn propagate(&self, dt: f64) -> Self {
        let new_pos = Point3::new(
            self.position.x + self.velocity.vx * dt + 0.5 * self.acceleration.ax * dt * dt,
            self.position.y + self.velocity.vy * dt + 0.5 * self.acceleration.ay * dt * dt,
            self.position.z + self.velocity.vz * dt + 0.5 * self.acceleration.az * dt * dt,
        );
        let dv = self.acceleration.velocity_change(dt);
        let new_vel = self.velocity.add(&dv);

        Self {
            position: new_pos,
            velocity: new_vel,
            acceleration: self.acceleration.clone(),
        }
    }

    /// Speed at current state.
    pub fn speed(&self) -> f64 {
        self.velocity.speed()
    }

    /// Distance traveled over dt (approximate, using average velocity).
    pub fn distance_traveled(&self, dt: f64) -> f64 {
        let future = self.propagate(dt);
        self.position.distance_to(&future.position)
    }
}

/// A trajectory: a sequence of timed positions.
///
/// In differential geometry, a trajectory is a curve γ: [t0, t1] → R³
/// parameterized by time. The velocity is the tangent vector γ'(t).
#[derive(Debug, Clone)]
pub struct Trajectory {
    pub points: Vec<TimedPosition>,
}

impl Trajectory {
    pub fn new(points: Vec<TimedPosition>) -> Self {
        Self { points }
    }

    /// Total path length (sum of segment distances).
    pub fn path_length(&self) -> f64 {
        self.points
            .windows(2)
            .map(|w| w[0].position.distance_to(&w[1].position))
            .sum()
    }

    /// Total duration from first to last point.
    pub fn duration(&self) -> Option<f64> {
        if self.points.len() < 2 {
            return None;
        }
        let first = &self.points[0].time;
        let last = &self.points[self.points.len() - 1].time;
        first.duration_to(last).map(|d| d.seconds())
    }

    /// Average speed over the trajectory.
    pub fn average_speed(&self) -> Option<f64> {
        let d = self.duration()?;
        if d.abs() < 1e-15 {
            return None;
        }
        Some(self.path_length() / d)
    }

    /// Number of waypoints.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Is the trajectory empty?
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

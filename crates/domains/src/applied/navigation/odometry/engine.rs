#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

/// Odometry pose: 2D position + heading.
#[derive(Debug, Clone, PartialEq)]
pub struct OdometryPose {
    /// X position (meters).
    pub x: f64,
    /// Y position (meters).
    pub y: f64,
    /// Heading angle (radians, 0 = forward/north).
    pub heading: f64,
}

impl OdometryPose {
    /// Create a new pose.
    pub fn new(x: f64, y: f64, heading: f64) -> Self {
        Self { x, y, heading }
    }

    /// Origin pose.
    pub fn origin() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            heading: 0.0,
        }
    }

    /// Euclidean distance from origin.
    pub fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// Odometry situation: current dead reckoning state.
#[derive(Debug, Clone, PartialEq)]
pub struct OdometrySituation {
    /// Current pose.
    pub pose: OdometryPose,
    /// Forward velocity (m/s).
    pub velocity: f64,
    /// Accumulated distance traveled (meters).
    pub distance_traveled: f64,
    /// Estimated position error (meters, 1-sigma).
    pub estimated_error: f64,
    /// Drift rate (fraction of distance).
    pub drift_rate: f64,
    /// Step counter.
    pub step: usize,
}

impl Situation for OdometrySituation {
    fn describe(&self) -> String {
        format!(
            "Odom step={}, pos=({:.2},{:.2}), heading={:.2} deg, dist={:.1}m, err={:.2}m",
            self.step,
            self.pose.x,
            self.pose.y,
            self.pose.heading.to_degrees(),
            self.distance_traveled,
            self.estimated_error,
        )
    }

    fn is_terminal(&self) -> bool {
        false
    }
}

/// Odometry action: dead reckoning updates.
#[derive(Debug, Clone)]
pub enum OdometryAction {
    /// Drive forward with given velocity and heading rate.
    DriveForward {
        /// Forward velocity (m/s).
        velocity: f64,
        /// Heading rate (rad/s).
        heading_rate: f64,
        /// Time step (seconds).
        dt: f64,
    },
    /// Wheel encoder tick: distance traveled by each wheel.
    WheelTick {
        /// Left wheel distance (meters).
        left: f64,
        /// Right wheel distance (meters).
        right: f64,
        /// Wheel base width (meters).
        wheel_base: f64,
    },
}

impl Action for OdometryAction {
    type Sit = OdometrySituation;

    fn describe(&self) -> String {
        match self {
            OdometryAction::DriveForward { velocity, dt, .. } => {
                format!("drive v={:.2}m/s dt={:.4}s", velocity, dt)
            }
            OdometryAction::WheelTick { left, right, .. } => {
                format!("wheel tick L={:.4}m R={:.4}m", left, right)
            }
        }
    }
}

/// Apply an odometry action: dead reckoning integration.
///
/// Source: Thrun, Burgard & Fox (2005) Section 5.3.
pub fn apply_odometry(
    situation: &OdometrySituation,
    action: &OdometryAction,
) -> Result<OdometrySituation, String> {
    match action {
        OdometryAction::DriveForward {
            velocity,
            heading_rate,
            dt,
        } => {
            if *dt < 0.0 {
                return Err("dt must be non-negative".into());
            }
            let distance = velocity * dt;
            let new_heading = situation.pose.heading + heading_rate * dt;
            // Use mid-heading for better integration accuracy
            let mid_heading = situation.pose.heading + heading_rate * dt * 0.5;
            let new_x = situation.pose.x + distance * mid_heading.cos();
            let new_y = situation.pose.y + distance * mid_heading.sin();

            let new_distance = situation.distance_traveled + distance.abs();
            let new_error = situation.drift_rate * new_distance;

            Ok(OdometrySituation {
                pose: OdometryPose::new(new_x, new_y, new_heading),
                velocity: *velocity,
                distance_traveled: new_distance,
                estimated_error: new_error,
                drift_rate: situation.drift_rate,
                step: situation.step + 1,
            })
        }
        OdometryAction::WheelTick {
            left,
            right,
            wheel_base,
        } => {
            if *wheel_base <= 0.0 {
                return Err("wheel base must be positive".into());
            }
            // Differential drive model
            let distance = (left + right) / 2.0;
            let dtheta = (right - left) / wheel_base;

            let mid_heading = situation.pose.heading + dtheta * 0.5;
            let new_x = situation.pose.x + distance * mid_heading.cos();
            let new_y = situation.pose.y + distance * mid_heading.sin();
            let new_heading = situation.pose.heading + dtheta;

            let new_distance = situation.distance_traveled + distance.abs();
            let new_error = situation.drift_rate * new_distance;

            Ok(OdometrySituation {
                pose: OdometryPose::new(new_x, new_y, new_heading),
                velocity: 0.0, // unknown without dt
                distance_traveled: new_distance,
                estimated_error: new_error,
                drift_rate: situation.drift_rate,
                step: situation.step + 1,
            })
        }
    }
}

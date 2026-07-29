#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

use crate::formal::math::angle::Angle;
use crate::formal::math::geometry::point::Point2;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::duration::Duration;

/// Odometry pose: 2D position + heading.
#[derive(Debug, Clone, PartialEq)]
pub struct OdometryPose {
    /// Position in the 2D navigation plane.
    pub position: Point2,
    /// Heading angle (0 = forward/north), an element of the circle group S¹.
    pub heading: Angle,
}

impl OdometryPose {
    /// Create a new pose.
    pub fn new(x: f64, y: f64, heading: Angle) -> Self {
        Self {
            position: Point2::new(x, y),
            heading,
        }
    }

    /// Origin pose.
    pub fn origin() -> Self {
        Self {
            position: Point2::origin(),
            heading: Angle::ZERO,
        }
    }

    /// Euclidean distance from origin.
    pub fn distance_from_origin(&self) -> Quantity {
        let distance = self.position.distance_to(&Point2::origin());
        Quantity::from_unit(distance.value, &unit::METER)
    }
}

/// Odometry situation: current dead reckoning state.
#[derive(Debug, Clone, PartialEq)]
pub struct OdometrySituation {
    /// Current pose.
    pub pose: OdometryPose,
    /// Forward velocity.
    pub velocity: Quantity,
    /// Accumulated distance traveled.
    pub distance_traveled: Quantity,
    /// Estimated position error (1-sigma).
    pub estimated_error: Quantity,
    /// Drift rate (fraction of distance).
    pub drift_rate: f64,
    /// Step counter.
    pub step: usize,
}

impl Situation for OdometrySituation {}

/// Odometry action: dead reckoning updates.
#[derive(Debug, Clone)]
pub enum OdometryAction {
    /// Drive forward with given velocity and heading rate.
    DriveForward {
        /// Forward velocity.
        velocity: Quantity,
        /// Heading rate.
        heading_rate: Quantity,
        /// Time step.
        dt: Duration,
    },
    /// Wheel encoder tick: distance traveled by each wheel.
    WheelTick {
        /// Left wheel distance.
        left: Quantity,
        /// Right wheel distance.
        right: Quantity,
        /// Wheel base width.
        wheel_base: Quantity,
    },
}

impl Action for OdometryAction {
    type Sit = OdometrySituation;
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
            if dt.is_negative() {
                return Err("dt must be non-negative".into());
            }
            let dt_secs = dt.seconds();
            let distance = velocity.value * dt_secs;
            let new_heading = situation.pose.heading.radians() + heading_rate.value * dt_secs;
            // Use mid-heading for better integration accuracy
            let mid_heading = situation.pose.heading.radians() + heading_rate.value * dt_secs * 0.5;
            let new_x = situation.pose.position.x + distance * mid_heading.cos();
            let new_y = situation.pose.position.y + distance * mid_heading.sin();

            let new_distance = situation.distance_traveled.value + distance.abs();
            let new_error = situation.drift_rate * new_distance;

            Ok(OdometrySituation {
                pose: OdometryPose::new(new_x, new_y, Angle::from_radians(new_heading)),
                velocity: velocity.clone(),
                distance_traveled: Quantity::from_unit(new_distance, &unit::METER),
                estimated_error: Quantity::from_unit(new_error, &unit::METER),
                drift_rate: situation.drift_rate,
                step: situation.step + 1,
            })
        }
        OdometryAction::WheelTick {
            left,
            right,
            wheel_base,
        } => {
            if wheel_base.value <= 0.0 {
                return Err("wheel base must be positive".into());
            }
            // Differential drive model
            let distance = (left.value + right.value) / 2.0;
            let dtheta = (right.value - left.value) / wheel_base.value;

            let mid_heading = situation.pose.heading.radians() + dtheta * 0.5;
            let new_x = situation.pose.position.x + distance * mid_heading.cos();
            let new_y = situation.pose.position.y + distance * mid_heading.sin();
            let new_heading = situation.pose.heading.radians() + dtheta;

            let new_distance = situation.distance_traveled.value + distance.abs();
            let new_error = situation.drift_rate * new_distance;

            Ok(OdometrySituation {
                pose: OdometryPose::new(new_x, new_y, Angle::from_radians(new_heading)),
                // unknown without dt
                velocity: Quantity::from_unit(0.0, &unit::METER_PER_SECOND),
                distance_traveled: Quantity::from_unit(new_distance, &unit::METER),
                estimated_error: Quantity::from_unit(new_error, &unit::METER),
                drift_rate: situation.drift_rate,
                step: situation.step + 1,
            })
        }
    }
}

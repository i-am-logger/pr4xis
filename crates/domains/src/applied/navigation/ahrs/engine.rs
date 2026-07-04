#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

use crate::formal::math::angle::Angle;
use crate::formal::math::linear_algebra::vector_space::Vector;

/// AHRS attitude estimate (Euler angles, dimension ANGLE).
#[derive(Debug, Clone, PartialEq)]
pub struct AttitudeEstimate {
    /// Roll angle (ANGLE), rotation about forward axis.
    pub roll: Angle,
    /// Pitch angle (ANGLE), rotation about right axis.
    pub pitch: Angle,
    /// Yaw angle (ANGLE), rotation about down axis.
    pub yaw: Angle,
}

impl AttitudeEstimate {
    /// Create a new attitude estimate from radian components.
    pub fn new(roll: f64, pitch: f64, yaw: f64) -> Self {
        Self {
            roll: Angle::from_radians(roll),
            pitch: Angle::from_radians(pitch),
            yaw: Angle::from_radians(yaw),
        }
    }

    /// Zero attitude (level, facing north).
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

/// AHRS situation: current attitude estimate and filter state.
#[derive(Debug, Clone, PartialEq)]
pub struct AhrsSituation {
    /// Current attitude estimate.
    pub attitude: AttitudeEstimate,
    /// Complementary filter coefficient alpha (0..1).
    /// Higher alpha trusts the gyro more.
    pub alpha: f64,
    /// Step counter.
    pub step: usize,
    /// Total elapsed time.
    pub total_time: f64,
}

impl Situation for AhrsSituation {}

/// AHRS action: sensor updates.
#[derive(Debug, Clone)]
pub enum AhrsAction {
    /// Gyroscope angular rate update.
    GyroUpdate {
        /// Angular rates [roll_rate, pitch_rate, yaw_rate] in rad/s (Body frame).
        angular_rate: Vector,
        /// Time step in seconds.
        dt: f64,
    },
    /// Accelerometer correction (determines roll and pitch).
    AccelCorrection {
        /// Accelerometer reading [ax, ay, az] in m/s^2 (Body frame).
        accel: Vector,
    },
    /// Magnetometer correction (determines yaw/heading).
    MagCorrection {
        /// Magnetometer reading [mx, my, mz] in Tesla (Body frame).
        mag: Vector,
    },
}

impl Action for AhrsAction {
    type Sit = AhrsSituation;
}

/// Apply an AHRS action to the current situation.
///
/// Implements a simple complementary filter:
///   attitude = alpha * (attitude + gyro*dt) + (1-alpha) * accel_attitude
///
/// Source: Madgwick (2010), basic complementary filter.
pub fn apply_ahrs(situation: &AhrsSituation, action: &AhrsAction) -> Result<AhrsSituation, String> {
    match action {
        AhrsAction::GyroUpdate { angular_rate, dt } => {
            if *dt < 0.0 {
                return Err("dt must be non-negative".into());
            }
            // Integrate gyro: attitude += angular_rate * dt
            // Pure gyro integration — alpha blending is only applied in AccelCorrection/MagCorrection
            let new_roll = situation.attitude.roll.radians() + angular_rate.get(0) * dt;
            let new_pitch = situation.attitude.pitch.radians() + angular_rate.get(1) * dt;
            let new_yaw = situation.attitude.yaw.radians() + angular_rate.get(2) * dt;

            Ok(AhrsSituation {
                attitude: AttitudeEstimate::new(new_roll, new_pitch, new_yaw),
                alpha: situation.alpha,
                step: situation.step + 1,
                total_time: situation.total_time + dt,
            })
        }
        AhrsAction::AccelCorrection { accel } => {
            let norm = (accel.get(0) * accel.get(0)
                + accel.get(1) * accel.get(1)
                + accel.get(2) * accel.get(2))
            .sqrt();
            if norm < 1e-6 {
                return Err("accelerometer reading too small (near zero-g)".into());
            }

            // Compute roll and pitch from accelerometer
            // roll = atan2(ay, -az), pitch = atan2(-ax, sqrt(ay^2 + az^2))
            let accel_roll = accel.get(1).atan2(-accel.get(2));
            let accel_pitch = (-accel.get(0))
                .atan2((accel.get(1) * accel.get(1) + accel.get(2) * accel.get(2)).sqrt());

            let alpha = situation.alpha;
            // Complementary filter: blend gyro-integrated attitude with accel reference
            let new_roll = alpha * situation.attitude.roll.radians() + (1.0 - alpha) * accel_roll;
            let new_pitch =
                alpha * situation.attitude.pitch.radians() + (1.0 - alpha) * accel_pitch;

            Ok(AhrsSituation {
                attitude: AttitudeEstimate::new(
                    new_roll,
                    new_pitch,
                    situation.attitude.yaw.radians(),
                ),
                alpha: situation.alpha,
                step: situation.step + 1,
                total_time: situation.total_time,
            })
        }
        AhrsAction::MagCorrection { mag } => {
            let norm =
                (mag.get(0) * mag.get(0) + mag.get(1) * mag.get(1) + mag.get(2) * mag.get(2))
                    .sqrt();
            if norm < 1e-12 {
                return Err("magnetometer reading too small".into());
            }

            // Compute heading from magnetometer (assuming level attitude)
            // heading = atan2(-my, mx)
            let mag_heading = (-mag.get(1)).atan2(mag.get(0));

            let alpha = situation.alpha;
            let new_yaw = alpha * situation.attitude.yaw.radians() + (1.0 - alpha) * mag_heading;

            Ok(AhrsSituation {
                attitude: AttitudeEstimate::new(
                    situation.attitude.roll.radians(),
                    situation.attitude.pitch.radians(),
                    new_yaw,
                ),
                alpha: situation.alpha,
                step: situation.step + 1,
                total_time: situation.total_time,
            })
        }
    }
}

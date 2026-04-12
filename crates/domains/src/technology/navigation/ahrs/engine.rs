use pr4xis::engine::{Action, Situation};

/// AHRS attitude estimate (Euler angles in radians).
#[derive(Debug, Clone, PartialEq)]
pub struct AttitudeEstimate {
    /// Roll angle (radians), rotation about forward axis.
    pub roll: f64,
    /// Pitch angle (radians), rotation about right axis.
    pub pitch: f64,
    /// Yaw angle (radians), rotation about down axis.
    pub yaw: f64,
}

impl AttitudeEstimate {
    /// Create a new attitude estimate.
    pub fn new(roll: f64, pitch: f64, yaw: f64) -> Self {
        Self { roll, pitch, yaw }
    }

    /// Zero attitude (level, facing north).
    pub fn zero() -> Self {
        Self {
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
        }
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

impl Situation for AhrsSituation {
    fn describe(&self) -> String {
        format!(
            "AHRS step={}, t={:.3}s, roll={:.2} pitch={:.2} yaw={:.2} deg",
            self.step,
            self.total_time,
            self.attitude.roll.to_degrees(),
            self.attitude.pitch.to_degrees(),
            self.attitude.yaw.to_degrees(),
        )
    }

    fn is_terminal(&self) -> bool {
        false
    }
}

/// AHRS action: sensor updates.
#[derive(Debug, Clone)]
pub enum AhrsAction {
    /// Gyroscope angular rate update.
    GyroUpdate {
        /// Angular rates [roll_rate, pitch_rate, yaw_rate] in rad/s.
        angular_rate: [f64; 3],
        /// Time step in seconds.
        dt: f64,
    },
    /// Accelerometer correction (determines roll and pitch).
    AccelCorrection {
        /// Accelerometer reading [ax, ay, az] in m/s^2.
        accel: [f64; 3],
    },
    /// Magnetometer correction (determines yaw/heading).
    MagCorrection {
        /// Magnetometer reading [mx, my, mz] in Tesla.
        mag: [f64; 3],
    },
}

impl Action for AhrsAction {
    type Sit = AhrsSituation;

    fn describe(&self) -> String {
        match self {
            AhrsAction::GyroUpdate { dt, .. } => format!("gyro update dt={:.4}s", dt),
            AhrsAction::AccelCorrection { .. } => "accel correction".to_string(),
            AhrsAction::MagCorrection { .. } => "mag correction".to_string(),
        }
    }
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
            let new_roll = situation.attitude.roll + angular_rate[0] * dt;
            let new_pitch = situation.attitude.pitch + angular_rate[1] * dt;
            let new_yaw = situation.attitude.yaw + angular_rate[2] * dt;

            Ok(AhrsSituation {
                attitude: AttitudeEstimate::new(new_roll, new_pitch, new_yaw),
                alpha: situation.alpha,
                step: situation.step + 1,
                total_time: situation.total_time + dt,
            })
        }
        AhrsAction::AccelCorrection { accel } => {
            let norm = (accel[0] * accel[0] + accel[1] * accel[1] + accel[2] * accel[2]).sqrt();
            if norm < 1e-6 {
                return Err("accelerometer reading too small (near zero-g)".into());
            }

            // Compute roll and pitch from accelerometer
            // roll = atan2(ay, -az), pitch = atan2(-ax, sqrt(ay^2 + az^2))
            let accel_roll = accel[1].atan2(-accel[2]);
            let accel_pitch = (-accel[0]).atan2((accel[1] * accel[1] + accel[2] * accel[2]).sqrt());

            let alpha = situation.alpha;
            // Complementary filter: blend gyro-integrated attitude with accel reference
            let new_roll = alpha * situation.attitude.roll + (1.0 - alpha) * accel_roll;
            let new_pitch = alpha * situation.attitude.pitch + (1.0 - alpha) * accel_pitch;

            Ok(AhrsSituation {
                attitude: AttitudeEstimate::new(new_roll, new_pitch, situation.attitude.yaw),
                alpha: situation.alpha,
                step: situation.step + 1,
                total_time: situation.total_time,
            })
        }
        AhrsAction::MagCorrection { mag } => {
            let norm = (mag[0] * mag[0] + mag[1] * mag[1] + mag[2] * mag[2]).sqrt();
            if norm < 1e-12 {
                return Err("magnetometer reading too small".into());
            }

            // Compute heading from magnetometer (assuming level attitude)
            // heading = atan2(-my, mx)
            let mag_heading = (-mag[1]).atan2(mag[0]);

            let alpha = situation.alpha;
            let new_yaw = alpha * situation.attitude.yaw + (1.0 - alpha) * mag_heading;

            Ok(AhrsSituation {
                attitude: AttitudeEstimate::new(
                    situation.attitude.roll,
                    situation.attitude.pitch,
                    new_yaw,
                ),
                alpha: situation.alpha,
                step: situation.step + 1,
                total_time: situation.total_time,
            })
        }
    }
}

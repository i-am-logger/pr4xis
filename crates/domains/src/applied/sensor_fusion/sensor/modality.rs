use pr4xis::category::Entity;

/// Sensor type taxonomy.
///
/// Classified by modality as defined in the sensor fusion literature:
/// - Proprioceptive: measures internal state (IMU, odometry)
/// - Exteroceptive: measures external environment (GNSS, LiDAR, camera, radar)
/// - Active: emits energy and measures return (radar, LiDAR, sonar)
/// - Passive: measures ambient energy (camera, GNSS, magnetometer)
///
/// Source: Groves (2013), Chapter 1; Bar-Shalom et al. (2001), Chapter 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensorType {
    // Abstract
    Sensor,
    ProprioceptiveSensor,
    ExteroceptiveSensor,
    ActiveSensor,
    PassiveSensor,

    // Inertial (proprioceptive)
    Accelerometer,
    Gyroscope,
    Magnetometer,

    // Position (exteroceptive, passive)
    GnssReceiver,
    StarTracker,

    // Range (exteroceptive, active)
    Radar,
    LiDAR,
    Sonar,

    // Vision (exteroceptive, passive)
    Camera,
    InfraredCamera,
    DepthCamera,

    // Pressure (proprioceptive)
    Barometer,
    DepthSensor,

    // Velocity (exteroceptive, active)
    DopplerVelocityLog,

    // Composite
    IMU,
    AHRS,
    INS,
}

impl Entity for SensorType {
    fn variants() -> Vec<Self> {
        vec![
            Self::Sensor,
            Self::ProprioceptiveSensor,
            Self::ExteroceptiveSensor,
            Self::ActiveSensor,
            Self::PassiveSensor,
            Self::Accelerometer,
            Self::Gyroscope,
            Self::Magnetometer,
            Self::GnssReceiver,
            Self::StarTracker,
            Self::Radar,
            Self::LiDAR,
            Self::Sonar,
            Self::Camera,
            Self::InfraredCamera,
            Self::DepthCamera,
            Self::Barometer,
            Self::DepthSensor,
            Self::DopplerVelocityLog,
            Self::IMU,
            Self::AHRS,
            Self::INS,
        ]
    }
}

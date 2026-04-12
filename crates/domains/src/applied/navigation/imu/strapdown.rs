use crate::formal::math::geometry::point::Point3;
use crate::formal::math::rotation::quaternion::Quaternion;
use crate::formal::math::signal_processing::filter::FirstOrderLowPass;
use crate::natural::physics::kinematics::acceleration::Acceleration;
use crate::natural::physics::kinematics::velocity::Velocity;

/// Strapdown mechanization: integrating body-frame IMU measurements
/// into navigation frame position, velocity, and attitude.
///
/// The strapdown equations (simplified, NED frame):
///   attitude: q(t+dt) = q(t) * Δq(ω, dt)
///   velocity: v(t+dt) = v(t) + (R(q) * f + g) * dt
///   position: p(t+dt) = p(t) + v(t) * dt + 0.5 * a * dt²
///
/// where:
///   q = attitude quaternion (body → nav)
///   ω = gyroscope angular rate (body frame)
///   f = accelerometer specific force (body frame)
///   g = gravity vector (nav frame)
///   R(q) = rotation matrix from body to nav
///
/// Source: Savage, P.G. (1998). "Strapdown Inertial Navigation Integration
///         Algorithm Design." Journal of Guidance, Control, and Dynamics.
///         Groves (2013), Chapter 5.
/// Navigation state: position + velocity + attitude.
#[derive(Debug, Clone, PartialEq)]
pub struct NavState {
    pub position: Point3,
    pub velocity: Velocity,
    pub attitude: Quaternion,
}

/// IMU sample: specific force + angular rate at a time instant.
#[derive(Debug, Clone)]
pub struct ImuSample {
    /// Specific force in body frame (m/s²). f = a - g.
    pub specific_force: [f64; 3],
    /// Angular rate in body frame (rad/s).
    pub angular_rate: [f64; 3],
    /// Time step (seconds).
    pub dt: f64,
}

/// Gravity vector in NED frame, from the quantity ontology.
pub fn gravity_ned() -> [f64; 3] {
    [
        0.0,
        0.0,
        crate::formal::math::quantity::constants::standard_gravity().value,
    ]
}

/// Filter raw IMU measurements to remove high-frequency noise.
/// Uses first-order low-pass from the signal processing ontology.
///
/// Each axis of specific force and angular rate is filtered independently.
/// The cutoff frequency should be chosen based on the vehicle dynamics
/// bandwidth — typically 10-50 Hz for ground vehicles, higher for missiles.
///
/// Source: Titterton & Weston (2004), Chapter 3.5 (sensor preprocessing).
pub fn filter_imu_sample(
    sample: &ImuSample,
    accel_filter: &mut [FirstOrderLowPass; 3],
    gyro_filter: &mut [FirstOrderLowPass; 3],
) -> ImuSample {
    ImuSample {
        specific_force: [
            accel_filter[0].update(sample.specific_force[0]),
            accel_filter[1].update(sample.specific_force[1]),
            accel_filter[2].update(sample.specific_force[2]),
        ],
        angular_rate: [
            gyro_filter[0].update(sample.angular_rate[0]),
            gyro_filter[1].update(sample.angular_rate[1]),
            gyro_filter[2].update(sample.angular_rate[2]),
        ],
        dt: sample.dt,
    }
}

/// Perform one strapdown mechanization step.
///
/// This is the core of inertial navigation: integrate gyro and accel
/// measurements to update position, velocity, and attitude.
pub fn mechanize(state: &NavState, sample: &ImuSample) -> NavState {
    let dt = sample.dt;

    // 1. Attitude update: integrate angular rate
    //    Δq = Quaternion from angular rate * dt
    let omega = sample.angular_rate;
    let angle = (omega[0] * omega[0] + omega[1] * omega[1] + omega[2] * omega[2]).sqrt() * dt;
    let dq = if angle > 1e-12 {
        let axis_norm = angle / dt;
        let axis = [
            omega[0] / axis_norm,
            omega[1] / axis_norm,
            omega[2] / axis_norm,
        ];
        Quaternion::from_axis_angle(axis, angle)
    } else {
        Quaternion::identity()
    };
    let new_attitude = state.attitude.multiply(&dq).normalize();

    // 2. Velocity update: rotate specific force to nav frame, add gravity
    let f_nav = state.attitude.rotate_vector(sample.specific_force);
    let g = gravity_ned();
    let accel = Acceleration::new(f_nav[0] + g[0], f_nav[1] + g[1], f_nav[2] + g[2]);
    let dv = accel.velocity_change(dt);
    let new_velocity = state.velocity.add(&dv);

    // 3. Position update: integrate velocity
    let displacement = state.velocity.displace(dt);
    let new_position = state.position.translate(&displacement);

    NavState {
        position: new_position,
        velocity: new_velocity,
        attitude: new_attitude,
    }
}

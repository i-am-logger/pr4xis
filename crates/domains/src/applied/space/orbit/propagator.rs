#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::vector_space::Vector;

/// Two-body orbit propagation.
///
/// Source: Vallado (2013), Chapter 2.
/// Earth gravitational parameter from quantity ontology (km³/s²).
pub fn mu_earth_km3s2() -> f64 {
    // Convert from SI (m³/s²) to km³/s² for orbital mechanics convention
    crate::formal::math::quantity::constants::mu_earth().value / 1e9
}

/// Orbital state vector (position + velocity in ECI).
#[derive(Debug, Clone)]
pub struct OrbitalState {
    /// Position (x, y, z) in km, expressed in the Earth-centred inertial (ECI) frame.
    pub position: Vector,
    /// Velocity (vx, vy, vz) in km/s, expressed in the Earth-centred inertial (ECI) frame.
    pub velocity: Vector,
}

impl OrbitalState {
    /// Compute the orbital radius (distance from central body).
    pub fn radius(&self) -> f64 {
        self.position.norm()
    }

    /// Compute the speed.
    pub fn speed(&self) -> f64 {
        self.velocity.norm()
    }

    /// Compute specific orbital energy (vis-viva).
    pub fn specific_energy(&self, mu: f64) -> f64 {
        self.speed().powi(2) / 2.0 - mu / self.radius()
    }

    /// Compute semi-major axis from vis-viva equation.
    pub fn semi_major_axis(&self, mu: f64) -> f64 {
        -mu / (2.0 * self.specific_energy(mu))
    }
}

/// Two-body gravitational acceleration in the Earth-centred inertial (ECI) frame.
pub fn two_body_acceleration(position: &Vector, mu: f64) -> Vector {
    let r2 = position.get(0).powi(2) + position.get(1).powi(2) + position.get(2).powi(2);
    let r3 = r2 * r2.sqrt();
    Vector::new(vec![
        -mu * position.get(0) / r3,
        -mu * position.get(1) / r3,
        -mu * position.get(2) / r3,
    ])
}

/// Propagate orbital state using RK4 integration.
///
/// dt: time step in seconds
/// mu: gravitational parameter (km^3/s^2)
pub fn propagate_rk4(state: &OrbitalState, dt: f64, mu: f64) -> OrbitalState {
    let pos = &state.position;
    let vel = &state.velocity;

    // k1
    let a1 = two_body_acceleration(pos, mu);
    let k1_pos = vel;
    let k1_vel = a1;

    // k2
    let pos2 = Vector::new(vec![
        pos.get(0) + 0.5 * dt * k1_pos.get(0),
        pos.get(1) + 0.5 * dt * k1_pos.get(1),
        pos.get(2) + 0.5 * dt * k1_pos.get(2),
    ]);
    let vel2 = Vector::new(vec![
        vel.get(0) + 0.5 * dt * k1_vel.get(0),
        vel.get(1) + 0.5 * dt * k1_vel.get(1),
        vel.get(2) + 0.5 * dt * k1_vel.get(2),
    ]);
    let a2 = two_body_acceleration(&pos2, mu);
    let k2_pos = &vel2;
    let k2_vel = a2;

    // k3
    let pos3 = Vector::new(vec![
        pos.get(0) + 0.5 * dt * k2_pos.get(0),
        pos.get(1) + 0.5 * dt * k2_pos.get(1),
        pos.get(2) + 0.5 * dt * k2_pos.get(2),
    ]);
    let vel3 = Vector::new(vec![
        vel.get(0) + 0.5 * dt * k2_vel.get(0),
        vel.get(1) + 0.5 * dt * k2_vel.get(1),
        vel.get(2) + 0.5 * dt * k2_vel.get(2),
    ]);
    let a3 = two_body_acceleration(&pos3, mu);
    let k3_pos = &vel3;
    let k3_vel = a3;

    // k4
    let pos4 = Vector::new(vec![
        pos.get(0) + dt * k3_pos.get(0),
        pos.get(1) + dt * k3_pos.get(1),
        pos.get(2) + dt * k3_pos.get(2),
    ]);
    let vel4 = Vector::new(vec![
        vel.get(0) + dt * k3_vel.get(0),
        vel.get(1) + dt * k3_vel.get(1),
        vel.get(2) + dt * k3_vel.get(2),
    ]);
    let a4 = two_body_acceleration(&pos4, mu);
    let k4_pos = &vel4;
    let k4_vel = a4;

    OrbitalState {
        position: Vector::new(vec![
            pos.get(0)
                + dt / 6.0
                    * (k1_pos.get(0) + 2.0 * k2_pos.get(0) + 2.0 * k3_pos.get(0) + k4_pos.get(0)),
            pos.get(1)
                + dt / 6.0
                    * (k1_pos.get(1) + 2.0 * k2_pos.get(1) + 2.0 * k3_pos.get(1) + k4_pos.get(1)),
            pos.get(2)
                + dt / 6.0
                    * (k1_pos.get(2) + 2.0 * k2_pos.get(2) + 2.0 * k3_pos.get(2) + k4_pos.get(2)),
        ]),
        velocity: Vector::new(vec![
            vel.get(0)
                + dt / 6.0
                    * (k1_vel.get(0) + 2.0 * k2_vel.get(0) + 2.0 * k3_vel.get(0) + k4_vel.get(0)),
            vel.get(1)
                + dt / 6.0
                    * (k1_vel.get(1) + 2.0 * k2_vel.get(1) + 2.0 * k3_vel.get(1) + k4_vel.get(1)),
            vel.get(2)
                + dt / 6.0
                    * (k1_vel.get(2) + 2.0 * k2_vel.get(2) + 2.0 * k3_vel.get(2) + k4_vel.get(2)),
        ]),
    }
}

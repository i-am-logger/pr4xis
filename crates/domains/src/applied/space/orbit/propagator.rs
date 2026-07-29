#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::geometry::point::Point3;
use crate::formal::math::geometry::vector::Vec3;
use crate::formal::math::quantity::dimension::Dimension;
use crate::formal::math::quantity::value::Quantity;
use crate::natural::physics::kinematics::velocity::Velocity;

/// Two-body orbit propagation.
///
/// Source: Vallado (2013), Chapter 2.
/// Earth gravitational parameter from quantity ontology (km³/s²).
///
/// Returns a [`Quantity`] tagged [`Dimension::GRAVITATIONAL_PARAMETER`]
/// (L³·T⁻²), never a bare `f64` — mirroring `constants::mu_earth`, whose SI
/// (m³/s²) value this rescales into the km³/s² convention this orbit module
/// uses throughout (positions are km `Point3`, velocities are km/s
/// `Velocity` — see `OrbitalState`), so `.value` stays numerically in that
/// convention rather than being SI-renormalized.
pub fn mu_earth_km3s2() -> Quantity {
    // Convert from SI (m³/s²) to km³/s² for orbital mechanics convention
    let km3_per_s2 = crate::formal::math::quantity::constants::mu_earth().value / 1e9;
    Quantity::new(km3_per_s2, Dimension::GRAVITATIONAL_PARAMETER)
}

/// Orbital state vector (position + velocity in ECI).
#[derive(Debug, Clone)]
pub struct OrbitalState {
    /// Position (x, y, z) in km, expressed in the Earth-centred inertial (ECI) frame.
    pub position: Point3,
    /// Velocity (vx, vy, vz) in km/s, expressed in the Earth-centred inertial (ECI) frame.
    pub velocity: Velocity,
}

impl OrbitalState {
    /// Compute the orbital radius (distance from central body).
    ///
    /// Returns a [`Quantity`] tagged [`Dimension::LENGTH`] (km convention).
    pub fn radius(&self) -> Quantity {
        let r = (self.position.x * self.position.x
            + self.position.y * self.position.y
            + self.position.z * self.position.z)
            .sqrt();
        Quantity::new(r, Dimension::LENGTH)
    }

    /// Compute the speed.
    ///
    /// Returns a [`Quantity`] tagged [`Dimension::VELOCITY`] (km/s convention).
    pub fn speed(&self) -> Quantity {
        self.velocity.speed()
    }

    /// Compute specific orbital energy (vis-viva).
    ///
    /// Returns a [`Quantity`] tagged [`Dimension::SPECIFIC_ENERGY`] (km²/s²
    /// convention — energy per unit mass, Vallado 2013 §2.3).
    pub fn specific_energy(&self, mu: &Quantity) -> Quantity {
        let v = self.speed().value;
        let r = self.radius().value;
        Quantity::new(v.powi(2) / 2.0 - mu.value / r, Dimension::SPECIFIC_ENERGY)
    }

    /// Compute semi-major axis from vis-viva equation.
    ///
    /// Returns a [`Quantity`] tagged [`Dimension::LENGTH`] (km convention).
    pub fn semi_major_axis(&self, mu: &Quantity) -> Quantity {
        let e = self.specific_energy(mu).value;
        Quantity::new(-mu.value / (2.0 * e), Dimension::LENGTH)
    }
}

/// Two-body gravitational acceleration in the Earth-centred inertial (ECI) frame.
pub fn two_body_acceleration(position: &Point3, mu: f64) -> Vec3 {
    let r2 = position.x.powi(2) + position.y.powi(2) + position.z.powi(2);
    let r3 = r2 * r2.sqrt();
    Vec3::new(
        -mu * position.x / r3,
        -mu * position.y / r3,
        -mu * position.z / r3,
    )
}

/// Propagate orbital state using RK4 integration.
///
/// dt: time step in seconds
/// mu: gravitational parameter (km^3/s^2)
pub fn propagate_rk4(state: &OrbitalState, dt: f64, mu: f64) -> OrbitalState {
    let pos = &state.position;
    let vel = &state.velocity;
    let vel_vec = Vec3::new(vel.vx, vel.vy, vel.vz);

    // k1
    let a1 = two_body_acceleration(pos, mu);
    let k1_pos = vel_vec.clone();
    let k1_vel = a1;

    // k2
    let pos2 = Point3::new(
        pos.x + 0.5 * dt * k1_pos.x,
        pos.y + 0.5 * dt * k1_pos.y,
        pos.z + 0.5 * dt * k1_pos.z,
    );
    let vel2 = Vec3::new(
        vel_vec.x + 0.5 * dt * k1_vel.x,
        vel_vec.y + 0.5 * dt * k1_vel.y,
        vel_vec.z + 0.5 * dt * k1_vel.z,
    );
    let a2 = two_body_acceleration(&pos2, mu);
    let k2_pos = vel2.clone();
    let k2_vel = a2;

    // k3
    let pos3 = Point3::new(
        pos.x + 0.5 * dt * k2_pos.x,
        pos.y + 0.5 * dt * k2_pos.y,
        pos.z + 0.5 * dt * k2_pos.z,
    );
    let vel3 = Vec3::new(
        vel_vec.x + 0.5 * dt * k2_vel.x,
        vel_vec.y + 0.5 * dt * k2_vel.y,
        vel_vec.z + 0.5 * dt * k2_vel.z,
    );
    let a3 = two_body_acceleration(&pos3, mu);
    let k3_pos = vel3.clone();
    let k3_vel = a3;

    // k4
    let pos4 = Point3::new(
        pos.x + dt * k3_pos.x,
        pos.y + dt * k3_pos.y,
        pos.z + dt * k3_pos.z,
    );
    let vel4 = Vec3::new(
        vel_vec.x + dt * k3_vel.x,
        vel_vec.y + dt * k3_vel.y,
        vel_vec.z + dt * k3_vel.z,
    );
    let a4 = two_body_acceleration(&pos4, mu);
    let k4_pos = vel4;
    let k4_vel = a4;

    OrbitalState {
        position: Point3::new(
            pos.x + dt / 6.0 * (k1_pos.x + 2.0 * k2_pos.x + 2.0 * k3_pos.x + k4_pos.x),
            pos.y + dt / 6.0 * (k1_pos.y + 2.0 * k2_pos.y + 2.0 * k3_pos.y + k4_pos.y),
            pos.z + dt / 6.0 * (k1_pos.z + 2.0 * k2_pos.z + 2.0 * k3_pos.z + k4_pos.z),
        ),
        velocity: Velocity::new(
            vel_vec.x + dt / 6.0 * (k1_vel.x + 2.0 * k2_vel.x + 2.0 * k3_vel.x + k4_vel.x),
            vel_vec.y + dt / 6.0 * (k1_vel.y + 2.0 * k2_vel.y + 2.0 * k3_vel.y + k4_vel.y),
            vel_vec.z + dt / 6.0 * (k1_vel.z + 2.0 * k2_vel.z + 2.0 * k3_vel.z + k4_vel.z),
        ),
    }
}

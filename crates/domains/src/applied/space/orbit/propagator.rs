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
    /// Position [x, y, z] in km.
    pub position: [f64; 3],
    /// Velocity [vx, vy, vz] in km/s.
    pub velocity: [f64; 3],
}

impl OrbitalState {
    /// Compute the orbital radius (distance from central body).
    pub fn radius(&self) -> f64 {
        let [x, y, z] = self.position;
        (x * x + y * y + z * z).sqrt()
    }

    /// Compute the speed.
    pub fn speed(&self) -> f64 {
        let [vx, vy, vz] = self.velocity;
        (vx * vx + vy * vy + vz * vz).sqrt()
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

/// Two-body gravitational acceleration.
pub fn two_body_acceleration(position: &[f64; 3], mu: f64) -> [f64; 3] {
    let r2 = position[0].powi(2) + position[1].powi(2) + position[2].powi(2);
    let r3 = r2 * r2.sqrt();
    [
        -mu * position[0] / r3,
        -mu * position[1] / r3,
        -mu * position[2] / r3,
    ]
}

/// Propagate orbital state using RK4 integration.
///
/// dt: time step in seconds
/// mu: gravitational parameter (km^3/s^2)
pub fn propagate_rk4(state: &OrbitalState, dt: f64, mu: f64) -> OrbitalState {
    let pos = state.position;
    let vel = state.velocity;

    // k1
    let a1 = two_body_acceleration(&pos, mu);
    let k1_pos = vel;
    let k1_vel = a1;

    // k2
    let pos2 = [
        pos[0] + 0.5 * dt * k1_pos[0],
        pos[1] + 0.5 * dt * k1_pos[1],
        pos[2] + 0.5 * dt * k1_pos[2],
    ];
    let vel2 = [
        vel[0] + 0.5 * dt * k1_vel[0],
        vel[1] + 0.5 * dt * k1_vel[1],
        vel[2] + 0.5 * dt * k1_vel[2],
    ];
    let a2 = two_body_acceleration(&pos2, mu);
    let k2_pos = vel2;
    let k2_vel = a2;

    // k3
    let pos3 = [
        pos[0] + 0.5 * dt * k2_pos[0],
        pos[1] + 0.5 * dt * k2_pos[1],
        pos[2] + 0.5 * dt * k2_pos[2],
    ];
    let vel3 = [
        vel[0] + 0.5 * dt * k2_vel[0],
        vel[1] + 0.5 * dt * k2_vel[1],
        vel[2] + 0.5 * dt * k2_vel[2],
    ];
    let a3 = two_body_acceleration(&pos3, mu);
    let k3_pos = vel3;
    let k3_vel = a3;

    // k4
    let pos4 = [
        pos[0] + dt * k3_pos[0],
        pos[1] + dt * k3_pos[1],
        pos[2] + dt * k3_pos[2],
    ];
    let vel4 = [
        vel[0] + dt * k3_vel[0],
        vel[1] + dt * k3_vel[1],
        vel[2] + dt * k3_vel[2],
    ];
    let a4 = two_body_acceleration(&pos4, mu);
    let k4_pos = vel4;
    let k4_vel = a4;

    OrbitalState {
        position: [
            pos[0] + dt / 6.0 * (k1_pos[0] + 2.0 * k2_pos[0] + 2.0 * k3_pos[0] + k4_pos[0]),
            pos[1] + dt / 6.0 * (k1_pos[1] + 2.0 * k2_pos[1] + 2.0 * k3_pos[1] + k4_pos[1]),
            pos[2] + dt / 6.0 * (k1_pos[2] + 2.0 * k2_pos[2] + 2.0 * k3_pos[2] + k4_pos[2]),
        ],
        velocity: [
            vel[0] + dt / 6.0 * (k1_vel[0] + 2.0 * k2_vel[0] + 2.0 * k3_vel[0] + k4_vel[0]),
            vel[1] + dt / 6.0 * (k1_vel[1] + 2.0 * k2_vel[1] + 2.0 * k3_vel[1] + k4_vel[1]),
            vel[2] + dt / 6.0 * (k1_vel[2] + 2.0 * k2_vel[2] + 2.0 * k3_vel[2] + k4_vel[2]),
        ],
    }
}

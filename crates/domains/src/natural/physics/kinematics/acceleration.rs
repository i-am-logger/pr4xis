/// Acceleration vector: the second derivative of position with respect to time.
///
/// a = dv/dt = d²x/dt² (meters per second squared).
///
/// In Newton's second law: F = ma, so acceleration = force / mass.
///
/// Source: Goldstein, *Classical Mechanics* (2002), Chapter 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Acceleration {
    pub ax: f64,
    pub ay: f64,
    pub az: f64,
}

impl Acceleration {
    pub fn new(ax: f64, ay: f64, az: f64) -> Self {
        Self { ax, ay, az }
    }

    pub fn zero() -> Self {
        Self {
            ax: 0.0,
            ay: 0.0,
            az: 0.0,
        }
    }

    /// Standard gravity (m/s²), from the quantity ontology.
    pub fn gravity() -> Self {
        let g = crate::formal::math::quantity::constants::standard_gravity().value;
        Self {
            ax: 0.0,
            ay: 0.0,
            az: -g,
        }
    }

    /// Magnitude: |a|.
    pub fn magnitude(&self) -> f64 {
        (self.ax * self.ax + self.ay * self.ay + self.az * self.az).sqrt()
    }

    /// Velocity change over duration dt: Δv = a * dt.
    pub fn velocity_change(
        &self,
        dt: f64,
    ) -> crate::natural::physics::kinematics::velocity::Velocity {
        crate::natural::physics::kinematics::velocity::Velocity::new(
            self.ax * dt,
            self.ay * dt,
            self.az * dt,
        )
    }

    /// Add accelerations.
    pub fn add(&self, other: &Self) -> Self {
        Self {
            ax: self.ax + other.ax,
            ay: self.ay + other.ay,
            az: self.az + other.az,
        }
    }
}

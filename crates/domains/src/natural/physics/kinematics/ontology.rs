use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::geometry::point::Point3;

use crate::natural::physics::kinematics::acceleration::Acceleration;
use crate::natural::physics::kinematics::motion_model::{self, MotionModelType};
use crate::natural::physics::kinematics::trajectory::KinematicState;
use crate::natural::physics::kinematics::velocity::Velocity;

// ---------------------------------------------------------------------------
// Entity: kinematic quantities
// ---------------------------------------------------------------------------

/// Classification of kinematic quantities.
///
/// These are the successive derivatives of position with respect to time:
/// position (0th), velocity (1st), acceleration (2nd), jerk (3rd).
///
/// Source: physics.info/kinematics-calculus (The Physics Hypertextbook).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum KinematicQuantity {
    /// 0th derivative: where (meters).
    Position,
    /// 1st derivative: how fast (meters/second).
    Velocity,
    /// 2nd derivative: change of velocity (meters/second²).
    Acceleration,
    /// 3rd derivative: change of acceleration (meters/second³).
    Jerk,
}

define_ontology! {
    /// The kinematics category.
    pub KinematicsOntology for KinematicsCategory {
        concepts: KinematicQuantity,
        relation: DerivativeRelation,
    }
}

/// Quality: derivative order (0 = position, 1 = velocity, ...).
#[derive(Debug, Clone)]
pub struct DerivativeOrder;

impl Quality for DerivativeOrder {
    type Individual = KinematicQuantity;
    type Value = usize;

    fn get(&self, q: &KinematicQuantity) -> Option<usize> {
        Some(match q {
            KinematicQuantity::Position => 0,
            KinematicQuantity::Velocity => 1,
            KinematicQuantity::Acceleration => 2,
            KinematicQuantity::Jerk => 3,
        })
    }
}

/// Quality: SI unit.
#[derive(Debug, Clone)]
pub struct SiUnit;

impl Quality for SiUnit {
    type Individual = KinematicQuantity;
    type Value = &'static str;

    fn get(&self, q: &KinematicQuantity) -> Option<&'static str> {
        Some(match q {
            KinematicQuantity::Position => "m",
            KinematicQuantity::Velocity => "m/s",
            KinematicQuantity::Acceleration => "m/s²",
            KinematicQuantity::Jerk => "m/s³",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Velocity is the derivative of position: v = Δx/Δt as Δt → 0.
pub struct VelocityIsDerivativeOfPosition;

impl Axiom for VelocityIsDerivativeOfPosition {
    fn description(&self) -> &str {
        "velocity = dx/dt: position change / time yields velocity"
    }

    fn holds(&self) -> bool {
        // For constant velocity, displacement should equal v*dt exactly
        let v = Velocity::new(3.0, 4.0, 0.0);
        let dt = 2.0;
        let displacement = v.displace(dt);
        (displacement.x - 6.0).abs() < 1e-12
            && (displacement.y - 8.0).abs() < 1e-12
            && (displacement.z - 0.0).abs() < 1e-12
    }
}

/// Acceleration is the derivative of velocity: a = Δv/Δt.
pub struct AccelerationIsDerivativeOfVelocity;

impl Axiom for AccelerationIsDerivativeOfVelocity {
    fn description(&self) -> &str {
        "acceleration = dv/dt: velocity change / time yields acceleration"
    }

    fn holds(&self) -> bool {
        let v1 = Velocity::new(0.0, 0.0, 0.0);
        let v2 = Velocity::new(10.0, 0.0, 0.0);
        let dt = 5.0;
        let a = v1.acceleration_to(&v2, dt).unwrap();
        (a.ax - 2.0).abs() < 1e-12 && a.ay.abs() < 1e-12 && a.az.abs() < 1e-12
    }
}

/// Constant velocity propagation: x(t+dt) = x(t) + v*dt.
pub struct ConstantVelocityPropagation;

impl Axiom for ConstantVelocityPropagation {
    fn description(&self) -> &str {
        "constant velocity: x(t+dt) = x(t) + v*dt"
    }

    fn holds(&self) -> bool {
        let state = KinematicState {
            position: Point3::new(0.0, 0.0, 0.0),
            velocity: Velocity::new(1.0, 2.0, 3.0),
            acceleration: Acceleration::zero(),
        };
        let dt = 5.0;
        let next = motion_model::propagate(&state, dt, MotionModelType::ConstantVelocity);
        (next.position.x - 5.0).abs() < 1e-10
            && (next.position.y - 10.0).abs() < 1e-10
            && (next.position.z - 15.0).abs() < 1e-10
    }
}

/// Constant acceleration: x(t+dt) = x(t) + v*dt + 0.5*a*dt².
pub struct ConstantAccelerationPropagation;

impl Axiom for ConstantAccelerationPropagation {
    fn description(&self) -> &str {
        "constant acceleration: x(t+dt) = x(t) + v*dt + 0.5*a*dt²"
    }

    fn holds(&self) -> bool {
        // Free fall: x0=0, v0=0, a=-g, after 1s: x = -0.5*g*1² = -4.903325
        let state = KinematicState {
            position: Point3::new(0.0, 0.0, 100.0),
            velocity: Velocity::zero(),
            acceleration: Acceleration::gravity(),
        };
        let dt = 1.0;
        let next = state.propagate(dt);
        let g = crate::formal::math::quantity::constants::standard_gravity().value;
        let expected_z = 100.0 + 0.5 * (-g) * 1.0;
        (next.position.z - expected_z).abs() < 1e-8
    }
}

/// Velocity updates under constant acceleration: v(t+dt) = v(t) + a*dt.
pub struct VelocityUpdateUnderAcceleration;

impl Axiom for VelocityUpdateUnderAcceleration {
    fn description(&self) -> &str {
        "v(t+dt) = v(t) + a*dt under constant acceleration"
    }

    fn holds(&self) -> bool {
        let state = KinematicState {
            position: Point3::origin(),
            velocity: Velocity::new(10.0, 0.0, 0.0),
            acceleration: Acceleration::new(2.0, 0.0, 0.0),
        };
        let dt = 3.0;
        let next = state.propagate(dt);
        // v_new = 10 + 2*3 = 16
        (next.velocity.vx - 16.0).abs() < 1e-10
    }
}

/// Static model: position doesn't change.
pub struct StaticModelInvariance;

impl Axiom for StaticModelInvariance {
    fn description(&self) -> &str {
        "static model: position unchanged after propagation"
    }

    fn holds(&self) -> bool {
        let state = KinematicState {
            position: Point3::new(1.0, 2.0, 3.0),
            velocity: Velocity::new(10.0, 20.0, 30.0), // velocity ignored for static
            acceleration: Acceleration::zero(),
        };
        let next = motion_model::propagate(&state, 100.0, MotionModelType::Static);
        next.position == state.position
    }
}

/// Speed is non-negative: |v| ≥ 0.
pub struct SpeedNonNegative;

impl Axiom for SpeedNonNegative {
    fn description(&self) -> &str {
        "speed is non-negative: |v| >= 0"
    }

    fn holds(&self) -> bool {
        let test_velocities = [
            Velocity::zero(),
            Velocity::new(1.0, 0.0, 0.0),
            Velocity::new(-5.0, 3.0, -2.0),
            Velocity::new(100.0, -200.0, 300.0),
        ];
        test_velocities.iter().all(|v| v.speed() >= 0.0)
    }
}

/// Galilean velocity addition is commutative: v1 + v2 = v2 + v1.
pub struct VelocityAdditionCommutative;

impl Axiom for VelocityAdditionCommutative {
    fn description(&self) -> &str {
        "Galilean velocity addition is commutative: v1 + v2 = v2 + v1"
    }

    fn holds(&self) -> bool {
        let v1 = Velocity::new(1.0, 2.0, 3.0);
        let v2 = Velocity::new(4.0, 5.0, 6.0);
        let a = v1.add(&v2);
        let b = v2.add(&v1);
        (a.vx - b.vx).abs() < 1e-15 && (a.vy - b.vy).abs() < 1e-15 && (a.vz - b.vz).abs() < 1e-15
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

// The kinematics ontology -- classical mechanics of motion.
//
// Founded on:
//   - Goldstein, H. (2002). *Classical Mechanics* (3rd ed.)
//   - Newton's laws of motion (Principia, 1687)
//   - Bar-Shalom et al. (2001). Motion models for tracking
impl Ontology for KinematicsOntology {
    type Cat = KinematicsCategory;
    type Qual = DerivativeOrder;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(VelocityIsDerivativeOfPosition),
            Box::new(AccelerationIsDerivativeOfVelocity),
            Box::new(ConstantVelocityPropagation),
            Box::new(ConstantAccelerationPropagation),
            Box::new(VelocityUpdateUnderAcceleration),
            Box::new(StaticModelInvariance),
            Box::new(SpeedNonNegative),
            Box::new(VelocityAdditionCommutative),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<KinematicsCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        KinematicsOntology::validate().unwrap();
    }
}

//! Kinematics — successive time-derivatives of position.
//!
//! Models the four canonical kinematic quantities (position, velocity,
//! acceleration, jerk) as a discrete concept set and ships the classical
//! propagation laws (constant velocity, constant acceleration) plus the
//! Galilean-relativity invariants used by motion-model tracking. The
//! continuous-time math lives in the sibling `acceleration.rs`,
//! `velocity.rs`, `motion_model.rs`, `trajectory.rs` modules; this
//! ontology is the discrete categorical layer.
//!
//! # Literature
//!
//! - **Newton (1687)** *Philosophiae Naturalis Principia Mathematica* —
//!   the laws of motion and the calculus of position derivatives.
//! - **Goldstein (2002)** *Classical Mechanics* (3rd ed.) — modern
//!   treatment of kinematics and Galilean transformations.
//! - **Bar-Shalom, Li & Kirubarajan (2001)** *Estimation with Applications
//!   to Tracking and Navigation* — Static / Constant-Velocity /
//!   Constant-Acceleration motion-model taxonomy.

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::geometry::point::Point3;

use crate::natural::physics::kinematics::acceleration::Acceleration as AccelerationVec;
use crate::natural::physics::kinematics::motion_model::{self, MotionModelType};
use crate::natural::physics::kinematics::trajectory::KinematicState;
use crate::natural::physics::kinematics::velocity::Velocity as VelocityVec;

pr4xis::ontology! {
    name: "Kinematics",
    source: "Newton (1687) Principia; Goldstein (2002) Classical Mechanics; Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation",

    concepts: [Position, Velocity, Acceleration, Jerk],

    labels: {
        Position: ("en", "Position",
            "0th time-derivative of position — where the body is, in metres. Newton (1687)."),
        Velocity: ("en", "Velocity",
            "1st time-derivative of position — rate of change of position, in m/s. Newton (1687)."),
        Acceleration: ("en", "Acceleration",
            "2nd time-derivative of position — rate of change of velocity, in m/s². Newton (1687)."),
        Jerk: ("en", "Jerk",
            "3rd time-derivative of position — rate of change of acceleration, in m/s³. Schot (1978) Am. J. Phys."),
    },

    // The derivative relation does NOT subsume; it's a typed sequence
    // (Pos→Vel→Acc→Jerk). Encoded as Causation per "d/dt produces the
    // next derivative" — Reichenbach (1956) treats temporal derivation
    // as a causation-typed arrow.
    causes: [
        (Position, Velocity),
        (Velocity, Acceleration),
        (Acceleration, Jerk),
    ],
}

/// Quality: derivative order (0 = position, 1 = velocity, 2 = acceleration, 3 = jerk).
#[derive(Debug, Clone)]
pub struct DerivativeOrder;

impl Quality for DerivativeOrder {
    type Individual = KinematicsConcept;
    type Value = usize;

    fn get(&self, q: &KinematicsConcept) -> Option<usize> {
        Some(match q {
            KinematicsConcept::Position => 0,
            KinematicsConcept::Velocity => 1,
            KinematicsConcept::Acceleration => 2,
            KinematicsConcept::Jerk => 3,
        })
    }
}

/// Quality: SI unit string.
#[derive(Debug, Clone)]
pub struct SiUnit;

impl Quality for SiUnit {
    type Individual = KinematicsConcept;
    type Value = &'static str;

    fn get(&self, q: &KinematicsConcept) -> Option<&'static str> {
        Some(match q {
            KinematicsConcept::Position => "m",
            KinematicsConcept::Velocity => "m/s",
            KinematicsConcept::Acceleration => "m/s²",
            KinematicsConcept::Jerk => "m/s³",
        })
    }
}

impl Ontology for KinematicsOntology {
    type Cat = KinematicsCategory;
    type Qual = DerivativeOrder;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(VelocityIsDerivativeOfPosition));
        axioms.push(Box::new(AccelerationIsDerivativeOfVelocity));
        axioms.push(Box::new(ConstantVelocityPropagation));
        axioms.push(Box::new(ConstantAccelerationPropagation));
        axioms.push(Box::new(VelocityUpdateUnderAcceleration));
        axioms.push(Box::new(StaticModelInvariance));
        axioms.push(Box::new(SpeedNonNegative));
        axioms.push(Box::new(VelocityAdditionCommutative));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: velocity is the time-derivative of position. v = Δx/Δt as Δt → 0.
///
/// Newton (1687): the first derivative of position with respect to time
/// is velocity. Verified by checking that displacement under constant
/// velocity equals v · dt exactly.
pub struct VelocityIsDerivativeOfPosition;

impl Axiom for VelocityIsDerivativeOfPosition {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let v = VelocityVec::new(3.0, 4.0, 0.0);
        let dt = 2.0;
        let displacement = v.displace(dt);
        if (displacement.x - 6.0).abs() < 1e-12
            && (displacement.y - 8.0).abs() < 1e-12
            && (displacement.z - 0.0).abs() < 1e-12
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "VelocityIsDerivativeOfPosition",
        "velocity = dx/dt: position change / time yields velocity",
        "Newton (1687) Principia"
    );
}

pr4xis::register_axiom!(VelocityIsDerivativeOfPosition, "Newton (1687) Principia");

/// Axiom: acceleration is the time-derivative of velocity. a = Δv/Δt.
///
/// Newton (1687): the second derivative of position with respect to
/// time is acceleration.
pub struct AccelerationIsDerivativeOfVelocity;

impl Axiom for AccelerationIsDerivativeOfVelocity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let v1 = VelocityVec::new(0.0, 0.0, 0.0);
        let v2 = VelocityVec::new(10.0, 0.0, 0.0);
        let dt = 5.0;
        let a = v1.acceleration_to(&v2, dt).unwrap();
        if (a.ax - 2.0).abs() < 1e-12 && a.ay.abs() < 1e-12 && a.az.abs() < 1e-12 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AccelerationIsDerivativeOfVelocity",
        "acceleration = dv/dt: velocity change / time yields acceleration",
        "Newton (1687) Principia"
    );
}

pr4xis::register_axiom!(
    AccelerationIsDerivativeOfVelocity,
    "Newton (1687) Principia"
);

/// Axiom: constant-velocity propagation. x(t+dt) = x(t) + v·dt.
///
/// Bar-Shalom et al. (2001) §6.3 — the canonical constant-velocity
/// motion model used in target tracking.
pub struct ConstantVelocityPropagation;

impl Axiom for ConstantVelocityPropagation {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let state = KinematicState {
            position: Point3::new(0.0, 0.0, 0.0),
            velocity: VelocityVec::new(1.0, 2.0, 3.0),
            acceleration: AccelerationVec::zero(),
        };
        let dt = 5.0;
        let next = motion_model::propagate(&state, dt, MotionModelType::ConstantVelocity);
        if (next.position.x - 5.0).abs() < 1e-10
            && (next.position.y - 10.0).abs() < 1e-10
            && (next.position.z - 15.0).abs() < 1e-10
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ConstantVelocityPropagation",
        "constant velocity: x(t+dt) = x(t) + v*dt",
        "Bar-Shalom, Li & Kirubarajan (2001) §6.3"
    );
}

pr4xis::register_axiom!(
    ConstantVelocityPropagation,
    "Bar-Shalom, Li & Kirubarajan (2001) §6.3"
);

/// Axiom: constant-acceleration propagation. x(t+dt) = x(t) + v·dt + ½·a·dt².
///
/// Newton (1687); Bar-Shalom et al. (2001) §6.3 — the constant-acceleration
/// motion model. Verified with free fall: x(t) = x₀ + ½·g·t².
pub struct ConstantAccelerationPropagation;

impl Axiom for ConstantAccelerationPropagation {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let state = KinematicState {
            position: Point3::new(0.0, 0.0, 100.0),
            velocity: VelocityVec::zero(),
            acceleration: AccelerationVec::gravity(),
        };
        let dt = 1.0;
        let next = state.propagate(dt);
        let g = crate::formal::math::quantity::constants::standard_gravity().value;
        let expected_z = 100.0 + 0.5 * (-g) * 1.0;
        if (next.position.z - expected_z).abs() < 1e-8 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ConstantAccelerationPropagation",
        "constant acceleration: x(t+dt) = x(t) + v*dt + 0.5*a*dt^2",
        "Newton (1687) Principia; Bar-Shalom et al. (2001) §6.3"
    );
}

pr4xis::register_axiom!(
    ConstantAccelerationPropagation,
    "Newton (1687) Principia; Bar-Shalom et al. (2001) §6.3"
);

/// Axiom: velocity updates linearly under constant acceleration. v(t+dt) = v(t) + a·dt.
pub struct VelocityUpdateUnderAcceleration;

impl Axiom for VelocityUpdateUnderAcceleration {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let state = KinematicState {
            position: Point3::origin(),
            velocity: VelocityVec::new(10.0, 0.0, 0.0),
            acceleration: AccelerationVec::new(2.0, 0.0, 0.0),
        };
        let dt = 3.0;
        let next = state.propagate(dt);
        if (next.velocity.vx - 16.0).abs() < 1e-10 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "VelocityUpdateUnderAcceleration",
        "v(t+dt) = v(t) + a*dt under constant acceleration",
        "Newton (1687) Principia"
    );
}

pr4xis::register_axiom!(VelocityUpdateUnderAcceleration, "Newton (1687) Principia");

/// Axiom: the Static motion model preserves position regardless of velocity.
///
/// Bar-Shalom et al. (2001) §6.2 — the Static model assumes the target
/// does not move; position is invariant under propagation.
pub struct StaticModelInvariance;

impl Axiom for StaticModelInvariance {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let state = KinematicState {
            position: Point3::new(1.0, 2.0, 3.0),
            velocity: VelocityVec::new(10.0, 20.0, 30.0),
            acceleration: AccelerationVec::zero(),
        };
        let next = motion_model::propagate(&state, 100.0, MotionModelType::Static);
        if next.position == state.position {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StaticModelInvariance",
        "static model: position unchanged after propagation",
        "Bar-Shalom, Li & Kirubarajan (2001) §6.2"
    );
}

pr4xis::register_axiom!(
    StaticModelInvariance,
    "Bar-Shalom, Li & Kirubarajan (2001) §6.2"
);

/// Axiom: speed (the magnitude of velocity) is non-negative. |v| ≥ 0.
///
/// Goldstein (2002) §1: speed is the Euclidean norm of velocity, hence
/// non-negative.
pub struct SpeedNonNegative;

impl Axiom for SpeedNonNegative {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let test_velocities = [
            VelocityVec::zero(),
            VelocityVec::new(1.0, 0.0, 0.0),
            VelocityVec::new(-5.0, 3.0, -2.0),
            VelocityVec::new(100.0, -200.0, 300.0),
        ];
        if test_velocities.iter().all(|v| v.speed() >= 0.0) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SpeedNonNegative",
        "speed is non-negative: |v| >= 0",
        "Goldstein (2002) Classical Mechanics §1"
    );
}

pr4xis::register_axiom!(SpeedNonNegative, "Goldstein (2002) Classical Mechanics §1");

/// Axiom: Galilean velocity addition is commutative. v₁ + v₂ = v₂ + v₁.
///
/// Newton (1687); Goldstein (2002) §1.5 — Galilean relativity treats
/// velocity addition as vector addition, which is commutative.
pub struct VelocityAdditionCommutative;

impl Axiom for VelocityAdditionCommutative {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let v1 = VelocityVec::new(1.0, 2.0, 3.0);
        let v2 = VelocityVec::new(4.0, 5.0, 6.0);
        let a = v1.add(&v2);
        let b = v2.add(&v1);
        if (a.vx - b.vx).abs() < 1e-15 && (a.vy - b.vy).abs() < 1e-15 && (a.vz - b.vz).abs() < 1e-15
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "VelocityAdditionCommutative",
        "Galilean velocity addition is commutative: v1 + v2 = v2 + v1",
        "Newton (1687); Goldstein (2002) Classical Mechanics §1.5"
    );
}

pr4xis::register_axiom!(
    VelocityAdditionCommutative,
    "Newton (1687); Goldstein (2002) Classical Mechanics §1.5"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<KinematicsCategory>();
    }

    #[test]
    fn ontology_validates() {
        KinematicsOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn four_quantities() {
        assert_eq!(KinematicsConcept::variants().len(), 4);
    }

    #[test]
    fn derivative_chain_is_causation() {
        let causes: Vec<_> = KinematicsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == KinematicsRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(causes.contains(&(KinematicsConcept::Position, KinematicsConcept::Velocity)));
        assert!(causes.contains(&(KinematicsConcept::Velocity, KinematicsConcept::Acceleration)));
        assert!(causes.contains(&(KinematicsConcept::Acceleration, KinematicsConcept::Jerk)));
    }

    #[test]
    fn derivative_order_total() {
        let q = DerivativeOrder;
        for c in KinematicsConcept::variants() {
            assert!(q.get(&c).is_some());
        }
        assert_eq!(q.get(&KinematicsConcept::Position), Some(0));
        assert_eq!(q.get(&KinematicsConcept::Jerk), Some(3));
    }

    #[test]
    fn si_unit_total() {
        let q = SiUnit;
        for c in KinematicsConcept::variants() {
            assert!(q.get(&c).is_some());
        }
    }

    #[test]
    fn velocity_is_derivative_of_position_holds() {
        assert!(VelocityIsDerivativeOfPosition.verify().is_ok());
    }

    #[test]
    fn acceleration_is_derivative_of_velocity_holds() {
        assert!(AccelerationIsDerivativeOfVelocity.verify().is_ok());
    }

    #[test]
    fn constant_velocity_propagation_holds() {
        assert!(ConstantVelocityPropagation.verify().is_ok());
    }

    #[test]
    fn constant_acceleration_propagation_holds() {
        assert!(ConstantAccelerationPropagation.verify().is_ok());
    }

    #[test]
    fn velocity_update_under_acceleration_holds() {
        assert!(VelocityUpdateUnderAcceleration.verify().is_ok());
    }

    #[test]
    fn static_model_invariance_holds() {
        assert!(StaticModelInvariance.verify().is_ok());
    }

    #[test]
    fn speed_non_negative_holds() {
        assert!(SpeedNonNegative.verify().is_ok());
    }

    #[test]
    fn velocity_addition_commutative_holds() {
        assert!(VelocityAdditionCommutative.verify().is_ok());
    }

    fn arb_quantity() -> impl Strategy<Value = KinematicsConcept> {
        proptest::sample::select(KinematicsConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_derivative_order_total(c in arb_quantity()) {
            prop_assert!(DerivativeOrder.get(&c).is_some());
        }

        #[test]
        fn prop_si_unit_total(c in arb_quantity()) {
            prop_assert!(SiUnit.get(&c).is_some());
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in KinematicsOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
}

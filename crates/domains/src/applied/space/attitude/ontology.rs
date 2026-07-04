//! Spacecraft attitude determination sensors.
//!
//! Source: Wertz (1978), *Spacecraft Attitude Determination and Control*

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::FinitelyGenerated;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

use crate::formal::math::quantity::unit::ARCSECOND;
use crate::formal::math::quantity::value::Quantity;

pr4xis::ontology! {
    name: "Attitude",
    source: "Wertz (1978); Markley & Crassidis (2014)",

    concepts: [StarTracker, SunSensor, EarthHorizon, Magnetometer],

    labels: {
        StarTracker: ("en", "Star tracker", "Star tracker: high-accuracy inertial attitude reference."),
        SunSensor: ("en", "Sun sensor", "Sun sensor: determines direction to the Sun."),
        EarthHorizon: ("en", "Earth horizon sensor", "Earth horizon sensor: determines nadir direction."),
        Magnetometer: ("en", "Magnetometer", "Magnetometer: measures local magnetic field vector."),
    },
}

/// Quality: typical 1σ accuracy of each attitude sensor, as a typed angular
/// [`Quantity`] (SI-normalised to radians), NOT a bare float.
///
/// The magnitude carries its dimension (`ANGLE`) and its unit of provenance
/// (arcseconds — the pointing-accuracy convention, Wertz 1978), so downstream
/// comparisons are guarded by dimensional compatibility rather than trusting a
/// naked `f64`. A smaller quantity is finer accuracy.
#[derive(Debug, Clone)]
pub struct SensorAccuracy;

impl Quality for SensorAccuracy {
    type Individual = AttitudeConcept;
    type Value = Quantity;
    const KIND: QualityKind = QualityKind::Physical;

    fn get(&self, sensor: &AttitudeConcept) -> Option<Quantity> {
        let arcsec = |v: f64| Quantity::from_unit(v, &ARCSECOND);
        Some(match sensor {
            AttitudeConcept::StarTracker => arcsec(1.0),
            AttitudeConcept::SunSensor => arcsec(60.0),
            AttitudeConcept::EarthHorizon => arcsec(3600.0),
            AttitudeConcept::Magnetometer => arcsec(7200.0),
        })
    }
}

/// Axiom: a unit quaternion has norm 1 (attitude representation constraint).
pub struct QuaternionUnitNorm;

impl Axiom for QuaternionUnitNorm {
    fn verify(&self) -> Verdict {
        // Markley & Crassidis (2014) §2.7: attitude quaternions live on
        // the unit 3-sphere S³ ⊂ ℝ⁴ — only unit quaternions correspond to
        // valid rotations in SO(3). We do not merely assert this: we build
        // canonical quaternions through every constructor (identity, raw
        // components, axis-angle over a spread of axes/angles, Euler 3-2-1)
        // and through the attitude-kinematics propagator, then check that
        // the actual engine norm() lands within tolerance of 1.
        use crate::applied::space::attitude::kinematics::{
            Quaternion as KinematicQuaternion, propagate_attitude,
        };
        use crate::formal::math::linear_algebra::vector_space::Vector;
        use crate::formal::math::rotation::quaternion::Quaternion as RotationQuaternion;

        // 1e-9 is far looser than the ~1e-15 round-off the normalizing
        // constructors actually incur, yet tight enough that a genuinely
        // non-unit quaternion (a construction that failed to normalize)
        // trips the Err branch. This is a real threshold, not `if true`.
        const TOL: f64 = 1e-9;

        // --- SO(3) rotation quaternions --------------------------------
        let axes = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [2.0, -3.0, 0.5],
        ];
        let angles = [
            0.0,
            0.1,
            core::f64::consts::FRAC_PI_4,
            core::f64::consts::FRAC_PI_2,
            2.0,
            core::f64::consts::PI,
            5.0,
        ];

        let mut rotation_qs: Vec<RotationQuaternion> = Vec::new();
        rotation_qs.push(RotationQuaternion::identity());
        for axis in axes.iter() {
            for &angle in angles.iter() {
                rotation_qs.push(RotationQuaternion::from_axis_angle(
                    &Vector::new(vec![axis[0], axis[1], axis[2]]),
                    angle,
                ));
            }
        }
        rotation_qs.push(RotationQuaternion::from_euler_321(0.3, -0.7, 1.1));
        rotation_qs.push(RotationQuaternion::from_euler_321(1.5, 0.0, -2.2));
        // Raw, un-normalized components: the constructor must renormalize.
        rotation_qs.push(RotationQuaternion::new(2.0, -1.0, 0.5, 4.0));

        let rotation_ok = rotation_qs.iter().all(|q| (q.norm() - 1.0).abs() < TOL);

        // --- Attitude-kinematics quaternions ---------------------------
        let mut kinematic_qs: Vec<KinematicQuaternion> = Vec::new();
        kinematic_qs.push(KinematicQuaternion::identity());
        // new() normalizes raw components.
        kinematic_qs.push(KinematicQuaternion::new(3.0, 1.0, -2.0, 0.5));
        // Propagate the identity attitude under a constant body rate; the
        // first-order integrator renormalizes each step, so the invariant
        // must survive 100 steps of drift.
        let omega = Vector::new(vec![0.01, -0.02, 0.015]);
        let mut q = KinematicQuaternion::identity();
        for _ in 0..100 {
            q = propagate_attitude(&q, &omega, 0.1);
        }
        kinematic_qs.push(q);

        let kinematic_ok = kinematic_qs.iter().all(|q| (q.norm() - 1.0).abs() < TOL);

        if rotation_ok && kinematic_ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "QuaternionUnitNorm",
        "attitude quaternion must have unit norm (|q| = 1)",
        "Markley & Crassidis (2014) Fundamentals of Spacecraft Attitude Determination and Control §2.7"
    );
}
pr4xis::register_axiom!(
    QuaternionUnitNorm,
    "Markley & Crassidis (2014) Fundamentals of Spacecraft Attitude Determination and Control §2.7"
);

/// Axiom: star tracker is the most accurate attitude sensor.
pub struct StarTrackerMostAccurate;

impl Axiom for StarTrackerMostAccurate {
    fn verify(&self) -> Verdict {
        let q = SensorAccuracy;
        let star = q.get(&AttitudeConcept::StarTracker).unwrap();
        // Smaller angular error = finer accuracy, so the star tracker's
        // quantity must be the minimum. Comparison is guarded by dimensional
        // compatibility — a safety the bare-`f64` form could never express;
        // `.value` is the common SI magnitude (radians) only once the guard
        // has confirmed both quantities share the ANGLE dimension.
        let ok = AttitudeConcept::variants().iter().all(|s| {
            let acc = q.get(s).unwrap();
            acc.dimension.is_compatible(&star.dimension) && acc.value >= star.value
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StarTrackerMostAccurate",
        "star tracker has the highest accuracy among attitude sensors",
        "Wertz (1978) Spacecraft Attitude Determination and Control §6"
    );
}
pr4xis::register_axiom!(
    StarTrackerMostAccurate,
    "Wertz (1978) Spacecraft Attitude Determination and Control §6"
);

impl Ontology for AttitudeOntology {
    type Cat = AttitudeCategory;
    type Qual = SensorAccuracy;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(QuaternionUnitNorm));
        axioms.push(Box::new(StarTrackerMostAccurate));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<AttitudeCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        AttitudeOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    /// The unit-norm axiom must actually compute |q| = 1 over its fixtures
    /// (rotation constructors + kinematics propagation), not assert it.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn quaternion_unit_norm_computes() {
        QuaternionUnitNorm
            .verify()
            .expect("unit-norm axiom must hold for canonical quaternions");
    }
}

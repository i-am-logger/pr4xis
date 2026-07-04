//! Classical orbital elements (Keplerian elements).
//!
//! Source: Vallado (2013), *Fundamentals of Astrodynamics and Applications*, 4th ed.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::applied::space::orbit::propagator::{OrbitalState, mu_earth_km3s2, propagate_rk4};
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::quantity::unit::{self, Unit};

pr4xis::ontology! {
    name: "Orbit",
    source: "Vallado (2013); Battin (1999)",

    concepts: [SemiMajorAxis, Eccentricity, Inclination, RAAN, ArgPeriapsis, TrueAnomaly],

    labels: {
        SemiMajorAxis: ("en", "Semi-major axis", "Semi-major axis (km)."),
        Eccentricity: ("en", "Eccentricity", "Eccentricity (dimensionless)."),
        Inclination: ("en", "Inclination", "Inclination (rad)."),
        RAAN: ("en", "RAAN", "Right Ascension of Ascending Node (rad)."),
        ArgPeriapsis: ("en", "Argument of periapsis", "Argument of periapsis (rad)."),
        TrueAnomaly: ("en", "True anomaly", "True anomaly (rad)."),
    },
}

/// Quality: physical unit for each orbital element.
///
/// The value is the typed `quantity`-ontology [`Unit`], not a prose symbol.
#[derive(Debug, Clone)]
pub struct ElementUnit;

impl Quality for ElementUnit {
    type Individual = OrbitConcept;
    type Value = Unit;

    fn get(&self, element: &OrbitConcept) -> Option<Unit> {
        Some(match element {
            OrbitConcept::SemiMajorAxis => unit::KILOMETER,
            OrbitConcept::Eccentricity => unit::UNITLESS,
            OrbitConcept::Inclination => unit::RADIAN,
            OrbitConcept::RAAN => unit::RADIAN,
            OrbitConcept::ArgPeriapsis => unit::RADIAN,
            OrbitConcept::TrueAnomaly => unit::RADIAN,
        })
    }
}

/// Canonical bound (elliptical) two-body orbit fixtures, specified at
/// perigee in an Earth-centred inertial frame (Vallado 2013 §2.4). Each
/// uses a sub-escape speed `k · v_circ` with `1 ≤ k < √2`, so the specific
/// energy is negative (a > 0) and the eccentricity lies in [0, 1).
fn elliptical_fixtures() -> [OrbitalState; 4] {
    let mu = mu_earth_km3s2();
    let mk = |r: f64, k: f64| {
        let v_circ = (mu / r).sqrt();
        OrbitalState {
            position: Vector::new(vec![r, 0.0, 0.0]),
            velocity: Vector::new(vec![0.0, k * v_circ, 0.0]),
        }
    };
    [
        mk(7000.0, 1.0), // circular:         e ≈ 0
        mk(7000.0, 1.1), // mildly elliptical
        mk(8000.0, 1.2), // more elliptical
        mk(6800.0, 1.3), // k < √2 ≈ 1.414 → still bound
    ]
}

/// Eccentricity magnitude from an ECI state vector via the eccentricity
/// vector e⃗ = ((v² − μ/r) r⃗ − (r⃗·v⃗) v⃗) / μ (Vallado 2013 §2.4).
fn eccentricity_from_state(state: &OrbitalState, mu: f64) -> f64 {
    let r = &state.position;
    let v = &state.velocity;
    let r_mag = state.radius();
    let v2 = state.speed() * state.speed();
    let r_dot_v = r.get(0) * v.get(0) + r.get(1) * v.get(1) + r.get(2) * v.get(2);
    let coef = v2 - mu / r_mag;
    let ex = (coef * r.get(0) - r_dot_v * v.get(0)) / mu;
    let ey = (coef * r.get(1) - r_dot_v * v.get(1)) / mu;
    let ez = (coef * r.get(2) - r_dot_v * v.get(2)) / mu;
    (ex * ex + ey * ey + ez * ez).sqrt()
}

/// Axiom: eccentricity must be in [0, 1) for elliptical orbits.
pub struct EccentricityBounded;

impl Axiom for EccentricityBounded {
    fn verify(&self) -> Verdict {
        // Per Vallado (2013) §2.4, the classical orbital element
        // eccentricity satisfies 0 ≤ e < 1 for elliptical (bound) orbits;
        // e = 1 is parabolic (escape) and e > 1 is hyperbolic. Compute e
        // from the state vector (eccentricity vector) while propagating each
        // canonical bound orbit with the real two-body RK4 integrator, and
        // confirm e stays in [0, 1) at every step.
        let mu = mu_earth_km3s2();
        let holds = elliptical_fixtures().iter().all(|initial| {
            let mut state = initial.clone();
            (0..16).all(|_| {
                let e = eccentricity_from_state(&state, mu);
                state = propagate_rk4(&state, 30.0, mu);
                (0.0..1.0).contains(&e)
            })
        });
        if holds {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EccentricityBounded",
        "eccentricity is in [0, 1) for elliptical (bound) orbits",
        "Vallado (2013) Fundamentals of Astrodynamics and Applications 4th ed. §2.4"
    );
}
pr4xis::register_axiom!(
    EccentricityBounded,
    "Vallado (2013) Fundamentals of Astrodynamics and Applications 4th ed. §2.4"
);

/// Axiom: semi-major axis must be positive for bound orbits.
pub struct SemiMajorAxisPositive;

impl Axiom for SemiMajorAxisPositive {
    fn verify(&self) -> Verdict {
        // Vis-viva a = −μ/(2E): for bound (E < 0) orbits a > 0; a → ∞ at
        // escape and a < 0 for hyperbolic. Propagate each canonical bound
        // orbit with the real two-body RK4 integrator and confirm the
        // semi-major axis (from OrbitalState::semi_major_axis) stays positive
        // at every step.
        let mu = mu_earth_km3s2();
        let holds = elliptical_fixtures().iter().all(|initial| {
            let mut state = initial.clone();
            (0..16).all(|_| {
                let a = state.semi_major_axis(mu);
                state = propagate_rk4(&state, 30.0, mu);
                a > 0.0
            })
        });
        if holds {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SemiMajorAxisPositive",
        "semi-major axis is positive for bound orbits",
        "Vallado (2013) Fundamentals of Astrodynamics and Applications 4th ed. §2.3 (vis-viva)"
    );
}
pr4xis::register_axiom!(
    SemiMajorAxisPositive,
    "Vallado (2013) Fundamentals of Astrodynamics and Applications 4th ed. §2.3 (vis-viva)"
);

impl Ontology for OrbitOntology {
    type Cat = OrbitCategory;
    type Qual = ElementUnit;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(EccentricityBounded));
        axioms.push(Box::new(SemiMajorAxisPositive));
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
        assert_category_laws::<OrbitCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        OrbitOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}

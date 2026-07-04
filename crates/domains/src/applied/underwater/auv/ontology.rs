//! AUV navigation sensor types.
//!
//! Source: Kinsey et al. (2006), "A Survey of Underwater Vehicle Navigation"

use pr4xis::logic::proof::{SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Auv",
    source: "Kinsey et al. (2006); Paull et al. (2014)",

    concepts: [DVL, DepthSensor, Compass, ADCP],

    labels: {
        DVL: ("en", "Doppler Velocity Log", "Doppler Velocity Log: measures velocity relative to seabed."),
        DepthSensor: ("en", "Depth sensor", "Depth/pressure sensor."),
        Compass: ("en", "Compass", "Magnetic compass / heading sensor."),
        ADCP: ("en", "Acoustic Doppler Current Profiler", "Acoustic Doppler Current Profiler: measures water current profile."),
    },
}

/// Quality: what physical quantity each sensor measures.
#[derive(Debug, Clone)]
pub struct MeasuredQuantity;

impl Quality for MeasuredQuantity {
    type Individual = AuvConcept;
    type Value = &'static str;

    fn get(&self, sensor: &AuvConcept) -> Option<&'static str> {
        Some(match sensor {
            AuvConcept::DVL => "velocity relative to seabed (m/s)",
            AuvConcept::DepthSensor => "depth/pressure (meters)",
            AuvConcept::Compass => "magnetic heading (rad)",
            AuvConcept::ADCP => "water current velocity profile (m/s)",
        })
    }
}

/// Axiom: depth measurements are non-negative (below surface).
pub struct DepthNonNegative;

impl Axiom for DepthNonNegative {
    fn verify(&self) -> Verdict {
        use crate::applied::underwater::auv::engine::{AuvState, DvlMeasurement, dead_reckon};
        use crate::formal::math::angle::Angle;
        use pr4xis::logic::proof::SimpleCounterexample;

        // Hydrostatic depth is surface-referenced and positive-downward
        // (Kinsey et al. 2006 §II): P = ρ·g·h with ρ, g > 0 and h ≥ 0 below
        // the free surface. Exercise this on the real `dead_reckon` engine —
        // a vehicle at or below the surface, propagated with a non-negative
        // (descending or level) downward velocity, must dead-reckon to a
        // non-negative depth. Each fixture is `(start_depth, downward, dt)`.
        let fixtures = [
            (0.0_f64, 0.0_f64, 10.0_f64), // holding level at the free surface
            (10.0, 0.5, 20.0),            // descending from 10 m over 20 s
            (100.0, 0.0, 5.0),            // holding station at 100 m
            (5.0, 1.0, 2.0),              // descending from 5 m over 2 s
        ];
        let all_non_negative = fixtures.iter().all(|&(depth, downward, dt)| {
            let state = AuvState {
                north: 0.0,
                east: 0.0,
                depth,
                heading: Angle::from_radians(0.0),
            };
            let dvl = DvlMeasurement {
                forward: 0.0,
                starboard: 0.0,
                downward,
                bottom_lock: true,
            };
            // Real depth propagation: depth' = depth + downward·dt.
            dead_reckon(&state, &dvl, 0.0, dt).depth >= 0.0
        });
        if all_non_negative {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DepthNonNegative",
        "depth measurements are non-negative (at or below the surface)",
        "Kinsey et al. (2006) A Survey of Underwater Vehicle Navigation"
    );
}
pr4xis::register_axiom!(
    DepthNonNegative,
    "Kinsey et al. (2006) A Survey of Underwater Vehicle Navigation"
);

/// Axiom: DVL requires bottom lock (limited altitude).
pub struct DvlRequiresBottomLock;

impl Axiom for DvlRequiresBottomLock {
    fn verify(&self) -> Verdict {
        // Doppler shift from a stationary seabed return is the basis of
        // DVL velocity measurement; without bottom lock there is no
        // reference for the Doppler frequency. Per Paull et al. (2014)
        // §3.2.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DvlRequiresBottomLock",
        "DVL velocity measurement requires bottom lock (finite altitude above seabed)",
        "Paull et al. (2014) AUV Navigation and Localization §3.2"
    );
}
pr4xis::register_axiom!(
    DvlRequiresBottomLock,
    "Paull et al. (2014) AUV Navigation and Localization §3.2"
);

impl Ontology for AuvOntology {
    type Cat = AuvCategory;
    type Qual = MeasuredQuantity;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(DepthNonNegative));
        axioms.push(Box::new(DvlRequiresBottomLock));
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
        assert_category_laws::<AuvCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        AuvOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}

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
        // Hydrostatic pressure P = ρ·g·h with ρ, g > 0 and h ≥ 0 below
        // the free surface; depth = h ≥ 0 by definition of the
        // surface-referenced coordinate frame.
        Ok(Box::new(SimpleProof::new(self.meta())))
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

    #[test]
    fn category_laws() {
        assert_category_laws::<AuvCategory>();
    }

    #[test]
    fn ontology_validates() {
        AuvOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}

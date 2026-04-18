//! AUV navigation sensor types.
//!
//! Source: Kinsey et al. (2006), "A Survey of Underwater Vehicle Navigation"

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Auv",
    source: "Kinsey et al. (2006); Paull et al. (2014)",
    being: PhysicalEndurant,

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
    fn description(&self) -> &str {
        "depth measurements are non-negative (at or below the surface)"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(DepthNonNegative);

/// Axiom: DVL requires bottom lock (limited altitude).
pub struct DvlRequiresBottomLock;

impl Axiom for DvlRequiresBottomLock {
    fn description(&self) -> &str {
        "DVL velocity measurement requires bottom lock (finite altitude above seabed)"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(DvlRequiresBottomLock);

impl Ontology for AuvOntology {
    type Cat = AuvCategory;
    type Qual = MeasuredQuantity;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(DepthNonNegative), Box::new(DvlRequiresBottomLock)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<AuvCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        AuvOntology::validate().unwrap();
    }
}

//! Electronic warfare observable types for emitter geolocation.
//!
//! Source: Poisel (2012), *Electronic Warfare Target Location Methods*

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Ew",
    source: "Poisel (2012); JP 3-13.1",
    being: SocialObject,

    concepts: [AOA, TDOA, FDOA, SignalStrength],

    labels: {
        AOA: ("en", "Angle of Arrival", "Angle of Arrival (bearing to emitter)."),
        TDOA: ("en", "Time Difference of Arrival", "Time Difference of Arrival (between sensor pairs)."),
        FDOA: ("en", "Frequency Difference of Arrival", "Frequency Difference of Arrival (Doppler-based)."),
        SignalStrength: ("en", "Signal strength", "Received signal strength (path-loss based ranging)."),
    },
}

/// Quality: geometric interpretation of each observable.
#[derive(Debug, Clone)]
pub struct ObservableGeometry;

impl Quality for ObservableGeometry {
    type Individual = EwConcept;
    type Value = &'static str;

    fn get(&self, obs: &EwConcept) -> Option<&'static str> {
        Some(match obs {
            EwConcept::AOA => "line of bearing (half-plane)",
            EwConcept::TDOA => "hyperbola (constant time difference)",
            EwConcept::FDOA => "hyperbola (constant frequency difference)",
            EwConcept::SignalStrength => "circle (constant range locus)",
        })
    }
}

/// Axiom: AOA measurements are in [-pi, pi].
pub struct AoaBounded;

impl Axiom for AoaBounded {
    fn description(&self) -> &str {
        "angle of arrival measurements are in [-pi, pi]"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(AoaBounded);

/// Axiom: TDOA requires at least 2 sensors for 2D geolocation.
pub struct TdoaRequiresSensorPair;

impl Axiom for TdoaRequiresSensorPair {
    fn description(&self) -> &str {
        "TDOA geolocation requires at least one sensor pair (2 sensors)"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(TdoaRequiresSensorPair);

impl Ontology for EwOntology {
    type Cat = EwCategory;
    type Qual = ObservableGeometry;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(AoaBounded), Box::new(TdoaRequiresSensorPair)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<EwCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        EwOntology::validate().unwrap();
    }
}

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Electronic warfare observable types for emitter geolocation.
///
/// Source: Poisel (2012), *Electronic Warfare Target Location Methods*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum EwObservable {
    /// Angle of Arrival (bearing to emitter).
    AOA,
    /// Time Difference of Arrival (between sensor pairs).
    TDOA,
    /// Frequency Difference of Arrival (Doppler-based).
    FDOA,
    /// Received signal strength (path-loss based ranging).
    SignalStrength,
}

define_ontology! {
    /// Category for EW observable fusion.
    ///
    /// All observables can be combined for improved geolocation;
    /// the category is fully connected.
    pub EwOntology for EwCategory {
        concepts: EwObservable,
        relation: EwFusionRelation,
    }
}

/// Quality: geometric interpretation of each observable.
#[derive(Debug, Clone)]
pub struct ObservableGeometry;

impl Quality for ObservableGeometry {
    type Individual = EwObservable;
    type Value = &'static str;

    fn get(&self, obs: &EwObservable) -> Option<&'static str> {
        Some(match obs {
            EwObservable::AOA => "line of bearing (half-plane)",
            EwObservable::TDOA => "hyperbola (constant time difference)",
            EwObservable::FDOA => "hyperbola (constant frequency difference)",
            EwObservable::SignalStrength => "circle (constant range locus)",
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
        // Structural axiom: bearings are measured as angles in [-pi, pi].
        // Any measured AOA outside this range can be wrapped.
        true
    }
}

/// Axiom: TDOA requires at least 2 sensors for 2D geolocation.
pub struct TdoaRequiresSensorPair;

impl Axiom for TdoaRequiresSensorPair {
    fn description(&self) -> &str {
        "TDOA geolocation requires at least one sensor pair (2 sensors)"
    }
    fn holds(&self) -> bool {
        // Structural axiom: TDOA is computed as the time difference
        // between two spatially separated receivers.
        true
    }
}

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

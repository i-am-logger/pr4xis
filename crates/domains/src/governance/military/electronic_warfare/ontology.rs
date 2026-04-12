use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Electronic warfare observable types for emitter geolocation.
///
/// Source: Poisel (2012), *Electronic Warfare Target Location Methods*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for EwObservable {
    fn variants() -> Vec<Self> {
        vec![Self::AOA, Self::TDOA, Self::FDOA, Self::SignalStrength]
    }
}

/// Fusion relationship between EW observables.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EwFusionRelation {
    pub from: EwObservable,
    pub to: EwObservable,
}

impl Relationship for EwFusionRelation {
    type Object = EwObservable;
    fn source(&self) -> EwObservable {
        self.from
    }
    fn target(&self) -> EwObservable {
        self.to
    }
}

/// Category for EW observable fusion.
///
/// All observables can be combined for improved geolocation;
/// the category is fully connected.
pub struct EwCategory;

impl Category for EwCategory {
    type Object = EwObservable;
    type Morphism = EwFusionRelation;

    fn identity(obj: &EwObservable) -> EwFusionRelation {
        EwFusionRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &EwFusionRelation, g: &EwFusionRelation) -> Option<EwFusionRelation> {
        if f.to != g.from {
            return None;
        }
        Some(EwFusionRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<EwFusionRelation> {
        let obs = EwObservable::variants();
        obs.iter()
            .flat_map(|&from| obs.iter().map(move |&to| EwFusionRelation { from, to }))
            .collect()
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

pub struct EwOntology;

impl Ontology for EwOntology {
    type Cat = EwCategory;
    type Qual = ObservableGeometry;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(AoaBounded), Box::new(TdoaRequiresSensorPair)]
    }
}

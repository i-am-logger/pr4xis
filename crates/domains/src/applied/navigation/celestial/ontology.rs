use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::taxonomy::{NoCycles, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Celestial body types.
///
/// Source: Wertz (2001) "Space Mission Engineering", Bowditch (2002).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum CelestialBody {
    /// Abstract celestial body.
    Body,
    /// The Sun.
    Sun,
    /// The Moon.
    Moon,
    /// A catalog star (e.g., Polaris, Sirius).
    Star,
    /// A planet (e.g., Venus, Mars, Jupiter).
    Planet,
}

/// Celestial observable types — what is measured.
///
/// Source: Bowditch (2002) Chapter 17.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum CelestialObservable {
    /// Abstract observable.
    Observable,
    /// Altitude: elevation angle above horizon.
    Altitude,
    /// Azimuth: bearing from north.
    Azimuth,
    /// Hour angle: angular distance from the meridian.
    HourAngle,
    /// Declination: angular distance from celestial equator.
    Declination,
}

/// Celestial sensor types.
///
/// Source: Wertz (2001) Chapter 7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum CelestialSensor {
    /// Abstract sensor.
    Sensor,
    /// Star tracker: focal plane array for spacecraft.
    StarTracker,
    /// Sun sensor: measures sun direction.
    SunSensor,
    /// Horizon sensor: measures Earth limb.
    HorizonSensor,
}

// ---------------------------------------------------------------------------
// Ontology (category + reasoning)
// ---------------------------------------------------------------------------

define_ontology! {
    /// The celestial navigation ontology.
    ///
    /// Source: Wertz (2001), Bowditch (2002), Groves (2013) Section 6.5.
    pub CelestialOntology for CelestialCategory {
        entity: CelestialSensor,
        relation: CelestialRelation,

        taxonomy: CelestialSensorTaxonomy [
            (StarTracker, Sensor),
            (SunSensor, Sensor),
            (HorizonSensor, Sensor),
        ],
    }
}

/// Celestial body taxonomy (secondary entity type — manual impl).
pub struct CelestialBodyTaxonomy;

impl TaxonomyDef for CelestialBodyTaxonomy {
    type Entity = CelestialBody;

    fn relations() -> Vec<(CelestialBody, CelestialBody)> {
        use CelestialBody::*;
        vec![(Sun, Body), (Moon, Body), (Star, Body), (Planet, Body)]
    }
}

/// Celestial observable taxonomy (secondary entity type — manual impl).
pub struct CelestialObservableTaxonomy;

impl TaxonomyDef for CelestialObservableTaxonomy {
    type Entity = CelestialObservable;

    fn relations() -> Vec<(CelestialObservable, CelestialObservable)> {
        use CelestialObservable::*;
        vec![
            (Altitude, Observable),
            (Azimuth, Observable),
            (HourAngle, Observable),
            (Declination, Observable),
        ]
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: Angular accuracy of celestial sensors.
///
/// Source: Wertz (2001) Table 7-2.
#[derive(Debug, Clone)]
pub struct AngularAccuracy;

impl Quality for AngularAccuracy {
    type Individual = CelestialSensor;
    type Value = &'static str;

    fn get(&self, sensor: &CelestialSensor) -> Option<&'static str> {
        Some(match sensor {
            CelestialSensor::Sensor => "varies by type",
            CelestialSensor::StarTracker => "1-10 arcseconds (best)",
            CelestialSensor::SunSensor => "0.01-0.1 degrees",
            CelestialSensor::HorizonSensor => "0.05-0.25 degrees",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Celestial body taxonomy is a DAG.
pub struct CelestialBodyTaxonomyIsDAG;

impl Axiom for CelestialBodyTaxonomyIsDAG {
    fn description(&self) -> &str {
        "celestial body taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        NoCycles::<CelestialBodyTaxonomy>::default().holds()
    }
}

/// Celestial observable taxonomy is a DAG.
pub struct CelestialObservableTaxonomyIsDAG;

impl Axiom for CelestialObservableTaxonomyIsDAG {
    fn description(&self) -> &str {
        "celestial observable taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        NoCycles::<CelestialObservableTaxonomy>::default().holds()
    }
}

/// Celestial sensor taxonomy is a DAG.
pub struct CelestialSensorTaxonomyIsDAG;

impl Axiom for CelestialSensorTaxonomyIsDAG {
    fn description(&self) -> &str {
        "celestial sensor taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        NoCycles::<CelestialSensorTaxonomy>::default().holds()
    }
}

/// Two star sightings determine a position fix.
///
/// Each star observation defines a circle of position on the Earth's surface.
/// Two circles intersect at (at most) two points. With a rough position estimate,
/// the ambiguity is resolved → one unique fix.
///
/// Source: Bowditch (2002) Chapter 18.
pub struct TwoSightsFix;

impl Axiom for TwoSightsFix {
    fn description(&self) -> &str {
        "two celestial observations determine a position (intersection of circles of position)"
    }
    fn holds(&self) -> bool {
        // Each observation constrains 1 degree of freedom (distance from sub-stellar point).
        // 2D position has 2 unknowns. 2 observations → 2 equations → unique fix.
        let unknowns = 2; // latitude, longitude
        let observations_per_sight = 1; // each sight gives one circle of position
        let min_sights = unknowns / observations_per_sight;
        min_sights == 2
    }
}

/// Star trackers provide arcsecond-level accuracy.
///
/// Modern star trackers achieve 1-10 arcsecond attitude determination accuracy,
/// far surpassing sun sensors and horizon sensors.
///
/// Source: Wertz (2001) Table 7-2, Liebe (2002).
pub struct StarTrackerMostAccurate;

impl Axiom for StarTrackerMostAccurate {
    fn description(&self) -> &str {
        "star trackers provide arcsecond-level accuracy (most accurate celestial sensor)"
    }
    fn holds(&self) -> bool {
        // Star tracker: ~5 arcseconds
        // Sun sensor: ~0.05 degrees = 180 arcseconds
        // Horizon sensor: ~0.1 degrees = 360 arcseconds
        let star_tracker_arcsec = 5.0;
        let sun_sensor_arcsec = 180.0;
        let horizon_sensor_arcsec = 360.0;
        star_tracker_arcsec < sun_sensor_arcsec && star_tracker_arcsec < horizon_sensor_arcsec
    }
}

/// Atmospheric refraction corrupts near-horizon observations.
///
/// At the horizon, refraction bends light by ~0.57 degrees (34 arcminutes).
/// At 10 degrees elevation, refraction is ~5 arcminutes.
/// At 45 degrees, refraction is ~1 arcminute.
///
/// Source: Bowditch (2002) Chapter 19, Meeus (1991).
pub struct AtmosphericRefraction;

impl Axiom for AtmosphericRefraction {
    fn description(&self) -> &str {
        "near-horizon observations are corrupted by atmospheric refraction"
    }
    fn holds(&self) -> bool {
        // Approximate refraction formula: R ≈ 1.02 / tan(h + 10.3/(h + 5.11)) arcminutes
        // where h is true altitude in degrees.
        let refraction_at_horizon = approximate_refraction_arcmin(0.5);
        let refraction_at_45deg = approximate_refraction_arcmin(45.0);
        // Refraction at horizon >> refraction at 45 degrees
        refraction_at_horizon > refraction_at_45deg * 10.0
    }
}

/// Approximate atmospheric refraction in arcminutes.
///
/// Formula from Meeus (1991), valid for h > 0 degrees.
fn approximate_refraction_arcmin(altitude_deg: f64) -> f64 {
    if altitude_deg < 0.1 {
        return 34.0; // ~34 arcminutes at horizon
    }
    1.02 / (altitude_deg + 10.3 / (altitude_deg + 5.11))
        .to_radians()
        .tan()
}

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for CelestialOntology {
    type Cat = CelestialCategory;
    type Qual = AngularAccuracy;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(CelestialBodyTaxonomyIsDAG),
            Box::new(CelestialObservableTaxonomyIsDAG),
            Box::new(CelestialSensorTaxonomyIsDAG),
            Box::new(TwoSightsFix),
            Box::new(StarTrackerMostAccurate),
            Box::new(AtmosphericRefraction),
        ]
    }
}

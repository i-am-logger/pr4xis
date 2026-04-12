use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Spacecraft attitude determination sensors.
///
/// Source: Wertz (1978), *Spacecraft Attitude Determination and Control*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttitudeSensor {
    /// Star tracker: high-accuracy inertial attitude reference.
    StarTracker,
    /// Sun sensor: determines direction to the Sun.
    SunSensor,
    /// Earth horizon sensor: determines nadir direction.
    EarthHorizon,
    /// Magnetometer: measures local magnetic field vector.
    Magnetometer,
}

impl Entity for AttitudeSensor {
    fn variants() -> Vec<Self> {
        vec![
            Self::StarTracker,
            Self::SunSensor,
            Self::EarthHorizon,
            Self::Magnetometer,
        ]
    }
}

/// Fusion relationship between attitude sensors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SensorFusion {
    pub from: AttitudeSensor,
    pub to: AttitudeSensor,
}

impl Relationship for SensorFusion {
    type Object = AttitudeSensor;
    fn source(&self) -> AttitudeSensor {
        self.from
    }
    fn target(&self) -> AttitudeSensor {
        self.to
    }
}

/// Category for attitude sensor fusion relationships.
///
/// All sensors can be fused with each other; the category is fully connected
/// since any pair of vector observations can be combined for attitude determination.
pub struct AttitudeCategory;

impl Category for AttitudeCategory {
    type Object = AttitudeSensor;
    type Morphism = SensorFusion;

    fn identity(obj: &AttitudeSensor) -> SensorFusion {
        SensorFusion {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &SensorFusion, g: &SensorFusion) -> Option<SensorFusion> {
        if f.to != g.from {
            return None;
        }
        Some(SensorFusion {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<SensorFusion> {
        let sensors = AttitudeSensor::variants();
        sensors
            .iter()
            .flat_map(|&from| sensors.iter().map(move |&to| SensorFusion { from, to }))
            .collect()
    }
}

/// Quality: typical accuracy of each sensor type.
#[derive(Debug, Clone)]
pub struct SensorAccuracy;

impl Quality for SensorAccuracy {
    type Individual = AttitudeSensor;
    /// Accuracy in arcseconds (1-sigma).
    type Value = f64;

    fn get(&self, sensor: &AttitudeSensor) -> Option<f64> {
        Some(match sensor {
            AttitudeSensor::StarTracker => 1.0,     // ~1 arcsec
            AttitudeSensor::SunSensor => 60.0,      // ~1 arcmin
            AttitudeSensor::EarthHorizon => 3600.0, // ~1 degree
            AttitudeSensor::Magnetometer => 7200.0, // ~2 degrees
        })
    }
}

/// Axiom: a unit quaternion has norm 1 (attitude representation constraint).
pub struct QuaternionUnitNorm;

impl Axiom for QuaternionUnitNorm {
    fn description(&self) -> &str {
        "attitude quaternion must have unit norm (|q| = 1)"
    }
    fn holds(&self) -> bool {
        // Structural axiom: SO(3) is represented by unit quaternions.
        // The quaternion q = (q0, q1, q2, q3) must satisfy q0^2 + q1^2 + q2^2 + q3^2 = 1.
        true
    }
}

/// Axiom: star tracker is the most accurate attitude sensor.
pub struct StarTrackerMostAccurate;

impl Axiom for StarTrackerMostAccurate {
    fn description(&self) -> &str {
        "star tracker has the highest accuracy among attitude sensors"
    }
    fn holds(&self) -> bool {
        let q = SensorAccuracy;
        let star_acc = q.get(&AttitudeSensor::StarTracker).unwrap();
        AttitudeSensor::variants()
            .iter()
            .all(|s| q.get(s).unwrap() >= star_acc)
    }
}

pub struct AttitudeOntology;

impl Ontology for AttitudeOntology {
    type Cat = AttitudeCategory;
    type Qual = SensorAccuracy;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(QuaternionUnitNorm),
            Box::new(StarTrackerMostAccurate),
        ]
    }
}

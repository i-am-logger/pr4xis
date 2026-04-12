use pr4xis::ontology::reasoning::taxonomy::{self, NoCycles, TaxonomyCategory, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::technology::sensor_fusion::sensor::modality::SensorType;

// ---------------------------------------------------------------------------
// Taxonomy: is-a hierarchy using praxis reasoning
// ---------------------------------------------------------------------------

pub struct SensorTaxonomy;

impl TaxonomyDef for SensorTaxonomy {
    type Entity = SensorType;

    fn relations() -> Vec<(SensorType, SensorType)> {
        use SensorType::*;
        vec![
            (ProprioceptiveSensor, Sensor),
            (ExteroceptiveSensor, Sensor),
            (ActiveSensor, Sensor),
            (PassiveSensor, Sensor),
            (Accelerometer, ProprioceptiveSensor),
            (Gyroscope, ProprioceptiveSensor),
            (Magnetometer, ProprioceptiveSensor),
            (Barometer, ProprioceptiveSensor),
            (DepthSensor, ProprioceptiveSensor),
            (GnssReceiver, ExteroceptiveSensor),
            (GnssReceiver, PassiveSensor),
            (StarTracker, ExteroceptiveSensor),
            (StarTracker, PassiveSensor),
            (Radar, ExteroceptiveSensor),
            (Radar, ActiveSensor),
            (LiDAR, ExteroceptiveSensor),
            (LiDAR, ActiveSensor),
            (Sonar, ExteroceptiveSensor),
            (Sonar, ActiveSensor),
            (DopplerVelocityLog, ExteroceptiveSensor),
            (DopplerVelocityLog, ActiveSensor),
            (Camera, ExteroceptiveSensor),
            (Camera, PassiveSensor),
            (InfraredCamera, ExteroceptiveSensor),
            (InfraredCamera, PassiveSensor),
            (DepthCamera, ExteroceptiveSensor),
            (DepthCamera, ActiveSensor),
            (IMU, ProprioceptiveSensor),
            (AHRS, ProprioceptiveSensor),
            (INS, ProprioceptiveSensor),
        ]
    }
}

// ---------------------------------------------------------------------------
// Mereology: has-a
// ---------------------------------------------------------------------------

fn mereology_relations() -> Vec<(SensorType, SensorType)> {
    use SensorType::*;
    vec![
        (IMU, Accelerometer),
        (IMU, Gyroscope),
        (AHRS, Accelerometer),
        (AHRS, Gyroscope),
        (AHRS, Magnetometer),
        (INS, Accelerometer),
        (INS, Gyroscope),
    ]
}

pub fn parts_of(whole: SensorType) -> Vec<SensorType> {
    mereology_relations()
        .into_iter()
        .filter(|(w, _)| *w == whole)
        .map(|(_, p)| p)
        .collect()
}

// ---------------------------------------------------------------------------
// Quality
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct IsProprioceptive;

impl Quality for IsProprioceptive {
    type Individual = SensorType;
    type Value = bool;

    fn get(&self, s: &SensorType) -> Option<bool> {
        Some(taxonomy::is_a::<SensorTaxonomy>(
            s,
            &SensorType::ProprioceptiveSensor,
        ))
    }
}

// ---------------------------------------------------------------------------
// Axioms — using praxis reasoning
// ---------------------------------------------------------------------------

pub struct TaxonomyIsDAG;

impl Axiom for TaxonomyIsDAG {
    fn description(&self) -> &str {
        "sensor taxonomy is a DAG (no cycles)"
    }
    fn holds(&self) -> bool {
        NoCycles::<SensorTaxonomy>::default().holds()
    }
}

pub struct AccelerometerIsSensor;

impl Axiom for AccelerometerIsSensor {
    fn description(&self) -> &str {
        "Accelerometer is-a Sensor (transitive via ProprioceptiveSensor)"
    }
    fn holds(&self) -> bool {
        taxonomy::is_a::<SensorTaxonomy>(&SensorType::Accelerometer, &SensorType::Sensor)
    }
}

pub struct ImuComposition;

impl Axiom for ImuComposition {
    fn description(&self) -> &str {
        "IMU has-a Accelerometer and has-a Gyroscope"
    }
    fn holds(&self) -> bool {
        let parts = parts_of(SensorType::IMU);
        parts.contains(&SensorType::Accelerometer) && parts.contains(&SensorType::Gyroscope)
    }
}

pub struct RadarDualClassification;

impl Axiom for RadarDualClassification {
    fn description(&self) -> &str {
        "Radar is-a ExteroceptiveSensor AND is-a ActiveSensor"
    }
    fn holds(&self) -> bool {
        taxonomy::is_a::<SensorTaxonomy>(&SensorType::Radar, &SensorType::ExteroceptiveSensor)
            && taxonomy::is_a::<SensorTaxonomy>(&SensorType::Radar, &SensorType::ActiveSensor)
    }
}

pub struct CameraIsPassive;

impl Axiom for CameraIsPassive {
    fn description(&self) -> &str {
        "Camera is-a PassiveSensor and is NOT ActiveSensor"
    }
    fn holds(&self) -> bool {
        taxonomy::is_a::<SensorTaxonomy>(&SensorType::Camera, &SensorType::PassiveSensor)
            && !taxonomy::is_a::<SensorTaxonomy>(&SensorType::Camera, &SensorType::ActiveSensor)
    }
}

// ---------------------------------------------------------------------------
// Ontology — uses TaxonomyCategory from praxis reasoning
// ---------------------------------------------------------------------------

pub struct SensorOntology;

impl Ontology for SensorOntology {
    type Cat = TaxonomyCategory<SensorTaxonomy>;
    type Qual = IsProprioceptive;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(TaxonomyIsDAG),
            Box::new(AccelerometerIsSensor),
            Box::new(ImuComposition),
            Box::new(RadarDualClassification),
            Box::new(CameraIsPassive),
        ]
    }
}

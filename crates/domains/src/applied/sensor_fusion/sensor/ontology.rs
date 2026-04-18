#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::applied::sensor_fusion::sensor::modality::SensorType;

// ---------------------------------------------------------------------------
// Ontology (category + reasoning)
// ---------------------------------------------------------------------------

define_ontology! {
    pub SensorOntology for SensorCategory {
        entity: SensorType,
        relation: SensorRelation,
        being: PhysicalEndurant,
        source: "Groves (2013); Bar-Shalom et al. (2001)",

        taxonomy: SensorTaxonomy [
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
        ],

        mereology: SensorMereology [
            (IMU, Accelerometer),
            (IMU, Gyroscope),
            (AHRS, Accelerometer),
            (AHRS, Gyroscope),
            (AHRS, Magnetometer),
            (INS, Accelerometer),
            (INS, Gyroscope),
        ],
    }
}

// ---------------------------------------------------------------------------
// Mereology helper
// ---------------------------------------------------------------------------

pub fn parts_of(whole: SensorType) -> Vec<SensorType> {
    use pr4xis::ontology::reasoning::mereology::MereologyDef;
    SensorMereology::relations()
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
// Axioms
// ---------------------------------------------------------------------------

pub struct AccelerometerIsSensor;

impl Axiom for AccelerometerIsSensor {
    fn description(&self) -> &str {
        "Accelerometer is-a Sensor (transitive via ProprioceptiveSensor)"
    }
    fn holds(&self) -> bool {
        taxonomy::is_a::<SensorTaxonomy>(&SensorType::Accelerometer, &SensorType::Sensor)
    }
}
pr4xis::register_axiom!(AccelerometerIsSensor);

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
pr4xis::register_axiom!(ImuComposition);

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
pr4xis::register_axiom!(RadarDualClassification);

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
pr4xis::register_axiom!(CameraIsPassive);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for SensorOntology {
    type Cat = SensorCategory;
    type Qual = IsProprioceptive;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(AccelerometerIsSensor),
            Box::new(ImuComposition),
            Box::new(RadarDualClassification),
            Box::new(CameraIsPassive),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<SensorCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        SensorOntology::validate().unwrap();
    }
}

use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::sensor_fusion::sensor::ontology::*;

#[test]
fn sensor_category_laws() {
    assert_category_laws::<SensorCategory>();
}

#[test]
fn sensor_ontology_validates() {
    SensorOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[test]
fn accelerometer_is_sensor() {
    assert!(AccelerometerIsSensor.verify().is_ok());
}

#[test]
fn imu_composition() {
    assert!(ImuComposition.verify().is_ok());
}

#[test]
fn radar_dual_classification() {
    assert!(RadarDualClassification.verify().is_ok());
}

#[test]
fn camera_is_passive() {
    assert!(CameraIsPassive.verify().is_ok());
}

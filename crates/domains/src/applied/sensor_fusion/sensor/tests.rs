use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::sensor_fusion::sensor::ontology::*;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn sensor_category_laws() {
    assert_category_laws::<SensorCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn sensor_ontology_validates() {
    SensorOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn accelerometer_is_sensor() {
    assert!(AccelerometerIsSensor.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn imu_composition() {
    assert!(ImuComposition.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn radar_dual_classification() {
    assert!(RadarDualClassification.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn camera_is_passive() {
    assert!(CameraIsPassive.verify().is_ok());
}

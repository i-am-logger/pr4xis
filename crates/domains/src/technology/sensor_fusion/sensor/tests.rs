use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
use pr4xis::ontology::{Axiom, Ontology};

use crate::technology::sensor_fusion::sensor::ontology::*;

#[test]
fn sensor_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<SensorTaxonomy>>().unwrap();
}

#[test]
fn sensor_ontology_validates() {
    SensorOntology::validate().unwrap();
}

#[test]
fn taxonomy_is_dag() {
    assert!(TaxonomyIsDAG.holds());
}

#[test]
fn accelerometer_is_sensor() {
    assert!(AccelerometerIsSensor.holds());
}

#[test]
fn imu_composition() {
    assert!(ImuComposition.holds());
}

#[test]
fn radar_dual_classification() {
    assert!(RadarDualClassification.holds());
}

#[test]
fn camera_is_passive() {
    assert!(CameraIsPassive.holds());
}

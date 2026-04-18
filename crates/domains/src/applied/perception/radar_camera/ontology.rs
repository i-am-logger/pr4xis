//! Fusion pipeline stages for radar+camera sensor fusion.
//!
//! Source: Nobis et al. (2019), "A Deep Learning-based Radar and Camera Sensor Fusion Architecture"

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "RadarCamera",
    source: "Nobis et al. (2019)",
    being: Process,

    concepts: [RadarDetection, CameraDetection, TemporalAlignment, SpatialAssociation, FusedOutput],

    labels: {
        RadarDetection: ("en", "Radar detection", "Raw radar detections (range, Doppler, azimuth)."),
        CameraDetection: ("en", "Camera detection", "Raw camera detections (bounding boxes)."),
        TemporalAlignment: ("en", "Temporal alignment", "Temporal alignment of radar and camera frames."),
        SpatialAssociation: ("en", "Spatial association", "Spatial association of radar targets with camera objects."),
        FusedOutput: ("en", "Fused output", "Fused output with range from radar and classification from camera."),
    },

    edges: [
        (RadarDetection, TemporalAlignment, Precedes),
        (CameraDetection, TemporalAlignment, Precedes),
        (TemporalAlignment, SpatialAssociation, Precedes),
        (SpatialAssociation, FusedOutput, Precedes),
    ],
}

#[derive(Debug, Clone)]
pub struct StageDescription;

impl Quality for StageDescription {
    type Individual = RadarCameraConcept;
    type Value = &'static str;

    fn get(&self, stage: &RadarCameraConcept) -> Option<&'static str> {
        Some(match stage {
            RadarCameraConcept::RadarDetection => "raw radar targets (range, Doppler, azimuth)",
            RadarCameraConcept::CameraDetection => "raw camera detections (bounding boxes)",
            RadarCameraConcept::TemporalAlignment => "radar and camera frames aligned in time",
            RadarCameraConcept::SpatialAssociation => {
                "radar targets associated with camera objects"
            }
            RadarCameraConcept::FusedOutput => "fused output with range + classification",
        })
    }
}

/// Axiom: both sensor modalities must be present before fusion.
pub struct BothModalitiesRequired;

impl Axiom for BothModalitiesRequired {
    fn description(&self) -> &str {
        "both radar and camera detections feed into temporal alignment"
    }
    fn holds(&self) -> bool {
        let morphisms = RadarCameraCategory::morphisms();
        let radar_to_align = morphisms.iter().any(|m| {
            m.from == RadarCameraConcept::RadarDetection
                && m.to == RadarCameraConcept::TemporalAlignment
        });
        let camera_to_align = morphisms.iter().any(|m| {
            m.from == RadarCameraConcept::CameraDetection
                && m.to == RadarCameraConcept::TemporalAlignment
        });
        radar_to_align && camera_to_align
    }
}
pr4xis::register_axiom!(BothModalitiesRequired);

/// Axiom: fused output is a terminal stage (no outgoing non-identity morphisms).
pub struct FusedOutputIsTerminal;

impl Axiom for FusedOutputIsTerminal {
    fn description(&self) -> &str {
        "fused output is the terminal stage of the pipeline"
    }
    fn holds(&self) -> bool {
        let morphisms = RadarCameraCategory::morphisms();
        !morphisms.iter().any(|m| {
            m.from == RadarCameraConcept::FusedOutput && m.to != RadarCameraConcept::FusedOutput
        })
    }
}
pr4xis::register_axiom!(FusedOutputIsTerminal);

impl Ontology for RadarCameraOntology {
    type Cat = RadarCameraCategory;
    type Qual = StageDescription;

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(BothModalitiesRequired),
            Box::new(FusedOutputIsTerminal),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<RadarCameraCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        RadarCameraOntology::validate().unwrap();
    }
}

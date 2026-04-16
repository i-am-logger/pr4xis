use pr4xis::category::{Category, Entity};
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Fusion pipeline stages for radar+camera sensor fusion.
///
/// Source: Nobis et al. (2019), "A Deep Learning-based Radar and Camera Sensor Fusion Architecture"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum RadarCameraStage {
    /// Raw radar detections (range, Doppler, azimuth).
    RadarDetection,
    /// Raw camera detections (bounding boxes).
    CameraDetection,
    /// Temporal alignment of radar and camera frames.
    TemporalAlignment,
    /// Spatial association of radar targets with camera objects.
    SpatialAssociation,
    /// Fused output with range from radar and classification from camera.
    FusedOutput,
}

define_ontology! {
    /// Category for radar-camera fusion pipeline.
    ///
    /// Both RadarDetection and CameraDetection feed into TemporalAlignment,
    /// then SpatialAssociation, then FusedOutput.
    pub RadarCameraOntologyMeta for RadarCameraCategory {
        concepts: RadarCameraStage,
        relation: RadarCameraStep,
        kind: RadarCameraStepKind,
        kinds: [
            /// Pipeline progression step.
            Precedes,
        ],
        edges: [
            // Radar and camera both feed into temporal alignment
            (RadarDetection, TemporalAlignment, Precedes),
            (CameraDetection, TemporalAlignment, Precedes),
            // Common path
            (TemporalAlignment, SpatialAssociation, Precedes),
            (SpatialAssociation, FusedOutput, Precedes),
        ],
        composed: [
            // Transitive closures
            (RadarDetection, SpatialAssociation),
            (RadarDetection, FusedOutput),
            (CameraDetection, SpatialAssociation),
            (CameraDetection, FusedOutput),
            (TemporalAlignment, FusedOutput),
        ],
        being: Process,
        source: "Nobis et al. (2019)",
    }
}

#[derive(Debug, Clone)]
pub struct StageDescription;

impl Quality for StageDescription {
    type Individual = RadarCameraStage;
    type Value = &'static str;

    fn get(&self, stage: &RadarCameraStage) -> Option<&'static str> {
        Some(match stage {
            RadarCameraStage::RadarDetection => "raw radar targets (range, Doppler, azimuth)",
            RadarCameraStage::CameraDetection => "raw camera detections (bounding boxes)",
            RadarCameraStage::TemporalAlignment => "radar and camera frames aligned in time",
            RadarCameraStage::SpatialAssociation => "radar targets associated with camera objects",
            RadarCameraStage::FusedOutput => "fused output with range + classification",
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
            m.from == RadarCameraStage::RadarDetection
                && m.to == RadarCameraStage::TemporalAlignment
        });
        let camera_to_align = morphisms.iter().any(|m| {
            m.from == RadarCameraStage::CameraDetection
                && m.to == RadarCameraStage::TemporalAlignment
        });
        radar_to_align && camera_to_align
    }
}

/// Axiom: fused output is a terminal stage (no outgoing non-identity morphisms).
pub struct FusedOutputIsTerminal;

impl Axiom for FusedOutputIsTerminal {
    fn description(&self) -> &str {
        "fused output is the terminal stage of the pipeline"
    }
    fn holds(&self) -> bool {
        let morphisms = RadarCameraCategory::morphisms();
        !morphisms.iter().any(|m| {
            m.from == RadarCameraStage::FusedOutput && m.to != RadarCameraStage::FusedOutput
        })
    }
}

pub struct RadarCameraOntology;

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

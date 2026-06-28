//! Fusion pipeline stages for radar+camera sensor fusion.
//!
//! Source: Nobis et al. (2019), "A Deep Learning-based Radar and Camera Sensor Fusion Architecture"

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Category;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "RadarCamera",
    source: "Nobis et al. (2019)",

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
    fn verify(&self) -> Verdict {
        let morphisms = RadarCameraCategory::morphisms();
        let radar_to_align = morphisms.iter().any(|m| {
            m.from == RadarCameraConcept::RadarDetection
                && m.to == RadarCameraConcept::TemporalAlignment
        });
        let camera_to_align = morphisms.iter().any(|m| {
            m.from == RadarCameraConcept::CameraDetection
                && m.to == RadarCameraConcept::TemporalAlignment
        });
        if radar_to_align && camera_to_align {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BothModalitiesRequired",
        "both radar and camera detections feed into temporal alignment",
        "Nobis et al. (2019) A Deep Learning-based Radar and Camera Sensor Fusion Architecture"
    );
}
pr4xis::register_axiom!(
    BothModalitiesRequired,
    "Nobis et al. (2019) A Deep Learning-based Radar and Camera Sensor Fusion Architecture"
);

/// Axiom: fused output is a terminal stage (no outgoing non-identity morphisms).
pub struct FusedOutputIsTerminal;

impl Axiom for FusedOutputIsTerminal {
    fn verify(&self) -> Verdict {
        let morphisms = RadarCameraCategory::morphisms();
        let ok = !morphisms.iter().any(|m| {
            m.from == RadarCameraConcept::FusedOutput && m.to != RadarCameraConcept::FusedOutput
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FusedOutputIsTerminal",
        "fused output is the terminal stage of the pipeline",
        "Nobis et al. (2019) A Deep Learning-based Radar and Camera Sensor Fusion Architecture"
    );
}
pr4xis::register_axiom!(
    FusedOutputIsTerminal,
    "Nobis et al. (2019) A Deep Learning-based Radar and Camera Sensor Fusion Architecture"
);

impl Ontology for RadarCameraOntology {
    type Cat = RadarCameraCategory;
    type Qual = StageDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(BothModalitiesRequired));
        axioms.push(Box::new(FusedOutputIsTerminal));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<RadarCameraCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        RadarCameraOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}

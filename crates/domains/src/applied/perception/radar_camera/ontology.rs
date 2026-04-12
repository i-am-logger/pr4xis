use pr4xis::category::{Category, Entity, Relationship};
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RadarCameraStep {
    pub from: RadarCameraStage,
    pub to: RadarCameraStage,
}

impl Relationship for RadarCameraStep {
    type Object = RadarCameraStage;
    fn source(&self) -> RadarCameraStage {
        self.from
    }
    fn target(&self) -> RadarCameraStage {
        self.to
    }
}

/// Category for radar-camera fusion pipeline.
///
/// Both RadarDetection and CameraDetection feed into TemporalAlignment,
/// then SpatialAssociation, then FusedOutput. All transitive closures included.
pub struct RadarCameraCategory;

impl Category for RadarCameraCategory {
    type Object = RadarCameraStage;
    type Morphism = RadarCameraStep;

    fn identity(obj: &RadarCameraStage) -> RadarCameraStep {
        RadarCameraStep {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &RadarCameraStep, g: &RadarCameraStep) -> Option<RadarCameraStep> {
        if f.to != g.from {
            return None;
        }
        let step = RadarCameraStep {
            from: f.from,
            to: g.to,
        };
        if Self::morphisms().contains(&step) {
            Some(step)
        } else {
            None
        }
    }

    fn morphisms() -> Vec<RadarCameraStep> {
        use RadarCameraStage::*;
        let mut m = Vec::new();
        // Identities
        for s in Self::Object::variants() {
            m.push(Self::identity(&s));
        }
        // Radar path: RadarDetection -> TemporalAlignment -> SpatialAssociation -> FusedOutput
        m.push(RadarCameraStep {
            from: RadarDetection,
            to: TemporalAlignment,
        });
        // Camera path: CameraDetection -> TemporalAlignment -> SpatialAssociation -> FusedOutput
        m.push(RadarCameraStep {
            from: CameraDetection,
            to: TemporalAlignment,
        });
        // Common path
        m.push(RadarCameraStep {
            from: TemporalAlignment,
            to: SpatialAssociation,
        });
        m.push(RadarCameraStep {
            from: SpatialAssociation,
            to: FusedOutput,
        });
        // Transitive closures
        m.push(RadarCameraStep {
            from: RadarDetection,
            to: SpatialAssociation,
        });
        m.push(RadarCameraStep {
            from: RadarDetection,
            to: FusedOutput,
        });
        m.push(RadarCameraStep {
            from: CameraDetection,
            to: SpatialAssociation,
        });
        m.push(RadarCameraStep {
            from: CameraDetection,
            to: FusedOutput,
        });
        m.push(RadarCameraStep {
            from: TemporalAlignment,
            to: FusedOutput,
        });
        m
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

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(BothModalitiesRequired),
            Box::new(FusedOutputIsTerminal),
        ]
    }
}

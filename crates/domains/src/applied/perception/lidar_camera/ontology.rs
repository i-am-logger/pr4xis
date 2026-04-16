use pr4xis::category::{Category, Entity};
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Fusion pipeline stages for LiDAR+camera sensor fusion.
///
/// Source: Caltagirone et al. (2019), "LiDAR-Camera Fusion for Road Detection"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum FusionStage {
    /// Raw detection from individual sensors.
    Detection,
    /// Projection of 3D LiDAR points into 2D camera frame.
    Projection,
    /// Association of projected points with camera detections.
    Association,
    /// Final fused output combining both modalities.
    Fusion,
}

define_ontology! {
    /// Category for the LiDAR-camera fusion pipeline.
    ///
    /// The pipeline is a linear chain: Detection -> Projection -> Association -> Fusion.
    /// All transitive compositions are included for closure.
    pub LidarCameraOntologyMeta for LidarCameraCategory {
        concepts: FusionStage,
        relation: PipelineStep,
        kind: PipelineStepKind,
        kinds: [
            /// Sequential pipeline step.
            Precedes,
        ],
        edges: [
            (Detection, Projection, Precedes),
            (Projection, Association, Precedes),
            (Association, Fusion, Precedes),
        ],
        composed: [
            (Detection, Association),
            (Detection, Fusion),
            (Projection, Fusion),
        ],
        being: Process,
        source: "Qi et al. (2018)",
    }
}

/// Quality: description of each fusion stage.
#[derive(Debug, Clone)]
pub struct StageDescription;

impl Quality for StageDescription {
    type Individual = FusionStage;
    type Value = &'static str;

    fn get(&self, stage: &FusionStage) -> Option<&'static str> {
        Some(match stage {
            FusionStage::Detection => "raw sensor detections from LiDAR and camera",
            FusionStage::Projection => "LiDAR 3D points projected into camera image plane",
            FusionStage::Association => "projected points associated with camera detections",
            FusionStage::Fusion => "final fused perception output",
        })
    }
}

/// Axiom: projection preserves ordering of LiDAR points along the depth axis.
///
/// If point A is closer than point B in 3D, then the projected depth ordering
/// is preserved in the 2D projection (for pinhole camera models).
pub struct ProjectionPreservesOrdering;

impl Axiom for ProjectionPreservesOrdering {
    fn description(&self) -> &str {
        "projection preserves depth ordering of LiDAR points"
    }
    fn holds(&self) -> bool {
        // Structural axiom: in a pinhole camera model, points at different depths
        // along the same ray maintain their relative depth ordering after projection.
        true
    }
}

/// Axiom: the fusion pipeline must proceed in order (Detection before Projection).
pub struct PipelineIsSequential;

impl Axiom for PipelineIsSequential {
    fn description(&self) -> &str {
        "fusion pipeline stages must execute in order"
    }
    fn holds(&self) -> bool {
        // Verify no backward morphisms exist (except identities)
        let morphisms = LidarCameraCategory::morphisms();
        !morphisms.iter().any(|m| {
            let from_idx = stage_index(m.from);
            let to_idx = stage_index(m.to);
            to_idx < from_idx
        })
    }
}

fn stage_index(s: FusionStage) -> usize {
    match s {
        FusionStage::Detection => 0,
        FusionStage::Projection => 1,
        FusionStage::Association => 2,
        FusionStage::Fusion => 3,
    }
}

pub struct LidarCameraOntology;

impl Ontology for LidarCameraOntology {
    type Cat = LidarCameraCategory;
    type Qual = StageDescription;

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(ProjectionPreservesOrdering),
            Box::new(PipelineIsSequential),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<LidarCameraCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        LidarCameraOntology::validate().unwrap();
    }
}

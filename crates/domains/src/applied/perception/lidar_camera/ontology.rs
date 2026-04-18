//! Fusion pipeline stages for LiDAR+camera sensor fusion.
//!
//! Source: Caltagirone et al. (2019), "LiDAR-Camera Fusion for Road Detection"

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "LidarCamera",
    source: "Caltagirone et al. (2019); Qi et al. (2018)",
    being: Process,

    concepts: [Detection, Projection, Association, Fusion],

    labels: {
        Detection: ("en", "Detection", "Raw detection from individual sensors."),
        Projection: ("en", "Projection", "Projection of 3D LiDAR points into 2D camera frame."),
        Association: ("en", "Association", "Association of projected points with camera detections."),
        Fusion: ("en", "Fusion", "Final fused output combining both modalities."),
    },

    edges: [
        (Detection, Projection, Precedes),
        (Projection, Association, Precedes),
        (Association, Fusion, Precedes),
    ],
}

/// Quality: description of each fusion stage.
#[derive(Debug, Clone)]
pub struct StageDescription;

impl Quality for StageDescription {
    type Individual = LidarCameraConcept;
    type Value = &'static str;

    fn get(&self, stage: &LidarCameraConcept) -> Option<&'static str> {
        Some(match stage {
            LidarCameraConcept::Detection => "raw sensor detections from LiDAR and camera",
            LidarCameraConcept::Projection => "LiDAR 3D points projected into camera image plane",
            LidarCameraConcept::Association => "projected points associated with camera detections",
            LidarCameraConcept::Fusion => "final fused perception output",
        })
    }
}

/// Axiom: projection preserves ordering of LiDAR points along the depth axis.
pub struct ProjectionPreservesOrdering;

impl Axiom for ProjectionPreservesOrdering {
    fn description(&self) -> &str {
        "projection preserves depth ordering of LiDAR points"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(ProjectionPreservesOrdering);

/// Axiom: the fusion pipeline must proceed in order (Detection before Projection).
pub struct PipelineIsSequential;

impl Axiom for PipelineIsSequential {
    fn description(&self) -> &str {
        "fusion pipeline stages must execute in order"
    }
    fn holds(&self) -> bool {
        let morphisms = LidarCameraCategory::morphisms();
        !morphisms.iter().any(|m| {
            let from_idx = stage_index(m.from);
            let to_idx = stage_index(m.to);
            to_idx < from_idx
        })
    }
}
pr4xis::register_axiom!(PipelineIsSequential);

fn stage_index(s: LidarCameraConcept) -> usize {
    match s {
        LidarCameraConcept::Detection => 0,
        LidarCameraConcept::Projection => 1,
        LidarCameraConcept::Association => 2,
        LidarCameraConcept::Fusion => 3,
    }
}

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

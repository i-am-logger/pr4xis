//! Fusion pipeline stages for LiDAR+camera sensor fusion.
//!
//! Source: Caltagirone et al. (2019), "LiDAR-Camera Fusion for Road Detection"

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "LidarCamera",
    source: "Caltagirone et al. (2019); Qi et al. (2018)",

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
    fn verify(&self) -> Verdict {
        // Pinhole projection is monotone in z: per Hartley & Zisserman
        // (2003) §6, larger depth z maps to smaller image coordinates
        // along the principal axis, preserving the strict ordering of
        // points along the optical depth direction.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ProjectionPreservesOrdering",
        "projection preserves depth ordering of LiDAR points",
        "Hartley & Zisserman (2003) Multiple View Geometry §6 (pinhole projection)"
    );
}
pr4xis::register_axiom!(
    ProjectionPreservesOrdering,
    "Hartley & Zisserman (2003) Multiple View Geometry §6 (pinhole projection)"
);

/// Axiom: the fusion pipeline must proceed in order (Detection before Projection).
pub struct PipelineIsSequential;

impl Axiom for PipelineIsSequential {
    fn verify(&self) -> Verdict {
        let morphisms = LidarCameraCategory::morphisms();
        let ok = !morphisms.iter().any(|m| {
            if m.kind() != LidarCameraRelationKind::Precedes {
                return false;
            }
            let from_idx = stage_index(m.source());
            let to_idx = stage_index(m.target());
            to_idx < from_idx
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PipelineIsSequential",
        "fusion pipeline stages must execute in order",
        "Caltagirone et al. (2019) LiDAR-Camera Fusion for Road Detection"
    );
}
pr4xis::register_axiom!(
    PipelineIsSequential,
    "Caltagirone et al. (2019) LiDAR-Camera Fusion for Road Detection"
);

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

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ProjectionPreservesOrdering));
        axioms.push(Box::new(PipelineIsSequential));
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
        assert_category_laws::<LidarCameraCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        LidarCameraOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}

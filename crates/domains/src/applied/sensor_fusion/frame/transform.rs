use pr4xis::category::Arrow;
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use crate::applied::sensor_fusion::frame::reference::ReferenceFrame;

/// A coordinate transform between two reference frames.
///
/// This is the morphism in the FrameCategory. For category structure,
/// equality is based on source and target frames. The actual SE(3)
/// numerical transformation is handled by Pose.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FrameTransform {
    pub from: ReferenceFrame,
    pub to: ReferenceFrame,
}

impl FrameTransform {
    pub fn new(from: ReferenceFrame, to: ReferenceFrame) -> Self {
        Self { from, to }
    }
}

/// Relation-kind tag for the frame category.
///
/// Per OBO-RO (Smith 2005), every arrow carries a canonical kind. Frame
/// transforms are SE(3) coordinate changes — a single kind suffices
/// (Sola et al. 2018: A micro Lie theory for state estimation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameRelationKind {
    CoordinateTransform,
}

impl Arrow for FrameTransform {
    type Object = ReferenceFrame;
    type Kind = FrameRelationKind;

    fn source(&self) -> ReferenceFrame {
        self.from
    }

    fn target(&self) -> ReferenceFrame {
        self.to
    }

    fn kind(&self) -> FrameRelationKind {
        FrameRelationKind::CoordinateTransform
    }

    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("FrameTransform"),
            description: Label::new_static(
                "SE(3) coordinate transform between two reference frames (Sola et al. 2018)",
            ),
            citation: Citation::parse_static(
                "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems Ch. 2; Sola et al. (2018) A micro Lie theory for state estimation in robotics",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

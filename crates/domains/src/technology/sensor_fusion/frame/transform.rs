use pr4xis::category::Relationship;

use crate::technology::sensor_fusion::frame::reference::ReferenceFrame;

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

impl Relationship for FrameTransform {
    type Object = ReferenceFrame;

    fn source(&self) -> ReferenceFrame {
        self.from
    }

    fn target(&self) -> ReferenceFrame {
        self.to
    }
}

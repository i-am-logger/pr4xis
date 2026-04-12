use pr4xis::category::{Category, Entity};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::technology::sensor_fusion::frame::reference::ReferenceFrame;
use crate::technology::sensor_fusion::frame::transform::FrameTransform;

// ---------------------------------------------------------------------------
// Category: reference frames connected by transforms
// ---------------------------------------------------------------------------

/// The frame category.
///
/// Objects: reference frames (ECEF, NED, Body, IMU, Camera, ...).
/// Morphisms: coordinate transforms between them.
///
/// This category encodes the fundamental structure of multi-frame
/// sensor fusion: every measurement exists in some frame, and fusion
/// requires transforming measurements into a common frame.
///
/// Source: Groves (2013), Chapter 2 — "Coordinate Frames."
///         Sola et al. (2018) — "A micro Lie theory for state estimation."
pub struct FrameCategory;

impl Category for FrameCategory {
    type Object = ReferenceFrame;
    type Morphism = FrameTransform;

    fn identity(obj: &ReferenceFrame) -> FrameTransform {
        FrameTransform::new(*obj, *obj)
    }

    fn compose(f: &FrameTransform, g: &FrameTransform) -> Option<FrameTransform> {
        if f.to != g.from {
            return None;
        }
        Some(FrameTransform::new(f.from, g.to))
    }

    fn morphisms() -> Vec<FrameTransform> {
        let frames = ReferenceFrame::variants();
        frames
            .iter()
            .flat_map(|&from| frames.iter().map(move |&to| FrameTransform::new(from, to)))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Quality: frame conventions
// ---------------------------------------------------------------------------

/// Quality: handedness convention of each reference frame.
///
/// All standard frames are right-handed. This quality encodes that
/// invariant and allows the system to detect misconfigured frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handedness {
    RightHanded,
}

/// Quality: the handedness of each reference frame.
#[derive(Debug, Clone)]
pub struct FrameConvention;

impl Quality for FrameConvention {
    type Individual = ReferenceFrame;
    type Value = Handedness;

    fn get(&self, _frame: &ReferenceFrame) -> Option<Handedness> {
        // All standard frames in sensor fusion are right-handed.
        // NED: North(x), East(y), Down(z) — right-handed.
        // ENU: East(x), North(y), Up(z) — right-handed.
        // ECEF: right-handed by definition (IERS conventions).
        // Body: right-handed (ISO 1151 / SAE J670).
        Some(Handedness::RightHanded)
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Transforms compose associatively: (f . g) . h = f . (g . h).
///
/// This is a fundamental category law. For frame transforms it means
/// the order of composition doesn't matter as long as source/target match.
pub struct TransformsComposeAssociatively;

impl Axiom for TransformsComposeAssociatively {
    fn description(&self) -> &str {
        "frame transforms compose associatively: (f . g) . h = f . (g . h)"
    }

    fn holds(&self) -> bool {
        use ReferenceFrame::*;
        let triples = [
            (ECEF, NED, Body, IMU),
            (NED, Body, Camera, LiDAR),
            (ECI, ECEF, NED, Body),
        ];
        for (a, b, c, d) in &triples {
            let f = FrameTransform::new(*a, *b);
            let g = FrameTransform::new(*b, *c);
            let h = FrameTransform::new(*c, *d);

            // (f . g) . h
            let fg = FrameCategory::compose(&f, &g).unwrap();
            let fgh_left = FrameCategory::compose(&fg, &h).unwrap();

            // f . (g . h)
            let gh = FrameCategory::compose(&g, &h).unwrap();
            let fgh_right = FrameCategory::compose(&f, &gh).unwrap();

            if fgh_left != fgh_right {
                return false;
            }
        }
        true
    }
}

/// Identity transform exists for every frame: T(A, A) composed with any
/// T(A, B) yields T(A, B).
pub struct IdentityExists;

impl Axiom for IdentityExists {
    fn description(&self) -> &str {
        "identity transform exists for every frame and is neutral under composition"
    }

    fn holds(&self) -> bool {
        for frame in ReferenceFrame::variants() {
            let id = FrameCategory::identity(&frame);
            // id: A -> A
            if id.from != frame || id.to != frame {
                return false;
            }
            // id . f = f
            for other in ReferenceFrame::variants() {
                let f = FrameTransform::new(frame, other);
                if let Some(composed) = FrameCategory::compose(&id, &f)
                    && composed != f
                {
                    return false;
                }
                // f . id = f (when f: other -> frame)
                let g = FrameTransform::new(other, frame);
                if let Some(composed) = FrameCategory::compose(&g, &id)
                    && composed != g
                {
                    return false;
                }
            }
        }
        true
    }
}

/// Every transform is invertible: for T(A, B) there exists T(B, A).
///
/// This reflects the physical reality that coordinate transforms are
/// always reversible (they are elements of the group SE(3)).
pub struct TransformsInvertible;

impl Axiom for TransformsInvertible {
    fn description(&self) -> &str {
        "every frame transform is invertible: T(A,B) implies T(B,A) exists"
    }

    fn holds(&self) -> bool {
        let morphisms = FrameCategory::morphisms();
        for m in &morphisms {
            let inverse = FrameTransform::new(m.to, m.from);
            if !morphisms.contains(&inverse) {
                return false;
            }
        }
        true
    }
}

/// All frames are right-handed.
pub struct AllFramesRightHanded;

impl Axiom for AllFramesRightHanded {
    fn description(&self) -> &str {
        "all reference frames use right-handed coordinate conventions"
    }

    fn holds(&self) -> bool {
        let convention = FrameConvention;
        for frame in ReferenceFrame::variants() {
            if convention.get(&frame) != Some(Handedness::RightHanded) {
                return false;
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The reference frame ontology.
///
/// Founded on:
///   - Groves (2013), *Principles of GNSS, Inertial, and Multisensor
///     Integrated Navigation Systems*, Chapter 2.
///   - Sola et al. (2018), "A micro Lie theory for state estimation in robotics."
///   - IERS Conventions (2010), Chapter 4 — "Terrestrial reference frames."
pub struct FrameOntology;

impl Ontology for FrameOntology {
    type Cat = FrameCategory;
    type Qual = FrameConvention;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(TransformsComposeAssociatively),
            Box::new(IdentityExists),
            Box::new(TransformsInvertible),
            Box::new(AllFramesRightHanded),
        ]
    }
}

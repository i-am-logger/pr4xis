use crate::cube::Cube;
use crate::face::{Color, Face};
use praxis_category::{Category, Entity, Relationship};
use praxis_ontology::{Axiom, Quality};

// =============================================================================
// Entity: Face (6 faces of the cube)
// =============================================================================

impl Entity for Face {
    fn variants() -> Vec<Self> {
        Face::all().to_vec()
    }
}

// =============================================================================
// Relationship: FaceRotation (a move affects a face)
// =============================================================================

/// A rotation of a face — the morphism in the Rubik category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceRotation {
    pub face: Face,
    pub target: Face,
}

impl Relationship for FaceRotation {
    type Object = Face;
    fn source(&self) -> Face {
        self.face
    }
    fn target(&self) -> Face {
        self.target
    }
}

// =============================================================================
// Category: RubikCategory (faces + rotations)
// =============================================================================

/// The Rubik category: faces are objects, rotations are morphisms.
/// This is a thin category — one morphism per (source, target) pair.
pub struct RubikCategory;

impl Category for RubikCategory {
    type Object = Face;
    type Morphism = FaceRotation;

    fn identity(obj: &Face) -> FaceRotation {
        FaceRotation {
            face: *obj,
            target: *obj,
        }
    }

    fn compose(f: &FaceRotation, g: &FaceRotation) -> Option<FaceRotation> {
        if f.target != g.face {
            return None;
        }
        Some(FaceRotation {
            face: f.face,
            target: g.target,
        })
    }

    fn morphisms() -> Vec<FaceRotation> {
        let faces = Face::all();
        let mut m = Vec::new();
        for &a in &faces {
            for &b in &faces {
                m.push(FaceRotation { face: a, target: b });
            }
        }
        m
    }
}

// =============================================================================
// Quality: CenterColor
// =============================================================================

#[derive(Debug, Clone)]
pub struct CenterColor {
    pub cube: Cube,
}

impl Quality for CenterColor {
    type Individual = Face;
    type Value = Color;

    fn get(&self, face: &Face) -> Option<Color> {
        Some(self.cube.get(*face, 4))
    }
}

// =============================================================================
// Axioms: concrete for RubikCategory
// =============================================================================

/// Centers never move.
pub struct CentersFixed {
    pub cube: Cube,
}

impl Axiom<RubikCategory> for CentersFixed {
    fn description(&self) -> &str {
        "center stickers must match their face color"
    }

    fn holds(&self) -> bool {
        Face::all()
            .iter()
            .all(|&face| self.cube.get(face, 4) == Color::of_face(face))
    }
}

/// Each color has exactly 9 stickers.
pub struct NinePerColor {
    pub cube: Cube,
}

impl Axiom<RubikCategory> for NinePerColor {
    fn description(&self) -> &str {
        "each color must have exactly 9 stickers"
    }

    fn holds(&self) -> bool {
        self.cube.color_counts().iter().all(|&c| c == 9)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::Move;

    #[test]
    fn test_face_entity() {
        assert_eq!(Face::variants().len(), 6);
    }

    #[test]
    fn test_rubik_category_laws() {
        praxis_category::validate::check_category_laws::<RubikCategory>().unwrap();
    }

    #[test]
    fn test_center_color_quality() {
        let quality = CenterColor {
            cube: Cube::solved(),
        };
        assert_eq!(quality.get(&Face::U), Some(Color::of_face(Face::U)));
        assert_eq!(quality.individuals_with().len(), 6);
    }

    #[test]
    fn test_centers_fixed_axiom() {
        let axiom = CentersFixed {
            cube: Cube::solved(),
        };
        assert!(axiom.holds());
    }

    #[test]
    fn test_centers_fixed_after_moves() {
        let cube = Cube::solved().apply(Move::R).apply(Move::U).apply(Move::Ri);
        let axiom = CentersFixed { cube };
        assert!(axiom.holds());
    }

    #[test]
    fn test_nine_per_color_axiom() {
        let axiom = NinePerColor {
            cube: Cube::solved(),
        };
        assert!(axiom.holds());
    }

    #[test]
    fn test_nine_per_color_after_moves() {
        let cube = Cube::solved().apply(Move::R).apply(Move::U).apply(Move::F);
        let axiom = NinePerColor { cube };
        assert!(axiom.holds());
    }
}

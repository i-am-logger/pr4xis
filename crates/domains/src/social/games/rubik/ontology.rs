#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::cube::Cube;
use super::face::{Color, Face};
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// =============================================================================
// Category + Ontology: RubikCategory (faces + rotations)
// =============================================================================

define_ontology! {
    /// The Rubik category: faces are objects, rotations are morphisms.
    /// This is a thin category — one morphism per (source, target) pair.
    pub RubikOntology for RubikCategory {
        concepts: Face,
        relation: FaceRotation,
        being: SocialObject,
        source: "Joyner (2008); Singmaster (1981)",
    }
}

// =============================================================================
// Quality: FaceIndex (ordinal position of each face)
// =============================================================================

#[derive(Debug, Clone)]
pub struct FaceIndex;

impl Quality for FaceIndex {
    type Individual = Face;
    type Value = usize;

    fn get(&self, face: &Face) -> Option<usize> {
        Some(match face {
            Face::U => 0,
            Face::D => 1,
            Face::F => 2,
            Face::B => 3,
            Face::L => 4,
            Face::R => 5,
        })
    }
}

impl Ontology for RubikOntology {
    type Cat = RubikCategory;
    type Qual = FaceIndex;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(CentersFixed {
                cube: Cube::solved(),
            }),
            Box::new(NinePerColor {
                cube: Cube::solved(),
            }),
        ]
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

impl Axiom for CentersFixed {
    fn description(&self) -> &str {
        "center stickers must match their face color"
    }

    fn holds(&self) -> bool {
        Face::all()
            .iter()
            .all(|&face| self.cube.get(face, 4) == Color::of_face(face))
    }
}
pr4xis::register_axiom!(CentersFixed);

/// Each color has exactly 9 stickers.
pub struct NinePerColor {
    pub cube: Cube,
}

impl Axiom for NinePerColor {
    fn description(&self) -> &str {
        "each color must have exactly 9 stickers"
    }

    fn holds(&self) -> bool {
        self.cube.color_counts().iter().all(|&c| c == 9)
    }
}
pr4xis::register_axiom!(NinePerColor);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::games::rubik::moves::Move;
    use pr4xis::category::Concept;

    #[test]
    fn test_face_entity() {
        assert_eq!(Face::variants().len(), 6);
    }

    #[test]
    fn test_rubik_category_laws() {
        pr4xis::category::validate::check_category_laws::<RubikCategory>().unwrap();
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

    #[test]
    fn ontology_validates() {
        RubikOntology::validate().unwrap();
    }
}

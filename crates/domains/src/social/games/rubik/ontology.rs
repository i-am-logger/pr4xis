//! Rubik's cube — the six faces, the cube-group axis pairs, and the
//! solved-state invariants.
//!
//! This ontology models the 3×3×3 cube as an abstract category over its
//! six faces, with the three opposite-face axis pairs as the canonical
//! `opposes:` morphisms (Singmaster 1981 axis notation). The rich `Cube`,
//! `Face`, `Move`, and `Color` types in the sibling modules carry the
//! 54-sticker state and move algebra; this ontology is the upper-layer
//! categorical view used by Praxis-level reasoning.
//!
//! # Literature
//!
//! - **Rubik (1975)** Hungarian Patent 170062 — the original Magic Cube
//!   patent; defines the 6-face × 9-sticker construction with fixed
//!   centers.
//! - **Singmaster (1981)** *Notes on Rubik's Magic Cube* — the canonical
//!   face notation (U/D/F/B/L/R) and centre-fixed invariant proof.
//! - **Joyner (2008)** *Adventures in Group Theory* (2nd ed.) §11 —
//!   the cube group, generators, and the 12-cubie / 8-cubie orbits.

use super::cube::Cube;
use super::face::{Color, Face};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Rubik",
    source: "Rubik (1975) Hungarian Patent 170062 (Magic Cube); Singmaster (1981) Notes on Rubik's Magic Cube; Joyner (2008) Adventures in Group Theory",

    concepts: [
        // The six faces — Singmaster (1981) standard notation.
        U, D, F, B, L, R,
    ],

    labels: {
        U: ("en", "Up face",
            "Singmaster (1981): the upper face of the cube in standard orientation."),
        D: ("en", "Down face",
            "Singmaster (1981): the lower face — opposite of U."),
        F: ("en", "Front face",
            "Singmaster (1981): the face oriented toward the solver."),
        B: ("en", "Back face",
            "Singmaster (1981): the rear face — opposite of F."),
        L: ("en", "Left face",
            "Singmaster (1981): the left face from the solver's view."),
        R: ("en", "Right face",
            "Singmaster (1981): the right face — opposite of L."),
    },

    opposes: [
        // The three axis pairs of the cube. Rotating a face does not move
        // the stickers on its opposite face — Singmaster (1981) "centre
        // and opposite-face invariants" — so the three pairs partition
        // the cube into three independent rotational axes.
        (U, D), (D, U),
        (F, B), (B, F),
        (L, R), (R, L),
    ],
}

/// Quality: canonical ordinal index for each face per Singmaster (1981).
///
/// U=0, D=1, F=2, B=3, L=4, R=5 — the standard enumeration order used
/// in the cube-state arrays (`cube.rs::Cube`).
#[derive(Debug, Clone)]
pub struct FaceIndex;

impl Quality for FaceIndex {
    type Individual = RubikConcept;
    type Value = usize;

    fn get(&self, face: &RubikConcept) -> Option<usize> {
        Some(match face {
            RubikConcept::U => 0,
            RubikConcept::D => 1,
            RubikConcept::F => 2,
            RubikConcept::B => 3,
            RubikConcept::L => 4,
            RubikConcept::R => 5,
        })
    }
}

/// Map an ontology concept back to the rich `Face` enum used by the
/// runtime cube state. Bridge between the categorical view and the
/// move/sticker algebra.
pub fn concept_to_face(c: RubikConcept) -> Face {
    match c {
        RubikConcept::U => Face::U,
        RubikConcept::D => Face::D,
        RubikConcept::F => Face::F,
        RubikConcept::B => Face::B,
        RubikConcept::L => Face::L,
        RubikConcept::R => Face::R,
    }
}

impl Ontology for RubikOntology {
    type Cat = RubikCategory;
    type Qual = FaceIndex;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(CentersFixed {
            cube: Cube::solved(),
        }));
        axioms.push(Box::new(NinePerColor {
            cube: Cube::solved(),
        }));
        axioms
    }
}

/// Axiom: each face's centre sticker matches its assigned colour.
///
/// Singmaster (1981): the centre piece of each face is fixed relative
/// to the cube's internal core; in any reachable state the centres
/// retain their original face-colour mapping (U=White, D=Yellow,
/// F=Green, B=Blue, L=Orange, R=Red).
pub struct CentersFixed {
    pub cube: Cube,
}

impl Axiom for CentersFixed {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = Face::all()
            .iter()
            .all(|&face| self.cube.get(face, 4) == Color::of_face(face));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CentersFixed",
        "every face's centre sticker matches its assigned colour",
        "Singmaster (1981) Notes on Rubik's Magic Cube"
    );
}

pr4xis::register_axiom!(
    CentersFixed,
    "Singmaster (1981) Notes on Rubik's Magic Cube"
);

/// Axiom: each of the six colours appears on exactly nine stickers.
///
/// Rubik (1975): the cube has 6 faces × 9 stickers = 54 stickers, with
/// one colour per face in the solved state; permuting stickers can never
/// change the multiset of colours, so each colour count is invariant at 9.
pub struct NinePerColor {
    pub cube: Cube,
}

impl Axiom for NinePerColor {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = self.cube.color_counts().iter().all(|&c| c == 9);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NinePerColor",
        "each colour appears on exactly nine stickers",
        "Rubik (1975) Hungarian Patent 170062 (Magic Cube)"
    );
}

pr4xis::register_axiom!(
    NinePerColor,
    "Rubik (1975) Hungarian Patent 170062 (Magic Cube)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::games::rubik::moves::Move;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<RubikCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        RubikOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn six_faces() {
        assert_eq!(RubikConcept::variants().len(), 6);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn three_axis_pairs_via_opposition() {
        // Singmaster (1981): the three axis pairs U↔D, F↔B, L↔R.
        let opposed: Vec<_> = RubikCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == RubikRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opposed.contains(&(RubikConcept::U, RubikConcept::D)));
        assert!(opposed.contains(&(RubikConcept::F, RubikConcept::B)));
        assert!(opposed.contains(&(RubikConcept::L, RubikConcept::R)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn face_index_canonical_order() {
        let q = FaceIndex;
        assert_eq!(q.get(&RubikConcept::U), Some(0));
        assert_eq!(q.get(&RubikConcept::D), Some(1));
        assert_eq!(q.get(&RubikConcept::F), Some(2));
        assert_eq!(q.get(&RubikConcept::B), Some(3));
        assert_eq!(q.get(&RubikConcept::L), Some(4));
        assert_eq!(q.get(&RubikConcept::R), Some(5));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn centers_fixed_on_solved_cube() {
        let axiom = CentersFixed {
            cube: Cube::solved(),
        };
        match axiom.verify() {
            Ok(_) => {}
            Err(c) => panic!(
                "CentersFixed failed on solved cube: {}",
                c.meta().name.as_str()
            ),
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn centers_fixed_after_moves() {
        let cube = Cube::solved().apply(Move::R).apply(Move::U).apply(Move::Ri);
        let axiom = CentersFixed { cube };
        match axiom.verify() {
            Ok(_) => {}
            Err(c) => panic!(
                "CentersFixed failed after R U R': {}",
                c.meta().name.as_str()
            ),
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nine_per_color_on_solved() {
        let axiom = NinePerColor {
            cube: Cube::solved(),
        };
        assert!(axiom.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nine_per_color_after_moves() {
        let cube = Cube::solved().apply(Move::R).apply(Move::U).apply(Move::F);
        let axiom = NinePerColor { cube };
        assert!(axiom.verify().is_ok());
    }

    #[pr4xis::praxis_value(Extensible, Verifiable)]
    #[test]
    fn concept_to_face_bijection_on_concepts() {
        for c in RubikConcept::variants() {
            let f = concept_to_face(c);
            // FaceIndex on the concept agrees with the position in Face::all().
            let idx = FaceIndex.get(&c).unwrap();
            assert_eq!(Face::all()[idx], f);
        }
    }

    fn arb_concept() -> impl Strategy<Value = RubikConcept> {
        proptest::sample::select(RubikConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_face_index_total(c in arb_concept()) {
            prop_assert!(FaceIndex.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in RubikCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in RubikOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        // NOTE: subsumption-targets-valid test removed — Rubik defines no
        // Subsumption variant (only Opposition), so the vacuous-quantifier
        // form of this test referenced a variant that doesn't exist.

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            // Each axis pair has both directions as explicit edges.
            let opposed: std::collections::HashSet<_> = RubikCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == RubikRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} ↔ {:?}", a, b);
            }
        }
    }

    pr4xis::register_praxis_value!(prop_face_index_total, Verifiable);
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
}

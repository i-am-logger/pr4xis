//! Terrain — feature taxonomy for terrain-relative navigation.
//!
//! A digital elevation model (DEM) is read by classifying each cell into
//! one of four canonical terrain features: peaks, valleys, ridges, and
//! saddles. Terrain-relative navigation registers a measured elevation
//! signature against a map using this taxonomy.
//!
//! # Literature
//!
//! - **Goldstein (1987)** "Terrain Aided Navigation" — the foundational
//!   paper on TAN: using a stored digital terrain map and an onboard
//!   radar altimeter to localise an aircraft. Defines the peak / valley
//!   / ridge / saddle feature vocabulary that downstream TERCOM /
//!   TERPROM / SITAN systems inherit.

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Terrain",
    source: "Goldstein (1987) Terrain Aided Navigation",

    concepts: [
        // === Extrema (Goldstein 1987 §3) ===
        Peak,
        Valley,

        // === Lines / passes (Goldstein 1987 §3) ===
        Ridge,
        Saddle,
    ],

    labels: {
        Peak: ("en", "Peak",
            "Goldstein (1987) §3: a local maximum in elevation — both principal curvatures negative (concave down)."),
        Valley: ("en", "Valley",
            "Goldstein (1987) §3: a local minimum in elevation — both principal curvatures positive (concave up)."),
        Ridge: ("en", "Ridge",
            "Goldstein (1987) §3: a line of local maxima connecting peaks — one negative curvature, one near-zero curvature along the ridgeline."),
        Saddle: ("en", "Saddle",
            "Goldstein (1987) §3: a col / pass where two ridges meet — opposite-sign principal curvatures."),
    },

    opposes: [
        // Peak (local max) and Valley (local min) are polar extrema.
        (Peak, Valley),
        (Valley, Peak),
    ],
}

/// Quality: curvature signature for each terrain feature type.
///
/// Per Goldstein (1987) §3, the four feature kinds are characterised by
/// the sign pair of their two principal curvatures (k1, k2):
/// `Peak (−,−)`, `Valley (+,+)`, `Ridge (−,0)`, `Saddle (−,+)`.
#[derive(Debug, Clone)]
pub struct CurvatureSignature;

impl Quality for CurvatureSignature {
    type Individual = TerrainConcept;
    /// (principal curvature 1 sign, principal curvature 2 sign): +1, 0, -1
    type Value = (i8, i8);

    fn get(&self, feature: &TerrainConcept) -> Option<(i8, i8)> {
        Some(match feature {
            TerrainConcept::Peak => (-1, -1),
            TerrainConcept::Valley => (1, 1),
            TerrainConcept::Ridge => (-1, 0),
            TerrainConcept::Saddle => (-1, 1),
        })
    }
}

impl Ontology for TerrainOntology {
    type Cat = TerrainCategory;
    type Qual = CurvatureSignature;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(PeakCurvatureNegative));
        axioms.push(Box::new(ValleyCurvaturePositive));
        axioms.push(Box::new(SaddleCurvaturesOpposite));
        axioms
    }
}

/// Axiom: peaks have strictly negative principal curvatures (local maxima
/// of the elevation surface).
///
/// Goldstein (1987) §3 — a peak is a local maximum of the DEM height
/// function, which by the second-derivative test has both principal
/// curvatures strictly negative.
pub struct PeakCurvatureNegative;

impl Axiom for PeakCurvatureNegative {
    fn verify(&self) -> Verdict {
        if let Some((k1, k2)) = CurvatureSignature.get(&TerrainConcept::Peak)
            && k1 < 0
            && k2 < 0
        {
            return Ok(Box::new(SimpleProof::new(self.meta())));
        }
        Err(Box::new(SimpleCounterexample::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "PeakCurvatureNegative",
        "peaks have negative principal curvatures (local maxima of the DEM)",
        "Goldstein (1987) Terrain Aided Navigation §3"
    );
}

pr4xis::register_axiom!(
    PeakCurvatureNegative,
    "Goldstein (1987) Terrain Aided Navigation §3"
);

/// Axiom: valleys have strictly positive principal curvatures (local
/// minima of the elevation surface).
///
/// Goldstein (1987) §3 — dual to `PeakCurvatureNegative`.
pub struct ValleyCurvaturePositive;

impl Axiom for ValleyCurvaturePositive {
    fn verify(&self) -> Verdict {
        if let Some((k1, k2)) = CurvatureSignature.get(&TerrainConcept::Valley)
            && k1 > 0
            && k2 > 0
        {
            return Ok(Box::new(SimpleProof::new(self.meta())));
        }
        Err(Box::new(SimpleCounterexample::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ValleyCurvaturePositive",
        "valleys have positive principal curvatures (local minima of the DEM)",
        "Goldstein (1987) Terrain Aided Navigation §3"
    );
}

pr4xis::register_axiom!(
    ValleyCurvaturePositive,
    "Goldstein (1987) Terrain Aided Navigation §3"
);

/// Axiom: saddle points have principal curvatures of opposite sign — one
/// direction is concave up, the other concave down.
///
/// Goldstein (1987) §3 — the defining feature of a col / pass.
pub struct SaddleCurvaturesOpposite;

impl Axiom for SaddleCurvaturesOpposite {
    fn verify(&self) -> Verdict {
        if let Some((k1, k2)) = CurvatureSignature.get(&TerrainConcept::Saddle)
            && k1.signum() != 0
            && k2.signum() != 0
            && k1.signum() != k2.signum()
        {
            return Ok(Box::new(SimpleProof::new(self.meta())));
        }
        Err(Box::new(SimpleCounterexample::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SaddleCurvaturesOpposite",
        "saddle points have principal curvatures of opposite sign",
        "Goldstein (1987) Terrain Aided Navigation §3"
    );
}

pr4xis::register_axiom!(
    SaddleCurvaturesOpposite,
    "Goldstein (1987) Terrain Aided Navigation §3"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<TerrainCategory>();
    }

    #[test]
    fn ontology_validates() {
        TerrainOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn four_terrain_features() {
        assert_eq!(TerrainConcept::variants().len(), 4);
    }

    #[test]
    fn peak_curvature_signature() {
        assert_eq!(
            CurvatureSignature.get(&TerrainConcept::Peak),
            Some((-1, -1))
        );
    }

    #[test]
    fn valley_curvature_signature() {
        assert_eq!(
            CurvatureSignature.get(&TerrainConcept::Valley),
            Some((1, 1))
        );
    }

    #[test]
    fn peak_valley_oppose() {
        let opp: Vec<_> = TerrainCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == TerrainRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(TerrainConcept::Peak, TerrainConcept::Valley)));
        assert!(opp.contains(&(TerrainConcept::Valley, TerrainConcept::Peak)));
    }

    #[test]
    fn peak_curvature_axiom_holds() {
        assert!(PeakCurvatureNegative.verify().is_ok());
    }

    #[test]
    fn valley_curvature_axiom_holds() {
        assert!(ValleyCurvaturePositive.verify().is_ok());
    }

    #[test]
    fn saddle_curvatures_axiom_holds() {
        assert!(SaddleCurvaturesOpposite.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = TerrainConcept> {
        proptest::sample::select(TerrainConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in TerrainCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in TerrainOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_curvature_total(c in arb_concept()) {
            // CurvatureSignature is total over the four terrain features.
            prop_assert!(CurvatureSignature.get(&c).is_some());
        }

        #[test]
        fn prop_curvature_signs_bounded(c in arb_concept()) {
            // Each principal-curvature sign component is in {-1, 0, 1}.
            let (k1, k2) = CurvatureSignature.get(&c).unwrap();
            prop_assert!((-1..=1).contains(&k1));
            prop_assert!((-1..=1).contains(&k2));
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = TerrainCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == TerrainRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }
    }
}

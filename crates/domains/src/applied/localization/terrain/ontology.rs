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

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

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

/// The sign of a principal curvature of the elevation surface — the typed
/// value the old positional `i8` encoded.
///
/// Second-derivative test (Goldstein 1987 §3): a negative principal curvature
/// is a locally **convex** (dome-like) surface, positive is **concave**
/// (bowl-like), zero is locally **planar** (e.g. along a ridgeline).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CurvatureSign {
    /// Negative principal curvature — surface locally convex (peak / ridge).
    Convex,
    /// Zero principal curvature — surface locally planar (along a ridge).
    Planar,
    /// Positive principal curvature — surface locally concave (valley / pit).
    Concave,
}

impl CurvatureSign {
    /// The `{-1, 0, +1}` sign under Goldstein's (1987) second-derivative
    /// convention: convex = −1, planar = 0, concave = +1.
    ///
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`) — a sign
    /// indicator is a pure number, not a bare `i8`.
    pub fn sign(&self) -> Quantity {
        let s = match self {
            CurvatureSign::Convex => -1.0,
            CurvatureSign::Planar => 0.0,
            CurvatureSign::Concave => 1.0,
        };
        Quantity::from_unit(s, &unit::UNITLESS)
    }
}

/// Quality: the principal-curvature signature of each terrain feature — a pair
/// of typed [`CurvatureSign`]s (k1, k2), not raw `i8`s.
///
/// Per Goldstein (1987) §3: `Peak (convex, convex)`, `Valley (concave, concave)`,
/// `Ridge (convex, planar)`, `Saddle (convex, concave)`.
#[derive(Debug, Clone)]
pub struct CurvatureSignature;

impl Quality for CurvatureSignature {
    type Individual = TerrainConcept;
    type Value = (CurvatureSign, CurvatureSign);

    fn get(&self, feature: &TerrainConcept) -> Option<(CurvatureSign, CurvatureSign)> {
        use CurvatureSign::{Concave, Convex, Planar};
        Some(match feature {
            TerrainConcept::Peak => (Convex, Convex),
            TerrainConcept::Valley => (Concave, Concave),
            TerrainConcept::Ridge => (Convex, Planar),
            TerrainConcept::Saddle => (Convex, Concave),
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
            && k1.sign().value < 0.0
            && k2.sign().value < 0.0
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
            && k1.sign().value > 0.0
            && k2.sign().value > 0.0
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
            && k1.sign().value != 0.0
            && k2.sign().value != 0.0
            && k1.sign().value != k2.sign().value
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

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<TerrainCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        TerrainOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn four_terrain_features() {
        assert_eq!(TerrainConcept::variants().len(), 4);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn peak_curvature_signature() {
        assert_eq!(
            CurvatureSignature.get(&TerrainConcept::Peak),
            Some((CurvatureSign::Convex, CurvatureSign::Convex))
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn valley_curvature_signature() {
        assert_eq!(
            CurvatureSignature.get(&TerrainConcept::Valley),
            Some((CurvatureSign::Concave, CurvatureSign::Concave))
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn peak_curvature_axiom_holds() {
        assert!(PeakCurvatureNegative.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn valley_curvature_axiom_holds() {
        assert!(ValleyCurvaturePositive.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
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
            // Each typed CurvatureSign maps into the {-1, 0, 1} sign convention.
            let (k1, k2) = CurvatureSignature.get(&c).unwrap();
            prop_assert!((-1.0..=1.0).contains(&k1.sign().value));
            prop_assert!((-1.0..=1.0).contains(&k2.sign().value));
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

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_curvature_total, Verifiable);
    pr4xis::register_praxis_value!(prop_curvature_signs_bounded, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
}

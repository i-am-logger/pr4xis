use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::applied::localization::terrain::engine::*;
use crate::applied::localization::terrain::ontology::*;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// Grid spacing shared by the fixed 3x3 test tiles below (1 meter).
fn one_meter() -> Quantity {
    Quantity::from_unit(1.0, &unit::METER)
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn terrain_category_laws() {
    assert_category_laws::<TerrainCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn terrain_ontology_validates() {
    TerrainOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn peak_curvature_negative_holds() {
    assert!(PeakCurvatureNegative.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn valley_curvature_positive_holds() {
    assert!(ValleyCurvaturePositive.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn saddle_curvatures_opposite_holds() {
    assert!(SaddleCurvaturesOpposite.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dem_peak_detection() {
    // 3x3 grid with center higher than all neighbors
    #[rustfmt::skip]
    let elevations = vec![
        1.0, 1.0, 1.0,
        1.0, 5.0, 1.0,
        1.0, 1.0, 1.0,
    ];
    let dem = DemTile::new(elevations, 3, 3, one_meter());
    let feature = dem.classify_feature(1, 1);
    assert_eq!(feature, Some(TerrainConcept::Peak));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dem_valley_detection() {
    #[rustfmt::skip]
    let elevations = vec![
        5.0, 5.0, 5.0,
        5.0, 1.0, 5.0,
        5.0, 5.0, 5.0,
    ];
    let dem = DemTile::new(elevations, 3, 3, one_meter());
    let feature = dem.classify_feature(1, 1);
    assert_eq!(feature, Some(TerrainConcept::Valley));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn dem_border_returns_none() {
    let elevations = vec![1.0; 9];
    let dem = DemTile::new(elevations, 3, 3, one_meter());
    assert_eq!(dem.classify_feature(0, 0), None);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dem_ridge_detection() {
    // A ridgeline running through the center column: elevation is high
    // along the column (planar in that direction) and falls off to either
    // side (convex in the perpendicular direction) — Goldstein (1987) §3's
    // "one negative curvature, one near-zero curvature" Ridge signature.
    #[rustfmt::skip]
    let elevations = vec![
        1.0, 5.0, 1.0,
        1.0, 5.0, 1.0,
        1.0, 5.0, 1.0,
    ];
    let dem = DemTile::new(elevations, 3, 3, one_meter());
    let feature = dem.classify_feature(1, 1);
    assert_eq!(feature, Some(TerrainConcept::Ridge));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dem_saddle_detection() {
    // A col/pass: z = x^2 - y^2 sampled on the 3x3 grid — opposite-sign
    // principal curvatures, Goldstein (1987) §3's Saddle signature.
    #[rustfmt::skip]
    let elevations = vec![
         0.0, -1.0,  0.0,
         1.0,  0.0,  1.0,
         0.0, -1.0,  0.0,
    ];
    let dem = DemTile::new(elevations, 3, 3, one_meter());
    let feature = dem.classify_feature(1, 1);
    assert_eq!(feature, Some(TerrainConcept::Saddle));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn classify_feature_agrees_with_curvature_signature() {
    // The engine's classification and the ontology's own CurvatureSignature
    // must never disagree — for every known geometry, the returned concept's
    // own declared signature is exactly what was matched against.
    let cases: [(Vec<f64>, TerrainConcept); 4] = [
        (
            vec![1.0, 1.0, 1.0, 1.0, 5.0, 1.0, 1.0, 1.0, 1.0],
            TerrainConcept::Peak,
        ),
        (
            vec![5.0, 5.0, 5.0, 5.0, 1.0, 5.0, 5.0, 5.0, 5.0],
            TerrainConcept::Valley,
        ),
        (
            vec![1.0, 5.0, 1.0, 1.0, 5.0, 1.0, 1.0, 5.0, 1.0],
            TerrainConcept::Ridge,
        ),
        (
            vec![0.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0],
            TerrainConcept::Saddle,
        ),
    ];
    for (elevations, expected) in cases {
        let dem = DemTile::new(elevations, 3, 3, one_meter());
        let feature = dem.classify_feature(1, 1);
        assert_eq!(feature, Some(expected));
        let (k1, k2) = CurvatureSignature.get(&expected).expect("total quality");
        // Sanity: the ontology's own signature is internally consistent
        // (never (Planar, Planar), the flat degenerate case none of the
        // four features declare).
        assert!(k1 != CurvatureSign::Planar || k2 != CurvatureSign::Planar);
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn terrain_match_perfect_profile() {
    #[rustfmt::skip]
    let elevations = vec![
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 9.0,
    ];
    let dem = DemTile::new(elevations, 3, 3, one_meter());
    let profile = vec![4.0, 5.0, 6.0];
    let score = dem.match_profile(0, 1, &profile).value;
    assert!(score < 1e-12, "perfect match should have zero error");
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn peak_always_detected_when_center_is_max(
            center in 10.0..100.0_f64,
            neighbor in 0.0..9.9_f64
        ) {
            #[rustfmt::skip]
            let elevations = vec![
                neighbor, neighbor, neighbor,
                neighbor, center,   neighbor,
                neighbor, neighbor, neighbor,
            ];
            let dem = DemTile::new(elevations, 3, 3, one_meter());
            let feature = dem.classify_feature(1, 1);
            prop_assert_eq!(feature, Some(TerrainConcept::Peak));
        }

        #[test]
        fn match_score_non_negative(
            elev in proptest::collection::vec(0.0..100.0_f64, 9..=9),
            profile in proptest::collection::vec(0.0..100.0_f64, 3..=3)
        ) {
            let dem = DemTile::new(elev, 3, 3, one_meter());
            let score = dem.match_profile(0, 1, &profile).value;
            prop_assert!(score >= 0.0, "match score must be non-negative");
        }

        /// Soundness: whenever `classify_feature` returns a concept, the
        /// concept's own ontology-declared `CurvatureSignature` is exactly
        /// what the classification matched against (by construction) — so a
        /// classified isotropic dome/bowl at any scale is never mislabeled.
        #[test]
        fn classification_is_sound(
            center in 10.0..100.0_f64,
            neighbor in 0.0..9.9_f64,
            resolution in 0.1..10.0_f64
        ) {
            #[rustfmt::skip]
            let elevations = vec![
                neighbor, neighbor, neighbor,
                neighbor, center,   neighbor,
                neighbor, neighbor, neighbor,
            ];
            let dem = DemTile::new(elevations, 3, 3, Quantity::from_unit(resolution, &unit::METER));
            let feature = dem.classify_feature(1, 1);
            if let Some(concept) = feature {
                let (k1, k2) = CurvatureSignature.get(&concept).expect("total quality");
                prop_assert!(k1 == CurvatureSign::Convex && k2 == CurvatureSign::Convex);
                prop_assert_eq!(concept, TerrainConcept::Peak);
            }
        }

        /// A tighter planarity tolerance never turns a strictly-signed
        /// eigenvalue into `Planar` if the standard tolerance already saw it
        /// as nonzero — tightening the criteria only sharpens borderline
        /// (near-flat) classification, it cannot flip a clear dome.
        #[test]
        fn tighter_tolerance_still_detects_clear_peak(
            center in 50.0..100.0_f64,
            neighbor in 0.0..9.9_f64
        ) {
            #[rustfmt::skip]
            let elevations = vec![
                neighbor, neighbor, neighbor,
                neighbor, center,   neighbor,
                neighbor, neighbor, neighbor,
            ];
            let dem = DemTile::new(elevations, 3, 3, one_meter());
            let tight = TerrainClassificationCriteria {
                planarity_tolerance: Quantity::from_unit(1e-6, &unit::RECIPROCAL_METER),
            };
            let feature = dem.classify_feature_with(1, 1, &tight);
            prop_assert_eq!(feature, Some(TerrainConcept::Peak));
        }
    }

    pr4xis::register_praxis_value!(peak_always_detected_when_center_is_max, Verifiable);
    pr4xis::register_praxis_value!(match_score_non_negative, Verifiable);
    pr4xis::register_praxis_value!(classification_is_sound, Verifiable);
    pr4xis::register_praxis_value!(tighter_tolerance_still_detects_clear_peak, Verifiable);
}

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::localization::terrain::engine::*;
use crate::applied::localization::terrain::ontology::*;

#[test]
fn terrain_category_laws() {
    check_category_laws::<TerrainCategory>().unwrap();
}

#[test]
fn terrain_ontology_validates() {
    TerrainOntology::validate().unwrap();
}

#[test]
fn elevation_bounded_holds() {
    assert!(ElevationBounded.holds());
}

#[test]
fn peak_curvature_negative_holds() {
    assert!(PeakCurvatureNegative.holds());
}

#[test]
fn dem_peak_detection() {
    // 3x3 grid with center higher than all neighbors
    #[rustfmt::skip]
    let elevations = vec![
        1.0, 1.0, 1.0,
        1.0, 5.0, 1.0,
        1.0, 1.0, 1.0,
    ];
    let dem = DemTile::new(elevations, 3, 3, 1.0);
    let feature = dem.classify_feature(1, 1);
    assert_eq!(feature, Some(TerrainFeature::Peak));
}

#[test]
fn dem_valley_detection() {
    #[rustfmt::skip]
    let elevations = vec![
        5.0, 5.0, 5.0,
        5.0, 1.0, 5.0,
        5.0, 5.0, 5.0,
    ];
    let dem = DemTile::new(elevations, 3, 3, 1.0);
    let feature = dem.classify_feature(1, 1);
    assert_eq!(feature, Some(TerrainFeature::Valley));
}

#[test]
fn dem_border_returns_none() {
    let elevations = vec![1.0; 9];
    let dem = DemTile::new(elevations, 3, 3, 1.0);
    assert_eq!(dem.classify_feature(0, 0), None);
}

#[test]
fn terrain_match_perfect_profile() {
    #[rustfmt::skip]
    let elevations = vec![
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 9.0,
    ];
    let dem = DemTile::new(elevations, 3, 3, 1.0);
    let profile = vec![4.0, 5.0, 6.0];
    let score = dem.match_profile(0, 1, &profile);
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
            let dem = DemTile::new(elevations, 3, 3, 1.0);
            let feature = dem.classify_feature(1, 1);
            prop_assert_eq!(feature, Some(TerrainFeature::Peak));
        }

        #[test]
        fn match_score_non_negative(
            elev in proptest::collection::vec(0.0..100.0_f64, 9..=9),
            profile in proptest::collection::vec(0.0..100.0_f64, 3..=3)
        ) {
            let dem = DemTile::new(elev, 3, 3, 1.0);
            let score = dem.match_profile(0, 1, &profile);
            prop_assert!(score >= 0.0, "match score must be non-negative");
        }
    }
}

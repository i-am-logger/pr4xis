use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::navigation::celestial::engine::*;
use crate::applied::navigation::celestial::ontology::*;

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

#[test]
fn celestial_body_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<CelestialBodyTaxonomy>>().unwrap();
}

#[test]
fn celestial_observable_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<CelestialObservableTaxonomy>>().unwrap();
}

#[test]
fn celestial_sensor_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<CelestialSensorTaxonomy>>().unwrap();
}

#[test]
fn celestial_ontology_validates() {
    CelestialOntology::validate().unwrap();
}

#[test]
fn celestial_body_taxonomy_is_dag() {
    assert!(CelestialBodyTaxonomyIsDAG.holds());
}

#[test]
fn celestial_observable_taxonomy_is_dag() {
    assert!(CelestialObservableTaxonomyIsDAG.holds());
}

#[test]
fn two_sights_fix_axiom() {
    assert!(TwoSightsFix.holds());
}

#[test]
fn star_tracker_most_accurate_axiom() {
    assert!(StarTrackerMostAccurate.holds());
}

#[test]
fn atmospheric_refraction_axiom() {
    assert!(AtmosphericRefraction.holds());
}

// ---------------------------------------------------------------------------
// Engine tests
// ---------------------------------------------------------------------------

#[test]
fn add_observation_increases_count() {
    let sit = CelestialSituation {
        observations: vec![],
        fix: None,
        assumed_position: (34.0, -118.0),
        step: 0,
    };
    let obs = CelestialObservation {
        body_name: "Polaris".to_string(),
        altitude_deg: 34.0,
        azimuth_deg: 0.0,
        declination_deg: 89.26,
        gha_deg: 315.0,
    };
    let next = apply_celestial(&sit, &CelestialAction::Observe(obs)).unwrap();
    assert_eq!(next.observations.len(), 1);
    assert_eq!(next.step, 1);
}

#[test]
fn invalid_altitude_rejected() {
    let sit = CelestialSituation {
        observations: vec![],
        fix: None,
        assumed_position: (34.0, -118.0),
        step: 0,
    };
    let obs = CelestialObservation {
        body_name: "Invalid".to_string(),
        altitude_deg: 91.0, // impossible: max is 90
        azimuth_deg: 0.0,
        declination_deg: 0.0,
        gha_deg: 0.0,
    };
    assert!(apply_celestial(&sit, &CelestialAction::Observe(obs)).is_err());
}

#[test]
fn fix_needs_2_observations() {
    let sit = CelestialSituation {
        observations: vec![CelestialObservation {
            body_name: "Polaris".to_string(),
            altitude_deg: 34.0,
            azimuth_deg: 0.0,
            declination_deg: 89.26,
            gha_deg: 315.0,
        }],
        fix: None,
        assumed_position: (34.0, -118.0),
        step: 0,
    };
    assert!(apply_celestial(&sit, &CelestialAction::ComputeFix).is_err());
}

#[test]
fn fix_with_two_observations() {
    // Assumed position: 45 N, 0 E
    let lat_ap = 45.0_f64;
    let lon_ap = 0.0_f64;
    let lat_rad = lat_ap.to_radians();
    let _lon_rad = lon_ap.to_radians();

    // Star A: dec=30, GHA=350 -> LHA=350+0=350 deg
    let dec_a = 30.0_f64;
    let gha_a = 350.0_f64;
    let lha_a = (gha_a + lon_ap).to_radians();
    let sin_hc_a = lat_rad.sin() * dec_a.to_radians().sin()
        + lat_rad.cos() * dec_a.to_radians().cos() * lha_a.cos();
    let hc_a = sin_hc_a.asin().to_degrees();

    // Star B: dec=60, GHA=90 -> LHA=90+0=90 deg
    let dec_b = 60.0_f64;
    let gha_b = 90.0_f64;
    let lha_b = (gha_b + lon_ap).to_radians();
    let sin_hc_b = lat_rad.sin() * dec_b.to_radians().sin()
        + lat_rad.cos() * dec_b.to_radians().cos() * lha_b.cos();
    let hc_b = sin_hc_b.asin().to_degrees();

    // Set observed altitude = calculated altitude -> zero intercept -> fix = AP
    let sit = CelestialSituation {
        observations: vec![
            CelestialObservation {
                body_name: "Star A".to_string(),
                altitude_deg: hc_a,
                azimuth_deg: 0.0, // north
                declination_deg: dec_a,
                gha_deg: gha_a,
            },
            CelestialObservation {
                body_name: "Star B".to_string(),
                altitude_deg: hc_b,
                azimuth_deg: 90.0, // east
                declination_deg: dec_b,
                gha_deg: gha_b,
            },
        ],
        fix: None,
        assumed_position: (lat_ap, lon_ap),
        step: 0,
    };
    let result = apply_celestial(&sit, &CelestialAction::ComputeFix).unwrap();
    assert!(result.fix.is_some());
    let fix = result.fix.unwrap();
    assert_eq!(fix.num_observations, 2);
    // With zero-intercept observations, fix should be near assumed position
    assert!(
        (fix.latitude - lat_ap).abs() < 1.0,
        "lat={} should be near {}",
        fix.latitude,
        lat_ap
    );
}

// ---------------------------------------------------------------------------
// H2: asin domain clamping — extreme values should not produce NaN
// ---------------------------------------------------------------------------

#[test]
fn celestial_fix_does_not_nan_on_extreme_declination() {
    // Extreme declinations can push sin_hc outside [-1,1] due to floating point.
    // This should not panic or produce NaN after clamping.
    let sit = CelestialSituation {
        observations: vec![
            CelestialObservation {
                body_name: "Star A".to_string(),
                altitude_deg: 89.0,
                azimuth_deg: 0.0,
                declination_deg: 89.9,
                gha_deg: 0.1,
            },
            CelestialObservation {
                body_name: "Star B".to_string(),
                altitude_deg: 45.0,
                azimuth_deg: 90.0,
                declination_deg: 45.0,
                gha_deg: 90.0,
            },
        ],
        fix: None,
        assumed_position: (89.0, 0.0),
        step: 0,
    };
    // Should not panic — may return Err for pole guard but must not NaN
    let result = apply_celestial(&sit, &CelestialAction::ComputeFix);
    if let Ok(s) = &result {
        if let Some(fix) = &s.fix {
            assert!(!fix.latitude.is_nan(), "latitude must not be NaN");
            assert!(!fix.longitude.is_nan(), "longitude must not be NaN");
        }
    }
}

// ---------------------------------------------------------------------------
// H3: Pole guard — fix at exactly 90 degrees latitude returns Err
// ---------------------------------------------------------------------------

#[test]
fn celestial_fix_at_north_pole_returns_err() {
    let sit = CelestialSituation {
        observations: vec![
            CelestialObservation {
                body_name: "Star A".to_string(),
                altitude_deg: 30.0,
                azimuth_deg: 0.0,
                declination_deg: 30.0,
                gha_deg: 0.0,
            },
            CelestialObservation {
                body_name: "Star B".to_string(),
                altitude_deg: 45.0,
                azimuth_deg: 90.0,
                declination_deg: 45.0,
                gha_deg: 90.0,
            },
        ],
        fix: None,
        assumed_position: (90.0, 0.0), // north pole
        step: 0,
    };
    let result = apply_celestial(&sit, &CelestialAction::ComputeFix);
    assert!(result.is_err(), "celestial fix at pole should return Err");
    assert!(
        result.unwrap_err().contains("pole"),
        "error should mention pole"
    );
}

// ---------------------------------------------------------------------------
// Proptest
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn observation_count_monotonically_increases(
            n in 1..8_usize,
        ) {
            let mut sit = CelestialSituation {
                observations: vec![],
                fix: None,
                assumed_position: (34.0, -118.0),
                step: 0,
            };
            for i in 0..n {
                let obs = CelestialObservation {
                    body_name: format!("Star {}", i),
                    altitude_deg: 30.0 + i as f64 * 5.0,
                    azimuth_deg: i as f64 * 45.0,
                    declination_deg: 20.0 + i as f64 * 10.0,
                    gha_deg: i as f64 * 30.0,
                };
                let next = apply_celestial(&sit, &CelestialAction::Observe(obs)).unwrap();
                prop_assert!(next.observations.len() > sit.observations.len());
                sit = next;
            }
        }

        #[test]
        fn valid_altitude_always_accepted(
            alt in -90.0..90.0_f64,
            az in 0.0..360.0_f64,
        ) {
            let sit = CelestialSituation {
                observations: vec![],
                fix: None,
                assumed_position: (34.0, -118.0),
                step: 0,
            };
            let obs = CelestialObservation {
                body_name: "Test".to_string(),
                altitude_deg: alt,
                azimuth_deg: az,
                declination_deg: 45.0,
                gha_deg: 0.0,
            };
            let result = apply_celestial(&sit, &CelestialAction::Observe(obs));
            prop_assert!(result.is_ok());
        }

        #[test]
        fn add_observation_is_deterministic(
            alt in -90.0..90.0_f64,
            az in 0.0..360.0_f64,
            dec in -90.0..90.0_f64,
            gha in 0.0..360.0_f64,
        ) {
            let sit = CelestialSituation {
                observations: vec![],
                fix: None,
                assumed_position: (34.0, -118.0),
                step: 0,
            };
            let obs = CelestialObservation {
                body_name: "Test".to_string(),
                altitude_deg: alt,
                azimuth_deg: az,
                declination_deg: dec,
                gha_deg: gha,
            };
            let r1 = apply_celestial(&sit, &CelestialAction::Observe(obs.clone())).unwrap();
            let r2 = apply_celestial(&sit, &CelestialAction::Observe(obs)).unwrap();
            prop_assert_eq!(r1.observations.len(), r2.observations.len());
            prop_assert_eq!(r1.step, r2.step);
        }
    }
}

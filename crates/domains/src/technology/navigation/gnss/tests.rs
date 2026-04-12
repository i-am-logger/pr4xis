use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
use pr4xis::ontology::{Axiom, Ontology};

use crate::technology::navigation::gnss::engine::*;
use crate::technology::navigation::gnss::ontology::*;

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

#[test]
fn gnss_observable_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<GnssObservableTaxonomy>>().unwrap();
}

#[test]
fn gnss_constellation_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<GnssConstellationTaxonomy>>().unwrap();
}

#[test]
fn gnss_ontology_validates() {
    GnssOntology::validate().unwrap();
}

#[test]
fn gnss_observable_taxonomy_is_dag() {
    assert!(GnssObservableTaxonomyIsDAG.holds());
}

#[test]
fn gnss_constellation_taxonomy_is_dag() {
    assert!(GnssConstellationTaxonomyIsDAG.holds());
}

#[test]
fn minimum_satellites_axiom() {
    assert!(MinimumSatellites.holds());
}

#[test]
fn dop_geometry_axiom() {
    assert!(DopGeometry.holds());
}

#[test]
fn pseudorange_positive_axiom() {
    assert!(PseudorangePositive.holds());
}

// ---------------------------------------------------------------------------
// Engine tests
// ---------------------------------------------------------------------------

#[test]
fn add_measurement_increases_count() {
    let sit = GnssSituation {
        measurements: vec![],
        solution: None,
        step: 0,
    };
    let m = GnssMeasurement {
        satellite_id: 1,
        pseudorange: 20_000_000.0,
        satellite_position: [20_000_000.0, 0.0, 0.0],
        cn0: 45.0,
    };
    let next = apply_gnss(&sit, &GnssAction::AddMeasurement(m)).unwrap();
    assert_eq!(next.measurements.len(), 1);
    assert_eq!(next.step, 1);
}

#[test]
fn negative_pseudorange_rejected() {
    let sit = GnssSituation {
        measurements: vec![],
        solution: None,
        step: 0,
    };
    let m = GnssMeasurement {
        satellite_id: 1,
        pseudorange: -100.0,
        satellite_position: [20_000_000.0, 0.0, 0.0],
        cn0: 45.0,
    };
    assert!(apply_gnss(&sit, &GnssAction::AddMeasurement(m)).is_err());
}

#[test]
fn compute_fix_needs_4_satellites() {
    let sit = GnssSituation {
        measurements: vec![
            GnssMeasurement {
                satellite_id: 1,
                pseudorange: 20_000_000.0,
                satellite_position: [20_000_000.0, 0.0, 0.0],
                cn0: 45.0,
            },
            GnssMeasurement {
                satellite_id: 2,
                pseudorange: 20_000_000.0,
                satellite_position: [0.0, 20_000_000.0, 0.0],
                cn0: 45.0,
            },
        ],
        solution: None,
        step: 0,
    };
    assert!(apply_gnss(&sit, &GnssAction::ComputeFix).is_err());
}

#[test]
fn compute_fix_with_4_satellites() {
    // Place receiver at origin, 4 satellites at known positions
    let r = 26_000_000.0; // ~GPS orbit radius in meters
    let measurements = vec![
        GnssMeasurement {
            satellite_id: 1,
            pseudorange: r,
            satellite_position: [r, 0.0, 0.0],
            cn0: 45.0,
        },
        GnssMeasurement {
            satellite_id: 2,
            pseudorange: r,
            satellite_position: [0.0, r, 0.0],
            cn0: 45.0,
        },
        GnssMeasurement {
            satellite_id: 3,
            pseudorange: r,
            satellite_position: [0.0, 0.0, r],
            cn0: 45.0,
        },
        GnssMeasurement {
            satellite_id: 4,
            pseudorange: r,
            satellite_position: [-r, 0.0, 0.0],
            cn0: 45.0,
        },
    ];
    let sit = GnssSituation {
        measurements,
        solution: None,
        step: 0,
    };
    let result = apply_gnss(&sit, &GnssAction::ComputeFix).unwrap();
    assert!(result.solution.is_some());
    let sol = result.solution.unwrap();
    assert_eq!(sol.num_satellites, 4);
    // Solution should be near origin
    let dist = (sol.position[0].powi(2) + sol.position[1].powi(2) + sol.position[2].powi(2)).sqrt();
    assert!(
        dist < 1000.0,
        "solution should be near origin, got distance={}",
        dist
    );
}

// ---------------------------------------------------------------------------
// H6: GDOP with fewer than 4 satellites returns MAX instead of panicking
// ---------------------------------------------------------------------------

#[test]
fn gdop_fewer_than_4_satellites_no_panic() {
    use crate::technology::navigation::gnss::ontology::compute_gdop_from_elevations_azimuths;
    // This would previously panic with assert!; now it should return f64::MAX
    let result = compute_gdop_from_elevations_azimuths(&[45.0, 45.0, 45.0], &[0.0, 90.0, 180.0]);
    assert_eq!(result, f64::MAX, "fewer than 4 sats should return MAX");
}

// ---------------------------------------------------------------------------
// Proptest
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    fn arb_measurement() -> impl Strategy<Value = GnssMeasurement> {
        (
            1..32_u32,
            20_000_000.0..30_000_000.0_f64,
            -30_000_000.0..30_000_000.0_f64,
            -30_000_000.0..30_000_000.0_f64,
            -30_000_000.0..30_000_000.0_f64,
            25.0..55.0_f64,
        )
            .prop_map(|(id, pr, sx, sy, sz, cn0)| GnssMeasurement {
                satellite_id: id,
                pseudorange: pr,
                satellite_position: [sx, sy, sz],
                cn0,
            })
    }

    proptest! {
        #[test]
        fn add_measurement_is_deterministic(m in arb_measurement()) {
            let sit = GnssSituation {
                measurements: vec![],
                solution: None,
                step: 0,
            };
            let r1 = apply_gnss(&sit, &GnssAction::AddMeasurement(m.clone())).unwrap();
            let r2 = apply_gnss(&sit, &GnssAction::AddMeasurement(m)).unwrap();
            prop_assert_eq!(r1.measurements.len(), r2.measurements.len());
            prop_assert_eq!(r1.step, r2.step);
        }

        #[test]
        fn positive_pseudorange_always_accepted(
            pr in 0.0..100_000_000.0_f64,
            sx in -30_000_000.0..30_000_000.0_f64,
            sy in -30_000_000.0..30_000_000.0_f64,
            sz in -30_000_000.0..30_000_000.0_f64,
        ) {
            let sit = GnssSituation {
                measurements: vec![],
                solution: None,
                step: 0,
            };
            let m = GnssMeasurement {
                satellite_id: 1,
                pseudorange: pr,
                satellite_position: [sx, sy, sz],
                cn0: 45.0,
            };
            let result = apply_gnss(&sit, &GnssAction::AddMeasurement(m));
            prop_assert!(result.is_ok());
        }

        #[test]
        fn measurement_count_monotonically_increases(
            n in 1..10_usize,
        ) {
            let mut sit = GnssSituation {
                measurements: vec![],
                solution: None,
                step: 0,
            };
            for i in 0..n {
                let m = GnssMeasurement {
                    satellite_id: (i + 1) as u32,
                    pseudorange: 20_000_000.0,
                    satellite_position: [20_000_000.0, 0.0, 0.0],
                    cn0: 45.0,
                };
                let next = apply_gnss(&sit, &GnssAction::AddMeasurement(m)).unwrap();
                prop_assert!(next.measurements.len() > sit.measurements.len());
                sit = next;
            }
        }
    }
}

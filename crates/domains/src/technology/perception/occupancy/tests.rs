use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::technology::perception::occupancy::engine::OccupancyGrid;
use crate::technology::perception::occupancy::ontology::*;

#[test]
fn occupancy_category_laws() {
    check_category_laws::<OccupancyCategory>().unwrap();
}

#[test]
fn occupancy_ontology_validates() {
    OccupancyOntology::validate().unwrap();
}

#[test]
fn probability_bounded_holds() {
    assert!(ProbabilityBounded.holds());
}

#[test]
fn log_odds_deterministic_holds() {
    assert!(LogOddsUpdateDeterministic.holds());
}

#[test]
fn new_grid_is_unknown() {
    let grid = OccupancyGrid::new(10, 10);
    let p = grid.probability(5, 5);
    assert!((p - 0.5).abs() < 1e-12, "new cell should have p=0.5");
}

#[test]
fn occupied_observation_increases_probability() {
    let mut grid = OccupancyGrid::new(10, 10);
    let p_before = grid.probability(3, 3);
    grid.update(3, 3, 1.0); // positive log-odds = occupied evidence
    let p_after = grid.probability(3, 3);
    assert!(p_after > p_before);
}

#[test]
fn free_observation_decreases_probability() {
    let mut grid = OccupancyGrid::new(10, 10);
    let p_before = grid.probability(3, 3);
    grid.update(3, 3, -1.0); // negative log-odds = free evidence
    let p_after = grid.probability(3, 3);
    assert!(p_after < p_before);
}

#[test]
fn log_odds_clamping_prevents_overconfidence() {
    let mut grid = OccupancyGrid::new(5, 5);
    for _ in 0..100 {
        grid.update(0, 0, 2.0);
    }
    let p = grid.probability(0, 0);
    assert!(p < 1.0, "probability must stay below 1.0");
    assert!(p > 0.99, "should be very confident but bounded");
}

#[test]
fn log_odds_roundtrip() {
    let p = 0.7;
    let l = OccupancyGrid::probability_to_log_odds(p);
    let p2 = OccupancyGrid::log_odds_to_probability(l);
    assert!((p - p2).abs() < 1e-12);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn log_odds_roundtrip_property(p in 0.01..0.99_f64) {
            let l = OccupancyGrid::probability_to_log_odds(p);
            let p2 = OccupancyGrid::log_odds_to_probability(l);
            prop_assert!((p - p2).abs() < 1e-10,
                "roundtrip failed: p={}, l={}, p2={}", p, l, p2);
        }

        #[test]
        fn probability_always_bounded(log_odds in -10.0..10.0_f64) {
            let p = OccupancyGrid::log_odds_to_probability(log_odds);
            prop_assert!(p > 0.0 && p < 1.0,
                "probability {} out of (0,1) for log_odds={}", p, log_odds);
        }
    }
}

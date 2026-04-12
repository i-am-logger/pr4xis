use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::navigation::ins_gnss::coupling::*;
use crate::applied::navigation::ins_gnss::engine::*;
use crate::applied::navigation::ins_gnss::ontology::*;

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

#[test]
fn coupling_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<CouplingTaxonomy>>().unwrap();
}

#[test]
fn ins_gnss_state_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<InsGnssStateTaxonomy>>().unwrap();
}

#[test]
fn ins_gnss_ontology_validates() {
    InsGnssOntology::validate().unwrap();
}

#[test]
fn ins_gnss_state_taxonomy_is_dag() {
    assert!(InsGnssStateTaxonomyIsDAG.holds());
}

#[test]
fn coasting_degrades_axiom() {
    assert!(CoastingDegrades.holds());
}

#[test]
fn gnss_update_reduces_error_axiom() {
    assert!(GnssUpdateReducesError.holds());
}

#[test]
fn tighter_coupling_better_axiom() {
    assert!(TighterCouplingBetter.holds());
}

// ---------------------------------------------------------------------------
// Coupling tests
// ---------------------------------------------------------------------------

#[test]
fn loosely_coupled_needs_4_satellites() {
    let mode = CouplingMode::for_level(CouplingLevel::LooselyCoupled);
    assert!(!mode.can_operate(3));
    assert!(mode.can_operate(4));
    assert!(mode.can_operate(8));
}

#[test]
fn tightly_coupled_needs_1_satellite() {
    let mode = CouplingMode::for_level(CouplingLevel::TightlyCoupled);
    assert!(!mode.can_operate(0));
    assert!(mode.can_operate(1));
    assert!(mode.can_operate(4));
}

#[test]
fn deeply_coupled_needs_0_satellites() {
    let mode = CouplingMode::for_level(CouplingLevel::DeeplyCoupled);
    assert!(mode.can_operate(0));
}

#[test]
fn coasting_error_grows_quadratically() {
    let bias = 0.01; // 1 mg in m/s^2
    let e1 = coasting_position_error(bias, 10.0);
    let e2 = coasting_position_error(bias, 20.0);
    // At 2x time, error should be 4x
    let ratio = e2 / e1;
    assert!((ratio - 4.0).abs() < 0.01, "ratio = {}", ratio);
}

#[test]
fn kalman_update_reduces_variance() {
    let prior = 100.0; // 10m 1-sigma
    let noise = 25.0; // 5m 1-sigma
    let post = scalar_kalman_update(prior, noise);
    assert!(post < prior, "post={} should be < prior={}", post, prior);
}

// ---------------------------------------------------------------------------
// Engine tests
// ---------------------------------------------------------------------------

#[test]
fn ins_propagation_increases_error() {
    let sit = InsGnssSituation {
        state: InsGnssState::Coasting,
        coupling: CouplingLevel::LooselyCoupled,
        position_error: 5.0,
        velocity_error: 0.1,
        time_since_gnss: 10.0,
        accel_bias: 0.01,
        step: 0,
    };
    let next = apply_ins_gnss(&sit, &InsGnssAction::InsPropagation { dt: 1.0 }).unwrap();
    assert!(next.position_error > sit.position_error);
    assert!(next.time_since_gnss > sit.time_since_gnss);
}

#[test]
fn gnss_update_reduces_position_error() {
    let sit = InsGnssSituation {
        state: InsGnssState::Coasting,
        coupling: CouplingLevel::LooselyCoupled,
        position_error: 50.0,
        velocity_error: 1.0,
        time_since_gnss: 30.0,
        accel_bias: 0.01,
        step: 0,
    };
    let next = apply_ins_gnss(
        &sit,
        &InsGnssAction::GnssUpdate {
            measurement_noise: 5.0,
            num_satellites: 8,
        },
    )
    .unwrap();
    assert!(next.position_error < sit.position_error);
    assert_eq!(next.time_since_gnss, 0.0);
    assert_eq!(next.state, InsGnssState::NavigationMode);
}

#[test]
fn loosely_coupled_rejects_insufficient_satellites() {
    let sit = InsGnssSituation {
        state: InsGnssState::NavigationMode,
        coupling: CouplingLevel::LooselyCoupled,
        position_error: 5.0,
        velocity_error: 0.1,
        time_since_gnss: 0.0,
        accel_bias: 0.01,
        step: 0,
    };
    let result = apply_ins_gnss(
        &sit,
        &InsGnssAction::GnssUpdate {
            measurement_noise: 5.0,
            num_satellites: 3,
        },
    );
    assert!(result.is_err());
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
        fn coasting_error_is_monotonic_in_time(
            bias in 0.001..0.1_f64,
            t1 in 1.0..100.0_f64,
            dt in 0.1..50.0_f64,
        ) {
            let t2 = t1 + dt;
            let e1 = coasting_position_error(bias, t1);
            let e2 = coasting_position_error(bias, t2);
            prop_assert!(e2 >= e1, "error should grow with time: e1={}, e2={}", e1, e2);
        }

        #[test]
        fn kalman_update_always_reduces_or_maintains_variance(
            prior in 1.0..10000.0_f64,
            noise in 0.1..10000.0_f64,
        ) {
            let post = scalar_kalman_update(prior, noise);
            prop_assert!(post <= prior + 1e-10,
                "post={} should be <= prior={}", post, prior);
            prop_assert!(post >= 0.0, "post={} should be non-negative", post);
        }

        #[test]
        fn ins_propagation_never_decreases_error(
            initial_error in 0.1..100.0_f64,
            bias in 0.001..0.1_f64,
            dt in 0.01..10.0_f64,
        ) {
            let sit = InsGnssSituation {
                state: InsGnssState::Coasting,
                coupling: CouplingLevel::LooselyCoupled,
                position_error: initial_error,
                velocity_error: 0.1,
                time_since_gnss: 10.0,
                accel_bias: bias,
                step: 0,
            };
            let next = apply_ins_gnss(&sit, &InsGnssAction::InsPropagation { dt }).unwrap();
            prop_assert!(next.position_error >= sit.position_error,
                "error should not decrease during propagation: {} vs {}",
                next.position_error, sit.position_error);
        }

        #[test]
        fn gnss_update_always_improves_position(
            initial_error in 10.0..1000.0_f64,
            noise in 1.0..50.0_f64,
        ) {
            let sit = InsGnssSituation {
                state: InsGnssState::Coasting,
                coupling: CouplingLevel::LooselyCoupled,
                position_error: initial_error,
                velocity_error: 1.0,
                time_since_gnss: 30.0,
                accel_bias: 0.01,
                step: 0,
            };
            let next = apply_ins_gnss(
                &sit,
                &InsGnssAction::GnssUpdate {
                    measurement_noise: noise,
                    num_satellites: 8,
                },
            ).unwrap();
            prop_assert!(next.position_error < sit.position_error,
                "GNSS should reduce error: {} vs {}", next.position_error, sit.position_error);
        }
    }
}

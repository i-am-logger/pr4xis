use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::technology::space::orbit::engine::*;
use crate::technology::space::orbit::ontology::*;
use crate::technology::space::orbit::propagator::*;

#[test]
fn orbit_category_laws() {
    check_category_laws::<OrbitCategory>().unwrap();
}

#[test]
fn orbit_ontology_validates() {
    OrbitOntology::validate().unwrap();
}

#[test]
fn eccentricity_bounded_holds() {
    assert!(EccentricityBounded.holds());
}

#[test]
fn semi_major_axis_positive_holds() {
    assert!(SemiMajorAxisPositive.holds());
}

#[test]
fn leo_orbit_is_bound() {
    // ISS-like orbit: ~408 km altitude, ~7.66 km/s
    let state = OrbitalState {
        position: [6786.0, 0.0, 0.0],
        velocity: [0.0, 7.66, 0.0],
    };
    assert!(is_bound_orbit(&state), "LEO orbit should be bound");
}

#[test]
fn energy_conservation_during_propagation() {
    // Circular orbit at ~7000 km radius
    let v_circ = (mu_earth_km3s2() / 7000.0).sqrt();
    let initial = OrbitalState {
        position: [7000.0, 0.0, 0.0],
        velocity: [0.0, v_circ, 0.0],
    };
    let e_initial = initial.specific_energy(mu_earth_km3s2());

    // Propagate for 100 steps of 10 seconds each
    let trajectory = propagate_orbit(&initial, 10.0, 100);
    let final_state = trajectory.last().unwrap();
    let e_final = final_state.specific_energy(mu_earth_km3s2());

    let relative_error = ((e_final - e_initial) / e_initial).abs();
    assert!(
        relative_error < 1e-6,
        "energy should be conserved: initial={}, final={}, error={}",
        e_initial,
        e_final,
        relative_error
    );
}

#[test]
fn propagation_preserves_radius_for_circular_orbit() {
    let r = 7000.0;
    let v = (mu_earth_km3s2() / r).sqrt();
    let initial = OrbitalState {
        position: [r, 0.0, 0.0],
        velocity: [0.0, v, 0.0],
    };
    let propagated = propagate_rk4(&initial, 60.0, mu_earth_km3s2());
    let r_after = propagated.radius();
    assert!(
        (r_after - r).abs() / r < 1e-4,
        "circular orbit radius should be ~constant: {} vs {}",
        r_after,
        r
    );
}

#[test]
fn radar_to_eci_at_zenith() {
    let obs = RadarObservation {
        range: 1000.0,
        range_rate: 0.0,
        azimuth: 0.0,
        elevation: std::f64::consts::FRAC_PI_2,
    };
    let pos = radar_to_eci(&obs);
    assert!(pos[0].abs() < 1e-10);
    assert!(pos[1].abs() < 1e-10);
    assert!((pos[2] - 1000.0).abs() < 1e-10);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn bound_orbit_has_negative_energy(
            r in 6500.0..50000.0_f64
        ) {
            let v = (mu_earth_km3s2() / r).sqrt(); // circular velocity
            let state = OrbitalState {
                position: [r, 0.0, 0.0],
                velocity: [0.0, v, 0.0],
            };
            prop_assert!(state.specific_energy(mu_earth_km3s2()) < 0.0,
                "circular orbit at r={} should have negative energy", r);
        }

        #[test]
        fn radar_range_preserved(
            range in 100.0..100000.0_f64,
            az in -3.14..3.14_f64,
            el in -1.5..1.5_f64
        ) {
            let obs = RadarObservation {
                range, range_rate: 0.0, azimuth: az, elevation: el,
            };
            let pos = radar_to_eci(&obs);
            let computed_range = (pos[0].powi(2) + pos[1].powi(2) + pos[2].powi(2)).sqrt();
            prop_assert!((computed_range - range).abs() / range < 1e-10,
                "range should be preserved: {} vs {}", computed_range, range);
        }
    }
}

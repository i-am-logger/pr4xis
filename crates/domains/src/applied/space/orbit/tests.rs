#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::laws::assert_category_laws;
use pr4xis::logic::Axiom;
use pr4xis::ontology::Ontology;

use crate::applied::space::orbit::engine::*;
use crate::applied::space::orbit::ontology::*;
use crate::applied::space::orbit::propagator::*;
use crate::formal::math::angle::Angle;
use crate::formal::math::geometry::point::Point3;
use crate::formal::math::quantity::dimension::Dimension;
use crate::formal::math::quantity::value::Quantity;
use crate::natural::physics::kinematics::velocity::Velocity;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn orbit_category_laws() {
    assert_category_laws::<OrbitCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn orbit_ontology_validates() {
    OrbitOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn eccentricity_bounded_holds() {
    assert!(EccentricityBounded.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn semi_major_axis_positive_holds() {
    assert!(SemiMajorAxisPositive.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn leo_orbit_is_bound() {
    // ISS-like orbit: ~408 km altitude, ~7.66 km/s
    let state = OrbitalState {
        position: Point3::new(6786.0, 0.0, 0.0),
        velocity: Velocity::new(0.0, 7.66, 0.0),
    };
    assert!(is_bound_orbit(&state), "LEO orbit should be bound");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn energy_conservation_during_propagation() {
    // Circular orbit at ~7000 km radius
    let v_circ = (mu_earth_km3s2().value / 7000.0).sqrt();
    let initial = OrbitalState {
        position: Point3::new(7000.0, 0.0, 0.0),
        velocity: Velocity::new(0.0, v_circ, 0.0),
    };
    let e_initial = initial.specific_energy(&mu_earth_km3s2()).value;

    // Propagate for 100 steps of 10 seconds each
    let trajectory = propagate_orbit(&initial, 10.0, 100);
    let final_state = trajectory.last().unwrap();
    let e_final = final_state.specific_energy(&mu_earth_km3s2()).value;

    let relative_error = ((e_final - e_initial) / e_initial).abs();
    assert!(
        relative_error < 1e-6,
        "energy should be conserved: initial={}, final={}, error={}",
        e_initial,
        e_final,
        relative_error
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn propagation_preserves_radius_for_circular_orbit() {
    let r = 7000.0;
    let v = (mu_earth_km3s2().value / r).sqrt();
    let initial = OrbitalState {
        position: Point3::new(r, 0.0, 0.0),
        velocity: Velocity::new(0.0, v, 0.0),
    };
    let propagated = propagate_rk4(&initial, 60.0, mu_earth_km3s2().value);
    let r_after = propagated.radius().value;
    assert!(
        (r_after - r).abs() / r < 1e-4,
        "circular orbit radius should be ~constant: {} vs {}",
        r_after,
        r
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn radar_to_eci_at_zenith() {
    let obs = RadarObservation {
        range: Quantity::new(1000.0, Dimension::LENGTH),
        range_rate: Quantity::new(0.0, Dimension::VELOCITY),
        azimuth: Angle::from_radians(0.0),
        elevation: Angle::from_radians(core::f64::consts::FRAC_PI_2),
    };
    let pos = radar_to_eci(&obs);
    assert!(pos.x.abs() < 1e-10);
    assert!(pos.y.abs() < 1e-10);
    assert!((pos.z - 1000.0).abs() < 1e-10);
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
            let v = (mu_earth_km3s2().value / r).sqrt(); // circular velocity
            let state = OrbitalState {
                position: Point3::new(r, 0.0, 0.0),
                velocity: Velocity::new(0.0, v, 0.0),
            };
            prop_assert!(state.specific_energy(&mu_earth_km3s2()).value < 0.0,
                "circular orbit at r={} should have negative energy", r);
        }

        #[test]
        fn radar_range_preserved(
            range in 100.0..100000.0_f64,
            az in -core::f64::consts::PI..core::f64::consts::PI,
            el in -1.5..1.5_f64
        ) {
            let obs = RadarObservation {
                range: Quantity::new(range, Dimension::LENGTH),
                range_rate: Quantity::new(0.0, Dimension::VELOCITY),
                azimuth: Angle::from_radians(az),
                elevation: Angle::from_radians(el),
            };
            let pos = radar_to_eci(&obs);
            let computed_range = (pos.x.powi(2) + pos.y.powi(2) + pos.z.powi(2)).sqrt();
            prop_assert!((computed_range - range).abs() / range < 1e-10,
                "range should be preserved: {} vs {}", computed_range, range);
        }
    }

    pr4xis::register_praxis_value!(bound_orbit_has_negative_energy, Verifiable);
    pr4xis::register_praxis_value!(radar_range_preserved, Verifiable);
}

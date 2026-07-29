use pr4xis::category::laws::assert_category_laws;
use pr4xis::logic::Axiom;
use pr4xis::ontology::Ontology;

use crate::applied::underwater::acoustic::engine::*;
use crate::applied::underwater::acoustic::ontology::*;
use crate::formal::math::angle::Angle;
use crate::formal::math::geometry::point::Point3;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::duration::Duration;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn acoustic_category_laws() {
    assert_category_laws::<AcousticCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn acoustic_ontology_validates() {
    AcousticOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn sound_speed_positive_holds() {
    assert!(SoundSpeedPositive.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn range_non_negative_holds() {
    assert!(RangeNonNegative.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn mackenzie_typical_surface_sound_speed() {
    // Typical ocean surface: T=15C, S=35 PSU, D=0m -> ~1507 m/s
    let c = mackenzie_sound_speed(
        Quantity::from_unit(15.0, &unit::CELSIUS),
        Quantity::from_unit(35.0, &unit::PSU),
        Quantity::from_unit(0.0, &unit::METER),
    )
    .value;
    assert!(
        c > 1400.0 && c < 1600.0,
        "surface sound speed should be ~1507 m/s, got {}",
        c
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn sound_speed_increases_with_depth() {
    let c_shallow = mackenzie_sound_speed(
        Quantity::from_unit(15.0, &unit::CELSIUS),
        Quantity::from_unit(35.0, &unit::PSU),
        Quantity::from_unit(0.0, &unit::METER),
    )
    .value;
    let c_deep = mackenzie_sound_speed(
        Quantity::from_unit(15.0, &unit::CELSIUS),
        Quantity::from_unit(35.0, &unit::PSU),
        Quantity::from_unit(1000.0, &unit::METER),
    )
    .value;
    assert!(
        c_deep > c_shallow,
        "sound speed should increase with depth (pressure effect)"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn range_from_travel_time_basic() {
    let range = range_from_travel_time(
        Duration::from_seconds(0.1),
        Quantity::from_unit(1500.0, &unit::METER_PER_SECOND),
    )
    .value;
    assert!(
        (range - 75.0).abs() < 1e-10,
        "0.1s two-way at 1500m/s = 75m"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn usbl_fix_to_cartesian_straight_down() {
    let fix = UsblFix {
        range: Quantity::from_unit(100.0, &unit::METER),
        bearing: Angle::from_radians(0.0),
        depression: Angle::from_radians(core::f64::consts::FRAC_PI_2),
    };
    let pos = fix.to_cartesian();
    assert!(pos.x.abs() < 1e-10);
    assert!(pos.y.abs() < 1e-10);
    assert!((pos.z - (-100.0)).abs() < 1e-10);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn lbl_trilateration_requires_three_transponders() {
    let transponders = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(100.0, 0.0, 0.0)];
    let ranges = vec![
        Quantity::from_unit(50.0, &unit::METER),
        Quantity::from_unit(50.0, &unit::METER),
    ];
    assert!(lbl_trilateration(&transponders, &ranges).is_none());
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn sound_speed_always_positive(
            temp in -2.0..35.0_f64,
            salinity in 0.0..40.0_f64,
            depth in 0.0..11000.0_f64
        ) {
            let c = mackenzie_sound_speed(
                Quantity::from_unit(temp, &unit::CELSIUS),
                Quantity::from_unit(salinity, &unit::PSU),
                Quantity::from_unit(depth, &unit::METER),
            ).value;
            prop_assert!(c > 0.0, "sound speed must be positive, got {} for T={}, S={}, D={}",
                c, temp, salinity, depth);
        }

        #[test]
        fn range_non_negative_property(
            travel_time in 0.0..10.0_f64,
            sound_speed in 1400.0..1600.0_f64
        ) {
            let range = range_from_travel_time(
                Duration::from_seconds(travel_time),
                Quantity::from_unit(sound_speed, &unit::METER_PER_SECOND),
            ).value;
            prop_assert!(range >= 0.0, "range must be non-negative");
        }
    }

    pr4xis::register_praxis_value!(sound_speed_always_positive, Verifiable);
    pr4xis::register_praxis_value!(range_non_negative_property, Verifiable);
}

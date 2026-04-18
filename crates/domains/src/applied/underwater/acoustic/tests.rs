#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::underwater::acoustic::engine::*;
use crate::applied::underwater::acoustic::ontology::*;

#[test]
fn acoustic_category_laws() {
    check_category_laws::<AcousticCategory>().unwrap();
}

#[test]
fn acoustic_ontology_validates() {
    AcousticOntology::validate().unwrap();
}

#[test]
fn sound_speed_positive_holds() {
    assert!(SoundSpeedPositive.holds());
}

#[test]
fn range_non_negative_holds() {
    assert!(RangeNonNegative.holds());
}

#[test]
fn mackenzie_typical_surface_sound_speed() {
    // Typical ocean surface: T=15C, S=35 PSU, D=0m -> ~1507 m/s
    let c = mackenzie_sound_speed(15.0, 35.0, 0.0);
    assert!(
        c > 1400.0 && c < 1600.0,
        "surface sound speed should be ~1507 m/s, got {}",
        c
    );
}

#[test]
fn sound_speed_increases_with_depth() {
    let c_shallow = mackenzie_sound_speed(15.0, 35.0, 0.0);
    let c_deep = mackenzie_sound_speed(15.0, 35.0, 1000.0);
    assert!(
        c_deep > c_shallow,
        "sound speed should increase with depth (pressure effect)"
    );
}

#[test]
fn range_from_travel_time_basic() {
    let range = range_from_travel_time(0.1, 1500.0);
    assert!(
        (range - 75.0).abs() < 1e-10,
        "0.1s two-way at 1500m/s = 75m"
    );
}

#[test]
fn usbl_fix_to_cartesian_straight_down() {
    let fix = UsblFix {
        range: 100.0,
        bearing: 0.0,
        depression: core::f64::consts::FRAC_PI_2,
    };
    let pos = fix.to_cartesian();
    assert!(pos[0].abs() < 1e-10);
    assert!(pos[1].abs() < 1e-10);
    assert!((pos[2] - (-100.0)).abs() < 1e-10);
}

#[test]
fn lbl_trilateration_requires_three_transponders() {
    let transponders = vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0]];
    let ranges = vec![50.0, 50.0];
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
            let c = mackenzie_sound_speed(temp, salinity, depth);
            prop_assert!(c > 0.0, "sound speed must be positive, got {} for T={}, S={}, D={}",
                c, temp, salinity, depth);
        }

        #[test]
        fn range_non_negative_property(
            travel_time in 0.0..10.0_f64,
            sound_speed in 1400.0..1600.0_f64
        ) {
            let range = range_from_travel_time(travel_time, sound_speed);
            prop_assert!(range >= 0.0, "range must be non-negative");
        }
    }
}

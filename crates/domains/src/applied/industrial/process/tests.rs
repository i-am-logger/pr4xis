use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::industrial::process::engine::*;
use crate::applied::industrial::process::ontology::*;

#[test]
fn process_category_laws() {
    check_category_laws::<ProcessCategory>().unwrap();
}

#[test]
fn process_ontology_validates() {
    ProcessOntology::validate().unwrap();
}

#[test]
fn temperature_above_absolute_zero_holds() {
    assert!(TemperatureAboveAbsoluteZero.holds());
}

#[test]
fn pressure_non_negative_holds() {
    assert!(PressureNonNegative.holds());
}

#[test]
fn celsius_kelvin_roundtrip() {
    let c = 25.0;
    let k = celsius_to_kelvin(c);
    let c2 = kelvin_to_celsius(k);
    assert!((c - c2).abs() < 1e-12);
}

#[test]
fn absolute_zero_is_zero_kelvin() {
    let k = celsius_to_kelvin(-273.15);
    assert!(k.abs() < 1e-10);
}

#[test]
fn validate_temperature_valid() {
    assert!(validate_temperature_k(300.0));
    assert!(validate_temperature_k(0.0));
}

#[test]
fn validate_temperature_invalid() {
    assert!(!validate_temperature_k(-1.0));
}

#[test]
fn validate_pressure_valid() {
    assert!(validate_pressure(101325.0));
    assert!(validate_pressure(0.0));
}

#[test]
fn validate_pressure_invalid() {
    assert!(!validate_pressure(-1.0));
}

#[test]
fn pid_controller_drives_to_setpoint() {
    let mut pid = PidController::new(1.0, 0.1, 0.01, 0.0, 100.0);
    let setpoint = 50.0;
    let mut value = 0.0;
    let dt = 0.1;

    for _ in 0..1000 {
        let output = pid.update(setpoint, value, dt);
        // Simple first-order plant: value changes proportionally to output
        value += (output - value) * dt;
    }

    assert!(
        (value - setpoint).abs() < 3.0,
        "PID should drive to setpoint: value={}, setpoint={}",
        value,
        setpoint
    );
}

#[test]
fn pid_output_clamped() {
    let mut pid = PidController::new(100.0, 0.0, 0.0, 0.0, 10.0);
    let output = pid.update(100.0, 0.0, 0.1);
    assert!(
        output <= 10.0,
        "output should be clamped to max: {}",
        output
    );
}

#[test]
fn pid_reset_clears_state() {
    let mut pid = PidController::new(1.0, 1.0, 1.0, -100.0, 100.0);
    pid.update(50.0, 0.0, 1.0);
    pid.reset();
    assert!((pid.integral()).abs() < 1e-12);
    assert!((pid.prev_error()).abs() < 1e-12);
}

/// The wrapped PID delegates to control_theory and produces the same results.
#[test]
fn pid_delegates_to_control_theory() {
    use crate::formal::math::control_theory::pid as ct_pid;

    let kp = 2.0;
    let ki = 0.5;
    let kd = 0.1;
    let dt = 0.1;

    // Create both: the wrapper and a raw control_theory PID
    let mut wrapper = PidController::new(kp, ki, kd, -100.0, 100.0);
    let gains = ct_pid::PidGains::new(kp, ki, kd);
    let mut raw = ct_pid::PidController::new(gains, dt).with_limits(-100.0, 100.0);

    // Feed the same error sequence and compare outputs
    let setpoint = 10.0;
    let measured = 3.0;
    let error = setpoint - measured;

    let out_wrapper = wrapper.update(setpoint, measured, dt);
    let out_raw = raw.update(error);

    assert!(
        (out_wrapper - out_raw).abs() < 1e-12,
        "wrapper={}, raw={}",
        out_wrapper,
        out_raw
    );
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn celsius_kelvin_roundtrip_property(celsius in -273.15..1000.0_f64) {
            let kelvin = celsius_to_kelvin(celsius);
            let back = kelvin_to_celsius(kelvin);
            prop_assert!((celsius - back).abs() < 1e-10,
                "roundtrip failed: {} -> {} -> {}", celsius, kelvin, back);
        }

        #[test]
        fn pid_output_always_clamped(
            setpoint in 0.0..100.0_f64,
            measured in 0.0..100.0_f64,
            kp in 0.1..10.0_f64
        ) {
            let mut pid = PidController::new(kp, 0.0, 0.0, 0.0, 100.0);
            let output = pid.update(setpoint, measured, 0.1);
            prop_assert!(output >= 0.0 && output <= 100.0,
                "output {} out of [0, 100]", output);
        }

        #[test]
        fn pid_output_bounded_by_saturation_limits(
            setpoint in -100.0..100.0_f64,
            measured in -100.0..100.0_f64,
            kp in 0.1..50.0_f64,
            ki in 0.0..10.0_f64,
            kd in 0.0..10.0_f64,
        ) {
            let lo = -42.0;
            let hi = 42.0;
            let mut pid = PidController::new(kp, ki, kd, lo, hi);
            // Feed multiple steps to accumulate integral
            for _ in 0..5 {
                let output = pid.update(setpoint, measured, 0.1);
                prop_assert!(output >= lo && output <= hi,
                    "output {} out of [{}, {}]", output, lo, hi);
            }
        }
    }
}

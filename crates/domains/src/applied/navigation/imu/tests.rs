use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::navigation::imu::ontology::*;
use crate::applied::navigation::imu::strapdown::*;
use crate::formal::math::geometry::point::Point3;
use crate::formal::math::rotation::quaternion::Quaternion;
use crate::natural::physics::kinematics::velocity::Velocity;

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

#[test]
fn imu_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<ImuTaxonomy>>().unwrap();
}

#[test]
fn imu_ontology_validates() {
    ImuOntology::validate().unwrap();
}

#[test]
fn bias_is_a_measurement() {
    assert!(BiasIsAMeasurement.holds());
}

#[test]
fn specific_force_definition() {
    assert!(SpecificForceDefinition.holds());
}

// ---------------------------------------------------------------------------
// Strapdown mechanization proofs
// ---------------------------------------------------------------------------

#[test]
fn stationary_imu_maintains_position() {
    // IMU at rest on Earth surface: accelerometer reads [0,0,-g]
    // (specific force = -gravity in body frame when body=NED)
    let state = NavState {
        position: Point3::new(0.0, 0.0, 0.0),
        velocity: Velocity::zero(),
        attitude: Quaternion::identity(), // body = nav frame
    };
    let sample = ImuSample {
        specific_force: [0.0, 0.0, -gravity_ned()[2]], // cancels gravity
        angular_rate: [0.0, 0.0, 0.0],
        dt: 1.0,
    };
    let next = mechanize(&state, &sample);
    // Position should not change (stationary)
    assert!(
        next.position.distance_to(&state.position) < 1e-6,
        "stationary IMU should not move: distance={}",
        next.position.distance_to(&state.position)
    );
}

#[test]
fn constant_velocity_propagation() {
    let state = NavState {
        position: Point3::new(0.0, 0.0, 0.0),
        velocity: Velocity::new(10.0, 0.0, 0.0), // 10 m/s north
        attitude: Quaternion::identity(),
    };
    // Specific force cancels gravity (level flight / zero acceleration)
    let sample = ImuSample {
        specific_force: [0.0, 0.0, -gravity_ned()[2]],
        angular_rate: [0.0, 0.0, 0.0],
        dt: 1.0,
    };
    let next = mechanize(&state, &sample);
    // Should move ~10m north
    assert!(
        (next.position.x - 10.0).abs() < 0.1,
        "should move 10m north: x={}",
        next.position.x
    );
}

#[test]
fn gyro_rotates_attitude() {
    let state = NavState {
        position: Point3::origin(),
        velocity: Velocity::zero(),
        attitude: Quaternion::identity(),
    };
    // Constant yaw rate of 0.1 rad/s about z-axis for 1 second
    let sample = ImuSample {
        specific_force: [0.0, 0.0, -gravity_ned()[2]],
        angular_rate: [0.0, 0.0, 0.1],
        dt: 1.0,
    };
    let next = mechanize(&state, &sample);
    // Attitude should have changed (not identity anymore)
    assert!(next.attitude != Quaternion::identity());
}

#[test]
fn zero_dt_preserves_state() {
    let state = NavState {
        position: Point3::new(1.0, 2.0, 3.0),
        velocity: Velocity::new(4.0, 5.0, 6.0),
        attitude: Quaternion::from_axis_angle([0.0, 0.0, 1.0], 0.5),
    };
    let sample = ImuSample {
        specific_force: [1.0, 2.0, 3.0],
        angular_rate: [0.1, 0.2, 0.3],
        dt: 0.0,
    };
    let next = mechanize(&state, &sample);
    assert!(
        next.position.distance_to(&state.position) < 1e-12,
        "zero dt should preserve position"
    );
}

// ---------------------------------------------------------------------------
// Signal processing: low-pass filtering of IMU data
// ---------------------------------------------------------------------------

#[test]
fn filtered_imu_has_lower_variance() {
    use crate::formal::math::signal_processing::filter::FirstOrderLowPass;

    // Generate noisy IMU samples: constant signal + high-frequency noise
    let n = 200;
    let dt = 0.01;
    let cutoff = 5.0; // 5 Hz cutoff, well below noise

    let mut accel_filter = [
        FirstOrderLowPass::new(cutoff, dt),
        FirstOrderLowPass::new(cutoff, dt),
        FirstOrderLowPass::new(cutoff, dt),
    ];
    let mut gyro_filter = [
        FirstOrderLowPass::new(cutoff, dt),
        FirstOrderLowPass::new(cutoff, dt),
        FirstOrderLowPass::new(cutoff, dt),
    ];

    let true_accel = [0.0, 0.0, -gravity_ned()[2]];
    let mut raw_samples = Vec::new();
    let mut filtered_samples = Vec::new();

    // Simple deterministic "noise": alternating +/- pattern
    for i in 0..n {
        let noise = if i % 2 == 0 { 2.0 } else { -2.0 };
        let sample = ImuSample {
            specific_force: [
                true_accel[0] + noise,
                true_accel[1] + noise * 0.5,
                true_accel[2] + noise * 0.3,
            ],
            angular_rate: [noise * 0.01, noise * 0.02, noise * 0.005],
            dt,
        };
        let filtered = filter_imu_sample(&sample, &mut accel_filter, &mut gyro_filter);
        raw_samples.push(sample);
        filtered_samples.push(filtered);
    }

    // Compute variance of raw vs filtered specific_force[0]
    // (skip first 20 samples to let filter settle)
    let settle = 20;
    let raw_mean: f64 = raw_samples[settle..]
        .iter()
        .map(|s| s.specific_force[0])
        .sum::<f64>()
        / (n - settle) as f64;
    let raw_var: f64 = raw_samples[settle..]
        .iter()
        .map(|s| (s.specific_force[0] - raw_mean).powi(2))
        .sum::<f64>()
        / (n - settle) as f64;

    let filt_mean: f64 = filtered_samples[settle..]
        .iter()
        .map(|s| s.specific_force[0])
        .sum::<f64>()
        / (n - settle) as f64;
    let filt_var: f64 = filtered_samples[settle..]
        .iter()
        .map(|s| (s.specific_force[0] - filt_mean).powi(2))
        .sum::<f64>()
        / (n - settle) as f64;

    assert!(
        filt_var < raw_var,
        "filtered variance ({}) should be less than raw ({})",
        filt_var,
        raw_var
    );
}

// ---------------------------------------------------------------------------
// Proptest
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    fn arb_imu_sample() -> impl Strategy<Value = ImuSample> {
        (
            -20.0..20.0_f64,
            -20.0..20.0_f64,
            -20.0..20.0_f64,
            -1.0..1.0_f64,
            -1.0..1.0_f64,
            -1.0..1.0_f64,
            0.001..0.1_f64,
        )
            .prop_map(|(fx, fy, fz, wx, wy, wz, dt)| ImuSample {
                specific_force: [fx, fy, fz],
                angular_rate: [wx, wy, wz],
                dt,
            })
    }

    proptest! {
        #[test]
        fn filtered_output_magnitude_bounded_by_input(
            fx in -20.0..20.0_f64,
            fy in -20.0..20.0_f64,
            fz in -20.0..20.0_f64,
            wx in -1.0..1.0_f64,
            wy in -1.0..1.0_f64,
            wz in -1.0..1.0_f64,
        ) {
            use crate::formal::math::signal_processing::filter::FirstOrderLowPass;
            let dt = 0.01;
            let cutoff = 10.0;
            let mut af = [
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
            ];
            let mut gf = [
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
            ];
            let sample = ImuSample {
                specific_force: [fx, fy, fz],
                angular_rate: [wx, wy, wz],
                dt,
            };
            let filtered = filter_imu_sample(&sample, &mut af, &mut gf);
            // First-order low-pass with zero initial: output = alpha * input
            // so |output| <= |input| (alpha in (0,1])
            prop_assert!(filtered.specific_force[0].abs() <= fx.abs() + 1e-10);
            prop_assert!(filtered.specific_force[1].abs() <= fy.abs() + 1e-10);
            prop_assert!(filtered.specific_force[2].abs() <= fz.abs() + 1e-10);
            prop_assert!(filtered.angular_rate[0].abs() <= wx.abs() + 1e-10);
            prop_assert!(filtered.angular_rate[1].abs() <= wy.abs() + 1e-10);
            prop_assert!(filtered.angular_rate[2].abs() <= wz.abs() + 1e-10);
        }

        #[test]
        fn filter_imu_is_deterministic(sample in arb_imu_sample()) {
            use crate::formal::math::signal_processing::filter::FirstOrderLowPass;
            let dt = 0.01;
            let cutoff = 10.0;
            let mut af1 = [
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
            ];
            let mut gf1 = [
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
            ];
            let mut af2 = [
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
            ];
            let mut gf2 = [
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
                FirstOrderLowPass::new(cutoff, dt),
            ];
            let r1 = filter_imu_sample(&sample, &mut af1, &mut gf1);
            let r2 = filter_imu_sample(&sample, &mut af2, &mut gf2);
            prop_assert!((r1.specific_force[0] - r2.specific_force[0]).abs() < 1e-15);
            prop_assert!((r1.specific_force[1] - r2.specific_force[1]).abs() < 1e-15);
            prop_assert!((r1.specific_force[2] - r2.specific_force[2]).abs() < 1e-15);
            prop_assert!((r1.angular_rate[0] - r2.angular_rate[0]).abs() < 1e-15);
            prop_assert!((r1.angular_rate[1] - r2.angular_rate[1]).abs() < 1e-15);
            prop_assert!((r1.angular_rate[2] - r2.angular_rate[2]).abs() < 1e-15);
        }

        #[test]
        fn strapdown_is_deterministic(sample in arb_imu_sample()) {
            let state = NavState {
                position: Point3::origin(),
                velocity: Velocity::zero(),
                attitude: Quaternion::identity(),
            };
            let r1 = mechanize(&state, &sample);
            let r2 = mechanize(&state, &sample);
            prop_assert!((r1.position.x - r2.position.x).abs() < 1e-15);
            prop_assert!((r1.position.y - r2.position.y).abs() < 1e-15);
            prop_assert!((r1.position.z - r2.position.z).abs() < 1e-15);
            prop_assert!((r1.velocity.vx - r2.velocity.vx).abs() < 1e-15);
            prop_assert!((r1.velocity.vy - r2.velocity.vy).abs() < 1e-15);
            prop_assert!((r1.velocity.vz - r2.velocity.vz).abs() < 1e-15);
        }

        #[test]
        fn attitude_stays_unit_quaternion(sample in arb_imu_sample()) {
            let state = NavState {
                position: Point3::origin(),
                velocity: Velocity::zero(),
                attitude: Quaternion::identity(),
            };
            let next = mechanize(&state, &sample);
            let norm = next.attitude.norm();
            prop_assert!((norm - 1.0).abs() < 1e-10,
                "attitude quaternion norm = {} (should be 1.0)", norm);
        }
    }
}

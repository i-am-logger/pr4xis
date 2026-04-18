//! Odometry methods — how relative motion is estimated.
//!
//! Odometry methods estimate motion from proprioceptive or exteroceptive
//! measurements. This ontology covers only the methods themselves; the state
//! they estimate (Position, Heading, Velocity) lives in the shared
//! `ObservableProperty` ontology, and the method → property mapping is
//! expressed via the `OdometryToProperty` functor (see `property_functor.rs`).
//!
//! Source: Borenstein et al. (1996) "Where am I?"; Thrun, Burgard & Fox (2005)
//!         Chapter 5; Scaramuzza & Fraundorfer (2011).

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Odometry",
    source: "Borenstein et al. (1996); Thrun, Burgard & Fox (2005); Scaramuzza & Fraundorfer (2011)",
    being: Process,

    concepts: [
        Source,
        WheelEncoder,
        VisualOdometry,
        InertialOdometry,
        LaserOdometry,
    ],

    labels: {
        Source: ("en", "Odometry source", "Abstract odometry source — the root of the method taxonomy."),
        WheelEncoder: ("en", "Wheel encoder", "Counts wheel rotations to estimate distance traveled. Borenstein et al. (1996)."),
        VisualOdometry: ("en", "Visual odometry", "Tracks features between camera frames to estimate motion. Scaramuzza & Fraundorfer (2011)."),
        InertialOdometry: ("en", "Inertial odometry", "Integrates IMU measurements to estimate motion; unbounded drift."),
        LaserOdometry: ("en", "Laser odometry", "Matches consecutive laser scans to estimate motion."),
    },

    is_a: [
        (WheelEncoder, Source),
        (VisualOdometry, Source),
        (InertialOdometry, Source),
        (LaserOdometry, Source),
    ],
}

/// Quality: Drift rate (meters of error per meter traveled).
///
/// Source: Borenstein et al. (1996), Table 3.
#[derive(Debug, Clone)]
pub struct DriftRate;

impl Quality for DriftRate {
    type Individual = OdometryConcept;
    type Value = &'static str;

    fn get(&self, source: &OdometryConcept) -> Option<&'static str> {
        Some(match source {
            OdometryConcept::Source => "varies by type",
            OdometryConcept::WheelEncoder => "1-5% of distance traveled",
            OdometryConcept::VisualOdometry => "0.5-2% of distance traveled",
            OdometryConcept::InertialOdometry => "grows as O(t^3) — unbounded",
            OdometryConcept::LaserOdometry => "0.5-1% of distance traveled",
        })
    }
}

/// Quality: Update rate in Hz.
///
/// Source: Scaramuzza & Fraundorfer (2011).
#[derive(Debug, Clone)]
pub struct UpdateRate;

impl Quality for UpdateRate {
    type Individual = OdometryConcept;
    type Value = &'static str;

    fn get(&self, source: &OdometryConcept) -> Option<&'static str> {
        Some(match source {
            OdometryConcept::Source => "varies",
            OdometryConcept::WheelEncoder => "~100 Hz",
            OdometryConcept::VisualOdometry => "~30 Hz (camera framerate)",
            OdometryConcept::InertialOdometry => "~200-400 Hz (IMU rate)",
            OdometryConcept::LaserOdometry => "~10-20 Hz (scan rate)",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Drift is unbounded: odometry error grows without bound over time.
///
/// Without an absolute reference (GNSS, landmarks), the position error
/// from odometry never decreases — it accumulates indefinitely.
///
/// Source: Thrun, Burgard & Fox (2005) Section 5.4.
pub struct DriftIsUnbounded;

impl Axiom for DriftIsUnbounded {
    fn description(&self) -> &str {
        "odometry error grows without bound (no absolute reference)"
    }
    fn holds(&self) -> bool {
        let drift_rate = 0.02;
        let d1 = 100.0;
        let d2 = 1000.0;
        let e1 = drift_rate * d1;
        let e2 = drift_rate * d2;
        e2 > e1 && e1 > 0.0
    }
}
pr4xis::register_axiom!(DriftIsUnbounded);

/// Relative motion only: odometry measures CHANGE, not absolute position.
///
/// Source: Borenstein et al. (1996).
pub struct RelativeMotionOnly;

impl Axiom for RelativeMotionOnly {
    fn description(&self) -> &str {
        "odometry measures change in position, not absolute position"
    }
    fn holds(&self) -> bool {
        let start_a: [f64; 2] = [0.0, 0.0];
        let start_b: [f64; 2] = [100.0, 200.0];
        let delta: [f64; 2] = [10.0, 5.0];

        let end_a = [start_a[0] + delta[0], start_a[1] + delta[1]];
        let end_b = [start_b[0] + delta[0], start_b[1] + delta[1]];

        let disp_a = [end_a[0] - start_a[0], end_a[1] - start_a[1]];
        let disp_b = [end_b[0] - start_b[0], end_b[1] - start_b[1]];
        (disp_a[0] - disp_b[0]).abs() < 1e-10 && (disp_a[1] - disp_b[1]).abs() < 1e-10
    }
}
pr4xis::register_axiom!(RelativeMotionOnly);

/// Slip corrupts wheel odometry: wheel slip causes measurement error.
///
/// Source: Borenstein et al. (1996), Section 3.2.
pub struct SlipCorruptsWheelOdometry;

impl Axiom for SlipCorruptsWheelOdometry {
    fn description(&self) -> &str {
        "wheel slip causes wheel encoder error"
    }
    fn holds(&self) -> bool {
        let encoder_distance = 100.0_f64;
        let slip_ratio = 0.1_f64;
        let actual_distance = encoder_distance * (1.0 - slip_ratio);
        let error = (encoder_distance - actual_distance).abs();
        error > 0.0 && actual_distance < encoder_distance
    }
}
pr4xis::register_axiom!(SlipCorruptsWheelOdometry);

impl Ontology for OdometryOntology {
    type Cat = OdometryCategory;
    type Qual = DriftRate;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(DriftIsUnbounded),
            Box::new(RelativeMotionOnly),
            Box::new(SlipCorruptsWheelOdometry),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<OdometryCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        OdometryOntology::validate().unwrap();
    }
}

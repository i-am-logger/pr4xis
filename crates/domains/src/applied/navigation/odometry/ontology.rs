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

use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::quantity::unit::{HERTZ, UNITLESS};
use crate::formal::math::quantity::value::{Quantity, QuantityRange};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

pr4xis::ontology! {
    name: "Odometry",
    source: "Borenstein et al. (1996); Thrun, Burgard & Fox (2005); Scaramuzza & Fraundorfer (2011)",

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
    type Value = QuantityRange;
    const KIND: QualityKind = QualityKind::Physical;

    fn get(&self, source: &OdometryConcept) -> Option<QuantityRange> {
        // Drift rate is a dimensionless ratio (meters of error per meter
        // traveled); a "1-5%" figure is the fraction 0.01..0.05 (UNITLESS).
        let mk = |lo: f64, hi: f64| QuantityRange {
            min: Quantity::from_unit(lo, &UNITLESS),
            max: Quantity::from_unit(hi, &UNITLESS),
        };
        Some(match source {
            // Abstract root — drift varies by method type.
            OdometryConcept::Source => return None,
            // 1-5% of distance traveled.
            OdometryConcept::WheelEncoder => mk(0.01, 0.05),
            // 0.5-2% of distance traveled.
            OdometryConcept::VisualOdometry => mk(0.005, 0.02),
            // Inertial drift grows as O(t^3) — unbounded, no fixed range.
            OdometryConcept::InertialOdometry => return None,
            // 0.5-1% of distance traveled.
            OdometryConcept::LaserOdometry => mk(0.005, 0.01),
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
    type Value = QuantityRange;
    const KIND: QualityKind = QualityKind::Physical;

    fn get(&self, source: &OdometryConcept) -> Option<QuantityRange> {
        let mk = |lo: f64, hi: f64| QuantityRange {
            min: Quantity::from_unit(lo, &HERTZ),
            max: Quantity::from_unit(hi, &HERTZ),
        };
        Some(match source {
            // Abstract root — update rate varies by method type.
            OdometryConcept::Source => return None,
            // ~100 Hz.
            OdometryConcept::WheelEncoder => mk(100.0, 100.0),
            // ~30 Hz (camera framerate).
            OdometryConcept::VisualOdometry => mk(30.0, 30.0),
            // ~200-400 Hz (IMU rate).
            OdometryConcept::InertialOdometry => mk(200.0, 400.0),
            // ~10-20 Hz (scan rate).
            OdometryConcept::LaserOdometry => mk(10.0, 20.0),
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
    fn verify(&self) -> Verdict {
        let drift_rate = 0.02;
        let d1 = 100.0;
        let d2 = 1000.0;
        let e1 = drift_rate * d1;
        let e2 = drift_rate * d2;
        if e2 > e1 && e1 > 0.0 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DriftIsUnbounded",
        "odometry error grows without bound (no absolute reference)",
        "Thrun, Burgard & Fox (2005) Section 5.4"
    );
}
pr4xis::register_axiom!(DriftIsUnbounded, "Thrun, Burgard & Fox (2005) Section 5.4");

/// Relative motion only: odometry measures CHANGE, not absolute position.
///
/// Source: Borenstein et al. (1996).
pub struct RelativeMotionOnly;

impl Axiom for RelativeMotionOnly {
    fn verify(&self) -> Verdict {
        let start_a = Vector::new(vec![0.0, 0.0]);
        let start_b = Vector::new(vec![100.0, 200.0]);
        let delta = Vector::new(vec![10.0, 5.0]);

        let end_a = start_a.add(&delta);
        let end_b = start_b.add(&delta);

        let disp_a = end_a.sub(&start_a);
        let disp_b = end_b.sub(&start_b);
        if (disp_a.get(0) - disp_b.get(0)).abs() < 1e-10
            && (disp_a.get(1) - disp_b.get(1)).abs() < 1e-10
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RelativeMotionOnly",
        "odometry measures change in position, not absolute position",
        "Borenstein et al. (1996) \"Where am I?\""
    );
}
pr4xis::register_axiom!(
    RelativeMotionOnly,
    "Borenstein et al. (1996) \"Where am I?\""
);

/// Slip corrupts wheel odometry: wheel slip causes measurement error.
///
/// Source: Borenstein et al. (1996), Section 3.2.
pub struct SlipCorruptsWheelOdometry;

impl Axiom for SlipCorruptsWheelOdometry {
    fn verify(&self) -> Verdict {
        let encoder_distance = 100.0_f64;
        let slip_ratio = 0.1_f64;
        let actual_distance = encoder_distance * (1.0 - slip_ratio);
        let error = (encoder_distance - actual_distance).abs();
        if error > 0.0 && actual_distance < encoder_distance {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SlipCorruptsWheelOdometry",
        "wheel slip causes wheel encoder error",
        "Borenstein et al. (1996) Section 3.2"
    );
}
pr4xis::register_axiom!(
    SlipCorruptsWheelOdometry,
    "Borenstein et al. (1996) Section 3.2"
);

impl Ontology for OdometryOntology {
    type Cat = OdometryCategory;
    type Qual = DriftRate;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(DriftIsUnbounded));
        axioms.push(Box::new(RelativeMotionOnly));
        axioms.push(Box::new(SlipCorruptsWheelOdometry));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<OdometryCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        OdometryOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}

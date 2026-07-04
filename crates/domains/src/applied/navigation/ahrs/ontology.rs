//! AHRS filter types — methods for estimating attitude.
//!
//! This ontology covers only the filter methods. The attitude state they
//! produce (Roll, Pitch, Yaw, Attitude) lives in the shared
//! `ObservableProperty` ontology; each filter's mapping to the property it
//! produces is expressed via the `AhrsToProperty` functor (see
//! `property_functor.rs`).
//!
//! Source: Madgwick (2010), Mahony et al. (2008), Titterton & Weston (2004)
//!         Chapter 10.

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

use crate::formal::math::quantity::unit::DEGREE;
use crate::formal::math::quantity::value::{Quantity, QuantityRange};

pr4xis::ontology! {
    name: "Ahrs",
    source: "Madgwick (2010); Mahony et al. (2008); Titterton & Weston (2004)",

    concepts: [
        Filter,
        ComplementaryFilter,
        MahonyFilter,
        MadgwickFilter,
        ExtendedKalmanFilter,
    ],

    labels: {
        Filter: ("en", "Attitude filter", "Abstract attitude estimation filter — the root of the filter taxonomy."),
        ComplementaryFilter: ("en", "Complementary filter", "Simple complementary filter: high-pass gyro + low-pass accel."),
        MahonyFilter: ("en", "Mahony filter", "Mahony (2008): proportional-integral filter on SO(3)."),
        MadgwickFilter: ("en", "Madgwick filter", "Madgwick (2010): gradient descent orientation filter."),
        ExtendedKalmanFilter: ("en", "Extended Kalman Filter", "Extended Kalman Filter for attitude estimation."),
    },

    // Each more-accurate filter refines the simpler one (is_a chain).
    is_a: [
        (ComplementaryFilter, Filter),
        (MahonyFilter, Filter),
        (MadgwickFilter, Filter),
        (ExtendedKalmanFilter, Filter),
        (MahonyFilter, ComplementaryFilter),
        (MadgwickFilter, MahonyFilter),
        (ExtendedKalmanFilter, MadgwickFilter),
    ],
}

/// Quality: static-condition RMS attitude accuracy of each filter, as an angular
/// [`QuantityRange`] (degrees), NOT a prose string.
///
/// `None` for the abstract `Filter` (accuracy is implementation-dependent). The
/// ranges are the typical 1σ static-RMS figures from Madgwick's (2010)
/// comparative study; a smaller range is a more accurate filter.
#[derive(Debug, Clone)]
pub struct AttitudeAccuracy;

impl Quality for AttitudeAccuracy {
    type Individual = AhrsConcept;
    type Value = QuantityRange;
    const KIND: QualityKind = QualityKind::Physical;

    fn get(&self, filter: &AhrsConcept) -> Option<QuantityRange> {
        let deg = |lo: f64, hi: f64| QuantityRange {
            min: Quantity::from_unit(lo, &DEGREE),
            max: Quantity::from_unit(hi, &DEGREE),
        };
        Some(match filter {
            // Abstract root — accuracy is implementation-dependent.
            AhrsConcept::Filter => return None,
            AhrsConcept::ComplementaryFilter => deg(2.0, 5.0),
            AhrsConcept::MahonyFilter => deg(1.0, 3.0),
            AhrsConcept::MadgwickFilter => deg(0.5, 2.0),
            AhrsConcept::ExtendedKalmanFilter => deg(0.1, 1.0),
        })
    }
}

/// Quality: Computational cost (relative FLOPS per update).
#[derive(Debug, Clone)]
pub struct ComputationalCost;

impl Quality for ComputationalCost {
    type Individual = AhrsConcept;
    type Value = &'static str;

    fn get(&self, filter: &AhrsConcept) -> Option<&'static str> {
        Some(match filter {
            AhrsConcept::Filter => "varies",
            AhrsConcept::ComplementaryFilter => "~20 FLOPS (lowest)",
            AhrsConcept::MahonyFilter => "~50 FLOPS",
            AhrsConcept::MadgwickFilter => "~100 FLOPS",
            AhrsConcept::ExtendedKalmanFilter => "~500+ FLOPS (highest)",
        })
    }
}

/// Gravity gives level attitude: accelerometer at rest determines roll/pitch.
///
/// Source: Titterton & Weston (2004) Section 10.3.
pub struct GravityGivesLevelAttitude;

impl Axiom for GravityGivesLevelAttitude {
    fn verify(&self) -> Verdict {
        // Standard gravity from the cited quantity constant (BIPM), not a raw
        // 9.80665 literal.
        let g = crate::formal::math::quantity::constants::standard_gravity().value;
        let tilt = 10.0_f64.to_radians();
        let ax = 0.0_f64;
        let ay = g * tilt.sin();
        let az = -g * tilt.cos();

        let roll_recovered = ay.atan2(-az);
        let pitch_recovered = (-ax).atan2((az * az + ay * ay).sqrt());

        if (roll_recovered - tilt).abs() < 0.01 && pitch_recovered.abs() < 0.01 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GravityGivesLevelAttitude",
        "accelerometer at rest determines roll/pitch via gravity vector",
        "Titterton & Weston (2004) Section 10.3"
    );
}
pr4xis::register_axiom!(
    GravityGivesLevelAttitude,
    "Titterton & Weston (2004) Section 10.3"
);

/// Magnetometer gives heading: mag + level attitude determines yaw.
///
/// Source: Groves (2013) Section 6.4.
pub struct MagnetometerGivesHeading;

impl Axiom for MagnetometerGivesHeading {
    fn verify(&self) -> Verdict {
        let b_horizontal = 20.0e-6;
        let bx = b_horizontal;
        let by = 0.0_f64;

        let heading = (-by).atan2(bx);
        if heading.abs() < 0.01 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MagnetometerGivesHeading",
        "magnetometer + level attitude determines yaw (heading)",
        "Groves (2013) Section 6.4"
    );
}
pr4xis::register_axiom!(MagnetometerGivesHeading, "Groves (2013) Section 6.4");

/// Gyroscope integration drifts: gyro-only attitude accumulates error.
///
/// Source: Titterton & Weston (2004) Section 10.2.
pub struct GyroIntegrationDrifts;

impl Axiom for GyroIntegrationDrifts {
    fn verify(&self) -> Verdict {
        let gyro_bias_deg_per_hr = 1.0;
        let gyro_bias_rad_per_s = gyro_bias_deg_per_hr * core::f64::consts::PI / (180.0 * 3600.0);
        let t = 3600.0;
        let drift_rad = gyro_bias_rad_per_s * t;
        let drift_deg = drift_rad * 180.0 / std::f64::consts::PI;
        if (drift_deg - 1.0).abs() < 0.01 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GyroIntegrationDrifts",
        "gyroscope-only attitude drifts over time (needs external correction)",
        "Titterton & Weston (2004) Section 10.2"
    );
}
pr4xis::register_axiom!(
    GyroIntegrationDrifts,
    "Titterton & Weston (2004) Section 10.2"
);

impl Ontology for AhrsOntology {
    type Cat = AhrsCategory;
    type Qual = AttitudeAccuracy;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(GravityGivesLevelAttitude));
        axioms.push(Box::new(MagnetometerGivesHeading));
        axioms.push(Box::new(GyroIntegrationDrifts));
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
        assert_category_laws::<AhrsCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        AhrsOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}

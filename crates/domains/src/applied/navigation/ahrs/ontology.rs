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

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Ahrs",
    source: "Madgwick (2010); Mahony et al. (2008); Titterton & Weston (2004)",
    being: Process,

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

/// Quality: Attitude accuracy (degrees RMS) for each filter type.
///
/// Source: Madgwick (2010), comparative study results.
#[derive(Debug, Clone)]
pub struct AttitudeAccuracy;

impl Quality for AttitudeAccuracy {
    type Individual = AhrsConcept;
    type Value = &'static str;

    fn get(&self, filter: &AhrsConcept) -> Option<&'static str> {
        Some(match filter {
            AhrsConcept::Filter => "depends on implementation",
            AhrsConcept::ComplementaryFilter => "~2-5 deg RMS static",
            AhrsConcept::MahonyFilter => "~1-3 deg RMS static",
            AhrsConcept::MadgwickFilter => "~0.5-2 deg RMS static",
            AhrsConcept::ExtendedKalmanFilter => "~0.1-1 deg RMS static",
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
    fn description(&self) -> &str {
        "accelerometer at rest determines roll/pitch via gravity vector"
    }
    fn holds(&self) -> bool {
        let g = 9.80665_f64;
        let tilt = 10.0_f64.to_radians();
        let ax = 0.0_f64;
        let ay = g * tilt.sin();
        let az = -g * tilt.cos();

        let roll_recovered = ay.atan2(-az);
        let pitch_recovered = (-ax).atan2((az * az + ay * ay).sqrt());

        (roll_recovered - tilt).abs() < 0.01 && pitch_recovered.abs() < 0.01
    }
}
pr4xis::register_axiom!(GravityGivesLevelAttitude);

/// Magnetometer gives heading: mag + level attitude determines yaw.
///
/// Source: Groves (2013) Section 6.4.
pub struct MagnetometerGivesHeading;

impl Axiom for MagnetometerGivesHeading {
    fn description(&self) -> &str {
        "magnetometer + level attitude determines yaw (heading)"
    }
    fn holds(&self) -> bool {
        let b_horizontal = 20.0e-6;
        let bx = b_horizontal;
        let by = 0.0_f64;

        let heading = (-by).atan2(bx);
        heading.abs() < 0.01
    }
}
pr4xis::register_axiom!(MagnetometerGivesHeading);

/// Gyroscope integration drifts: gyro-only attitude accumulates error.
///
/// Source: Titterton & Weston (2004) Section 10.2.
pub struct GyroIntegrationDrifts;

impl Axiom for GyroIntegrationDrifts {
    fn description(&self) -> &str {
        "gyroscope-only attitude drifts over time (needs external correction)"
    }
    fn holds(&self) -> bool {
        let gyro_bias_deg_per_hr = 1.0;
        let gyro_bias_rad_per_s = gyro_bias_deg_per_hr * std::f64::consts::PI / (180.0 * 3600.0);
        let t = 3600.0;
        let drift_rad = gyro_bias_rad_per_s * t;
        let drift_deg = drift_rad * 180.0 / std::f64::consts::PI;
        (drift_deg - 1.0).abs() < 0.01
    }
}
pr4xis::register_axiom!(GyroIntegrationDrifts);

impl Ontology for AhrsOntology {
    type Cat = AhrsCategory;
    type Qual = AttitudeAccuracy;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(GravityGivesLevelAttitude),
            Box::new(MagnetometerGivesHeading),
            Box::new(GyroIntegrationDrifts),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<AhrsCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        AhrsOntology::validate().unwrap();
    }
}

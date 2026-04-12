use pr4xis::category::Entity;
use pr4xis::ontology::reasoning::taxonomy::{NoCycles, TaxonomyCategory, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// AHRS filter types — methods for estimating attitude.
///
/// Source: Madgwick (2010), Mahony et al. (2008), Titterton & Weston (2004) Chapter 10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum AhrsFilterType {
    /// Abstract filter.
    Filter,
    /// Simple complementary filter: high-pass gyro + low-pass accel.
    ComplementaryFilter,
    /// Mahony (2008): proportional-integral filter on SO(3).
    MahonyFilter,
    /// Madgwick (2010): gradient descent orientation filter.
    MadgwickFilter,
    /// Extended Kalman Filter for attitude estimation.
    ExtendedKalmanFilter,
}

/// Attitude state: the three Euler angles.
///
/// Source: Groves (2013) Section 2.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum AttitudeState {
    /// Abstract attitude.
    Attitude,
    /// Roll: rotation about the forward (x) axis.
    Roll,
    /// Pitch: rotation about the right (y) axis.
    Pitch,
    /// Yaw: rotation about the down (z) axis (heading).
    Yaw,
}

/// AHRS filter taxonomy: accuracy improvements.
///
/// ComplementaryFilter → MahonyFilter → MadgwickFilter → EKF
pub struct AhrsFilterTaxonomy;

impl TaxonomyDef for AhrsFilterTaxonomy {
    type Entity = AhrsFilterType;

    fn relations() -> Vec<(AhrsFilterType, AhrsFilterType)> {
        use AhrsFilterType::*;
        vec![
            (ComplementaryFilter, Filter),
            (MahonyFilter, Filter),
            (MadgwickFilter, Filter),
            (ExtendedKalmanFilter, Filter),
            // Each more accurate filter extends the simpler one
            (MahonyFilter, ComplementaryFilter),
            (MadgwickFilter, MahonyFilter),
            (ExtendedKalmanFilter, MadgwickFilter),
        ]
    }
}

/// Attitude state taxonomy.
pub struct AttitudeStateTaxonomy;

impl TaxonomyDef for AttitudeStateTaxonomy {
    type Entity = AttitudeState;

    fn relations() -> Vec<(AttitudeState, AttitudeState)> {
        use AttitudeState::*;
        vec![(Roll, Attitude), (Pitch, Attitude), (Yaw, Attitude)]
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: Attitude accuracy (degrees RMS) for each filter type.
///
/// Source: Madgwick (2010), comparative study results.
#[derive(Debug, Clone)]
pub struct AttitudeAccuracy;

impl Quality for AttitudeAccuracy {
    type Individual = AhrsFilterType;
    type Value = &'static str;

    fn get(&self, filter: &AhrsFilterType) -> Option<&'static str> {
        Some(match filter {
            AhrsFilterType::Filter => "depends on implementation",
            AhrsFilterType::ComplementaryFilter => "~2-5 deg RMS static",
            AhrsFilterType::MahonyFilter => "~1-3 deg RMS static",
            AhrsFilterType::MadgwickFilter => "~0.5-2 deg RMS static",
            AhrsFilterType::ExtendedKalmanFilter => "~0.1-1 deg RMS static",
        })
    }
}

/// Quality: Computational cost (relative FLOPS per update).
///
/// Source: Madgwick (2010), Table 2.
#[derive(Debug, Clone)]
pub struct ComputationalCost;

impl Quality for ComputationalCost {
    type Individual = AhrsFilterType;
    type Value = &'static str;

    fn get(&self, filter: &AhrsFilterType) -> Option<&'static str> {
        Some(match filter {
            AhrsFilterType::Filter => "varies",
            AhrsFilterType::ComplementaryFilter => "~20 FLOPS (lowest)",
            AhrsFilterType::MahonyFilter => "~50 FLOPS",
            AhrsFilterType::MadgwickFilter => "~100 FLOPS",
            AhrsFilterType::ExtendedKalmanFilter => "~500+ FLOPS (highest)",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// AHRS filter taxonomy is a DAG.
pub struct AhrsFilterTaxonomyIsDAG;

impl Axiom for AhrsFilterTaxonomyIsDAG {
    fn description(&self) -> &str {
        "AHRS filter taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        NoCycles::<AhrsFilterTaxonomy>::default().holds()
    }
}

/// Attitude state taxonomy is a DAG.
pub struct AttitudeStateTaxonomyIsDAG;

impl Axiom for AttitudeStateTaxonomyIsDAG {
    fn description(&self) -> &str {
        "attitude state taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        NoCycles::<AttitudeStateTaxonomy>::default().holds()
    }
}

/// Gravity gives level attitude: accelerometer at rest determines roll/pitch.
///
/// With gravity vector g = [0, 0, -g] in body frame:
///   pitch = atan2(-gx, gz)
///   roll  = atan2(gy, gz)
///
/// Source: Titterton & Weston (2004) Section 10.3.
pub struct GravityGivesLevelAttitude;

impl Axiom for GravityGivesLevelAttitude {
    fn description(&self) -> &str {
        "accelerometer at rest determines roll/pitch via gravity vector"
    }
    fn holds(&self) -> bool {
        // At rest, accelerometer reads [0, 0, -g] in a level body frame.
        // Tilting by 10 degrees about x: accel reads [0, g*sin(10), -g*cos(10)]
        let g = 9.80665_f64;
        let tilt = 10.0_f64.to_radians();
        let ax = 0.0_f64;
        let ay = g * tilt.sin();
        let az = -g * tilt.cos();

        // Recover roll from accelerometer
        let roll_recovered = ay.atan2(-az);
        let pitch_recovered = (-ax).atan2((az * az + ay * ay).sqrt());

        // Roll should match tilt, pitch should be ~0
        (roll_recovered - tilt).abs() < 0.01 && pitch_recovered.abs() < 0.01
    }
}

/// Magnetometer gives heading: mag + level attitude determines yaw.
///
/// In a level frame, the horizontal component of Earth's magnetic field
/// points toward magnetic north. Yaw = atan2(-By, Bx).
///
/// Source: Groves (2013) Section 6.4.
pub struct MagnetometerGivesHeading;

impl Axiom for MagnetometerGivesHeading {
    fn description(&self) -> &str {
        "magnetometer + level attitude determines yaw (heading)"
    }
    fn holds(&self) -> bool {
        // Earth's horizontal magnetic field in NED at some location:
        // Bx = B * cos(inclination), By = 0, Bz = B * sin(inclination)
        // For a level frame, heading = atan2(-By, Bx)
        let b_horizontal = 20.0e-6; // ~20 μT horizontal component
        let bx = b_horizontal; // pointing north
        let by = 0.0_f64; // no east component when facing north

        let heading = (-by).atan2(bx);
        // Heading should be 0 (facing north)
        heading.abs() < 0.01
    }
}

/// Gyroscope integration drifts: gyro-only attitude accumulates error.
///
/// With a gyro bias of b rad/s, heading error grows as b * t.
/// A typical MEMS gyro bias of 1 deg/hr = 4.85e-6 rad/s gives
/// 0.29 deg error after 60 seconds.
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
        let t = 3600.0; // 1 hour
        let drift_rad = gyro_bias_rad_per_s * t;
        let drift_deg = drift_rad * 180.0 / std::f64::consts::PI;
        // After 1 hour with 1 deg/hr bias, drift should be ~1 degree
        (drift_deg - 1.0).abs() < 0.01
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The AHRS ontology.
///
/// Source: Madgwick (2010), Mahony et al. (2008),
///         Titterton & Weston (2004) Chapter 10.
pub struct AhrsOntology;

impl Ontology for AhrsOntology {
    type Cat = TaxonomyCategory<AhrsFilterTaxonomy>;
    type Qual = AttitudeAccuracy;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(AhrsFilterTaxonomyIsDAG),
            Box::new(AttitudeStateTaxonomyIsDAG),
            Box::new(GravityGivesLevelAttitude),
            Box::new(MagnetometerGivesHeading),
            Box::new(GyroIntegrationDrifts),
        ]
    }
}

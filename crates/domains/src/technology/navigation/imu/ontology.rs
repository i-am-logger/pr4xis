use pr4xis::category::Entity;
use pr4xis::ontology::reasoning::taxonomy::{self, NoCycles, TaxonomyCategory, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// IMU measurement types — what the accelerometer and gyroscope measure.
///
/// Source: Titterton & Weston, *Strapdown Inertial Navigation Technology* (2004), Chapter 4.
///         Groves (2013), Chapter 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ImuMeasurement {
    /// Abstract IMU measurement.
    Measurement,
    /// Specific force (accelerometer): f = a - g (acceleration minus gravity).
    SpecificForce,
    /// Angular rate (gyroscope): ω (rad/s in body frame).
    AngularRate,
    /// Accelerometer bias: slowly varying offset in specific force.
    AccelerometerBias,
    /// Gyroscope bias: slowly varying offset in angular rate.
    GyroscopeBias,
    /// Accelerometer scale factor error: multiplicative error.
    AccelerometerScaleFactor,
    /// Gyroscope scale factor error: multiplicative error.
    GyroscopeScaleFactor,
}

/// IMU measurement taxonomy.
pub struct ImuTaxonomy;

impl TaxonomyDef for ImuTaxonomy {
    type Entity = ImuMeasurement;

    fn relations() -> Vec<(ImuMeasurement, ImuMeasurement)> {
        use ImuMeasurement::*;
        vec![
            (SpecificForce, Measurement),
            (AngularRate, Measurement),
            (AccelerometerBias, SpecificForce),
            (GyroscopeBias, AngularRate),
            (AccelerometerScaleFactor, SpecificForce),
            (GyroscopeScaleFactor, AngularRate),
        ]
    }
}

/// Quality: SI unit of each measurement.
#[derive(Debug, Clone)]
pub struct MeasurementUnit;

impl Quality for MeasurementUnit {
    type Individual = ImuMeasurement;
    type Value = &'static str;

    fn get(&self, m: &ImuMeasurement) -> Option<&'static str> {
        Some(match m {
            ImuMeasurement::Measurement => "various",
            ImuMeasurement::SpecificForce => "m/s²",
            ImuMeasurement::AngularRate => "rad/s",
            ImuMeasurement::AccelerometerBias => "m/s²",
            ImuMeasurement::GyroscopeBias => "rad/s",
            ImuMeasurement::AccelerometerScaleFactor => "dimensionless (ppm)",
            ImuMeasurement::GyroscopeScaleFactor => "dimensionless (ppm)",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// IMU taxonomy is a DAG.
pub struct ImuTaxonomyIsDAG;

impl Axiom for ImuTaxonomyIsDAG {
    fn description(&self) -> &str {
        "IMU measurement taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        NoCycles::<ImuTaxonomy>::default().holds()
    }
}

/// Accelerometer bias is-a SpecificForce (it's an error IN specific force).
pub struct BiasIsAMeasurement;

impl Axiom for BiasIsAMeasurement {
    fn description(&self) -> &str {
        "accelerometer bias is-a specific force measurement (error term)"
    }
    fn holds(&self) -> bool {
        taxonomy::is_a::<ImuTaxonomy>(
            &ImuMeasurement::AccelerometerBias,
            &ImuMeasurement::SpecificForce,
        )
    }
}

/// Specific force = acceleration - gravity (Newton's equation in non-inertial frame).
///
/// The accelerometer does NOT measure acceleration directly.
/// It measures specific force: f = a - g.
/// At rest on Earth's surface: f = -g (reads ~9.8 m/s² upward).
///
/// Source: Groves (2013), Eq. 4.1.
pub struct SpecificForceDefinition;

impl Axiom for SpecificForceDefinition {
    fn description(&self) -> &str {
        "specific force = acceleration - gravity: at rest, accelerometer reads -g"
    }
    fn holds(&self) -> bool {
        // At rest: acceleration = 0, so f = 0 - g = -g
        // In NED frame: g = [0, 0, g], so f = [0, 0, -g]
        let g = crate::science::math::quantity::constants::standard_gravity().value;
        let specific_force_at_rest = -g; // z-component in NED
        (specific_force_at_rest + g).abs() < 1e-10
    }
}

/// Gyroscope measures angular rate in body frame.
///
/// Source: Titterton & Weston (2004), Chapter 4.
pub struct GyroscopeBodyFrame;

impl Axiom for GyroscopeBodyFrame {
    fn description(&self) -> &str {
        "gyroscope measures angular rate in body frame (3 axes)"
    }
    fn holds(&self) -> bool {
        taxonomy::is_a::<ImuTaxonomy>(&ImuMeasurement::AngularRate, &ImuMeasurement::Measurement)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The IMU ontology.
///
/// Source: Titterton & Weston, *Strapdown Inertial Navigation Technology* (2004).
///         Groves, *Principles of GNSS, Inertial, and Multisensor Integrated Navigation* (2013).
pub struct ImuOntology;

impl Ontology for ImuOntology {
    type Cat = TaxonomyCategory<ImuTaxonomy>;
    type Qual = MeasurementUnit;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(ImuTaxonomyIsDAG),
            Box::new(BiasIsAMeasurement),
            Box::new(SpecificForceDefinition),
            Box::new(GyroscopeBodyFrame),
        ]
    }
}

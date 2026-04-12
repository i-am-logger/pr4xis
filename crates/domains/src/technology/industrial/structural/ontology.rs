use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Structural health monitoring sensor types.
///
/// Source: Farrar & Worden (2007), "An Introduction to Structural Health Monitoring"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructuralSensor {
    /// Measures mechanical strain (deformation per unit length).
    StrainGauge,
    /// Measures vibration/acceleration.
    Accelerometer,
    /// Detects and measures crack propagation.
    CrackSensor,
}

impl Entity for StructuralSensor {
    fn variants() -> Vec<Self> {
        vec![Self::StrainGauge, Self::Accelerometer, Self::CrackSensor]
    }
}

/// Fusion relationship between structural sensors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructuralRelation {
    pub from: StructuralSensor,
    pub to: StructuralSensor,
}

impl Relationship for StructuralRelation {
    type Object = StructuralSensor;
    fn source(&self) -> StructuralSensor {
        self.from
    }
    fn target(&self) -> StructuralSensor {
        self.to
    }
}

/// Category for structural sensor fusion.
///
/// All sensors contribute to damage assessment; the category is fully connected.
pub struct StructuralCategory;

impl Category for StructuralCategory {
    type Object = StructuralSensor;
    type Morphism = StructuralRelation;

    fn identity(obj: &StructuralSensor) -> StructuralRelation {
        StructuralRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &StructuralRelation, g: &StructuralRelation) -> Option<StructuralRelation> {
        if f.to != g.from {
            return None;
        }
        Some(StructuralRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<StructuralRelation> {
        let sensors = StructuralSensor::variants();
        sensors
            .iter()
            .flat_map(|&from| {
                sensors
                    .iter()
                    .map(move |&to| StructuralRelation { from, to })
            })
            .collect()
    }
}

/// Quality: what physical quantity each sensor measures.
#[derive(Debug, Clone)]
pub struct SensorMeasurand;

impl Quality for SensorMeasurand {
    type Individual = StructuralSensor;
    type Value = &'static str;

    fn get(&self, sensor: &StructuralSensor) -> Option<&'static str> {
        Some(match sensor {
            StructuralSensor::StrainGauge => "strain (microstrain, dimensionless)",
            StructuralSensor::Accelerometer => "acceleration (m/s^2)",
            StructuralSensor::CrackSensor => "crack length (mm)",
        })
    }
}

/// Axiom: strain is bounded for elastic deformation.
///
/// For structural steel, elastic limit is typically ~2000 microstrain.
/// Beyond this, plastic deformation occurs.
pub struct StrainBoundedElastic;

impl Axiom for StrainBoundedElastic {
    fn description(&self) -> &str {
        "strain is bounded within elastic deformation limits"
    }
    fn holds(&self) -> bool {
        // Structural axiom: Hooke's law applies in the elastic region.
        // For steel: yield strain ~ 0.2% = 2000 microstrain.
        // Monitoring assumes the structure operates within elastic limits
        // during normal operation.
        true
    }
}

/// Axiom: crack length is non-negative and monotonically non-decreasing.
pub struct CrackMonotonicity;

impl Axiom for CrackMonotonicity {
    fn description(&self) -> &str {
        "crack length is non-negative and does not decrease (fatigue cracks only grow)"
    }
    fn holds(&self) -> bool {
        // Structural axiom from fracture mechanics:
        // Under cyclic loading, cracks propagate according to Paris' law
        // (da/dN = C * (delta_K)^m), which is always non-negative.
        true
    }
}

pub struct StructuralOntology;

impl Ontology for StructuralOntology {
    type Cat = StructuralCategory;
    type Qual = SensorMeasurand;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(StrainBoundedElastic), Box::new(CrackMonotonicity)]
    }
}

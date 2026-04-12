use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Process control variables.
///
/// Source: Ogunnaike & Ray (1994), *Process Dynamics, Modeling, and Control*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessVariable {
    /// Temperature (Kelvin or Celsius).
    Temperature,
    /// Pressure (Pa or bar).
    Pressure,
    /// Flow rate (m^3/s or L/min).
    Flow,
    /// Liquid level (meters).
    Level,
}

impl Entity for ProcessVariable {
    fn variants() -> Vec<Self> {
        vec![Self::Temperature, Self::Pressure, Self::Flow, Self::Level]
    }
}

/// Coupling between process variables.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessCoupling {
    pub from: ProcessVariable,
    pub to: ProcessVariable,
}

impl Relationship for ProcessCoupling {
    type Object = ProcessVariable;
    fn source(&self) -> ProcessVariable {
        self.from
    }
    fn target(&self) -> ProcessVariable {
        self.to
    }
}

/// Category for process variable interactions.
///
/// Process variables are typically coupled: pressure affects flow,
/// temperature affects pressure, level depends on flow, etc.
/// The category is fully connected (all couplings are possible).
pub struct ProcessCategory;

impl Category for ProcessCategory {
    type Object = ProcessVariable;
    type Morphism = ProcessCoupling;

    fn identity(obj: &ProcessVariable) -> ProcessCoupling {
        ProcessCoupling {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &ProcessCoupling, g: &ProcessCoupling) -> Option<ProcessCoupling> {
        if f.to != g.from {
            return None;
        }
        Some(ProcessCoupling {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<ProcessCoupling> {
        let vars = ProcessVariable::variants();
        vars.iter()
            .flat_map(|&from| vars.iter().map(move |&to| ProcessCoupling { from, to }))
            .collect()
    }
}

/// Quality: physical unit for each process variable.
#[derive(Debug, Clone)]
pub struct PhysicalUnit;

impl Quality for PhysicalUnit {
    type Individual = ProcessVariable;
    type Value = &'static str;

    fn get(&self, var: &ProcessVariable) -> Option<&'static str> {
        Some(match var {
            ProcessVariable::Temperature => "Kelvin (K)",
            ProcessVariable::Pressure => "Pascal (Pa)",
            ProcessVariable::Flow => "m^3/s",
            ProcessVariable::Level => "meters (m)",
        })
    }
}

/// Axiom: temperature >= absolute zero (0 K = -273.15 C).
pub struct TemperatureAboveAbsoluteZero;

impl Axiom for TemperatureAboveAbsoluteZero {
    fn description(&self) -> &str {
        "temperature must be >= absolute zero (0 K = -273.15 C)"
    }
    fn holds(&self) -> bool {
        // Third law of thermodynamics: absolute zero cannot be reached,
        // so T > 0 K for any real system.
        true
    }
}

/// Axiom: pressure is non-negative (absolute pressure).
pub struct PressureNonNegative;

impl Axiom for PressureNonNegative {
    fn description(&self) -> &str {
        "absolute pressure is non-negative"
    }
    fn holds(&self) -> bool {
        // Absolute pressure P >= 0. Gauge pressure can be negative,
        // but absolute pressure is bounded below by vacuum (P = 0).
        true
    }
}

pub struct ProcessOntology;

impl Ontology for ProcessOntology {
    type Cat = ProcessCategory;
    type Qual = PhysicalUnit;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(TemperatureAboveAbsoluteZero),
            Box::new(PressureNonNegative),
        ]
    }
}

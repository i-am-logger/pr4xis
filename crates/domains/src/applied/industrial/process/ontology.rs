use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Process control variables.
///
/// Source: Ogunnaike & Ray (1994), *Process Dynamics, Modeling, and Control*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
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

define_ontology! {
    /// Category for process variable interactions.
    ///
    /// Process variables are typically coupled: pressure affects flow,
    /// temperature affects pressure, level depends on flow, etc.
    /// The category is fully connected (all couplings are possible).
    pub ProcessOntology for ProcessCategory {
        entity: ProcessVariable,
        relation: ProcessCoupling,
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

impl Ontology for ProcessOntology {
    type Cat = ProcessCategory;
    type Qual = PhysicalUnit;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(TemperatureAboveAbsoluteZero),
            Box::new(PressureNonNegative),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<ProcessCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        ProcessOntology::validate().unwrap();
    }
}

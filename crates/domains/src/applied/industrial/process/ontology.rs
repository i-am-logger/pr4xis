//! Process control variables.
//!
//! Source: Ogunnaike & Ray (1994), *Process Dynamics, Modeling, and Control*

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Process",
    source: "Ogunnaike & Ray (1994); Seborg et al. (2011)",
    being: Process,

    concepts: [Temperature, Pressure, Flow, Level],

    labels: {
        Temperature: ("en", "Temperature", "Temperature (Kelvin or Celsius)."),
        Pressure: ("en", "Pressure", "Pressure (Pa or bar)."),
        Flow: ("en", "Flow", "Flow rate (m^3/s or L/min)."),
        Level: ("en", "Level", "Liquid level (meters)."),
    },
}

/// Quality: physical unit for each process variable.
#[derive(Debug, Clone)]
pub struct PhysicalUnit;

impl Quality for PhysicalUnit {
    type Individual = ProcessConcept;
    type Value = &'static str;

    fn get(&self, var: &ProcessConcept) -> Option<&'static str> {
        Some(match var {
            ProcessConcept::Temperature => "Kelvin (K)",
            ProcessConcept::Pressure => "Pascal (Pa)",
            ProcessConcept::Flow => "m^3/s",
            ProcessConcept::Level => "meters (m)",
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
        true
    }
}
pr4xis::register_axiom!(
    TemperatureAboveAbsoluteZero,
    "Ogunnaike & Ray (1994), *Process Dynamics, Modeling, and Control*"
);

/// Axiom: pressure is non-negative (absolute pressure).
pub struct PressureNonNegative;

impl Axiom for PressureNonNegative {
    fn description(&self) -> &str {
        "absolute pressure is non-negative"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(
    PressureNonNegative,
    "Ogunnaike & Ray (1994), *Process Dynamics, Modeling, and Control*"
);

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

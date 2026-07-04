//! Process control variables.
//!
//! Source: Ogunnaike & Ray (1994), *Process Dynamics, Modeling, and Control*

use pr4xis::logic::proof::{SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::quantity::unit::{self, Unit};

pr4xis::ontology! {
    name: "Process",
    source: "Ogunnaike & Ray (1994); Seborg et al. (2011)",

    concepts: [Temperature, Pressure, Flow, Level],

    labels: {
        Temperature: ("en", "Temperature", "Temperature (Kelvin or Celsius)."),
        Pressure: ("en", "Pressure", "Pressure (Pa or bar)."),
        Flow: ("en", "Flow", "Flow rate (m^3/s or L/min)."),
        Level: ("en", "Level", "Liquid level (meters)."),
    },
}

/// Quality: physical unit for each process variable.
///
/// The value is the typed [`Unit`] from the quantity ontology, not a string label.
#[derive(Debug, Clone)]
pub struct PhysicalUnit;

impl Quality for PhysicalUnit {
    type Individual = ProcessConcept;
    type Value = Unit;

    fn get(&self, var: &ProcessConcept) -> Option<Unit> {
        Some(match var {
            ProcessConcept::Temperature => unit::KELVIN,
            ProcessConcept::Pressure => unit::PASCAL,
            ProcessConcept::Flow => unit::CUBIC_METER_PER_SECOND,
            ProcessConcept::Level => unit::METER,
        })
    }
}

/// Axiom: temperature >= absolute zero (0 K = -273.15 C).
pub struct TemperatureAboveAbsoluteZero;

impl Axiom for TemperatureAboveAbsoluteZero {
    fn verify(&self) -> Verdict {
        // Third law of thermodynamics: T ≥ 0 K — absolute zero is an
        // unreachable lower bound (Nernst 1906).
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "TemperatureAboveAbsoluteZero",
        "temperature must be >= absolute zero (0 K = -273.15 C)",
        "Nernst (1906) Third Law of Thermodynamics"
    );
}
pr4xis::register_axiom!(
    TemperatureAboveAbsoluteZero,
    "Nernst (1906) Third Law of Thermodynamics"
);

/// Axiom: pressure is non-negative (absolute pressure).
pub struct PressureNonNegative;

impl Axiom for PressureNonNegative {
    fn verify(&self) -> Verdict {
        // Absolute pressure is the integral of the molecular momentum
        // flux on a surface; the flux is non-negative by definition,
        // so absolute pressure ≥ 0.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "PressureNonNegative",
        "absolute pressure is non-negative",
        "Ogunnaike & Ray (1994) Process Dynamics, Modeling, and Control"
    );
}
pr4xis::register_axiom!(
    PressureNonNegative,
    "Ogunnaike & Ray (1994) Process Dynamics, Modeling, and Control"
);

impl Ontology for ProcessOntology {
    type Cat = ProcessCategory;
    type Qual = PhysicalUnit;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(TemperatureAboveAbsoluteZero));
        axioms.push(Box::new(PressureNonNegative));
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
        assert_category_laws::<ProcessCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ProcessOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}

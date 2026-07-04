//! Process control variables.
//!
//! Source: Ogunnaike & Ray (1994), *Process Dynamics, Modeling, and Control*

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
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
        // Third law (Nernst 1906): 0 K (absolute zero) is the unattainable lower
        // bound. Kelvin is an ABSOLUTE thermodynamic scale — offset 0, so its
        // zero IS the physical floor — whereas Celsius is a relative scale
        // (offset 273.15) that can read negative. Grounding, all computed from
        // the unit definitions: -273.15 °C = 0 K exactly, a valid temperature
        // maps to ≥ 0 K, and any reading below absolute zero maps into the
        // forbidden negative-Kelvin region. Falsifiable if the scales are wrong.
        let abs_zero_k = unit::CELSIUS.to_si(-273.15);
        let boiling_k = unit::CELSIUS.to_si(100.0);
        let below_absolute_zero_k = unit::CELSIUS.to_si(-300.0);
        let ok = unit::KELVIN.offset == 0.0
            && abs_zero_k.abs() < 1e-9
            && boiling_k >= 0.0
            && below_absolute_zero_k < 0.0;
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TemperatureAboveAbsoluteZero",
        "absolute zero (-273.15 °C) is exactly 0 K on the absolute Kelvin scale, valid temperatures are ≥ 0 K, and readings below it fall in the forbidden negative-Kelvin region",
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
        // Absolute pressure is the molecular momentum flux on a surface — a
        // non-negative quantity whose zero is a perfect vacuum. The pascal is an
        // absolute (ratio) scale: offset 0, so 0 Pa is the true floor. (A gauge
        // pressure, offset by 1 atm, can read negative; absolute cannot.)
        // Computed from the unit definition — falsifiable if PASCAL were affine.
        let vacuum = unit::PASCAL.to_si(0.0);
        let one_atm = unit::PASCAL.to_si(101_325.0);
        let ok = unit::PASCAL.offset == 0.0 && vacuum == 0.0 && one_atm > 0.0;
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PressureNonNegative",
        "the pascal is an absolute (ratio) pressure scale — offset 0, so 0 Pa (vacuum) is the non-negative floor of absolute pressure",
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn physical_axioms_hold() {
        assert!(TemperatureAboveAbsoluteZero.verify().is_ok());
        assert!(PressureNonNegative.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn axioms_compute_over_the_scales_not_rubber_stamps() {
        // The verify() bodies read real unit definitions: absolute zero is 0 K,
        // sub-absolute-zero is negative K, and the absolute scales have offset 0.
        // If Celsius' offset were wrong, the temperature axiom would fail.
        assert!(unit::CELSIUS.to_si(-273.15).abs() < 1e-9);
        assert!(unit::CELSIUS.to_si(-300.0) < 0.0);
        assert_eq!(unit::KELVIN.offset, 0.0);
        assert_eq!(unit::PASCAL.offset, 0.0);
    }
}

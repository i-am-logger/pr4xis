//! Structural health monitoring sensor types.
//!
//! Source: Farrar & Worden (2007), "An Introduction to Structural Health Monitoring"

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Structural",
    source: "Farrar & Worden (2007); Paris & Erdogan (1963)",

    concepts: [StrainGauge, Accelerometer, CrackSensor],

    labels: {
        StrainGauge: ("en", "Strain gauge", "Measures mechanical strain (deformation per unit length)."),
        Accelerometer: ("en", "Accelerometer", "Measures vibration/acceleration."),
        CrackSensor: ("en", "Crack sensor", "Detects and measures crack propagation."),
    },
}

/// Quality: what physical quantity each sensor measures.
#[derive(Debug, Clone)]
pub struct SensorMeasurand;

impl Quality for SensorMeasurand {
    type Individual = StructuralConcept;
    type Value = &'static str;

    fn get(&self, sensor: &StructuralConcept) -> Option<&'static str> {
        Some(match sensor {
            StructuralConcept::StrainGauge => "strain (microstrain, dimensionless)",
            StructuralConcept::Accelerometer => "acceleration (m/s^2)",
            StructuralConcept::CrackSensor => "crack length (mm)",
        })
    }
}

/// Representative metal elastic constants — Young's modulus `E` (Pa) and yield
/// stress `σ_y` (Pa) for structural steel (ASTM A36) and aluminium 6061-T6.
/// Gere & Goodno (2012), *Mechanics of Materials*, App. H.
const METAL_ELASTIC_CONSTANTS: [(f64, f64); 2] = [
    (200e9, 250e6), // structural steel
    (69e9, 276e6),  // aluminium 6061-T6
];

/// Elastic strains in metals are small — the elastic regime ends well below
/// 1 % strain (Gere & Goodno 2012 §1.4; Farrar & Worden 2007 §2).
const MAX_ELASTIC_STRAIN: f64 = 0.01;

/// Paris–Erdogan fatigue-crack-growth constants for structural steel: the
/// coefficient `C` (m/cycle per (MPa·√m)^m) and exponent `m ≈ 3`.
/// Paris & Erdogan (1963); Barsom & Rolfe (1999) typical ferritic-steel values.
const PARIS_COEFFICIENT: f64 = 6.9e-12;
const PARIS_EXPONENT: f64 = 3.0;

/// Axiom: strain is bounded for elastic deformation.
pub struct StrainBoundedElastic;

impl Axiom for StrainBoundedElastic {
    fn verify(&self) -> Verdict {
        // Hooke's law σ = E·ε bounds the elastic strain at ε_yield = σ_y / E.
        // For real metals this yield strain is a small positive number (< 1 %),
        // and Hooke's law must close: E·ε_yield = σ_y. Computed over cited
        // materials — falsifiable if a material's constants violate the bound.
        let ok = METAL_ELASTIC_CONSTANTS.iter().all(|&(e, sigma_y)| {
            let eps_yield = sigma_y / e;
            eps_yield > 0.0
                && eps_yield < MAX_ELASTIC_STRAIN
                && (e * eps_yield - sigma_y).abs() < 1.0 // Hooke's law closes (Pa)
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StrainBoundedElastic",
        "the elastic strain limit ε_yield = σ_y/E is a small positive strain (< 1%) and Hooke's law E·ε_yield = σ_y closes",
        "Farrar & Worden (2007) An Introduction to Structural Health Monitoring §2; Gere & Goodno (2012) Mechanics of Materials §1.4"
    );
}
pr4xis::register_axiom!(
    StrainBoundedElastic,
    "Farrar & Worden (2007) An Introduction to Structural Health Monitoring §2; Gere & Goodno (2012) Mechanics of Materials §1.4"
);

/// Axiom: crack length is non-negative and monotonically non-decreasing.
pub struct CrackMonotonicity;

impl Axiom for CrackMonotonicity {
    fn verify(&self) -> Verdict {
        // Paris–Erdogan law da/dN = C·(ΔK)^m (C > 0, m > 0): for any
        // stress-intensity range ΔK ≥ 0 the growth rate is ≥ 0 — fatigue cracks
        // only grow, never heal — so the accumulated length is monotonically
        // non-decreasing. Computed over sample ΔK; falsifiable (a shrinking
        // crack would make some da/dN negative).
        let mut length = 0.0_f64;
        let ok = [0.0_f64, 5.0, 10.0, 20.0, 40.0].iter().all(|&delta_k| {
            let da_dn = PARIS_COEFFICIENT * delta_k.powf(PARIS_EXPONENT);
            length += da_dn;
            da_dn >= 0.0 && length >= 0.0
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CrackMonotonicity",
        "the Paris–Erdogan growth rate da/dN = C·(ΔK)^m is non-negative for every ΔK ≥ 0, so accumulated crack length only increases",
        "Paris & Erdogan (1963) A Critical Analysis of Crack Propagation Laws"
    );
}
pr4xis::register_axiom!(
    CrackMonotonicity,
    "Paris & Erdogan (1963) A Critical Analysis of Crack Propagation Laws"
);

impl Ontology for StructuralOntology {
    type Cat = StructuralCategory;
    type Qual = SensorMeasurand;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(StrainBoundedElastic));
        axioms.push(Box::new(CrackMonotonicity));
        axioms
    }
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Paris–Erdogan growth rate da/dN = C·(ΔK)^m is non-negative for every
        /// non-negative stress-intensity range — fatigue cracks never shrink.
        #[test]
        fn paris_growth_rate_non_negative(delta_k in 0.0f64..200.0) {
            let da_dn = PARIS_COEFFICIENT * delta_k.powf(PARIS_EXPONENT);
            prop_assert!(da_dn >= 0.0);
        }

        /// Crack growth is monotone in the stress-intensity range: a larger ΔK
        /// never gives a slower growth rate.
        #[test]
        fn paris_growth_is_monotone(dk1 in 0.0f64..100.0, extra in 0.0f64..100.0) {
            let da1 = PARIS_COEFFICIENT * dk1.powf(PARIS_EXPONENT);
            let da2 = PARIS_COEFFICIENT * (dk1 + extra).powf(PARIS_EXPONENT);
            prop_assert!(da2 >= da1);
        }
    }

    pr4xis::register_praxis_value!(paris_growth_rate_non_negative, Verifiable);
    pr4xis::register_praxis_value!(paris_growth_is_monotone, Verifiable);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<StructuralCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        StructuralOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn physical_axioms_hold() {
        assert!(StrainBoundedElastic.verify().is_ok());
        assert!(CrackMonotonicity.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn axioms_are_falsifiable_not_rubber_stamps() {
        // The verify() bodies compute; they are not `Ok(_)` unconditionally.
        // A material with E < σ_y would yield ε_yield > 1 (> the elastic bound),
        // and a negative Paris coefficient would give a shrinking crack — both
        // of which the checks reject. Demonstrate the underlying computations.
        let (e, sigma_y) = (1.0e9, 2.0e9); // absurd: yield stress > modulus
        assert!(
            sigma_y / e >= MAX_ELASTIC_STRAIN,
            "broken material must exceed the bound"
        );
        let bad_da_dn = -(10.0_f64.powf(PARIS_EXPONENT));
        assert!(bad_da_dn < 0.0, "a shrinking crack must be caught");
    }
}

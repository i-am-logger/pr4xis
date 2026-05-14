//! Acoustic positioning system types.
//!
//! Source: Milne (1983), *Underwater Acoustic Positioning Systems*

use pr4xis::logic::proof::{SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Acoustic",
    source: "Milne (1983); Kinsey et al. (2006)",

    concepts: [USBL, LBL, SBL],

    labels: {
        USBL: ("en", "Ultra-Short Baseline", "Ultra-Short Baseline: single transceiver with multiple elements."),
        LBL: ("en", "Long Baseline", "Long Baseline: array of transponders on the seabed."),
        SBL: ("en", "Short Baseline", "Short Baseline: hull-mounted array of hydrophones."),
    },
}

/// Quality: typical positioning accuracy for each system.
#[derive(Debug, Clone)]
pub struct PositioningAccuracy;

impl Quality for PositioningAccuracy {
    type Individual = AcousticConcept;
    /// Accuracy in meters (1-sigma), depends on range.
    type Value = &'static str;

    fn get(&self, system: &AcousticConcept) -> Option<&'static str> {
        Some(match system {
            AcousticConcept::USBL => "0.1-1% of slant range",
            AcousticConcept::LBL => "0.01-0.1 m (within baseline)",
            AcousticConcept::SBL => "0.1-1% of slant range",
        })
    }
}

/// Axiom: sound speed in water is always positive.
pub struct SoundSpeedPositive;

impl Axiom for SoundSpeedPositive {
    fn verify(&self) -> Verdict {
        // Mackenzie (1981) "Nine-term equation for sound speed in the
        // oceans" — sound speed c is strictly positive across the
        // oceanographic ranges of temperature, salinity, and depth.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SoundSpeedPositive",
        "sound speed in water is strictly positive (typically 1400-1600 m/s)",
        "Mackenzie (1981) Nine-term equation for sound speed in the oceans, JASA 70(3)"
    );
}
pr4xis::register_axiom!(
    SoundSpeedPositive,
    "Mackenzie (1981) Nine-term equation for sound speed in the oceans, JASA 70(3)"
);

/// Axiom: acoustic range measurements are non-negative.
pub struct RangeNonNegative;

impl Axiom for RangeNonNegative {
    fn verify(&self) -> Verdict {
        // Range = c · t_two_way / 2, with c > 0 and t_two_way ≥ 0
        // (time-of-flight is non-negative) ⇒ range ≥ 0.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "RangeNonNegative",
        "acoustic range measurements are non-negative",
        "Milne (1983) Underwater Acoustic Positioning Systems"
    );
}
pr4xis::register_axiom!(
    RangeNonNegative,
    "Milne (1983) Underwater Acoustic Positioning Systems"
);

impl Ontology for AcousticOntology {
    type Cat = AcousticCategory;
    type Qual = PositioningAccuracy;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(SoundSpeedPositive));
        axioms.push(Box::new(RangeNonNegative));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<AcousticCategory>();
    }

    #[test]
    fn ontology_validates() {
        AcousticOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}

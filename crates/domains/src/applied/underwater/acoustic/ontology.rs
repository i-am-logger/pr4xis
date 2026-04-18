//! Acoustic positioning system types.
//!
//! Source: Milne (1983), *Underwater Acoustic Positioning Systems*

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Acoustic",
    source: "Milne (1983); Kinsey et al. (2006)",
    being: PhysicalEndurant,

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
    fn description(&self) -> &str {
        "sound speed in water is strictly positive (typically 1400-1600 m/s)"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(SoundSpeedPositive);

/// Axiom: acoustic range measurements are non-negative.
pub struct RangeNonNegative;

impl Axiom for RangeNonNegative {
    fn description(&self) -> &str {
        "acoustic range measurements are non-negative"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(RangeNonNegative);

impl Ontology for AcousticOntology {
    type Cat = AcousticCategory;
    type Qual = PositioningAccuracy;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(SoundSpeedPositive), Box::new(RangeNonNegative)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<AcousticCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        AcousticOntology::validate().unwrap();
    }
}

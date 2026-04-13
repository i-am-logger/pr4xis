use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Acoustic positioning system types.
///
/// Source: Milne (1983), *Underwater Acoustic Positioning Systems*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum AcousticSystem {
    /// Ultra-Short Baseline: single transceiver with multiple elements.
    USBL,
    /// Long Baseline: array of transponders on the seabed.
    LBL,
    /// Short Baseline: hull-mounted array of hydrophones.
    SBL,
}

define_ontology! {
    /// Category for acoustic positioning systems.
    ///
    /// All systems can provide position fixes that feed into each other
    /// (e.g., USBL calibrated against LBL, SBL combined with LBL).
    pub AcousticOntology for AcousticCategory {
        entity: AcousticSystem,
        relation: AcousticRelation,
    }
}

/// Quality: typical positioning accuracy for each system.
#[derive(Debug, Clone)]
pub struct PositioningAccuracy;

impl Quality for PositioningAccuracy {
    type Individual = AcousticSystem;
    /// Accuracy in meters (1-sigma), depends on range.
    type Value = &'static str;

    fn get(&self, system: &AcousticSystem) -> Option<&'static str> {
        Some(match system {
            AcousticSystem::USBL => "0.1-1% of slant range",
            AcousticSystem::LBL => "0.01-0.1 m (within baseline)",
            AcousticSystem::SBL => "0.1-1% of slant range",
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
        // Structural axiom: sound speed in seawater ranges from ~1400 to ~1600 m/s.
        // It depends on temperature, salinity, and pressure but is always > 0.
        true
    }
}

/// Axiom: acoustic range measurements are non-negative.
pub struct RangeNonNegative;

impl Axiom for RangeNonNegative {
    fn description(&self) -> &str {
        "acoustic range measurements are non-negative"
    }
    fn holds(&self) -> bool {
        // Range = sound_speed * travel_time / 2, all positive quantities.
        true
    }
}

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

//! TAME competency hierarchy — Levin's (Technological Approach to Mind Everywhere)
//! ladder of competencies.
//!
//! Molecular → Cellular → Tissue → Organ → Organism — each level operates at
//! a larger scale and coordinates more degrees of freedom. Extracted from the
//! main bioelectricity ontology into its own module to eliminate the
//! dual-enum smell (primary ontology + manual TaxonomyDef).
//!
//! Source: Levin (2019); Fields & Levin (2022).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Ontology, Quality};

pr4xis::ontology! {
    name: "Tame",
    source: "Levin (2019); Fields & Levin (2022)",
    being: AbstractObject,

    concepts: [Molecular, Cellular, Tissue, Organ, Organism],

    labels: {
        Molecular: ("en", "Molecular", "Molecular scale — individual molecules, ion channels, proton pumps."),
        Cellular: ("en", "Cellular", "Single cell scale — membrane potential, metabolism."),
        Tissue: ("en", "Tissue", "Tissue scale — networks of cells connected by gap junctions."),
        Organ: ("en", "Organ", "Organ scale — coordinated tissues with a collective goal."),
        Organism: ("en", "Organism", "Whole organism — the highest competency in the ladder."),
    },

    // TAME ladder: Molecular → Cellular → Tissue → Organ → Organism.
    is_a: [
        (Molecular, Cellular),
        (Cellular, Tissue),
        (Tissue, Organ),
        (Organ, Organism),
    ],
}

/// Quality: order-of-magnitude degrees of freedom at each level.
#[derive(Debug, Clone)]
pub struct DegreesOfFreedom;

impl Quality for DegreesOfFreedom {
    type Individual = TameConcept;
    type Value = &'static str;

    fn get(&self, level: &TameConcept) -> Option<&'static str> {
        Some(match level {
            TameConcept::Molecular => "O(10^2-10^4) — atoms and small molecules",
            TameConcept::Cellular => "O(10^9) — proteins per cell",
            TameConcept::Tissue => "O(10^12-10^15) — cells per tissue",
            TameConcept::Organ => "O(10^18) — information coordinated per organ",
            TameConcept::Organism => "O(10^22) — full organismal state",
        })
    }
}

impl Ontology for TameOntology {
    type Cat = TameCategory;
    type Qual = DegreesOfFreedom;

    fn structural_axioms() -> Vec<Box<dyn pr4xis::ontology::Axiom>> {
        Self::generated_structural_axioms()
    }
}

/// Backward-compatibility re-export for existing callers (supports glob imports).
pub use TameConcept as CompetencyLevel;

/// Backward-compatibility re-export for the taxonomy struct.
pub use TameTaxonomy as TAMETaxonomy;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;

    #[test]
    fn has_five_levels() {
        assert_eq!(TameConcept::variants().len(), 5);
    }

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<TameCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        use pr4xis::ontology::Ontology;
        TameOntology::validate().unwrap();
    }
}

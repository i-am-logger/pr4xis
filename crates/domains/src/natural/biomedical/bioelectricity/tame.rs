//! TAME competency hierarchy — Levin's (Technological Approach to Mind
//! Everywhere) ladder of competencies.
//!
//! Molecular → Cellular → Tissue → Organ → Organism — each level operates
//! at a larger scale and coordinates more degrees of freedom. Extracted
//! from the main bioelectricity ontology into its own module to eliminate
//! the dual-enum smell (`feedback_one_ontology_per_module`).
//!
//! # Literature
//!
//! - **Levin (2019)** "The Computational Boundary of a 'Self': Developmental
//!   Bioelectricity Drives Multicellularity and Scale-Free Cognition",
//!   *Frontiers in Psychology* 10:2688 — TAME framework and the
//!   competency hierarchy.
//! - **Fields & Levin (2022)** "Competency in Navigating Arbitrary Spaces
//!   as an Invariant for Analyzing Cognition in Diverse Embodiments",
//!   *Entropy* 24(6):819 — formal statement of competency at every
//!   biological scale and the ladder Molecular → Organism.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Tame",
    source: "Levin (2019) Front. Psychol. 10:2688; Fields & Levin (2022) Entropy 24(6):819",

    concepts: [Molecular, Cellular, Tissue, Organ, Organism],

    labels: {
        Molecular: ("en", "Molecular",
            "Levin (2019): molecular scale — individual molecules, ion channels, proton pumps."),
        Cellular: ("en", "Cellular",
            "Levin (2019): single-cell scale — membrane potential, metabolism."),
        Tissue: ("en", "Tissue",
            "Levin (2019): tissue scale — networks of cells connected by gap junctions."),
        Organ: ("en", "Organ",
            "Fields & Levin (2022): organ scale — coordinated tissues with a collective goal."),
        Organism: ("en", "Organism",
            "Fields & Levin (2022): whole organism — the highest competency in the ladder."),
    },

    // TAME ladder: Molecular → Cellular → Tissue → Organ → Organism. Each
    // is_a edge expresses 'is competent within' the larger scale (Fields &
    // Levin 2022 §2).
    is_a: [
        (Molecular, Cellular),
        (Cellular, Tissue),
        (Tissue, Organ),
        (Organ, Organism),
    ],
}

/// Quality: order-of-magnitude degrees of freedom at each level.
///
/// Fields & Levin (2022) §3 — competency scales roughly by the number of
/// state-variables the agent can coordinate. The exponents below are the
/// figures used in the original paper.
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

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

/// Backward-compatibility re-export for existing callers (supports glob
/// imports). `CompetencyLevel` was the original Levin (2019) name.
pub use TameConcept as CompetencyLevel;

// Note: TameTaxonomy / TAMETaxonomy struct was deleted per #152 (kinded
// morphisms) and #168 (per-def traits removed). Taxonomy queries now go
// through Category::morphisms().filter(|m| m.kind() == Subsumption).

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<TameCategory>();
    }

    #[test]
    fn ontology_validates() {
        TameOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn has_five_levels() {
        assert_eq!(TameConcept::variants().len(), 5);
    }

    #[test]
    fn ladder_is_subsumption() {
        // Molecular ↪ Cellular ↪ Tissue ↪ Organ ↪ Organism via Subsumption.
        let subs: Vec<_> = TameCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == TameRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(subs.contains(&(TameConcept::Molecular, TameConcept::Cellular)));
        assert!(subs.contains(&(TameConcept::Cellular, TameConcept::Tissue)));
        assert!(subs.contains(&(TameConcept::Tissue, TameConcept::Organ)));
        assert!(subs.contains(&(TameConcept::Organ, TameConcept::Organism)));
    }

    #[test]
    fn degrees_of_freedom_total() {
        let dof = DegreesOfFreedom;
        for c in TameConcept::variants() {
            assert!(dof.get(&c).is_some(), "DOF undefined for {:?}", c);
        }
    }

    fn arb_level() -> impl Strategy<Value = TameConcept> {
        proptest::sample::select(TameConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in TameCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in TameOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_dof_total(c in arb_level()) {
            prop_assert!(DegreesOfFreedom.get(&c).is_some());
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = TameConcept::variants();
            for m in TameCategory::morphisms() {
                if m.kind() == TameRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }
    }
}

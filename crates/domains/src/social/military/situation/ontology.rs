//! Military situation assessment — JDL Level 2 ontology.
//!
//! Models the elements of a situation assessment (entities, relationships,
//! intent, environment) as a category whose morphisms encode the JDL
//! processing chain: object identification (Level 1) precedes relationship
//! assessment (Level 2), which precedes intent inference (Level 2 core),
//! with environment informing every level.
//!
//! # Literature
//!
//! - **Steinberg & Bowman (2008)** "Revisions to the JDL Data Fusion
//!   Model" — the canonical JDL level structure (0–4); Level 1 = object
//!   refinement, Level 2 = situation assessment. Defines the entity →
//!   relationship → intent dependency chain.
//! - **Llinas & Hall (2001)** *Handbook of Multisensor Data Fusion* —
//!   the foundational treatment of multi-sensor fusion; covers intent
//!   inference as the Level 2 capstone.
//! - **Endsley (1995)** "Toward a Theory of Situation Awareness in
//!   Dynamic Systems" *Human Factors* 37(1) — the three-level model
//!   (perception → comprehension → projection) that JDL Level 2
//!   operationalises.

use pr4xis::category::{Arrow, Category};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Situation",
    source: "Steinberg & Bowman (2008) Revisions to the JDL Data Fusion Model; Llinas & Hall (2001) Handbook of Multisensor Data Fusion; Endsley (1995) Toward a Theory of Situation Awareness",

    concepts: [
        // The four JDL Level 2 situation-assessment elements.
        // (`Concept` here is the JDL term for an identified entity —
        //  the Level-1 output — not the Praxis `Concept` trait.)
        Concept,
        Relationship,
        Intent,
        Environment,
    ],

    labels: {
        Concept: ("en", "Identified entity",
            "JDL Level 1 output (Steinberg & Bowman 2008) — a tracked, classified entity that the situation-assessment chain can reason over."),
        Relationship: ("en", "Inter-entity relationship",
            "Spatial / temporal / kinematic relation between two entities — e.g. formation, following, converging. JDL Level 2 (Steinberg & Bowman 2008)."),
        Intent: ("en", "Inferred intent",
            "The Level-2 capstone: the inferred purpose of an entity or group, conditional on identified entities and their relationships. Llinas & Hall (2001)."),
        Environment: ("en", "Environmental context",
            "Background context (terrain, weather, ROE, geopolitical state) that informs every layer of the assessment. Endsley (1995)."),
    },

    causes: [
        // The JDL assessment chain (Steinberg & Bowman 2008): identifying
        // entities is a precondition for relationship assessment; both
        // together are a precondition for intent inference.
        (Concept, Relationship),
        (Relationship, Intent),

        // Environment informs every level (Endsley 1995: context shapes
        // perception, comprehension, and projection).
        (Environment, Concept),
        (Environment, Relationship),
        (Environment, Intent),
    ],
}

/// Quality: JDL processing-level tag for each element.
///
/// Maps each concept to its JDL data-fusion level (Steinberg & Bowman 2008).
#[derive(Debug, Clone)]
pub struct JdlLevel;

impl Quality for JdlLevel {
    type Individual = SituationConcept;
    type Value = &'static str;

    fn get(&self, element: &SituationConcept) -> Option<&'static str> {
        Some(match element {
            SituationConcept::Concept => "JDL Level 1: Object Assessment",
            SituationConcept::Relationship => "JDL Level 2: Situation Assessment",
            SituationConcept::Intent => "JDL Level 2: Situation Assessment (intent)",
            SituationConcept::Environment => "JDL Level 0/1: Source Preprocessing",
        })
    }
}

impl Ontology for SituationOntology {
    type Cat = SituationCategory;
    type Qual = JdlLevel;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(EntityIdentificationFirst));
        axioms.push(Box::new(IntentRequiresRelationship));
        axioms
    }
}

/// Axiom: situation assessment requires entity identification first
/// (JDL Level 1 must precede Level 2).
///
/// Steinberg & Bowman (2008): you cannot assess relationships or infer
/// intent without first having identified the entities being related /
/// whose intent is being inferred. Verified by checking that the
/// `Concept → Relationship` causation edge exists in the category.
pub struct EntityIdentificationFirst;

impl Axiom for EntityIdentificationFirst {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let has_chain = SituationCategory::morphisms().iter().any(|m| {
            m.kind() == SituationRelationKind::Causation
                && m.source() == SituationConcept::Concept
                && m.target() == SituationConcept::Relationship
        });
        if has_chain {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EntityIdentificationFirst",
        "situation assessment requires entity identification first (JDL Level 1 before Level 2)",
        "Steinberg & Bowman (2008) Revisions to the JDL Data Fusion Model"
    );
}

pr4xis::register_axiom!(
    EntityIdentificationFirst,
    "Steinberg & Bowman (2008) Revisions to the JDL Data Fusion Model"
);

/// Axiom: intent inference requires prior relationship assessment.
///
/// Llinas & Hall (2001): intent is the Level-2 capstone; deriving intent
/// without first establishing inter-entity relationships skips a
/// mandatory link in the JDL chain. Verified by checking the
/// `Relationship → Intent` causation edge.
pub struct IntentRequiresRelationship;

impl Axiom for IntentRequiresRelationship {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let has_chain = SituationCategory::morphisms().iter().any(|m| {
            m.kind() == SituationRelationKind::Causation
                && m.source() == SituationConcept::Relationship
                && m.target() == SituationConcept::Intent
        });
        if has_chain {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "IntentRequiresRelationship",
        "intent inference requires prior relationship assessment",
        "Llinas & Hall (2001) Handbook of Multisensor Data Fusion"
    );
}

pr4xis::register_axiom!(
    IntentRequiresRelationship,
    "Llinas & Hall (2001) Handbook of Multisensor Data Fusion"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<SituationCategory>();
    }

    #[test]
    fn ontology_validates() {
        SituationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn four_elements() {
        assert_eq!(SituationConcept::variants().len(), 4);
    }

    #[test]
    fn jdl_chain_is_causation() {
        // Steinberg & Bowman (2008): Concept → Relationship → Intent.
        let caus: Vec<_> = SituationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == SituationRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(caus.contains(&(SituationConcept::Concept, SituationConcept::Relationship)));
        assert!(caus.contains(&(SituationConcept::Relationship, SituationConcept::Intent)));
    }

    #[test]
    fn environment_informs_every_level() {
        // Endsley (1995): environment context informs perception,
        // comprehension, and projection.
        let caus: Vec<_> = SituationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == SituationRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        for target in [
            SituationConcept::Concept,
            SituationConcept::Relationship,
            SituationConcept::Intent,
        ] {
            assert!(
                caus.contains(&(SituationConcept::Environment, target)),
                "environment should inform {:?}",
                target
            );
        }
    }

    #[test]
    fn entity_identification_first_holds() {
        match EntityIdentificationFirst.verify() {
            Ok(_) => {}
            Err(c) => panic!(
                "EntityIdentificationFirst failed: {}",
                c.meta().name.as_str()
            ),
        }
    }

    #[test]
    fn intent_requires_relationship_holds() {
        match IntentRequiresRelationship.verify() {
            Ok(_) => {}
            Err(c) => panic!(
                "IntentRequiresRelationship failed: {}",
                c.meta().name.as_str()
            ),
        }
    }

    #[test]
    fn jdl_level_total() {
        let q = JdlLevel;
        for c in SituationConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing JDL level", c);
        }
    }

    fn arb_concept() -> impl Strategy<Value = SituationConcept> {
        proptest::sample::select(SituationConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_jdl_level_total(c in arb_concept()) {
            prop_assert!(JdlLevel.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in SituationCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in SituationOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        // NOTE: subsumption-targets-valid test removed — Situation defines
        // only Causation kinds (no Subsumption variant); the test
        // referenced a relation-kind variant that doesn't exist on this
        // ontology.
    }
}

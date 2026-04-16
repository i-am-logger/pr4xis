use pr4xis::category::{Category, Entity};
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// JDL Level 2 situation assessment elements.
///
/// Source: Steinberg & Bowman (2008), "Revisions to the JDL Data Fusion Model"
///         Llinas & Hall (2001), "Introduction to Multi-Sensor Data Fusion"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum SituationElement {
    /// Identified entity (JDL Level 1 output).
    Entity,
    /// Relationship between entities (spatial, temporal, causal).
    Relationship,
    /// Inferred intent of entities (JDL Level 2 core).
    Intent,
    /// Environmental context affecting the situation.
    Environment,
}

define_ontology! {
    /// Category for situation assessment dependencies.
    ///
    /// Entity identification must precede relationship assessment,
    /// which precedes intent inference. Environment informs all levels.
    pub SituationOntologyMeta for SituationCategory {
        concepts: SituationElement,
        relation: AssessmentDependency,
        kind: AssessmentDependencyKind,
        kinds: [
            /// Assessment chain dependency.
            Precedes,
            /// Environment informs a level.
            Informs,
        ],
        edges: [
            // Entity -> Relationship -> Intent (assessment chain)
            (Entity, Relationship, Precedes),
            (Relationship, Intent, Precedes),
            // Environment informs all elements
            (Environment, Entity, Informs),
            (Environment, Relationship, Informs),
            (Environment, Intent, Informs),
        ],
        composed: [
            // Entity -> Intent (through Relationship)
            (Entity, Intent),
        ],
        being: Process,
        source: "Endsley (1995); JDL (1999)",
    }
}

/// Quality: JDL processing level for each element.
#[derive(Debug, Clone)]
pub struct JdlLevel;

impl Quality for JdlLevel {
    type Individual = SituationElement;
    type Value = &'static str;

    fn get(&self, element: &SituationElement) -> Option<&'static str> {
        Some(match element {
            SituationElement::Entity => "JDL Level 1: Object Assessment",
            SituationElement::Relationship => "JDL Level 2: Situation Assessment",
            SituationElement::Intent => "JDL Level 2: Situation Assessment (intent)",
            SituationElement::Environment => "JDL Level 0/1: Source Preprocessing",
        })
    }
}

/// Axiom: situation assessment requires entity identification first.
///
/// You cannot assess relationships or intent without first identifying entities.
pub struct EntityIdentificationFirst;

impl Axiom for EntityIdentificationFirst {
    fn description(&self) -> &str {
        "situation assessment requires entity identification first (JDL Level 1 before Level 2)"
    }
    fn holds(&self) -> bool {
        let morphisms = SituationCategory::morphisms();
        // Verify Entity -> Relationship exists
        let entity_to_rel = morphisms
            .iter()
            .any(|m| m.from == SituationElement::Entity && m.to == SituationElement::Relationship);
        // Verify no Intent without Entity (i.e., Entity -> Intent path exists)
        let entity_to_intent = morphisms
            .iter()
            .any(|m| m.from == SituationElement::Entity && m.to == SituationElement::Intent);
        entity_to_rel && entity_to_intent
    }
}

/// Axiom: intent cannot be assessed without relationship context.
pub struct IntentRequiresRelationship;

impl Axiom for IntentRequiresRelationship {
    fn description(&self) -> &str {
        "intent inference requires relationship assessment"
    }
    fn holds(&self) -> bool {
        let morphisms = SituationCategory::morphisms();
        morphisms
            .iter()
            .any(|m| m.from == SituationElement::Relationship && m.to == SituationElement::Intent)
    }
}

pub struct SituationOntology;

impl Ontology for SituationOntology {
    type Cat = SituationCategory;
    type Qual = JdlLevel;

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(EntityIdentificationFirst),
            Box::new(IntentRequiresRelationship),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<SituationCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        SituationOntology::validate().unwrap();
    }
}

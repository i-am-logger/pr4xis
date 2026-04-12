use pr4xis::category::{Category, Entity, Relationship};
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

/// Assessment dependency between situation elements.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssessmentDependency {
    pub from: SituationElement,
    pub to: SituationElement,
}

impl Relationship for AssessmentDependency {
    type Object = SituationElement;
    fn source(&self) -> SituationElement {
        self.from
    }
    fn target(&self) -> SituationElement {
        self.to
    }
}

/// Category for situation assessment dependencies.
///
/// Entity identification must precede relationship assessment,
/// which precedes intent inference. Environment informs all levels.
pub struct SituationCategory;

impl Category for SituationCategory {
    type Object = SituationElement;
    type Morphism = AssessmentDependency;

    fn identity(obj: &SituationElement) -> AssessmentDependency {
        AssessmentDependency {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &AssessmentDependency, g: &AssessmentDependency) -> Option<AssessmentDependency> {
        if f.to != g.from {
            return None;
        }
        let dep = AssessmentDependency {
            from: f.from,
            to: g.to,
        };
        if Self::morphisms().contains(&dep) {
            Some(dep)
        } else {
            None
        }
    }

    fn morphisms() -> Vec<AssessmentDependency> {
        use SituationElement::*;
        let mut m = Vec::new();
        // Identities
        for s in SituationElement::variants() {
            m.push(AssessmentDependency { from: s, to: s });
        }
        // Entity -> Relationship -> Intent (assessment chain)
        m.push(AssessmentDependency {
            from: Entity,
            to: Relationship,
        });
        m.push(AssessmentDependency {
            from: Relationship,
            to: Intent,
        });
        m.push(AssessmentDependency {
            from: Entity,
            to: Intent,
        }); // transitive

        // Environment informs all elements
        m.push(AssessmentDependency {
            from: Environment,
            to: Entity,
        });
        m.push(AssessmentDependency {
            from: Environment,
            to: Relationship,
        });
        m.push(AssessmentDependency {
            from: Environment,
            to: Intent,
        });
        m
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

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(EntityIdentificationFirst),
            Box::new(IntentRequiresRelationship),
        ]
    }
}

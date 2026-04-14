use crate::category::entity::Entity;
use crate::category::relationship::Relationship;
use crate::category::{Category, Functor};

use super::being::Being;
use super::category::{DolceCategory, OntologicalRelation, RelationKind};

/// The praxis meta-category: our current type system modeled as a category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pr4xisType {
    Entity,
    Situation,
    Action,
    Quality,
    CategoryType,
    Axiom,
    Proposition,
}

impl Entity for Pr4xisType {
    fn variants() -> Vec<Self> {
        vec![
            Self::Entity,
            Self::Situation,
            Self::Action,
            Self::Quality,
            Self::CategoryType,
            Self::Axiom,
            Self::Proposition,
        ]
    }
}

/// Relationships between praxis types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PraxisRelation {
    pub from: Pr4xisType,
    pub to: Pr4xisType,
    pub name: &'static str,
}

impl Relationship for PraxisRelation {
    type Object = Pr4xisType;
    fn source(&self) -> Pr4xisType {
        self.from
    }
    fn target(&self) -> Pr4xisType {
        self.to
    }
}

/// The praxis meta-category.
pub struct Pr4xisMetaCategory;

impl Category for Pr4xisMetaCategory {
    type Object = Pr4xisType;
    type Morphism = PraxisRelation;

    fn identity(obj: &Pr4xisType) -> PraxisRelation {
        PraxisRelation {
            from: *obj,
            to: *obj,
            name: "identity",
        }
    }

    fn compose(f: &PraxisRelation, g: &PraxisRelation) -> Option<PraxisRelation> {
        if f.to != g.from {
            return None;
        }
        if f.name == "identity" {
            return Some(g.clone());
        }
        if g.name == "identity" {
            return Some(f.clone());
        }
        Some(PraxisRelation {
            from: f.from,
            to: g.to,
            name: "composed",
        })
    }

    fn morphisms() -> Vec<PraxisRelation> {
        let mut m = Vec::new();
        for t in Pr4xisType::variants() {
            m.push(Self::identity(&t));
        }
        m.push(PraxisRelation {
            from: Pr4xisType::Action,
            to: Pr4xisType::Situation,
            name: "operates_on",
        });
        m.push(PraxisRelation {
            from: Pr4xisType::Quality,
            to: Pr4xisType::Entity,
            name: "inheres_in",
        });
        m.push(PraxisRelation {
            from: Pr4xisType::Axiom,
            to: Pr4xisType::CategoryType,
            name: "constrains",
        });
        m.push(PraxisRelation {
            from: Pr4xisType::Proposition,
            to: Pr4xisType::Entity,
            name: "evaluates",
        });
        // Composed morphisms for closure
        m.push(PraxisRelation {
            from: Pr4xisType::Action,
            to: Pr4xisType::Entity,
            name: "composed",
        });
        m
    }
}

/// Functor from praxis types to DOLCE Being types.
///
/// This is the structure-preserving map that proves our type system
/// correctly aligns with the DOLCE upper ontology.
pub struct Pr4xisToDolce;

impl Functor for Pr4xisToDolce {
    type Source = Pr4xisMetaCategory;
    type Target = DolceCategory;

    fn map_object(obj: &Pr4xisType) -> Being {
        match obj {
            Pr4xisType::Entity => Being::AbstractObject,
            Pr4xisType::Situation => Being::SocialObject,
            Pr4xisType::Action => Being::Event,
            Pr4xisType::Quality => Being::Quality,
            Pr4xisType::CategoryType => Being::AbstractObject,
            Pr4xisType::Axiom => Being::AbstractObject,
            Pr4xisType::Proposition => Being::AbstractObject,
        }
    }

    fn map_morphism(m: &PraxisRelation) -> OntologicalRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = if m.name == "identity" || from == to {
            RelationKind::Identity
        } else {
            // Find the appropriate DOLCE relation kind
            match (from, to) {
                (Being::Quality, b) if b.is_endurant() => RelationKind::InheresIn,
                (e, p) if e.is_endurant() && p.is_perdurant() => RelationKind::ParticipatesIn,
                _ => RelationKind::Composed,
            }
        };
        OntologicalRelation { from, to, kind }
    }
}

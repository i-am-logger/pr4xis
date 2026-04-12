use crate::category::entity::Entity;
use crate::category::relationship::Relationship;
use crate::category::{Category, Functor};

use super::being::Being;
use super::category::{DolceCategory, OntologicalRelation, RelationKind};

/// The praxis meta-category: our current type system modeled as a category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PraxisType {
    Entity,
    Situation,
    Action,
    Quality,
    CategoryType,
    Axiom,
    Proposition,
}

impl Entity for PraxisType {
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
    pub from: PraxisType,
    pub to: PraxisType,
    pub name: &'static str,
}

impl Relationship for PraxisRelation {
    type Object = PraxisType;
    fn source(&self) -> PraxisType {
        self.from
    }
    fn target(&self) -> PraxisType {
        self.to
    }
}

/// The praxis meta-category.
pub struct PraxisMetaCategory;

impl Category for PraxisMetaCategory {
    type Object = PraxisType;
    type Morphism = PraxisRelation;

    fn identity(obj: &PraxisType) -> PraxisRelation {
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
        for t in PraxisType::variants() {
            m.push(Self::identity(&t));
        }
        m.push(PraxisRelation {
            from: PraxisType::Action,
            to: PraxisType::Situation,
            name: "operates_on",
        });
        m.push(PraxisRelation {
            from: PraxisType::Quality,
            to: PraxisType::Entity,
            name: "inheres_in",
        });
        m.push(PraxisRelation {
            from: PraxisType::Axiom,
            to: PraxisType::CategoryType,
            name: "constrains",
        });
        m.push(PraxisRelation {
            from: PraxisType::Proposition,
            to: PraxisType::Entity,
            name: "evaluates",
        });
        // Composed morphisms for closure
        m.push(PraxisRelation {
            from: PraxisType::Action,
            to: PraxisType::Entity,
            name: "composed",
        });
        m
    }
}

/// Functor from praxis types to DOLCE Being types.
///
/// This is the structure-preserving map that proves our type system
/// correctly aligns with the DOLCE upper ontology.
pub struct PraxisToDolce;

impl Functor for PraxisToDolce {
    type Source = PraxisMetaCategory;
    type Target = DolceCategory;

    fn map_object(obj: &PraxisType) -> Being {
        match obj {
            PraxisType::Entity => Being::AbstractObject,
            PraxisType::Situation => Being::SocialObject,
            PraxisType::Action => Being::Event,
            PraxisType::Quality => Being::Quality,
            PraxisType::CategoryType => Being::AbstractObject,
            PraxisType::Axiom => Being::AbstractObject,
            PraxisType::Proposition => Being::AbstractObject,
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

use crate::category::entity::Concept;
use crate::category::relationship::Relationship;
use crate::category::{Category, Functor};
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::being::Being;
use super::category::{DolceCategory, OntologicalRelation, RelationKind};

/// The workspace's own meta-category: our current type system modeled as a category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OwnType {
    Concept,
    Situation,
    Action,
    Quality,
    CategoryType,
    Axiom,
    Proposition,
}

impl Concept for OwnType {
    fn variants() -> Vec<Self> {
        vec![
            Self::Concept,
            Self::Situation,
            Self::Action,
            Self::Quality,
            Self::CategoryType,
            Self::Axiom,
            Self::Proposition,
        ]
    }
}

/// Relationships between our own types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwnRelation {
    pub from: OwnType,
    pub to: OwnType,
    pub name: &'static str,
}

impl Relationship for OwnRelation {
    type Object = OwnType;
    type Kind = ();
    fn source(&self) -> OwnType {
        self.from
    }
    fn target(&self) -> OwnType {
        self.to
    }
    fn kind(&self) {}
}

/// The workspace's own meta-category.
pub struct OwnMetaCategory;

impl Category for OwnMetaCategory {
    type Object = OwnType;
    type Morphism = OwnRelation;

    fn identity(obj: &OwnType) -> OwnRelation {
        OwnRelation {
            from: *obj,
            to: *obj,
            name: "identity",
        }
    }

    fn compose(f: &OwnRelation, g: &OwnRelation) -> Option<OwnRelation> {
        if f.to != g.from {
            return None;
        }
        if f.name == "identity" {
            return Some(g.clone());
        }
        if g.name == "identity" {
            return Some(f.clone());
        }
        Some(OwnRelation {
            from: f.from,
            to: g.to,
            name: "composed",
        })
    }

    fn morphisms() -> Vec<OwnRelation> {
        let mut m = Vec::new();
        for t in OwnType::variants() {
            m.push(Self::identity(&t));
        }
        m.push(OwnRelation {
            from: OwnType::Action,
            to: OwnType::Situation,
            name: "operates_on",
        });
        m.push(OwnRelation {
            from: OwnType::Quality,
            to: OwnType::Concept,
            name: "inheres_in",
        });
        m.push(OwnRelation {
            from: OwnType::Axiom,
            to: OwnType::CategoryType,
            name: "constrains",
        });
        m.push(OwnRelation {
            from: OwnType::Proposition,
            to: OwnType::Concept,
            name: "evaluates",
        });
        // Composed morphisms for closure
        m.push(OwnRelation {
            from: OwnType::Action,
            to: OwnType::Concept,
            name: "composed",
        });
        m
    }
}

/// Functor from own types to DOLCE Being types.
///
/// This is the structure-preserving map that proves our type system
/// correctly aligns with the DOLCE upper ontology.
pub struct OwnToDolce;

impl Functor for OwnToDolce {
    type Source = OwnMetaCategory;
    type Target = DolceCategory;

    fn map_object(obj: &OwnType) -> Being {
        match obj {
            OwnType::Concept => Being::AbstractObject,
            OwnType::Situation => Being::SocialObject,
            OwnType::Action => Being::Event,
            OwnType::Quality => Being::Quality,
            OwnType::CategoryType => Being::AbstractObject,
            OwnType::Axiom => Being::AbstractObject,
            OwnType::Proposition => Being::AbstractObject,
        }
    }

    fn map_morphism(m: &OwnRelation) -> OntologicalRelation {
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

    crate::relationship_meta!(
        "OwnToDolce",
        "map pr4xis's own meta-types to DOLCE's upper ontology",
        "Masolo et al. (2003) WonderWeb D18 'DOLCE'"
    );
}

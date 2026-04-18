use crate::category::Category;
use crate::category::entity::Concept;
use crate::category::relationship::Relationship;
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::being::Being;

/// Ontological relationship between types of being.
///
/// A directed relationship from one Being type to another.
/// Named relationships encode fundamental DOLCE relations;
/// composed relationships track transitive connections.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OntologicalRelation {
    pub from: Being,
    pub to: Being,
    pub kind: RelationKind,
}

/// The kind of ontological relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationKind {
    /// Identity morphism.
    Identity,
    /// An endurant participates in a perdurant.
    ParticipatesIn,
    /// A quality inheres in an endurant.
    InheresIn,
    /// A physical endurant constitutes a social object.
    Constitutes,
    /// An event is part of a process.
    PartOf,
    /// A composed relationship (transitive).
    Composed,
}

impl OntologicalRelation {
    fn new(from: Being, to: Being, kind: RelationKind) -> Self {
        Self { from, to, kind }
    }
}

impl Relationship for OntologicalRelation {
    type Object = Being;
    type Kind = RelationKind;

    fn source(&self) -> Being {
        self.from
    }

    fn target(&self) -> Being {
        self.to
    }

    fn kind(&self) -> RelationKind {
        self.kind
    }
}

/// The DOLCE upper category: Being types as objects, ontological relations as morphisms.
pub struct DolceCategory;

impl Category for DolceCategory {
    type Object = Being;
    type Morphism = OntologicalRelation;

    fn identity(obj: &Being) -> OntologicalRelation {
        OntologicalRelation::new(*obj, *obj, RelationKind::Identity)
    }

    fn compose(f: &OntologicalRelation, g: &OntologicalRelation) -> Option<OntologicalRelation> {
        if f.to != g.from {
            return None;
        }

        // Identity laws
        if f.kind == RelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == RelationKind::Identity {
            return Some(f.clone());
        }

        // All other compositions produce a Composed morphism from f.from to g.to
        Some(OntologicalRelation::new(
            f.from,
            g.to,
            RelationKind::Composed,
        ))
    }

    fn morphisms() -> Vec<OntologicalRelation> {
        use Being::*;
        use RelationKind::*;

        let mut m = Vec::new();

        // Identity for each Being
        for b in Being::variants() {
            m.push(OntologicalRelation::new(b, b, Identity));
        }

        // Participation: endurants participate in perdurants
        for endurant in [PhysicalEndurant, SocialObject, MentalObject, AbstractObject] {
            for perdurant in [Event, Process] {
                m.push(OntologicalRelation::new(
                    endurant,
                    perdurant,
                    ParticipatesIn,
                ));
            }
        }

        // Inherence: quality inheres in endurants
        for bearer in [PhysicalEndurant, SocialObject, MentalObject, AbstractObject] {
            m.push(OntologicalRelation::new(Quality, bearer, InheresIn));
        }

        // Constitution: physical constitutes social
        m.push(OntologicalRelation::new(
            PhysicalEndurant,
            SocialObject,
            Constitutes,
        ));

        // Part-of: event part of process
        m.push(OntologicalRelation::new(Event, Process, PartOf));

        // Composed morphisms (transitive closure for closure law)
        // Quality → Endurant → Perdurant (inherence then participation)
        for perdurant in [Event, Process] {
            m.push(OntologicalRelation::new(Quality, perdurant, Composed));
        }
        // Physical → Social → Perdurant (constitution then participation)
        for perdurant in [Event, Process] {
            m.push(OntologicalRelation::new(
                PhysicalEndurant,
                perdurant,
                Composed,
            ));
        }
        // Event → Process (part-of already covers this)
        // Quality → Social (inherence already covers this, but also via physical)
        // Constitution + inherence compositions are already covered by direct morphisms

        m
    }
}

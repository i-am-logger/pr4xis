// C2 → Metacognition functor.
//
// C2 (self-monitoring) maps directly to Metacognition because
// they ARE the same dimension (Dehaene, Lau & Kouider 2017).
//
// C2's self-monitoring IS Nelson-Narens metacognition:
// HigherOrderRepresentation → MetaLevel
// FirstOrderState → ObjectLevel
// AccessConsciousness → Trace (what's reportable)
// CauseEffectStructure → Evaluation
// Mechanism → Monitoring
// Repertoire → Control
// PhenomenalConsciousness → EpistemicAssessment
//
// Source: Dehaene et al. (2017); Nelson & Narens (1990); Koriat (2007)

use pr4xis::category::Functor;

use super::ontology::{C2Category, C2Concept, C2Relation, C2RelationKind};
use crate::cognitive::cognition::metacognition::{
    MetaCognitionCategory, MetaConcept, MetaRelation, MetaRelationKind,
};

pub struct C2ToMetacognition;

impl Functor for C2ToMetacognition {
    type Source = C2Category;
    type Target = MetaCognitionCategory;

    fn map_object(obj: &C2Concept) -> MetaConcept {
        match obj {
            C2Concept::HigherOrderRepresentation => MetaConcept::MetaLevel,
            C2Concept::FirstOrderState => MetaConcept::ObjectLevel,
            C2Concept::AccessConsciousness => MetaConcept::Trace,
            C2Concept::CauseEffectStructure => MetaConcept::Evaluation,
            C2Concept::Mechanism => MetaConcept::Monitoring,
            C2Concept::Repertoire => MetaConcept::Control,
            C2Concept::PhenomenalConsciousness => MetaConcept::EpistemicAssessment,
        }
    }

    fn map_morphism(m: &C2Relation) -> MetaRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            C2RelationKind::Identity => MetaRelationKind::Identity,
            C2RelationKind::Represents => MetaRelationKind::Observes,
            C2RelationKind::Generates => MetaRelationKind::Records,
            C2RelationKind::HasComponent => MetaRelationKind::Assesses,
            C2RelationKind::Enables => MetaRelationKind::Classifies,
            C2RelationKind::Composed => MetaRelationKind::Composed,
        };
        MetaRelation { from, to, kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<C2ToMetacognition>().unwrap();
    }
}

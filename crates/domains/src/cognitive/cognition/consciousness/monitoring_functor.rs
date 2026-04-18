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
    MetaCognitionCategory, MetaCognitionConcept, MetaCognitionRelation, MetaCognitionRelationKind,
};

pub struct C2ToMetacognition;

impl Functor for C2ToMetacognition {
    type Source = C2Category;
    type Target = MetaCognitionCategory;

    fn map_object(obj: &C2Concept) -> MetaCognitionConcept {
        match obj {
            C2Concept::HigherOrderRepresentation => MetaCognitionConcept::MetaLevel,
            C2Concept::FirstOrderState => MetaCognitionConcept::ObjectLevel,
            C2Concept::AccessConsciousness => MetaCognitionConcept::Trace,
            C2Concept::CauseEffectStructure => MetaCognitionConcept::Evaluation,
            C2Concept::Mechanism => MetaCognitionConcept::Monitoring,
            C2Concept::Repertoire => MetaCognitionConcept::Control,
            C2Concept::PhenomenalConsciousness => MetaCognitionConcept::EpistemicAssessment,
        }
    }

    fn map_morphism(m: &C2Relation) -> MetaCognitionRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            C2RelationKind::Identity => MetaCognitionRelationKind::Identity,
            C2RelationKind::Represents => MetaCognitionRelationKind::Observes,
            C2RelationKind::Generates => MetaCognitionRelationKind::Records,
            C2RelationKind::HasComponent => MetaCognitionRelationKind::Assesses,
            C2RelationKind::Enables => MetaCognitionRelationKind::Classifies,
            C2RelationKind::Composed => MetaCognitionRelationKind::Composed,
        };
        MetaCognitionRelation { from, to, kind }
    }
}
pr4xis::register_functor!(C2ToMetacognition);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<C2ToMetacognition>().unwrap();
    }
}

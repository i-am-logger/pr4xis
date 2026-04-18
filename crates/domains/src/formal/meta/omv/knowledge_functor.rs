// OMV → Knowledge functor.
//
// Proof that ontology metadata (OMV/MOD) maps to knowledge structure (VoID).
// A SemanticArtefact IS a Vocabulary. A Catalog IS a KnowledgeBase.
// Analytics IS a Descriptor. The structural overlap is exact for
// the core metadata; OMV's engineering/evaluation concepts are richer.
//
// Source: Hartmann (2005) × W3C VoID (2011)

use pr4xis::category::Functor;

use super::ontology::*;
use crate::formal::information::knowledge::ontology::*;

pub struct OmvToKnowledge;

impl Functor for OmvToKnowledge {
    type Source = OmvCategory;
    type Target = KnowledgeBaseCategory;

    fn map_object(obj: &OmvConcept) -> KnowledgeConcept {
        match obj {
            OmvConcept::SemanticArtefact => KnowledgeConcept::Vocabulary,
            OmvConcept::Catalog => KnowledgeConcept::KnowledgeBase,
            OmvConcept::Analytics => KnowledgeConcept::Descriptor,
            OmvConcept::FormalityLevel => KnowledgeConcept::Schema,
            OmvConcept::RepresentationParadigm => KnowledgeConcept::Schema,
            OmvConcept::Methodology => KnowledgeConcept::DataSource,
            OmvConcept::DesignedTask => KnowledgeConcept::Entry,
            OmvConcept::Evaluation => KnowledgeConcept::Descriptor,
            OmvConcept::NaturalLanguage => KnowledgeConcept::Entry,
            OmvConcept::CompetencyQuestion => KnowledgeConcept::Entry,
        }
    }

    fn map_morphism(m: &OmvRelation) -> KnowledgeRelation {
        KnowledgeRelation {
            from: Self::map_object(&m.from),
            to: Self::map_object(&m.to),
            kind: map_kind(&m.kind),
        }
    }
}
pr4xis::register_functor!(OmvToKnowledge);

fn map_kind(kind: &OmvRelationKind) -> KnowledgeRelationKind {
    match kind {
        OmvRelationKind::Catalogs => KnowledgeRelationKind::Catalogs,
        OmvRelationKind::HasAnalytics => KnowledgeRelationKind::DescribedBy,
        OmvRelationKind::HasEvaluation => KnowledgeRelationKind::DescribedBy,
        OmvRelationKind::HasFormalityLevel => KnowledgeRelationKind::ConformsTo,
        OmvRelationKind::HasRepresentation => KnowledgeRelationKind::ConformsTo,
        OmvRelationKind::UsedMethodology => KnowledgeRelationKind::DerivedFrom,
        OmvRelationKind::DesignedFor => KnowledgeRelationKind::Contains,
        OmvRelationKind::HasLanguage => KnowledgeRelationKind::Contains,
        OmvRelationKind::HasCompetencyQuestion => KnowledgeRelationKind::Contains,
        OmvRelationKind::Identity => KnowledgeRelationKind::Identity,
        OmvRelationKind::Composed => KnowledgeRelationKind::Composed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<OmvToKnowledge>().unwrap();
    }
}

// NLG → Pipeline functor.
//
// NLG output connects to the Generate direction of the pipeline.
// Realization produces SurfaceForm. The NLG stages map to pipeline
// stages in reverse (Generate is the right adjoint of Parse).
//
// Source: Reiter & Dale (2000) → de Groote (2001)

use pr4xis::category::Functor;

use super::nlg::{NlgCategory, NlgConcept, NlgRelation};
use crate::cognitive::linguistics::pipeline::ontology::{
    PipelineCategory, PipelineConcept, PipelineRelation,
};

pub struct NlgToPipeline;

impl Functor for NlgToPipeline {
    type Source = NlgCategory;
    type Target = PipelineCategory;

    fn map_object(obj: &NlgConcept) -> PipelineConcept {
        match obj {
            NlgConcept::CommunicativeGoal => PipelineConcept::SemanticRepresentation,
            NlgConcept::ContentDetermination => PipelineConcept::SemanticRepresentation,
            NlgConcept::Message => PipelineConcept::SemanticRepresentation,
            NlgConcept::DocumentPlanning => PipelineConcept::SyntacticStructure,
            NlgConcept::RhetoricalRelation => PipelineConcept::SyntacticStructure,
            NlgConcept::Microplanning => PipelineConcept::LexiconHomomorphism,
            NlgConcept::ReferringExpression => PipelineConcept::ProofTerm,
            NlgConcept::Realization => PipelineConcept::Generate,
            NlgConcept::SurfaceText => PipelineConcept::SurfaceForm,
            NlgConcept::KnowledgeGathering => PipelineConcept::MeaningFunctor,
            NlgConcept::Monitor => PipelineConcept::Unit,
        }
    }

    fn map_morphism(m: &NlgRelation) -> PipelineRelation {
        PipelineRelation {
            from: Self::map_object(&m.from),
            to: Self::map_object(&m.to),
        }
    }
}
pr4xis::register_functor!(NlgToPipeline);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<NlgToPipeline>().unwrap();
    }
}

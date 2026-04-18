// C1 → Pipeline functor.
//
// C1 (global broadcasting) connects to the language pipeline because
// broadcasting IS what makes information available for processing.
// The global workspace IS where the Parse ⊣ Generate pipeline operates.
//
// Dehaene et al. (2017): C1 = selection of information for global
// broadcasting, making it flexibly available for computation and report.
// This IS the pipeline — information enters the workspace (Parse),
// gets processed, and broadcasts the result (Generate).
//
// Source: Dehaene, Lau & Kouider (2017); Baars (1988)

use pr4xis::category::Functor;

use super::ontology::{C1Category, C1Concept, C1Relation};
use crate::cognitive::linguistics::pipeline::ontology::{
    PipelineCategory, PipelineConcept, PipelineRelation,
};

pub struct C1ToPipeline;

impl Functor for C1ToPipeline {
    type Source = C1Category;
    type Target = PipelineCategory;

    fn map_object(obj: &C1Concept) -> PipelineConcept {
        match obj {
            // GlobalWorkspace IS where the pipeline operates
            C1Concept::GlobalWorkspace => PipelineConcept::SurfaceForm,
            // Coalition = competing parse interpretations
            C1Concept::Coalition => PipelineConcept::SyntacticStructure,
            // BroadcastMessage = the result broadcast after processing
            C1Concept::BroadcastMessage => PipelineConcept::SemanticRepresentation,
            // UnconsciousProcessor = processing outside the pipeline
            C1Concept::UnconsciousProcessor => PipelineConcept::PartialResult,
            // ConsciousAccess = entering the parse pipeline
            C1Concept::ConsciousAccess => PipelineConcept::Parse,
            // Attention = the meaning functor (selects what to focus on)
            C1Concept::Attention => PipelineConcept::MeaningFunctor,
            // IntegratedInformation = the lexicon (integrates all word knowledge)
            C1Concept::IntegratedInformation => PipelineConcept::LexiconHomomorphism,
        }
    }

    fn map_morphism(m: &C1Relation) -> PipelineRelation {
        PipelineRelation {
            from: Self::map_object(&m.from),
            to: Self::map_object(&m.to),
        }
    }
}
pr4xis::register_functor!(C1ToPipeline);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<C1ToPipeline>().unwrap();
    }
}

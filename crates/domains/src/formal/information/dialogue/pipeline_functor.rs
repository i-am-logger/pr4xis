// Dialogue → Pipeline functor.
//
// Proof that dialogue processing IS the Parse ⊣ Generate pipeline.
// Understanding an utterance IS parsing (left adjoint).
// Generating a response IS generation (right adjoint).
// The dialogue state IS the semantic representation.
// The utterance IS the surface form.
//
// This connects conversational AI to the categorical pipeline:
// a question arrives through dialogue, gets parsed, migrated
// through the algebra, and the answer returns through generation.

use pr4xis::category::Functor;

use super::ontology::*;
use crate::cognitive::linguistics::pipeline::ontology::*;

pub struct DialogueToPipeline;

impl Functor for DialogueToPipeline {
    type Source = DialogueCategory;
    type Target = PipelineCategory;

    fn map_object(obj: &DialogueConcept) -> PipelineConcept {
        match obj {
            // Utterance IS the surface form
            DialogueConcept::Utterance => PipelineConcept::SurfaceForm,
            // Understanding IS parsing
            DialogueConcept::Understanding => PipelineConcept::Parse,
            // Generation IS generation
            DialogueConcept::Generation => PipelineConcept::Generate,
            // DialogueState IS semantic representation (shared meaning)
            DialogueConcept::DialogueState => PipelineConcept::SemanticRepresentation,
            // DialogueAct IS syntactic structure (the typed speech act)
            DialogueConcept::DialogueAct => PipelineConcept::SyntacticStructure,
            // Topic IS semantic representation (what's being discussed)
            DialogueConcept::Topic => PipelineConcept::SemanticRepresentation,
            // History IS a stream (incremental accumulation)
            DialogueConcept::History => PipelineConcept::Stream,
            // Participant IS surface (the source/target of forms)
            DialogueConcept::Participant => PipelineConcept::SurfaceForm,
            // TurnManagement IS partial result (incomplete exchange)
            DialogueConcept::TurnManagement => PipelineConcept::PartialResult,
            // Grounding IS the unit (what survives the round-trip)
            DialogueConcept::Grounding => PipelineConcept::Unit,
            // QUD IS the meaning functor (drives interpretation)
            DialogueConcept::QUD => PipelineConcept::MeaningFunctor,
            // CommonGround IS semantic representation
            DialogueConcept::CommonGround => PipelineConcept::SemanticRepresentation,
            // Intention IS semantic representation (what the speaker means)
            DialogueConcept::Intention => PipelineConcept::SemanticRepresentation,
            // GroundingAct IS the counit (round-trip verification)
            DialogueConcept::GroundingAct => PipelineConcept::Counit,
            // Repair IS partial result (incomplete understanding)
            DialogueConcept::Repair => PipelineConcept::PartialResult,
        }
    }

    fn map_morphism(m: &DialogueRelation) -> PipelineRelation {
        PipelineRelation {
            from: Self::map_object(&m.from),
            to: Self::map_object(&m.to),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<DialogueToPipeline>().unwrap();
    }
}

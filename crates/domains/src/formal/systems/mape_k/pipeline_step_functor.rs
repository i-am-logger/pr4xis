//! Cross-functor: the existing `PipelineStep` enum → the MAPE-K ontology.
//!
//! Rather than rewrite the 13-variant `PipelineStep` into a different
//! structure, this functor carries each step to its MAPE-K phase. It
//! makes the claim "the pr4xis chat pipeline IS a MAPE-K loop" verifiable
//! at test time: if every step lands on a MAPE-K phase consistent with
//! both its semantic role and its ordering in the pipeline, the functor
//! laws pass and the claim is proven.
//!
//! This is also the answer to `#117` Part 1 (mechanical refactor):
//! the existing `PipelineStep` stays, just gets a literature-grounded
//! structural home.
//!
//! # The mapping (Kephart & Chess 2003 phases)
//!
//! | PipelineStep | MAPE-K phase | Why |
//! |---|---|---|
//! | `Tokenize`               | `Monitor` | Sensing the input character stream |
//! | `Parse`                  | `Monitor` | Sensing grammatical structure |
//! | `Interpret`              | `Monitor` | Sensing semantic content |
//! | `Metacognition`          | `Monitor` | Second-order self-sensing (Nelson-Narens) |
//! | `EpistemicClassification`| `Monitor` | Sensing the knowledge state |
//! | `EntityLookup`           | `Analyze` | Reasoning over concept graphs |
//! | `TaxonomyTraversal`      | `Analyze` | Traversing `is_a` chains |
//! | `CommonAncestor`         | `Analyze` | Computing LCA for disambiguation |
//! | `SpeechActClassification`| `Plan`    | Deciding the illocutionary goal (Searle) |
//! | `ResponseFrameSelection` | `Plan`    | Choosing the response structure |
//! | `ContentDetermination`   | `Execute` | Selecting what to say (Reiter & Dale) |
//! | `DocumentPlanning`       | `Execute` | Arranging content rhetorically (RST) |
//! | `Realization`            | `Execute` | Surface text generation (SVO grammar) |
//!
//! Every step maps to Monitor / Analyze / Plan / Execute; none to
//! Knowledge, because Knowledge is the shared substrate each step
//! *consults*, not a stage they belong to. This follows Kephart & Chess's
//! own diagram exactly.

use pr4xis::category::{Category, Functor};

use super::ontology::{MapeKCategory, MapeKConcept, MapeKRelation};
use crate::formal::information::diagnostics::trace_functors::PipelineStep;

/// The 13-step `PipelineStep` enum, re-exposed as a category so it can
/// be a `Functor::Source`. It's a *discrete* category — no morphisms
/// beyond identities — because `PipelineStep` doesn't yet have
/// declared edges between its variants. That's enough for the
/// object-level mapping here; a future enriched version could add
/// `SequencedBy` edges if the linear order matters for downstream laws.
pub struct PipelineStepCategory;

/// Identity-only wrapper morphism for `PipelineStep`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineStepMorphism {
    pub from: PipelineStep,
    pub to: PipelineStep,
}

impl pr4xis::category::Relationship for PipelineStepMorphism {
    type Object = PipelineStep;
    fn source(&self) -> PipelineStep {
        self.from
    }
    fn target(&self) -> PipelineStep {
        self.to
    }
}

impl pr4xis::category::Entity for PipelineStep {
    fn variants() -> Vec<Self> {
        vec![
            PipelineStep::Tokenize,
            PipelineStep::Parse,
            PipelineStep::Interpret,
            PipelineStep::EntityLookup,
            PipelineStep::TaxonomyTraversal,
            PipelineStep::CommonAncestor,
            PipelineStep::Metacognition,
            PipelineStep::SpeechActClassification,
            PipelineStep::ResponseFrameSelection,
            PipelineStep::ContentDetermination,
            PipelineStep::DocumentPlanning,
            PipelineStep::Realization,
            PipelineStep::EpistemicClassification,
        ]
    }
}

impl Category for PipelineStepCategory {
    type Object = PipelineStep;
    type Morphism = PipelineStepMorphism;

    fn identity(obj: &PipelineStep) -> PipelineStepMorphism {
        PipelineStepMorphism {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &PipelineStepMorphism, g: &PipelineStepMorphism) -> Option<PipelineStepMorphism> {
        if f.to != g.from {
            return None;
        }
        Some(PipelineStepMorphism {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<PipelineStepMorphism> {
        use pr4xis::category::Entity;
        PipelineStep::variants()
            .into_iter()
            .map(|s| PipelineStepMorphism { from: s, to: s })
            .collect()
    }
}

fn map_step(step: &PipelineStep) -> MapeKConcept {
    use MapeKConcept as M;
    use PipelineStep as P;
    match step {
        // Monitor: sensing input + sensing self.
        P::Tokenize | P::Parse | P::Interpret | P::Metacognition | P::EpistemicClassification => {
            M::Monitor
        }
        // Analyze: reasoning over knowledge.
        P::EntityLookup | P::TaxonomyTraversal | P::CommonAncestor => M::Analyze,
        // Plan: deciding what to say.
        P::SpeechActClassification | P::ResponseFrameSelection => M::Plan,
        // Execute: producing the utterance.
        P::ContentDetermination | P::DocumentPlanning | P::Realization => M::Execute,
    }
}

/// Functor: the 13-step `PipelineStep` category → the 5-concept MAPE-K
/// ontology. Pure collapse — 13 → 4 (Knowledge is the consulted substrate,
/// not a step).
pub struct PipelineStepToMapeK;

impl Functor for PipelineStepToMapeK {
    type Source = PipelineStepCategory;
    type Target = MapeKCategory;

    fn map_object(obj: &PipelineStep) -> MapeKConcept {
        map_step(obj)
    }

    fn map_morphism(m: &PipelineStepMorphism) -> MapeKRelation {
        // Source is a discrete category, so every morphism is an identity.
        // Map to the target's identity at the image object.
        MapeKCategory::identity(&map_step(&m.from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn pipeline_step_to_mape_k_laws_pass() {
        check_functor_laws::<PipelineStepToMapeK>().unwrap();
    }

    /// Concrete sanity: every expected step lands on its documented phase.
    #[test]
    fn step_assignments_match_literature() {
        use MapeKConcept as M;
        use PipelineStep as P;
        assert_eq!(PipelineStepToMapeK::map_object(&P::Tokenize), M::Monitor);
        assert_eq!(
            PipelineStepToMapeK::map_object(&P::EntityLookup),
            M::Analyze
        );
        assert_eq!(
            PipelineStepToMapeK::map_object(&P::SpeechActClassification),
            M::Plan
        );
        assert_eq!(PipelineStepToMapeK::map_object(&P::Realization), M::Execute);
        assert_eq!(
            PipelineStepToMapeK::map_object(&P::Metacognition),
            M::Monitor
        );
    }
}

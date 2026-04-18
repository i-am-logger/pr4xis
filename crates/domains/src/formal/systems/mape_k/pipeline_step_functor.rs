//! Cross-functor: the existing `PipelineStep` enum → the MAPE-K ontology.
//!
//! Rather than rewrite the 13-variant `PipelineStep` into a different
//! structure, this functor carries each step to its MAPE-K phase. It
//! makes the pipeline's phase classification verifiable at test time:
//! if every step lands on a MAPE-K phase consistent with its semantic
//! role, the functor laws confirm the object mapping is well-defined
//! and identity-preserving.
//!
//! Because the source category here is discrete (identity-only morphisms),
//! these laws do **not** by themselves verify pipeline ordering or prove
//! an M → A → P → E → M loop; that would require enriching
//! `PipelineStepCategory` with sequencing morphisms between steps and
//! mapping them into the corresponding `HandsOffTo` relations.
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
///
/// Fields are private so callers can't construct non-identity morphisms
/// that would break the discrete-category contract assumed by
/// `map_morphism` below. Construct via `identity()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineStepMorphism {
    from: PipelineStep,
    to: PipelineStep,
}

impl PipelineStepMorphism {
    /// The only public constructor — produces an identity morphism on `step`.
    /// Enforces the discrete-category invariant at the type-system boundary.
    pub fn identity(step: PipelineStep) -> Self {
        Self {
            from: step,
            to: step,
        }
    }
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

// `PipelineStep` derives `Entity` at its definition in `trace_functors.rs`,
// so variants() stays in sync with the enum automatically.

impl Category for PipelineStepCategory {
    type Object = PipelineStep;
    type Morphism = PipelineStepMorphism;

    fn identity(obj: &PipelineStep) -> PipelineStepMorphism {
        PipelineStepMorphism::identity(*obj)
    }

    fn compose(f: &PipelineStepMorphism, g: &PipelineStepMorphism) -> Option<PipelineStepMorphism> {
        // Discrete category: the only valid composition is identity ∘ identity
        // on the same object. Any other shape is rejected.
        if f.from == f.to && g.from == g.to && f.to == g.from {
            Some(PipelineStepMorphism::identity(f.from))
        } else {
            None
        }
    }

    fn morphisms() -> Vec<PipelineStepMorphism> {
        use pr4xis::category::Entity;
        PipelineStep::variants()
            .into_iter()
            .map(PipelineStepMorphism::identity)
            .collect()
    }
}

fn map_step(step: &PipelineStep) -> MapeKConcept {
    // `PipelineStep` now directly carries its MAPE-K phase — the mapping
    // is definitional rather than pattern-matched. The functor simply
    // projects out the phase field.
    step.phase()
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
        // The `PipelineStepMorphism::identity(..)` constructor is the only
        // public path in, so this invariant holds by construction. The
        // debug_assert documents the assumption and guards against future
        // drift if new constructors get added.
        use pr4xis::category::Relationship;
        debug_assert_eq!(
            m.source(),
            m.target(),
            "PipelineStepCategory is discrete — non-identity morphisms should be unreachable"
        );
        MapeKCategory::identity(&map_step(&m.source()))
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
        assert_eq!(PipelineStepToMapeK::map_object(&P::TOKENIZE), M::Monitor);
        assert_eq!(
            PipelineStepToMapeK::map_object(&P::ENTITY_LOOKUP),
            M::Analyze
        );
        assert_eq!(
            PipelineStepToMapeK::map_object(&P::SPEECH_ACT_CLASSIFICATION),
            M::Plan
        );
        assert_eq!(PipelineStepToMapeK::map_object(&P::REALIZATION), M::Execute);
        assert_eq!(
            PipelineStepToMapeK::map_object(&P::METACOGNITION),
            M::Monitor
        );
    }
}

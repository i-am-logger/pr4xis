use super::trace_functors::{PipelineStep, Traceable};
use crate::science::linguistics::lambek::reduce::ReductionResult;
use crate::science::linguistics::lambek::reduce::TypedToken;

// Traceable implementations — the trace functor applied to each result type.
//
// Each pipeline result knows how to describe itself for the trace.
// The caller just calls trace.trace_result(&result) — no manual construction.

/// Traceable wrapper for tokenize results.
pub struct TokenizeResult<'a> {
    pub tokens: &'a [TypedToken],
}

impl Traceable for TokenizeResult<'_> {
    fn step(&self) -> PipelineStep {
        PipelineStep::Tokenize
    }

    fn trace_detail(&self) -> String {
        if self.tokens.is_empty() {
            return "empty input — no tokens produced".into();
        }
        self.tokens
            .iter()
            .map(|t| {
                let role = if t.lambek_type.is_noun() {
                    "noun"
                } else if t.lambek_type.is_noun_phrase() {
                    "noun phrase"
                } else if t.lambek_type.is_sentence() {
                    "sentence"
                } else {
                    "modifier"
                };
                format!("{} ({})", t.word, role)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn trace_success(&self) -> bool {
        !self.tokens.is_empty()
    }
}

/// Traceable wrapper for parse (reduction) results.
impl Traceable for ReductionResult {
    fn step(&self) -> PipelineStep {
        PipelineStep::Parse
    }

    fn trace_detail(&self) -> String {
        if self.success {
            let final_type = self
                .final_type
                .as_ref()
                .map(|t| t.notation())
                .unwrap_or_default();
            format!("success → {}", final_type)
        } else {
            "failed — could not reduce to S".into()
        }
    }

    fn trace_success(&self) -> bool {
        self.success
    }
}

/// Traceable wrapper for Montague interpretation results.
pub struct InterpretResult<'a> {
    pub meaning: &'a crate::science::linguistics::lambek::montague::Sem,
}

impl Traceable for InterpretResult<'_> {
    fn step(&self) -> PipelineStep {
        PipelineStep::Interpret
    }

    fn trace_detail(&self) -> String {
        use crate::science::linguistics::lambek::montague::Sem;
        match self.meaning {
            Sem::Question {
                predicate,
                arguments,
            } => {
                let args: Vec<String> = arguments.iter().map(|a| a.describe()).collect();
                format!("question: {}({})", predicate, args.join(", "))
            }
            Sem::Prop {
                predicate,
                arguments,
            } => {
                let args: Vec<String> = arguments.iter().map(|a| a.describe()).collect();
                format!("statement: {}({})", predicate, args.join(", "))
            }
            Sem::Entity { word, .. } => format!("entity: {word}"),
            Sem::Pred { word } => format!("concept: {word}"),
            Sem::Func { word, .. } => format!("function: {word}"),
        }
    }

    fn trace_success(&self) -> bool {
        true
    }
}

/// Traceable for epistemic classification.
pub struct EpistemicResult {
    pub state: crate::science::cognition::epistemics::EpistemicState,
    pub known_words: Vec<String>,
    pub unknown_words: Vec<String>,
}

impl Traceable for EpistemicResult {
    fn step(&self) -> PipelineStep {
        PipelineStep::EpistemicClassification
    }

    fn trace_detail(&self) -> String {
        format!(
            "{:?} — known: [{}], unknown: [{}]",
            self.state,
            self.known_words.join(", "),
            self.unknown_words.join(", ")
        )
    }

    fn trace_success(&self) -> bool {
        true
    }
}

/// Traceable for entity lookup.
pub struct EntityLookupResult {
    pub word: String,
    pub found: bool,
    pub concept_count: usize,
}

impl Traceable for EntityLookupResult {
    fn step(&self) -> PipelineStep {
        PipelineStep::EntityLookup
    }

    fn trace_detail(&self) -> String {
        if self.found {
            format!("{} → {} concept(s)", self.word, self.concept_count)
        } else {
            format!("{} → not found", self.word)
        }
    }

    fn trace_success(&self) -> bool {
        self.found
    }
}

/// Traceable for taxonomy traversal.
pub struct TaxonomyResult {
    pub child: String,
    pub parent: String,
    pub is_a: bool,
}

impl Traceable for TaxonomyResult {
    fn step(&self) -> PipelineStep {
        PipelineStep::TaxonomyTraversal
    }

    fn trace_detail(&self) -> String {
        if self.is_a {
            format!("{} is a {} ✓", self.child, self.parent)
        } else {
            format!("{} is NOT a {} ✗", self.child, self.parent)
        }
    }

    fn trace_success(&self) -> bool {
        // Both positive and negative answers are successful — we have knowledge
        true
    }
}

/// Traceable for NLG realization.
pub struct RealizationResult {
    pub section_count: usize,
}

impl Traceable for RealizationResult {
    fn step(&self) -> PipelineStep {
        PipelineStep::Realization
    }

    fn trace_detail(&self) -> String {
        format!("{} sections generated", self.section_count)
    }

    fn trace_success(&self) -> bool {
        self.section_count > 0
    }
}

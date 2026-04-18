use super::trace_functors::{PipelineStep, Traceable};
use crate::cognitive::linguistics::lambek::reduce::ReductionResult;
use crate::cognitive::linguistics::lambek::reduce::TypedToken;
use crate::cognitive::linguistics::pragmatics::speech_act::SpeechAct;

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
        PipelineStep::TOKENIZE
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
        PipelineStep::PARSE
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
    pub meaning: &'a crate::cognitive::linguistics::lambek::montague::Sem,
}

impl Traceable for InterpretResult<'_> {
    fn step(&self) -> PipelineStep {
        PipelineStep::INTERPRET
    }

    fn trace_detail(&self) -> String {
        use crate::cognitive::linguistics::lambek::montague::Sem;
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
    pub state: crate::cognitive::cognition::epistemics::EpistemicConcept,
    pub known_words: Vec<String>,
    pub unknown_words: Vec<String>,
}

impl Traceable for EpistemicResult {
    fn step(&self) -> PipelineStep {
        PipelineStep::EPISTEMIC_CLASSIFICATION
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
        PipelineStep::ENTITY_LOOKUP
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
        PipelineStep::TAXONOMY_TRAVERSAL
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

/// Traceable for NLG realization — final surface-form production.
///
/// `char_count` is the length of the realised surface utterance. A
/// non-zero count is the success criterion: producing zero characters
/// means the Realization step emitted nothing.
pub struct RealizationResult {
    pub char_count: usize,
}

impl Traceable for RealizationResult {
    fn step(&self) -> PipelineStep {
        PipelineStep::REALIZATION
    }

    fn trace_detail(&self) -> String {
        format!("{} chars generated", self.char_count)
    }

    fn trace_success(&self) -> bool {
        self.char_count > 0
    }
}

/// Traceable for the Plan/SpeechActClassification step — the illocutionary
/// classification of the user's utterance (Searle 1969, Cohen & Perrault 1979).
pub struct SpeechActClassificationResult {
    pub user_act: SpeechAct,
}

impl Traceable for SpeechActClassificationResult {
    fn step(&self) -> PipelineStep {
        PipelineStep::SPEECH_ACT_CLASSIFICATION
    }

    fn trace_detail(&self) -> String {
        format!("{:?}", self.user_act)
    }

    fn trace_success(&self) -> bool {
        // Classification always succeeds — every utterance maps to an illocution.
        true
    }
}

/// Traceable for the Monitor/Metacognition step — the decision branch the
/// metacognitive monitor chose after observing the interpretation result.
pub struct MetacognitionResult {
    pub decision: &'static str,
    pub parsed: bool,
}

impl Traceable for MetacognitionResult {
    fn step(&self) -> PipelineStep {
        PipelineStep::METACOGNITION
    }

    fn trace_detail(&self) -> String {
        self.decision.to_string()
    }

    fn trace_success(&self) -> bool {
        // The monitor always produces a decision; the *parsed* flag reflects
        // whether the upstream Parse step succeeded, which is the relevant
        // signal for whether the chosen branch is a normal-path or repair.
        self.parsed
    }
}

/// Traceable for the full response generation pipeline.
/// Wraps the response text + what ontologies were consulted.
pub struct ResponseResult {
    pub response: String,
    pub entities_found: Vec<String>,
    pub taxonomy_checked: Option<(String, String, bool)>,
    pub from_ontology: bool,
}

impl Traceable for ResponseResult {
    fn step(&self) -> PipelineStep {
        PipelineStep::CONTENT_DETERMINATION
    }

    fn trace_detail(&self) -> String {
        let mut parts = Vec::new();
        if !self.entities_found.is_empty() {
            parts.push(format!("entities: [{}]", self.entities_found.join(", ")));
        }
        if let Some((child, parent, is_a)) = &self.taxonomy_checked {
            if *is_a {
                parts.push(format!("{child} is a {parent} ✓"));
            } else {
                parts.push(format!("{child} is NOT a {parent} ✗"));
            }
        }
        if parts.is_empty() {
            "no ontology data found".into()
        } else {
            parts.join(" → ")
        }
    }

    fn trace_success(&self) -> bool {
        self.from_ontology
    }
}

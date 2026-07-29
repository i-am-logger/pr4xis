#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::trace_functors::{PipelineStep, Traceable};
use crate::cognitive::linguistics::lambek::reduce::ReductionResult;
use crate::cognitive::linguistics::lambek::reduce::TypedToken;
use crate::cognitive::linguistics::pragmatics::realize::DefinitionProvenance;
use crate::cognitive::linguistics::pragmatics::response::ResponseFrame;
use crate::cognitive::linguistics::pragmatics::speech_act::SpeechAct;
use pr4xis::ontology::meta::OntologyName;

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
                ..
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
            Sem::Concept { word, .. } => format!("entity: {word}"),
            Sem::Pred { word, .. } => format!("concept: {word}"),
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
    /// The communicative goal ([`ResponseFrame`]) this turn realized its
    /// `response` from — carried alongside the surface so the "Why?" layer
    /// ([`realize_why`](crate::cognitive::linguistics::pragmatics::realize::realize_why))
    /// can key its plain-language explanation off the SAME frame that produced
    /// the answer, never a re-derived guess. Set through
    /// [`ResponseResult::new`], the one constructor every call site routes
    /// through.
    pub frame: ResponseFrame,
    pub response: String,
    pub entities_found: Vec<String>,
    pub taxonomy_checked: Option<(String, String, bool)>,
    pub from_ontology: bool,
    /// The LOADED ontologies this answer reasoned over, by [`OntologyName`] (doc
    /// §2.3) — empty when the answer came from the embedded substrate. Carried
    /// into the trace so a turn can NAME the loaded `.prx` (e.g. a USC Title) it
    /// drew on, not just the compiled pipeline ontologies.
    pub reasoned_over: Vec<OntologyName>,
    /// Set only by the rule-governed branch of `answer_question`: the
    /// `ConditionalRule` this turn identified plus its currently unfilled
    /// `Required` elements. `None` for every other branch —
    /// `process_with_reasoner` checks this FIRST when deriving `ChatOutcome`,
    /// ahead of the `from_ontology` bool.
    pub conditional: Option<(
        crate::social::judicial::conditional_rule::ConditionalRule,
        pr4xis::category::NonEmpty<crate::social::judicial::conditional_rule::AppliedElement>,
    )>,
    /// Content Grice's Quantity maxim excludes from the primary `response`
    /// (Reiter 1990; Reiter & Dale 2000 Ch.3's Content-Determination stage) —
    /// a multi-hop relational answer's intermediate-rung glosses and
    /// subtypes, deferred here rather than dumped into `response` (a genuine
    /// prior bug: the primary answer to "is a dog an animal" once carried
    /// every rung's full gloss up to "chordate"). `None` when there is
    /// nothing to defer (a direct edge, or a non-relational answer).
    /// Appended to the "Why?" panel by `process_with_reasoner`, never
    /// silently dropped.
    pub deferred_elaboration: Option<String>,
    /// The documents a recited definition was AUTHORED FROM — the loaded
    /// lexicon's own `dcterms:source` edges, carried out of the answer stage so
    /// the explanation layer can state them.
    ///
    /// A third channel, disjoint in meaning from [`reasoned_over`](Self::reasoned_over):
    /// that one names ontologies THIS TURN OPENED, this one names documents
    /// someone else read when writing a gloss. Empty for every turn that recites
    /// no sourced definition.
    pub definition_provenance: Vec<DefinitionProvenance>,
}

impl ResponseResult {
    /// Construct a response result carrying its communicative goal (`frame`) and
    /// realized surface (`response`). Every other field defaults to the
    /// "no extra signal" state (no entities, no taxonomy verdict, not an
    /// ontology-grounded answer, nothing reasoned over, no pending rule) and is
    /// set through a chained builder method below.
    ///
    /// Routing EVERY construction through this one constructor is the blast-radius
    /// mitigation the "Why?" layer needs: a new field (`frame`, and anything the
    /// explanation layer later keys off) lands HERE, not at the ~30 struct-literal
    /// call sites across the chat answer pipeline.
    pub fn new(frame: ResponseFrame, response: String) -> Self {
        Self {
            frame,
            response,
            entities_found: Vec::new(),
            taxonomy_checked: None,
            from_ontology: false,
            reasoned_over: Vec::new(),
            conditional: None,
            deferred_elaboration: None,
            definition_provenance: Vec::new(),
        }
    }

    /// The surfaces this turn named (resolved or not).
    #[must_use]
    pub fn with_entities_found(mut self, entities: Vec<String>) -> Self {
        self.entities_found = entities;
        self
    }

    /// The `(child, parent, holds)` taxonomy/relation verdict this turn checked.
    #[must_use]
    pub fn with_taxonomy_checked(mut self, checked: Option<(String, String, bool)>) -> Self {
        self.taxonomy_checked = checked;
        self
    }

    /// Whether this turn is a genuine ontology-grounded answer (drives
    /// `ChatOutcome::Answered` vs `Abstained`).
    #[must_use]
    pub fn grounded(mut self, from_ontology: bool) -> Self {
        self.from_ontology = from_ontology;
        self
    }

    /// The loaded ontologies this answer reasoned over.
    #[must_use]
    pub fn with_reasoned_over(mut self, reasoned_over: Vec<OntologyName>) -> Self {
        self.reasoned_over = reasoned_over;
        self
    }

    /// The rule-governed pending `(rule, missing)` — set only by the conditional
    /// branch of the answer pipeline.
    #[must_use]
    pub fn with_conditional(
        mut self,
        conditional: Option<(
            crate::social::judicial::conditional_rule::ConditionalRule,
            pr4xis::category::NonEmpty<crate::social::judicial::conditional_rule::AppliedElement>,
        )>,
    ) -> Self {
        self.conditional = conditional;
        self
    }

    /// Content deferred out of the primary `response` under Quantity
    /// (Grice 1975) — see the field doc.
    #[must_use]
    pub fn with_deferred_elaboration(mut self, deferred: Option<String>) -> Self {
        self.deferred_elaboration = deferred;
        self
    }

    /// The documents a recited definition was authored from — see the field doc.
    #[must_use]
    pub fn with_definition_provenance(mut self, sources: Vec<DefinitionProvenance>) -> Self {
        self.definition_provenance = sources;
        self
    }
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

    fn trace_reasoned_over(&self) -> Vec<OntologyName> {
        self.reasoned_over.clone()
    }
}

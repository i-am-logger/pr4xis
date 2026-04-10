use super::ontology::DiagnosticConcept;
use crate::science::information::provenance::ontology::ProvenanceConcept;

// Trace functors — map domain ontology results to diagnostic/provenance records.
//
// Each pipeline stage has a functor TO the Diagnostics ontology.
// The operations stay clean — the functors compose their output
// with the trace ontology.
//
// Categorically: these are functors F: DomainCategory → DiagnosticCategory
// that map domain concepts to diagnostic concepts.
//
// The chat pipeline calls these functors on each result:
//   tokenize_result → LanguageToTrace → TraceContext
//   parse_result → GrammarToTrace → Evidence/Symptom
//   interpret_result → SemanticsToTrace → Hypothesis
//   answer_result → NlgToTrace → Diagnosis/Remedy
//
// References:
// - W3C PROV-O (2013): Activity, Entity, Agent, wasGeneratedBy, used
// - Reiter (1987): the diagnostic cycle applied to computation
// - Joyal-Street-Verity (1996): trace as functor

/// Map a pipeline step to a diagnostic concept.
///
/// Each step in the processing pipeline corresponds to a concept
/// in the Diagnostics ontology — the pipeline IS a diagnostic process.
pub fn pipeline_step_to_diagnostic(step: PipelineStep) -> DiagnosticConcept {
    match step {
        // Input analysis = observation
        PipelineStep::Tokenize => DiagnosticConcept::TraceContext,
        PipelineStep::Parse => DiagnosticConcept::Test,
        PipelineStep::Interpret => DiagnosticConcept::Hypothesis,

        // Knowledge gathering = evidence collection
        PipelineStep::EntityLookup => DiagnosticConcept::Evidence,
        PipelineStep::TaxonomyTraversal => DiagnosticConcept::Evidence,
        PipelineStep::CommonAncestor => DiagnosticConcept::Evidence,

        // Response generation = diagnosis + remedy
        PipelineStep::ContentDetermination => DiagnosticConcept::Diagnosis,
        PipelineStep::DocumentPlanning => DiagnosticConcept::Diagnosis,
        PipelineStep::Realization => DiagnosticConcept::Remedy,

        // Metacognition = residual detection
        PipelineStep::EpistemicClassification => DiagnosticConcept::Residual,
    }
}

/// Map a pipeline step to a provenance concept.
///
/// Each step is a PROV Activity that used inputs and generated outputs.
pub fn pipeline_step_to_provenance(step: PipelineStep) -> ProvenanceConcept {
    match step {
        // All processing steps are Activities
        PipelineStep::Tokenize
        | PipelineStep::Parse
        | PipelineStep::Interpret
        | PipelineStep::EntityLookup
        | PipelineStep::TaxonomyTraversal
        | PipelineStep::CommonAncestor
        | PipelineStep::ContentDetermination
        | PipelineStep::DocumentPlanning
        | PipelineStep::Realization
        | PipelineStep::EpistemicClassification => ProvenanceConcept::Activity,
    }
}

/// The steps in the chat processing pipeline.
///
/// Each step corresponds to an ontology operation.
/// The step knows which ontology it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineStep {
    /// Language ontology: text → typed tokens
    Tokenize,
    /// Lambek Grammar: tokens → parse tree (CYK chart)
    Parse,
    /// Montague Semantics: parse tree → meaning
    Interpret,
    /// WordNet: word → concept IDs
    EntityLookup,
    /// WordNet Taxonomy: concept → is_a chain
    TaxonomyTraversal,
    /// WordNet Taxonomy: two concepts → lowest common ancestor
    CommonAncestor,
    /// NLG (Reiter-Dale): meaning → content selection
    ContentDetermination,
    /// NLG (RST): content → rhetorical structure
    DocumentPlanning,
    /// SVO Grammar: structure → surface text
    Realization,
    /// Epistemics: classify knowledge state (KK/KU/UK/UU)
    EpistemicClassification,
}

impl PipelineStep {
    /// Which ontology does this step belong to?
    pub fn ontology_name(&self) -> &'static str {
        match self {
            Self::Tokenize => "Language (English)",
            Self::Parse => "Lambek Grammar",
            Self::Interpret => "Montague Semantics",
            Self::EntityLookup => "WordNet",
            Self::TaxonomyTraversal => "WordNet Taxonomy",
            Self::CommonAncestor => "WordNet Taxonomy",
            Self::ContentDetermination => "NLG (Reiter-Dale)",
            Self::DocumentPlanning => "Document Planning (RST)",
            Self::Realization => "SVO Grammar",
            Self::EpistemicClassification => "Epistemics",
        }
    }

    /// What operation does this step perform?
    pub fn operation_name(&self) -> &'static str {
        match self {
            Self::Tokenize => "tokenize",
            Self::Parse => "CYK chart parse",
            Self::Interpret => "interpret",
            Self::EntityLookup => "entity lookup",
            Self::TaxonomyTraversal => "is_a traversal",
            Self::CommonAncestor => "LCA search",
            Self::ContentDetermination => "gather knowledge",
            Self::DocumentPlanning => "organize (RST)",
            Self::Realization => "realize",
            Self::EpistemicClassification => "classify",
        }
    }
}

/// A trace entry produced by applying the trace functor to a pipeline result.
///
/// This IS a PROV Activity (W3C PROV-O):
/// - ontology = prov:Agent (who did it)
/// - operation = prov:Activity (what happened)
/// - detail = prov:Entity (what was produced)
/// - status = the postcondition
/// - step = which pipeline step (for ordering)
#[derive(Debug, Clone)]
pub struct PipelineTraceEntry {
    pub step: PipelineStep,
    pub detail: String,
    pub success: bool,
}

impl PipelineTraceEntry {
    /// Apply the trace functor: pipeline result → trace entry.
    pub fn from_step(step: PipelineStep, detail: &str, success: bool) -> Self {
        Self {
            step,
            detail: detail.into(),
            success,
        }
    }

    /// The ontology name (from the PipelineStep).
    pub fn ontology(&self) -> &'static str {
        self.step.ontology_name()
    }

    /// The operation name.
    pub fn operation(&self) -> &'static str {
        self.step.operation_name()
    }

    /// Serialize for JSON transport.
    pub fn serialize(&self) -> String {
        let status = if self.success { "ok" } else { "warn" };
        format!(
            "{}:{}:{}:{}",
            status,
            self.ontology(),
            self.operation(),
            self.detail
        )
    }
}

/// The trace functor trait — maps a pipeline result to a trace entry.
///
/// Each pipeline result type implements this to describe itself
/// for the trace. The functor extracts the detail and success status
/// from the result — the caller doesn't construct the trace manually.
///
/// Categorically: this is a natural transformation from the result
/// type's category to the Diagnostics category.
pub trait Traceable {
    /// Which pipeline step produced this result?
    fn step(&self) -> PipelineStep;
    /// Describe the result for the trace.
    fn trace_detail(&self) -> String;
    /// Did this step succeed?
    fn trace_success(&self) -> bool;
}

/// A complete pipeline trace — the composition of all trace functor applications.
#[derive(Debug, Clone, Default)]
pub struct PipelineTrace {
    pub entries: Vec<PipelineTraceEntry>,
}

impl PipelineTrace {
    /// Apply the trace functor to a pipeline step result and accumulate.
    /// This is the low-level method — prefer `trace_result()` when possible.
    pub fn record(&mut self, step: PipelineStep, detail: &str, success: bool) {
        self.entries
            .push(PipelineTraceEntry::from_step(step, detail, success));
    }

    /// Apply the trace functor to any Traceable result.
    /// The result knows its own step, detail, and success — no manual construction.
    pub fn trace_result(&mut self, result: &dyn Traceable) {
        self.entries.push(PipelineTraceEntry::from_step(
            result.step(),
            &result.trace_detail(),
            result.trace_success(),
        ));
    }

    /// Serialize the full trace for JSON transport.
    pub fn serialize(&self) -> String {
        self.entries
            .iter()
            .map(|e| e.serialize())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::entity::Entity;

    #[test]
    fn pipeline_steps_map_to_diagnostics() {
        // Every step maps to a valid diagnostic concept
        let steps = [
            PipelineStep::Tokenize,
            PipelineStep::Parse,
            PipelineStep::Interpret,
            PipelineStep::EntityLookup,
            PipelineStep::TaxonomyTraversal,
            PipelineStep::CommonAncestor,
            PipelineStep::ContentDetermination,
            PipelineStep::DocumentPlanning,
            PipelineStep::Realization,
            PipelineStep::EpistemicClassification,
        ];
        for step in steps {
            let diag = pipeline_step_to_diagnostic(step);
            assert!(DiagnosticConcept::variants().contains(&diag));
        }
    }

    #[test]
    fn pipeline_steps_map_to_provenance() {
        // All steps map to Activity (they are all PROV Activities)
        for step in [
            PipelineStep::Tokenize,
            PipelineStep::Parse,
            PipelineStep::Interpret,
        ] {
            assert_eq!(
                pipeline_step_to_provenance(step),
                ProvenanceConcept::Activity
            );
        }
    }

    #[test]
    fn trace_entry_from_step() {
        let entry = PipelineTraceEntry::from_step(PipelineStep::Parse, "success → S[q]", true);
        assert_eq!(entry.ontology(), "Lambek Grammar");
        assert_eq!(entry.operation(), "CYK chart parse");
        assert!(entry.success);
    }

    #[test]
    fn pipeline_trace_accumulates() {
        let mut trace = PipelineTrace::default();
        trace.record(PipelineStep::Tokenize, "5 tokens", true);
        trace.record(PipelineStep::Parse, "success → S[q]", true);
        trace.record(PipelineStep::Interpret, "question: is(dog, animal)", true);

        assert_eq!(trace.entries.len(), 3);
        assert_eq!(trace.entries[0].ontology(), "Language (English)");
        assert_eq!(trace.entries[1].ontology(), "Lambek Grammar");
        assert_eq!(trace.entries[2].ontology(), "Montague Semantics");
    }

    #[test]
    fn serialize_format() {
        let entry = PipelineTraceEntry::from_step(PipelineStep::Parse, "failed", false);
        assert_eq!(
            entry.serialize(),
            "warn:Lambek Grammar:CYK chart parse:failed"
        );
    }

    #[test]
    fn every_step_has_ontology_name() {
        let steps = [
            PipelineStep::Tokenize,
            PipelineStep::Parse,
            PipelineStep::Interpret,
            PipelineStep::EntityLookup,
            PipelineStep::TaxonomyTraversal,
            PipelineStep::CommonAncestor,
            PipelineStep::ContentDetermination,
            PipelineStep::DocumentPlanning,
            PipelineStep::Realization,
            PipelineStep::EpistemicClassification,
        ];
        for step in steps {
            assert!(!step.ontology_name().is_empty());
            assert!(!step.operation_name().is_empty());
        }
    }

    // --- The functor preserves the diagnostic cycle ---

    #[test]
    fn input_steps_map_to_observation_phase() {
        // Tokenize → TraceContext (observation context)
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::Tokenize),
            DiagnosticConcept::TraceContext
        );
    }

    #[test]
    fn parse_maps_to_test() {
        // Parsing IS a test — it tests whether the input is grammatical
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::Parse),
            DiagnosticConcept::Test
        );
    }

    #[test]
    fn interpretation_maps_to_hypothesis() {
        // The meaning is a hypothesis about what the user intended
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::Interpret),
            DiagnosticConcept::Hypothesis
        );
    }

    #[test]
    fn knowledge_gathering_maps_to_evidence() {
        // Looking up facts IS gathering evidence
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::EntityLookup),
            DiagnosticConcept::Evidence
        );
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::TaxonomyTraversal),
            DiagnosticConcept::Evidence
        );
    }

    #[test]
    fn realization_maps_to_remedy() {
        // The generated response IS the remedy — it addresses the user's need
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::Realization),
            DiagnosticConcept::Remedy
        );
    }
}

use super::ontology::DiagnosticConcept;
use crate::formal::information::provenance::ontology::ProvenanceConcept;

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

        // Metacognition cycle
        PipelineStep::Metacognition => DiagnosticConcept::Residual,
        PipelineStep::EpistemicClassification => DiagnosticConcept::Residual,

        // Pragmatics
        PipelineStep::SpeechActClassification => DiagnosticConcept::Hypothesis,
        PipelineStep::ResponseFrameSelection => DiagnosticConcept::Diagnosis,

        // Response generation = diagnosis + remedy
        PipelineStep::ContentDetermination => DiagnosticConcept::Diagnosis,
        PipelineStep::DocumentPlanning => DiagnosticConcept::Diagnosis,
        PipelineStep::Realization => DiagnosticConcept::Remedy,
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
        | PipelineStep::Metacognition
        | PipelineStep::SpeechActClassification
        | PipelineStep::ResponseFrameSelection
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
    /// Metacognition: monitor → evaluate → control → repair cycle
    Metacognition,
    /// Pragmatics (Searle): classify the speech act
    SpeechActClassification,
    /// Pragmatics: select response frame from epistemic state
    ResponseFrameSelection,
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
            Self::Metacognition => "Metacognition",
            Self::SpeechActClassification => "Pragmatics (Searle)",
            Self::ResponseFrameSelection => "Pragmatics",
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
            Self::Metacognition => "monitor → evaluate → control",
            Self::SpeechActClassification => "classify speech act",
            Self::ResponseFrameSelection => "select response frame",
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

    /// Get the functor connections for this trace entry.
    pub fn functor_connections(&self) -> Vec<FunctorConnection> {
        functor_connections(self.step)
    }

    /// Serialize with functor connections visible.
    pub fn serialize_with_functors(&self) -> String {
        let base = self.serialize();
        let conns = self.functor_connections();
        if conns.is_empty() {
            return base;
        }
        let chain: Vec<String> = conns
            .iter()
            .map(|c| format!("→{}", c.target_ontology))
            .collect();
        format!("{} [{}]", base, chain.join(", "))
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

/// Functor-derived connection: an ontology that participates via a proven functor.
///
/// When a pipeline step uses ontology X, and functor F: X → Y exists,
/// then Y also participates. This makes the functor connections visible
/// in the trace — showing that communication IS control, DRT IS dialogue, etc.
#[derive(Debug, Clone)]
pub struct FunctorConnection {
    pub target_ontology: &'static str,
    pub functor_name: &'static str,
    pub reference: &'static str,
}

/// For each pipeline step, return the ontologies that participate
/// through proven functor connections.
pub fn functor_connections(step: PipelineStep) -> Vec<FunctorConnection> {
    match step {
        PipelineStep::Tokenize => vec![
            FunctorConnection {
                target_ontology: "Communication (Shannon)",
                functor_name: "NoisyChannel→Communication",
                reference: "Shannon 1948",
            },
            FunctorConnection {
                target_ontology: "Control (Wiener)",
                functor_name: "Communication→Control",
                reference: "Wiener 1948",
            },
        ],
        PipelineStep::Parse => vec![FunctorConnection {
            target_ontology: "Communication (Shannon)",
            functor_name: "NoisyChannel→Communication",
            reference: "Shannon 1948",
        }],
        PipelineStep::Interpret => vec![
            FunctorConnection {
                target_ontology: "DRT (Kamp)",
                functor_name: "Reference→Dialogue",
                reference: "Kamp 1981",
            },
            FunctorConnection {
                target_ontology: "Dialogue (Grosz)",
                functor_name: "Dialogue→Communication",
                reference: "Grosz 1995",
            },
        ],
        PipelineStep::Metacognition | PipelineStep::EpistemicClassification => vec![
            FunctorConnection {
                target_ontology: "Diagnostics (Reiter)",
                functor_name: "Diagnostics→Metacognition",
                reference: "Reiter 1987",
            },
            FunctorConnection {
                target_ontology: "Control (Wiener)",
                functor_name: "Diagnostics→Control",
                reference: "Gertler 1998",
            },
        ],
        PipelineStep::SpeechActClassification | PipelineStep::ResponseFrameSelection => {
            vec![FunctorConnection {
                target_ontology: "Communication (Jakobson)",
                functor_name: "Dialogue→Communication",
                reference: "Jakobson 1960",
            }]
        }
        PipelineStep::ContentDetermination
        | PipelineStep::DocumentPlanning
        | PipelineStep::Realization => vec![FunctorConnection {
            target_ontology: "Communication (Shannon)",
            functor_name: "NLG→Communication",
            reference: "Reiter & Dale 2000",
        }],
        PipelineStep::EntityLookup
        | PipelineStep::TaxonomyTraversal
        | PipelineStep::CommonAncestor => vec![
            FunctorConnection {
                target_ontology: "Schema (Spivak)",
                functor_name: "Systems→Schema",
                reference: "Spivak 2012",
            },
            FunctorConnection {
                target_ontology: "TraceSchema",
                functor_name: "Schema→TraceSchema",
                reference: "Spivak 2012 + PROV-O",
            },
        ],
    }
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

    /// Serialize with functor connections visible in each entry.
    pub fn serialize_with_functors(&self) -> String {
        self.entries
            .iter()
            .map(|e| e.serialize_with_functors())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /// Collect all unique ontology names that participated (direct + via functors).
    pub fn all_participating_ontologies(&self) -> Vec<&'static str> {
        let mut ontologies = Vec::new();
        for entry in &self.entries {
            let name = entry.ontology();
            if !ontologies.contains(&name) {
                ontologies.push(name);
            }
            for conn in entry.functor_connections() {
                if !ontologies.contains(&conn.target_ontology) {
                    ontologies.push(conn.target_ontology);
                }
            }
        }
        ontologies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::entity::Entity;

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

    // --- Functor connection tests ---

    #[test]
    fn tokenize_connects_to_communication() {
        let conns = functor_connections(PipelineStep::Tokenize);
        assert!(
            conns
                .iter()
                .any(|c| c.target_ontology == "Communication (Shannon)")
        );
    }

    #[test]
    fn tokenize_connects_to_control() {
        let conns = functor_connections(PipelineStep::Tokenize);
        assert!(
            conns
                .iter()
                .any(|c| c.target_ontology == "Control (Wiener)")
        );
    }

    #[test]
    fn interpret_connects_to_drt_and_dialogue() {
        let conns = functor_connections(PipelineStep::Interpret);
        assert!(conns.iter().any(|c| c.target_ontology == "DRT (Kamp)"));
        assert!(
            conns
                .iter()
                .any(|c| c.target_ontology == "Dialogue (Grosz)")
        );
    }

    #[test]
    fn metacognition_connects_to_diagnostics_and_control() {
        let conns = functor_connections(PipelineStep::Metacognition);
        assert!(
            conns
                .iter()
                .any(|c| c.target_ontology == "Diagnostics (Reiter)")
        );
        assert!(
            conns
                .iter()
                .any(|c| c.target_ontology == "Control (Wiener)")
        );
    }

    #[test]
    fn all_participating_ontologies_includes_functors() {
        let mut trace = PipelineTrace::default();
        trace.record(PipelineStep::Tokenize, "5 tokens", true);
        trace.record(PipelineStep::Parse, "success", true);
        trace.record(PipelineStep::Interpret, "question", true);
        trace.record(PipelineStep::Metacognition, "classified", true);

        let all = trace.all_participating_ontologies();
        // Direct ontologies
        assert!(all.contains(&"Language (English)"));
        assert!(all.contains(&"Lambek Grammar"));
        assert!(all.contains(&"Montague Semantics"));
        assert!(all.contains(&"Metacognition"));
        // Via functors
        assert!(all.contains(&"Communication (Shannon)"));
        assert!(all.contains(&"Control (Wiener)"));
        assert!(all.contains(&"DRT (Kamp)"));
        assert!(all.contains(&"Dialogue (Grosz)"));
        assert!(all.contains(&"Diagnostics (Reiter)"));
    }

    #[test]
    fn serialize_with_functors_shows_connections() {
        let entry = PipelineTraceEntry::from_step(PipelineStep::Tokenize, "5 tokens", true);
        let s = entry.serialize_with_functors();
        assert!(s.contains("→Communication (Shannon)"));
        assert!(s.contains("→Control (Wiener)"));
    }

    #[test]
    fn every_step_has_functor_connections() {
        // Every pipeline step should connect to at least one other ontology via functor
        let steps = [
            PipelineStep::Tokenize,
            PipelineStep::Parse,
            PipelineStep::Interpret,
            PipelineStep::EntityLookup,
            PipelineStep::TaxonomyTraversal,
            PipelineStep::Metacognition,
            PipelineStep::SpeechActClassification,
            PipelineStep::ContentDetermination,
            PipelineStep::Realization,
            PipelineStep::EpistemicClassification,
        ];
        for step in steps {
            let conns = functor_connections(step);
            assert!(
                !conns.is_empty(),
                "{:?} should have functor connections",
                step
            );
        }
    }
}

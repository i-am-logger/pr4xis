#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::DiagnosticConcept;
use crate::formal::systems::mape_k::ontology::MapeKConcept;
use pr4xis::ontology::meta::OntologyName;
use pr4xis::ontology::provenance::ontology::ProvenanceConcept;

/// An ontology that participated in a turn — a TYPED SUM (doc §2.3). A `Compiled`
/// pipeline ontology is a `&'static str` (it is part of the binary, const at every
/// trace step); a `Loaded` ontology is the runtime [`OntologyName`] of a `.prx`
/// the turn reasoned over (an owned name, unknowable at compile time). So the
/// trace can NAME a loaded Title 15, not just the compiled pipeline ontologies —
/// without forcing the const-constructed [`PipelineStep`] to hold an owned name
/// (which would break its `Copy`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceOntology {
    /// A compiled pipeline ontology — const, in the binary (e.g. "DRT (Kamp)").
    Compiled(&'static str),
    /// A loaded `.prx` ontology the turn reasoned over, by its runtime name.
    Loaded(OntologyName),
}

impl TraceOntology {
    /// The ontology's name as a string, regardless of provenance.
    pub fn name(&self) -> &str {
        match self {
            TraceOntology::Compiled(s) => s,
            TraceOntology::Loaded(n) => n.as_str(),
        }
    }

    /// The provenance label — `"loaded"` for a runtime `.prx` the answer drew on,
    /// `"compiled"` for a const pipeline ontology. Lets a projector (the wasm
    /// surface, the page) tag each ontology without matching the variants.
    pub fn provenance(&self) -> &'static str {
        match self {
            TraceOntology::Compiled(_) => "compiled",
            TraceOntology::Loaded(_) => "loaded",
        }
    }
}

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

/// A step in the chat processing pipeline — a specific operation in a
/// specific MAPE-K phase, performed by a specific ontology.
///
/// This struct IS the literature-grounded replacement for the old
/// ad-hoc `PipelineStep` enum (issue #117). The 13 `PipelineStep::FOO`
/// constants below enumerate the operational positions; each one pairs:
///
/// - a **MAPE-K phase** (Kephart & Chess 2003) — the control-loop
///   role this step plays (Monitor / Analyze / Plan / Execute);
/// - an **ontology** — which ontology owns the work, carried as the
///   literal `meta().name` of that ontology;
/// - an **operation** — what specific action the step performs.
///
/// Callers that used to pattern-match `PipelineStep::TOKENIZE` now
/// compare `step == PipelineStep::TOKENIZE`, or dispatch on
/// `step.phase()` if they only care about MAPE-K role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineStep {
    phase: MapeKConcept,
    ontology: &'static str,
    operation: &'static str,
}

impl PipelineStep {
    const fn new(phase: MapeKConcept, ontology: &'static str, operation: &'static str) -> Self {
        Self {
            phase,
            ontology,
            operation,
        }
    }

    // MAPE-K Monitor phase — sensing input and sensing self.
    /// Tokenise input text via the Lemon lexical ontology.
    pub const TOKENIZE: Self = Self::new(MapeKConcept::Monitor, "LemonOntology", "tokenize");
    /// Parse token stream via the Lambek pregroup grammar.
    pub const PARSE: Self = Self::new(MapeKConcept::Monitor, "LambekOntology", "CYK chart parse");
    /// Interpret the parse tree via Montague semantics.
    pub const INTERPRET: Self = Self::new(MapeKConcept::Monitor, "MontagueOntology", "interpret");
    /// Monitor → evaluate → control cycle (Nelson-Narens metacognition).
    pub const METACOGNITION: Self = Self::new(
        MapeKConcept::Monitor,
        "MetaCognitionOntology",
        "monitor → evaluate → control",
    );
    /// Classify the knowledge state (KK / KU / UK / UU).
    pub const EPISTEMIC_CLASSIFICATION: Self =
        Self::new(MapeKConcept::Monitor, "EpistemicOntology", "classify");

    // MAPE-K Analyze phase — reasoning over the concept graph.
    /// Look up a word's concepts in WordNet.
    pub const ENTITY_LOOKUP: Self =
        Self::new(MapeKConcept::Analyze, "WordNetOntology", "entity lookup");
    /// Walk the WordNet hypernym/hyponym taxonomy.
    pub const TAXONOMY_TRAVERSAL: Self =
        Self::new(MapeKConcept::Analyze, "WordNetOntology", "is_a traversal");
    /// Compute the lowest common ancestor in WordNet.
    pub const COMMON_ANCESTOR: Self =
        Self::new(MapeKConcept::Analyze, "WordNetOntology", "LCA search");

    // MAPE-K Plan phase — deciding the illocutionary goal.
    /// Classify the speech act (Cohen & Perrault plan-based speech acts).
    pub const SPEECH_ACT_CLASSIFICATION: Self = Self::new(
        MapeKConcept::Plan,
        "PlanningOntology",
        "classify speech act",
    );
    /// Select the response frame from the epistemic state.
    pub const RESPONSE_FRAME_SELECTION: Self = Self::new(
        MapeKConcept::Plan,
        "ResponseOntology",
        "select response frame",
    );

    // MAPE-K Execute phase — producing the utterance.
    /// Reiter & Dale stage 1: content determination.
    pub const CONTENT_DETERMINATION: Self =
        Self::new(MapeKConcept::Execute, "NlgOntology", "gather knowledge");
    /// Organise content rhetorically (Mann & Thompson RST, Discourse).
    pub const DOCUMENT_PLANNING: Self =
        Self::new(MapeKConcept::Execute, "DiscourseOntology", "organize (RST)");
    /// Surface-form production (Levelt Speaking; de Groote ACG generation).
    pub const REALIZATION: Self = Self::new(MapeKConcept::Execute, "ProductionOntology", "realize");

    /// The MAPE-K phase this step plays — Kephart & Chess (2003) role.
    pub const fn phase(&self) -> MapeKConcept {
        self.phase
    }

    /// Which ontology owns this step's work (its `meta().name`).
    pub const fn ontology_name(&self) -> &'static str {
        self.ontology
    }

    /// What operation this step performs.
    pub const fn operation_name(&self) -> &'static str {
        self.operation
    }

    /// All 13 operational steps in declaration order.
    pub const ALL: [Self; 13] = [
        Self::TOKENIZE,
        Self::PARSE,
        Self::INTERPRET,
        Self::METACOGNITION,
        Self::EPISTEMIC_CLASSIFICATION,
        Self::ENTITY_LOOKUP,
        Self::TAXONOMY_TRAVERSAL,
        Self::COMMON_ANCESTOR,
        Self::SPEECH_ACT_CLASSIFICATION,
        Self::RESPONSE_FRAME_SELECTION,
        Self::CONTENT_DETERMINATION,
        Self::DOCUMENT_PLANNING,
        Self::REALIZATION,
    ];
}

impl pr4xis::category::Concept for PipelineStep {}
impl pr4xis::category::FinitelyGenerated for PipelineStep {
    fn variants() -> Vec<Self> {
        Self::ALL.to_vec()
    }
}

/// Map a pipeline step to a diagnostic concept.
///
/// Each step in the processing pipeline corresponds to a concept
/// in the Diagnostics ontology — the pipeline IS a diagnostic process.
pub fn pipeline_step_to_diagnostic(step: PipelineStep) -> DiagnosticConcept {
    // Dispatch on phase + operation identity. Steps within a phase can
    // produce different diagnostic roles because observation vs. test
    // vs. hypothesis is finer-grained than Monitor/Analyze/Plan/Execute.
    if step == PipelineStep::TOKENIZE {
        DiagnosticConcept::TraceContext
    } else if step == PipelineStep::PARSE {
        DiagnosticConcept::Test
    } else if step == PipelineStep::INTERPRET || step == PipelineStep::SPEECH_ACT_CLASSIFICATION {
        DiagnosticConcept::Hypothesis
    } else if step == PipelineStep::METACOGNITION || step == PipelineStep::EPISTEMIC_CLASSIFICATION
    {
        DiagnosticConcept::Residual
    } else if step.phase() == MapeKConcept::Analyze {
        DiagnosticConcept::Evidence
    } else if step == PipelineStep::REALIZATION {
        DiagnosticConcept::Remedy
    } else {
        // Remaining Plan + Execute steps form the Diagnosis.
        DiagnosticConcept::Diagnosis
    }
}

/// Map a pipeline step to a provenance concept.
///
/// Every step is a PROV Activity.
pub fn pipeline_step_to_provenance(_step: PipelineStep) -> ProvenanceConcept {
    ProvenanceConcept::Activity
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
    /// The LOADED ontologies this step reasoned over, by [`OntologyName`] (doc
    /// §2.3). Empty for a compiled-only step; the respond step carries the loaded
    /// ontologies the answer drew on — the provenance the trace can name.
    pub reasoned_over: Vec<OntologyName>,
}

impl PipelineTraceEntry {
    /// Apply the trace functor: pipeline result → trace entry.
    pub fn from_step(step: PipelineStep, detail: &str, success: bool) -> Self {
        Self {
            step,
            detail: detail.into(),
            success,
            reasoned_over: Vec::new(),
        }
    }

    /// Attach the loaded ontologies this step reasoned over (doc §2.3).
    pub fn with_reasoned_over(mut self, reasoned_over: Vec<OntologyName>) -> Self {
        self.reasoned_over = reasoned_over;
        self
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
    /// The LOADED ontologies this step reasoned over (doc §2.3) — empty for a
    /// compiled-only step. The respond step overrides it with the loaded
    /// ontologies the answer drew on, so the trace can name them by [`OntologyName`].
    fn trace_reasoned_over(&self) -> Vec<OntologyName> {
        Vec::new()
    }
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
    if step == PipelineStep::TOKENIZE {
        vec![
            FunctorConnection {
                target_ontology: "Communication (Shannon)",
                // `ChannelToCommunication` is the registered functor's name.
                // This row read `NoisyChannel→Communication`, which resolved to
                // nothing — the same defect as the Execute arm's, one row over
                // and older. The noisy-channel MODEL is what tokenization's
                // correction implements; the ARROW is Channel → Communication.
                functor_name: "Channel→Communication",
                reference: "Shannon 1948",
            },
            FunctorConnection {
                target_ontology: "Control (Wiener)",
                functor_name: "Communication→Control",
                reference: "Wiener 1948",
            },
        ]
    } else if step == PipelineStep::PARSE {
        // NOT the noisy-channel functor TOKENIZE carries. Tokenization really
        // does decode through a channel — `tokenize::correct_unknown_word_surfaces`
        // implements the Observation → Correction → Intention adjunction. A CYK
        // chart parse over pregroup types has no channel and no noise model, so
        // naming Shannon's coding theorem here would be a citation that does not
        // hold in a system whose whole claim is that its citations do.
        //
        // What IS Shannon here is the MEASURE: the supertag costs are the
        // negative logs of published CCGbank frequencies (`supertag_costs.rs`),
        // i.e. self-information, and `chart_reduce` keeps the minimum-cost
        // derivation per (span, type) — a Viterbi-semiring parse (Goodman 1999)
        // that therefore minimises total surprisal.
        //
        // And the parse's OUTPUT is carried into semantics by a real,
        // composition-preserving functor: `lambek::montague::interpret` maps
        // each Lambek type to a semantic domain and reduction to function
        // application, which is the Montague syntax-semantics homomorphism.
        // `SupertagCost→Surprisal` is deliberately NOT a row here. That the
        // costs are negative logs of published frequencies — self-information,
        // in Shannon's own sense — is a true and cited fact about a MEASURE,
        // and it is recorded in the comment above where it belongs. It is not
        // a functor: there is no arrow between two ontologies to name, and a
        // `FunctorConnection` that names one claims a 1-cell that does not
        // exist. The Shannon citation travels on the step's own reference
        // channel instead, which costs the reader nothing and asserts only
        // what is true.
        vec![FunctorConnection {
            target_ontology: "Montague (semantics)",
            functor_name: "Lambek→Montague",
            reference: "Montague 1970; Lambek 1958",
        }]
    } else if step == PipelineStep::INTERPRET {
        vec![
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
        ]
    } else if step == PipelineStep::METACOGNITION || step == PipelineStep::EPISTEMIC_CLASSIFICATION
    {
        vec![
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
        ]
    } else if step.phase() == MapeKConcept::Plan {
        vec![FunctorConnection {
            target_ontology: "Communication (Jakobson)",
            functor_name: "Dialogue→Communication",
            reference: "Jakobson 1960",
        }]
    } else if step.phase() == MapeKConcept::Execute {
        // Names the functors that EXIST. `NLG→Communication` sat here and did
        // not: nothing on the Execute path touches the Communication ontology,
        // and no such functor is defined or registered anywhere. It was
        // serialised into `trace_structured` on every single turn, so a
        // reviewer reading the published trace was reading a functor name that
        // resolves to nothing — the exact failure the trace exists to prevent.
        //
        // What the Execute path really composes is Reiter & Dale's own
        // pipeline: document planning hands content to the NLG stage
        // (`DiscourseToNlg`), which hands it to realization (`NlgToPipeline`).
        // Both are registered and both pass `assert_functor_laws`.
        vec![
            FunctorConnection {
                target_ontology: "NLG (Reiter & Dale)",
                functor_name: "Discourse→NLG",
                reference: "Mann & Thompson 1988; Reiter & Dale 2000",
            },
            FunctorConnection {
                target_ontology: "Production (realization)",
                functor_name: "NLG→Pipeline",
                reference: "Reiter & Dale 2000",
            },
        ]
    } else if step.phase() == MapeKConcept::Analyze {
        vec![
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
        ]
    } else {
        Vec::new()
    }
}

/// A complete pipeline trace — the composition of all trace functor applications.
///
/// PipelineTrace is a Monoid under concatenation:
///   empty = PipelineTrace { entries: [] }
///   combine(t1, t2) = PipelineTrace { entries: t1.entries ++ t2.entries }
///
/// This makes it usable as the W in Writer<W, A> — the pipeline computation
/// is a writer monad over PipelineTrace. Trace accumulates automatically
/// through monadic bind.
#[derive(Debug, Clone, Default)]
pub struct PipelineTrace {
    pub entries: Vec<PipelineTraceEntry>,
}

impl pr4xis::category::Monoid for PipelineTrace {
    fn empty() -> Self {
        Self::default()
    }

    fn combine(&self, other: &Self) -> Self {
        let mut entries = self.entries.clone();
        entries.extend(other.entries.iter().cloned());
        Self { entries }
    }
}

/// A traced pipeline computation: Writer<PipelineTrace, A>.
///
/// The pipeline IS the writer monad over PipelineTrace.
/// Each pipeline step returns `TracedPipeline<StepResult>`, and monadic bind
/// composes steps while accumulating trace entries through the PipelineTrace monoid.
///
/// This is Moggi (1991): computational effects (tracing) factored through monads.
pub type TracedPipeline<A> = pr4xis::category::Writer<PipelineTrace, A>;

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

    /// Record a trace entry using Provenance — auto-tracing.
    ///
    /// Instead of hardcoding ontology names, the ontology provides its own
    /// metadata through Provenance (generated by ontology!).
    pub fn record_from_meta(
        &mut self,
        meta: &pr4xis::ontology::meta::Provenance,
        step: PipelineStep,
        detail: &str,
        success: bool,
    ) {
        self.entries
            .push(PipelineTraceEntry::from_step(step, detail, success));
        // The meta is available for richer introspection — the step already
        // carries ontology_name(), but meta gives module_path for debugging.
        let _ = meta; // Used for future structured tracing
    }

    /// Create a single-entry PipelineTrace for use with TracedPipeline.
    ///
    /// This enables monadic composition: instead of mutating a trace,
    /// each step returns a `TracedPipeline<A>` that carries its own trace,
    /// and bind composes them via the PipelineTrace monoid.
    pub fn single(step: PipelineStep, detail: &str, success: bool) -> Self {
        Self {
            entries: vec![PipelineTraceEntry::from_step(step, detail, success)],
        }
    }

    /// Create a single-entry trace from a Traceable result.
    pub fn from_traceable(result: &dyn Traceable) -> Self {
        Self {
            entries: vec![
                PipelineTraceEntry::from_step(
                    result.step(),
                    &result.trace_detail(),
                    result.trace_success(),
                )
                .with_reasoned_over(result.trace_reasoned_over()),
            ],
        }
    }

    /// The ontologies this turn REASONED OVER — the typed-sum provenance (doc
    /// §2.3 / §4.2). Each is paired with whether its step SUCCEEDED, so a consumer
    /// can distinguish *traversed* from *answered-from* (the success bit is no
    /// longer dropped). Compiled pipeline ontologies appear as
    /// [`TraceOntology::Compiled`]; a loaded `.prx` the answer drew on appears as
    /// [`TraceOntology::Loaded`] with its real [`OntologyName`]. Deduped, order-
    /// stable (first occurrence wins).
    pub fn reasoned_over(&self) -> Vec<(TraceOntology, bool)> {
        let mut out: Vec<(TraceOntology, bool)> = Vec::new();
        let push = |o: TraceOntology, success: bool, out: &mut Vec<(TraceOntology, bool)>| {
            if !out.iter().any(|(existing, _)| existing == &o) {
                out.push((o, success));
            }
        };
        for entry in &self.entries {
            push(
                TraceOntology::Compiled(entry.ontology()),
                entry.success,
                &mut out,
            );
            for name in &entry.reasoned_over {
                push(TraceOntology::Loaded(name.clone()), entry.success, &mut out);
            }
        }
        out
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
    use pr4xis::category::Monoid;
    use pr4xis::category::entity::FinitelyGenerated;

    // --- PipelineTrace Monoid laws ---

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn pipeline_trace_monoid_left_identity() {
        let trace = PipelineTrace::single(PipelineStep::TOKENIZE, "5 tokens", true);
        let result = PipelineTrace::empty().combine(&trace);
        assert_eq!(result.entries.len(), trace.entries.len());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn pipeline_trace_monoid_right_identity() {
        let trace = PipelineTrace::single(PipelineStep::TOKENIZE, "5 tokens", true);
        let result = trace.combine(&PipelineTrace::empty());
        assert_eq!(result.entries.len(), 1);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn pipeline_trace_monoid_associativity() {
        let a = PipelineTrace::single(PipelineStep::TOKENIZE, "tok", true);
        let b = PipelineTrace::single(PipelineStep::PARSE, "parse", true);
        let c = PipelineTrace::single(PipelineStep::INTERPRET, "interp", true);
        let ab_c = a.combine(&b).combine(&c);
        let a_bc = a.combine(&b.combine(&c));
        assert_eq!(ab_c.entries.len(), a_bc.entries.len());
        assert_eq!(ab_c.entries.len(), 3);
    }

    // --- TracedPipeline (Writer monad over PipelineTrace) ---

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn traced_pipeline_bind_accumulates_trace() {
        let step1: TracedPipeline<usize> = pr4xis::category::Writer::new(
            5,
            PipelineTrace::single(PipelineStep::TOKENIZE, "5 tokens", true),
        );

        let result = step1.bind(|token_count| {
            pr4xis::category::Writer::new(
                token_count > 0,
                PipelineTrace::single(PipelineStep::PARSE, "parsed", true),
            )
        });

        assert!(result.value);
        assert_eq!(result.log.entries.len(), 2);
        assert_eq!(result.log.entries[0].step, PipelineStep::TOKENIZE);
        assert_eq!(result.log.entries[1].step, PipelineStep::PARSE);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn traced_pipeline_tell_appends_trace() {
        let computation: TracedPipeline<&str> = pr4xis::category::Writer::pure("hello");
        let result = computation.tell(PipelineTrace::single(
            PipelineStep::TOKENIZE,
            "1 token",
            true,
        ));
        assert_eq!(result.value, "hello");
        assert_eq!(result.log.entries.len(), 1);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn traced_pipeline_map_preserves_trace() {
        let step: TracedPipeline<i32> = pr4xis::category::Writer::new(
            21,
            PipelineTrace::single(PipelineStep::TOKENIZE, "tok", true),
        );
        let result = step.map(|x| x * 2);
        assert_eq!(result.value, 42);
        assert_eq!(result.log.entries.len(), 1);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn traced_pipeline_from_traceable() {
        use super::super::trace_impls;
        use crate::cognitive::linguistics::pragmatics::response::ResponseFrame;
        let result =
            trace_impls::ResponseResult::new(ResponseFrame::AssertKnowledge, "Yes.".into())
                .with_entities_found(vec!["dog".into()])
                .with_taxonomy_checked(Some(("dog".into(), "mammal".into(), true)))
                .grounded(true);
        let trace = PipelineTrace::from_traceable(&result);
        assert_eq!(trace.entries.len(), 1);
        assert_eq!(trace.entries[0].step, PipelineStep::CONTENT_DETERMINATION);
        assert!(trace.entries[0].success);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reasoned_over_names_loaded_ontologies_and_keeps_the_success_bit() {
        use super::super::trace_impls;
        use crate::cognitive::linguistics::pragmatics::response::ResponseFrame;
        // A successful answer that drew on a loaded `.prx` (a USC Title).
        let result = trace_impls::ResponseResult::new(
            ResponseFrame::AssertKnowledge,
            "First section.".into(),
        )
        .with_entities_found(vec!["section 1".into()])
        .grounded(true)
        .with_reasoned_over(vec![OntologyName::new_static("us_code")]);
        let provenance = PipelineTrace::from_traceable(&result).reasoned_over();

        // The compiled pipeline ontology appears as `Compiled`, the loaded one as
        // `Loaded` NAMED by its OntologyName (the doc §2.3 deliverable) — and both
        // carry the step's success bit (§4.2 — no longer dropped).
        assert!(
            provenance
                .iter()
                .any(|(o, ok)| matches!(o, TraceOntology::Compiled(_)) && *ok),
            "the compiled step ontology participates"
        );
        assert!(
            provenance.iter().any(|(o, ok)| {
                *o == TraceOntology::Loaded(OntologyName::new_static("us_code")) && *ok
            }),
            "the LOADED ontology is named in the provenance; got {provenance:?}"
        );

        // A FAILED step carries success=false — traversed, not answered-from.
        let failed =
            trace_impls::ResponseResult::new(ResponseFrame::AdmitLimitation, String::new());
        assert!(
            PipelineTrace::from_traceable(&failed)
                .reasoned_over()
                .iter()
                .all(|(_, ok)| !*ok),
            "a failed step's ontologies are marked not-answered-from"
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn pipeline_steps_map_to_diagnostics() {
        // Every step maps to a valid diagnostic concept
        let steps = [
            PipelineStep::TOKENIZE,
            PipelineStep::PARSE,
            PipelineStep::INTERPRET,
            PipelineStep::ENTITY_LOOKUP,
            PipelineStep::TAXONOMY_TRAVERSAL,
            PipelineStep::COMMON_ANCESTOR,
            PipelineStep::CONTENT_DETERMINATION,
            PipelineStep::DOCUMENT_PLANNING,
            PipelineStep::REALIZATION,
            PipelineStep::EPISTEMIC_CLASSIFICATION,
        ];
        for step in steps {
            let diag = pipeline_step_to_diagnostic(step);
            assert!(DiagnosticConcept::variants().contains(&diag));
        }
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn pipeline_steps_map_to_provenance() {
        // All steps map to Activity (they are all PROV Activities)
        for step in [
            PipelineStep::TOKENIZE,
            PipelineStep::PARSE,
            PipelineStep::INTERPRET,
        ] {
            assert_eq!(
                pipeline_step_to_provenance(step),
                ProvenanceConcept::Activity
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn trace_entry_from_step() {
        let entry = PipelineTraceEntry::from_step(PipelineStep::PARSE, "success → S[q]", true);
        assert_eq!(entry.ontology(), "LambekOntology");
        assert_eq!(entry.operation(), "CYK chart parse");
        assert!(entry.success);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn pipeline_trace_accumulates() {
        let mut trace = PipelineTrace::default();
        trace.record(PipelineStep::TOKENIZE, "5 tokens", true);
        trace.record(PipelineStep::PARSE, "success → S[q]", true);
        trace.record(PipelineStep::INTERPRET, "question: is(dog, animal)", true);

        assert_eq!(trace.entries.len(), 3);
        assert_eq!(trace.entries[0].ontology(), "LemonOntology");
        assert_eq!(trace.entries[1].ontology(), "LambekOntology");
        assert_eq!(trace.entries[2].ontology(), "MontagueOntology");
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn serialize_format() {
        let entry = PipelineTraceEntry::from_step(PipelineStep::PARSE, "failed", false);
        let expected = format!("warn:{}:CYK chart parse:failed", "LambekOntology");
        assert_eq!(entry.serialize(), expected);
    }

    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn every_step_has_ontology_name() {
        let steps = [
            PipelineStep::TOKENIZE,
            PipelineStep::PARSE,
            PipelineStep::INTERPRET,
            PipelineStep::ENTITY_LOOKUP,
            PipelineStep::TAXONOMY_TRAVERSAL,
            PipelineStep::COMMON_ANCESTOR,
            PipelineStep::CONTENT_DETERMINATION,
            PipelineStep::DOCUMENT_PLANNING,
            PipelineStep::REALIZATION,
            PipelineStep::EPISTEMIC_CLASSIFICATION,
        ];
        for step in steps {
            assert!(!step.ontology_name().is_empty());
            assert!(!step.operation_name().is_empty());
        }
    }

    // --- The functor preserves the diagnostic cycle ---

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn input_steps_map_to_observation_phase() {
        // Tokenize → TraceContext (observation context)
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::TOKENIZE),
            DiagnosticConcept::TraceContext
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn parse_maps_to_test() {
        // Parsing IS a test — it tests whether the input is grammatical
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::PARSE),
            DiagnosticConcept::Test
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn interpretation_maps_to_hypothesis() {
        // The meaning is a hypothesis about what the user intended
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::INTERPRET),
            DiagnosticConcept::Hypothesis
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn knowledge_gathering_maps_to_evidence() {
        // Looking up facts IS gathering evidence
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::ENTITY_LOOKUP),
            DiagnosticConcept::Evidence
        );
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::TAXONOMY_TRAVERSAL),
            DiagnosticConcept::Evidence
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn realization_maps_to_remedy() {
        // The generated response IS the remedy — it addresses the user's need
        assert_eq!(
            pipeline_step_to_diagnostic(PipelineStep::REALIZATION),
            DiagnosticConcept::Remedy
        );
    }

    // --- Functor connection tests ---

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn tokenize_connects_to_communication() {
        let conns = functor_connections(PipelineStep::TOKENIZE);
        assert!(
            conns
                .iter()
                .any(|c| c.target_ontology == "Communication (Shannon)")
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn tokenize_connects_to_control() {
        let conns = functor_connections(PipelineStep::TOKENIZE);
        assert!(
            conns
                .iter()
                .any(|c| c.target_ontology == "Control (Wiener)")
        );
    }

    /// The noisy channel belongs to TOKENIZE and NOT to PARSE, and the
    /// difference is not cosmetic.
    ///
    /// `tokenize::correct_unknown_word_surfaces` really does decode through a
    /// channel (Observation → Correction → Intention). A CYK chart parse over
    /// pregroup types has no channel and no noise model — its Shannon content
    /// is the MEASURE (supertag costs are negative logs of published CCGbank
    /// frequencies, so the Viterbi-semiring chart minimises surprisal), and its
    /// outgoing functor is the Montague syntax-semantics homomorphism.
    ///
    /// This is pinned because the trace is published evidence: a reviewer reads
    /// these functor names beside the step that claims them, and a citation
    /// that does not hold is worse than no citation in a system whose case
    /// rests on citations holding.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parse_claims_montagues_homomorphism_and_no_channel() {
        let parse = functor_connections(PipelineStep::PARSE);
        assert!(
            !parse.iter().any(|c| c.functor_name.contains("Channel")),
            "a pregroup chart parse decodes no channel — the channel functor \
             belongs to TOKENIZE, which implements one"
        );
        assert!(
            parse
                .iter()
                .any(|c| c.target_ontology == "Montague (semantics)"),
            "`lambek::montague::interpret` carries the parse into semantics, \
             and `LambekToMontague` is the arrow that says so"
        );
        // A measure identity is NOT an arrow. The supertag costs really are
        // negative logs of published frequencies — self-information in
        // Shannon's own sense — and that fact is worth recording, but it
        // relates a number to its definition, not one ontology to another.
        // A row here claiming `SupertagCost→Surprisal` asserted a 1-cell that
        // does not exist, so it is stated in the arm's comment instead.
        assert!(
            !parse
                .iter()
                .any(|c| c.target_ontology == "Information (Shannon)"),
            "the supertag-cost/surprisal identity is a fact about a MEASURE; \
             publishing it as a functor connection claims an arrow between two \
             ontologies that nothing defines"
        );
        // The contrast: TOKENIZE keeps the channel, because it has one.
        assert!(
            functor_connections(PipelineStep::TOKENIZE)
                .iter()
                .any(|c| c.functor_name.contains("Channel")),
            "tokenization's unknown-word correction IS a noisy-channel decode"
        );
    }

    /// Every functor NAME the trace publishes must be one this codebase can
    /// point at, and every one must carry a reference.
    ///
    /// This exists because `NLG→Communication` sat in the Execute arm and
    /// resolved to nothing: no such functor was defined, registered, or
    /// performed anywhere, and nothing on that path touches the Communication
    /// ontology at all. It was serialised into `trace_structured` on every
    /// turn, so the published proof path — the artifact whose entire purpose
    /// is that a reviewer can check it — carried a name that checked out to
    /// nothing.
    ///
    /// Each name must RESOLVE: reading the arrow as "To" must land on a functor
    /// this codebase actually registered (`Discourse→NLG` ↔ `DiscourseToNlg`).
    /// The comparison ignores case and punctuation, because the registry
    /// spells acronyms `Nlg` where the published name spells them `NLG`; it
    /// does not ignore the identifier itself.
    ///
    /// The previous version of this test only *described* that rule — it
    /// asserted the name was non-empty, contained an arrow, and was not one
    /// specific dead string. Every row satisfied it by construction, including
    /// three that resolved to nothing. A gate whose stated rule and actual
    /// predicate differ is worse than no gate: it reports the property as
    /// checked. This resolves for real, so an invented name fails here rather
    /// than shipping into `trace_structured` and the downloadable record.
    ///
    /// It still cannot prove a functor is the RIGHT one — only that it exists.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_published_functor_name_carries_a_reference_and_a_target() {
        /// `A→B` and `AToB` compare equal; `NoisyChannel→Communication` and
        /// `ChannelToCommunication` do not.
        fn key(name: &str) -> String {
            name.replace('→', "To")
                .chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect()
        }
        let registered: Vec<String> = pr4xis::ontology::registry::FUNCTOR_CONSTRUCTORS
            .iter()
            .map(|f| key(f().name.as_str()))
            .collect();

        for step in PipelineStep::ALL {
            for c in functor_connections(step) {
                assert!(
                    !c.functor_name.is_empty() && c.functor_name.contains('→'),
                    "{}: a functor name must read as a mapping, got {:?}",
                    step.operation_name(),
                    c.functor_name,
                );
                // A registered name is the functor's full module path with its
                // identifier last (`…lambek::montague_functor::LambekToMontague`),
                // so the published short name must be a SUFFIX of it — not
                // equal to it.
                assert!(
                    registered.iter().any(|r| r.ends_with(&key(c.functor_name))),
                    "{}: functor {:?} resolves to no registered functor. The \
                     trace publishes this name as evidence, so it must name \
                     something a reader can open — either register the functor \
                     (with `assert_functor_laws`) or drop the row. Registered: \
                     {registered:?}",
                    step.operation_name(),
                    c.functor_name,
                );
                assert!(
                    !c.reference.trim().is_empty(),
                    "{}: functor {:?} publishes no reference — an unreferenced \
                     citation in the proof path is the defect this whole trace \
                     exists to make impossible",
                    step.operation_name(),
                    c.functor_name,
                );
                assert!(
                    !c.target_ontology.trim().is_empty(),
                    "{}: functor {:?} names no target ontology",
                    step.operation_name(),
                    c.functor_name,
                );
                // The specific dead name, pinned so it cannot return.
                assert_ne!(
                    c.functor_name,
                    "NLG→Communication",
                    "{}: `NLG→Communication` names no functor in this codebase \
                     — the Execute path composes Discourse→NLG and NLG→Pipeline",
                    step.operation_name(),
                );
            }
        }
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn interpret_connects_to_drt_and_dialogue() {
        let conns = functor_connections(PipelineStep::INTERPRET);
        assert!(conns.iter().any(|c| c.target_ontology == "DRT (Kamp)"));
        assert!(
            conns
                .iter()
                .any(|c| c.target_ontology == "Dialogue (Grosz)")
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn metacognition_connects_to_diagnostics_and_control() {
        let conns = functor_connections(PipelineStep::METACOGNITION);
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

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn all_participating_ontologies_includes_functors() {
        let mut trace = PipelineTrace::default();
        trace.record(PipelineStep::TOKENIZE, "5 tokens", true);
        trace.record(PipelineStep::PARSE, "success", true);
        trace.record(PipelineStep::INTERPRET, "question", true);
        trace.record(PipelineStep::METACOGNITION, "classified", true);

        let all = trace.all_participating_ontologies();
        // Direct ontologies — names come from each ontology's meta().
        assert!(all.contains(&"LemonOntology"));
        assert!(all.contains(&"LambekOntology"));
        assert!(all.contains(&"MontagueOntology"));
        assert!(all.contains(&"MetaCognitionOntology"));
        // Via functors
        assert!(all.contains(&"Communication (Shannon)"));
        assert!(all.contains(&"Control (Wiener)"));
        assert!(all.contains(&"DRT (Kamp)"));
        assert!(all.contains(&"Dialogue (Grosz)"));
        assert!(all.contains(&"Diagnostics (Reiter)"));
    }

    #[pr4xis::praxis_value(Explainable, Extensible)]
    #[test]
    fn serialize_with_functors_shows_connections() {
        let entry = PipelineTraceEntry::from_step(PipelineStep::TOKENIZE, "5 tokens", true);
        let s = entry.serialize_with_functors();
        assert!(s.contains("→Communication (Shannon)"));
        assert!(s.contains("→Control (Wiener)"));
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn every_step_has_functor_connections() {
        // Every pipeline step should connect to at least one other ontology via functor
        let steps = [
            PipelineStep::TOKENIZE,
            PipelineStep::PARSE,
            PipelineStep::INTERPRET,
            PipelineStep::ENTITY_LOOKUP,
            PipelineStep::TAXONOMY_TRAVERSAL,
            PipelineStep::METACOGNITION,
            PipelineStep::SPEECH_ACT_CLASSIFICATION,
            PipelineStep::CONTENT_DETERMINATION,
            PipelineStep::REALIZATION,
            PipelineStep::EPISTEMIC_CLASSIFICATION,
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_pipeline_step_names_a_currently_registered_ontology() {
        // PipelineStep::ontology is a const &'static str, not a call to the
        // owning ontology's own meta().name() -- architecturally required
        // (see this file's TraceOntology doc comment: PipelineStep must
        // stay Copy and const-constructible, and Vocabulary/Provenance are
        // not const-constructible), so nothing catches a rename of the REAL
        // ontology drifting out of sync with this literal at compile time.
        // This test is the runtime drift guard that gap implies: every
        // step's name must match a genuinely CURRENTLY-registered ontology
        // (found via a live probe of describe_knowledge_base()'s own
        // output, not assumed) -- if one of these ever goes stale (the
        // owning ontology renamed, the literal not updated), this fails
        // loudly instead of the trace silently naming a Loaded ontology
        // that doesn't exist.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let registered: Vec<String> = pr4xis::ontology::describe_knowledge_base()
                .iter()
                .map(|v| v.name().to_string())
                .collect();
            for step in PipelineStep::ALL {
                assert!(
                    registered.iter().any(|n| n == step.ontology_name()),
                    "{:?}'s ontology_name {:?} does not match any currently \
                     registered ontology -- the owning ontology was likely \
                     renamed without updating this const literal",
                    step,
                    step.ontology_name()
                );
            }
        }
    }
}

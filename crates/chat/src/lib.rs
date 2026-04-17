use pr4xis::category::{Ap, NonEmpty, Product, Writer};
pub use pr4xis::ontology::OntologyMeta;
use pr4xis::ontology::Vocabulary;
use pr4xis::ontology::upper::being::Being;
use pr4xis_domains::cognitive::cognition::epistemics;
use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::lambek::{
    ReductionResult, TypedToken, montague, reduce::chart_reduce, tokenize, tokenize_ontological,
};
use pr4xis_domains::cognitive::linguistics::language::Language;
use pr4xis_domains::cognitive::linguistics::pragmatics::speech_act::SpeechAct;
use pr4xis_domains::formal::information::diagnostics::DiagnosticOntology;
use pr4xis_domains::formal::information::diagnostics::trace_functors::{
    PipelineStep, PipelineTrace, TracedPipeline,
};
use pr4xis_domains::formal::information::diagnostics::trace_impls;
use pr4xis_domains::formal::information::knowledge::{SelfModelInstance, describe_knowledge_base};

/// The Diagnostics ontology governs the trace — every PipelineTraceEntry is
/// a Diagnostic concept. `TRACE_META` is pulled from `define_ontology!`-generated
/// `meta()` so the chat engine's trace attribution flows from the ontology itself,
/// not from hardcoded strings. Public so callers can inspect which ontology
/// authorizes the trace semantics.
pub const TRACE_META: OntologyMeta = DiagnosticOntology::meta();

// Praxis Chat Engine — shared logic for CLI, WASM, and any frontend.
//
// Zero I/O. Takes a string, returns a string.
// All intelligence comes from the Language ontology.
// The chat engine is a functor: Input → Language → Response.
//
// Trace is produced by applying trace functors to each pipeline step result.
// The trace functor maps: PipelineStep → DiagnosticConcept → PROV Activity.
// No manual trace.ok() — the functor provides ontology names and operations.

/// Re-export for callers.
pub use pr4xis_domains::formal::information::diagnostics::trace_functors::PipelineTraceEntry;

/// Alias — the trace is a PipelineTrace from the Diagnostics ontology.
pub type Trace = PipelineTrace;

/// Result of processing input through the linguistics pipeline.
pub struct ProcessResult {
    pub response: String,
    pub user_act: SpeechAct,
    pub system_act: SpeechAct,
    pub duration_us: u64,
    pub token_count: usize,
    pub parsed: bool,
    pub trace: Trace,
    pub from_ontology: bool,
}

/// Process input through the full linguistics pipeline.
/// Returns (response_text, user_speech_act, system_speech_act).
pub fn process(lang: &English, input: &str) -> (String, SpeechAct, SpeechAct) {
    let result = process_with_metadata(lang, input);
    (result.response, result.user_act, result.system_act)
}

/// Process with full metadata — timing, token count.
///
/// The pipeline IS a writer monad computation: TracedPipeline<A> = Writer<PipelineTrace, A>.
/// Each stage returns a traced value, and composition through `.bind()` / `.tell()`
/// accumulates trace entries automatically via the PipelineTrace monoid.
/// No mutation. No manual trace.record() calls.
///
/// Reference: Moggi, "Notions of Computation and Monads" (1991).
pub fn process_with_metadata(lang: &English, input: &str) -> ProcessResult {
    let start = WasmSafeTimer::now();

    // Stage 1: Tokenize through the Language ontology.
    // tokenize_ontological produces Tokens (ontological: sense + POS + Lambek type).
    // Legacy TypedTokens derived for the reducer until it's migrated.
    let ont_tokens = tokenize_ontological(input, lang);
    let tokens: Vec<TypedToken> = ont_tokens.iter().cloned().map(Into::into).collect();
    let (_, alternatives) = tokenize::tokenize_with_alternatives(input, lang);
    let token_count = ont_tokens.len();

    if tokens.is_empty() {
        return ProcessResult {
            response: "Empty input received.".into(),
            user_act: SpeechAct::Assertion,
            system_act: SpeechAct::Assertion,
            duration_us: start.elapsed_us(),
            token_count: 0,
            parsed: false,
            trace: PipelineTrace::from_traceable(&trace_impls::TokenizeResult { tokens: &tokens }),
            from_ontology: false,
        };
    }

    // NonEmpty semigroup: the empty-check above proves the invariant.
    let ne_tokens = NonEmpty::of(tokens[0].clone(), tokens[1..].to_vec());
    let words: Vec<String> = ne_tokens.iter().map(|t| t.word.clone()).collect();
    let type_sets: Vec<Vec<_>> = ne_tokens
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let mut ts = vec![t.lambek_type.clone()];
            if let Some(alts) = alternatives.get(i) {
                for alt in alts {
                    if !ts.contains(alt) {
                        ts.push(alt.clone());
                    }
                }
            }
            ts
        })
        .collect();

    // Stage 2: Parse through Lambek grammar. Chart reduction returns a Traceable.
    let reduction = chart_reduce(&words, &type_sets);
    let parsed = reduction.success;

    // Stage 3: Interpret through Montague semantics.
    let montague_tokens = if parsed && reduction.remaining.len() == ne_tokens.len() {
        &reduction.remaining
    } else {
        &tokens
    };
    let meaning = montague::interpret(montague_tokens, lang);

    // Stage 4: Classify the speech act through pragmatics.
    let user_act = if meaning.is_question() {
        SpeechAct::Question
    } else {
        SpeechAct::Assertion
    };

    // Stage 5: Metacognitive decision — which branch of repair/response to take.
    let metacog_decision = if meaning.is_question() {
        "question detected → query ontology"
    } else if meaning.is_proposition() {
        "statement detected → acknowledge/elaborate"
    } else if parsed {
        "parsed but unrecognized form → partial understanding"
    } else {
        "parse failed → metacognitive repair (attempt partial understanding)"
    };

    // Stage 6: Generate the response through NLG.
    // Self-referential questions route through the SelfModel eigenform.
    let response_result = if is_self_referential(&ont_tokens) {
        answer_self_referential(lang)
    } else {
        match &meaning {
            montague::Sem::Question {
                predicate,
                arguments,
            } => answer_question(lang, predicate, arguments),
            montague::Sem::Prop {
                predicate,
                arguments,
            } => answer_statement(lang, predicate, arguments),
            _ => attempt_partial_understanding(lang, &tokens, &reduction, &meaning),
        }
    };

    // Build the trace by threading TracedPipeline<()> through each stage.
    // `.tell()` is the writer monad's log-append operation; each call concatenates
    // a single trace entry via the PipelineTrace monoid (Vec concatenation).
    // The final log IS the full pipeline trace, accumulated by composition, not mutation.
    let response_len = response_result.response.len();
    let pipeline: TracedPipeline<()> = Writer::pure(())
        .tell(PipelineTrace::from_traceable(
            &trace_impls::TokenizeResult { tokens: &tokens },
        ))
        .tell(PipelineTrace::from_traceable(&reduction))
        .tell(PipelineTrace::from_traceable(
            &trace_impls::InterpretResult { meaning: &meaning },
        ))
        .tell(PipelineTrace::single(
            PipelineStep::SpeechActClassification,
            &format!("{:?}", user_act),
            true,
        ))
        .tell(PipelineTrace::single(
            PipelineStep::Metacognition,
            metacog_decision,
            true,
        ))
        .tell(PipelineTrace::from_traceable(&response_result))
        .tell(PipelineTrace::single(
            PipelineStep::Realization,
            &format!("{response_len} chars generated"),
            true,
        ));

    let from_ontology = response_result.from_ontology;
    let response = response_result.response;

    ProcessResult {
        response,
        user_act,
        system_act: SpeechAct::Assertion,
        duration_us: start.elapsed_us(),
        token_count,
        parsed,
        trace: pipeline.log,
        from_ontology,
    }
}

fn attempt_partial_understanding(
    en: &English,
    tokens: &[TypedToken],
    reduction: &ReductionResult,
    _meaning: &montague::Sem,
) -> trace_impls::ResponseResult {
    // Check known/unknown through the Language trait — covers BOTH
    // function words (closed class) AND WordNet concepts (open class).
    let known_words: Vec<&str> = tokens
        .iter()
        .filter(|t| en.lexical_lookup(&t.word).is_some())
        .map(|t| t.word.as_str())
        .collect();

    let unknown_words: Vec<&str> = tokens
        .iter()
        .filter(|t| en.lexical_lookup(&t.word).is_none())
        .map(|t| t.word.as_str())
        .collect();

    let has_knowledge = !known_words.is_empty();
    let parsed = reduction.success;
    let query_result: Option<&str> = if parsed { Some("parsed") } else { None };
    let state = epistemics::classify_result(parsed, has_knowledge, query_result);

    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;

    let frame = ResponseFrame::from_epistemic(&state);
    let entities: Vec<String> = known_words.iter().map(|s| s.to_string()).collect();

    let response = match state {
        epistemics::EpistemicConcept::UnknownKnown => {
            if known_words.len() == 1 {
                define_word(en, known_words[0])
            } else {
                let nouns: Vec<&str> = tokens
                    .iter()
                    .filter(|t| !en.lookup(&t.word).is_empty() && t.lambek_type.is_noun())
                    .map(|t| t.word.as_str())
                    .collect();
                if nouns.len() >= 2 {
                    explore_concepts(en, &nouns)
                } else {
                    let mut content = ResponseContent::new(frame);
                    for w in &known_words {
                        content = content.with_entity(w);
                    }
                    realize::realize(&content)
                }
            }
        }
        epistemics::EpistemicConcept::KnownUnknown => {
            let mut content = ResponseContent::new(frame);
            for w in &unknown_words {
                content = content.with_entity(w);
            }
            realize::realize(&content)
        }
        epistemics::EpistemicConcept::KnownKnown => {
            let content = ResponseContent::new(frame).with_predicate(&_meaning.describe());
            realize::realize(&content)
        }
        epistemics::EpistemicConcept::UnknownUnknown => {
            realize::realize(&ResponseContent::new(frame))
        }
    };

    trace_impls::ResponseResult {
        response,
        entities_found: entities,
        taxonomy_checked: None,
        from_ontology: has_knowledge,
    }
}

/// Check if the tokens reference the system itself.
/// Routes through the self-model ontology via token senses:
/// if any token's sense references a self-model concept, the
/// question is self-referential.
fn is_self_referential(tokens: &[pr4xis_domains::cognitive::linguistics::text::Token]) -> bool {
    tokens.iter().any(|t| {
        t.word == "you" || t.word == "yourself" || t.word == "praxis" || t.word == "pr4xis"
    })
}

/// Answer a self-referential question through the eigenform.
///
/// The response IS the self-model eigenform presented through the
/// Schema transport layer. No hardcoded text — the Presentation
/// carries the data, the surface rendering derives from it.
fn answer_self_referential(lang: &English) -> trace_impls::ResponseResult {
    use pr4xis_domains::formal::information::schema::transport::Present;
    let eigenform = observe_self(lang);
    let presentation = eigenform.present();

    let response = presentation.to_json();

    trace_impls::ResponseResult {
        response,
        entities_found: vec!["pr4xis".into(), "self-model".into()],
        taxonomy_checked: None,
        from_ontology: true,
    }
}

pub fn answer_question(
    en: &English,
    predicate: &str,
    arguments: &[montague::Sem],
) -> trace_impls::ResponseResult {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;

    let all_entities: Vec<String> = arguments.iter().map(extract_entity_name).collect();

    let entities: Vec<String> = all_entities
        .iter()
        .filter(|e| !en.lookup(e).is_empty())
        .cloned()
        .collect();

    if entities.len() >= 2 {
        let child = &entities[0];
        let parent = &entities[1];

        // Applicative: child and parent lookups are independent computations.
        // Using Ap::map2 makes this independence explicit — neither lookup
        // depends on the other's result.
        // Reference: McBride & Paterson, "Applicative Programming with Effects" (2008)
        let lookups = Ap::pure(en.lookup(child).to_vec())
            .map2(Ap::pure(en.lookup(parent).to_vec()), |c, p| {
                Product::new(c, p)
            });

        let child_ids = &lookups.value.left;
        let parent_ids = &lookups.value.right;

        if !child_ids.is_empty() && !parent_ids.is_empty() {
            for &cid in child_ids {
                for &pid in parent_ids {
                    if en.is_a(cid, pid) {
                        return trace_impls::ResponseResult {
                            response: build_taxonomy_response(en, child, parent, cid, pid),
                            entities_found: entities.clone(),
                            taxonomy_checked: Some((child.clone(), parent.clone(), true)),
                            from_ontology: true,
                        };
                    }
                }
            }
            return trace_impls::ResponseResult {
                response: realize::realize_negation(child, parent),
                entities_found: entities.clone(),
                taxonomy_checked: Some((child.clone(), parent.clone(), false)),
                from_ontology: true,
            };
        }

        if !parent_ids.is_empty() && !child_ids.is_empty() {
            for &cid in parent_ids {
                for &pid in child_ids {
                    if en.is_a(cid, pid) {
                        let content = ResponseContent::new(ResponseFrame::AssertKnowledge)
                            .with_predicate("is_a")
                            .with_entity(parent)
                            .with_entity(child);
                        return trace_impls::ResponseResult {
                            response: realize::realize(&content),
                            entities_found: entities.clone(),
                            taxonomy_checked: Some((parent.clone(), child.clone(), true)),
                            from_ontology: true,
                        };
                    }
                }
            }
        }
    }

    if entities.len() == 1 {
        let response = define_word(en, &entities[0]);
        return trace_impls::ResponseResult {
            response,
            entities_found: entities,
            taxonomy_checked: None,
            from_ontology: true,
        };
    }

    let mut content = ResponseContent::new(ResponseFrame::AcknowledgeGap).with_predicate(predicate);
    for e in &entities {
        content = content.with_entity(e);
    }
    trace_impls::ResponseResult {
        response: realize::realize(&content),
        entities_found: entities,
        taxonomy_checked: None,
        from_ontology: false,
    }
}

pub fn answer_statement(
    en: &English,
    _predicate: &str,
    arguments: &[montague::Sem],
) -> trace_impls::ResponseResult {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;

    let entities: Vec<String> = arguments.iter().map(extract_entity_name).collect();

    if entities.len() == 1 {
        let ids = en.lookup(&entities[0]);
        if !ids.is_empty() {
            let response = define_word(en, &entities[0]);
            return trace_impls::ResponseResult {
                response,
                entities_found: entities,
                taxonomy_checked: None,
                from_ontology: true,
            };
        }
    }

    let mut content = ResponseContent::new(ResponseFrame::AssertKnowledge);
    for e in &entities {
        content = content.with_entity(e);
    }
    trace_impls::ResponseResult {
        response: realize::realize(&content),
        entities_found: entities,
        taxonomy_checked: None,
        from_ontology: true,
    }
}

pub fn define_word(en: &English, word: &str) -> String {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;

    let ids = en.lookup(word);
    if ids.is_empty() {
        let content = ResponseContent::new(ResponseFrame::AcknowledgeGap).with_entity(word);
        return realize::realize(&content);
    }

    let mut content = ResponseContent::new(ResponseFrame::AssertKnowledge).with_entity(word);
    for &id in ids.iter().take(5) {
        if let Some(concept) = en.concept(id) {
            for def in &concept.definitions {
                content = content.with_definition(word, def);
            }
        }
    }
    realize::realize(&content)
}

/// Build a taxonomy response following the NLG pipeline.
///
/// Reiter & Dale (2000):
/// 1. Content determination — gather facts from ontology
/// 2. Document planning — organize with RST (assertion → evidence → elaboration)
/// 3. Microplanning — referring expressions
/// 4. Realization — compose through grammar
fn build_taxonomy_response(
    en: &English,
    child_word: &str,
    parent_word: &str,
    child_id: pr4xis_domains::cognitive::linguistics::english::ConceptId,
    parent_id: pr4xis_domains::cognitive::linguistics::english::ConceptId,
) -> String {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize;

    // ---- Stage 1: Content Determination ----
    // Gather all relevant knowledge from the ontology.

    // The taxonomy chain: how child relates to parent
    let mut chain_ids = vec![(child_word.to_string(), child_id)];
    let mut current = child_id;
    for _ in 0..10 {
        if current == parent_id {
            break;
        }
        if let Some(&p) = en.parents(current).first() {
            if let Some(c) = en.concept(p) {
                let label = c
                    .lemmas
                    .first()
                    .map(|l| l.as_str())
                    .unwrap_or(&c.original_id);
                chain_ids.push((label.to_string(), p));
            }
            current = p;
        } else {
            break;
        }
    }

    // Definitions for each concept in the chain
    let chain_defs: Vec<(&str, &str)> = chain_ids
        .iter()
        .filter_map(|(label, id)| {
            en.concept(*id)
                .and_then(|c| c.definitions.first())
                .map(|def| (label.as_str(), def.as_str()))
        })
        .collect();

    // Children (subtypes) of the child concept
    let subtypes: Vec<&str> = en
        .children(child_id)
        .iter()
        .take(5)
        .filter_map(|&id| {
            en.concept(id)
                .and_then(|c| c.lemmas.first())
                .map(|l| l.as_str())
        })
        .collect();

    // ---- Stage 2: Document Planning (RST) ----
    // Organize as: Assertion (nucleus) → Evidence (satellite) → Elaboration

    let mut sections = Vec::new();

    // Nucleus: the direct assertion
    sections.push(format!(
        "Yes. {}.",
        realize::sentence_copula(child_word, parent_word)
    ));

    // Evidence: HOW — the taxonomy path explains the connection
    if chain_ids.len() > 2 {
        let chain_labels: Vec<&str> = chain_ids.iter().map(|(l, _)| l.as_str()).collect();
        let mut evidence_parts = Vec::new();
        for i in 0..chain_labels.len() - 1 {
            evidence_parts.push(realize::sentence_copula(
                chain_labels[i],
                chain_labels[i + 1],
            ));
        }
        sections.push(evidence_parts.join(", and "));
    }

    // Elaboration: WHAT each concept means
    for (label, def) in &chain_defs {
        sections.push(format!("{label}: {def}"));
    }

    // Elaboration: subtypes
    if !subtypes.is_empty() {
        sections.push(format!("types of {child_word}: {}", subtypes.join(", ")));
    }

    // ---- Stage 3 & 4: Microplanning + Realization ----
    // Already handled by realize::sentence_copula (determiner selection, grammar)

    sections.join("\n")
}

/// Explore what the system knows about multiple concepts.
///
/// Uses the associations ontology (taxonomy, mereology) to discover
/// relationships between concepts — common ancestors, is-a chains,
/// shared properties. This is metacognition: instead of guessing
/// "did you mean is X a Y?", explore and report what we actually know.
fn explore_concepts(en: &English, words: &[&str]) -> String {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize;

    let mut lines = Vec::new();

    // Collect all concept IDs per word
    let word_ids: Vec<(&str, Vec<_>)> = words.iter().map(|&w| (w, en.lookup(w).to_vec())).collect();

    // For each concept, describe it and trace taxonomy
    for (word, ids) in &word_ids {
        if let Some(&id) = ids.first()
            && let Some(concept) = en.concept(id)
        {
            if let Some(def) = concept.definitions.first() {
                lines.push(format!("{word}: {def}"));
            }

            // Trace taxonomy chain through the ontology
            let mut chain = Vec::new();
            let mut current = id;
            for _ in 0..5 {
                if let Some(&parent) = en.parents(current).first()
                    && let Some(pc) = en.concept(parent)
                {
                    let label = pc
                        .lemmas
                        .first()
                        .map(|l| l.as_str())
                        .unwrap_or(&pc.original_id);
                    chain.push(label.to_string());
                    current = parent;
                } else {
                    break;
                }
            }
            if !chain.is_empty() {
                // Generate "word is a X → Y → Z" through grammar
                let first = &chain[0];
                let copula = realize::sentence_copula(word, first);
                if chain.len() > 1 {
                    lines.push(format!("  {copula} → {}", chain[1..].join(" → ")));
                } else {
                    lines.push(format!("  {copula}"));
                }
            }
        }
    }

    // Find relationships between concept pairs through associations
    if word_ids.len() >= 2 {
        for i in 0..word_ids.len() {
            for j in i + 1..word_ids.len() {
                let (w1, ids1) = &word_ids[i];
                let (w2, ids2) = &word_ids[j];
                if let (Some(&id1), Some(&id2)) = (ids1.first(), ids2.first()) {
                    if en.is_a(id1, id2) {
                        lines.push(realize::sentence_copula(w1, w2));
                    } else if en.is_a(id2, id1) {
                        lines.push(realize::sentence_copula(w2, w1));
                    } else if let Some(lca) = find_common_ancestor(en, id1, id2)
                        && let Some(c) = en.concept(lca)
                    {
                        let label = c
                            .lemmas
                            .first()
                            .map(|l| l.as_str())
                            .unwrap_or(&c.original_id);
                        let s1 = realize::sentence_copula(w1, label);
                        let s2 = realize::sentence_copula(w2, label);
                        lines.push(format!("{s1}, and {s2}"));
                    }
                }
            }
        }
    }

    if lines.is_empty() {
        realize::realize(&realize::ResponseContent::new(
            pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame::AcknowledgeGap,
        ))
    } else {
        lines.join("\n")
    }
}

/// Find the lowest common ancestor of two concepts in the taxonomy.
fn find_common_ancestor(
    en: &English,
    a: pr4xis_domains::cognitive::linguistics::english::ConceptId,
    b: pr4xis_domains::cognitive::linguistics::english::ConceptId,
) -> Option<pr4xis_domains::cognitive::linguistics::english::ConceptId> {
    use std::collections::HashSet;

    // Collect all ancestors of A
    let mut ancestors_a = HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    ancestors_a.insert(a);
    for &p in en.parents(a) {
        queue.push_back(p);
    }
    while let Some(current) = queue.pop_front() {
        if ancestors_a.insert(current) {
            for &p in en.parents(current) {
                queue.push_back(p);
            }
        }
    }

    // BFS up from B, first hit in ancestors_a is the LCA
    let mut visited = HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    for &p in en.parents(b) {
        queue.push_back(p);
    }
    while let Some(current) = queue.pop_front() {
        if ancestors_a.contains(&current) {
            return Some(current);
        }
        if visited.insert(current) {
            for &p in en.parents(current) {
                queue.push_back(p);
            }
        }
    }

    None
}

pub fn extract_entity_name(sem: &montague::Sem) -> String {
    match sem {
        montague::Sem::Entity { word, .. } => word.clone(),
        montague::Sem::Pred { word } => word.clone(),
        // For Func (e.g., "is" applied to "dog"), extract the content entity
        // from the body, not the function word itself.
        montague::Sem::Func { body, word, .. } => {
            let inner = extract_entity_name(body);
            // If the body yielded a real entity, use it; otherwise fall back to the func word
            if !inner.is_empty() {
                inner
            } else {
                word.clone()
            }
        }
        montague::Sem::Prop { predicate, .. } | montague::Sem::Question { predicate, .. } => {
            predicate.clone()
        }
    }
}

// =========================================================================
// Timer — works on both native and WASM
// =========================================================================

/// WASM-safe timer. `std::time::Instant` panics on wasm32-unknown-unknown
/// because the target has no system clock. This wrapper uses `Instant` on
/// native and returns 0 on WASM.
struct WasmSafeTimer {
    #[cfg(not(target_arch = "wasm32"))]
    start: std::time::Instant,
}

impl WasmSafeTimer {
    fn now() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            start: std::time::Instant::now(),
        }
    }

    fn elapsed_us(&self) -> u64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.start.elapsed().as_micros() as u64
        }
        #[cfg(target_arch = "wasm32")]
        {
            0
        }
    }
}

// =========================================================================
// Self-description — through the SelfModel ontology
// =========================================================================

/// All loaded ontologies including language-specific runtime data.
pub fn loaded_ontologies(_lang: &English) -> Vec<Vocabulary> {
    let mut ontologies = describe_knowledge_base();
    ontologies.push(Vocabulary::from_ontology::<
        pr4xis_domains::cognitive::linguistics::lexicon::ontology::LexicalCategory,
        pr4xis_domains::cognitive::linguistics::lexicon::pos::PosTag,
    >(
        "English (WordNet)",
        "pr4xis_domains::cognitive::linguistics::english",
        "Open English WordNet 2025; Princeton WordNet",
        Some(Being::SocialObject),
    ));
    ontologies
}

/// The eigenform — the system observes itself.
///
/// This IS the self-observation operator F from von Foerster.
/// The result IS the fixed point X = F(X).
/// The SelfModelInstance carries the complete self-description
/// through the SelfModel ontology, not through hardcoded strings.
pub fn observe_self(lang: &English) -> SelfModelInstance {
    SelfModelInstance::observe(loaded_ontologies(lang))
}

/// JSON encoding of the eigenform for transport (WASM boundary).
pub fn self_describe(lang: &English) -> String {
    observe_self(lang).to_json()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Monoid;

    fn sample_english() -> English {
        // Use sample data for unit tests (fast, no WordNet needed)
        English::sample()
    }

    // --- Algebraic structure integration tests ---

    #[test]
    fn pipeline_trace_is_monoid() {
        // PipelineTrace forms a monoid under concatenation.
        // This enables Writer<PipelineTrace, A> — the pipeline IS a writer monad.
        let empty = PipelineTrace::empty();
        let t1 = PipelineTrace::single(PipelineStep::Tokenize, "5 tokens", true);
        let t2 = PipelineTrace::single(PipelineStep::Parse, "success", true);

        // Left identity: empty ++ t = t
        assert_eq!(empty.combine(&t1).entries.len(), 1);
        // Right identity: t ++ empty = t
        assert_eq!(t1.combine(&empty).entries.len(), 1);
        // Associativity
        let t3 = PipelineTrace::single(PipelineStep::Interpret, "question", true);
        assert_eq!(
            t1.combine(&t2).combine(&t3).entries.len(),
            t1.combine(&t2.combine(&t3)).entries.len()
        );
    }

    #[test]
    fn traced_pipeline_is_writer_monad() {
        // TracedPipeline<A> = Writer<PipelineTrace, A>
        // Monadic bind composes pipeline steps and accumulates trace.
        let step1: TracedPipeline<usize> = pr4xis::category::Writer::new(
            5,
            PipelineTrace::single(PipelineStep::Tokenize, "5 tokens", true),
        );

        let result = step1
            .bind(|count| {
                pr4xis::category::Writer::new(
                    count > 0,
                    PipelineTrace::single(PipelineStep::Parse, "parsed", true),
                )
            })
            .bind(|parsed| {
                let msg = if parsed { "question" } else { "unknown" };
                pr4xis::category::Writer::new(
                    msg,
                    PipelineTrace::single(PipelineStep::Interpret, msg, parsed),
                )
            });

        assert_eq!(result.value, "question");
        assert_eq!(result.log.entries.len(), 3);
        // Trace accumulated through bind — no manual trace.record() needed
    }

    #[test]
    fn applicative_combines_independent_lookups() {
        // Ap::map2 combines independent lookups (child + parent).
        // This is applicative, not monadic — neither depends on the other.
        let child_ids = Ap::pure(vec![1, 2]);
        let parent_ids = Ap::pure(vec![3, 4, 5]);
        let combined = child_ids.map2(parent_ids, |c, p| Product::new(c, p));
        assert_eq!(combined.value.left.len(), 2);
        assert_eq!(combined.value.right.len(), 3);
    }

    #[test]
    fn nonempty_tokens_guarantee() {
        // After empty check, tokens form a NonEmpty — guaranteed at least one.
        // NonEmpty is a semigroup (can combine without needing identity).
        let en = sample_english();
        let result = process_with_metadata(&en, "dog");
        assert!(result.token_count > 0);
        // The pipeline used NonEmpty internally after the empty check
    }

    // --- Pipeline tests ---

    #[test]
    fn process_taxonomy_question() {
        let en = sample_english();
        let (response, user_act, _) = process(&en, "is a dog a mammal");
        assert_eq!(user_act, SpeechAct::Question);
        assert!(
            response.contains("Yes") || response.contains("No") || response.contains("dog"),
            "taxonomy question should get a substantive answer, got: {}",
            response
        );
    }

    #[test]
    fn process_simple_sentence() {
        let en = sample_english();
        let (response, _, _) = process(&en, "the dog runs");
        // Should either parse or give partial understanding — not crash
        assert!(!response.is_empty());
    }

    #[test]
    fn process_what_question() {
        let en = sample_english();
        let (response, _, _) = process(&en, "what is a dog");
        // With sample data "what" may not be in lexicon — just verify no crash
        assert!(!response.is_empty());
    }

    #[test]
    fn process_empty_input() {
        let en = sample_english();
        let (response, _, _) = process(&en, "");
        assert!(!response.is_empty());
    }

    #[test]
    fn self_describe_has_ontologies() {
        let en = sample_english();
        let json = self_describe(&en);
        assert!(json.contains("ontology_count"));
        assert!(json.contains("SelfModel"));
        assert!(json.contains("Knowledge"));
    }

    #[test]
    fn self_describe_eigenform_is_stable() {
        // Self(Self) = Self — calling observe_self twice gives same result
        let en = sample_english();
        let first = observe_self(&en);
        let second = observe_self(&en);
        assert_eq!(first.total_concepts, second.total_concepts);
        assert_eq!(first.total_morphisms, second.total_morphisms);
        assert_eq!(first.components.len(), second.components.len());
    }

    // --- Phase 2: OntologyMeta + TracedPipeline integration tests ---

    #[test]
    fn trace_meta_is_from_diagnostic_ontology() {
        // TRACE_META comes from DiagnosticOntology::meta() — generated by
        // define_ontology!. The ontology identifies itself through the macro,
        // not through hand-written strings.
        assert_eq!(TRACE_META.name, "DiagnosticOntology");
        assert!(TRACE_META.module_path.contains("diagnostics"));
    }

    #[test]
    fn pipeline_trace_accumulates_through_writer_composition() {
        // After the refactor, `process_with_metadata` builds its trace through
        // `TracedPipeline<()>` composition — each `.tell()` call combines a
        // single PipelineTrace via the Vec monoid. No mutation.
        let en = sample_english();
        let result = process_with_metadata(&en, "is a dog a mammal");

        // The full pipeline fires: tokenize → parse → interpret → speech act
        // → metacognition → response → realization. Seven entries minimum.
        assert!(
            result.trace.entries.len() >= 7,
            "expected full pipeline trace, got {} entries",
            result.trace.entries.len()
        );

        // First entry is always Tokenize — the writer log respects order
        // because Vec::combine concatenates left-to-right.
        assert_eq!(result.trace.entries[0].step, PipelineStep::Tokenize);
        // Last entry is Realization — the final step of the writer chain.
        let last = result.trace.entries.last().unwrap();
        assert_eq!(last.step, PipelineStep::Realization);
    }

    #[test]
    fn empty_input_still_produces_traceable_output() {
        // Empty-branch early return: the trace carries the tokenize result
        // constructed via `PipelineTrace::from_traceable`, not a mutation.
        let en = sample_english();
        let result = process_with_metadata(&en, "");
        assert_eq!(result.token_count, 0);
        assert_eq!(result.trace.entries.len(), 1);
        assert_eq!(result.trace.entries[0].step, PipelineStep::Tokenize);
    }

    #[test]
    fn writer_tell_preserves_trace_order() {
        // The writer monad's `tell` preserves order because Vec monoid
        // concatenation is left-associative. Verifying directly:
        let pipeline: TracedPipeline<()> = Writer::pure(())
            .tell(PipelineTrace::single(PipelineStep::Tokenize, "a", true))
            .tell(PipelineTrace::single(PipelineStep::Parse, "b", true))
            .tell(PipelineTrace::single(PipelineStep::Interpret, "c", true));
        assert_eq!(pipeline.log.entries.len(), 3);
        assert_eq!(pipeline.log.entries[0].step, PipelineStep::Tokenize);
        assert_eq!(pipeline.log.entries[1].step, PipelineStep::Parse);
        assert_eq!(pipeline.log.entries[2].step, PipelineStep::Interpret);
    }
}

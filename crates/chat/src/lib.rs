use pr4xis::category::{Ap, NonEmpty, Product, Writer};
use pr4xis::ontology::Vocabulary;
use pr4xis::ontology::meta::OntologyName;
pub use pr4xis::ontology::meta::Provenance;
pub use pr4xis::ontology::meta::Provenance as RelationshipMeta;
use pr4xis_domains::cognitive::cognition::epistemics;
use pr4xis_domains::cognitive::linguistics::english::{ConceptId, English, LexicalReasoner};
use pr4xis_domains::cognitive::linguistics::lambek::{
    ReductionResult, TypedToken, montague, reduce::chart_reduce, tokenize, tokenize_ontological,
};
use pr4xis_domains::cognitive::linguistics::language::Language;
use pr4xis_domains::cognitive::linguistics::pragmatics::speech_act::SpeechAct;
use pr4xis_domains::formal::information::diagnostics::DiagnosticOntology;
use pr4xis_domains::formal::information::diagnostics::trace_functors::{
    PipelineTrace, TracedPipeline,
};
use pr4xis_domains::formal::information::diagnostics::trace_impls;
use pr4xis_domains::formal::information::knowledge::{
    SelfModelInstance, describe_knowledge_base, is_self_referent,
};

/// The Diagnostics ontology governs the trace — every PipelineTraceEntry is
/// a Diagnostic concept. `trace_meta()` is pulled from `ontology!`-generated
/// `meta()` so the chat engine's trace attribution flows from the ontology itself,
/// not from hardcoded strings. Public so callers can inspect which ontology
/// authorizes the trace semantics.
///
/// Returns a [`RelationshipMeta`] — the unified Lemon+PROV-O record every
/// structural entity in pr4xis carries (issue #153).
pub fn trace_meta() -> RelationshipMeta {
    DiagnosticOntology::meta()
}

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

/// The TYPED outcome of a turn (doc §4.1) — a self-aware system models what it
/// CANNOT answer, not just what it can. Abstention stops being a string the UI
/// sniffs ("I don't know …") and becomes a value: an `Abstained` turn names the
/// surfaces it could not resolve — the "asked but not loaded" set, i.e. WHAT TO
/// LOAD next. Reiter (1978) closed-world: the system knows the boundary of its
/// own knowledge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatOutcome {
    /// The turn answered from knowledge (the embedded model or a loaded ontology).
    Answered,
    /// The turn abstained — it could not ground the query. `unresolved` is the set
    /// of asked surfaces no loaded ontology knows (empty when the input itself
    /// carried no resolvable entity, e.g. empty input).
    Abstained { unresolved: Vec<String> },
}

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
    /// The typed outcome — answered, or abstained with the unresolved surfaces.
    pub outcome: ChatOutcome,
}

/// Process input through the full linguistics pipeline.
/// Returns (response_text, user_speech_act, system_speech_act).
///
/// Source-agnostic: the pipeline reasons over the embedded English
/// language model. Loaded corpora (statutes, …) are surfaced through the
/// self-model catalog (see `self_describe`), not threaded into the
/// linguistic pipeline.
pub fn process(lang: &English, input: &str) -> (String, SpeechAct, SpeechAct) {
    let result = process_with_metadata(lang, input);
    (result.response, result.user_act, result.system_act)
}

/// Process with full metadata — timing, token count.
///
/// The pipeline IS a writer monad computation: `TracedPipeline<A> = Writer<PipelineTrace, A>`.
/// Each stage returns a traced value, and composition through `.bind()` / `.tell()`
/// accumulates trace entries automatically via the PipelineTrace monoid.
/// No mutation. No manual trace.record() calls.
///
/// Reference: Moggi, "Notions of Computation and Monads" (1991).
///
/// The linguistic pipeline stages (tokenize/parse/interpret/respond)
/// reason over `lang` only. Self-referential queries route through the
/// self-model (see `self_describe`); loaded corpora live in its catalog.
///
/// The lexical-reasoning surface (`lookup` / `is_a` / `concept` / define)
/// is threaded through a `&dyn LexicalReasoner` — `English` itself by
/// default. A caller that has loaded corpora can inject a
/// [`process_with_reasoner`] reasoner that grounds those corpora into the
/// same lexical surface (see `ComposedReasoner` in `pr4xis-domains`).
pub fn process_with_metadata(lang: &English, input: &str) -> ProcessResult {
    // Default reasoner = the embedded English model itself (behavior-
    // preserving: English is both the linguistic substrate and the lexical
    // reasoner when no corpus is loaded).
    process_with_reasoner(lang, lang, input)
}

/// Process with an explicit lexical reasoner injected.
///
/// `lang` is the linguistic substrate (tokenize / parse / Montague lexicon /
/// self-model); `reasoner` is the lexical-reasoning surface the answer stages
/// query (`lookup` / `is_a` / `concept` / `parents` / `children`). For the
/// embedded-only case `reasoner == lang`. When a corpus is loaded, `reasoner`
/// is a `ComposedReasoner` whose grounded lexicon UNIONs the loaded ontology's
/// glossed concepts into the same surface — so a "what is X" over a loaded
/// concept reads its loaded gloss, and an unloaded concept abstains exactly as
/// the embedded model already does.
pub fn process_with_reasoner(
    lang: &English,
    reasoner: &dyn LexicalReasoner,
    input: &str,
) -> ProcessResult {
    let start = WasmSafeTimer::now();

    // Stage 1: Tokenize through the Language ontology.
    // tokenize_ontological produces Tokens (ontological: sense + POS + Lambek type).
    // Legacy TypedTokens derived for the reducer until it's migrated.
    let ont_tokens = tokenize_ontological(input, lang);
    let raw_tokens: Vec<TypedToken> = ont_tokens.iter().cloned().map(Into::into).collect();
    let (_, alternatives) = tokenize::tokenize_with_alternatives(input, lang);
    let token_count = ont_tokens.len();

    // Multi-token surface recognition: collapse maximal known multi-word surfaces
    // (a loaded citation/label, a WordNet collocation) into single proper-noun
    // tokens BEFORE the parse, so every downstream stage — chart_reduce, interpret,
    // and the partial-understanding fallback — sees one lookup unit. Data-driven
    // (the reasoner's loaded surface set); a no-op when `max_surface_words == 1`
    // (embedded English), so single-token chat is byte-identical.
    let (tokens, mut type_sets) = tokenize::collapse_multiword_surfaces(
        &raw_tokens,
        &alternatives,
        reasoner.max_surface_words(),
        |s| {
            use pr4xis_domains::cognitive::linguistics::lambek::types::svo;
            // An ENTITY surface (a loaded citation/label/collocation) collapses to
            // a proper noun (NP); a RELATIONAL surface ("part of") collapses to a
            // relational predicate so "is X part of Y" parses. Both are loaded
            // data (the reasoner's surface + relation indices), never a pattern.
            if !reasoner.lookup(s).is_empty() {
                Some(svo::proper_noun())
            } else if reasoner.relation_for_surface(s).is_some() {
                Some(svo::relational_predicate())
            } else {
                None
            }
        },
    );

    // Single-word LOADED entity → offer a proper-noun (NP) reading too, so
    // "is <one-word-loaded-X> part of Y" parses (collapse only NP-types MULTI-word
    // spans). Gated on LOADED-corpus membership — NOT the union lookup — so English
    // function words (which never resolve to a loaded ontology) keep their copula/
    // determiner/wh types; the parse is not disturbed. NP is ADDED as an
    // alternative, never replacing the base type, so a determiner case ("a title")
    // still reduces via the noun reading and the chart keeps whichever derives S.
    {
        use pr4xis_domains::cognitive::linguistics::lambek::types::svo;
        let np = svo::proper_noun();
        for (i, tok) in tokens.iter().enumerate() {
            if reasoner.is_loaded_surface(&tok.word) && !type_sets[i].contains(&np) {
                type_sets[i].push(np.clone());
            }
        }
    }

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
            // Empty input named nothing — an abstention with no surface to load.
            outcome: ChatOutcome::Abstained {
                unresolved: Vec::new(),
            },
        };
    }

    // NonEmpty semigroup: the empty-check above proves the invariant.
    let ne_tokens = NonEmpty::of(tokens[0].clone(), tokens[1..].to_vec());
    let words: Vec<String> = ne_tokens.iter().map(|t| t.word.clone()).collect();
    // `type_sets` was built by `collapse_multiword_surfaces` above, aligned to the
    // (possibly collapsed) `tokens`: a collapsed span → `[proper_noun()]`; an
    // uncollapsed token → its own type plus that position's alternatives — exactly
    // the set this stage built before, so an uncollapsed stream is unchanged.

    // Stage 2: Parse through Lambek grammar. Chart reduction returns a Traceable.
    let reduction = chart_reduce(&words, &type_sets);
    let parsed = reduction.success;

    // Stage 3: Interpret through Montague semantics.
    let montague_tokens = if parsed && reduction.remaining.len() == ne_tokens.len() {
        &reduction.remaining
    } else {
        &tokens
    };
    let meaning = montague::interpret(montague_tokens, reasoner);

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
            } => answer_question(reasoner, predicate, arguments),
            montague::Sem::Prop {
                predicate,
                arguments,
            } => answer_statement(reasoner, predicate, arguments),
            _ => attempt_partial_understanding(reasoner, lang, &tokens, &reduction, &meaning),
        }
    };

    // Build the trace by threading TracedPipeline<()> through each stage.
    // `.tell()` is the writer monad's log-append operation; each call concatenates
    // a single trace entry via the PipelineTrace monoid (Vec concatenation).
    // The final log IS the full pipeline trace, accumulated by composition, not mutation.
    //
    // Every step produces its trace entry through a `Traceable` impl — the
    // ontology (Diagnostic/Trace) owns the entry's shape; the call site just
    // hands it the step's domain result. No inline string construction.
    let realization = trace_impls::RealizationResult {
        char_count: response_result.response.len(),
    };
    let speech_act_result = trace_impls::SpeechActClassificationResult { user_act };
    let metacog_result = trace_impls::MetacognitionResult {
        decision: metacog_decision,
        parsed,
    };
    let pipeline: TracedPipeline<()> = Writer::pure(())
        .tell(PipelineTrace::from_traceable(
            &trace_impls::TokenizeResult { tokens: &tokens },
        ))
        .tell(PipelineTrace::from_traceable(&reduction))
        .tell(PipelineTrace::from_traceable(
            &trace_impls::InterpretResult { meaning: &meaning },
        ))
        .tell(PipelineTrace::from_traceable(&speech_act_result))
        .tell(PipelineTrace::from_traceable(&metacog_result))
        .tell(PipelineTrace::from_traceable(&response_result))
        .tell(PipelineTrace::from_traceable(&realization));

    let from_ontology = response_result.from_ontology;
    let response = response_result.response;

    // The typed outcome (doc §4.1): answered from knowledge, or abstained — naming
    // the surfaces no loaded ontology could ground (what to load next). Derived
    // from the answer's `from_ontology` verdict, never sniffed from the text.
    let outcome = if from_ontology {
        ChatOutcome::Answered
    } else {
        ChatOutcome::Abstained {
            unresolved: unresolved_surfaces(&meaning, reasoner),
        }
    };

    ProcessResult {
        response,
        user_act,
        system_act: SpeechAct::Assertion,
        duration_us: start.elapsed_us(),
        token_count,
        parsed,
        trace: pipeline.log,
        from_ontology,
        outcome,
    }
}

/// The surfaces a turn NAMED but could not resolve to any concept — the "asked but
/// not loaded" set (doc §4.1), read off the interpreted meaning's entities and
/// filtered to those the reasoner's lexicon does not know. This IS the abstention
/// reason: what the user would have to load for the system to answer.
fn unresolved_surfaces(meaning: &montague::Sem, reasoner: &dyn LexicalReasoner) -> Vec<String> {
    let named: Vec<String> = match meaning {
        montague::Sem::Question { arguments, .. } | montague::Sem::Prop { arguments, .. } => {
            arguments.iter().map(extract_entity_name).collect()
        }
        other => vec![extract_entity_name(other)],
    };
    let mut unresolved = Vec::new();
    for surface in named {
        if !surface.is_empty()
            && reasoner.lookup(&surface).is_empty()
            && !unresolved.contains(&surface)
        {
            unresolved.push(surface);
        }
    }
    unresolved
}

fn attempt_partial_understanding(
    en: &dyn LexicalReasoner,
    lang: &dyn Language,
    tokens: &[TypedToken],
    reduction: &ReductionResult,
    _meaning: &montague::Sem,
) -> trace_impls::ResponseResult {
    // Check known/unknown through the Language trait — covers BOTH
    // function words (closed class) AND WordNet concepts (open class).
    let known_words: Vec<&str> = tokens
        .iter()
        .filter(|t| lang.lexical_lookup(&t.word).is_some())
        .map(|t| t.word.as_str())
        .collect();

    let unknown_words: Vec<&str> = tokens
        .iter()
        .filter(|t| lang.lexical_lookup(&t.word).is_none())
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

    // The known entities may resolve to LOADED concepts (this is the path "what is
    // <loaded surface>" takes); record which loaded ontologies were reasoned over.
    let reasoned_over = {
        let ids: Vec<ConceptId> = entities
            .iter()
            .flat_map(|e| en.lookup(e).iter().copied())
            .collect();
        loaded_ontologies_of(en, &ids)
    };
    trace_impls::ResponseResult {
        response,
        entities_found: entities,
        taxonomy_checked: None,
        from_ontology: has_knowledge,
        reasoned_over,
    }
}

/// Check if the tokens reference the system itself.
///
/// The self-reference decision is owned by the self-model layer: a token is
/// self-referential iff its surface form is one of the self-model's typed
/// self-referents ([`is_self_referent`] — the system's identity name and the
/// indexicals English resolves to the addressee). The routing body asks the
/// self-model rather than enumerating word literals here.
///
/// SMALLEST TYPED STEP (#186): the membership test is against the self-model's
/// self-referent *surface* set. The fully typed form — resolve each token to a
/// SelfModel `ConceptId`/`SenseId` and test membership in the SelfModel
/// reflexive closure — needs an indexical→SelfModel sense bridge that the
/// pipeline does not yet have; tracked as a follow-up.
fn is_self_referential(tokens: &[pr4xis_domains::cognitive::linguistics::text::Token]) -> bool {
    tokens.iter().any(|t| is_self_referent(&t.word))
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
        // A self-referential answer reasons over the eigenform, not a loaded .prx.
        reasoned_over: Vec::new(),
    }
}

/// The LOADED ontologies the given concepts belong to, deduped (doc §2.3 — the
/// provenance a turn records). Embedded-English concepts contribute nothing
/// (`ontology_of_concept` → `None`); a loaded `.prx` concept contributes its
/// `OntologyName`, so an answer over a USC Title names that Title.
fn loaded_ontologies_of(en: &dyn LexicalReasoner, ids: &[ConceptId]) -> Vec<OntologyName> {
    let mut names = Vec::new();
    for &id in ids {
        if let Some(name) = en.ontology_of_concept(id)
            && !names.contains(&name)
        {
            names.push(name);
        }
    }
    names
}

pub fn answer_question(
    en: &dyn LexicalReasoner,
    predicate: &str,
    arguments: &[montague::Sem],
) -> trace_impls::ResponseResult {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;
    use pr4xis_domains::cognitive::linguistics::relation_lexicon::subsumption_kind;

    let all_entities: Vec<String> = arguments.iter().map(extract_entity_name).collect();

    let entities: Vec<String> = all_entities
        .iter()
        .filter(|e| !en.lookup(e).is_empty())
        .cloned()
        .collect();

    if entities.len() >= 2 {
        let child = &entities[0];
        let parent = &entities[1];

        // Lower the question's surface predicate to its TYPED relation kind
        // through the loaded relation lexicon ("part of" → Parthood); an unknown
        // surface (the bare copula "is") falls back to Subsumption. The relation
        // is loaded data the reasoner interprets — never a `match predicate` in
        // Rust. `child → parent` is the asserted direction (part → whole, subtype
        // → supertype); a relation like Parthood is antisymmetric (BFO:0000050),
        // so the reverse direction is never re-checked.
        let kind = en
            .relation_for_surface(predicate)
            .unwrap_or_else(subsumption_kind);

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
            // The RELATION-PARAMETRIC query: does `child` reach `parent` along
            // `kind`? `is_a` is the `kind = Subsumption` case; a USC Parthood
            // question reads the SAME materialized closure, keyed on Parthood.
            for &cid in child_ids {
                for &pid in parent_ids {
                    if en.reaches(cid, pid, &kind) {
                        return trace_impls::ResponseResult {
                            response: build_taxonomy_response(
                                en,
                                en.surface_for_relation(&kind).as_deref(),
                                child,
                                parent,
                                cid,
                                pid,
                            ),
                            entities_found: entities.clone(),
                            taxonomy_checked: Some((child.clone(), parent.clone(), true)),
                            from_ontology: true,
                            reasoned_over: loaded_ontologies_of(en, &[cid, pid]),
                        };
                    }
                }
            }
            // The negation still TRAVERSED the loaded closure — record it. The
            // denial phrases with the relation's loaded surface ("is not part of"),
            // not "is not a".
            let traversed: Vec<ConceptId> = child_ids.iter().chain(parent_ids).copied().collect();
            let connective = en.surface_for_relation(&kind);
            return trace_impls::ResponseResult {
                response: realize::realize_negation(child, parent, connective.as_deref()),
                entities_found: entities.clone(),
                taxonomy_checked: Some((child.clone(), parent.clone(), false)),
                from_ontology: true,
                reasoned_over: loaded_ontologies_of(en, &traversed),
            };
        }
    }

    if entities.len() == 1 {
        let response = define_word(en, &entities[0]);
        let reasoned_over = loaded_ontologies_of(en, en.lookup(&entities[0]));
        return trace_impls::ResponseResult {
            response,
            entities_found: entities,
            taxonomy_checked: None,
            from_ontology: true,
            reasoned_over,
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
        reasoned_over: Vec::new(),
    }
}

pub fn answer_statement(
    en: &dyn LexicalReasoner,
    _predicate: &str,
    arguments: &[montague::Sem],
) -> trace_impls::ResponseResult {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{self, ResponseContent};
    use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;

    let entities: Vec<String> = arguments.iter().map(extract_entity_name).collect();

    if entities.len() == 1 {
        let ids = en.lookup(&entities[0]);
        if !ids.is_empty() {
            let reasoned_over = loaded_ontologies_of(en, ids);
            let response = define_word(en, &entities[0]);
            return trace_impls::ResponseResult {
                response,
                entities_found: entities,
                taxonomy_checked: None,
                from_ontology: true,
                reasoned_over,
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
        reasoned_over: Vec::new(),
        from_ontology: true,
    }
}

pub fn define_word(en: &dyn LexicalReasoner, word: &str) -> String {
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
    en: &dyn LexicalReasoner,
    // The relation's loaded surface ("part of" for Parthood), already resolved by
    // the caller from the typed kind — so the affirmation phrases "X is part of Y",
    // not "X is a Y". `None` (Subsumption / is-a) keeps the copula "is a". Loaded
    // data, never a hardcoded relation string.
    connective: Option<&str>,
    child_word: &str,
    parent_word: &str,
    child_id: pr4xis_domains::cognitive::linguistics::english::ConceptId,
    parent_id: pr4xis_domains::cognitive::linguistics::english::ConceptId,
) -> String {
    use pr4xis_domains::cognitive::linguistics::pragmatics::realize;

    // ---- Stage 1: Content Determination ----
    // Gather all relevant knowledge from the ontology.

    // The taxonomy chain: how child relates to parent. The ORDERED is-a evidence
    // path is owned by the reasoner's MATERIALIZED hypernym closure — we ask for
    // `ancestor_chain` rather than hand-walking `parents()` in a bounded loop, so
    // even the justification is closure-derived, not re-walked. This function is
    // only reached after `is_a(child, parent)` already proved (see the caller),
    // so the chain is always present; an absent chain degrades to the endpoints
    // rather than re-deriving anything.
    let chain_ids: Vec<(
        String,
        pr4xis_domains::cognitive::linguistics::english::ConceptId,
    )> = en
        .ancestor_chain(child_id, parent_id)
        .unwrap_or_else(|| vec![child_id, parent_id])
        .into_iter()
        .enumerate()
        .map(|(i, id)| {
            // The chain's first element is `child`; render it with the caller's
            // surface word, not the lemma, so the evidence reads in the user's
            // term. Every other rung uses its concept's primary lemma.
            let label = if i == 0 {
                child_word.to_string()
            } else {
                en.concept(id)
                    .map(|c| {
                        c.lemmas
                            .first()
                            .map(|l| l.as_str())
                            .unwrap_or(&c.original_id)
                            .to_string()
                    })
                    .unwrap_or_else(|| parent_word.to_string())
            };
            (label, id)
        })
        .collect();

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

    // Nucleus: the direct assertion — phrased with the relation's loaded connective
    // ("a subsection is part of a section" for Parthood; "is a" for Subsumption).
    sections.push(format!(
        "Yes. {}.",
        realize::sentence_relation(child_word, parent_word, connective)
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
fn explore_concepts(en: &dyn LexicalReasoner, words: &[&str]) -> String {
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

            // Trace the taxonomy chain off the reasoner's MATERIALIZED hypernym
            // closure: `ancestors(id)` is the reflexive is-a image, nearest-
            // first, owned by the closure — never a hand-walk of `parents()`. We
            // drop the reflexive head (`id` itself, distance 0) to render the
            // STRICT ancestor lineage "word is a X → Y → Z".
            let chain: Vec<String> = en
                .ancestors(id)
                .into_iter()
                .skip(1)
                .filter_map(|anc| {
                    en.concept(anc).map(|pc| {
                        pc.lemmas
                            .first()
                            .map(|l| l.as_str())
                            .unwrap_or(&pc.original_id)
                            .to_string()
                    })
                })
                .collect();
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
                    } else if let Some(lca) = en.common_ancestor(id1, id2)
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

pub fn extract_entity_name(sem: &montague::Sem) -> String {
    match sem {
        montague::Sem::Concept { word, .. } => word.clone(),
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

/// The always-present substrate ontologies: the auto-registered
/// knowledge base plus the embedded English language model.
///
/// Source-agnostic — it knows nothing of statutes or any loaded corpus.
/// Loaded corpora are reported separately through the self-model catalog
/// (`SelfModelInstance::with_catalog`); a caller that has loaded sources
/// attaches them there. English is the one embedded base, so it is the
/// only runtime corpus reflected here directly.
pub fn loaded_ontologies(_lang: &English) -> Vec<Vocabulary> {
    let mut ontologies = describe_knowledge_base();
    ontologies.push(Vocabulary::from_ontology::<
        pr4xis_domains::cognitive::linguistics::lexicon::ontology::LexicalCategory,
        pr4xis_domains::cognitive::linguistics::lexicon::pos::PosTag,
    >(
        "English (WordNet)",
        "pr4xis_domains::cognitive::linguistics::english",
        "Open English WordNet 2025; Princeton WordNet",
    ));
    ontologies
}

/// The eigenform — the system observes itself.
///
/// This IS the self-observation operator F from von Foerster.
/// The result IS the fixed point X = F(X). The returned
/// [`SelfModelInstance`] carries the substrate ontologies; callers
/// attach the loaded-source catalog (the knowledge boundary) via
/// [`SelfModelInstance::with_catalog`].
pub fn observe_self(lang: &English) -> SelfModelInstance {
    observe_self_with_loaded(lang, Vec::new())
}

/// The eigenform over the LIVE loaded set — `F` applied to the object level it
/// actually has, not a constant (doc §2). The substrate ontologies
/// ([`loaded_ontologies`]) PLUS one `Vocabulary` per loaded runtime ontology
/// (built by the caller via
/// [`runtime_ontology_vocabulary`](pr4xis_domains::formal::information::knowledge::runtime_ontology_vocabulary)),
/// so `total_concepts`/`total_morphisms` MOVE the moment a corpus is loaded — the
/// self-model is causally connected to what is loaded, not a vacuous fixed point.
/// `loaded` is empty for the English-only path (then this is [`observe_self`]).
pub fn observe_self_with_loaded(lang: &English, loaded: Vec<Vocabulary>) -> SelfModelInstance {
    let mut components = loaded_ontologies(lang);
    components.extend(loaded);
    SelfModelInstance::observe(components)
}

/// Describe the eigenform structurally. Callers that need JSON (WASM
/// boundary) should call `.to_json()` on the result themselves, and may
/// first attach the source catalog via
/// [`SelfModelInstance::with_catalog`].
pub fn self_describe(lang: &English) -> SelfModelInstance {
    observe_self(lang)
}

/// [`self_describe`] over the live loaded set — see [`observe_self_with_loaded`].
pub fn self_describe_with_loaded(lang: &English, loaded: Vec<Vocabulary>) -> SelfModelInstance {
    observe_self_with_loaded(lang, loaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Monoid;
    use pr4xis_domains::formal::information::diagnostics::trace_functors::PipelineStep;

    fn sample_english() -> English {
        // Use sample data for unit tests (fast, no WordNet needed)
        English::sample()
    }

    #[test]
    fn observe_self_with_loaded_moves_the_totals() {
        // The eigenform's causal connection (doc §2): feeding the live loaded set
        // in MOVES total_concepts/morphisms and adds the ontology as a component —
        // the self-model is no longer blind to what is loaded.
        use pr4xis::ontology::Vocabulary;
        use pr4xis::ontology::meta::{ConceptName, Morphism, MorphismKind};

        let english = sample_english();
        let base = observe_self(&english);

        let loaded = Vocabulary::from_captured(
            "TestCorpus".to_string(),
            "test::corpus",
            "test fixture",
            vec![
                ConceptName::new("a"),
                ConceptName::new("b"),
                ConceptName::new("c"),
            ],
            vec![Morphism::new(
                ConceptName::new("a"),
                ConceptName::new("b"),
                MorphismKind::Subsumption,
            )],
        );
        let with = observe_self_with_loaded(&english, vec![loaded]);

        assert_eq!(
            with.total_concepts,
            base.total_concepts + 3,
            "loading 3 concepts moves the eigenform's concept total"
        );
        assert_eq!(
            with.total_morphisms,
            base.total_morphisms + 1,
            "loading 1 morphism moves the eigenform's morphism total"
        );
        assert!(
            with.components.iter().any(|v| v.name() == "TestCorpus"),
            "the loaded ontology is a component the self-model observes"
        );
        // English-only is unchanged — the old eigenform, with an empty loaded set.
        assert_eq!(observe_self(&english).total_concepts, base.total_concepts);
    }

    // --- Algebraic structure integration tests ---

    #[test]
    fn pipeline_trace_is_monoid() {
        // PipelineTrace forms a monoid under concatenation.
        // This enables Writer<PipelineTrace, A> — the pipeline IS a writer monad.
        let empty = PipelineTrace::empty();
        let t1 = PipelineTrace::single(PipelineStep::TOKENIZE, "5 tokens", true);
        let t2 = PipelineTrace::single(PipelineStep::PARSE, "success", true);

        // Left identity: empty ++ t = t
        assert_eq!(empty.combine(&t1).entries.len(), 1);
        // Right identity: t ++ empty = t
        assert_eq!(t1.combine(&empty).entries.len(), 1);
        // Associativity
        let t3 = PipelineTrace::single(PipelineStep::INTERPRET, "question", true);
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
            PipelineTrace::single(PipelineStep::TOKENIZE, "5 tokens", true),
        );

        let result = step1
            .bind(|count| {
                pr4xis::category::Writer::new(
                    count > 0,
                    PipelineTrace::single(PipelineStep::PARSE, "parsed", true),
                )
            })
            .bind(|parsed| {
                let msg = if parsed { "question" } else { "unknown" };
                pr4xis::category::Writer::new(
                    msg,
                    PipelineTrace::single(PipelineStep::INTERPRET, msg, parsed),
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
        let combined = child_ids.map2(parent_ids, Product::new);
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

    /// Per memory `feedback_ontological_assertions.md`: each test claim is
    /// an `Axiom` impl in the domain (here `knowledge::instance`); the
    /// `#[test]` is a thin wrapper. The claim is then discoverable via
    /// `Ontology::axioms()`, citable, and reusable.
    #[test]
    fn self_describe_has_ontologies() {
        use pr4xis::ontology::Axiom;
        use pr4xis_domains::formal::information::knowledge::instance::{
            KnowledgeBaseIsNonEmpty, KnowledgeIsRegistered, SelfModelIsRegistered,
        };

        // Exercise the chat surface — confirms `self_describe` returns a
        // structural `SelfModelInstance` that can be observed downstream.
        let _ = self_describe(&sample_english());

        for axiom in [
            &KnowledgeBaseIsNonEmpty as &dyn Axiom,
            &SelfModelIsRegistered,
            &KnowledgeIsRegistered,
        ] {
            assert!(axiom.verify().is_ok(), "{}", axiom.description());
        }
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

    // --- Phase 2: RelationshipMeta + TracedPipeline integration tests ---

    #[test]
    fn trace_meta_is_from_diagnostic_ontology() {
        // trace_meta() comes from DiagnosticOntology::meta() — generated by
        // ontology!. The ontology identifies itself through the macro,
        // not through hand-written strings.
        let m = trace_meta();
        assert_eq!(m.name.as_str(), "DiagnosticOntology");
        assert!(m.module_path.as_str().contains("diagnostics"));
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
        assert_eq!(result.trace.entries[0].step, PipelineStep::TOKENIZE);
        // Last entry is Realization — the final step of the writer chain.
        let last = result.trace.entries.last().unwrap();
        assert_eq!(last.step, PipelineStep::REALIZATION);
    }

    #[test]
    fn empty_input_still_produces_traceable_output() {
        // Empty-branch early return: the trace carries the tokenize result
        // constructed via `PipelineTrace::from_traceable`, not a mutation.
        let en = sample_english();
        let result = process_with_metadata(&en, "");
        assert_eq!(result.token_count, 0);
        assert_eq!(result.trace.entries.len(), 1);
        assert_eq!(result.trace.entries[0].step, PipelineStep::TOKENIZE);
    }

    #[test]
    fn writer_tell_preserves_trace_order() {
        // The writer monad's `tell` preserves order because Vec monoid
        // concatenation is left-associative. Verifying directly:
        let pipeline: TracedPipeline<()> = Writer::pure(())
            .tell(PipelineTrace::single(PipelineStep::TOKENIZE, "a", true))
            .tell(PipelineTrace::single(PipelineStep::PARSE, "b", true))
            .tell(PipelineTrace::single(PipelineStep::INTERPRET, "c", true));
        assert_eq!(pipeline.log.entries.len(), 3);
        assert_eq!(pipeline.log.entries[0].step, PipelineStep::TOKENIZE);
        assert_eq!(pipeline.log.entries[1].step, PipelineStep::PARSE);
        assert_eq!(pipeline.log.entries[2].step, PipelineStep::INTERPRET);
    }
}

// =========================================================================
// Loaded-corpus demo — the with/without behavioral contrast
// =========================================================================
//
// The heart of the runtime demo: a chat that answers a question about the
// CONTENT of a LOADED ontology, grounded through English, and abstains when
// that ontology is not loaded. The same question, the same chat code — only
// the reasoner differs:
//
//   - WITHOUT the corpus  →  `process(english, …)`            →  abstains
//   - WITH the corpus     →  `process_with_reasoner(english, composed, …)`
//                                                              →  answers from
//                                                                 the loaded gloss
//
// The corpus is a REAL compiled praxis ontology projected to a `.prx` Archive
// via `emit::<Cat>()`, materialized into a `RuntimeOntology`, and grounded into
// the English lexicon by the `ComposedReasoner` (the Lemon functor). Nothing is
// hardcoded: the answer text the WITH case asserts is the ontology's OWN gloss,
// read back through `RuntimeOntology::lexical`.
#[cfg(test)]
mod loaded_corpus_demo {
    use super::*;
    use pr4xis::ontology::meta::OntologyName;
    use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
    use pr4xis_runtime::emit::emit;
    use pr4xis_runtime::ontology::{RuntimeOntology, materialize};

    // A small REAL legal-corpus ontology: a statute is divided into Titles,
    // which are divided into Sections. Compiled by the same `ontology!` macro
    // every domain ontology uses — labels (the lexical glosses) and the
    // materialized Subsumption taxonomy and all.
    pr4xis::ontology! {
        name: "Statute",
        source: "loaded-corpus demo fixture (US Code structural skeleton: Title → Section)",
        concepts: [Statute, Title, Section],
        labels: {
            Statute: ("en", "Statute",
                "A written law enacted by a legislature."),
            Title: ("en", "Title",
                "A formal top-level subdivision of a statute, grouping related sections."),
            Section: ("en", "Section",
                "The smallest numbered unit of a statute, stating a single rule."),
        },
        is_a: [
            // A Section is part of the statute's structure; a Title is too. For
            // the demo we only need the lexical grounding, but a real taxonomy
            // edge keeps the fixture honest (Title and Section are Statute
            // structural units).
            (Title, Statute),
            (Section, Statute),
        ],
    }

    /// Materialize the Statute ontology into a `RuntimeOntology`.
    ///
    /// `emit::<StatuteCategory>()` now carries each concept's gloss INTO the
    /// `.prx` itself (via `Concept::lexical`, filled from the `ontology!` labels
    /// table) — so the demo no longer post-processes the archive to attach
    /// glosses from `labels()`. The gloss travels with the projected ontology,
    /// exactly as it must for a `.prx` loaded with no compile-time labels table
    /// (e.g. in the browser).
    fn statute_corpus() -> RuntimeOntology {
        let archive = emit::<StatuteCategory>();
        materialize(archive, OntologyName::new_static("Statute"))
            .expect("Statute corpus materializes")
    }

    /// The gloss the loaded ontology carries for `Title` — the exact text the
    /// WITH case must surface, read back from the materialized ontology's own
    /// `lexical` (i.e. from what `emit` carried into the `.prx`), never from a
    /// compile-time side table.
    fn title_gloss() -> String {
        let corpus = statute_corpus();
        corpus
            .lexical(&corpus.concept("Title"))
            .expect("the emitted Statute ontology carries Title's gloss")
            .to_string()
    }

    #[test]
    fn what_is_a_title_answers_from_the_loaded_gloss_with_the_corpus_and_abstains_without() {
        let english = English::sample();
        let question = "what is a title";
        let gloss = title_gloss();

        // --- WITHOUT the corpus: english-only. "title" is not an English word
        //     the sample model knows, so the chat abstains. ---
        let (without, _, _) = process(&english, question);
        assert!(
            without.to_lowercase().contains("do not")
                || without.to_lowercase().contains("don't")
                || without.to_lowercase().contains("not know"),
            "english-only must abstain on an unloaded concept; got: {without:?}"
        );
        assert!(
            !without.contains(gloss.as_str()),
            "english-only must NOT surface the loaded gloss (it isn't loaded); got: {without:?}"
        );

        // --- WITH the corpus: ground the loaded Statute ontology into English
        //     via the ComposedReasoner, then ask the SAME question through the
        //     SAME pipeline. The answer is the loaded gloss. ---
        let composed = ComposedReasoner::new(English::sample(), vec![statute_corpus()]);
        let with = process_with_reasoner(&english, &composed, question).response;
        assert!(
            with.contains(gloss.as_str()),
            "with the corpus loaded, the chat must answer from the loaded Title gloss \
             ({gloss:?}); got: {with:?}"
        );
        assert!(
            with.to_lowercase().contains("title"),
            "the answer must name the queried concept; got: {with:?}"
        );

        // The contrast is the whole demo: same question, same code, opposite
        // epistemic outcome — grounded entirely through the lexicon, never a
        // hardcoded branch.
        assert_ne!(without, with, "loading the corpus must change the answer");
    }

    #[test]
    fn a_multi_token_loaded_surface_resolves_through_the_chat() {
        // The multi-token (phrase/citation) recognizer end-to-end: a loaded
        // ontology whose surface is MULTI-WORD ("section 1514a") becomes answerable
        // in chat, where word-by-word tokenization alone would split + miss it.
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::Definition;

        let english = English::sample();
        let question = "what is section 1514a";
        let gloss = "Civil action to protect against retaliation in fraud cases.";

        // english-only: max_surface_words == 1, so no collapse — "section" and
        // "1514a" are looked up separately and miss → the chat abstains.
        let (without, _, _) = process(&english, question);
        assert!(
            without.to_lowercase().contains("not") || without.to_lowercase().contains("don't"),
            "english-only must abstain on the multi-token citation; got: {without:?}"
        );

        // composed: a loaded ontology indexes the multi-word surface (its node
        // name); the recognizer collapses ["section","1514a"] into ONE proper-noun
        // token and the chat answers from the node's gloss — the §9 use case.
        let archive = Archive {
            nodes: vec![Definition {
                kind: "Concept".to_string(),
                name: "section 1514a".to_string(),
                edges: vec![],
                axioms: vec![],
                lexical: Some(gloss.to_string()),
            }],
            connections: vec![],
        };
        let onto = materialize(archive, OntologyName::new_static("usc_test"))
            .expect("the multi-word-named ontology materializes");
        let composed = ComposedReasoner::new(English::sample(), vec![onto]);
        assert!(
            composed.max_surface_words() >= 2,
            "the loaded surface 'section 1514a' is multi-word, so the recognizer is active"
        );

        let with = process_with_reasoner(&english, &composed, question).response;
        assert!(
            with.contains(gloss),
            "the multi-token citation must collapse + resolve to the loaded gloss; got: {with:?}"
        );
        assert_ne!(
            without, with,
            "the multi-token recognizer changes an abstention into an answer"
        );
    }

    #[test]
    fn an_owl_label_makes_a_class_chat_answerable() {
        // The §9 OWL path end-to-end: an OWL entity is answerable by its
        // `rdfs:label` (minted as a `canonicalForm` Form by the owl bridge), not
        // only its opaque IRI. A MULTI-WORD label exercises §9 + the phrase-lookup
        // together.
        use pr4xis_domains::social::software::markup::xml::owl::bridge::owl_runtime_ontology;
        use pr4xis_domains::social::software::markup::xml::owl::reader::read_owl;
        use pr4xis_domains::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary;

        const OWL: &str = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#">
  <owl:ObjectProperty rdf:about="http://purl.org/spar/cito/citesAsEvidence">
    <rdfs:label>cites as evidence</rdfs:label>
    <rdfs:comment>The citing entity cites the cited entity as evidence.</rdfs:comment>
  </owl:ObjectProperty>
</rdf:RDF>"#;

        let english = English::sample();
        let ont = read_owl(OWL).expect("parse OWL");
        let vocab = LoadedOwlVocabulary::from_owl_ontology(&ont);
        let onto = owl_runtime_ontology(&vocab, OntologyName::new_static("cito"))
            .expect("the OWL vocabulary materializes");
        let composed = ComposedReasoner::new(English::sample(), vec![onto]);

        let resp = process_with_reasoner(&english, &composed, "what is cites as evidence").response;
        assert!(
            resp.to_lowercase().contains("citing"),
            "the OWL property must answer from its rdfs:comment gloss via its label \
             surface; got: {resp:?}"
        );
    }

    #[test]
    fn grounding_unions_the_loaded_surface_into_the_lexicon() {
        // The Lemon grounding is what makes "title" resolvable at all: english
        // alone returns nothing for it; the composed reasoner returns the loaded
        // concept id (typed-disjoint from English's), and `define_word` reads
        // its gloss straight from the materialized ontology.
        let composed = ComposedReasoner::new(English::sample(), vec![statute_corpus()]);

        assert!(
            English::sample().lookup("title").is_empty(),
            "precondition: the embedded model does not know 'title'"
        );
        let ids = composed.lookup("title");
        assert!(
            !ids.is_empty(),
            "the grounded lexicon must resolve the loaded surface 'title'"
        );

        // `define_word` over the composed reasoner reads the loaded gloss.
        let defined = define_word(&composed, "title");
        assert!(
            defined.contains(title_gloss().as_str()),
            "define_word must surface the loaded gloss; got: {defined:?}"
        );

        // And the Lemon lexicon carries the typed reference (ontology + name).
        let label = composed.lexicon().label_for("Statute", "Title");
        assert_eq!(
            label,
            Some("title"),
            "the grounded entry's surface form is the lowercased node name"
        );
    }

    #[test]
    fn a_loaded_answer_names_the_loaded_ontology_in_its_provenance() {
        // Doc §2.3 — the "Title-names-Title" deliverable: a turn that answers from
        // a loaded `.prx` (the Statute corpus) NAMES it in the trace provenance,
        // by its OntologyName, success-marked. English-only names no loaded ontology.
        use pr4xis_domains::formal::information::diagnostics::trace_functors::TraceOntology;

        let english = English::sample();
        let composed = ComposedReasoner::new(English::sample(), vec![statute_corpus()]);

        let with = process_with_reasoner(&english, &composed, "what is a title");
        let provenance = with.trace.reasoned_over();
        assert!(
            provenance.iter().any(
                |(o, ok)| matches!(o, TraceOntology::Loaded(n) if n.as_str() == "Statute") && *ok
            ),
            "the trace must name the loaded Statute ontology it reasoned over; got: {provenance:?}"
        );

        // English-only reasons over no loaded ontology — the contrast.
        let without = process_with_reasoner(&english, &English::sample(), "what is a title");
        assert!(
            !without
                .trace
                .reasoned_over()
                .iter()
                .any(|(o, _)| matches!(o, TraceOntology::Loaded(_))),
            "english-only must name no loaded ontology in its provenance"
        );
    }

    #[test]
    fn abstention_is_a_typed_outcome_naming_what_to_load() {
        // Doc §4.1 (highest): a self-aware system models what it CANNOT answer.
        // With the corpus, "title" resolves → Answered; without it, the turn
        // ABSTAINS and NAMES the unresolved surface (what to load), as a value —
        // not a string the UI has to sniff.
        let english = English::sample();
        let composed = ComposedReasoner::new(English::sample(), vec![statute_corpus()]);

        let answered = process_with_reasoner(&english, &composed, "what is a title");
        assert_eq!(
            answered.outcome,
            ChatOutcome::Answered,
            "with the corpus, the loaded concept answers; got {:?}",
            answered.outcome
        );

        let abstained = process_with_reasoner(&english, &English::sample(), "what is a title");
        match abstained.outcome {
            ChatOutcome::Abstained { unresolved } => assert!(
                unresolved.iter().any(|s| s == "title"),
                "abstention must name the unresolved surface to load; got {unresolved:?}"
            ),
            ChatOutcome::Answered => {
                panic!("english-only must abstain on the unloaded concept 'title'")
            }
        }
    }

    /// A loaded ontology carrying a PARTHOOD mereology in the USC orientation
    /// (part → whole): a `subsection` is part of a `section`. Hand-built so the
    /// edge direction is explicit and matches `uslm::corpus::bridge` (a
    /// subdivision Composes INTO its parent → Parthood part→whole) — NOT the
    /// `ontology!` `has_a:` sugar, which orients whole→part (has-part), the
    /// inverse. `materialize` folds the edge into the transitive Parthood closure.
    fn parthood_corpus() -> RuntimeOntology {
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::{Definition, EdgeTarget};
        let archive = Archive {
            nodes: vec![
                Definition {
                    kind: "Concept".to_string(),
                    name: "subsection".to_string(),
                    edges: vec![(
                        "Parthood".to_string(),
                        EdgeTarget::Local("section".to_string()),
                    )],
                    axioms: vec![],
                    lexical: Some("A lettered subdivision of a section.".to_string()),
                },
                Definition {
                    kind: "Concept".to_string(),
                    name: "section".to_string(),
                    edges: vec![],
                    axioms: vec![],
                    lexical: Some("The smallest numbered unit of a statute.".to_string()),
                },
            ],
            connections: vec![],
        };
        materialize(archive, OntologyName::new_static("PartCorpus"))
            .expect("the Parthood corpus materializes")
    }

    /// Two `Sem::Concept` (NP) arguments naming the entities of a relational
    /// question — what `answer_question` reads via `extract_entity_name`.
    fn entity_args(child: &str, parent: &str) -> [montague::Sem; 2] {
        [
            montague::Sem::Concept {
                word: child.to_string(),
                concepts: Vec::new(),
            },
            montague::Sem::Concept {
                word: parent.to_string(),
                concepts: Vec::new(),
            },
        ]
    }

    #[test]
    fn is_subsection_part_of_section_answers_yes_over_a_loaded_parthood_mereology() {
        // Step 3 end-to-end: the relational predicate "part of" resolves through
        // the loaded relation lexicon to the Parthood kind, and the reasoner reads
        // the loaded ontology's MATERIALIZED Parthood closure — the structural
        // query Track C + §9 left dark (gloss worked; "is X part of Y" did not).
        let composed = ComposedReasoner::new(English::sample(), vec![parthood_corpus()]);
        let args = entity_args("subsection", "section");

        let yes = answer_question(&composed, "part of", &args);
        assert_eq!(
            yes.taxonomy_checked,
            Some(("subsection".to_string(), "section".to_string(), true)),
            "a subsection IS part of its section (Parthood closure); got {:?}",
            yes.taxonomy_checked
        );
        // The affirmation PHRASES the relation it answered (its loaded surface),
        // not "is a" — Parthood is not Subsumption (the realize.rs fix).
        assert!(
            yes.response.contains("is part of"),
            "a Parthood affirmation must read 'is part of', not 'is a'; got: {:?}",
            yes.response
        );

        // English-only cannot witness Parthood: it has no relation lexicon
        // (relation_for_surface → None → Subsumption) and no loaded closure.
        let en_only = answer_question(&English::sample(), "part of", &args);
        assert_ne!(
            en_only.taxonomy_checked,
            Some(("subsection".to_string(), "section".to_string(), true)),
            "english-only must not affirm a Parthood it cannot witness"
        );
    }

    #[test]
    fn is_section_part_of_subsection_answers_no_parthood_is_directional() {
        // Parthood is antisymmetric (BFO:0000050): the whole is NOT part of its
        // part. The reverse-direction convenience that would mask this is gone.
        let composed = ComposedReasoner::new(English::sample(), vec![parthood_corpus()]);
        let no = answer_question(&composed, "part of", &entity_args("section", "subsection"));
        assert_eq!(
            no.taxonomy_checked,
            Some(("section".to_string(), "subsection".to_string(), false)),
            "a section is NOT part of its subsection; got {:?}",
            no.taxonomy_checked
        );
        // The denial phrases the relation too: "is not part of", not "is not a".
        assert!(
            no.response.contains("not part of"),
            "a Parthood denial must read 'is not part of'; got: {:?}",
            no.response
        );
    }

    /// A Parthood corpus with MULTI-WORD citation surfaces (the USC reality:
    /// "section ninety", "title fifteen") — so the recognizer collapses each to a
    /// proper-noun NP and "is X part of Y" parses from raw text.
    fn parthood_corpus_multiword() -> RuntimeOntology {
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::{Definition, EdgeTarget};
        let archive = Archive {
            nodes: vec![
                Definition {
                    kind: "Concept".to_string(),
                    name: "section ninety".to_string(),
                    edges: vec![(
                        "Parthood".to_string(),
                        EdgeTarget::Local("title fifteen".to_string()),
                    )],
                    axioms: vec![],
                    lexical: Some("A section within title fifteen.".to_string()),
                },
                Definition {
                    kind: "Concept".to_string(),
                    name: "title fifteen".to_string(),
                    edges: vec![],
                    axioms: vec![],
                    lexical: Some("Commerce and trade.".to_string()),
                },
            ],
            connections: vec![],
        };
        materialize(archive, OntologyName::new_static("PartCorpusMW"))
            .expect("the multi-word Parthood corpus materializes")
    }

    #[test]
    fn is_x_part_of_y_parses_and_answers_from_raw_chat_text() {
        // Step 5 — the WHOLE path from RAW TEXT: tokenize → collapse "section
        // ninety"/"title fifteen" to NPs + "part of" to a relational predicate →
        // the predicative question-copula parses S[q] → interpret lifts the
        // relation from the complement → answer_question reads the Parthood
        // closure. No constructed Sem — a person types the question.
        let english = English::sample();
        let composed = ComposedReasoner::new(English::sample(), vec![parthood_corpus_multiword()]);
        let question = "is section ninety part of title fifteen";

        let with = process_with_reasoner(&english, &composed, question);

        assert!(
            with.parsed,
            "the predicative question-copula must parse 'is X part of Y'; got: {:?}",
            with.response
        );
        assert!(
            with.response.to_lowercase().contains("yes"),
            "section ninety IS part of title fifteen — the Parthood closure affirms; got: {:?}",
            with.response
        );
        // The loaded gloss rides the answer (the concepts resolved, not guessed).
        assert!(
            with.response.contains("Commerce and trade"),
            "the affirmation surfaces the loaded gloss; got: {:?}",
            with.response
        );

        // english-only abstains (it knows none of these surfaces and has no
        // relation lexicon) — the contrast that proves the corpus + lexicon did it.
        let (without, _, _) = process(&english, question);
        assert!(
            without.to_lowercase().contains("not") || without.to_lowercase().contains("don't"),
            "english-only must abstain on the loaded relational question; got: {without:?}"
        );
        assert_ne!(
            without, with.response,
            "loading the corpus + relation lexicon must change the answer"
        );
    }

    #[test]
    fn is_x_part_of_y_parses_with_single_word_loaded_entities() {
        // The single-word-np fix: a SINGLE-word loaded entity ("subsection",
        // "section" — not a multi-word citation) now offers an NP reading, so
        // "is X part of Y" parses from raw text. Gated on loaded membership, so
        // English function words are untouched (process_taxonomy_question etc. green).
        let english = English::sample();
        let composed = ComposedReasoner::new(English::sample(), vec![parthood_corpus()]);
        let question = "is subsection part of section";

        let with = process_with_reasoner(&english, &composed, question);
        assert!(
            with.parsed,
            "single-word loaded entities must type NP so 'is X part of Y' parses; got: {:?}",
            with.response
        );
        assert!(
            with.response.to_lowercase().contains("yes") && with.response.contains("is part of"),
            "a subsection IS part of a section; got: {:?}",
            with.response
        );

        // English-only abstains — the contrast (it has no loaded entity, no NP upgrade).
        let (without, _, _) = process(&english, question);
        assert_ne!(
            without, with.response,
            "the loaded corpus must change the answer"
        );
    }

    #[test]
    fn part_of_and_is_a_read_distinct_closures_over_the_same_pair() {
        // The Smith et al. (2005) part_of ≠ is_a distinction, end-to-end: the
        // subsection→section edge is Parthood, so "is X part of Y" is true but
        // "is X a Y" (the bare copula "is" → Subsumption fallback) is false.
        let composed = ComposedReasoner::new(English::sample(), vec![parthood_corpus()]);
        let args = entity_args("subsection", "section");

        assert_eq!(
            answer_question(&composed, "part of", &args).taxonomy_checked,
            Some(("subsection".to_string(), "section".to_string(), true)),
            "part of → Parthood → true"
        );
        assert_eq!(
            answer_question(&composed, "is", &args).taxonomy_checked,
            Some(("subsection".to_string(), "section".to_string(), false)),
            "is → Subsumption fallback → false (the edge is Parthood, not is-a)"
        );
    }
}

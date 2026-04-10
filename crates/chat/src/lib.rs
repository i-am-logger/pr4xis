use praxis::ontology::upper::being::Being;
use praxis_domains::science::cognition::epistemics;
use praxis_domains::science::information::knowledge::{
    SelfModelInstance, VocabularyDescriptor, describe_knowledge_base,
};
use praxis_domains::science::linguistics::english::English;
use praxis_domains::science::linguistics::lambek::{
    ReductionResult, TypedToken, montague, reduce::chart_reduce, tokenize,
};
use praxis_domains::science::linguistics::pragmatics::speech_act::SpeechAct;

// Praxis Chat Engine — shared logic for CLI, WASM, and any frontend.
//
// Zero I/O. Takes a string, returns a string.
// All intelligence comes from the Language ontology.
// The chat engine is a functor: Input → Language → Response.

/// Result of processing input through the linguistics pipeline.
pub struct ProcessResult {
    pub response: String,
    pub user_act: SpeechAct,
    pub system_act: SpeechAct,
    /// Processing time in microseconds.
    pub duration_us: u64,
    /// Number of tokens processed.
    pub token_count: usize,
    /// Whether the parser succeeded.
    pub parsed: bool,
    /// Metacognition trace — the epistemic path taken.
    pub trace: String,
}

/// Process input through the full linguistics pipeline.
/// Returns (response_text, user_speech_act, system_speech_act).
pub fn process(lang: &English, input: &str) -> (String, SpeechAct, SpeechAct) {
    let result = process_with_metadata(lang, input);
    (result.response, result.user_act, result.system_act)
}

/// Process with full metadata — timing, token count.
pub fn process_with_metadata(lang: &English, input: &str) -> ProcessResult {
    // Note: std::time::Instant panics on wasm32-unknown-unknown.
    // Use a wrapper that returns 0 on unsupported platforms.
    let start = WasmSafeTimer::now();

    // Tokenize with ALL types per word (chart parser input).
    let (tokens, alternatives) = tokenize::tokenize_with_alternatives(input, lang);
    let token_count = tokens.len();
    if tokens.is_empty() {
        return ProcessResult {
            response: "Empty input received.".into(),
            user_act: SpeechAct::Assertion,
            system_act: SpeechAct::Assertion,
            duration_us: start.elapsed_us(),
            token_count: 0,
            parsed: false,
            trace: "empty input".into(),
        };
    }

    // CYK chart parser — tries all type combinations simultaneously.
    let words: Vec<String> = tokens.iter().map(|t| t.word.clone()).collect();
    let type_sets: Vec<Vec<_>> = tokens
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let mut types = vec![t.lambek_type.clone()];
            if let Some(alts) = alternatives.get(i) {
                for alt in alts {
                    if !types.contains(alt) {
                        types.push(alt.clone());
                    }
                }
            }
            types
        })
        .collect();
    let reduction = chart_reduce(&words, &type_sets);

    // Montague uses WINNING types from chart backtracking.
    let montague_tokens = if reduction.success && reduction.remaining.len() == tokens.len() {
        &reduction.remaining
    } else {
        &tokens
    };
    let meaning = montague::interpret(montague_tokens, lang);
    let parsed = reduction.success;
    let duration_us = start.elapsed_us();

    let token_strs: Vec<String> = tokens
        .iter()
        .map(|t| format!("{}:{:?}", t.word, t.lambek_type))
        .collect();
    let trace = format!(
        "tokenize: [{}] | parse: {} ({:?}) | interpret: {:?}",
        token_strs.join(", "),
        if parsed { "OK" } else { "FAIL" },
        reduction.final_type,
        meaning,
    );

    match &meaning {
        montague::Sem::Question {
            predicate,
            arguments,
        } => {
            let response = answer_question(lang, predicate, arguments);
            ProcessResult {
                response,
                user_act: SpeechAct::Question,
                system_act: SpeechAct::Assertion,
                duration_us,
                token_count,
                parsed,
                trace,
            }
        }

        montague::Sem::Prop {
            predicate,
            arguments,
        } => {
            let response = answer_statement(lang, predicate, arguments);
            ProcessResult {
                response,
                user_act: SpeechAct::Assertion,
                system_act: SpeechAct::Assertion,
                duration_us,
                token_count,
                parsed,
                trace,
            }
        }

        _ => {
            let response = attempt_partial_understanding(lang, &tokens, &reduction, &meaning);
            ProcessResult {
                response,
                user_act: SpeechAct::Assertion,
                system_act: SpeechAct::Assertion,
                duration_us,
                token_count,
                parsed,
                trace,
            }
        }
    }
}

fn attempt_partial_understanding(
    en: &English,
    tokens: &[TypedToken],
    reduction: &ReductionResult,
    meaning: &montague::Sem,
) -> String {
    use praxis_domains::science::linguistics::language::Language;

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

    use praxis_domains::science::linguistics::pragmatics::realize::{self, ResponseContent};
    use praxis_domains::science::linguistics::pragmatics::response::ResponseFrame;

    let frame = ResponseFrame::from_epistemic(&state);

    match state {
        epistemics::EpistemicState::UnknownKnown => {
            if known_words.len() == 1 {
                return define_word(en, known_words[0]);
            }
            let nouns: Vec<&str> = tokens
                .iter()
                .filter(|t| !en.lookup(&t.word).is_empty() && t.lambek_type.is_noun())
                .map(|t| t.word.as_str())
                .collect();
            let mut content = ResponseContent::new(frame);
            if nouns.len() >= 2 {
                content = content.with_entity(nouns[0]).with_entity(nouns[1]);
            } else {
                for w in &known_words {
                    content = content.with_entity(w);
                }
            }
            realize::realize(&content)
        }
        epistemics::EpistemicState::KnownUnknown => {
            let mut content = ResponseContent::new(frame);
            for w in &unknown_words {
                content = content.with_entity(w);
            }
            realize::realize(&content)
        }
        epistemics::EpistemicState::KnownKnown => {
            let content = ResponseContent::new(frame).with_predicate(&meaning.describe());
            realize::realize(&content)
        }
        epistemics::EpistemicState::UnknownUnknown => {
            realize::realize(&ResponseContent::new(frame))
        }
    }
}

pub fn answer_question(en: &English, predicate: &str, arguments: &[montague::Sem]) -> String {
    use praxis_domains::science::linguistics::pragmatics::realize::{self, ResponseContent};
    use praxis_domains::science::linguistics::pragmatics::response::ResponseFrame;

    let all_entities: Vec<String> = arguments.iter().map(extract_entity_name).collect();

    // Filter to content entities (in WordNet) — skip function words like "what", "is"
    let entities: Vec<String> = all_entities
        .iter()
        .filter(|e| !en.lookup(e).is_empty())
        .cloned()
        .collect();

    if entities.len() >= 2 {
        let child = &entities[0];
        let parent = &entities[1];

        let child_ids = en.lookup(child);
        let parent_ids = en.lookup(parent);

        if !child_ids.is_empty() && !parent_ids.is_empty() {
            for &cid in child_ids {
                for &pid in parent_ids {
                    if en.is_a(cid, pid) {
                        let c_def = en
                            .concept(cid)
                            .and_then(|c| c.definitions.first())
                            .map(|d| d.as_str())
                            .unwrap_or("");
                        let p_def = en
                            .concept(pid)
                            .and_then(|p| p.definitions.first())
                            .map(|d| d.as_str())
                            .unwrap_or("");
                        let content = ResponseContent::new(ResponseFrame::AssertKnowledge)
                            .with_predicate("is_a")
                            .with_entity(child)
                            .with_entity(parent)
                            .with_definition(child, c_def)
                            .with_definition(parent, p_def);
                        return realize::realize(&content);
                    }
                }
            }
            return realize::realize_negation(child, parent);
        }

        if !parent_ids.is_empty() && !child_ids.is_empty() {
            for &cid in parent_ids {
                for &pid in child_ids {
                    if en.is_a(cid, pid) {
                        let content = ResponseContent::new(ResponseFrame::AssertKnowledge)
                            .with_predicate("is_a")
                            .with_entity(parent)
                            .with_entity(child);
                        return realize::realize(&content);
                    }
                }
            }
        }
    }

    if entities.len() == 1 {
        return define_word(en, &entities[0]);
    }

    let mut content = ResponseContent::new(ResponseFrame::AcknowledgeGap).with_predicate(predicate);
    for e in &entities {
        content = content.with_entity(e);
    }
    realize::realize(&content)
}

pub fn answer_statement(en: &English, _predicate: &str, arguments: &[montague::Sem]) -> String {
    use praxis_domains::science::linguistics::pragmatics::realize::{self, ResponseContent};
    use praxis_domains::science::linguistics::pragmatics::response::ResponseFrame;

    let entities: Vec<String> = arguments.iter().map(extract_entity_name).collect();

    if entities.len() == 1 {
        let ids = en.lookup(&entities[0]);
        if !ids.is_empty() {
            return define_word(en, &entities[0]);
        }
    }

    let mut content = ResponseContent::new(ResponseFrame::AssertKnowledge);
    for e in &entities {
        content = content.with_entity(e);
    }
    realize::realize(&content)
}

pub fn define_word(en: &English, word: &str) -> String {
    use praxis_domains::science::linguistics::pragmatics::realize::{self, ResponseContent};
    use praxis_domains::science::linguistics::pragmatics::response::ResponseFrame;

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
pub fn loaded_ontologies(lang: &English) -> Vec<VocabularyDescriptor> {
    let mut ontologies = describe_knowledge_base();
    ontologies.push(VocabularyDescriptor {
        name: "English (WordNet)",
        domain: "science.linguistics.english",
        being: Being::SocialObject,
        reason: "natural language is an evolved social convention",
        source: "Open English WordNet 2025; Princeton WordNet",
        concepts: lang.concept_count(),
        morphisms: lang.word_count(),
    });
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

    fn sample_english() -> English {
        // Use sample data for unit tests (fast, no WordNet needed)
        English::sample()
    }

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
    fn debug_function_words() {
        use praxis_domains::science::linguistics::language::Language;
        let en = sample_english();
        eprintln!(
            "word_count: {}, concept_count: {}",
            en.word_count(),
            en.concept_count()
        );
        for word in ["the", "is", "a", "what", "hello", "dog"] {
            let lookup = en.lexical_lookup(word);
            eprintln!("  lexical_lookup({word:?}) = {lookup:?}");
        }
    }

    #[test]
    fn debug_chat_responses() {
        use praxis_domains::science::linguistics::lambek::{reduce::chart_reduce, tokenize};
        use praxis_domains::science::linguistics::language::Language;

        let en = sample_english();
        let inputs = [
            "dog",
            "hello",
            "is a dog a mammal",
            "is a dog a mammal?",
            "the dog runs",
            "what is a dog",
        ];
        for input in &inputs {
            eprintln!("=== INPUT: {input:?} ===");

            // Metacognition: trace the pipeline
            let (tokens, alts) = tokenize::tokenize_with_alternatives(input, &en);
            eprintln!(
                "  tokens: {:?}",
                tokens
                    .iter()
                    .map(|t| format!("{}:{:?}", t.word, t.lambek_type))
                    .collect::<Vec<_>>()
            );

            let words: Vec<String> = tokens.iter().map(|t| t.word.clone()).collect();
            let type_sets: Vec<Vec<_>> = tokens
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let mut types = vec![t.lambek_type.clone()];
                    if let Some(a) = alts.get(i) {
                        types.extend(a.iter().cloned());
                    }
                    types
                })
                .collect();
            let reduction = chart_reduce(&words, &type_sets);
            eprintln!(
                "  parsed: {}, final_type: {:?}",
                reduction.success, reduction.final_type
            );

            // Known/unknown via Language trait
            for t in &tokens {
                let known = en.lexical_lookup(&t.word).is_some();
                eprintln!("  word {}: known={}", t.word, known);
            }

            let montague_tokens = if reduction.success && reduction.remaining.len() == tokens.len()
            {
                &reduction.remaining
            } else {
                &tokens
            };
            let meaning = praxis_domains::science::linguistics::lambek::montague::interpret(
                montague_tokens,
                &en,
            );
            eprintln!("  meaning: {:?}", meaning);

            let (response, user_act, _) = process(&en, input);
            eprintln!("  user_act: {user_act:?}");
            eprintln!("  RESPONSE: {response:?}");
            eprintln!();

            assert!(
                !response.is_empty(),
                "response for {input:?} should not be empty"
            );
        }
    }

    #[test]
    fn self_describe_has_ontologies() {
        let en = sample_english();
        let json = self_describe(&en);
        assert!(json.contains("ontology_count"));
        assert!(json.contains("Self-Model"));
        assert!(json.contains("Knowledge Base"));
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
}

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
    /// Whether the response was generated from the ontology (true)
    /// or is a hardcoded/fallback string (false).
    pub from_ontology: bool,
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
            from_ontology: false,
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
                from_ontology: true,
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
                from_ontology: true,
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
                from_ontology: parsed,
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
            // Find content words (nouns in WordNet)
            let nouns: Vec<&str> = tokens
                .iter()
                .filter(|t| !en.lookup(&t.word).is_empty() && t.lambek_type.is_noun())
                .map(|t| t.word.as_str())
                .collect();
            if nouns.len() >= 2 {
                // Explore what we KNOW about these concepts through the taxonomy
                return explore_concepts(en, &nouns);
            }
            let mut content = ResponseContent::new(frame);
            for w in &known_words {
                content = content.with_entity(w);
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
                        return build_taxonomy_response(en, child, parent, cid, pid);
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
    child_id: praxis_domains::science::linguistics::english::ConceptId,
    parent_id: praxis_domains::science::linguistics::english::ConceptId,
) -> String {
    use praxis_domains::science::linguistics::pragmatics::realize;

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
    use praxis_domains::science::linguistics::pragmatics::realize;

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
            praxis_domains::science::linguistics::pragmatics::response::ResponseFrame::AcknowledgeGap,
        ))
    } else {
        lines.join("\n")
    }
}

/// Find the lowest common ancestor of two concepts in the taxonomy.
fn find_common_ancestor(
    en: &English,
    a: praxis_domains::science::linguistics::english::ConceptId,
    b: praxis_domains::science::linguistics::english::ConceptId,
) -> Option<praxis_domains::science::linguistics::english::ConceptId> {
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

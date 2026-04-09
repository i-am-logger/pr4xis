use praxis_domains::science::cognition::epistemics;
use praxis_domains::science::linguistics::english::English;
use praxis_domains::science::linguistics::lambek::{
    ReductionResult, TypedToken, montague, reduce_sequence, tokenize,
};
use praxis_domains::science::linguistics::pragmatics::speech_act::SpeechAct;

// Praxis Chat Engine — shared logic for CLI, WASM, and any frontend.
//
// Zero I/O. Takes a string, returns a string.
// All intelligence comes from the Language ontology.
// The chat engine is a functor: Input → Language → Response.

/// Process input through the full linguistics pipeline.
/// Returns (response_text, user_speech_act, system_speech_act).
pub fn process(lang: &English, input: &str) -> (String, SpeechAct, SpeechAct) {
    let tokens = tokenize::tokenize(input, lang);
    if tokens.is_empty() {
        return (
            "I received empty input.".into(),
            SpeechAct::Assertion,
            SpeechAct::Assertion,
        );
    }

    let reduction = reduce_sequence(&tokens);
    let meaning = montague::interpret(&tokens, lang);

    match &meaning {
        montague::Sem::Question {
            predicate,
            arguments,
        } => {
            let response = answer_question(lang, predicate, arguments);
            (response, SpeechAct::Question, SpeechAct::Assertion)
        }

        montague::Sem::Prop {
            predicate,
            arguments,
        } => {
            let response = answer_statement(lang, predicate, arguments);
            (response, SpeechAct::Assertion, SpeechAct::Assertion)
        }

        _ => {
            let response = attempt_partial_understanding(lang, &tokens, &reduction, &meaning);
            (response, SpeechAct::Assertion, SpeechAct::Assertion)
        }
    }
}

fn attempt_partial_understanding(
    en: &English,
    tokens: &[TypedToken],
    reduction: &ReductionResult,
    meaning: &montague::Sem,
) -> String {
    let known_words: Vec<&str> = tokens
        .iter()
        .filter(|t| !en.lookup(&t.word).is_empty())
        .map(|t| t.word.as_str())
        .collect();

    let unknown_words: Vec<&str> = tokens
        .iter()
        .filter(|t| en.lookup(&t.word).is_empty())
        .map(|t| t.word.as_str())
        .collect();

    let has_knowledge = !known_words.is_empty();
    let parsed = reduction.success;
    let query_result: Option<&str> = if parsed { Some("parsed") } else { None };
    let state = epistemics::classify_result(parsed, has_knowledge, query_result);

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
            if nouns.len() >= 2 {
                return format!(
                    "I couldn't parse the full sentence, but I found two concepts.\nDid you mean: is {} a {}?",
                    nouns[0], nouns[1]
                );
            }
            format!(
                "I know the words {:?} but couldn't understand the sentence structure.\nCould you rephrase as 'is X a Y' or 'what is X'?",
                known_words
            )
        }
        epistemics::EpistemicState::KnownUnknown => {
            format!(
                "I don't know the word(s): {:?}\nI know {} of the {} words you used.",
                unknown_words,
                known_words.len(),
                tokens.len()
            )
        }
        epistemics::EpistemicState::KnownKnown => {
            format!("I understood: {}", meaning.describe())
        }
        epistemics::EpistemicState::UnknownUnknown => {
            "I don't understand. Could you try a simpler question like 'is a dog a mammal'?".into()
        }
    }
}

pub fn answer_question(en: &English, predicate: &str, arguments: &[montague::Sem]) -> String {
    let entities: Vec<String> = arguments.iter().map(extract_entity_name).collect();

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
                        return format!(
                            "Yes. {} is a {}.\n  {} -- {}\n  {} -- {}",
                            child, parent, child, c_def, parent, p_def
                        );
                    }
                }
            }
            return format!("No, {} is not a {}.", child, parent);
        }

        if !parent_ids.is_empty() && !child_ids.is_empty() {
            for &cid in parent_ids {
                for &pid in child_ids {
                    if en.is_a(cid, pid) {
                        return format!("Yes. {} is a {}.", parent, child);
                    }
                }
            }
        }
    }

    if entities.len() == 1 {
        return define_word(en, &entities[0]);
    }

    format!(
        "I understood the question but couldn't find an answer for: {}({})",
        predicate,
        entities.join(", ")
    )
}

pub fn answer_statement(en: &English, _predicate: &str, arguments: &[montague::Sem]) -> String {
    let entities: Vec<String> = arguments.iter().map(extract_entity_name).collect();

    if entities.len() == 1 {
        let ids = en.lookup(&entities[0]);
        if !ids.is_empty() {
            return define_word(en, &entities[0]);
        }
    }

    format!(
        "I understood that as a statement about: {}",
        entities.join(", ")
    )
}

pub fn define_word(en: &English, word: &str) -> String {
    let ids = en.lookup(word);
    if ids.is_empty() {
        return format!("I don't know the word '{}'.", word);
    }

    let mut lines = Vec::new();
    for (i, &id) in ids.iter().take(5).enumerate() {
        if let Some(concept) = en.concept(id) {
            for def in &concept.definitions {
                lines.push(format!("  {}. {}", i + 1, def));
            }
        }
    }

    if lines.is_empty() {
        format!("I know '{}' but have no definition for it.", word)
    } else {
        format!("{}:\n{}", word, lines.join("\n"))
    }
}

pub fn extract_entity_name(sem: &montague::Sem) -> String {
    match sem {
        montague::Sem::Entity { word, .. } => word.clone(),
        montague::Sem::Pred { word } => word.clone(),
        montague::Sem::Func { word, .. } => word.clone(),
        montague::Sem::Prop { predicate, .. } | montague::Sem::Question { predicate, .. } => {
            predicate.clone()
        }
    }
}

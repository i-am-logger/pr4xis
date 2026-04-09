use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::Arc;

use praxis_domains::science::cognition::epistemics;
use praxis_domains::science::information::dialogue::engine::{self, DialogueAction};
use praxis_domains::science::linguistics::lambek::{
    ReductionResult, TypedToken, montague, reduce_sequence, tokenize,
};
use praxis_domains::science::linguistics::language::{EnglishLanguage, Language};
use praxis_domains::science::linguistics::pragmatics::speech_act::SpeechAct;
use praxis_domains::technology::software::markup::xml::lmf;

fn main() {
    let wordnet_path = std::env::var("WORDNET_XML")
        .unwrap_or_else(|_| "crates/domains/data/wordnet/english-wordnet-2025.xml".into());

    let language = match load_language(&wordnet_path) {
        Ok(lang) => Arc::new(lang),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    println!("praxis — axiomatic intelligence");
    println!(
        "  {} concepts, {} words",
        language.concept_count(),
        language.word_count()
    );
    println!("  type 'quit' to exit");
    println!();

    let mut engine = engine::dialogue_engine();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).unwrap() == 0 {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Farewell detection through the language's lexicon.
        // The LANGUAGE determines what counts as farewell, not the CLI.
        let clean = input.trim().to_lowercase();
        if let Some(entry) = language.lexical_lookup(&clean)
            && entry.is_farewell()
        {
            let _ = engine.next(DialogueAction::EndDialogue);
            break;
        }

        // DRT: resolve anaphoric expressions via language lexicon + Centering Theory
        let resolved_input = resolve_pronouns(input, engine.situation(), language.as_ref());

        // Process through the linguistics pipeline
        let (response_text, user_act, sys_act) = process(&language, &resolved_input);

        // Extract referents using the language's lexicon (not hardcoded)
        let referents: Vec<String> = resolved_input
            .split_whitespace()
            .filter_map(|w| {
                let clean = w
                    .trim_matches(|c: char| c.is_ascii_punctuation())
                    .to_lowercase();
                language
                    .lexical_lookup(&clean)
                    .filter(|e| e.pos_tag().is_noun())
                    .map(|_| clean)
            })
            .collect();

        // Feed through the dialogue engine
        engine = match engine.next(DialogueAction::UserUtterance {
            text: input.to_string(),
            speech_act: user_act,
            referents,
        }) {
            Ok(e) => e,
            Err(praxis::engine::EngineError::Violated {
                engine: e,
                violations,
            }) => {
                for v in &violations {
                    eprintln!("  [dialogue violation] {}", v.reason());
                }
                e
            }
            Err(praxis::engine::EngineError::LogicalError { engine: e, reason }) => {
                eprintln!("  [dialogue error] {}", reason);
                e
            }
        };

        println!("{}", response_text);
        println!();

        engine = match engine.next(DialogueAction::SystemResponse {
            text: response_text,
            speech_act: sys_act,
        }) {
            Ok(e) => e,
            Err(praxis::engine::EngineError::Violated { engine: e, .. }) => e,
            Err(praxis::engine::EngineError::LogicalError { engine: e, .. }) => e,
        };
    }
}

/// The linguistics pipeline: text → tokens → types → semantics → response.
fn process(lang: &EnglishLanguage, input: &str) -> (String, SpeechAct, SpeechAct) {
    let tokens = tokenize::tokenize(input, lang);
    if tokens.is_empty() {
        return (
            "I received empty input.".into(),
            SpeechAct::Assertion,
            SpeechAct::Assertion,
        );
    }

    let reduction = reduce_sequence(&tokens);
    let meaning = montague::interpret(&tokens, lang.english());

    match &meaning {
        montague::Sem::Question {
            predicate,
            arguments,
        } => {
            let response = answer_question(lang.english(), predicate, arguments);
            (response, SpeechAct::Question, SpeechAct::Assertion)
        }

        montague::Sem::Prop {
            predicate,
            arguments,
        } => {
            let response = answer_statement(lang.english(), predicate, arguments);
            (response, SpeechAct::Assertion, SpeechAct::Assertion)
        }

        _ => {
            let response =
                attempt_partial_understanding(lang.english(), &tokens, &reduction, &meaning);
            (response, SpeechAct::Assertion, SpeechAct::Assertion)
        }
    }
}

fn attempt_partial_understanding(
    en: &praxis_domains::science::linguistics::english::English,
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

fn answer_question(
    en: &praxis_domains::science::linguistics::english::English,
    predicate: &str,
    arguments: &[montague::Sem],
) -> String {
    let entities: Vec<String> = arguments.iter().map(extract_entity_name).collect();

    if entities.len() >= 2 {
        let child = &entities[0];
        let parent_or_pred = &entities[1];

        let child_ids = en.lookup(child);
        let parent_ids = en.lookup(parent_or_pred);

        if !child_ids.is_empty() && !parent_ids.is_empty() {
            for &cid in child_ids {
                for &pid in parent_ids {
                    if en.is_a(cid, pid) {
                        let c = en.concept(cid);
                        let p = en.concept(pid);
                        let c_def = c
                            .and_then(|c| c.definitions.first())
                            .map(|d| d.as_str())
                            .unwrap_or("");
                        let p_def = p
                            .and_then(|p| p.definitions.first())
                            .map(|d| d.as_str())
                            .unwrap_or("");
                        return format!(
                            "Yes. {} is a {}.\n  {} — {}\n  {} — {}",
                            child, parent_or_pred, child, c_def, parent_or_pred, p_def
                        );
                    }
                }
            }
            return format!("No, {} is not a {}.", child, parent_or_pred);
        }

        if !parent_ids.is_empty() && !child_ids.is_empty() {
            for &cid in parent_ids {
                for &pid in child_ids {
                    if en.is_a(cid, pid) {
                        return format!("Yes. {} is a {}.", parent_or_pred, child);
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

fn answer_statement(
    en: &praxis_domains::science::linguistics::english::English,
    _predicate: &str,
    arguments: &[montague::Sem],
) -> String {
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

fn define_word(en: &praxis_domains::science::linguistics::english::English, word: &str) -> String {
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

fn extract_entity_name(sem: &montague::Sem) -> String {
    match sem {
        montague::Sem::Entity { word, .. } => word.clone(),
        montague::Sem::Pred { word } => word.clone(),
        montague::Sem::Func { word, .. } => word.clone(),
        montague::Sem::Prop { predicate, .. } | montague::Sem::Question { predicate, .. } => {
            predicate.clone()
        }
    }
}

/// Resolve anaphoric expressions using language lexicon + discourse state.
/// The LANGUAGE determines which words are anaphoric (via PronounKind).
/// The DISCOURSE STATE provides the referent (via Centering Theory).
fn resolve_pronouns(input: &str, state: &engine::DialogueState, language: &dyn Language) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    let resolved: Vec<String> = words
        .iter()
        .map(|&word| {
            let clean = word
                .trim_matches(|c: char| c.is_ascii_punctuation())
                .to_lowercase();
            // Check the language's lexicon: is this word anaphoric?
            let is_anaphoric = language
                .lexical_lookup(&clean)
                .is_some_and(|e| e.is_anaphoric());
            if is_anaphoric && let Some(referent) = state.resolve_anaphor() {
                return referent.to_string();
            }
            word.to_string()
        })
        .collect();
    resolved.join(" ")
}

fn load_language(path: &str) -> Result<EnglishLanguage, String> {
    if !Path::new(path).exists() {
        return Err(format!(
            "WordNet XML not found at: {}\nSet WORDNET_XML or download from:\n  https://github.com/globalwordnet/english-wordnet/releases",
            path
        ));
    }

    eprint!("Loading English ontology... ");
    let xml = std::fs::read_to_string(path).map_err(|e| format!("Failed to read: {}", e))?;
    let wn =
        lmf::reader::read_wordnet(&xml).map_err(|e| format!("Failed to parse WordNet: {}", e))?;
    let language = EnglishLanguage::from_wordnet(&wn);
    eprintln!(
        "done ({} concepts, {} words)",
        language.concept_count(),
        language.word_count()
    );
    Ok(language)
}

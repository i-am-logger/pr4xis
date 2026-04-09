use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::Arc;

use praxis_chat as chat;
use praxis_domains::science::information::dialogue::engine::{self, DialogueAction};
use praxis_domains::science::linguistics::english::English;
use praxis_domains::science::linguistics::language::Language;
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

        // Farewell detection through the language's lexicon
        let clean = input.trim().to_lowercase();
        if let Some(entry) = language.lexical_lookup(&clean)
            && entry.is_farewell()
        {
            let _ = engine.next(DialogueAction::EndDialogue);
            break;
        }

        // Resolve anaphoric expressions via language lexicon + Centering Theory
        let resolved_input = resolve_pronouns(input, engine.situation(), language.as_ref());

        // Process through praxis-chat (shared logic — zero I/O)
        let (response_text, user_act, _sys_act) = chat::process(&language, &resolved_input);

        // Extract referents for discourse tracking
        let referents: Vec<String> = resolved_input
            .split_whitespace()
            .filter_map(|w| {
                let c = w
                    .trim_matches(|c: char| c.is_ascii_punctuation())
                    .to_lowercase();
                language
                    .lexical_lookup(&c)
                    .filter(|e| e.pos_tag().is_noun())
                    .map(|_| c)
            })
            .collect();

        // Feed through the dialogue engine
        engine = match engine.next(DialogueAction::UserUtterance {
            text: input.to_string(),
            speech_act: user_act,
            referents,
        }) {
            Ok(e) => e,
            Err(praxis::engine::EngineError::Violated { engine: e, .. }) => e,
            Err(praxis::engine::EngineError::LogicalError { engine: e, .. }) => e,
        };

        println!("{}", response_text);
        println!();

        engine = match engine.next(DialogueAction::SystemResponse {
            text: response_text,
            speech_act: SpeechAct::Assertion,
        }) {
            Ok(e) => e,
            Err(praxis::engine::EngineError::Violated { engine: e, .. }) => e,
            Err(praxis::engine::EngineError::LogicalError { engine: e, .. }) => e,
        };
    }
}

/// Resolve anaphoric expressions using language lexicon + discourse state.
fn resolve_pronouns(input: &str, state: &engine::DialogueState, language: &dyn Language) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    let resolved: Vec<String> = words
        .iter()
        .map(|&word| {
            let clean = word
                .trim_matches(|c: char| c.is_ascii_punctuation())
                .to_lowercase();
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

fn load_language(path: &str) -> Result<English, String> {
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
    let language = English::from_wordnet(&wn);
    eprintln!(
        "done ({} concepts, {} words)",
        language.concept_count(),
        language.word_count()
    );
    Ok(language)
}

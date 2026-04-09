use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::Arc;

use praxis_domains::science::linguistics::english::English;
use praxis_domains::science::linguistics::lambek::{montague, reduce_sequence, tokenize};
use praxis_domains::technology::software::markup::xml::lmf;

fn main() {
    let wordnet_path = std::env::var("WORDNET_XML")
        .unwrap_or_else(|_| "crates/domains/data/wordnet/english-wordnet-2025.xml".into());

    let english = match load_english(&wordnet_path) {
        Ok(en) => Arc::new(en),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    println!("praxis — axiomatic intelligence");
    println!(
        "  {} concepts, {} words",
        english.concept_count(),
        english.word_count()
    );
    println!("  type 'quit' to exit");
    println!();

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
        if input == "quit" || input == "exit" {
            break;
        }

        let response = process(&english, input);
        println!("{}", response);
        println!();
    }
}

/// The cybernetic loop: input → understand → reason → respond.
/// If understanding fails → feedback (explain what went wrong).
fn process(en: &English, input: &str) -> String {
    // Step 1: Text → Tokens (via lexicon ontology)
    let tokens = tokenize::tokenize(input);
    if tokens.is_empty() {
        return "I received empty input.".into();
    }

    // Step 2-3: Tokens → Types → Reduction (Lambek grammar)
    let reduction = reduce_sequence(&tokens);

    // Step 4: Reduction → Semantics (Montague functor)
    let meaning = montague::interpret(&tokens, en);

    // Step 5-6: Semantics → Query → Result
    match &meaning {
        montague::Sem::Question {
            predicate,
            arguments,
        } => answer_question(en, predicate, arguments),
        montague::Sem::Prop {
            predicate,
            arguments,
        } => answer_statement(predicate, arguments),
        _ => {
            if !reduction.success {
                let types: Vec<String> = tokens
                    .iter()
                    .map(|t| format!("{}:{}", t.word, t.lambek_type.notation()))
                    .collect();
                format!(
                    "I couldn't fully parse that. I see: {}\nCould you rephrase?",
                    types.join(" + ")
                )
            } else {
                format!(
                    "I understood: {}\nBut I don't know how to respond yet.",
                    meaning.describe()
                )
            }
        }
    }
}

fn answer_question(en: &English, predicate: &str, arguments: &[montague::Sem]) -> String {
    // Extract entity names from arguments
    let entities: Vec<String> = arguments.iter().map(extract_entity_name).collect();

    // Map predicate to ontology query
    if entities.len() >= 2 {
        // "is X a Y" → taxonomy query
        let child = &entities[entities.len() - 1]; // last arg is usually the subject after inversion
        let parent_or_pred = &entities[0];

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

        // Try reverse order
        let child_ids = en.lookup(parent_or_pred);
        let parent_ids = en.lookup(child);
        if !child_ids.is_empty() && !parent_ids.is_empty() {
            for &cid in child_ids {
                for &pid in parent_ids {
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
        "I understood the question ?{}({}) but couldn't find an answer.",
        predicate,
        entities.join(", ")
    )
}

fn answer_statement(predicate: &str, arguments: &[montague::Sem]) -> String {
    let entities: Vec<String> = arguments.iter().map(extract_entity_name).collect();
    format!("I understood: {}({})", predicate, entities.join(", "))
}

fn define_word(en: &English, word: &str) -> String {
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

fn load_english(path: &str) -> Result<English, String> {
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
    let english = English::from_wordnet(&wn);
    eprintln!("done ({} concepts)", english.concept_count());
    Ok(english)
}

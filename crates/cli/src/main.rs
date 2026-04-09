use std::io::{self, BufRead, Write};
use std::path::Path;

use praxis_domains::science::linguistics::english::English;
use praxis_domains::technology::software::markup::xml::lmf;

fn main() {
    let wordnet_path = std::env::var("WORDNET_XML")
        .unwrap_or_else(|_| "crates/domains/data/wordnet/english-wordnet-2025.xml".into());

    let english = load_english(&wordnet_path);

    println!("praxis — axiomatic intelligence");
    println!(
        "  {} concepts, {} words",
        english.concept_count(),
        english.word_count()
    );
    println!("  type a question or 'quit' to exit");
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

fn load_english(path: &str) -> English {
    if !Path::new(path).exists() {
        eprintln!("WordNet XML not found at: {}", path);
        eprintln!("Set WORDNET_XML environment variable or download from:");
        eprintln!("  https://github.com/globalwordnet/english-wordnet/releases");
        std::process::exit(1);
    }

    eprint!("Loading English ontology... ");
    let xml = std::fs::read_to_string(path).unwrap();
    let wn = lmf::reader::read_wordnet(&xml).unwrap();
    let english = English::from_wordnet(&wn);
    eprintln!("done ({} concepts)", english.concept_count());
    english
}

fn process(en: &English, input: &str) -> String {
    let lower = input.to_lowercase();
    let lower = lower.trim_end_matches('?').trim();

    // "is X a Y" / "is a X a Y"
    if let Some(rest) = lower.strip_prefix("is ") {
        let rest = rest.strip_prefix("a ").unwrap_or(rest);
        let rest = rest.strip_prefix("an ").unwrap_or(rest);
        if let Some((child, parent)) = rest.split_once(" a ") {
            let parent = parent.strip_prefix("n ").unwrap_or(parent);
            return query_is_a(en, child.trim(), parent.trim());
        }
    }

    // "what is X" / "define X"
    if let Some(word) = lower.strip_prefix("what is ") {
        let word = word.strip_prefix("a ").unwrap_or(word);
        let word = word.strip_prefix("an ").unwrap_or(word);
        return query_define(en, word.trim());
    }
    if let Some(word) = lower.strip_prefix("define ") {
        return query_define(en, word.trim());
    }

    // "synonyms of X"
    if let Some(word) = lower.strip_prefix("synonyms of ") {
        return query_synonyms(en, word.trim());
    }

    // "parts of X"
    if let Some(word) = lower.strip_prefix("parts of ") {
        let word = word.strip_prefix("a ").unwrap_or(word);
        return query_parts(en, word.trim());
    }

    // Just a word — look it up
    if !lower.contains(' ') {
        return query_define(en, lower);
    }

    "I understand questions like:\n  \
     is a dog a mammal?\n  \
     what is a dog?\n  \
     define ontology\n  \
     synonyms of big\n  \
     parts of a car"
        .into()
}

fn query_is_a(en: &English, child: &str, parent: &str) -> String {
    let child_ids = en.lookup(child);
    let parent_ids = en.lookup(parent);

    if child_ids.is_empty() {
        return format!("I don't know the word '{}'", child);
    }
    if parent_ids.is_empty() {
        return format!("I don't know the word '{}'", parent);
    }

    for &cid in child_ids {
        for &pid in parent_ids {
            if en.is_a(cid, pid) {
                let child_concept = en.concept(cid).unwrap();
                let parent_concept = en.concept(pid).unwrap();
                let child_def = child_concept
                    .definitions
                    .first()
                    .map(|d| d.as_str())
                    .unwrap_or("");
                let parent_def = parent_concept
                    .definitions
                    .first()
                    .map(|d| d.as_str())
                    .unwrap_or("");
                return format!(
                    "Yes. {} is a {}.\n  {} — {}\n  {} — {}",
                    child, parent, child, child_def, parent, parent_def
                );
            }
        }
    }

    format!("No, {} is not a {}.", child, parent)
}

fn query_define(en: &English, word: &str) -> String {
    let ids = en.lookup(word);
    if ids.is_empty() {
        return format!("I don't know the word '{}'", word);
    }

    let mut lines = Vec::new();
    for (i, &id) in ids.iter().enumerate() {
        let concept = en.concept(id).unwrap();
        let pos = concept.pos.to_tag();
        for def in &concept.definitions {
            lines.push(format!("  {}. ({}) {}", i + 1, pos, def));
        }
        if concept.lemmas.len() > 1 {
            let synonyms: Vec<&str> = concept
                .lemmas
                .iter()
                .filter(|l| l.as_str() != word)
                .map(|l| l.as_str())
                .collect();
            if !synonyms.is_empty() {
                lines.push(format!("     synonyms: {}", synonyms.join(", ")));
            }
        }
    }

    format!("{}:\n{}", word, lines.join("\n"))
}

fn query_synonyms(en: &English, word: &str) -> String {
    let ids = en.lookup(word);
    if ids.is_empty() {
        return format!("I don't know the word '{}'", word);
    }

    let mut all_synonyms = Vec::new();
    for &id in ids {
        let concept = en.concept(id).unwrap();
        for lemma in &concept.lemmas {
            if lemma != word && !all_synonyms.contains(lemma) {
                all_synonyms.push(lemma.clone());
            }
        }
    }

    if all_synonyms.is_empty() {
        format!("No synonyms found for '{}'", word)
    } else {
        format!("Synonyms of {}: {}", word, all_synonyms.join(", "))
    }
}

fn query_parts(en: &English, word: &str) -> String {
    let ids = en.lookup(word);
    if ids.is_empty() {
        return format!("I don't know the word '{}'", word);
    }

    let mut parts_list = Vec::new();
    for &id in ids {
        let parts = en.parts(id);
        for &part_id in parts {
            if let Some(concept) = en.concept(part_id) {
                let name = concept.lemmas.first().map(|l| l.as_str()).unwrap_or("?");
                if !parts_list.contains(&name.to_string()) {
                    parts_list.push(name.to_string());
                }
            }
        }
    }

    if parts_list.is_empty() {
        format!("No parts found for '{}'", word)
    } else {
        format!("Parts of {}: {}", word, parts_list.join(", "))
    }
}

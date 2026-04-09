use wasm_bindgen::prelude::*;

use praxis_domains::science::linguistics::english::English;
use praxis_domains::science::linguistics::lambek::{montague, tokenize};
use praxis_domains::science::linguistics::language::Language;
use praxis_domains::technology::software::markup::xml::lmf;

// Praxis WASM — the entire chatbot runs in the browser.
//
// No server. No API. The ontology IS in the binary.
// The browser IS the runtime.

static SAMPLE_WN: &str = include_str!("../../domains/data/wordnet/english-wordnet-2025.xml");

#[wasm_bindgen]
pub struct Praxis {
    english: English,
}

#[wasm_bindgen]
impl Praxis {
    /// Create a new Praxis instance. Loads the full English ontology.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let wn = lmf::reader::read_wordnet(SAMPLE_WN).expect("failed to parse WordNet");
        Self {
            english: English::from_wordnet(&wn),
        }
    }

    /// Process an input and return a response.
    pub fn chat(&self, input: &str) -> String {
        let tokens = tokenize::tokenize(input, &self.english);
        if tokens.is_empty() {
            return "I received empty input.".to_string();
        }

        let meaning = montague::interpret(&tokens, &self.english);

        match &meaning {
            montague::Sem::Question {
                predicate,
                arguments,
            } => self.answer_question(predicate, arguments),

            montague::Sem::Prop {
                predicate,
                arguments,
            } => {
                let entities: Vec<String> = arguments.iter().map(Self::entity_name).collect();
                if entities.len() == 1 {
                    return self.define_word(&entities[0]);
                }
                format!("I understood: {}({})", predicate, entities.join(", "))
            }

            _ => format!("I understood: {}", meaning.describe()),
        }
    }

    /// Get the number of concepts loaded.
    pub fn concept_count(&self) -> usize {
        self.english.concept_count()
    }

    /// Get the number of words loaded.
    pub fn word_count(&self) -> usize {
        self.english.word_count()
    }
}

impl Praxis {
    fn answer_question(&self, _predicate: &str, arguments: &[montague::Sem]) -> String {
        let entities: Vec<String> = arguments.iter().map(Self::entity_name).collect();

        if entities.len() >= 2 {
            let child = &entities[0];
            let parent = &entities[1];
            let child_ids = self.english.lookup(child);
            let parent_ids = self.english.lookup(parent);

            if !child_ids.is_empty() && !parent_ids.is_empty() {
                for &cid in child_ids {
                    for &pid in parent_ids {
                        if self.english.is_a(cid, pid) {
                            let c_def = self.english.concept(cid)
                                .and_then(|c| c.definitions.first())
                                .map(|d| d.as_str())
                                .unwrap_or("");
                            let p_def = self.english.concept(pid)
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
        }

        if entities.len() == 1 {
            return self.define_word(&entities[0]);
        }

        format!("I understood the question about: {}", entities.join(", "))
    }

    fn define_word(&self, word: &str) -> String {
        let ids = self.english.lookup(word);
        if ids.is_empty() {
            return format!("I don't know the word '{}'.", word);
        }
        let mut lines = Vec::new();
        for (i, &id) in ids.iter().take(5).enumerate() {
            if let Some(concept) = self.english.concept(id) {
                for def in &concept.definitions {
                    lines.push(format!("  {}. {}", i + 1, def));
                }
            }
        }
        if lines.is_empty() {
            format!("I know '{}' but have no definition.", word)
        } else {
            format!("{}:\n{}", word, lines.join("\n"))
        }
    }

    fn entity_name(sem: &montague::Sem) -> String {
        match sem {
            montague::Sem::Entity { word, .. } => word.clone(),
            montague::Sem::Pred { word } => word.clone(),
            montague::Sem::Func { word, .. } => word.clone(),
            montague::Sem::Prop { predicate, .. }
            | montague::Sem::Question { predicate, .. } => predicate.clone(),
        }
    }
}

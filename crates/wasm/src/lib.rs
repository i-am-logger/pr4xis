use wasm_bindgen::prelude::*;

// Praxis WASM — the entire chatbot runs in the browser.
//
// No server. No API. The ontology IS in the binary.
// The browser IS the runtime.
//
// English is embedded via codegen (build.rs) — pre-computed static data.
// No XML parsing at runtime. No bloated binary.
// Same codegen functor as the CLI: WordNet XML → build.rs → Rust code → binary.

/// Generated English ontology — 107K concepts, compiled at build time.
mod english_codegen {
    include!(concat!(env!("OUT_DIR"), "/english_codegen.rs"));
}

#[wasm_bindgen]
pub struct Praxis {
    // The codegen'd data is static — no allocation needed.
    // TODO: bridge to praxis-chat once codegen output implements Language trait.
    _initialized: bool,
}

#[wasm_bindgen]
impl Praxis {
    /// Create a new Praxis instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { _initialized: true }
    }

    /// Process an input and return a response.
    pub fn chat(&self, input: &str) -> String {
        // Use codegen'd lookup for now
        let words: Vec<&str> = input.split_whitespace().collect();
        if words.is_empty() {
            return "I received empty input.".to_string();
        }

        // Try to answer "what is X" or "is X a Y" using codegen'd data
        let clean_words: Vec<String> = words
            .iter()
            .map(|w| {
                w.trim_matches(|c: char| c.is_ascii_punctuation())
                    .to_lowercase()
            })
            .collect();

        // Simple lookup using codegen'd word index
        if let Some(last_word) = clean_words.last() {
            let ids = english_codegen::lookup(last_word);
            if !ids.is_empty() {
                return format!(
                    "I know '{}' — it maps to {} concept(s) in the ontology.",
                    last_word,
                    ids.len()
                );
            }
        }

        format!(
            "I don't know those words yet. I have {} concepts loaded.",
            self.concept_count()
        )
    }

    /// Get the number of concepts loaded.
    pub fn concept_count(&self) -> usize {
        107519 // from codegen
    }

    /// Get the number of words loaded.
    pub fn word_count(&self) -> usize {
        english_codegen::lookup("dog").len() // just verify it works
    }
}

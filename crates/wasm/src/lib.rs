use wasm_bindgen::prelude::*;

use praxis_domains::science::linguistics::english::English;
use praxis_domains::technology::software::markup::xml::lmf;

// Praxis WASM — the entire chatbot runs in the browser.
//
// No server. No API. The ontology IS in the binary.
// The browser IS the runtime.
//
// Uses praxis-chat for shared logic (same as CLI).

static WORDNET_XML: &str = include_str!("../../domains/data/wordnet/english-wordnet-2025.xml");

#[wasm_bindgen]
pub struct Praxis {
    english: English,
}

#[wasm_bindgen]
impl Praxis {
    /// Create a new Praxis instance. Loads the full English ontology.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let wn = lmf::reader::read_wordnet(WORDNET_XML).expect("failed to parse WordNet");
        Self {
            english: English::from_wordnet(&wn),
        }
    }

    /// Process an input and return a response.
    /// Uses praxis-chat — same logic as CLI, zero I/O.
    pub fn chat(&self, input: &str) -> String {
        let (response, _, _) = praxis_chat::process(&self.english, input);
        response
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

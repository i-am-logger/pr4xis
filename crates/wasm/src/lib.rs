use wasm_bindgen::prelude::*;

use praxis_domains::science::linguistics::english::English;
use praxis_domains::technology::software::markup::xml::lmf;

// Praxis WASM — the entire chatbot runs in the browser.
//
// No server. No API. The ontology IS in the binary.
// The browser IS the runtime.
//
// Currently: XML parsed at init (takes ~2s in browser).
// TODO: bridge codegen output to Language trait for instant init.

static WORDNET_XML: &str = include_str!("../../domains/data/wordnet/english-wordnet-2025.xml");

#[wasm_bindgen]
pub struct Praxis {
    english: English,
}

#[wasm_bindgen]
impl Praxis {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let wn = lmf::reader::read_wordnet(WORDNET_XML).expect("failed to parse WordNet");
        Self {
            english: English::from_wordnet(&wn),
        }
    }

    /// Process input through the full praxis-chat pipeline.
    pub fn chat(&self, input: &str) -> String {
        let (response, _, _) = praxis_chat::process(&self.english, input);
        response
    }

    pub fn concept_count(&self) -> usize {
        self.english.concept_count()
    }

    pub fn word_count(&self) -> usize {
        self.english.word_count()
    }
}

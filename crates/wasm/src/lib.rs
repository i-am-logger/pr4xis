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
    /// Returns JSON with response, timing, and token count.
    pub fn chat(&self, input: &str) -> String {
        let result = praxis_chat::process_with_metadata(&self.english, input);
        format!(
            r#"{{"response":"{}","duration_us":{},"token_count":{},"tokens_per_sec":{}}}"#,
            result.response.replace('"', "\\\"").replace('\n', "\\n"),
            result.duration_us,
            result.token_count,
            if result.duration_us > 0 {
                (result.token_count as u64 * 1_000_000) / result.duration_us
            } else {
                0
            },
        )
    }

    pub fn concept_count(&self) -> usize {
        self.english.concept_count()
    }

    pub fn word_count(&self) -> usize {
        self.english.word_count()
    }

    /// The eigenform — the system describes itself through the SelfModel ontology.
    pub fn self_describe(&self) -> String {
        praxis_chat::self_describe(&self.english)
    }
}

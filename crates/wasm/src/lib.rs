use wasm_bindgen::prelude::*;

use praxis_domains::science::linguistics::english::English;

// Praxis WASM — the entire chatbot runs in the browser.
//
// No server. No API. The ontology IS in the binary.
// The browser IS the runtime.
//
// English is embedded via codegen (build.rs) — pre-computed static data.
// No XML parsing at runtime. No include_str! of raw XML.
// Same codegen functor as the CLI: WordNet XML → build.rs → Rust code → binary.
//
// TODO: bridge codegen output to Language trait for zero-cost init.
// Currently: parse the codegen'd data into English at init (fast, no XML).

/// Generated English ontology — compiled at build time from WordNet XML.
#[allow(dead_code)]
mod english_codegen {
    include!(concat!(env!("OUT_DIR"), "/english_codegen.rs"));
}

#[wasm_bindgen]
pub struct Praxis {
    english: English,
}

#[wasm_bindgen]
impl Praxis {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // TODO: use English::from_codegen() once the bridge is built.
        // For now, use sample data to avoid include_str! of 86MB XML.
        // The codegen output is available via english_codegen:: module.
        Self {
            english: English::sample(),
        }
    }

    /// Process input through the full praxis-chat pipeline.
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

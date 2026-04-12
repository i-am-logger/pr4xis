use wasm_bindgen::prelude::*;

use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::language;

// Pr4xis WASM — the entire chatbot runs in the browser.
//
// No server. No API. The ontology IS in the binary.
// The browser IS the runtime.
//
// English is embedded via codegen (build.rs) — pre-computed static data.
// No XML parsing at runtime. No include_str! of raw XML.
// Same codegen functor as the CLI: WordNet XML → build.rs → Rust code → binary.
//
// The codegen is language-agnostic: Data → CodegenData → Language functor.

/// Generated ontology — compiled at build time from WordNet XML.
/// Language-agnostic static data consumed by the Language functor.
#[allow(dead_code)]
mod codegen_output {
    include!(concat!(env!("OUT_DIR"), "/english_codegen.rs"));
}

#[wasm_bindgen]
pub struct Pr4xis {
    english: English,
}

impl Default for Pr4xis {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl Pr4xis {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            english: language::from_codegen(&codegen_output::CODEGEN_DATA),
        }
    }

    /// Process input through the full pr4xis-chat pipeline.
    pub fn chat(&self, input: &str) -> String {
        let result = pr4xis_chat::process_with_metadata(&self.english, input);
        let tps = if result.duration_us > 0 {
            (result.token_count as u64 * 1_000_000) / result.duration_us
        } else {
            0
        };
        let response = json_escape(&result.response);
        let trace = json_escape(&result.trace.serialize_with_functors());
        let ontologies = result.trace.all_participating_ontologies();
        let ontology_count = ontologies.len();
        let ontology_list = json_escape(&ontologies.join(", "));
        format!(
            r#"{{"response":"{response}","duration_us":{},"token_count":{},"tokens_per_sec":{tps},"parsed":{},"from_ontology":{},"ontology_count":{ontology_count},"ontologies":"{ontology_list}","trace":"{trace}"}}"#,
            result.duration_us, result.token_count, result.parsed, result.from_ontology,
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
        pr4xis_chat::self_describe(&self.english)
    }
}

/// Escape a string for safe JSON embedding.
fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

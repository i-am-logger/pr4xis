use wasm_bindgen::prelude::*;

use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::language;

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

#[derive(serde::Serialize)]
struct ChatResponse<'a> {
    response: &'a str,
    duration_us: u64,
    token_count: usize,
    tokens_per_sec: u64,
    parsed: bool,
    from_ontology: bool,
    ontology_count: usize,
    ontologies: &'a str,
    trace: &'a str,
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

    pub fn chat(&self, input: &str) -> String {
        let result = pr4xis_chat::process_with_metadata(&self.english, input);
        let tps = if result.duration_us > 0 {
            (result.token_count as u64 * 1_000_000) / result.duration_us
        } else {
            0
        };
        let trace = result.trace.serialize_with_functors();
        let ontologies = result.trace.all_participating_ontologies().join(", ");
        let resp = ChatResponse {
            response: &result.response,
            duration_us: result.duration_us,
            token_count: result.token_count,
            tokens_per_sec: tps,
            parsed: result.parsed,
            from_ontology: result.from_ontology,
            ontology_count: result.trace.all_participating_ontologies().len(),
            ontologies: &ontologies,
            trace: &trace,
        };
        serde_json::to_string(&resp).unwrap_or_default()
    }

    pub fn concept_count(&self) -> usize {
        self.english.concept_count()
    }

    pub fn word_count(&self) -> usize {
        self.english.word_count()
    }

    pub fn self_describe(&self) -> String {
        pr4xis_chat::self_describe(&self.english)
    }
}

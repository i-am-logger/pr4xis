use wasm_bindgen::prelude::*;

use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::language;
use pr4xis_domains::formal::information::schema::transport::{Presentation, SchemaValue};

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

    pub fn chat(&self, input: &str) -> String {
        let result = pr4xis_chat::process_with_metadata(&self.english, input);
        let ontologies = result.trace.all_participating_ontologies();
        let trace = result.trace.serialize_with_functors();

        let mut p = Presentation::new();
        p.set("response", SchemaValue::Text(result.response));
        p.set("duration_us", SchemaValue::Unsigned(result.duration_us));
        p.set("parsed", SchemaValue::Boolean(result.parsed));
        p.set("from_ontology", SchemaValue::Boolean(result.from_ontology));
        p.set(
            "ontologies",
            SchemaValue::List(
                ontologies
                    .into_iter()
                    .map(|o| SchemaValue::Text(o.to_string()))
                    .collect(),
            ),
        );
        p.set("trace", SchemaValue::Text(trace));
        p.to_json()
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

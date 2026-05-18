use wasm_bindgen::prelude::*;

use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::language;
use pr4xis_domains::formal::information::schema::transport::{Presentation, SchemaValue};

#[allow(dead_code)]
mod codegen_output {
    include!(concat!(env!("OUT_DIR"), "/english_codegen.rs"));
}

/// Build-time USC corpus produced by `pr4xis::codegen::usc_corpus`.
/// M4.ε.4 will wire this into the `Pr4xis` struct alongside `english`;
/// for now the `include!` keeps the codegen output in the build graph
/// and the test-only smoke check confirms the runtime functor accepts
/// the static without runtime cost in release builds.
#[allow(dead_code)]
mod usc_codegen_output {
    include!(concat!(env!("OUT_DIR"), "/usc_codegen.rs"));
}

#[cfg(test)]
#[allow(dead_code)]
fn usc_codegen_smoke_check() -> pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode
{
    pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode::from_codegen(
        &usc_codegen_output::CODEGEN_DATA,
    )
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
        p.set("response", result.response.into());
        p.set("duration_us", result.duration_us.into());
        p.set("parsed", result.parsed.into());
        p.set("from_ontology", result.from_ontology.into());
        p.set(
            "ontologies",
            SchemaValue::List(ontologies.into_iter().map(|o| o.into()).collect()),
        );
        p.set("trace", trace.into());
        p.to_json()
    }

    pub fn concept_count(&self) -> usize {
        self.english.concept_count()
    }

    pub fn word_count(&self) -> usize {
        self.english.word_count()
    }

    pub fn self_describe(&self) -> String {
        // WASM boundary: serialize the structural SelfModelInstance to JSON
        // for JS interop. Native callers should use pr4xis_chat::self_describe
        // directly to avoid the JSON detour.
        pr4xis_chat::self_describe(&self.english).to_json()
    }
}

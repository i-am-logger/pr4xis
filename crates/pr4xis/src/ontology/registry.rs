// Ontology registry — auto-populated via linkme distributed_slice (native targets).
//
// Each ontology definition (via `ontology!` proc macro or `define_ontology!`
// declarative macro) emits a `#[distributed_slice]` entry pointing at its
// `vocabulary()` function. At link time, all entries are gathered into the
// VOCABULARIES slice without any central registry file.
//
// On wasm32, linkme is unsupported — VOCABULARIES is empty. Wasm consumers
// of `describe_knowledge_base()` get an empty vec; if they need the full
// list they must call `pr4xis_domains::describe_knowledge_base_fallback()`.

use crate::ontology::Vocabulary;

/// All registered ontology vocabularies (native only).
///
/// Empty on wasm32 — linkme is unsupported there.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static VOCABULARIES: [fn() -> Vocabulary];

/// Describe the entire knowledge base — all registered ontologies.
///
/// On native targets, returns auto-populated VOCABULARIES.
/// On wasm32, returns an empty vec — use the domain-specific fallback.
#[cfg(not(target_arch = "wasm32"))]
pub fn describe_knowledge_base() -> Vec<Vocabulary> {
    VOCABULARIES.iter().map(|f| f()).collect()
}

/// Describe the entire knowledge base (wasm32 stub).
///
/// Returns an empty vec; consumers should use the wasm-specific fallback.
#[cfg(target_arch = "wasm32")]
pub fn describe_knowledge_base() -> Vec<Vocabulary> {
    Vec::new()
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn registry_is_accessible() {
        // Core crate alone has no registrations — domains crate provides them.
        let vocabs = describe_knowledge_base();
        let _ = vocabs.len();
    }
}

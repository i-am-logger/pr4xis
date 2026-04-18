// Lexicon registry — auto-populated via linkme distributed_slice (native targets).
//
// Four parallel slices, one per structural entity kind, so the full Lemon
// lexicon (ontologies + axioms + functors + adjunctions + natural
// transformations) is reachable without a central registry file. Each
// declaring macro (`ontology!`, `axioms:` clause, `functor!`,
// `adjunction!`, `natural_transformation!`) emits its own
// `#[distributed_slice]` entry; at link time every structural entity in
// the workspace is gathered here.
//
// On wasm32, linkme is unsupported — all slices are empty. Wasm consumers
// build a registry via domain-specific fallback instead.

use crate::category::{AdjunctionMeta, FunctorMeta, NaturalTransformationMeta};
use crate::logic::axiom::AxiomMeta;
use crate::ontology::Vocabulary;

/// All registered ontology vocabularies (native only).
///
/// Empty on wasm32 — linkme is unsupported there.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static VOCABULARIES: [fn() -> Vocabulary];

/// All registered axiom metadata (native only). Populated by the
/// `axioms:` clause inside `ontology!` and by manual registration for
/// structural-axiom families.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static AXIOMS: [fn() -> AxiomMeta];

/// All registered functor metadata (native only). Populated by
/// `pr4xis::functor!` declarations.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static FUNCTORS: [fn() -> FunctorMeta];

/// All registered adjunction metadata (native only). Populated by
/// `pr4xis::adjunction!` declarations.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static ADJUNCTIONS: [fn() -> AdjunctionMeta];

/// All registered natural-transformation metadata (native only).
/// Populated by `pr4xis::natural_transformation!` declarations.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static NATURAL_TRANSFORMATIONS: [fn() -> NaturalTransformationMeta];

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

/// All declared axioms with structured metadata.
#[cfg(not(target_arch = "wasm32"))]
pub fn describe_axioms() -> Vec<AxiomMeta> {
    AXIOMS.iter().map(|f| f()).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn describe_axioms() -> Vec<AxiomMeta> {
    Vec::new()
}

/// All declared functors with structured metadata.
#[cfg(not(target_arch = "wasm32"))]
pub fn describe_functors() -> Vec<FunctorMeta> {
    FUNCTORS.iter().map(|f| f()).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn describe_functors() -> Vec<FunctorMeta> {
    Vec::new()
}

/// All declared adjunctions with structured metadata.
#[cfg(not(target_arch = "wasm32"))]
pub fn describe_adjunctions() -> Vec<AdjunctionMeta> {
    ADJUNCTIONS.iter().map(|f| f()).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn describe_adjunctions() -> Vec<AdjunctionMeta> {
    Vec::new()
}

/// All declared natural transformations with structured metadata.
#[cfg(not(target_arch = "wasm32"))]
pub fn describe_natural_transformations() -> Vec<NaturalTransformationMeta> {
    NATURAL_TRANSFORMATIONS.iter().map(|f| f()).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn describe_natural_transformations() -> Vec<NaturalTransformationMeta> {
    Vec::new()
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn registry_is_accessible() {
        // Core crate alone has no registrations — domains crate provides them.
        let _ = describe_knowledge_base().len();
        let _ = describe_axioms().len();
        let _ = describe_functors().len();
        let _ = describe_adjunctions().len();
        let _ = describe_natural_transformations().len();
    }
}

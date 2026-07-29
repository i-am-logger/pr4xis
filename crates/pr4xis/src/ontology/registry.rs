#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
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

use crate::category::ConnectionGenerators;
use crate::logic::axiom::Axiom;
use crate::ontology::Vocabulary;
use crate::ontology::meta::Provenance;

/// The re-bind handler-table value type: a boxed runnable axiom.
pub type BoxedAxiom = Box<dyn Axiom>;

/// Box an axiom for the constructor registry — called by
/// `register_axiom!`'s constructor arm so the `Box` is allocated inside
/// this crate rather than the (possibly differently-configured) caller.
pub fn boxed_axiom<A: Axiom + 'static>(a: A) -> BoxedAxiom {
    Box::new(a)
}

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
pub static AXIOMS: [fn() -> Provenance];

/// Axiom *constructors* (native only) — the re-bind handler table.
/// Populated by `register_axiom!(Name, constructor)`. Unlike [`AXIOMS`]
/// (metadata only), each entry RECONSTRUCTS a runnable axiom, so a
/// deserialized `AxiomNode` can re-bind to its predicate by stable name
/// ([`axiom_by_name`]) — the load-time rebind the knowledge-graph wire
/// protocol depends on.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static AXIOM_CONSTRUCTORS: [fn() -> BoxedAxiom];

/// All registered functor metadata (native only). Populated by
/// `pr4xis::functor!` declarations.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static FUNCTORS: [fn() -> Provenance];

/// All registered adjunction metadata (native only). Populated by
/// `pr4xis::adjunction!` declarations.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static ADJUNCTIONS: [fn() -> Provenance];

/// All registered natural-transformation metadata (native only).
/// Populated by `pr4xis::natural_transformation!` declarations.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static NATURAL_TRANSFORMATIONS: [fn() -> Provenance];

/// Functor *constructors* (native only) — the connection-extraction table, the
/// 1-cell analogue of [`AXIOM_CONSTRUCTORS`]. Each entry runs
/// [`crate::category::extract_functor`] for one registered functor, recovering
/// its source/target ontology names and finite action-on-generators (the
/// finite-presentation theorem). A projection (`pr4xis-runtime::emit`) reads
/// these to serialize every functor touching a given ontology as a
/// content-addressed `Connection`. Populated by `functor!` /
/// `register_functor!`.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static FUNCTOR_CONSTRUCTORS: [fn() -> ConnectionGenerators];

/// Adjunction constructors (native only) — the structured-2-cell-pair analogue
/// of [`FUNCTOR_CONSTRUCTORS`]. Each runs
/// [`crate::category::extract_adjunction`], recovering both functors' object
/// maps plus the unit/counit families. Populated by `adjunction!` /
/// `register_adjunction!`.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static ADJUNCTION_CONSTRUCTORS: [fn() -> ConnectionGenerators];

/// Natural-transformation constructors (native only) — the 2-cell analogue.
/// Each runs [`crate::category::extract_natural_transformation`], recovering the
/// component family. Populated by `natural_transformation!` /
/// `register_natural_transformation!`.
#[cfg(not(target_arch = "wasm32"))]
#[linkme::distributed_slice]
pub static NATURAL_TRANSFORMATION_CONSTRUCTORS: [fn() -> ConnectionGenerators];

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
pub fn describe_axioms() -> Vec<Provenance> {
    AXIOMS.iter().map(|f| f()).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn describe_axioms() -> Vec<Provenance> {
    Vec::new()
}

/// Every registered axiom constructor, each reconstructed into a runnable
/// [`BoxedAxiom`] (native only — empty on wasm32, where linkme is
/// unsupported, which is the correct fail-closed "every binding unbound").
#[cfg(not(target_arch = "wasm32"))]
pub fn axiom_constructors() -> Vec<BoxedAxiom> {
    AXIOM_CONSTRUCTORS.iter().map(|f| f()).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn axiom_constructors() -> Vec<BoxedAxiom> {
    Vec::new()
}

/// Re-bind a persisted axiom binding by its stable name: reconstruct the
/// registered axiom whose [`Axiom::name`] matches `name`. `None` if no
/// constructor is registered under that name — fail-closed for the load
/// gate. Native only; always `None` on wasm32.
#[cfg(not(target_arch = "wasm32"))]
pub fn axiom_by_name(name: &str) -> Option<BoxedAxiom> {
    // Identity flows through the typed name: `OntologyName: PartialEq<str>`
    // decides the match, so the rebind gate compares typed axiom names — it
    // never unwraps to `&str` for a bare `String ==`.
    AXIOM_CONSTRUCTORS
        .iter()
        .map(|f| f())
        .find(|a| a.name() == *name)
}

#[cfg(target_arch = "wasm32")]
pub fn axiom_by_name(_name: &str) -> Option<BoxedAxiom> {
    None
}

/// Re-bind a persisted functor binding by its stable name: the registered functor
/// whose extracted [`ConnectionGenerators::name`] (its `FUNCTORS`-slice
/// [`Provenance`] name, set by `extract_functor` from `F::meta().name`) matches
/// `name`. `None` on miss — fail-closed for the load gate. Native only; always
/// `None` on wasm32 (linkme unsupported → every binding unbound, fail-closed).
///
/// The 1-cell analogue of [`axiom_by_name`]: the same iterate-constructors-and-find
/// shape and typed-name comparison (`OntologyName: PartialEq<str>` — identity flows
/// through the typed name, never a bare `String ==`). The key is the struct field
/// `.name` rather than a `dyn` method, because `ConnectionGenerators` already
/// self-carries its binding name (no `FUNCTORS`/`FUNCTOR_CONSTRUCTORS` index
/// correlation — the name travels with the constructor).
#[cfg(not(target_arch = "wasm32"))]
pub fn functor_by_name(name: &str) -> Option<ConnectionGenerators> {
    FUNCTOR_CONSTRUCTORS
        .iter()
        .map(|f| f())
        .find(|g| g.name == *name)
}

#[cfg(target_arch = "wasm32")]
pub fn functor_by_name(_name: &str) -> Option<ConnectionGenerators> {
    None
}

/// Re-bind a persisted adjunction binding by its stable name: the registered
/// adjunction whose [`ConnectionGenerators::name`] (its `ADJUNCTIONS`-slice name,
/// set by `extract_adjunction` from the adjunction's `meta().name`) matches `name`.
/// `None` on miss. Native only; `None` on wasm32. The structured-2-cell-pair
/// analogue of [`functor_by_name`].
#[cfg(not(target_arch = "wasm32"))]
pub fn adjunction_by_name(name: &str) -> Option<ConnectionGenerators> {
    ADJUNCTION_CONSTRUCTORS
        .iter()
        .map(|f| f())
        .find(|g| g.name == *name)
}

#[cfg(target_arch = "wasm32")]
pub fn adjunction_by_name(_name: &str) -> Option<ConnectionGenerators> {
    None
}

/// All declared functors with structured metadata.
#[cfg(not(target_arch = "wasm32"))]
pub fn describe_functors() -> Vec<Provenance> {
    FUNCTORS.iter().map(|f| f()).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn describe_functors() -> Vec<Provenance> {
    Vec::new()
}

/// All declared adjunctions with structured metadata.
#[cfg(not(target_arch = "wasm32"))]
pub fn describe_adjunctions() -> Vec<Provenance> {
    ADJUNCTIONS.iter().map(|f| f()).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn describe_adjunctions() -> Vec<Provenance> {
    Vec::new()
}

/// All declared natural transformations with structured metadata.
#[cfg(not(target_arch = "wasm32"))]
pub fn describe_natural_transformations() -> Vec<Provenance> {
    NATURAL_TRANSFORMATIONS.iter().map(|f| f()).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn describe_natural_transformations() -> Vec<Provenance> {
    Vec::new()
}

/// Every registered connection (functor + adjunction + natural transformation),
/// each reconstructed into its [`ConnectionGenerators`] — the typed
/// source/target ontology names plus the finite action-on-generators a
/// projection serializes. Native only; empty on wasm32 (linkme unsupported —
/// the fail-closed "no connections extractable" the wasm path expects).
///
/// This is the connection-layer mirror of [`axiom_constructors`]: where that
/// reconstructs runnable axioms by name, this reconstructs each morphism's
/// finite presentation so the emit projection can content-address it.
#[cfg(not(target_arch = "wasm32"))]
pub fn connection_constructors() -> Vec<ConnectionGenerators> {
    let mut all: Vec<ConnectionGenerators> = FUNCTOR_CONSTRUCTORS.iter().map(|f| f()).collect();
    all.extend(ADJUNCTION_CONSTRUCTORS.iter().map(|f| f()));
    all.extend(NATURAL_TRANSFORMATION_CONSTRUCTORS.iter().map(|f| f()));
    all
}

#[cfg(target_arch = "wasm32")]
pub fn connection_constructors() -> Vec<ConnectionGenerators> {
    Vec::new()
}

/// Every arrow in the workspace, flattened across the three cell-dimensions
/// of the 2-category Cat (Mac Lane XII.3): 1-cell functors, 2-cell natural
/// transformations, and structured-2-cell-pair adjunctions.
///
/// Consumers that don't need to discriminate by dimension get a single
/// list; consumers that do keep using `describe_functors()` /
/// `describe_adjunctions()` / `describe_natural_transformations()`
/// directly. All entries share the unified [`Provenance`] shape
/// (`Arrow::meta` — issue #155).
pub fn describe_all_arrows() -> Vec<Provenance> {
    let mut arrows = describe_functors();
    arrows.extend(describe_adjunctions());
    arrows.extend(describe_natural_transformations());
    arrows
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[crate::praxis_value(Explainable, Verifiable)]
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

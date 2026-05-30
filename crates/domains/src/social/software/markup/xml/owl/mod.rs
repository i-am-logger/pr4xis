pub mod lens;
pub mod ontology;
pub mod reader;
pub mod writer;

// Runtime corpus + praxis `Category` over a loaded OWL vocabulary —
// the hydration target a later rkyv `.prx` archive loads into, and the
// `from_codegen` functor analogous to `UsCode::from_codegen`. Consumes
// `CodegenData` (not codegen-gated); needs `std` for its process-
// lifetime `OnceLock` singleton, matching the USC `corpus` module.
#[cfg(feature = "std")]
pub mod vocabulary;

// OWL vocabulary → praxis `CodegenData` codegen. Gated on
// `any(test, feature = "codegen")` because it uses `pr4xis::codegen`,
// which `pr4xis` only exposes under its `codegen` feature — present in
// this crate's `[build-dependencies]` and `[dev-dependencies]`, not the
// normal (WASM-facing) dep.
#[cfg(any(test, feature = "codegen"))]
pub mod owl_vocabulary;

// The self-describing, load-validated `.prx.gz` distribution envelope for
// a loaded OWL vocabulary (the OWL leaf of the M4.ι archival path): rkyv
// archive of the `CodegenData` interchange + OMV/PROV-O-grounded metadata,
// gzip-wrapped, validated against the `praxis.lock` source-hash pin on
// load. Gated on `feature = "prx"`, which brings the rkyv (archival) and
// flate2 (gzip, RFC 1952) deps — both pure-Rust and wasm32-buildable, so
// the WASM runtime gets the load path. `fetch` implies `prx` (adding the
// network substrate on top), keeping the CLI's `fetch`-driven emit path
// unchanged. The emit-from-OWL helper inside additionally needs `codegen`
// (it calls `owl_to_builder`); the load + validate path needs only `prx`.
#[cfg(feature = "prx")]
pub mod prx;

// Registry-driven loaded OWL vocabularies (the SPAR family + PROV-O) and
// the corpus-wide audit. Walks every `OntologyVocabulary` source in the
// registry, hydrates each through `build_envelope` →
// `to_codegen_data_leaked` → `from_codegen`, and walks every record + edge
// of every vocabulary in the audit. Gated on both `fetch` (for `prx`'s
// `build_envelope` / `OwnedCodegenData`) and `any(test, feature =
// "codegen")` (because `build_envelope` itself, and the `owl_to_builder` it
// calls, are codegen-gated) — the WASM default-runtime `.prx.gz`/source
// load path is a separate milestone.
#[cfg(all(feature = "fetch", any(test, feature = "codegen")))]
pub mod loaded_vocabularies;

pub use ontology::*;

// Test-only `Arbitrary OwlOntology` strategy shared by the
// `writer.rs` and `lens.rs` proptest modules. No public surface.
#[cfg(test)]
pub(crate) mod test_arb;

#[cfg(test)]
mod tests;

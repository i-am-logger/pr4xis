pub mod ontology;
pub mod reader;

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

pub use ontology::*;

#[cfg(test)]
mod tests;

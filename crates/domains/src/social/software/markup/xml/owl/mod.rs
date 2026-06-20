pub mod lens;
pub mod ontology;
pub mod reader;
pub mod writer;

// The byte-exact graph-faithful RDF/XML structural writer — the OWL leaf of
// #186's graph-faithful tier (sibling of `lmf::writer` / `uslm::writer`). Builds
// the source element backbone from a structured RDF/XML serialization striping
// (`RdfXmlStructure`) and closes byte-exact via the generic residue machinery
// (`serialize_document_exact` + `SourceSyntax`/`RegeneratedComplement`). Always
// present (the structural fold is feature-free); the carried-in-`.prx` residue
// types are `prx`-gated for their rkyv derives, like the parser residue.
pub mod rdfxml_writer;

// The byte-exact graph-faithful OWL lens (`bytes ↔ OWL graph + structured RDF/XML
// complement`, FIDELITY = ByteExactGraphFaithful) and the registrations that flip
// EVERY bundled OWL vocab off the floor (the sibling of
// `lmf::lens::WordNetLmfLens`) — the FLAT SPAR family `cito@2.8.1`, `biro@1.1.1`,
// `c4o@1.2`, `doco@1.3` AND the STRIPED `prov_o@2013-04-30`, `olia@2026-04-09`
// (the L3 byte kernel: verbatim DOCTYPE, numeric/general references, interspersed
// comments). Native register_lens! is wasm32-skipped inside the macro.
pub mod graph_faithful_lens;

// Runtime corpus + praxis `Category` over a loaded OWL vocabulary —
// the hydration target a later rkyv `.prx` archive loads into, and the
// `from_codegen` functor analogous to `UsCode::from_codegen`. Consumes
// `CodegenData` (not codegen-gated); needs `std` for its process-
// lifetime `OnceLock` singleton, matching the USC `corpus` module.
#[cfg(feature = "std")]
pub mod vocabulary;

// OWL → praxis projection, the functor-as-data way (the OWL analog of the WordNet
// `english::bridge`): a raw structural projection + the committed
// `owl_functor.prx` (loaded fail-closed) carried AS DATA + the one `apply`
// interpreter. `std`-gated because it consumes `vocabulary` (above).
#[cfg(feature = "std")]
pub mod bridge;

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

// Registry-driven loaded OWL vocabularies (the SPAR family + PROV-O + OLiA)
// and the corpus-wide staleness-guard audit. The single generalized
// `load_owl_vocabulary` mechanism reads each `OntologyVocabulary`'s COMMITTED
// compact `.prx.gz` and admits it through the fail-closed
// `[compact_archive_signatures]` content gate (`load_compact_prx_gz_gated`),
// so the PURE LOAD PATH (`prx_path` / `load_owl_vocabulary` /
// `loaded_vocabularies`) needs only `prx` — it runs on the WASM/std-prx build
// the same way `olia::reference_model` does. The corpus-wide AUDIT inside
// additionally re-parses the FETCHED raw `.owl` (`read_owl` / `build_envelope`)
// as the staleness cross-check, so the audit ITEMS are gated on
// `any(test, feature = "codegen")` (where `build_envelope` lives) within the
// module.
#[cfg(feature = "prx")]
pub mod loaded_vocabularies;

pub use ontology::*;

// Test-only `Arbitrary OwlOntology` strategy shared by the
// `writer.rs` and `lens.rs` proptest modules. No public surface.
#[cfg(test)]
pub(crate) mod test_arb;

#[cfg(test)]
mod tests;

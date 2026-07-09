/// Meta-ontology — ontology about ontologies.
///
/// Formalizes gap detection, ontology engineering methodology,
/// and self-referential analysis of ontological structure.
///
/// Key references:
/// - Guarino, "Formal Ontology in Information Systems" (1998)
/// - Herre & Loebe, "A Meta-ontological Architecture" (FOIS 2005)
pub mod algebra;
pub mod artifact_identity;
pub mod canonical_codec;
pub mod categorical_structure;
pub mod citation_quality;
pub mod constitution_coverage;
pub mod dtd;
pub mod gap_analysis;
pub mod grounding;
pub mod identifier_format;
pub mod lens;
pub mod lens_composition;
pub mod omv;
pub mod ontology_archive;
pub mod ontology_diagnostics;
pub mod packed_csr;
pub mod praxis_knowledge_graph;
pub mod source_syntax;
pub mod source_taxonomy;
pub mod staging;
/// Self-describing ontology of the compact bit-packed `.prx` succinct codec.
/// Gated on `feature = "prx"`: the codec it describes
/// ([`crate::social::software::markup::xml::succinct`] + `owl::prx`) only exists
/// under `prx`.
#[cfg(feature = "prx")]
pub mod succinct_codec;
pub mod syntrometry;
pub mod versioning;
pub mod well_behaved_lens;
pub mod xsd;

// category_theory moved to core: `pr4xis::category::category_theory`.
// It grounds the `Arrow`, `Morphism`, `Functor`, `NaturalTransformation`,
// `Adjunction` trait / struct machinery in core.

/// Metadata about an ontology — for tracing and introspection.
///
/// Generated automatically by `define_ontology!`. The engine uses this
/// to identify which ontologies participated in a computation.
#[derive(Debug, Clone, Copy)]
pub struct OntologyMeta {
    /// The ontology name (e.g., "BiologyOntology").
    pub name: &'static str,
    /// The module path (e.g., "pr4xis_domains::natural::biomedical::biology").
    pub module_path: &'static str,
}

/// Report — data → visualization → surface → frozen artifact pipeline.
///
/// Validates ontology data against axioms, picks visual encodings via the
/// visualization ontology, renders to a target surface, and freezes the
/// result as a report (JSON, HTML, SVG, PDF). The report is a functor:
/// ValidationResults → OutputFormat. Different surfaces consume the same
/// data through different branches of the functor.
pub mod generator;
pub mod spec;
pub mod validator;

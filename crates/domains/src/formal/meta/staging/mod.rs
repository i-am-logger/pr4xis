/// Staging — multi-stage computation and partial evaluation.
///
/// Formalizes Futamura 1971's partial-evaluation framework as a meta-ontology.
/// Unifies pr4xis's existing `freeze: Dynamic → Static` patterns (codegen,
/// async ontology loading, PDF report generation, session archival) as
/// instances of the same functor.
pub mod ontology;

#[cfg(test)]
mod tests;

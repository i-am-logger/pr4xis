use crate::ontology::Ontology;
use praxis_category::Category;

/// Validate an ontology completely.
///
/// Checks category laws (identity, associativity, closure) + all axioms.
pub fn check_ontology<O: Ontology>() -> Result<(), Vec<String>>
where
    <O::Cat as Category>::Morphism: PartialEq,
{
    O::validate()
}

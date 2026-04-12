use crate::category::Category;
use crate::ontology::Ontology;

/// Validate an ontology completely.
///
/// Checks category laws (identity, associativity, closure) + all axioms.
pub fn check_ontology<O: Ontology>() -> Result<(), Vec<String>>
where
    <O::Cat as Category>::Morphism: PartialEq,
{
    O::validate()
}

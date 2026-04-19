use crate::category::Category;
use crate::logic::proof::Verdict;
use crate::ontology::Ontology;
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Validate an ontology completely — returns a typed [`Verdict`] aggregating
/// category-law checks and axiom verifications (#162).
///
/// Pattern-match the return. Core does not expose `is_valid() -> bool`
/// helpers (see `feedback_core_no_bool_api`).
pub fn check_ontology<O: Ontology>() -> Verdict
where
    O::Cat: 'static,
    <O::Cat as Category>::Morphism: PartialEq + 'static,
{
    O::validate()
}

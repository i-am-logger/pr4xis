use core::fmt::Debug;

use super::entity::Concept;

/// A relationship between entities — a morphism in a category.
///
/// Every relationship has a source entity, a target entity, and a
/// relation-kind tag. Per Gruber (1993) KAS 5 — "ontology =
/// formally-named relations" — and OBO Relation Ontology (Smith et
/// al. 2005), every morphism carries a canonical relation-type. The
/// `Kind` associated type is that tag.
///
/// The kind enum is category-local (declared by `ontology!` or
/// `define_category!`), but its variant names conventionally match
/// concept names in the Relations umbrella ontology
/// (`formal::relations`) so structural axioms and the Lemon registry
/// can reason about relation kinds uniformly.
pub trait Relationship: Sized + Clone + Debug + Eq {
    type Object: Concept;
    type Kind: Copy + Debug + Eq;

    /// The entity this relationship originates from.
    fn source(&self) -> Self::Object;

    /// The entity this relationship points to.
    fn target(&self) -> Self::Object;

    /// The relation-kind tag carried by this morphism.
    fn kind(&self) -> Self::Kind;
}

use std::fmt::Debug;

use crate::entity::Entity;

/// A relationship between entities — a morphism in the category.
///
/// Every relationship has a source and target entity, representing
/// a directed connection between objects in the ontology.
pub trait Relationship: Sized + Clone + Debug + Eq {
    type Object: Entity;

    /// The entity this relationship originates from.
    fn source(&self) -> Self::Object;

    /// The entity this relationship points to.
    fn target(&self) -> Self::Object;
}

use crate::entity::Entity;
use crate::relationship::Relationship;

/// A category: a collection of objects (entities) and morphisms (relationships)
/// where composition and identity laws hold.
///
/// # Laws
///
/// A valid category must satisfy:
/// - **Identity**: For every object A, there exists an identity morphism id_A
///   such that compose(id_A, f) == f and compose(f, id_A) == f for all f.
/// - **Associativity**: compose(compose(f, g), h) == compose(f, compose(g, h))
/// - **Closure**: If f: A → B and g: B → C, then compose(f, g): A → C exists.
///
/// Use [`crate::validate`] to verify these laws via property-based testing.
pub trait Category {
    type Object: Entity;
    type Morphism: Relationship<Object = Self::Object>;

    /// The identity morphism for an object (id_A: A → A).
    fn identity(obj: &Self::Object) -> Self::Morphism;

    /// Compose two morphisms: given f: A → B and g: B → C, produce g∘f: A → C.
    ///
    /// Returns `None` if the morphisms don't compose (f.target != g.source).
    fn compose(f: &Self::Morphism, g: &Self::Morphism) -> Option<Self::Morphism>;

    /// All morphisms in this category.
    fn morphisms() -> Vec<Self::Morphism>;

    /// All morphisms originating from the given object.
    fn morphisms_from(obj: &Self::Object) -> Vec<Self::Morphism> {
        Self::morphisms()
            .into_iter()
            .filter(|m| &m.source() == obj)
            .collect()
    }

    /// All morphisms targeting the given object.
    fn morphisms_to(obj: &Self::Object) -> Vec<Self::Morphism> {
        Self::morphisms()
            .into_iter()
            .filter(|m| &m.target() == obj)
            .collect()
    }
}

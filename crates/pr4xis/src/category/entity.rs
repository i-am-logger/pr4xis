use std::fmt::Debug;
use std::hash::Hash;

/// An entity is a thing that exists in an ontology — an object in the category.
///
/// Entities must be finite and enumerable. Every entity can list all possible
/// values of its type, enabling exhaustive validation of ontology properties.
///
/// Can be derived for enums with unit variants:
/// ```
/// use pr4xis::category::Entity;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
/// enum Color { Red, Green, Blue }
///
/// assert_eq!(Color::variants().len(), 3);
/// ```
pub trait Entity: Sized + Clone + Eq + Hash + Debug {
    /// All possible entities of this type.
    fn variants() -> Vec<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis_derive::Entity;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
    enum TestEntity {
        A,
        B,
        C,
    }

    #[test]
    fn derive_entity_produces_all_variants() {
        let v = TestEntity::variants();
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], TestEntity::A);
        assert_eq!(v[1], TestEntity::B);
        assert_eq!(v[2], TestEntity::C);
    }

    #[test]
    fn derive_entity_single_variant() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
        enum Single {
            Only,
        }
        assert_eq!(Single::variants(), vec![Single::Only]);
    }
}

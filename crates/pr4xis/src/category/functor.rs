use super::category::Category;

/// A functor is a structure-preserving map between two categories.
///
/// Given categories C and D, a functor F: C → D maps:
/// - Objects of C to objects of D
/// - Morphisms of C to morphisms of D
///
/// # Laws
///
/// A valid functor must satisfy:
/// - **Identity preservation**: F(id_A) == id_{F(A)}
/// - **Composition preservation**: F(g∘f) == F(g)∘F(f)
///
/// Use [`crate::validate`] to verify these laws.
pub trait Functor {
    type Source: Category;
    type Target: Category;

    /// Map an object from the source category to the target category.
    fn map_object(obj: &<Self::Source as Category>::Object) -> <Self::Target as Category>::Object;

    /// Map a morphism from the source category to the target category.
    fn map_morphism(
        m: &<Self::Source as Category>::Morphism,
    ) -> <Self::Target as Category>::Morphism;
}

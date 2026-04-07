use crate::category::Category;
use crate::functor::Functor;

/// A natural transformation is a morphism between two functors.
///
/// Given functors F, G: C → D, a natural transformation η: F ⇒ G
/// assigns to each object A in C a morphism η_A: F(A) → G(A) in D.
///
/// # Naturality condition
///
/// For every morphism f: A → B in C:
///   η_B ∘ F(f) == G(f) ∘ η_A
///
/// This ensures the transformation is "natural" — it commutes with
/// the structure of the categories.
pub trait NaturalTransformation {
    type SourceFunctor: Functor;
    type TargetFunctor: Functor<
            Source = <Self::SourceFunctor as Functor>::Source,
            Target = <Self::SourceFunctor as Functor>::Target,
        >;

    /// The component of this transformation at an object.
    ///
    /// Given object A in the source category, returns the morphism
    /// η_A: F(A) → G(A) in the target category.
    fn component(
        obj: &<<Self::SourceFunctor as Functor>::Source as Category>::Object,
    ) -> <<Self::SourceFunctor as Functor>::Target as Category>::Morphism;
}

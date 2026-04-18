use super::category::Category;
use super::functor::Functor;
use super::kinds::NatTransKind;
use crate::ontology::meta::{Citation, Label, ModulePath, OntologyName, RelationshipMeta};
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

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
///
/// # Metadata + kind
///
/// Per Mac Lane (1971) I.4, a natural transformation is a 2-cell in the
/// 2-category Cat. Metadata flows through the unified
/// [`RelationshipMeta`] shape; the [`NatTransKind`] tag records
/// whether every component is an iso, mono, epi, or the transformation
/// is the unit/counit of an adjunction.
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

    /// Classification of this natural transformation
    /// (iso / mono / epi / unit / counit / ...). Defaults to `Generic`.
    const KIND: NatTransKind = NatTransKind::Generic;

    /// Structured metadata — name, citation, module path.
    ///
    /// Default is an **honest placeholder** (type_name + empty citation)
    /// — "literature citation not yet declared". Override via
    /// `pr4xis::natural_transformation!` or
    /// [`relationship_meta!`](crate::relationship_meta!).
    fn meta() -> RelationshipMeta {
        let tn = core::any::type_name::<Self>().to_string();
        RelationshipMeta {
            name: OntologyName::new(tn.clone()),
            description: Label::new(tn),
            citation: Citation::EMPTY,
            module_path: ModulePath::new(module_path!().to_string()),
        }
    }
}

use super::category::Category;
use super::kinds::FunctorKind;
use crate::ontology::meta::{Citation, Label, ModulePath, OntologyName, RelationshipMeta};
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

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
///
/// # Metadata + kind
///
/// Per Mac Lane (1971) XII.3, a functor is a 1-cell in the 2-category Cat.
/// Its metadata flows through the unified [`RelationshipMeta`] shape that
/// every arrow in pr4xis shares, and it carries a [`FunctorKind`] tag
/// (faithful / full / forgetful / free / ...) per the Gruber (1993) /
/// OBO-RO (Smith 2005) principle that every relation is formally-named.
pub trait Functor {
    type Source: Category;
    type Target: Category;

    /// Map an object from the source category to the target category.
    fn map_object(obj: &<Self::Source as Category>::Object) -> <Self::Target as Category>::Object;

    /// Map a morphism from the source category to the target category.
    fn map_morphism(
        m: &<Self::Source as Category>::Morphism,
    ) -> <Self::Target as Category>::Morphism;

    /// Classification of this functor (faithful / full / forgetful / etc).
    /// Defaults to `Generic`; override in impls where the literature
    /// classification is known.
    const KIND: FunctorKind = FunctorKind::Generic;

    /// Structured metadata — name, citation, module path.
    ///
    /// The default is an **honest placeholder** using `core::any::type_name`
    /// and an empty citation — "literature citation not yet declared".
    /// Functors declared via `pr4xis::functor!` or with the
    /// [`relationship_meta!`](crate::relationship_meta!) helper override it.
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

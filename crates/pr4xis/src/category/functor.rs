use super::category::Category;
use crate::ontology::meta::{Citation, ModulePath, OntologyName};

/// Lemon-style lexical metadata for a functor — its identity, citation,
/// and module path. Matches `OntologyMeta` / `AxiomMeta` shape so the
/// lexicon treats ontologies, axioms, and functors uniformly (issue #148:
/// "every structural entity announces itself lexically").
///
/// Construct via `pr4xis::functor!` for the generated case, or
/// [`functor_meta!`](crate::functor_meta!) for hand-written impls.
#[derive(Debug, Clone)]
pub struct FunctorMeta {
    pub name: OntologyName,
    pub citation: Citation,
    pub module_path: ModulePath,
}

/// Helper: write the `meta()` associated function for a hand-written
/// `impl Functor` with a literature citation in one line.
///
/// # Example
///
/// ```ignore
/// impl Functor for MyFunctor {
///     type Source = ...;
///     type Target = ...;
///     fn map_object(...) -> ... { ... }
///     fn map_morphism(...) -> ... { ... }
///     pr4xis::functor_meta!("MyFunctor", "Mac Lane (1971) Ch. II §1");
/// }
/// ```
#[macro_export]
macro_rules! functor_meta {
    ($name:literal, $citation:literal) => {
        fn meta() -> $crate::category::FunctorMeta {
            $crate::category::FunctorMeta {
                name: $crate::ontology::meta::OntologyName::new_static($name),
                citation: $crate::ontology::meta::Citation::parse_static($citation),
                module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
            }
        }
    };
}

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
/// Every functor announces itself via `meta()` — its name, citation,
/// and module path, carried in the same Lemon-style wrappers as
/// ontologies and axioms.
pub trait Functor {
    type Source: Category;
    type Target: Category;

    /// Map an object from the source category to the target category.
    fn map_object(obj: &<Self::Source as Category>::Object) -> <Self::Target as Category>::Object;

    /// Map a morphism from the source category to the target category.
    fn map_morphism(
        m: &<Self::Source as Category>::Morphism,
    ) -> <Self::Target as Category>::Morphism;

    /// Structured metadata — name, citation, module path.
    ///
    /// The default is an **honest placeholder** using `std::any::type_name`
    /// and an empty citation — "literature citation not yet declared".
    /// Functors declared via `pr4xis::functor!` or with the
    /// [`functor_meta!`](crate::functor_meta!) helper override it.
    fn meta() -> FunctorMeta {
        FunctorMeta {
            name: OntologyName::new(std::any::type_name::<Self>().to_string()),
            citation: Citation::EMPTY,
            module_path: ModulePath::new(module_path!().to_string()),
        }
    }
}

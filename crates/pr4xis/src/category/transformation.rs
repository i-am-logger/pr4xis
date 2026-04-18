use super::category::Category;
use super::functor::Functor;
use crate::ontology::meta::{Citation, ModulePath, OntologyName};

/// Lemon-style lexical metadata for a natural transformation — name,
/// citation, module path. Matches `OntologyMeta` / `AxiomMeta` /
/// `FunctorMeta` / `AdjunctionMeta` shape (issue #148).
#[derive(Debug, Clone)]
pub struct NaturalTransformationMeta {
    pub name: OntologyName,
    pub citation: Citation,
    pub module_path: ModulePath,
}

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
/// Every natural transformation announces itself via `meta()` — same
/// Lemon-uniform shape as ontologies, axioms, functors, and adjunctions.
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

    /// Structured metadata — name, citation, module path.
    ///
    /// Default is an **honest placeholder** (type_name + empty citation)
    /// — "literature citation not yet declared". Override via
    /// `pr4xis::natural_transformation!` or
    /// [`natural_transformation_meta!`](crate::natural_transformation_meta!).
    fn meta() -> NaturalTransformationMeta {
        NaturalTransformationMeta {
            name: OntologyName::new(std::any::type_name::<Self>().to_string()),
            citation: Citation::EMPTY,
            module_path: ModulePath::new(module_path!().to_string()),
        }
    }
}

/// Helper: write the `meta()` associated function for a hand-written
/// `impl NaturalTransformation` with a literature citation in one line.
#[macro_export]
macro_rules! natural_transformation_meta {
    ($name:literal, $citation:literal) => {
        fn meta() -> $crate::category::NaturalTransformationMeta {
            $crate::category::NaturalTransformationMeta {
                name: $crate::ontology::meta::OntologyName::new_static($name),
                citation: $crate::ontology::meta::Citation::parse_static($citation),
                module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
            }
        }
    };
}

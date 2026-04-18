use super::category::Category;
use super::functor::Functor;
use crate::ontology::meta::{Citation, ModulePath, OntologyName};

// Adjunction — a pair of functors that are "optimally inverse."
//
// Given categories C and D, an adjunction F ⊣ G consists of:
// - Left adjoint F: C → D (the "free" functor)
// - Right adjoint G: D → C (the "forgetful" functor)
//
// The defining property: for all objects A in C and B in D,
//   Hom_D(F(A), B) ≅ Hom_C(A, G(B))
//
// This natural bijection means: a morphism F(A) → B in D
// corresponds uniquely to a morphism A → G(B) in C.
//
// Adjunctions are everywhere in praxis:
// - Parse ⊣ Generate — parsing text to semantics, generating text from semantics
// - Channel ⊣ Decode — noisy channel corruption, Bayesian correction
// - Abstract ⊣ Concretize — moving between abstraction levels
// - Forget ⊣ Free — forgetting structure, freely generating structure
//
// The unit η: Id_C → G∘F and counit ε: F∘G → Id_D capture the
// information loss: G∘F ≠ Id (you can't perfectly recover from
// the round trip), but η and ε are the "best approximation."
//
// References:
// - Mac Lane, Categories for the Working Mathematician (1971), Ch. IV
// - Awodey, Category Theory (2010), Ch. 9
// - Lambek & Scott, Introduction to Higher Order Categorical Logic (1986)

/// Lemon-style lexical metadata for an adjunction — name, citation,
/// module path. Matches `OntologyMeta` / `AxiomMeta` / `FunctorMeta`
/// shape (issue #148: "every structural entity announces itself lexically").
#[derive(Debug, Clone)]
pub struct AdjunctionMeta {
    pub name: OntologyName,
    pub citation: Citation,
    pub module_path: ModulePath,
}

/// An adjunction F ⊣ G between two categories.
///
/// F is the left adjoint (free construction / forward transform).
/// G is the right adjoint (forgetful / inverse transform).
///
/// The unit η: Id → G∘F captures what is preserved by the round trip.
/// The counit ε: F∘G → Id captures what is lost.
///
/// Every adjunction announces itself via `meta()` — same Lemon-uniform
/// shape as ontologies, axioms, and functors.
pub trait Adjunction {
    /// The left adjoint functor F: C → D.
    type Left: Functor;
    /// The right adjoint functor G: D → C.
    type Right: Functor<Source = <Self::Left as Functor>::Target, Target = <Self::Left as Functor>::Source>;

    /// The unit component at an object A in C.
    ///
    /// η_A: A → G(F(A))
    ///
    /// Embeds A into the round-trip G∘F. If η is an isomorphism,
    /// the adjunction is a reflection (no information loss on C's side).
    fn unit(
        obj: &<<Self::Left as Functor>::Source as Category>::Object,
    ) -> <<Self::Left as Functor>::Source as Category>::Morphism;

    /// The counit component at an object B in D.
    ///
    /// ε_B: F(G(B)) → B
    ///
    /// Projects the round-trip F∘G back to B. If ε is an isomorphism,
    /// the adjunction is a coreflection (no information loss on D's side).
    fn counit(
        obj: &<<Self::Left as Functor>::Target as Category>::Object,
    ) -> <<Self::Left as Functor>::Target as Category>::Morphism;

    /// Structured metadata — name, citation, module path.
    ///
    /// Default is an **honest placeholder** (type_name + empty citation)
    /// — "literature citation not yet declared". Override via
    /// `pr4xis::adjunction!` or the
    /// [`adjunction_meta!`](crate::adjunction_meta!) helper.
    fn meta() -> AdjunctionMeta {
        AdjunctionMeta {
            name: OntologyName::new(std::any::type_name::<Self>().to_string()),
            citation: Citation::EMPTY,
            module_path: ModulePath::new(module_path!().to_string()),
        }
    }
}

/// Helper: write the `meta()` associated function for a hand-written
/// `impl Adjunction` with a literature citation in one line.
#[macro_export]
macro_rules! adjunction_meta {
    ($name:literal, $citation:literal) => {
        fn meta() -> $crate::category::AdjunctionMeta {
            $crate::category::AdjunctionMeta {
                name: $crate::ontology::meta::OntologyName::new_static($name),
                citation: $crate::ontology::meta::Citation::parse_static($citation),
                module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
            }
        }
    };
}

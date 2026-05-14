//! Interpretation functor between the typed computational morphism
//! handle (`category::Morphism<C>`) and the descriptive Lemon record
//! (`ontology::meta::Morphism`) — issue #164.
//!
//! # Literature
//!
//! The relationship between these two representations is a canonical
//! pattern in applied category theory and ontology engineering,
//! appearing under multiple names:
//!
//! - **Spivak (2012)** *Functorial Data Migration*, Information and
//!   Computation 217, §3: a schema `C` is a small category; an instance
//!   `I : C → Set` is a functor into Set. Schema arrows and instance
//!   functions are related by **functor application** (evaluating `I`
//!   at an arrow).
//! - **Mac Lane (1971)** *Categories for the Working Mathematician*
//!   §II.2: the **hom-functor** `Hom_C(A, −) : C → Set` reflects
//!   morphism-as-element-of-a-set into a functor category.
//!   §III.2: the **Yoneda embedding** `Y : C → Set^{C^op}` is the
//!   canonical reflection of a category into a presheaf category.
//! - **Barr & Wells (1999)** *Category Theory for Computing Science*
//!   Ch. 10 (sketches): a sketch is a syntactic presentation; its
//!   models are functors into Set. The sketch-arrow / model-function
//!   distinction IS this interpretation split.
//! - **ONTOLEX-Lemon (W3C 2016)** §5: `ontolex:denotes` links a
//!   lexical entry to an ontology resource — the lexicon-to-ontology
//!   mapping is semantically a denotation function (same shape as an
//!   interpretation functor).
//! - **Awodey (2010)** *Category Theory* §1.7: free category on a graph
//!   `FreeCat(G) → C` via an interpretation functor `⟦−⟧`.
//!
//! # The pr4xis mapping
//!
//! - `category::Morphism<C>` (generic over `C: Category`) is the
//!   computational handle — Spivak's instance-side `I(f)`, Mac Lane's
//!   morphism-as-element, Barr-Wells's model function.
//! - `ontology::meta::Morphism` (non-generic, carries `ConceptName` /
//!   `MorphismKind` / `Lexical`) is the descriptive / reified side —
//!   Spivak's schema arrow, Mac Lane's presheaf component, Barr-Wells's
//!   sketch arrow, ONTOLEX-Lemon's lexical entry.
//!
//! This module provides the projection `reify` from the typed handle
//! to the descriptive record. The reverse direction (interpretation
//! from record to typed handle) is ontology-specific and lives with
//! each ontology's own `Concept` name-lookup.

use crate::category::arrow::Arrow;
use crate::category::entity::Concept;
use crate::category::{Category, Morphism as CategoryMorphism};
use crate::ontology::meta::{ConceptName, Morphism as MetaMorphism, MorphismKind};

/// Project a typed object to its descriptive [`ConceptName`].
///
/// Uses the `Concept::name()` method when it returns a non-empty value;
/// falls back to Debug output for anonymous / newtype concepts. This is
/// the object-side of the interpretation functor.
pub fn reify_concept<O: Concept>(obj: &O) -> ConceptName {
    let name = obj.name();
    if name.is_empty() {
        ConceptName::new(format!("{obj:?}"))
    } else {
        ConceptName::new(name.to_string())
    }
}

/// Project a typed morphism-kind (local to its category) to the
/// canonical [`MorphismKind`] by matching its Debug identifier against
/// the Relations-ontology canonical names.
///
/// Variant names that match canonical relation types (Subsumption,
/// Parthood, Causation, Opposition, Equivalence, Identity) map to
/// those variants; everything else becomes `Custom(variant_name)`.
pub fn reify_kind<K: core::fmt::Debug>(kind: &K) -> MorphismKind {
    let kind_id = format!("{kind:?}");
    match kind_id.as_str() {
        "Identity" => MorphismKind::Identity,
        "Subsumption" => MorphismKind::Subsumption,
        "Parthood" => MorphismKind::Parthood,
        "Causation" => MorphismKind::Causation,
        "Opposition" => MorphismKind::Opposition,
        "Equivalence" => MorphismKind::Equivalence,
        _ => MorphismKind::Custom(std::borrow::Cow::Owned(kind_id)),
    }
}

/// Project a typed [`CategoryMorphism`] handle to its descriptive
/// [`MetaMorphism`] record.
///
/// This is the **denotation / interpretation functor's morphism map**
/// per Spivak FDM §3, Mac Lane Yoneda, and ONTOLEX-Lemon
/// `ontolex:denotes` — the typed handle is what the descriptive
/// record denotes; `reify` recovers the description from the handle.
///
/// # Functoriality
///
/// Preserves identity: `reify(identity(a)) = Morphism::new(reify_concept(a), reify_concept(a), Identity)`.
/// Preserves composition of the ambient category (kind projection via
/// `reify_kind`, source/target via `reify_concept`).
pub fn reify<C>(m: &CategoryMorphism<C>) -> MetaMorphism
where
    C: Category,
    C::Object: Concept,
    C::Morphism: Arrow<Object = C::Object>,
    <C::Morphism as Arrow>::Kind: core::fmt::Debug,
{
    MetaMorphism::new(
        reify_concept(&m.source()),
        reify_concept(&m.target()),
        reify_kind(&m.inner().kind()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::{Arrow, Category};
    use crate::ontology::meta::MorphismKind;

    // Minimal test category — Subsumption-kinded.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Obj {
        A,
        B,
    }

    impl Concept for Obj {
        fn variants() -> Vec<Self> {
            vec![Obj::A, Obj::B]
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Kind {
        Identity,
        Subsumption,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct Morph {
        from: Obj,
        to: Obj,
        kind: Kind,
    }

    impl Arrow for Morph {
        type Object = Obj;
        type Kind = Kind;
        fn source(&self) -> Obj {
            self.from
        }
        fn target(&self) -> Obj {
            self.to
        }
        fn kind(&self) -> Kind {
            self.kind
        }
    }

    struct TestCat;
    impl Category for TestCat {
        type Object = Obj;
        type Morphism = Morph;
        fn identity(obj: &Obj) -> Morph {
            Morph {
                from: *obj,
                to: *obj,
                kind: Kind::Identity,
            }
        }
        fn compose(_f: &Morph, _g: &Morph) -> Option<Morph> {
            None // irrelevant for this test
        }
        fn morphisms() -> Vec<Morph> {
            vec![
                Morph {
                    from: Obj::A,
                    to: Obj::A,
                    kind: Kind::Identity,
                },
                Morph {
                    from: Obj::A,
                    to: Obj::B,
                    kind: Kind::Subsumption,
                },
            ]
        }
    }

    #[test]
    fn reify_projects_identity() {
        let id = TestCat::identity(&Obj::A);
        let handle = CategoryMorphism::<TestCat>::of(id);
        let reified = reify(&handle);
        assert_eq!(reified.kind, MorphismKind::Identity);
        // Source and target agree on identity.
        assert_eq!(reified.from, reified.to);
    }

    #[test]
    fn reify_projects_subsumption() {
        let sub = Morph {
            from: Obj::A,
            to: Obj::B,
            kind: Kind::Subsumption,
        };
        let handle = CategoryMorphism::<TestCat>::of(sub);
        let reified = reify(&handle);
        assert_eq!(reified.kind, MorphismKind::Subsumption);
    }

    #[test]
    fn reify_kind_maps_canonical_variants() {
        #[derive(Debug)]
        #[allow(dead_code)]
        enum K {
            Subsumption,
            Parthood,
            Unknown,
        }
        assert_eq!(reify_kind(&K::Subsumption), MorphismKind::Subsumption);
        assert_eq!(reify_kind(&K::Parthood), MorphismKind::Parthood);
        assert!(matches!(reify_kind(&K::Unknown), MorphismKind::Custom(_)));
    }
}

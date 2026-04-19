//! Category Theory — the meta-ontology grounding pr4xis's categorical substrate.
//!
//! # Why this ontology exists
//!
//! pr4xis's Rust core defines trait and struct names (`Category`, `Arrow`,
//! `Morphism`, `Functor`, `NaturalTransformation`, `Adjunction`, …) for the
//! substrate that every domain ontology sits on. Per
//! `feedback_api_ontological_from_day_one` — "ALL code uses typed
//! ontological concepts, never primitives. No exceptions." — those
//! substrate types must themselves be grounded in an ontology.
//!
//! This module IS that ontology. Every name used in `pr4xis::category::*`
//! is an instance of a concept declared here, cited to primary literature
//! (Mac Lane 1971; Awodey 2010; Bénabou 1967; Leinster 2004).
//!
//! # Synonymy — Morphism and Arrow
//!
//! Mac Lane uses `morphism` and `arrow` interchangeably (CWM Ch. I §1).
//! Awodey (2010) uses `arrow` as primary. In pr4xis:
//! - **One concept**: `Morphism`
//! - **Two labels**: "Morphism" (Mac Lane primary) and "Arrow" (Awodey
//!   primary / Mac Lane intuitive) — recorded in the definition string
//!   since Lemon single-primary-label-per-language limits one surface.
//!
//! The Rust trait is named `Arrow`; the Rust struct for a morphism
//! instance is named `Morphism`. Both surface forms point at the
//! [`CategoryTheoryConcept::Morphism`] concept.
//!
//! # Literature
//!
//! - Mac Lane (1971) *Categories for the Working Mathematician* — canonical source
//! - Awodey (2010) *Category Theory* — uses "arrow" as primary
//! - Bénabou (1967) *Introduction to Bicategories* — n-cell uniformity
//! - Leinster (2004) *Higher Operads, Higher Categories* — modern n-category treatment
//! - Gruber (1993) *A Translation Approach to Portable Ontology Specifications* — "formally-named relations"
//! - Smith et al. (2005) *Relations in Biomedical Ontologies* (OBO-RO) — kind tag requirement

use crate as pr4xis;
use crate::ontology::{Axiom, Ontology};

pr4xis::ontology! {
    name: "CategoryTheory",
    source: "Mac Lane (1971); Awodey (2010); Bénabou (1967); Leinster (2004)",

    concepts: [
        // =========================================================================
        // 0-cells and 1-cells within a category
        // =========================================================================

        // A 0-cell — the basic entity inside a category.
        Object,

        // A 1-cell — a directed structure-preserving map between two objects.
        // Primary concept. The Rust trait `pr4xis::category::Arrow` and the
        // Rust struct `pr4xis::category::Morphism<C>` are both instances of
        // this concept — "Morphism" (Mac Lane 1971 primary) and "Arrow"
        // (Awodey 2010 primary; Mac Lane synonym) are synonymous labels.
        Morphism,

        // The operation that combines composable morphisms.
        // Mac Lane (1971) Ch. I §1: given `f: A → B` and `g: B → C`,
        // produces `g ∘ f: A → C`.
        Composition,

        // An identity morphism `id_A: A → A` — one per object.
        // Mac Lane (1971) Ch. I §1.
        Identity,

        // The domain of a morphism — what it comes from.
        Source,

        // The codomain of a morphism — what it goes to.
        Target,

        // The relation-type tag carried by every morphism.
        // Per OBO-RO (Smith et al. 2005), every morphism has a named kind
        // (Subsumption / Parthood / Causation / Opposition / …).
        Kind,

        // =========================================================================
        // Specialised morphism classes
        // =========================================================================

        // A morphism whose source and target are the same object.
        // Mac Lane (1971) Ch. I §5.
        Endomorphism,

        // A morphism with a two-sided inverse.
        // Mac Lane (1971) Ch. I §5.
        Isomorphism,

        // An invertible endomorphism — an automorphism.
        // Mac Lane (1971) Ch. I §5.
        Automorphism,

        // A left-cancellative morphism: `m ∘ f = m ∘ g` implies `f = g`.
        // Mac Lane (1971) Ch. I §5.
        Monomorphism,

        // A right-cancellative morphism: `f ∘ e = g ∘ e` implies `f = g`.
        // Mac Lane (1971) Ch. I §5.
        Epimorphism,

        // =========================================================================
        // Whole structures and higher cells
        // =========================================================================

        // A category: objects + morphisms + composition + identity.
        // Mac Lane (1971) Ch. I §1.
        CategoryStructure,

        // A 1-cell in the 2-category Cat — a structure-preserving map
        // between categories.
        // Mac Lane (1971) Ch. II §1.
        Functor,

        // A 2-cell in Cat — a map between parallel functors preserving
        // their component-wise action.
        // Mac Lane (1971) Ch. II §4.
        NaturalTransformation,

        // A structured pair of functors F ⊣ G with unit and counit
        // satisfying the triangle identities.
        // Mac Lane (1971) Ch. IV §1.
        Adjunction,

        // The unit η: 1 ⇒ G∘F of an adjunction.
        // Mac Lane (1971) Ch. IV §1.
        Unit,

        // The counit ε: F∘G ⇒ 1 of an adjunction.
        // Mac Lane (1971) Ch. IV §1.
        Counit,

        // A bicategory — Bénabou's weakening where composition is
        // associative only up to coherent 2-isomorphism.
        // Bénabou (1967).
        Bicategory,

        // The 2-category Cat itself — 0-cells are categories, 1-cells
        // are functors, 2-cells are natural transformations.
        // Mac Lane (1971) XII.3.
        TwoCategory,

        // An n-category — Leinster's generalisation to arbitrary
        // dimensions.
        // Leinster (2004).
        HigherCategory,
    ],

    labels: {
        Object: ("en", "Object", "A 0-cell inside a category — the basic entity. Mac Lane (1971) CWM Ch. I §1."),
        Morphism: ("en", "Morphism / Arrow", "A 1-cell — directed structure-preserving map between objects. Mac Lane (1971) uses 'morphism' as primary and 'arrow' as synonym; Awodey (2010) uses 'arrow' as primary. Both labels refer to this concept."),
        Composition: ("en", "Composition", "The operation combining composable morphisms: given f: A → B and g: B → C, produces g ∘ f: A → C. Mac Lane (1971) Ch. I §1."),
        Identity: ("en", "Identity morphism", "For every object A, a morphism id_A: A → A that is left and right neutral for composition. Mac Lane (1971) Ch. I §1."),
        Source: ("en", "Source", "The domain of a morphism — what it comes from."),
        Target: ("en", "Target", "The codomain of a morphism — what it goes to."),
        Kind: ("en", "Relation kind", "The named relation-type tag carried by every morphism per OBO-RO (Smith et al. 2005) — Subsumption, Parthood, Causation, Opposition, etc."),

        Endomorphism: ("en", "Endomorphism", "A morphism whose source and target are the same object. Mac Lane (1971) Ch. I §5."),
        Isomorphism: ("en", "Isomorphism", "A morphism with a two-sided inverse. Mac Lane (1971) Ch. I §5."),
        Automorphism: ("en", "Automorphism", "An isomorphism that is also an endomorphism. Mac Lane (1971) Ch. I §5."),
        Monomorphism: ("en", "Monomorphism", "A left-cancellative morphism. Mac Lane (1971) Ch. I §5."),
        Epimorphism: ("en", "Epimorphism", "A right-cancellative morphism. Mac Lane (1971) Ch. I §5."),

        CategoryStructure: ("en", "Category", "The structure of objects + morphisms + composition + identity satisfying the category laws. Mac Lane (1971) Ch. I §1."),
        Functor: ("en", "Functor", "A 1-cell in the 2-category Cat — a structure-preserving map between categories. Mac Lane (1971) Ch. II §1."),
        NaturalTransformation: ("en", "Natural transformation", "A 2-cell in Cat — a map between parallel functors. Mac Lane (1971) Ch. II §4."),
        Adjunction: ("en", "Adjunction", "A structured pair F ⊣ G with unit and counit satisfying the triangle identities. Mac Lane (1971) Ch. IV §1."),
        Unit: ("en", "Unit", "The natural transformation η: 1 ⇒ G∘F of an adjunction. Mac Lane (1971) Ch. IV §1."),
        Counit: ("en", "Counit", "The natural transformation ε: F∘G ⇒ 1 of an adjunction. Mac Lane (1971) Ch. IV §1."),

        Bicategory: ("en", "Bicategory", "A weak 2-category where associativity and identity hold only up to coherent 2-isomorphism. Bénabou (1967)."),
        TwoCategory: ("en", "2-category", "Cat is a 2-category — 0-cells are categories, 1-cells are functors, 2-cells are natural transformations. Mac Lane (1971) XII.3."),
        HigherCategory: ("en", "Higher category", "An n-category generalising 2-categories to arbitrary dimensions. Leinster (2004)."),
    },

    is_a: [
        // Specialised morphisms are morphisms.
        (Endomorphism, Morphism),
        (Isomorphism, Morphism),
        (Automorphism, Endomorphism),
        (Automorphism, Isomorphism),
        (Monomorphism, Morphism),
        (Epimorphism, Morphism),

        // Higher-dimensional cells are morphisms at their dimension
        // (Mac Lane XII.3 — Cat is a 2-category; Bénabou 1967 — n-cells
        // are morphisms at dimension n).
        (Functor, Morphism),
        (NaturalTransformation, Morphism),

        // Adjunction components are natural transformations.
        (Unit, NaturalTransformation),
        (Counit, NaturalTransformation),

        // Higher category structures.
        (TwoCategory, HigherCategory),
        (Bicategory, HigherCategory),
    ],

    has_a: [
        // A morphism has source, target, kind (OBO-RO: every relation is named).
        (Morphism, Source),
        (Morphism, Target),
        (Morphism, Kind),

        // A category has the pieces Mac Lane names.
        (CategoryStructure, Object),
        (CategoryStructure, Morphism),
        (CategoryStructure, Composition),
        (CategoryStructure, Identity),

        // An adjunction has a unit and a counit.
        (Adjunction, Unit),
        (Adjunction, Counit),

        // A 2-category has the three cell dimensions.
        (TwoCategory, CategoryStructure),
        (TwoCategory, Functor),
        (TwoCategory, NaturalTransformation),
    ],
}

/// This meta-ontology declares structure only; no per-concept quality
/// is attached. Domain ontologies *about* categorical structure
/// (algebra, relation properties) have their own qualities. Here the
/// concepts are pure classifications with no attached attribute.
#[derive(Debug, Clone)]
pub struct NoQuality;

impl crate::ontology::Quality for NoQuality {
    type Individual = CategoryTheoryConcept;
    type Value = ();

    fn get(&self, _: &CategoryTheoryConcept) -> Option<()> {
        None
    }
}

impl Ontology for CategoryTheoryOntology {
    type Cat = CategoryTheoryCategory;
    type Qual = NoQuality;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        crate::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::laws::assert_category_laws;

    #[test]
    fn category_theory_ontology_category_laws() {
        assert_category_laws::<CategoryTheoryCategory>();
    }

    #[test]
    fn category_theory_ontology_validates() {
        CategoryTheoryOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}

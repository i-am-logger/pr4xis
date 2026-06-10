//! Optics — the substrate ontology grounding `category/optics.rs`.
//!
//! # Why this ontology lives in core
//!
//! `category/optics.rs` defines `Iso<A, B>`, `Lens<S, A>`, and
//! `Prism<S, A>` — bidirectional accessors for data structures. Per the
//! substrate-grounding principle, the concept vocabulary (Optic, Lens,
//! Prism, Iso, Traversal, Getter, Setter, Fold, …) belongs here in core
//! alongside the machinery.
//!
//! Named `optics_theory/` (not `optics/`) because `optics.rs` already
//! exists as a sibling file with the machinery. Same naming pattern as
//! `proof_theory/`, `propositional_logic/`, `category_theory/`.
//!
//! # Literature
//!
//! - **Foster, Greenwald, Moore, Pierce, Schmitt (2007)** "Combinators
//!   for Bidirectional Tree Transformations: A Linguistic Approach to
//!   the View-Update Problem" — original lens formulation.
//! - **van Laarhoven (2009)** "CPS Based Functional References" —
//!   the van Laarhoven representation: lenses as `∀f. (a → f b) → (s → f t)`.
//! - **Kmett, E. (2012+)** *lens* Haskell library — canonical practical
//!   implementation; introduced the `Lens`/`Prism`/`Iso`/`Traversal`
//!   hierarchy.
//! - **Pickering, Gibbons & Wu (2017)** "Profunctor Optics: Modular
//!   Data Accessors" — profunctor-based unification; Optic as the
//!   umbrella profunctor concept.
//! - **Boisseau & Gibbons (2018)** "What You Needa Know about Yoneda:
//!   Profunctor Optics and the Yoneda Lemma" — Yoneda-lemma derivation
//!   of profunctor optics.

use crate as pr4xis;
use crate::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Optics",
    source: "Foster-Greenwald-Moore-Pierce-Schmitt (2007); van Laarhoven (2009); Kmett lens library (2012+); Pickering-Gibbons-Wu (2017) Profunctor Optics; Boisseau-Gibbons (2018)",

    concepts: [
        // === Umbrella ===
        Optic,
        Focus,
        ViewFunction,
        UpdateFunction,

        // === The classical optics hierarchy (Kmett; Pickering-Gibbons-Wu) ===
        Iso,
        Lens,
        Prism,
        Traversal,
        Getter,
        Setter,
        Fold,
        Optional,
        Review,

        // === Profunctor representation (Pickering-Gibbons-Wu 2017) ===
        ProfunctorOptic,
        Profunctor,

        // === Use cases by data-structure shape ===
        Product,
        Sum,
        Container,

        // === Laws ===
        LensLaws,
        PrismLaws,
        IsoLaws,
    ],

    labels: {
        Optic: ("en", "Optic",
            "Umbrella concept for bidirectional data accessors — Lens, Prism, Iso, Traversal, Fold, etc. Pickering-Gibbons-Wu (2017) unify these as profunctor-indexed data."),
        Focus: ("en", "Focus",
            "The sub-structure an optic accesses within a larger whole. 'The part of the record this lens looks at'."),
        ViewFunction: ("en", "View function",
            "The 'read' half of an optic: `get: s -> a`. Foster et al. (2007)."),
        UpdateFunction: ("en", "Update function",
            "The 'write' half of an optic: `set: s -> a -> t` (for lenses) or `build: a -> t` (for prisms). Foster et al. (2007)."),

        Iso: ("en", "Isomorphism (Iso)",
            "A reversible optic — `Iso s a` witnesses `s ≅ a`. Simultaneously a lens, a prism, a traversal, a getter, etc. Mac Lane (1971) Ch. I §5 background; Kmett."),
        Lens: ("en", "Lens",
            "A well-behaved bidirectional accessor for a SINGLE PART of a product: `get: s -> a` + `set: s -> a -> s`. Foster et al. (2007); Kmett; characterised by get-put, put-get, put-put laws."),
        Prism: ("en", "Prism",
            "A bidirectional accessor for a SINGLE CASE of a sum: `preview: s -> Maybe a` + `review: a -> s`. Dual of lens in profunctor optics. Kmett."),
        Traversal: ("en", "Traversal",
            "An optic targeting ZERO-OR-MORE foci simultaneously — generalises Lens to multiple positions. McBride & Paterson (2008) applicative-based. Kmett."),
        Getter: ("en", "Getter",
            "A read-only optic: `get: s -> a` — no update component. Kmett."),
        Setter: ("en", "Setter",
            "A write-only optic: `over: (a -> b) -> s -> t` with no meaningful get. Kmett."),
        Fold: ("en", "Fold",
            "A read-many optic: folds over all foci. Kmett."),
        Optional: ("en", "Optional (Affine)",
            "An optic targeting AT MOST ONE focus — between Lens (exactly one) and Traversal (zero or more). Sometimes called 'affine traversal'."),
        Review: ("en", "Review",
            "The reverse direction of a prism — `review: a -> s`. Kmett."),

        ProfunctorOptic: ("en", "Profunctor optic",
            "An optic expressed as a natural transformation `p a b -> p s t` for certain profunctor constraints on p. Pickering-Gibbons-Wu (2017) — unifies the hierarchy via profunctor class constraints."),
        Profunctor: ("en", "Profunctor",
            "A functor `C^op × D -> Set` — natural generalisation of a relation. Profunctor optics live in the category of profunctors. Bénabou (1973)."),

        Product: ("en", "Product (data shape)",
            "Data-structure shape: a tuple, record, or struct — paired projections. Lenses target products. Mac Lane (1971) Ch. III §4."),
        Sum: ("en", "Sum (data shape)",
            "Data-structure shape: a tagged union, enum, or sum type. Prisms target sums. Mac Lane (1971) Ch. III §4."),
        Container: ("en", "Container",
            "Data-structure shape with multiple foci (list, tree, map). Traversals target containers. Abbott, Altenkirch & Ghani (2005) containers."),

        LensLaws: ("en", "Lens laws",
            "The three laws characterising well-behaved lenses: get-put (setting the view yields the original), put-get (getting after put returns what was put), put-put (double put equals single put with the second value). Foster et al. (2007)."),
        PrismLaws: ("en", "Prism laws",
            "preview . review ≡ Just, and preview s ≡ Just a implies review a ≡ s (mirror of lens laws for sums)."),
        IsoLaws: ("en", "Iso laws",
            "view . review ≡ id and review . view ≡ id — full invertibility."),
    },

    is_a: [
        // The classic optics hierarchy (Kmett)
        // Iso is the strictest: invertible in both directions
        (Iso, Lens),
        (Iso, Prism),

        // Lens and Prism are specialised traversals
        (Lens, Traversal),
        (Prism, Traversal),

        // Optional is between Lens and Traversal
        (Optional, Traversal),
        (Lens, Optional),
        (Prism, Optional),

        // Read-only specialisations
        (Getter, Fold),

        // All are Optics
        (Iso, Optic),
        (Lens, Optic),
        (Prism, Optic),
        (Traversal, Optic),
        (Getter, Optic),
        (Setter, Optic),
        (Fold, Optic),
        (Optional, Optic),
        (Review, Optic),

        // Profunctor view
        (ProfunctorOptic, Optic),
    ],

    has_a: [
        // Every optic has a focus
        (Optic, Focus),

        // A Lens has view + update
        (Lens, ViewFunction),
        (Lens, UpdateFunction),

        // A Prism has preview (view-like) + review (update-like)
        (Prism, ViewFunction),
        (Prism, Review),

        // An Iso inherits from both
        (Iso, ViewFunction),
        (Iso, UpdateFunction),

        // Getter has only view
        (Getter, ViewFunction),

        // Setter has only update
        (Setter, UpdateFunction),

        // Profunctor optic uses a Profunctor
        (ProfunctorOptic, Profunctor),

        // Data shapes — lenses target products, prisms target sums
        (Lens, Product),
        (Prism, Sum),
        (Traversal, Container),

        // Lawful optics have their laws
        (Lens, LensLaws),
        (Prism, PrismLaws),
        (Iso, IsoLaws),
    ],

    opposes: [
        // Lens vs Prism — product vs sum dual. Pickering-Gibbons-Wu
        // explicitly frame them as dual via profunctor classes.
        (Lens, Prism),
        (Prism, Lens),

        // Product vs Sum — the classical type-theoretic duality.
        (Product, Sum),
        (Sum, Product),

        // Getter vs Setter — read-only vs write-only duality.
        (Getter, Setter),
        (Setter, Getter),
    ],
}

/// Which literature/author introduces each optics concept.
#[derive(Debug, Clone)]
pub struct OpticsLineage;

impl Quality for OpticsLineage {
    type Individual = OpticsConcept;
    type Value = &'static str;

    fn get(&self, c: &OpticsConcept) -> Option<&'static str> {
        use OpticsConcept as O;
        Some(match c {
            O::Optic | O::Focus | O::ViewFunction | O::UpdateFunction => "foster-et-al-2007",
            O::Lens | O::LensLaws => "foster-et-al-2007",
            O::Iso | O::IsoLaws => "mac-lane-1971",
            O::Prism
            | O::PrismLaws
            | O::Traversal
            | O::Getter
            | O::Setter
            | O::Fold
            | O::Optional
            | O::Review => "kmett-lens",
            O::ProfunctorOptic => "pickering-gibbons-wu-2017",
            O::Profunctor => "benabou-1973",
            O::Product | O::Sum => "mac-lane-1971",
            O::Container => "abbott-altenkirch-ghani-2005",
        })
    }
}

impl Ontology for OpticsOntology {
    type Cat = OpticsCategory;
    type Qual = OpticsLineage;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        crate::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::laws::assert_category_laws;
    use crate::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<OpticsCategory>();
    }

    #[test]
    fn ontology_validates() {
        OpticsOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn iso_is_both_lens_and_prism() {
        // Kmett: Iso is the strictest optic — both a Lens and a Prism.
        let sub: Vec<_> = OpticsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == OpticsRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(sub.contains(&(OpticsConcept::Iso, OpticsConcept::Lens)));
        assert!(sub.contains(&(OpticsConcept::Iso, OpticsConcept::Prism)));
    }

    #[test]
    fn lens_targets_product_prism_targets_sum() {
        // Foster et al. (2007) / Kmett: lens ↔ products, prism ↔ sums.
        let parthood: Vec<_> = OpticsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == OpticsRelationKind::Parthood)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(parthood.contains(&(OpticsConcept::Lens, OpticsConcept::Product)));
        assert!(parthood.contains(&(OpticsConcept::Prism, OpticsConcept::Sum)));
    }

    #[test]
    fn lens_opposes_prism() {
        // Pickering-Gibbons-Wu (2017): lens and prism are dual optics —
        // product vs sum via profunctor class duality.
        let opp: Vec<_> = OpticsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == OpticsRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(OpticsConcept::Lens, OpticsConcept::Prism)));
        assert!(opp.contains(&(OpticsConcept::Prism, OpticsConcept::Lens)));
    }

    #[test]
    fn all_optics_are_optics() {
        // The hierarchy roots at Optic.
        let sub: Vec<_> = OpticsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == OpticsRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for o in [
            OpticsConcept::Iso,
            OpticsConcept::Lens,
            OpticsConcept::Prism,
            OpticsConcept::Traversal,
            OpticsConcept::Getter,
            OpticsConcept::Setter,
            OpticsConcept::Fold,
            OpticsConcept::Optional,
            OpticsConcept::Review,
        ] {
            assert!(
                sub.contains(&(o, OpticsConcept::Optic)),
                "{:?} should be-a Optic",
                o
            );
        }
    }

    #[test]
    fn every_concept_has_lineage() {
        let q = OpticsLineage;
        for c in OpticsConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing lineage", c);
        }
    }

    proptest! {
        #[test]
        fn prop_lineage_total(_seed in any::<u32>()) {
            let q = OpticsLineage;
            for c in OpticsConcept::variants() {
                prop_assert!(q.get(&c).is_some());
            }
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in OpticsCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in OpticsOntology::axioms() {
                match axiom.verify() {
                    Ok(_) => {}
                    Err(c) => prop_assert!(
                        false,
                        "structural axiom failed: {}",
                        c.meta().name.as_str()
                    ),
                }
            }
        }

        #[test]
        fn prop_opposition_symmetric(_seed in any::<u32>()) {
            let opp: Vec<_> = OpticsCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == OpticsRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in &opp {
                prop_assert!(
                    opp.contains(&(*b, *a)),
                    "opposition ({:?}, {:?}) missing symmetric partner",
                    a, b
                );
            }
        }
    }
}

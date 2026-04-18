//! MereologyTheory — formal parthood theory (issue #152).
//!
//! The *richer* vocabulary behind pr4xis's `has_a:` clause. Where
//! domain code wants to talk about proper parts, overlaps, fusions,
//! atoms, and gunk, this is the ontology that supplies the terms with
//! their proper literature.
//!
//! Four lineages of mereological thought:
//!
//! 1. **Classical Extensional Mereology (CEM)** — Leśniewski (1916,
//!    collected in *Collected Works*, 1992, Kluwer); Leonard & Goodman
//!    (1940) "The Calculus of Individuals", J. Symbolic Logic 5. The
//!    axiomatic foundation: parthood as a strict partial order with
//!    unique fusions.
//!
//! 2. **Philosophical systematisation** — Simons (1987) *Parts: A
//!    Study in Ontology*, Oxford. Source of `Supplementation` (weak and
//!    strong forms), `Atom`, `Gunk`. Simons's variants relax CEM's
//!    full uniqueness of sums.
//!
//! 3. **Applied mereotopology** — Casati & Varzi (1999) *Parts and
//!    Places: The Structures of Spatial Representation*, MIT Press.
//!    Source of the concept-set used across pr4xis ontologies and the
//!    mereotopological operations (overlap, underlap, disjoint, sum,
//!    product).
//!
//! 4. **Formal calculus** — Varzi (2007) "Spatial Reasoning and
//!    Ontology", handbook article; Varzi (2019) *Mereology* (SEP).
//!    Contemporary formal treatments keeping track of the axiomatic
//!    variants.
//!
//! Source: Leśniewski (1916); Leonard & Goodman (1940); Simons (1987);
//! Casati & Varzi (1999); Varzi (2007, 2019).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "MereologyTheory",
    source: "Leśniewski (1916) Foundations of Mereology; Leonard & Goodman (1940); Simons (1987) Parts; Casati & Varzi (1999) Parts and Places; Varzi (2019) SEP Mereology",
    being: AbstractObject,

    concepts: [
        // === Roles ===
        Part,
        Whole,

        // === Parthood variants (Casati & Varzi Ch. 3) ===
        ProperPart,

        // === Overlap / underlap / disjoint (mereotopology basics) ===
        Overlap,
        Underlap,
        Disjoint,

        // === Operations ===
        Fusion,
        Sum,
        Product,
        Composition,

        // === Extrema (Simons Ch. 1.3, Lewis 1991) ===
        Atom,
        Gunk,

        // === Axiom-named concept ===
        Supplementation,
    ],

    labels: {
        Part: ("en", "Part",
            "The smaller-or-equal term in a parthood relation x ≤ y. In Leśniewski's original system, parts include the whole itself; in the proper-part variant, x < y excludes x = y."),
        Whole: ("en", "Whole",
            "The larger-or-equal term in parthood. Casati & Varzi: a whole is the fusion of its parts."),

        ProperPart: ("en", "Proper part",
            "x is a proper part of y iff x ≤ y and x ≠ y. Casati & Varzi (1999) §3.1: the core asymmetric parthood that pr4xis domain ontologies most often mean by 'part of'."),

        Overlap: ("en", "Overlap",
            "x overlaps y iff ∃z: z ≤ x ∧ z ≤ y. Casati & Varzi §2.2: shared-part relation. Reflexive and symmetric but not transitive."),
        Underlap: ("en", "Underlap",
            "x and y underlap iff ∃z: x ≤ z ∧ y ≤ z. The dual of Overlap: both have a common upper bound. Holds trivially if a universe-object exists."),
        Disjoint: ("en", "Disjoint",
            "x and y are disjoint iff ¬Overlap(x, y). Casati & Varzi: the classical no-shared-parts condition."),

        Fusion: ("en", "Fusion (general sum)",
            "Leśniewski §9; Varzi §4: the unique object that is a sum of everything satisfying a predicate φ. Classical Mereology requires fusions exist for any non-empty predicate; Simons relaxes this."),
        Sum: ("en", "Sum (binary fusion)",
            "The binary special case of Fusion: x + y is the unique object whose parts are exactly {z : z ≤ x ∨ z ≤ y}."),
        Product: ("en", "Product (mereological intersection)",
            "x · y is the unique object whose parts are exactly the common parts of x and y. Exists iff Overlap(x, y). Casati & Varzi §4.2."),
        Composition: ("en", "Composition",
            "The operation that assembles parts into a whole. In CEM, Composition is total (any non-empty collection composes into a unique whole); in more restrictive systems (Simons, van Inwagen) it's partial."),

        Atom: ("en", "Atom",
            "An object with no proper parts. Simons (1987) §1.3: atomism is the claim that every object is composed of atoms; denied by gunk views."),
        Gunk: ("en", "Gunk",
            "Lewis (1991) *Parts of Classes*: an object every proper part of which itself has proper parts. No atoms — the divisions go all the way down."),

        Supplementation: ("en", "Supplementation",
            "Simons (1987) §3.2 / Casati & Varzi (1999) Axiom (P.4): if x is a proper part of y, then y has another proper part disjoint from x. Prevents 'lonely' proper parts, rules out collapse of distinct wholes."),
    },

    is_a: [
        // ProperPart is a stronger kind of Part
        (ProperPart, Part),
        // Sum is a binary Fusion; Product is an intersection-like Fusion variant
        (Sum, Fusion),
        // Atom and Gunk are extremal kinds of Whole
        (Atom, Whole),
        (Gunk, Whole),
    ],

    edges: [
        // Part composes into Whole
        (Part, Whole, ComposesInto),
        // Fusion produces a Whole from Parts
        (Part, Fusion, ParticipatesIn),
        (Fusion, Whole, Produces),
        // Sum and Product are operations on Parts
        (Part, Sum, CombinesInto),
        (Part, Product, IntersectsInto),
        // Composition is the general operation
        (Part, Composition, Undergoes),
        (Composition, Whole, Produces),
        // Overlap/Underlap/Disjoint are relations between Parts
        (Part, Overlap, RelatesVia),
        (Part, Underlap, RelatesVia),
        (Part, Disjoint, RelatesVia),
        // Supplementation is an axiom about ProperPart structure
        (Supplementation, ProperPart, ConstrainsStructureOf),
    ],

    axioms: {
        ProperPartIsStrictPart: {
            source: "Casati & Varzi (1999) §3.1 — x < y iff x ≤ y ∧ x ≠ y",
            description: "ProperPart is declared as a specialisation of Part (via is_a), encoding the CEM definition of proper parthood as non-reflexive parthood",
            holds: {
                use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
                MereologyTheoryTaxonomy::relations()
                    .iter()
                    .any(|(c, p)| {
                        *c == MereologyTheoryConcept::ProperPart
                            && *p == MereologyTheoryConcept::Part
                    })
            },
        },
        AtomAndGunkAreDual: {
            source: "Simons (1987) §1.3; Lewis (1991) Parts of Classes",
            description: "Atom and Gunk both specialise Whole but exclude each other: an atom has no proper parts, gunk's every proper part has proper parts",
            holds: {
                use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
                let rels = MereologyTheoryTaxonomy::relations();
                let atom_is_whole = rels.iter().any(|(c, p)| {
                    *c == MereologyTheoryConcept::Atom && *p == MereologyTheoryConcept::Whole
                });
                let gunk_is_whole = rels.iter().any(|(c, p)| {
                    *c == MereologyTheoryConcept::Gunk && *p == MereologyTheoryConcept::Whole
                });
                atom_is_whole && gunk_is_whole
            },
        },
        SupplementationConstrainsProperPart: {
            source: "Simons (1987) §3.2; Casati & Varzi (1999) P.4",
            description: "the edge (Supplementation, ProperPart, ConstrainsStructureOf) exists, encoding that Supplementation is an axiomatic constraint on proper-part structure",
            holds: {
                use pr4xis::category::Category;
                MereologyTheoryCategory::morphisms().iter().any(|r| {
                    r.from == MereologyTheoryConcept::Supplementation
                        && r.to == MereologyTheoryConcept::ProperPart
                        && r.kind == MereologyTheoryRelationKind::ConstrainsStructureOf
                })
            },
        },
        SumIsBinaryFusion: {
            source: "Leśniewski (1916); Leonard & Goodman (1940); Varzi (2019) SEP",
            description: "Sum is declared as a specialisation of Fusion (via is_a), capturing the CEM definition of binary sum as the restricted two-argument fusion",
            holds: {
                use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
                MereologyTheoryTaxonomy::relations()
                    .iter()
                    .any(|(c, p)| {
                        *c == MereologyTheoryConcept::Sum
                            && *p == MereologyTheoryConcept::Fusion
                    })
            },
        },
        FusionProducesWhole: {
            source: "Leśniewski (1916) §9; Casati & Varzi (1999) §4",
            description: "the edge (Fusion, Whole, Produces) exists, encoding that any fusion — by definition — yields a whole (the general sum of its inputs)",
            holds: {
                use pr4xis::category::Category;
                MereologyTheoryCategory::morphisms().iter().any(|r| {
                    r.from == MereologyTheoryConcept::Fusion
                        && r.to == MereologyTheoryConcept::Whole
                        && r.kind == MereologyTheoryRelationKind::Produces
                })
            },
        },
    },
}

// -----------------------------------------------------------------------------
// MereologyKind — Leśniewski / Simons / Casati-Varzi / Lewis lineage tags.
// -----------------------------------------------------------------------------

/// Quality: which literature-lineage introduces each concept?
#[derive(Debug, Clone)]
pub struct MereologyKind;

impl Quality for MereologyKind {
    type Individual = MereologyTheoryConcept;
    type Value = &'static str;

    fn get(&self, c: &MereologyTheoryConcept) -> Option<&'static str> {
        use MereologyTheoryConcept as M;
        Some(match c {
            M::Part | M::Whole | M::Fusion => "lesniewski",
            M::ProperPart
            | M::Overlap
            | M::Underlap
            | M::Disjoint
            | M::Sum
            | M::Product
            | M::Composition => "casati-varzi",
            M::Atom | M::Supplementation => "simons",
            M::Gunk => "lewis",
        })
    }
}

impl Ontology for MereologyTheoryOntology {
    type Cat = MereologyTheoryCategory;
    type Qual = MereologyKind;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        MereologyTheoryOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        MereologyTheoryOntology::generated_domain_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<MereologyTheoryCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        MereologyTheoryOntology::validate().unwrap();
    }

    #[test]
    fn proper_part_axiom_holds() {
        assert!(ProperPartIsStrictPart.holds());
    }

    #[test]
    fn atom_gunk_dual_holds() {
        assert!(AtomAndGunkAreDual.holds());
    }

    #[test]
    fn supplementation_constrains_holds() {
        assert!(SupplementationConstrainsProperPart.holds());
    }

    #[test]
    fn sum_is_binary_fusion_holds() {
        assert!(SumIsBinaryFusion.holds());
    }

    #[test]
    fn fusion_produces_whole_holds() {
        assert!(FusionProducesWhole.holds());
    }
}

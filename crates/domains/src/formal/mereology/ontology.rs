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

}

// -----------------------------------------------------------------------------
// Domain axioms — separate `impl Axiom` blocks (new `verify` / `axiom_meta!`
// shape per #160 / #167). Each axiom filters
// `MereologyTheoryCategory::morphisms()` by relation kind, per the
// kinded-morphism canonical pattern (per_def traits are gone).
// -----------------------------------------------------------------------------

fn subsumption_pair_exists(child: MereologyTheoryConcept, parent: MereologyTheoryConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    MereologyTheoryCategory::morphisms().iter().any(|m| {
        m.source() == child
            && m.target() == parent
            && m.kind() == MereologyTheoryRelationKind::Subsumption
    })
}

fn kinded_edge_exists(
    from: MereologyTheoryConcept,
    to: MereologyTheoryConcept,
    kind: MereologyTheoryRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    MereologyTheoryCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

/// CEM: ProperPart is the strict (non-reflexive) Part relation.
pub struct ProperPartIsStrictPart;

impl Axiom for ProperPartIsStrictPart {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if subsumption_pair_exists(
            MereologyTheoryConcept::ProperPart,
            MereologyTheoryConcept::Part,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ProperPartIsStrictPart",
        "ProperPart is-a Part (CEM: x < y iff x \u{2264} y \u{2227} x \u{2260} y)",
        "Casati & Varzi (1999) Parts and Places \u{00a7}3.1"
    );
}

/// Simons / Lewis dual: Atom and Gunk both specialise Whole.
pub struct AtomAndGunkAreDual;

impl Axiom for AtomAndGunkAreDual {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let atom_is_whole =
            subsumption_pair_exists(MereologyTheoryConcept::Atom, MereologyTheoryConcept::Whole);
        let gunk_is_whole =
            subsumption_pair_exists(MereologyTheoryConcept::Gunk, MereologyTheoryConcept::Whole);
        if atom_is_whole && gunk_is_whole {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AtomAndGunkAreDual",
        "Atom and Gunk both specialise Whole but are duals: atoms have no proper parts; gunk's every proper part has proper parts",
        "Simons (1987) Parts \u{00a7}1.3; Lewis (1991) Parts of Classes"
    );
}

/// Simons / Casati-Varzi P.4 — Supplementation constrains ProperPart structure.
pub struct SupplementationConstrainsProperPart;

impl Axiom for SupplementationConstrainsProperPart {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if kinded_edge_exists(
            MereologyTheoryConcept::Supplementation,
            MereologyTheoryConcept::ProperPart,
            MereologyTheoryRelationKind::ConstrainsStructureOf,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SupplementationConstrainsProperPart",
        "Supplementation is an axiomatic constraint on proper-part structure: if x < y then y has another proper part disjoint from x",
        "Simons (1987) Parts \u{00a7}3.2; Casati & Varzi (1999) Parts and Places P.4"
    );
}

/// CEM: Sum is the binary special case of Fusion.
pub struct SumIsBinaryFusion;

impl Axiom for SumIsBinaryFusion {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if subsumption_pair_exists(MereologyTheoryConcept::Sum, MereologyTheoryConcept::Fusion) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SumIsBinaryFusion",
        "Sum is-a Fusion (CEM: binary sum is the two-argument restricted fusion)",
        "Le\u{015b}niewski (1916); Leonard & Goodman (1940) J. Symbolic Logic 5; Varzi (2019) SEP Mereology"
    );
}

/// Leśniewski / Casati-Varzi: a fusion produces a whole.
pub struct FusionProducesWhole;

impl Axiom for FusionProducesWhole {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if kinded_edge_exists(
            MereologyTheoryConcept::Fusion,
            MereologyTheoryConcept::Whole,
            MereologyTheoryRelationKind::Produces,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FusionProducesWhole",
        "(Fusion, Whole, Produces): a fusion yields a whole (the general sum of its inputs)",
        "Le\u{015b}niewski (1916) Foundations of Mereology \u{00a7}9; Casati & Varzi (1999) Parts and Places \u{00a7}4"
    );
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

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ProperPartIsStrictPart));
        axioms.push(Box::new(AtomAndGunkAreDual));
        axioms.push(Box::new(SupplementationConstrainsProperPart));
        axioms.push(Box::new(SumIsBinaryFusion));
        axioms.push(Box::new(FusionProducesWhole));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<MereologyTheoryCategory>();
    }

    #[test]
    fn ontology_validates() {
        MereologyTheoryOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn proper_part_axiom_holds() {
        assert!(ProperPartIsStrictPart.verify().is_ok());
    }

    #[test]
    fn atom_gunk_dual_holds() {
        assert!(AtomAndGunkAreDual.verify().is_ok());
    }

    #[test]
    fn supplementation_constrains_holds() {
        assert!(SupplementationConstrainsProperPart.verify().is_ok());
    }

    #[test]
    fn sum_is_binary_fusion_holds() {
        assert!(SumIsBinaryFusion.verify().is_ok());
    }

    #[test]
    fn fusion_produces_whole_holds() {
        assert!(FusionProducesWhole.verify().is_ok());
    }
}

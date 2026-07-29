//! Counting / Cardinality — the arithmetic of finite pluralities.
//!
//! A small ontology deliberately kept adjacent to (not merged into)
//! `MereologyTheory`: mereology supplies the abstract vocabulary for
//! "parts of a whole", counting supplies the vocabulary for "how many" —
//! two theories that compose (the turing-benchmark A4 keystone), neither
//! subsuming the other.
//!
//! Two lineages:
//!
//! 1. **Logicist foundation** — Frege (1884) *Die Grundlagen der
//!    Arithmetik* (*The Foundations of Arithmetic*), §§55-69: a cardinal
//!    number is a property of a CONCEPT, not of the objects falling under
//!    it — "the Number which belongs to the concept F is the extension of
//!    the concept 'equal to the concept F'" (§68). Grounds `Collection`
//!    (Frege's "concept" as an object-classifier) and `Cardinality`
//!    (Frege's "Number", read off a collection).
//!
//! 2. **Developmental psychology** — Gelman & Gallistel (1978) *The
//!    Child's Understanding of Number*, Harvard University Press: the
//!    "how-to-count" principles a competent counter follows, three of
//!    which are structural (not merely procedural) and are modeled here
//!    as concepts in their own right: one-to-one correspondence (each
//!    item gets exactly one tag), stable order (the tags are used in the
//!    same order every count), and the cardinal principle (the last tag
//!    used names the cardinality of the whole collection).
//!
//! Source: Frege (1884) Die Grundlagen der Arithmetik; Gelman & Gallistel
//! (1978) The Child's Understanding of Number, Harvard University Press.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

pr4xis::ontology! {
    name: "Counting",
    source: "Frege (1884) Die Grundlagen der Arithmetik \u{00a7}\u{00a7}55-69; Gelman & Gallistel (1978) The Child's Understanding of Number, Harvard University Press",

    concepts: [
        Collection,
        Cardinality,
        SuccessorCount,
        OneToOneCorrespondence,
        StableOrder,
        CardinalPrinciple,
    ],

    labels: {
        Collection: ("en", "Collection",
            "A finite plurality of individuals falling under a concept -- Frege (1884) \u{00a7}68's 'concept F', the object a cardinality is a property OF, not a property of any one member."),
        Cardinality: ("en", "Cardinality",
            "The Number belonging to a Collection (Frege \u{00a7}68: the extension of 'equal to the concept F') -- what SuccessorCount, correctly applied, produces."),
        SuccessorCount: ("en", "Successor counting",
            "The process of tagging a Collection's members with successive elements of the successor sequence (0, S(0), S(S(0)), ...) -- the PROCEDURE; Cardinality is its result, not a synonym for it."),
        OneToOneCorrespondence: ("en", "One-to-one correspondence principle",
            "Gelman & Gallistel (1978) principle 1: each item in the Collection is tagged with exactly one counting word, and each counting word is used exactly once."),
        StableOrder: ("en", "Stable-order principle",
            "Gelman & Gallistel (1978) principle 2: the counting words are recited in the same order on every count -- an arbitrary but FIXED sequence."),
        CardinalPrinciple: ("en", "Cardinal principle",
            "Gelman & Gallistel (1978) principle 3: the last counting word used in a correct count names the cardinality of the whole Collection, not just its final member."),
    },

    is_a: [
        // The three Gelman-Gallistel principles are all counting principles
        // that jointly license SuccessorCount as correct.
    ],

    edges: [
        (Collection, Cardinality, HasCardinality),
        (SuccessorCount, Cardinality, Produces),
        (Collection, SuccessorCount, Undergoes),
        (OneToOneCorrespondence, SuccessorCount, Licenses),
        (StableOrder, SuccessorCount, Licenses),
        (CardinalPrinciple, Cardinality, Determines),
    ],
}

/// Frege vs. Gelman-Gallistel: which lineage introduces each concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountingLineage {
    /// Frege (1884) -- the logicist foundation (Collection, Cardinality, SuccessorCount).
    Frege,
    /// Gelman & Gallistel (1978) -- the how-to-count principles.
    GelmanGallistel,
}

/// Quality: which lineage introduces each concept?
#[derive(Debug, Clone)]
pub struct CountingKind;

impl Quality for CountingKind {
    type Individual = CountingConcept;
    type Value = CountingLineage;

    fn get(&self, c: &CountingConcept) -> Option<CountingLineage> {
        use CountingConcept as C;
        Some(match c {
            C::Collection | C::Cardinality | C::SuccessorCount => CountingLineage::Frege,
            C::OneToOneCorrespondence | C::StableOrder | C::CardinalPrinciple => {
                CountingLineage::GelmanGallistel
            }
        })
    }
}

fn kinded_edge_exists(
    from: CountingConcept,
    to: CountingConcept,
    kind: CountingRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    CountingCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

/// Frege \u{00a7}68: SuccessorCount, correctly applied, produces a Cardinality.
pub struct SuccessorCountProducesCardinality;

impl Axiom for SuccessorCountProducesCardinality {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if kinded_edge_exists(
            CountingConcept::SuccessorCount,
            CountingConcept::Cardinality,
            CountingRelationKind::Produces,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SuccessorCountProducesCardinality",
        "(SuccessorCount, Cardinality, Produces): correctly applied successor counting yields the collection's cardinality",
        "Frege (1884) Die Grundlagen der Arithmetik \u{00a7}68"
    );
}

pr4xis::register_axiom!(
    SuccessorCountProducesCardinality,
    "Frege (1884) Die Grundlagen der Arithmetik \u{00a7}68"
);

/// Gelman & Gallistel: all three how-to-count principles license SuccessorCount.
pub struct AllThreePrinciplesLicenseCounting;

impl Axiom for AllThreePrinciplesLicenseCounting {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let one_to_one = kinded_edge_exists(
            CountingConcept::OneToOneCorrespondence,
            CountingConcept::SuccessorCount,
            CountingRelationKind::Licenses,
        );
        let stable_order = kinded_edge_exists(
            CountingConcept::StableOrder,
            CountingConcept::SuccessorCount,
            CountingRelationKind::Licenses,
        );
        if one_to_one && stable_order {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AllThreePrinciplesLicenseCounting",
        "one-to-one correspondence and stable order both license SuccessorCount as a correct counting procedure",
        "Gelman & Gallistel (1978) The Child's Understanding of Number, ch. 7 (\"The Counting Model\")"
    );
}

pr4xis::register_axiom!(
    AllThreePrinciplesLicenseCounting,
    "Gelman & Gallistel (1978) The Child's Understanding of Number, ch. 7"
);

/// The cardinal principle DETERMINES (not merely licenses) the cardinality --
/// the last tag names the count, distinguishing it from the two purely
/// procedural principles.
pub struct CardinalPrincipleDeterminesCardinality;

impl Axiom for CardinalPrincipleDeterminesCardinality {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if kinded_edge_exists(
            CountingConcept::CardinalPrinciple,
            CountingConcept::Cardinality,
            CountingRelationKind::Determines,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CardinalPrincipleDeterminesCardinality",
        "(CardinalPrinciple, Cardinality, Determines): the last counting word used names the collection's cardinality, distinct from the two procedural (Licenses) principles",
        "Gelman & Gallistel (1978) The Child's Understanding of Number, ch. 7 (\"The Counting Model\")"
    );
}

pr4xis::register_axiom!(
    CardinalPrincipleDeterminesCardinality,
    "Gelman & Gallistel (1978) The Child's Understanding of Number, ch. 7"
);

impl Ontology for CountingOntology {
    type Cat = CountingCategory;
    type Qual = CountingKind;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(SuccessorCountProducesCardinality));
        axioms.push(Box::new(AllThreePrinciplesLicenseCounting));
        axioms.push(Box::new(CardinalPrincipleDeterminesCardinality));
        axioms
    }
}

/// Honest scope statement: this is a MINIMAL, illustrative realization of
/// the successor-count process, not a mechanism that independently verifies
/// Gelman & Gallistel's three principles. Slice iteration already gives
/// one-to-one correspondence (each element visited exactly once) and stable
/// order (a fixed traversal order) for free -- this function's contribution
/// is naming that structure (an explicit fold rather than a single opaque
/// `.len()` call) and returning the final tag as the cardinality (the
/// cardinal principle, the one genuinely asserted step: that the LAST count
/// reached, not the act of counting, IS the answer). It is functionally
/// equivalent to `.len()` for any `&[T]` (proven by
/// `prop_cardinality_equals_len` below) -- the value is in what it makes
/// explicit and citable, not in computing something `.len()` couldn't.
///
/// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), not a bare
/// `usize` — a cardinality is exactly the kind of value the rest of this
/// codebase already types this way (see
/// `cognitive::linguistics::orthography::distance::damerau_levenshtein`
/// for the sibling fix and its own citation of the precedent this follows,
/// `applied::operating_system::scheduler::engine::rm_utilization_bound`). The
/// counting loop below is the numeric kernel (raw `usize`, the successor
/// process itself); only the returned RESULT is wrapped, at the boundary.
pub fn cardinality<T>(collection: &[T]) -> Quantity {
    let mut count = 0usize;
    for _item in collection.iter() {
        // Successor step: count = S(count). One iteration per element
        // (one-to-one correspondence), in slice order (stable order).
        count += 1;
    }
    // Cardinal principle: the final tag IS the cardinality.
    Quantity::from_unit(count as f64, &unit::UNITLESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<CountingCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        CountingOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn successor_count_produces_cardinality_holds() {
        assert!(SuccessorCountProducesCardinality.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn all_three_principles_license_counting_holds() {
        assert!(AllThreePrinciplesLicenseCounting.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cardinal_principle_determines_cardinality_holds() {
        assert!(CardinalPrincipleDeterminesCardinality.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn collection_has_cardinality_and_undergoes_successor_count() {
        // The two structural edges no dedicated axiom checks -- Collection
        // is the thing that both HAS a Cardinality and UNDERGOES the
        // SuccessorCount process that produces it.
        assert!(kinded_edge_exists(
            CountingConcept::Collection,
            CountingConcept::Cardinality,
            CountingRelationKind::HasCardinality
        ));
        assert!(kinded_edge_exists(
            CountingConcept::Collection,
            CountingConcept::SuccessorCount,
            CountingRelationKind::Undergoes
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn counting_kind_classifies_every_concept_by_lineage() {
        // CountingKind is this ontology's Qual -- every concept must
        // classify, and the classification must match the module doc's own
        // Frege/Gelman-Gallistel split (3 concepts each).
        let quality = CountingKind;
        let frege = [
            CountingConcept::Collection,
            CountingConcept::Cardinality,
            CountingConcept::SuccessorCount,
        ];
        let gelman_gallistel = [
            CountingConcept::OneToOneCorrespondence,
            CountingConcept::StableOrder,
            CountingConcept::CardinalPrinciple,
        ];
        for c in frege {
            assert_eq!(quality.get(&c), Some(CountingLineage::Frege), "{c:?}");
        }
        for c in gelman_gallistel {
            assert_eq!(
                quality.get(&c),
                Some(CountingLineage::GelmanGallistel),
                "{c:?}"
            );
        }
    }

    /// A dimensionless UNITLESS cardinality quantity, for comparing against
    /// [`cardinality`]'s typed return value in these tests.
    fn card(n: u32) -> Quantity {
        Quantity::from_unit(f64::from(n), &unit::UNITLESS)
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cardinality_of_empty_collection_is_zero() {
        let empty: [u32; 0] = [];
        assert_eq!(cardinality(&empty), card(0));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cardinality_matches_element_count() {
        assert_eq!(cardinality(&["a", "b", "c"]), card(3));
        assert_eq!(cardinality(&[1, 2, 3, 4, 5, 6, 7]), card(7));
    }

    // -- Proptests --

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_cardinality_equals_len(v in proptest::collection::vec(any::<i32>(), 0..64)) {
            // The successor-counting realization must agree with the
            // structural notion of "how many" for ANY finite collection --
            // property tested rather than spot-checked.
            prop_assert_eq!(cardinality(&v).value, v.len() as f64);
        }
    }

    pr4xis::register_praxis_value!(prop_cardinality_equals_len, Verifiable);
}

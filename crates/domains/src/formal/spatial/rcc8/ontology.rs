//! RCC-8 — the Region Connection Calculus's 8 base spatial relations
//! (Randell, Cui & Cohn 1992), the topological substrate `natural::
//! geography`'s `borders`/`contains` reasoning grounds in.
//!
//! # Literature
//!
//! - **Randell, Cui & Cohn (1992)** *A Spatial Logic Based on Regions and
//!   Connection*, KR'92 — the 8 base relations (§3, Table 1): DC, EC, PO,
//!   TPP, NTPP, TPPi, NTPPi, EQ, a jointly-exhaustive-pairwise-disjoint
//!   (JEPD) partition of every possible relationship between two regions.
//! - **Cohn, Bennett, Gooday & Gotts (1997)** *Qualitative Spatial
//!   Representation and Reasoning with the Region Connection Calculus*,
//!   GeoInformatica 1(3) — the composition table and the RCC-5 coarsening
//!   (grounding the `is_a` structure below).

use pr4xis::ontology::{Axiom, Ontology, Quality};

use super::interval::{self, Interval};

pr4xis::ontology! {
    name: "Rcc8",
    source: "Randell, Cui & Cohn (1992) A Spatial Logic Based on Regions and Connection, KR'92; Cohn, Bennett, Gooday & Gotts (1997) GeoInformatica 1(3)",

    concepts: [DisConnected, ExternallyConnected, PartiallyOverlapping, TangentialProperPart, NonTangentialProperPart, TangentialProperPartInverse, NonTangentialProperPartInverse, Equal, ProperPart, ProperPartInverse],

    labels: {
        DisConnected: ("en", "DC",
            "The regions share no point at all. Randell, Cui & Cohn (1992) \u{00a7}3."),
        ExternallyConnected: ("en", "EC",
            "The regions share only boundary points -- they touch but their interiors are disjoint. Randell, Cui & Cohn (1992) \u{00a7}3."),
        PartiallyOverlapping: ("en", "PO",
            "The regions' interiors share points, but neither is a part of the other. Randell, Cui & Cohn (1992) \u{00a7}3."),
        TangentialProperPart: ("en", "TPP",
            "The first region is a proper part of the second AND their boundaries touch. Randell, Cui & Cohn (1992) \u{00a7}3."),
        NonTangentialProperPart: ("en", "NTPP",
            "The first region is a proper part of the second and lies strictly in its interior (boundaries do not touch). Randell, Cui & Cohn (1992) \u{00a7}3."),
        TangentialProperPartInverse: ("en", "TPPi",
            "The inverse of TPP -- the second region is a tangential proper part of the first. Randell, Cui & Cohn (1992) \u{00a7}3."),
        NonTangentialProperPartInverse: ("en", "NTPPi",
            "The inverse of NTPP. Randell, Cui & Cohn (1992) \u{00a7}3."),
        Equal: ("en", "EQ",
            "The regions are identical. Randell, Cui & Cohn (1992) \u{00a7}3."),
        ProperPart: ("en", "Proper part (RCC-5 P)",
            "The RCC-5 coarsening of TPP/NTPP that drops the tangential/non-tangential distinction. Cohn, Bennett, Gooday & Gotts (1997)."),
        ProperPartInverse: ("en", "Proper part inverse (RCC-5 Pi)",
            "The RCC-5 coarsening of TPPi/NTPPi. Cohn, Bennett, Gooday & Gotts (1997)."),
    },

    // The RCC-5 coarsening (Cohn et al. 1997): TPP and NTPP are SIBLING
    // refinements of the coarser "proper part" (P) relation, not one
    // subsuming the other -- both is_a ProperPart, never TPP is_a NTPP.
    // Likewise their inverses is_a ProperPartInverse.
    is_a: [
        (TangentialProperPart, ProperPart),
        (NonTangentialProperPart, ProperPart),
        (TangentialProperPartInverse, ProperPartInverse),
        (NonTangentialProperPartInverse, ProperPartInverse),
    ],
}

/// Quality: does this RCC-8 relation entail that the two regions are
/// CONNECTED (`C(a,b)`, Randell, Cui & Cohn 1992 §2 — the calculus's sole
/// primitive, from which every relation including DC's negation is
/// defined)? Every relation entails connection except DC itself.
#[derive(Debug, Clone)]
pub struct EntailsConnection;

impl Quality for EntailsConnection {
    type Individual = Rcc8Concept;
    type Value = bool;

    fn get(&self, c: &Rcc8Concept) -> Option<bool> {
        use Rcc8Concept as C;
        // Every relation entails connection except DC itself -- including
        // the RCC-5 coarsening abstractions ProperPart/ProperPartInverse,
        // which also imply connection.
        Some(!matches!(c, C::DisConnected))
    }
}

impl Ontology for Rcc8Ontology {
    type Cat = Rcc8Category;
    type Qual = EntailsConnection;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ClassificationIsJointlyExhaustive));
        axioms.push(Box::new(EqualityIsReflexive));
        axioms.push(Box::new(ProperPartInversesAreSymmetricPairs));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: for any two intervals, the realized classifier returns EXACTLY
/// one of the 8 relations, and it is deterministic (jointly exhaustive,
/// pairwise disjoint — Randell, Cui & Cohn 1992's own JEPD requirement on
/// the calculus, §3).
pub struct ClassificationIsJointlyExhaustive;

impl Axiom for ClassificationIsJointlyExhaustive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let pairs = [
            (Interval::new(0.0, 1.0), Interval::new(2.0, 3.0)), // DC
            (Interval::new(0.0, 1.0), Interval::new(1.0, 2.0)), // EC
            (Interval::new(0.0, 2.0), Interval::new(1.0, 3.0)), // PO
            (Interval::new(0.0, 1.0), Interval::new(0.0, 3.0)), // TPP
            (Interval::new(1.0, 2.0), Interval::new(0.0, 3.0)), // NTPP
            (Interval::new(0.0, 3.0), Interval::new(0.0, 1.0)), // TPPi
            (Interval::new(0.0, 3.0), Interval::new(1.0, 2.0)), // NTPPi
            (Interval::new(0.0, 1.0), Interval::new(0.0, 1.0)), // EQ
        ];
        // Deterministic: re-classifying the same pair agrees with itself.
        let deterministic = pairs
            .iter()
            .all(|&(a, b)| interval::classify(a, b) == interval::classify(a, b));
        // Every one of the 8 canonical shapes above actually produces a
        // DIFFERENT relation (exhaustive coverage of the 8 by these
        // examples, not a degenerate constant classifier).
        let classifications: alloc::vec::Vec<interval::Relation> = pairs
            .iter()
            .map(|&(a, b)| interval::classify(a, b))
            .collect();
        let all_distinct = (0..classifications.len()).all(|i| {
            (i + 1..classifications.len()).all(|j| classifications[i] != classifications[j])
        });
        if deterministic && all_distinct && classifications.len() == 8 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ClassificationIsJointlyExhaustive",
        "the realized classifier produces a deterministic, unique relation for each of the 8 canonical region-pair shapes",
        "Randell, Cui & Cohn (1992) \u{00a7}3"
    );
}

pr4xis::register_axiom!(
    ClassificationIsJointlyExhaustive,
    "Randell, Cui & Cohn (1992) \u{00a7}3"
);

/// Axiom: every region is EQ to itself (reflexivity of equality).
pub struct EqualityIsReflexive;

impl Axiom for EqualityIsReflexive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let regions = [
            Interval::new(0.0, 1.0),
            Interval::new(-5.0, 5.0),
            Interval::new(3.0, 3.0),
        ];
        if regions
            .iter()
            .all(|&r| interval::classify(r, r) == interval::Relation::Equal)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EqualityIsReflexive",
        "every region is EQ to itself",
        "Randell, Cui & Cohn (1992) \u{00a7}3"
    );
}

pr4xis::register_axiom!(EqualityIsReflexive, "Randell, Cui & Cohn (1992) \u{00a7}3");

/// Axiom: TPP/NTPP and their inverses TPPi/NTPPi always occur as a
/// swapped pair when the argument order is reversed (Randell, Cui & Cohn
/// 1992 §3's inverse-relation table).
pub struct ProperPartInversesAreSymmetricPairs;

impl Axiom for ProperPartInversesAreSymmetricPairs {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let inner = Interval::new(1.0, 2.0);
        let outer = Interval::new(0.0, 3.0);
        let ntpp_ok = interval::classify(inner, outer)
            == interval::Relation::NonTangentialProperPart
            && interval::classify(outer, inner)
                == interval::Relation::NonTangentialProperPartInverse;
        let boundary_inner = Interval::new(0.0, 1.0);
        let tpp_ok = interval::classify(boundary_inner, outer)
            == interval::Relation::TangentialProperPart
            && interval::classify(outer, boundary_inner)
                == interval::Relation::TangentialProperPartInverse;
        if ntpp_ok && tpp_ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ProperPartInversesAreSymmetricPairs",
        "TPP/NTPP and TPPi/NTPPi swap when the argument order is reversed",
        "Randell, Cui & Cohn (1992) \u{00a7}3"
    );
}

pr4xis::register_axiom!(
    ProperPartInversesAreSymmetricPairs,
    "Randell, Cui & Cohn (1992) \u{00a7}3"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<Rcc8Category>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        Rcc8Ontology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ten_concepts() {
        // The 8 base RCC-8 relations plus the 2 RCC-5 coarsening
        // abstractions (ProperPart, ProperPartInverse) TPP/NTPP and
        // TPPi/NTPPi are_a.
        assert_eq!(Rcc8Concept::variants().len(), 10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn entails_connection_is_total_and_only_dc_is_false() {
        let q = EntailsConnection;
        for c in Rcc8Concept::variants() {
            assert!(q.get(&c).is_some(), "{c:?} has no EntailsConnection");
        }
        assert_eq!(q.get(&Rcc8Concept::DisConnected), Some(false));
        assert_eq!(q.get(&Rcc8Concept::ExternallyConnected), Some(true));
        assert_eq!(q.get(&Rcc8Concept::Equal), Some(true));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn classification_is_jointly_exhaustive_holds() {
        assert!(ClassificationIsJointlyExhaustive.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn equality_is_reflexive_holds() {
        assert!(EqualityIsReflexive.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn proper_part_inverses_are_symmetric_pairs_holds() {
        assert!(ProperPartInversesAreSymmetricPairs.verify().is_ok());
    }

    fn arb_interval() -> impl Strategy<Value = Interval> {
        (-10.0f64..10.0, 0.0f64..10.0)
            .prop_map(|(start, width)| Interval::new(start, start + width))
    }

    proptest! {
        #[test]
        fn prop_classification_is_symmetric_under_the_expected_inverse(
            a in arb_interval(), b in arb_interval(),
        ) {
            let ab = interval::classify(a, b);
            let ba = interval::classify(b, a);
            let expected = match ab {
                interval::Relation::TangentialProperPart => interval::Relation::TangentialProperPartInverse,
                interval::Relation::NonTangentialProperPart => interval::Relation::NonTangentialProperPartInverse,
                interval::Relation::TangentialProperPartInverse => interval::Relation::TangentialProperPart,
                interval::Relation::NonTangentialProperPartInverse => interval::Relation::NonTangentialProperPart,
                same => same,
            };
            prop_assert_eq!(ba, expected);
        }

        #[test]
        fn prop_equal_regions_classify_as_eq(a in arb_interval()) {
            prop_assert_eq!(interval::classify(a, a), interval::Relation::Equal);
        }

        #[test]
        fn prop_disconnected_iff_not_connected(a in arb_interval(), b in arb_interval()) {
            let is_dc = interval::classify(a, b) == interval::Relation::DisConnected;
            prop_assert_eq!(is_dc, !interval::connected(a, b));
        }
    }

    pr4xis::register_praxis_value!(
        prop_classification_is_symmetric_under_the_expected_inverse,
        Verifiable
    );
    pr4xis::register_praxis_value!(prop_equal_regions_classify_as_eq, Verifiable);
    pr4xis::register_praxis_value!(prop_disconnected_iff_not_connected, Verifiable);
}

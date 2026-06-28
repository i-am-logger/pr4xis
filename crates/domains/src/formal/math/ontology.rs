//! Number-system hierarchy ontology — the inclusion chain
//! N ⊂ Z ⊂ Q ⊂ R ⊂ C.
//!
//! The five canonical number systems of mathematical analysis,
//! constructed in turn by Landau (1930): the naturals as Peano's
//! inductive set; the integers as quotient-pairs of naturals; the
//! rationals as quotient-pairs of integers; the reals via Dedekind
//! cuts or Cauchy sequences; the complex numbers as pairs of reals
//! under the Hamilton multiplication rule.
//!
//! # Literature
//!
//! - **Landau (1930)** *Grundlagen der Analysis* — the canonical
//!   construction of N → Z → Q → R → C as inclusions of number systems.
//! - **Peano (1889)** *Arithmetices Principia, Nova Methodo Exposita* —
//!   the axioms for the natural numbers.
//! - **Hamilton (1837)** "Theory of Conjugate Functions, or Algebraic
//!   Couples", *Trans. Royal Irish Academy* 17 — the construction of
//!   complex numbers as ordered pairs of reals.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Number",
    source: "Landau (1930) Grundlagen der Analysis; Peano (1889) Arithmetices Principia; Hamilton (1837) Theory of Conjugate Functions, Trans. Royal Irish Academy 17",

    concepts: [
        NaturalNumbers,
        Integers,
        Rationals,
        Reals,
        Complex,
    ],

    labels: {
        NaturalNumbers: ("en", "Natural numbers",
            "Peano (1889): the non-negative integers {0, 1, 2, ...} characterised by the Peano axioms (zero, successor, induction)."),
        Integers: ("en", "Integers",
            "Landau (1930) §2: the ring Z constructed as equivalence classes of pairs of naturals under the relation (a,b) ~ (c,d) iff a+d = b+c."),
        Rationals: ("en", "Rationals",
            "Landau (1930) §3: the field Q constructed as equivalence classes of pairs of integers (p, q != 0) under (a,b) ~ (c,d) iff ad = bc."),
        Reals: ("en", "Reals",
            "Landau (1930) §4: the field R obtained by Dedekind cuts (or equivalently Cauchy sequences) of rationals; the unique complete ordered field."),
        Complex: ("en", "Complex numbers",
            "Hamilton (1837): the field C constructed as ordered pairs (a, b) of reals with multiplication (a,b)(c,d) = (ac-bd, ad+bc); equivalently R[X]/(X^2 + 1)."),
    },

    // Strict inclusion chain — each smaller system embeds into the next
    // larger one. Landau (1930) constructs each step explicitly.
    is_a: [
        (NaturalNumbers, Integers),
        (Integers, Rationals),
        (Rationals, Reals),
        (Reals, Complex),
    ],
}

/// Quality: position of each number system in the inclusion chain
/// N ⊂ Z ⊂ Q ⊂ R ⊂ C — Landau (1930) construction order.
#[derive(Debug, Clone)]
pub struct DomainOrder;

impl Quality for DomainOrder {
    type Individual = NumberConcept;
    type Value = u8;

    fn get(&self, domain: &NumberConcept) -> Option<u8> {
        use pr4xis::category::{Arrow, Category, FinitelyGenerated};
        // Position in the N ⊂ Z ⊂ Q ⊂ R ⊂ C inclusion chain = the number of
        // systems strictly contained in `domain` (its proper descendants in the
        // loaded `is_a` graph), DERIVED from the morphisms — not a hand-numbered
        // u8 that must be re-typed if the chain changes (audit 2026-06-12 D-20).
        let sub: Vec<(NumberConcept, NumberConcept)> = NumberCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == NumberRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        let ancestors = |start: NumberConcept| -> Vec<NumberConcept> {
            let mut out = Vec::new();
            let mut stack = vec![start];
            while let Some(c) = stack.pop() {
                for (s, t) in &sub {
                    if *s == c && !out.contains(t) {
                        out.push(*t);
                        stack.push(*t);
                    }
                }
            }
            out
        };
        let descendants = NumberConcept::variants()
            .into_iter()
            .filter(|c| c != domain && ancestors(*c).contains(domain))
            .count();
        Some(descendants as u8)
    }
}

/// Quality: which systems form a field — closed under division.
/// Landau (1930) §3: Q is the smallest field containing the integers,
/// so Q, R, C are fields and N, Z are not.
#[derive(Debug, Clone)]
pub struct SupportsDivision;

impl Quality for SupportsDivision {
    type Individual = NumberConcept;
    type Value = ();

    fn get(&self, domain: &NumberConcept) -> Option<()> {
        match domain {
            NumberConcept::Rationals | NumberConcept::Reals | NumberConcept::Complex => Some(()),
            _ => None,
        }
    }
}

/// Quality: which systems support square roots of negatives.
/// Hamilton (1837): only C contains a root of X² + 1.
#[derive(Debug, Clone)]
pub struct SupportsNegativeSqrt;

impl Quality for SupportsNegativeSqrt {
    type Individual = NumberConcept;
    type Value = ();

    fn get(&self, domain: &NumberConcept) -> Option<()> {
        match domain {
            NumberConcept::Complex => Some(()),
            _ => None,
        }
    }
}

impl Ontology for NumberOntology {
    type Cat = NumberCategory;
    type Qual = DomainOrder;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ContainmentChain));
        axioms
    }
}

/// Domain axiom: the construction order N < Z < Q < R < C is strict —
/// each domain's order index is strictly less than its successor's.
/// Verifies the `DomainOrder` quality is consistent with the
/// `is_a` chain declared above.
pub struct ContainmentChain;

impl Axiom for ContainmentChain {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::FinitelyGenerated;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let order = DomainOrder;
        let domains = NumberConcept::variants();
        for i in 0..domains.len() {
            for j in i + 1..domains.len() {
                if order.get(&domains[i]).unwrap() >= order.get(&domains[j]).unwrap() {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ContainmentChain",
        "N < Z < Q < R < C (strict inclusion of number systems)",
        "Landau (1930) Grundlagen der Analysis"
    );
}

pr4xis::register_axiom!(ContainmentChain, "Landau (1930) Grundlagen der Analysis");

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn five_number_systems() {
        assert_eq!(NumberConcept::variants().len(), 5);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<NumberCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn containment_chain_holds() {
        assert!(ContainmentChain.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn division_supported_in_q_r_c() {
        // Three of the five (Q, R, C) form fields.
        let q = SupportsDivision;
        let count = NumberConcept::variants()
            .iter()
            .filter(|d| q.get(d).is_some())
            .count();
        assert_eq!(count, 3);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn negative_sqrt_only_in_complex() {
        let q = SupportsNegativeSqrt;
        let count = NumberConcept::variants()
            .iter()
            .filter(|d| q.get(d).is_some())
            .count();
        assert_eq!(count, 1);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn domain_ordering() {
        let order = DomainOrder;
        assert!(
            order.get(&NumberConcept::NaturalNumbers).unwrap()
                < order.get(&NumberConcept::Complex).unwrap()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        NumberOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}

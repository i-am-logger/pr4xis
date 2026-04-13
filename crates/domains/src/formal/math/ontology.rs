/// Math ontology: mathematical domains as entities with axioms.
use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Mathematical domains as entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum MathDomain {
    NaturalNumbers,
    Integers,
    Rationals,
    Reals,
    Complex,
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_ontology! {
    /// Discrete category over MathDomain entities.
    pub MathOntology for NumberHierarchy {
        concepts: MathDomain,
        relation: Subset,
    }
}

/// Quality: ordering of number domains by containment.
#[derive(Debug, Clone)]
pub struct DomainOrder;

impl Quality for DomainOrder {
    type Individual = MathDomain;
    type Value = u8;

    fn get(&self, domain: &MathDomain) -> Option<u8> {
        Some(match domain {
            MathDomain::NaturalNumbers => 0,
            MathDomain::Integers => 1,
            MathDomain::Rationals => 2,
            MathDomain::Reals => 3,
            MathDomain::Complex => 4,
        })
    }
}

/// Quality: does this domain support division?
#[derive(Debug, Clone)]
pub struct SupportsDivision;

impl Quality for SupportsDivision {
    type Individual = MathDomain;
    type Value = ();

    fn get(&self, domain: &MathDomain) -> Option<()> {
        match domain {
            MathDomain::Rationals | MathDomain::Reals | MathDomain::Complex => Some(()),
            _ => None,
        }
    }
}

/// Quality: does this domain support square roots of negatives?
#[derive(Debug, Clone)]
pub struct SupportsNegativeSqrt;

impl Quality for SupportsNegativeSqrt {
    type Individual = MathDomain;
    type Value = ();

    fn get(&self, domain: &MathDomain) -> Option<()> {
        match domain {
            MathDomain::Complex => Some(()),
            _ => None,
        }
    }
}

impl Ontology for MathOntology {
    type Cat = NumberHierarchy;
    type Qual = DomainOrder;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(ContainmentChain)]
    }
}

/// Axiom: N < Z < Q < R < C (strict containment chain).
pub struct ContainmentChain;

impl Axiom for ContainmentChain {
    fn description(&self) -> &str {
        "N < Z < Q < R < C"
    }

    fn holds(&self) -> bool {
        let order = DomainOrder;
        let domains = MathDomain::variants();
        // Each domain has a strictly increasing order
        for i in 0..domains.len() {
            for j in i + 1..domains.len() {
                if order.get(&domains[i]).unwrap() >= order.get(&domains[j]).unwrap() {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_5_domains() {
        assert_eq!(MathDomain::variants().len(), 5);
    }

    #[test]
    fn test_category_laws() {
        pr4xis::category::validate::check_category_laws::<NumberHierarchy>().unwrap();
    }

    #[test]
    fn test_containment_chain() {
        assert!(ContainmentChain.holds());
    }

    #[test]
    fn test_division_support() {
        assert_eq!(SupportsDivision.individuals_with().len(), 3); // Q, R, C
    }

    #[test]
    fn test_negative_sqrt_only_complex() {
        assert_eq!(SupportsNegativeSqrt.individuals_with().len(), 1); // C only
    }

    #[test]
    fn test_domain_ordering() {
        let order = DomainOrder;
        assert!(
            order.get(&MathDomain::NaturalNumbers).unwrap()
                < order.get(&MathDomain::Complex).unwrap()
        );
    }

    #[test]
    fn ontology_validates() {
        MathOntology::validate().unwrap();
    }
}

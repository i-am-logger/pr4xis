use super::ontology::*;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

#[test]
fn category_laws() {
    assert_category_laws::<AlgebraCategory>();
}

#[test]
fn ontology_validates() {
    AlgebraOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[test]
fn fourteen_concepts() {
    assert_eq!(AlgebraConcept::variants().len(), 14);
}

#[test]
fn adjoint_triple_holds() {
    assert!(AdjointTriple.verify().is_ok());
}

#[test]
fn coproduct_product_dual_holds() {
    assert!(CoproductProductDual.verify().is_ok());
}

#[test]
fn coproduct_is_a_colimit() {
    let sub: Vec<_> = AlgebraCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == AlgebraRelationKind::Subsumption)
        .map(|m| (m.source(), m.target()))
        .collect();
    assert!(sub.contains(&(AlgebraConcept::Coproduct, AlgebraConcept::Colimit)));
}

#[test]
fn product_is_a_limit() {
    let sub: Vec<_> = AlgebraCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == AlgebraRelationKind::Subsumption)
        .map(|m| (m.source(), m.target()))
        .collect();
    assert!(sub.contains(&(AlgebraConcept::Product, AlgebraConcept::Limit)));
}

#[test]
fn pushout_needs_span() {
    let parts: Vec<_> = AlgebraCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == AlgebraRelationKind::Parthood)
        .map(|m| (m.source(), m.target()))
        .collect();
    assert!(parts.contains(&(AlgebraConcept::Pushout, AlgebraConcept::Span)));
}

#[test]
fn is_operation_total() {
    let q = IsOperation;
    for c in AlgebraConcept::variants() {
        assert!(q.get(&c).is_some());
    }
}

fn arb_concept() -> impl Strategy<Value = AlgebraConcept> {
    proptest::sample::select(AlgebraConcept::variants())
}

proptest! {
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in AlgebraCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    #[test]
    fn prop_structural_axioms_hold(_seed in any::<u32>()) {
        for axiom in AlgebraOntology::axioms() {
            if let Err(c) = axiom.verify() {
                prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
            }
        }
    }

    #[test]
    fn prop_is_operation_total(c in arb_concept()) {
        prop_assert!(IsOperation.get(&c).is_some());
    }

    #[test]
    fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
        let opposed: std::collections::HashSet<_> = AlgebraCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AlgebraRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        for (a, b) in opposed.iter() {
            prop_assert!(opposed.contains(&(*b, *a)),
                "opposition not symmetric: {:?} -> {:?} but not back", a, b);
        }
    }
}

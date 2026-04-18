#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use pr4xis::category::entity::Concept;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn c1_category_laws() {
    check_category_laws::<C1Category>().unwrap();
}

#[test]
fn c2_category_laws() {
    check_category_laws::<C2Category>().unwrap();
}

#[test]
fn c1_validates() {
    C1Ontology::validate().unwrap();
}

#[test]
fn c2_validates() {
    C2Ontology::validate().unwrap();
}

#[test]
fn c1_seven_concepts() {
    assert_eq!(C1Concept::variants().len(), 7);
}

#[test]
fn c2_seven_concepts() {
    assert_eq!(C2Concept::variants().len(), 7);
}

#[test]
fn attention_selects_access() {
    assert!(AttentionCausesAccess.holds());
}

#[test]
fn higher_order_represents_first() {
    assert!(HigherOrderRepresentsFirst.holds());
}

#[test]
fn c1_c2_orthogonal() {
    let c1_names: Vec<_> = C1Concept::variants().iter().map(|c| c.name()).collect();
    let c2_names: Vec<_> = C2Concept::variants().iter().map(|c| c.name()).collect();
    let shared: Vec<_> = c1_names.iter().filter(|n| c2_names.contains(n)).collect();
    assert!(
        shared.is_empty(),
        "C1 and C2 should be orthogonal (no shared concepts), but share: {:?}",
        shared
    );
}

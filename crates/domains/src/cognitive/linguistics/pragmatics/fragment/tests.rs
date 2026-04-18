use super::ontology::*;
use pr4xis::category::entity::Concept;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    check_category_laws::<FragmentCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    FragmentOntology::validate().unwrap();
}

#[test]
fn twelve_concepts() {
    assert_eq!(FragmentConcept::variants().len(), 12);
}

#[test]
fn all_fragments_classified() {
    assert!(AllFragmentsClassified.holds());
}

#[test]
fn eight_fragment_types() {
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
    let rels = FragmentTaxonomy::relations();
    let count = rels
        .iter()
        .filter(|(_, parent)| *parent == FragmentConcept::Fragment)
        .count();
    assert_eq!(count, 8);
}

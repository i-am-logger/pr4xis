use super::ontology::*;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    check_category_laws::<TextCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    TextOntology::validate().unwrap();
}

#[test]
fn nine_concepts() {
    assert_eq!(TextConcept::variants().len(), 9);
}

#[test]
fn word_is_fully_connected() {
    assert!(WordIsFullyConnected.holds());
}

#[test]
fn two_level_containment() {
    assert!(TwoLevelContainment.holds());
}

#[test]
fn word_is_a_span() {
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
    let rels = TextTaxonomy::relations();
    assert!(
        rels.iter()
            .any(|(child, parent)| *child == TextConcept::Word && *parent == TextConcept::Span)
    );
}

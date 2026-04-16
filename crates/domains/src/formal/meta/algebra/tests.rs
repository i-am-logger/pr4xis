use super::ontology::*;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    check_category_laws::<AlgebraCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    AlgebraOntology::validate().unwrap();
}

#[test]
fn fourteen_concepts() {
    assert_eq!(AlgebraConcept::variants().len(), 14);
}

#[test]
fn adjoint_triple() {
    assert!(AdjointTriple.holds());
}

#[test]
fn coproduct_product_dual() {
    assert!(CoproductProductDual.holds());
}

#[test]
fn coproduct_is_a_colimit() {
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
    let rels = AlgebraTaxonomy::relations();
    assert!(
        rels.iter()
            .any(|(c, p)| *c == AlgebraConcept::Coproduct && *p == AlgebraConcept::Colimit)
    );
}

#[test]
fn product_is_a_limit() {
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
    let rels = AlgebraTaxonomy::relations();
    assert!(
        rels.iter()
            .any(|(c, p)| *c == AlgebraConcept::Product && *p == AlgebraConcept::Limit)
    );
}

#[test]
fn pushout_needs_span() {
    use pr4xis::ontology::reasoning::mereology::MereologyDef;
    let parts = AlgebraMereology::relations();
    assert!(
        parts
            .iter()
            .any(|(w, p)| *w == AlgebraConcept::Pushout && *p == AlgebraConcept::Span)
    );
}

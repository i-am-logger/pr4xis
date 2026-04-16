use super::ontology::*;
use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    check_category_laws::<DiscourseCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    DiscourseOntology::validate().unwrap();
}

#[test]
fn six_concepts() {
    assert_eq!(DiscourseConcept::variants().len(), 6);
}

#[test]
fn nucleus_satellite_asymmetric() {
    assert!(NucleusSatelliteAsymmetric.holds());
}

#[test]
fn multinuclear_exists() {
    assert!(MultinuclearExists.holds());
}

#[test]
fn elaboration_connects_nucleus_to_satellite() {
    let m = DiscourseCategory::morphisms();
    assert!(m.iter().any(|r| r.from == DiscourseConcept::Nucleus
        && r.to == DiscourseConcept::Satellite
        && r.kind == DiscourseRelationKind::Elaboration));
}

#[test]
fn structure_contains_segments() {
    let m = DiscourseCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.from == DiscourseConcept::DiscourseStructure
                && r.to == DiscourseConcept::DiscourseSegment
                && r.kind == DiscourseRelationKind::Contains)
    );
}

use super::ontology::*;
use pr4xis::category::Category;
use pr4xis::category::entity::Concept;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    assert_category_laws::<DiscourseCategory>();
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
    assert!(NucleusSatelliteAsymmetric.verify().is_ok());
}

#[test]
fn multinuclear_exists() {
    assert!(MultinuclearExists.verify().is_ok());
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

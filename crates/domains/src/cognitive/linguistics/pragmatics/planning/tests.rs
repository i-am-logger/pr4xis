use super::ontology::*;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[test]
fn category_laws() {
    check_category_laws::<PlanningCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    PlanningOntology::validate().unwrap();
}

#[test]
fn fourteen_concepts() {
    assert_eq!(PlanningConcept::variants().len(), 14);
}

#[test]
fn bdi_produces_intention() {
    assert!(BdiProducesIntention.holds());
}

#[test]
fn effect_updates_common_ground() {
    assert!(EffectUpdatesCommonGround.holds());
}

#[test]
fn goals_specialize() {
    assert!(GoalsSpecialize.holds());
}

#[test]
fn all_concepts_have_role() {
    for c in PlanningConcept::variants() {
        assert!(ConceptRole.get(&c).is_some(), "{:?} missing role", c);
    }
}

#[test]
fn plan_reaches_common_ground() {
    use pr4xis::category::Category;
    let m = PlanningCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.from == PlanningConcept::Plan && r.to == PlanningConcept::CommonGround),
        "Plan should reach CommonGround transitively"
    );
}

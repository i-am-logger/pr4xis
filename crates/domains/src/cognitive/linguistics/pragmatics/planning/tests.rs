use super::ontology::*;
use pr4xis::category::entity::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[test]
fn category_laws() {
    assert_category_laws::<PlanningCategory>();
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
    assert!(BdiProducesIntention.verify().is_ok());
}

#[test]
fn effect_updates_common_ground() {
    assert!(EffectUpdatesCommonGround.verify().is_ok());
}

#[test]
fn goals_specialize() {
    assert!(GoalsSpecialize.verify().is_ok());
}

#[test]
fn all_concepts_have_role() {
    for c in PlanningConcept::variants() {
        assert!(ConceptRole.get(&c).is_some(), "{:?} missing role", c);
    }
}

#[test]
fn plan_reaches_common_ground() {
    // Plan → Action → … → CommonGround spans heterogeneous kinds. Per #166
    // closure across heterogeneous kinds isn't a single morphism — walk
    // the graph.
    use pr4xis::category::{Arrow, Category};
    use std::collections::{HashSet, VecDeque};
    let ms = PlanningCategory::morphisms();
    let mut visited: HashSet<PlanningConcept> = HashSet::new();
    let mut queue: VecDeque<PlanningConcept> = VecDeque::new();
    queue.push_back(PlanningConcept::Plan);
    let mut reaches = false;
    while let Some(n) = queue.pop_front() {
        if n == PlanningConcept::CommonGround {
            reaches = true;
            break;
        }
        if !visited.insert(n) {
            continue;
        }
        for m in ms.iter().filter(|m| m.source() == n) {
            queue.push_back(m.target());
        }
    }
    assert!(reaches, "Plan should reach CommonGround transitively");
}

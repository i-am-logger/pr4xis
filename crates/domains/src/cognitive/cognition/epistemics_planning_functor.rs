// Epistemics → Planning functor.
//
// What we know becomes what we believe in the BDI model.
// The epistemic state IS the Belief component of BDI.
// Epistemic transitions map to planning operations.
//
// KnownKnown → Belief (I know → I believe)
// KnownUnknown → Desire (I know I don't know → I want to know)
// UnknownKnown → Precondition (knowledge exists but needs surfacing)
// UnknownUnknown → CommunicativeGoal (need to establish context)
//
// Source: von Foerster (1981) × Bratman (1987)

use pr4xis::category::Functor;

use crate::cognitive::cognition::epistemics::{
    EpistemicCategory, EpistemicConcept, EpistemicRelation, EpistemicRelationKind,
};
use crate::cognitive::linguistics::pragmatics::planning::ontology::{
    PlanningCategory, PlanningConcept, PlanningRelation, PlanningRelationKind,
};

pub struct EpistemicsToPlanning;

impl Functor for EpistemicsToPlanning {
    type Source = EpistemicCategory;
    type Target = PlanningCategory;

    fn map_object(obj: &EpistemicConcept) -> PlanningConcept {
        match obj {
            EpistemicConcept::KnownKnown => PlanningConcept::Belief,
            EpistemicConcept::KnownUnknown => PlanningConcept::Desire,
            EpistemicConcept::UnknownKnown => PlanningConcept::Precondition,
            EpistemicConcept::UnknownUnknown => PlanningConcept::CommunicativeGoal,
        }
    }

    fn map_morphism(m: &EpistemicRelation) -> PlanningRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            EpistemicRelationKind::Identity => PlanningRelationKind::Identity,
            EpistemicRelationKind::Observation => PlanningRelationKind::Produces,
            EpistemicRelationKind::Learning => PlanningRelationKind::Produces,
            EpistemicRelationKind::Repair => PlanningRelationKind::Produces,
            EpistemicRelationKind::Discovery => PlanningRelationKind::Produces,
            EpistemicRelationKind::Forgetting => PlanningRelationKind::Updates,
            EpistemicRelationKind::Composed => PlanningRelationKind::Composed,
        };
        PlanningRelation { from, to, kind }
    }
}
pr4xis::register_functor!(EpistemicsToPlanning);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<EpistemicsToPlanning>().unwrap();
    }
}

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
    EpistemicCategory, EpistemicState, EpistemicTransition, TransitionKind,
};
use crate::cognitive::linguistics::pragmatics::planning::ontology::{
    PlanningCategory, PlanningConcept, PlanningRelation, PlanningRelationKind,
};

pub struct EpistemicsToPlanning;

impl Functor for EpistemicsToPlanning {
    type Source = EpistemicCategory;
    type Target = PlanningCategory;

    fn map_object(obj: &EpistemicState) -> PlanningConcept {
        match obj {
            EpistemicState::KnownKnown => PlanningConcept::Belief,
            EpistemicState::KnownUnknown => PlanningConcept::Desire,
            EpistemicState::UnknownKnown => PlanningConcept::Precondition,
            EpistemicState::UnknownUnknown => PlanningConcept::CommunicativeGoal,
        }
    }

    fn map_morphism(m: &EpistemicTransition) -> PlanningRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            TransitionKind::Identity => PlanningRelationKind::Identity,
            TransitionKind::Observation => PlanningRelationKind::Produces,
            TransitionKind::Learning => PlanningRelationKind::Produces,
            TransitionKind::Repair => PlanningRelationKind::Produces,
            TransitionKind::Discovery => PlanningRelationKind::Produces,
            TransitionKind::Forgetting => PlanningRelationKind::Updates,
            TransitionKind::Composed => PlanningRelationKind::Composed,
        };
        PlanningRelation { from, to, kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<EpistemicsToPlanning>().unwrap();
    }
}

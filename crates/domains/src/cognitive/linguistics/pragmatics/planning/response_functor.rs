// Planning → Response functor.
//
// The plan determines the response: BDI intention becomes response
// intent, the plan's speech act operator becomes the speech act type,
// the common ground update becomes the surface form.
//
// Plan → SpeechActType (what kind of response)
// Intention → Intent (what we intend to express)
// Effect → Content (the information to convey)
// CommonGround → Context (shared knowledge)
// CommunicativeGoal → EpistemicFrame (what goal frames the response)
//
// Source: Cohen & Perrault (1979) → Reiter & Dale (2000)

use pr4xis::category::Functor;

use super::ontology::{PlanningCategory, PlanningConcept, PlanningRelation, PlanningRelationKind};
use crate::cognitive::linguistics::pragmatics::response::{
    ResponseCategory, ResponseConcept, ResponseRelation, ResponseRelationKind,
};

pub struct PlanningToResponse;

impl Functor for PlanningToResponse {
    type Source = PlanningCategory;
    type Target = ResponseCategory;

    fn map_object(obj: &PlanningConcept) -> ResponseConcept {
        match obj {
            // BDI → Response:
            PlanningConcept::Belief => ResponseConcept::Content,
            PlanningConcept::Desire => ResponseConcept::Intent,
            PlanningConcept::Intention => ResponseConcept::Intent,
            // Planning → Response:
            PlanningConcept::SpeechActOperator => ResponseConcept::SpeechActType,
            PlanningConcept::Precondition => ResponseConcept::EpistemicFrame,
            PlanningConcept::Effect => ResponseConcept::Content,
            PlanningConcept::Plan => ResponseConcept::SpeechActType,
            // Stalnaker → Response:
            PlanningConcept::CommonGround => ResponseConcept::Context,
            PlanningConcept::CommonGroundUpdate => ResponseConcept::SurfaceForm,
            // Jakobson goals → Response:
            PlanningConcept::CommunicativeGoal => ResponseConcept::EpistemicFrame,
            PlanningConcept::InformativeGoal => ResponseConcept::Content,
            PlanningConcept::PhaticGoal => ResponseConcept::SurfaceForm,
            PlanningConcept::DirectiveGoal => ResponseConcept::Intent,
            PlanningConcept::ExpressiveGoal => ResponseConcept::SurfaceForm,
        }
    }

    fn map_morphism(m: &PlanningRelation) -> ResponseRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            PlanningRelationKind::Identity => ResponseRelationKind::Identity,
            PlanningRelationKind::Produces => ResponseRelationKind::Determines,
            PlanningRelationKind::Selects => ResponseRelationKind::Selects,
            PlanningRelationKind::ConsistsOf => ResponseRelationKind::Shapes,
            PlanningRelationKind::HasPrecondition => ResponseRelationKind::Frames,
            PlanningRelationKind::HasEffect => ResponseRelationKind::Realizes,
            PlanningRelationKind::Updates => ResponseRelationKind::Constrains,
            PlanningRelationKind::Specializes => ResponseRelationKind::Determines,
            PlanningRelationKind::Composed => ResponseRelationKind::Composed,
        };
        ResponseRelation { from, to, kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<PlanningToResponse>().unwrap();
    }
}

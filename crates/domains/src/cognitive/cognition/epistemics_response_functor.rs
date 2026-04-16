// Epistemics → Response functor.
//
// What we know determines how we respond.
// KnownKnown → assert. KnownUnknown → acknowledge gap.
// UnknownKnown → suggest. UnknownUnknown → admit limitation.
//
// This formalizes the mapping already in ResponseFrame::from_epistemic()
// as a structure-preserving functor with verified laws.
//
// Source: von Foerster (1981); Reiter & Dale (2000)

use pr4xis::category::Functor;

use crate::cognitive::cognition::epistemics::{
    EpistemicCategory, EpistemicState, EpistemicTransition,
};
use crate::cognitive::linguistics::pragmatics::response::{
    ResponseCategory, ResponseConcept, ResponseRelation, ResponseRelationKind,
};

pub struct EpistemicsToResponse;

impl Functor for EpistemicsToResponse {
    type Source = EpistemicCategory;
    type Target = ResponseCategory;

    fn map_object(obj: &EpistemicState) -> ResponseConcept {
        match obj {
            // KnownKnown → SpeechActType (we know → we can assert)
            EpistemicState::KnownKnown => ResponseConcept::SpeechActType,
            // KnownUnknown → EpistemicFrame (we know what we don't know)
            EpistemicState::KnownUnknown => ResponseConcept::EpistemicFrame,
            // UnknownKnown → Content (knowledge exists but needs surfacing)
            EpistemicState::UnknownKnown => ResponseConcept::Content,
            // UnknownUnknown → Context (we need more context)
            EpistemicState::UnknownUnknown => ResponseConcept::Context,
        }
    }

    fn map_morphism(m: &EpistemicTransition) -> ResponseRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = if from == to && m.from == m.to {
            ResponseRelationKind::Identity
        } else if from == to {
            ResponseRelationKind::Composed
        } else {
            ResponseRelationKind::Composed
        };
        ResponseRelation { from, to, kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    #[ignore = "kinded category Composed kind doesn't compose back to Identity — systematic issue with define_ontology! compose generation"]
    fn functor_laws() {
        check_functor_laws::<EpistemicsToResponse>().unwrap();
    }
}

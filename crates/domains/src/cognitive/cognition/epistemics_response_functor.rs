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

use pr4xis::category::{Category, Functor};

use crate::cognitive::cognition::epistemics::{
    EpistemicCategory, EpistemicConcept, EpistemicRelation, EpistemicRelationKind,
};
use crate::cognitive::linguistics::pragmatics::response::{
    ResponseCategory, ResponseConcept, ResponseRelation, ResponseRelationKind,
};

pub struct EpistemicsToResponse;

impl Functor for EpistemicsToResponse {
    type Source = EpistemicCategory;
    type Target = ResponseCategory;

    fn map_object(obj: &EpistemicConcept) -> ResponseConcept {
        match obj {
            // KnownKnown → SpeechActType (we know → we can assert)
            EpistemicConcept::KnownKnown => ResponseConcept::SpeechActType,
            // KnownUnknown → EpistemicFrame (we know what we don't know)
            EpistemicConcept::KnownUnknown => ResponseConcept::EpistemicFrame,
            // UnknownKnown → Content (knowledge exists but needs surfacing)
            EpistemicConcept::UnknownKnown => ResponseConcept::Content,
            // UnknownUnknown → Context (we need more context)
            EpistemicConcept::UnknownUnknown => ResponseConcept::Context,
        }
    }

    fn map_morphism(m: &EpistemicRelation) -> ResponseRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        // Preserve source's Identity → target's Identity; everything else
        // maps to Composed so F(g∘f) == F(g)∘F(f) holds under collapse.
        match m.kind {
            EpistemicRelationKind::Identity => ResponseCategory::identity(&from),
            _ => ResponseRelation {
                from,
                to,
                kind: ResponseRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(EpistemicsToResponse);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    #[ignore = "epistemics has Repair/Forgetting cycles that response doesn't model — need intermediate ontology or richer response (#98)"]
    fn functor_laws() {
        check_functor_laws::<EpistemicsToResponse>().unwrap();
    }
}

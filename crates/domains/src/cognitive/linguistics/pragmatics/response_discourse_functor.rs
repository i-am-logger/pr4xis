// Response → Discourse functor.
//
// Response frame determines rhetorical structure.
// Rhetorical relations are now morphism kinds in Discourse, not concepts.
//
// Source: Reiter & Dale (2000); Mann & Thompson RST (1988)

use pr4xis::category::Functor;

use super::discourse::ontology::{
    DiscourseCategory, DiscourseConcept, DiscourseRelation, DiscourseRelationKind,
};
use super::response::{ResponseCategory, ResponseConcept, ResponseRelation, ResponseRelationKind};

pub struct ResponseToDiscourse;

impl Functor for ResponseToDiscourse {
    type Source = ResponseCategory;
    type Target = DiscourseCategory;

    fn map_object(obj: &ResponseConcept) -> DiscourseConcept {
        match obj {
            ResponseConcept::Intent => DiscourseConcept::Topic,
            ResponseConcept::EpistemicFrame => DiscourseConcept::DiscourseStructure,
            ResponseConcept::Content => DiscourseConcept::Nucleus,
            ResponseConcept::SpeechActType => DiscourseConcept::DiscourseSegment,
            ResponseConcept::SurfaceForm => DiscourseConcept::TextSpan,
            ResponseConcept::Context => DiscourseConcept::Satellite,
        }
    }

    fn map_morphism(m: &ResponseRelation) -> DiscourseRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            ResponseRelationKind::Identity => DiscourseRelationKind::Identity,
            ResponseRelationKind::Determines => DiscourseRelationKind::Cause,
            ResponseRelationKind::Frames => DiscourseRelationKind::Background,
            ResponseRelationKind::Selects => DiscourseRelationKind::Elaboration,
            ResponseRelationKind::Realizes => DiscourseRelationKind::Restatement,
            ResponseRelationKind::Constrains => DiscourseRelationKind::Condition,
            ResponseRelationKind::Shapes => DiscourseRelationKind::Contains,
            ResponseRelationKind::Composed => DiscourseRelationKind::Composed,
        };
        DiscourseRelation { from, to, kind }
    }
}
pr4xis::register_functor!(ResponseToDiscourse);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<ResponseToDiscourse>().unwrap();
    }
}

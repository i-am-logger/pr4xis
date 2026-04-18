// Discourse → NLG functor.
//
// Discourse structural concepts map to NLG pipeline stages.
// Rhetorical relations (now morphism kinds in Discourse) map
// to NLG relation kinds.
//
// Source: Mann & Thompson (1988) → Reiter & Dale (2000)

use pr4xis::category::Functor;

use super::discourse::ontology::{
    DiscourseCategory, DiscourseConcept, DiscourseRelation, DiscourseRelationKind,
};
use super::nlg::{NlgCategory, NlgConcept, NlgRelation, NlgRelationKind};

pub struct DiscourseToNlg;

impl Functor for DiscourseToNlg {
    type Source = DiscourseCategory;
    type Target = NlgCategory;

    fn map_object(obj: &DiscourseConcept) -> NlgConcept {
        match obj {
            DiscourseConcept::TextSpan => NlgConcept::SurfaceText,
            DiscourseConcept::Nucleus => NlgConcept::Message,
            DiscourseConcept::Satellite => NlgConcept::ReferringExpression,
            DiscourseConcept::DiscourseSegment => NlgConcept::ContentDetermination,
            DiscourseConcept::DiscourseStructure => NlgConcept::DocumentPlanning,
            DiscourseConcept::Topic => NlgConcept::CommunicativeGoal,
        }
    }

    fn map_morphism(m: &DiscourseRelation) -> NlgRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            DiscourseRelationKind::Identity => NlgRelationKind::Identity,
            DiscourseRelationKind::Contains => NlgRelationKind::Organizes,
            DiscourseRelationKind::Elaboration => NlgRelationKind::Organizes,
            DiscourseRelationKind::Sequence => NlgRelationKind::Organizes,
            DiscourseRelationKind::Cause => NlgRelationKind::Organizes,
            DiscourseRelationKind::Condition => NlgRelationKind::Organizes,
            DiscourseRelationKind::Purpose => NlgRelationKind::Organizes,
            DiscourseRelationKind::Background => NlgRelationKind::Gathers,
            DiscourseRelationKind::Justify => NlgRelationKind::Organizes,
            DiscourseRelationKind::Restatement => NlgRelationKind::Produces,
            DiscourseRelationKind::Concession => NlgRelationKind::Organizes,
            DiscourseRelationKind::Contrast => NlgRelationKind::Organizes,
            DiscourseRelationKind::Narration => NlgRelationKind::Precedes,
            DiscourseRelationKind::Explanation => NlgRelationKind::Organizes,
            DiscourseRelationKind::Parallel => NlgRelationKind::Organizes,
            DiscourseRelationKind::Continuation => NlgRelationKind::Precedes,
            DiscourseRelationKind::Composed => NlgRelationKind::Composed,
            DiscourseRelationKind::Subsumption => NlgRelationKind::Organizes,
            DiscourseRelationKind::Opposition => NlgRelationKind::Organizes,
        };
        NlgRelation { from, to, kind }
    }
}
pr4xis::register_functor!(DiscourseToNlg);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<DiscourseToNlg>().unwrap();
    }
}

// Metacognition → Epistemics functor.
//
// What metacognition monitors becomes what we know.
// The meta-level's assessment of its own processing maps to
// the epistemic state of the system.
//
// Nelson & Narens (1990) → von Foerster (1981):
// Monitoring → Observation (detecting knowledge states)
// Gap → KnownUnknown (I know I don't know)
// Evaluation → KnownKnown (assessment produces knowledge)
// Repair → Repair (direct match)
//
// Source: Nelson & Narens (1990); von Foerster (1981); Koriat (2007)

use pr4xis::category::Functor;

use crate::cognitive::cognition::epistemics::{
    EpistemicCategory, EpistemicConcept, EpistemicRelation, EpistemicRelationKind,
};
use crate::cognitive::cognition::metacognition::{
    MetaCognitionCategory, MetaCognitionConcept, MetaCognitionRelation,
};

pub struct MetacognitionToEpistemics;

impl Functor for MetacognitionToEpistemics {
    type Source = MetaCognitionCategory;
    type Target = EpistemicCategory;

    fn map_object(obj: &MetaCognitionConcept) -> EpistemicConcept {
        match obj {
            // MetaLevel IS knowing-that-you-know
            MetaCognitionConcept::MetaLevel => EpistemicConcept::KnownKnown,
            // ObjectLevel may be inaccessible (processing without awareness)
            MetaCognitionConcept::ObjectLevel => EpistemicConcept::UnknownKnown,
            // Monitoring IS self-observation
            MetaCognitionConcept::Monitoring => EpistemicConcept::KnownKnown,
            // Evaluation produces knowledge
            MetaCognitionConcept::Evaluation => EpistemicConcept::KnownKnown,
            // Control directs learning
            MetaCognitionConcept::Control => EpistemicConcept::KnownKnown,
            // Trace IS evidence of knowledge
            MetaCognitionConcept::Trace => EpistemicConcept::KnownKnown,
            // Gap IS knowing you don't know (Rumsfeld's known unknown)
            MetaCognitionConcept::Gap => EpistemicConcept::KnownUnknown,
            // Repair maps directly
            MetaCognitionConcept::Repair => EpistemicConcept::KnownKnown,
            // Clarification = acknowledging ignorance
            MetaCognitionConcept::Clarification => EpistemicConcept::KnownUnknown,
            // EpistemicAssessment IS already epistemic
            MetaCognitionConcept::EpistemicAssessment => EpistemicConcept::KnownKnown,
        }
    }

    fn map_morphism(m: &MetaCognitionRelation) -> EpistemicRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = if from == to && m.from == m.to {
            EpistemicRelationKind::Identity
        } else if from == to {
            EpistemicRelationKind::Composed
        } else {
            match (&from, &to) {
                (EpistemicConcept::UnknownKnown, EpistemicConcept::KnownKnown) => {
                    EpistemicRelationKind::Repair
                }
                (EpistemicConcept::KnownUnknown, EpistemicConcept::KnownKnown) => {
                    EpistemicRelationKind::Learning
                }
                _ => EpistemicRelationKind::Composed,
            }
        };
        EpistemicRelation { from, to, kind }
    }
}
pr4xis::register_functor!(MetacognitionToEpistemics);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<MetacognitionToEpistemics>().unwrap();
    }
}

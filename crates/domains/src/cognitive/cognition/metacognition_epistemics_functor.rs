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
    EpistemicCategory, EpistemicState, EpistemicTransition, TransitionKind,
};
use crate::cognitive::cognition::metacognition::{
    MetaCognitionCategory, MetaConcept, MetaRelation,
};

pub struct MetacognitionToEpistemics;

impl Functor for MetacognitionToEpistemics {
    type Source = MetaCognitionCategory;
    type Target = EpistemicCategory;

    fn map_object(obj: &MetaConcept) -> EpistemicState {
        match obj {
            // MetaLevel IS knowing-that-you-know
            MetaConcept::MetaLevel => EpistemicState::KnownKnown,
            // ObjectLevel may be inaccessible (processing without awareness)
            MetaConcept::ObjectLevel => EpistemicState::UnknownKnown,
            // Monitoring IS self-observation
            MetaConcept::Monitoring => EpistemicState::KnownKnown,
            // Evaluation produces knowledge
            MetaConcept::Evaluation => EpistemicState::KnownKnown,
            // Control directs learning
            MetaConcept::Control => EpistemicState::KnownKnown,
            // Trace IS evidence of knowledge
            MetaConcept::Trace => EpistemicState::KnownKnown,
            // Gap IS knowing you don't know (Rumsfeld's known unknown)
            MetaConcept::Gap => EpistemicState::KnownUnknown,
            // Repair maps directly
            MetaConcept::Repair => EpistemicState::KnownKnown,
            // Clarification = acknowledging ignorance
            MetaConcept::Clarification => EpistemicState::KnownUnknown,
            // EpistemicAssessment IS already epistemic
            MetaConcept::EpistemicAssessment => EpistemicState::KnownKnown,
        }
    }

    fn map_morphism(m: &MetaRelation) -> EpistemicTransition {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = if from == to && m.from == m.to {
            TransitionKind::Identity
        } else if from == to {
            TransitionKind::Composed
        } else {
            match (&from, &to) {
                (EpistemicState::UnknownKnown, EpistemicState::KnownKnown) => {
                    TransitionKind::Repair
                }
                (EpistemicState::KnownUnknown, EpistemicState::KnownKnown) => {
                    TransitionKind::Learning
                }
                _ => TransitionKind::Composed,
            }
        };
        EpistemicTransition { from, to, kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<MetacognitionToEpistemics>().unwrap();
    }
}

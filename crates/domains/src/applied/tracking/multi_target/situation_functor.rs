//! Functor: MultiTarget → Situation (a track is an identified entity).
//!
//! Closes the "track → threat picture" seam: the multi-target track-management
//! category maps into the JDL Level-2 situation ontology. Every track, in any
//! lifecycle state (Tentative / Confirmed / Coasting / Deleted), is an
//! identified entity — the JDL Level-1 output (`SituationConcept::Concept`) that
//! situation assessment reasons over (Steinberg & Bowman 2008: Level 1 object
//! refinement feeds the entity input to Level 2). The lifecycle transitions
//! (Confirm / Miss / Delete / ReDetect) are below the resolution of the
//! situation layer, so this is the **constant functor** onto `Concept`: it
//! forgets the track-lifecycle structure and retains only entity-hood.
//!
//! Composed with `SituationToCompliance`, this realises the documented decision
//! chain end to end: a track grounds the `Identify` escalation posture, and the
//! situation ontology's own `Concept → Relationship → Intent` chain carries it
//! up to the warning posture — never, by construction, to engagement.
//!
//! Source: Steinberg & Bowman (2008) Revisions to the JDL Data Fusion Model;
//! Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking
//! and Navigation Ch. 7.

use pr4xis::category::{Category, Functor};

use crate::applied::tracking::multi_target::ontology::{MultiTargetCategory, MultiTargetConcept};
use crate::social::military::situation::ontology::{SituationCategory, SituationConcept};

/// Maps every track-lifecycle state to the single JDL identified-entity concept.
pub struct MultiTargetToSituation;

impl Functor for MultiTargetToSituation {
    type Source = MultiTargetCategory;
    type Target = SituationCategory;

    fn map_object(_track_state: &MultiTargetConcept) -> SituationConcept {
        // Every track — tentative, confirmed, coasting, or deleted — is an
        // identified entity from the situation-assessment view (JDL Level 1
        // output feeding Level 2, Steinberg & Bowman 2008).
        SituationConcept::Concept
    }

    fn map_morphism(
        _m: &<MultiTargetCategory as Category>::Morphism,
    ) -> <SituationCategory as Category>::Morphism {
        // Both endpoints map to `Concept`; the only `Concept → Concept` morphism
        // in the situation category is the identity. The constant functor sends
        // every lifecycle transition (and every identity) to `id_Concept`, which
        // is identity- and composition-preserving by construction: lifecycle
        // edges carry custom, non-transitive kinds, so no non-identity source
        // composite exists for the composition law to constrain.
        SituationCategory::identity(&SituationConcept::Concept)
    }
}
pr4xis::register_functor!(
    MultiTargetToSituation,
    "Steinberg & Bowman (2008) Revisions to the JDL Data Fusion Model; Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation Ch. 7"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<MultiTargetToSituation>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn every_track_state_is_an_identified_entity() {
        for state in MultiTargetConcept::variants() {
            assert_eq!(
                MultiTargetToSituation::map_object(&state),
                SituationConcept::Concept,
                "{state:?} should map to the identified-entity concept",
            );
        }
    }

    /// The two seams compose: a track (any lifecycle state) → identified entity
    /// → the `Identify` escalation posture. This witnesses the full
    /// tracking → situation → compliance decision chain the substrate documents.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn track_grounds_identify_posture_end_to_end() {
        use crate::social::compliance::escalation::EscalationLevel;
        use crate::social::military::situation::compliance_functor::SituationToCompliance;

        for state in MultiTargetConcept::variants() {
            let entity = MultiTargetToSituation::map_object(&state);
            let posture = SituationToCompliance::map_object(&entity);
            assert_eq!(
                posture,
                EscalationLevel::Identify,
                "a {state:?} track should ground the Identify escalation posture",
            );
        }
    }
}

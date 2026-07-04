//! Functor: Situation → Compliance (JDL assessment → escalation-of-force ladder).
//!
//! This closes the "threat → engagement decision" seam. Each element of a JDL
//! Level-2 situation assessment grounds a rung of the rules-of-engagement
//! escalation ladder: an identified entity grounds `Identify`, an assessed
//! inter-entity relationship grounds `Classify`, and inferred intent grounds
//! `Warn` — the first escalation-of-force step. Environmental context is the
//! baseline `Observe` posture.
//!
//! The object map is **monotone**: Environment ≤ Concept ≤ Relationship ≤ Intent
//! (the JDL causation order, Steinberg & Bowman 2008) maps to
//! Observe < Identify < Classify < Warn (the sequential escalation order,
//! Additional Protocol I Art. 57; NATO MC 362/1). So the assessment chain
//! `Concept → Relationship → Intent` is carried onto forward escalation
//! transitions — you cannot reach the intent-grounded warning posture without
//! passing through the entity- and relationship-grounded postures, exactly
//! mirroring the `SequentialEscalation` LOAC axiom. Because the source is a thin
//! poset (≤ 1 morphism per hom-set), the functor is faithful; its image is a
//! proper (not full) subcategory of the escalation ladder.
//!
//! # Safety
//!
//! The functor's image tops out at `Warn`, **strictly below `Engage`**. No
//! situation-assessment element maps to an engagement-authorising level:
//! inferred intent authorises a *warning*, never autonomous engagement, which
//! stays gated by the human-in-the-loop LOAC requirement (DoD Directive
//! 3000.09, 2023). This is enforced structurally, not by convention, by
//! [`NoAssessmentAloneAuthorizesEngagement`].
//!
//! Source: Steinberg & Bowman (2008); Endsley (1995); US DoD Directive
//! 3000.09 (2023); Additional Protocol I (1977) Art. 57; NATO MC 362/1.

use pr4xis::category::{Arrow, Category, FinitelyGenerated, Functor, FunctorKind};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use crate::social::compliance::escalation::EscalationLevel;
use crate::social::compliance::ontology::{ComplianceCategory, EscalationTransition};
use crate::social::military::situation::ontology::{
    SituationCategory, SituationConcept, SituationRelationKind,
};

/// Maps each JDL Level-2 situation-assessment element to the rules-of-engagement
/// escalation posture it grounds.
pub struct SituationToCompliance;

impl Functor for SituationToCompliance {
    type Source = SituationCategory;
    type Target = ComplianceCategory;

    /// A thin poset embeds faithfully into the escalation ladder; the image is a
    /// proper (not full) subcategory, so this is `Faithful`, not `FullyFaithful`
    /// (Mac Lane 1971 CWM Ch. I §4).
    const KIND: FunctorKind = FunctorKind::Faithful;

    fn map_object(obj: &SituationConcept) -> EscalationLevel {
        match obj {
            // Environmental context is the baseline observing posture.
            SituationConcept::Environment => EscalationLevel::Observe,
            // An identified entity (JDL Level 1 output) grounds Identify.
            SituationConcept::Concept => EscalationLevel::Identify,
            // An assessed inter-entity relationship grounds Classify.
            SituationConcept::Relationship => EscalationLevel::Classify,
            // Inferred intent grounds Warn — the first escalation-of-force step,
            // and deliberately BELOW every kinetic level (ShowForce..Engage):
            // assessment warrants a warning, not force.
            SituationConcept::Intent => EscalationLevel::Warn,
        }
    }

    fn map_morphism(m: &<SituationCategory as Category>::Morphism) -> EscalationTransition {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind() {
            // Preserve identities: F(id_A) = id_{F(A)} (functor identity law).
            SituationRelationKind::Identity => ComplianceCategory::identity(&from),
            // Every other declared morphism is a Causation edge; under the
            // monotone object map it becomes a forward escalation transition,
            // which is a declared morphism in the Warshall closure of the ladder.
            _ => EscalationTransition { from, to },
        }
    }
}
pr4xis::register_functor!(
    SituationToCompliance,
    "Steinberg & Bowman (2008) Revisions to the JDL Data Fusion Model; US DoD Directive 3000.09 (2023); Additional Protocol I (1977) Art. 57; NATO MC 362/1 Rules of Engagement"
);

/// Axiom: no situation-assessment element alone authorises engagement.
///
/// The image of [`SituationToCompliance`] contains no escalation level at or
/// above `Engage`. Inferred hostile intent maps to `Warn` (an
/// escalation-of-force warning), never to an engagement-authorising posture:
/// autonomous engagement stays gated by the human-in-the-loop requirement
/// (DoD Directive 3000.09, 2023; Additional Protocol I Art. 57(2)). Verified by
/// mapping every situation concept and checking none reaches `Engage`.
pub struct NoAssessmentAloneAuthorizesEngagement;

impl Axiom for NoAssessmentAloneAuthorizesEngagement {
    fn verify(&self) -> Verdict {
        let none_reaches_engage = SituationConcept::variants()
            .iter()
            .all(|c| SituationToCompliance::map_object(c) != EscalationLevel::Engage);
        if none_reaches_engage {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NoAssessmentAloneAuthorizesEngagement",
        "no JDL situation-assessment element maps to an engagement-authorising escalation level — assessment warrants at most a warning; engagement stays human-in-the-loop",
        "US DoD Directive 3000.09 (2023); Additional Protocol I (1977) Art. 57(2)"
    );
}
pr4xis::register_axiom!(
    NoAssessmentAloneAuthorizesEngagement,
    "US DoD Directive 3000.09 (2023); Additional Protocol I (1977) Art. 57(2)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::{FunctorFaithfulLaw, assert_functor_laws};

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<SituationToCompliance>();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_is_faithful() {
        assert!(
            FunctorFaithfulLaw::<SituationToCompliance>::new()
                .verify()
                .is_ok(),
            "SituationToCompliance should be faithful (thin source poset)"
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn assessment_chain_maps_to_declared_escalation_transitions() {
        // Every situation morphism must map to a morphism the compliance
        // category actually declares (the functor-composition law needs this,
        // but assert it directly for clarity too).
        let transitions = ComplianceCategory::morphisms();
        for m in SituationCategory::morphisms() {
            let image = SituationToCompliance::map_morphism(&m);
            assert!(
                transitions.contains(&image),
                "{:?} → {:?} maps to an undeclared escalation transition {:?}",
                m.source(),
                m.target(),
                image,
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn intent_grounds_warning_below_engagement() {
        // Inferred intent authorises a warning, never engagement.
        assert_eq!(
            SituationToCompliance::map_object(&SituationConcept::Intent),
            EscalationLevel::Warn,
        );
        assert_ne!(
            SituationToCompliance::map_object(&SituationConcept::Intent),
            EscalationLevel::Engage,
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn no_assessment_alone_authorizes_engagement_holds() {
        assert!(NoAssessmentAloneAuthorizesEngagement.verify().is_ok());
    }
}

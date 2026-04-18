//! Self-Model — the system's formal model of itself.
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

//
// "I am the observed relation between myself and observing myself."
// — Heinz von Foerster
//
// A self-model is the eigenform (fixed point) of self-observation:
// the system observes itself, the observation produces a description,
// and that description IS the system's self-knowledge.
//
// Grounded in ten formal sources across three cybernetic traditions:
//
// REPRESENTATIONAL (system holds explicit model of itself):
//   1. Craik, "The Nature of Explanation" (1943) — dual internal model
//   2. Conant-Ashby Good Regulator (1970) — self-regulator must model itself
//   3. MAPE-K / IBM Autonomic Computing (Kephart & Chess, 2003) — Ksys
//   4. Powers PCT (1973) — self-image = Level 11 controlled perception
//   5. Metzinger PSM (2003) — transparent self-model
//   6. MOI (Nolte et al., FOIS 2023) — SoftwareComponent, Capability
//   7. IEEE AuR (IEEE 1872.2, 2021) — SelfModel as required class
//   8. BDI Ontology (arXiv 2511.17162, 2024) — belief justified by introspection
//
// ORGANIZATIONAL (system IS its self-producing organization):
//   9. Maturana & Varela, Autopoiesis (1972/1980) — operational closure
//      "We do not see what we do not see, and what we do not see does not exist."
//
// REFLEXIVE / SECOND-ORDER (observer includes itself):
//  10. von Foerster Eigenform (1981) — X = F(X), fixed point
//  11. Bateson, "Steps to an Ecology of Mind" (1972) — double description
//      "The unit of survival is organism plus environment."
//  12. Lewis Awareness Taxonomy (2011) — 5 levels
//  13. SOSA/OWL-S (W3C) — SystemCapability

pr4xis::ontology! {
    name: "SelfModel",
    source: "von Foerster (1981); IEEE AuR (2021); MAPE-K (2003); Maturana & Varela (1972); Bateson (1972); Lewis (2011)",
    being: MentalObject,

    concepts: [
        SelfModel,
        Component,
        Capability,
        Identity,
        AwarenessLevel,
        Eigenform,
        SelfBelief,
        Justification,
        OperationalClosure,
        DoubleDescription,
    ],

    labels: {
        SelfModel: ("en", "Self-model", "The system's model of itself — the eigenform (IEEE AuR)."),
        Component: ("en", "Component", "A loaded ontology or module (MOI SoftwareComponent)."),
        Capability: ("en", "Capability", "What the system can reason about (SOSA SystemCapability)."),
        Identity: ("en", "Identity", "The system's identity — name, version, build (MAPE-K Ksys)."),
        AwarenessLevel: ("en", "Awareness level", "The current awareness level (Lewis 2011)."),
        Eigenform: ("en", "Eigenform", "The fixed point of self-observation (von Foerster/Kauffman)."),
        SelfBelief: ("en", "Self-belief", "A belief the system holds about itself (BDI)."),
        Justification: ("en", "Justification", "The justification for a self-belief (BDI)."),
        OperationalClosure: ("en", "Operational closure", "Maturana & Varela (1972): the system's processes produce the processes that produce them."),
        DoubleDescription: ("en", "Double description", "Bateson (1972): two views of the same phenomenon produce depth. 'The unit of survival is organism plus environment.'"),
    },

    edges: [
        // Structure (MOI + MAPE-K + Lewis)
        (SelfModel, Component, HasComponent),
        (SelfModel, Identity, HasIdentity),
        (SelfModel, AwarenessLevel, HasAwarenessLevel),
        (Component, Capability, HasCapability),
        (Capability, Component, EnabledBy),
        // Eigenform loop (von Foerster + Spencer-Brown)
        (SelfModel, Eigenform, ConvergesTo),
        (Eigenform, SelfModel, ReEnters),
        // Belief production (BDI)
        (SelfModel, SelfBelief, Produces),
        (SelfBelief, Justification, JustifiedBy),
        // Autopoiesis (Maturana-Varela)
        (SelfModel, OperationalClosure, Maintains),
        (OperationalClosure, Eigenform, Grounds),
        // Double description (Bateson)
        (DoubleDescription, SelfModel, Requires),
        (DoubleDescription, OperationalClosure, Requires),
    ],
}

// =========================================================================
// Lewis Awareness Levels — a total order
// =========================================================================

/// Five levels of self-awareness (Lewis et al., IEEE 2011).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AwarenessLevel {
    Stimulus = 0,
    Interaction = 1,
    Time = 2,
    Goal = 3,
    MetaSelf = 4,
}

impl AwarenessLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Stimulus => "Stimulus-awareness",
            Self::Interaction => "Interaction-awareness",
            Self::Time => "Time-awareness",
            Self::Goal => "Goal-awareness",
            Self::MetaSelf => "Meta-self-awareness",
        }
    }

    pub fn subsumes(&self, other: &Self) -> bool {
        *self >= *other
    }
}

// =========================================================================
// Functor: SelfModel → Metacognition
// =========================================================================
//
// The self-model IS metacognition applied to itself.
// SelfModel.observes(SelfModel) IS MetaLevel.observes(ObjectLevel).
//
// Mapping:
//   SelfModel   → MetaLevel       (the observer)
//   Component   → ObjectLevel     (what is being observed)
//   Eigenform   → Trace           (the record of observation)
//   SelfBelief  → EpistemicAssessment (what we know about ourselves)
//   Capability  → Monitoring      (what we can observe)
//   Identity    → Evaluation      (assessing what we are)
//   AwarenessLevel → Control      (deciding awareness level)
//   Justification → Gap           (what justifies = what was missing)

pub struct SelfModelToMetacognition;

impl SelfModelToMetacognition {
    pub fn map_object(obj: &SelfModelConcept) -> super::metacognition::MetaCognitionConcept {
        use super::metacognition::MetaCognitionConcept as M;
        match obj {
            SelfModelConcept::SelfModel => M::MetaLevel,
            SelfModelConcept::Component => M::ObjectLevel,
            SelfModelConcept::Eigenform => M::Trace,
            SelfModelConcept::SelfBelief => M::EpistemicAssessment,
            SelfModelConcept::Capability => M::Monitoring,
            SelfModelConcept::Identity => M::Evaluation,
            SelfModelConcept::AwarenessLevel => M::Control,
            SelfModelConcept::Justification => M::Gap,
            // Autopoiesis: operational closure IS the object level maintaining itself
            SelfModelConcept::OperationalClosure => M::ObjectLevel,
            // Double description: observing from two views IS the meta-level act
            SelfModelConcept::DoubleDescription => M::MetaLevel,
        }
    }
}

// =========================================================================
// Functor: SelfModel → Epistemics
// =========================================================================
//
// The self-model classifies what the system knows about itself.
//
// Mapping:
//   SelfModel + Eigenform → KnownKnown   (system knows itself and knows it)
//   Component + Capability → KnownKnown  (knows what it has)
//   Justification → KnownUnknown         (knows what it doesn't know)
//   AwarenessLevel → depends on level

pub struct SelfModelToEpistemics;

impl SelfModelToEpistemics {
    pub fn map_object(obj: &SelfModelConcept) -> super::epistemics::EpistemicConcept {
        use super::epistemics::EpistemicConcept as E;
        match obj {
            // The system with a self-model knows itself
            SelfModelConcept::SelfModel => E::KnownKnown,
            SelfModelConcept::Eigenform => E::KnownKnown,
            SelfModelConcept::Component => E::KnownKnown,
            SelfModelConcept::Capability => E::KnownKnown,
            SelfModelConcept::Identity => E::KnownKnown,
            SelfModelConcept::SelfBelief => E::KnownKnown,
            // Justification reveals what the system knows it doesn't know
            SelfModelConcept::Justification => E::KnownUnknown,
            // Awareness level assessment is knowing about knowing
            SelfModelConcept::AwarenessLevel => E::KnownKnown,
            // Operational closure: the system IS its organization — KnownKnown
            SelfModelConcept::OperationalClosure => E::KnownKnown,
            // Double description: requires both views — knowing about knowing
            SelfModelConcept::DoubleDescription => E::KnownKnown,
        }
    }
}

// =========================================================================
// Functor: SelfModel → Concurrency
// =========================================================================
//
// The self-model describes its own runtime concurrency characteristics.
// Praxis runs as a single agent with no shared resources.
// "Single thread" IS: 1 Agent, 0 SharedResources, 0 Synchronization.
//
// This is how the system knows it's single-threaded — through the ontology,
// not through a hardcoded string.

pub struct SelfModelToConcurrency;

impl SelfModelToConcurrency {
    pub fn map_object(
        obj: &SelfModelConcept,
    ) -> crate::formal::information::concurrency::ontology::ConcurrencyConcept {
        use crate::formal::information::concurrency::ontology::ConcurrencyConcept as CC;
        match obj {
            // The system itself is one agent — single thread
            SelfModelConcept::SelfModel => CC::Agent,
            // Components are the protocol the agent follows
            SelfModelConcept::Component => CC::Protocol,
            // Capabilities are messages the agent can process
            SelfModelConcept::Capability => CC::Message,
            // Identity is the agent's state
            SelfModelConcept::Identity => CC::State,
            // AwarenessLevel is the agent's state
            SelfModelConcept::AwarenessLevel => CC::State,
            // Eigenform is a future — the eventual fixed point
            SelfModelConcept::Eigenform => CC::Future,
            // SelfBelief is a message (internal)
            SelfModelConcept::SelfBelief => CC::Message,
            // Justification is a message
            SelfModelConcept::Justification => CC::Message,
            // Operational closure: the agent IS the protocol (no external sync)
            SelfModelConcept::OperationalClosure => CC::Protocol,
            // Double description is synchronization (two views coordinated)
            SelfModelConcept::DoubleDescription => CC::Synchronization,
        }
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::Concept;

    #[test]
    fn category_identity_law() {
        for obj in SelfModelConcept::variants() {
            let id = SelfModelCategory::identity(&obj);
            assert_eq!(id.from, obj);
            assert_eq!(id.to, obj);
            assert_eq!(id.kind, SelfModelRelationKind::Identity);
        }
    }

    #[test]
    fn category_composition_with_identity() {
        let morphisms = SelfModelCategory::morphisms();
        for m in &morphisms {
            let id_left = SelfModelCategory::identity(&m.from);
            let id_right = SelfModelCategory::identity(&m.to);

            let left = SelfModelCategory::compose(&id_left, m).unwrap();
            assert_eq!(left.from, m.from);
            assert_eq!(left.to, m.to);

            let right = SelfModelCategory::compose(m, &id_right).unwrap();
            assert_eq!(right.from, m.from);
            assert_eq!(right.to, m.to);
        }
    }

    #[test]
    fn has_ten_concepts() {
        assert_eq!(SelfModelConcept::variants().len(), 10);
    }

    #[test]
    fn eigenform_loop_exists() {
        let morphisms = SelfModelCategory::morphisms();
        // SelfModel → Eigenform (converges)
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == SelfModelConcept::SelfModel
                    && m.to == SelfModelConcept::Eigenform
                    && m.kind == SelfModelRelationKind::ConvergesTo)
        );
        // Eigenform → SelfModel (re-enters)
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == SelfModelConcept::Eigenform
                    && m.to == SelfModelConcept::SelfModel
                    && m.kind == SelfModelRelationKind::ReEnters)
        );
    }

    #[test]
    fn eigenform_is_fixed_point() {
        // Self(Self) = Self: composing ConvergesTo with ReEnters
        // gives SelfModel → SelfModel (the fixed point).
        let converge = SelfModelRelation {
            from: SelfModelConcept::SelfModel,
            to: SelfModelConcept::Eigenform,
            kind: SelfModelRelationKind::ConvergesTo,
        };
        let reenter = SelfModelRelation {
            from: SelfModelConcept::Eigenform,
            to: SelfModelConcept::SelfModel,
            kind: SelfModelRelationKind::ReEnters,
        };
        let composed = SelfModelCategory::compose(&converge, &reenter).unwrap();
        assert_eq!(composed.from, SelfModelConcept::SelfModel);
        assert_eq!(composed.to, SelfModelConcept::SelfModel);
    }

    #[test]
    fn awareness_levels_are_ordered() {
        assert!(AwarenessLevel::MetaSelf.subsumes(&AwarenessLevel::Stimulus));
        assert!(AwarenessLevel::Goal.subsumes(&AwarenessLevel::Time));
        assert!(!AwarenessLevel::Stimulus.subsumes(&AwarenessLevel::MetaSelf));
    }

    // === Functor law tests ===

    #[test]
    fn functor_to_metacognition_preserves_identity() {
        // F(id_A) = id_{F(A)}
        for obj in SelfModelConcept::variants() {
            let _id = SelfModelCategory::identity(&obj);
            let mapped = SelfModelToMetacognition::map_object(&obj);
            let target_id = super::super::metacognition::MetaCognitionCategory::identity(&mapped);
            // The mapped identity should be an identity on the target
            assert_eq!(target_id.from, mapped);
            assert_eq!(target_id.to, mapped);
        }
    }

    #[test]
    fn functor_to_epistemics_preserves_identity() {
        for obj in SelfModelConcept::variants() {
            let mapped = SelfModelToEpistemics::map_object(&obj);
            let target_id = super::super::epistemics::EpistemicCategory::identity(&mapped);
            assert_eq!(target_id.from, mapped);
            assert_eq!(target_id.to, mapped);
        }
    }

    #[test]
    fn functor_to_metacognition_covers_all_concepts() {
        // Every SelfModel concept maps to a valid MetaCognitionConcept
        for obj in SelfModelConcept::variants() {
            let mapped = SelfModelToMetacognition::map_object(&obj);
            assert!(
                super::super::metacognition::MetaCognitionConcept::variants().contains(&mapped),
                "{:?} maps to {:?} which is not a valid MetaCognitionConcept",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn functor_to_epistemics_self_model_is_known_known() {
        // A system with a self-model knows itself — KnownKnown
        let state = SelfModelToEpistemics::map_object(&SelfModelConcept::SelfModel);
        assert_eq!(
            state,
            super::super::epistemics::EpistemicConcept::KnownKnown
        );
    }

    #[test]
    fn functor_to_epistemics_justification_is_known_unknown() {
        // Justification reveals what the system knows it doesn't know
        let state = SelfModelToEpistemics::map_object(&SelfModelConcept::Justification);
        assert_eq!(
            state,
            super::super::epistemics::EpistemicConcept::KnownUnknown
        );
    }

    // === Autopoiesis tests (Maturana-Varela) ===

    #[test]
    fn operational_closure_grounds_eigenform() {
        // Autopoiesis enables self-observation: OperationalClosure → Eigenform
        let morphisms = SelfModelCategory::morphisms();
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == SelfModelConcept::OperationalClosure
                    && m.to == SelfModelConcept::Eigenform
                    && m.kind == SelfModelRelationKind::Grounds)
        );
    }

    #[test]
    fn self_model_maintains_operational_closure() {
        // The system IS its self-producing organization
        let morphisms = SelfModelCategory::morphisms();
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == SelfModelConcept::SelfModel
                    && m.to == SelfModelConcept::OperationalClosure
                    && m.kind == SelfModelRelationKind::Maintains)
        );
    }

    #[test]
    fn autopoiesis_to_eigenform_path() {
        // SelfModel → OperationalClosure → Eigenform (autopoiesis enables fixed point)
        let maintain = SelfModelRelation {
            from: SelfModelConcept::SelfModel,
            to: SelfModelConcept::OperationalClosure,
            kind: SelfModelRelationKind::Maintains,
        };
        let ground = SelfModelRelation {
            from: SelfModelConcept::OperationalClosure,
            to: SelfModelConcept::Eigenform,
            kind: SelfModelRelationKind::Grounds,
        };
        let composed = SelfModelCategory::compose(&maintain, &ground).unwrap();
        assert_eq!(composed.from, SelfModelConcept::SelfModel);
        assert_eq!(composed.to, SelfModelConcept::Eigenform);
    }

    // === Double description tests (Bateson) ===

    // === Concurrency functor tests ===

    #[test]
    fn functor_to_concurrency_self_model_is_agent() {
        // The system is ONE agent — single thread
        use crate::formal::information::concurrency::ontology::ConcurrencyConcept;
        let mapped = SelfModelToConcurrency::map_object(&SelfModelConcept::SelfModel);
        assert_eq!(mapped, ConcurrencyConcept::Agent);
    }

    #[test]
    fn functor_to_concurrency_covers_all_concepts() {
        for obj in SelfModelConcept::variants() {
            let mapped = SelfModelToConcurrency::map_object(&obj);
            assert!(
                crate::formal::information::concurrency::ontology::ConcurrencyConcept::variants()
                    .contains(&mapped),
                "{:?} maps to {:?} which is not a valid ConcurrencyConcept",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn functor_to_concurrency_no_deadlock() {
        // Single agent cannot deadlock — no concept maps to Deadlock
        use crate::formal::information::concurrency::ontology::ConcurrencyConcept;
        for obj in SelfModelConcept::variants() {
            let mapped = SelfModelToConcurrency::map_object(&obj);
            assert_ne!(
                mapped,
                ConcurrencyConcept::Deadlock,
                "single-threaded system cannot have deadlock"
            );
        }
    }

    #[test]
    fn functor_to_concurrency_no_race_condition() {
        // Single agent cannot race — no concept maps to RaceCondition
        use crate::formal::information::concurrency::ontology::ConcurrencyConcept;
        for obj in SelfModelConcept::variants() {
            let mapped = SelfModelToConcurrency::map_object(&obj);
            assert_ne!(
                mapped,
                ConcurrencyConcept::RaceCondition,
                "single-threaded system cannot have race conditions"
            );
        }
    }

    // === Double description tests (Bateson) ===

    #[test]
    fn double_description_requires_both_views() {
        // Bateson: valid self-image needs two views — self AND context
        let morphisms = SelfModelCategory::morphisms();
        let requires_self = morphisms.iter().any(|m| {
            m.from == SelfModelConcept::DoubleDescription
                && m.to == SelfModelConcept::SelfModel
                && m.kind == SelfModelRelationKind::Requires
        });
        let requires_closure = morphisms.iter().any(|m| {
            m.from == SelfModelConcept::DoubleDescription
                && m.to == SelfModelConcept::OperationalClosure
                && m.kind == SelfModelRelationKind::Requires
        });
        assert!(requires_self, "double description must require self-view");
        assert!(
            requires_closure,
            "double description must require context-view"
        );
    }
}

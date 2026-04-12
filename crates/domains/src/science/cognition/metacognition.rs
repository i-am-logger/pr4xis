use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

// Metacognition — the system observing its own reasoning.
//
// Second-order cybernetics formalized: the observer IS part of the system.
// The meta-level monitors, evaluates, and controls the object level.
//
// When the grammar fails to parse, the meta-level:
// 1. Detects the failure (monitoring)
// 2. Diagnoses the cause (evaluation — which type reduction failed?)
// 3. Decides what to do (control — ask for clarification? attempt repair?)
//
// References:
// - von Foerster, Observing Systems (1981)
// - Glanville, Second Order Cybernetics (docs/papers/)
// - Olivares-Alarcos et al., MOI: Meta-Ontology for Introspection (2023)

/// Metacognitive concepts — the bi-level architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetaConcept {
    /// The actual reasoning happening (grammar, semantics, queries).
    ObjectLevel,
    /// The observer of the reasoning (monitors, evaluates, controls).
    MetaLevel,
    /// What the meta-level watches: the trace of object-level steps.
    Monitoring,
    /// Assessing whether the object level succeeded or failed.
    Evaluation,
    /// Deciding what to do based on evaluation (continue, repair, ask).
    Control,
    /// The record of what happened at the object level (the Engine Trace).
    Trace,
    /// A detected gap — something the object level couldn't handle.
    Gap,
    /// An attempt to fix a gap without external help.
    Repair,
    /// A request for external help (asking the user to clarify).
    Clarification,
    /// The epistemic state assessment (from epistemics.rs).
    EpistemicAssessment,
}

impl Entity for MetaConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::ObjectLevel,
            Self::MetaLevel,
            Self::Monitoring,
            Self::Evaluation,
            Self::Control,
            Self::Trace,
            Self::Gap,
            Self::Repair,
            Self::Clarification,
            Self::EpistemicAssessment,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetaRelation {
    pub from: MetaConcept,
    pub to: MetaConcept,
    pub kind: MetaRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetaRelationKind {
    Identity,
    /// MetaLevel observes ObjectLevel.
    Observes,
    /// Monitoring produces Trace.
    Records,
    /// Evaluation assesses Trace.
    Assesses,
    /// Evaluation detects Gap.
    Detects,
    /// Control decides action (Repair or Clarification).
    Decides,
    /// Gap triggers Repair or Clarification.
    Triggers,
    /// EpistemicAssessment classifies the state of knowledge.
    Classifies,
    Composed,
}

impl Relationship for MetaRelation {
    type Object = MetaConcept;
    fn source(&self) -> MetaConcept {
        self.from
    }
    fn target(&self) -> MetaConcept {
        self.to
    }
}

pub struct MetaCognitionCategory;

impl Category for MetaCognitionCategory {
    type Object = MetaConcept;
    type Morphism = MetaRelation;

    fn identity(obj: &MetaConcept) -> MetaRelation {
        MetaRelation {
            from: *obj,
            to: *obj,
            kind: MetaRelationKind::Identity,
        }
    }

    fn compose(f: &MetaRelation, g: &MetaRelation) -> Option<MetaRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == MetaRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == MetaRelationKind::Identity {
            return Some(f.clone());
        }
        Some(MetaRelation {
            from: f.from,
            to: g.to,
            kind: MetaRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<MetaRelation> {
        use MetaConcept::*;
        use MetaRelationKind::*;

        let mut m = Vec::new();

        for c in MetaConcept::variants() {
            m.push(MetaRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // MetaLevel observes ObjectLevel
        m.push(MetaRelation {
            from: MetaLevel,
            to: ObjectLevel,
            kind: Observes,
        });
        // Monitoring records Trace
        m.push(MetaRelation {
            from: Monitoring,
            to: Trace,
            kind: Records,
        });
        // Evaluation assesses Trace
        m.push(MetaRelation {
            from: Evaluation,
            to: Trace,
            kind: Assesses,
        });
        // Evaluation detects Gap
        m.push(MetaRelation {
            from: Evaluation,
            to: Gap,
            kind: Detects,
        });
        // Control decides Repair or Clarification
        m.push(MetaRelation {
            from: Control,
            to: Repair,
            kind: Decides,
        });
        m.push(MetaRelation {
            from: Control,
            to: Clarification,
            kind: Decides,
        });
        // Gap triggers Repair
        m.push(MetaRelation {
            from: Gap,
            to: Repair,
            kind: Triggers,
        });
        // Gap triggers Clarification
        m.push(MetaRelation {
            from: Gap,
            to: Clarification,
            kind: Triggers,
        });
        // EpistemicAssessment classifies Gap
        m.push(MetaRelation {
            from: EpistemicAssessment,
            to: Gap,
            kind: Classifies,
        });

        // The second-order loop: MetaLevel → ... → everything
        m.push(MetaRelation {
            from: MetaLevel,
            to: EpistemicAssessment,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: MetaLevel,
            to: Monitoring,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: MetaLevel,
            to: Trace,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: MetaLevel,
            to: Evaluation,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: MetaLevel,
            to: Gap,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: MetaLevel,
            to: Control,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: MetaLevel,
            to: Repair,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: MetaLevel,
            to: Clarification,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: Monitoring,
            to: Evaluation,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: Monitoring,
            to: Gap,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: Evaluation,
            to: Repair,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: Evaluation,
            to: Clarification,
            kind: Composed,
        });
        m.push(MetaRelation {
            from: Evaluation,
            to: Control,
            kind: Composed,
        });

        // Self-composed
        for c in MetaConcept::variants() {
            m.push(MetaRelation {
                from: c,
                to: c,
                kind: Composed,
            });
        }

        m
    }
}

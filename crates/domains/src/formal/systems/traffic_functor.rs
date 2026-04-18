use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;
use pr4xis::category::{Category, Functor};

use super::ontology::{SystemConcept, SystemRelation, SystemRelationKind, SystemsCategory};

/// Traffic system concepts — the objects in the traffic domain
/// that map to systems thinking concepts.
///
/// This is a simplified view of the traffic domain focused on
/// its systemic properties, not its full detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrafficSystemElement {
    /// A traffic signal (component).
    Signal,
    /// The conflict between directions (interaction).
    DirectionConflict,
    /// The current intersection state (all signals together).
    IntersectionState,
    /// Advancing a signal (transition).
    SignalAdvance,
    /// Safety check — no conflicting greens (constraint).
    SafetyRule,
    /// Congestion sensing (feedback).
    CongestionFeedback,
    /// Green wave timing (homeostasis).
    GreenWaveTiming,
    /// Traffic flow rate (emergence).
    FlowRate,
    /// Intersection perimeter (boundary).
    IntersectionBoundary,
    /// Signal controller hardware (controller).
    SignalController,
}

impl Entity for TrafficSystemElement {
    fn variants() -> Vec<Self> {
        vec![
            Self::Signal,
            Self::DirectionConflict,
            Self::IntersectionState,
            Self::SignalAdvance,
            Self::SafetyRule,
            Self::CongestionFeedback,
            Self::GreenWaveTiming,
            Self::FlowRate,
            Self::IntersectionBoundary,
            Self::SignalController,
        ]
    }
}

/// Relationships between traffic system elements.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrafficSystemRelation {
    pub from: TrafficSystemElement,
    pub to: TrafficSystemElement,
    pub kind: TrafficRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrafficRelationKind {
    Identity,
    /// Signals compose the intersection state.
    ComposesInto,
    /// Signal advance changes intersection state.
    Changes,
    /// Safety rule governs signal advance.
    Governs,
    /// Congestion feeds back to timing.
    FeedsBack,
    /// Green wave stabilizes flow.
    Stabilizes,
    /// Flow rate emerges from direction conflicts.
    ArisesFrom,
    /// Controller regulates via safety rules.
    Regulates,
    /// Boundary contains signals.
    Separates,
    Composed,
}

impl Relationship for TrafficSystemRelation {
    type Object = TrafficSystemElement;
    fn source(&self) -> TrafficSystemElement {
        self.from
    }
    fn target(&self) -> TrafficSystemElement {
        self.to
    }
}

/// The traffic system category.
pub struct TrafficSystemCategory;

impl Category for TrafficSystemCategory {
    type Object = TrafficSystemElement;
    type Morphism = TrafficSystemRelation;

    fn identity(obj: &TrafficSystemElement) -> TrafficSystemRelation {
        TrafficSystemRelation {
            from: *obj,
            to: *obj,
            kind: TrafficRelationKind::Identity,
        }
    }

    fn compose(
        f: &TrafficSystemRelation,
        g: &TrafficSystemRelation,
    ) -> Option<TrafficSystemRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == TrafficRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == TrafficRelationKind::Identity {
            return Some(f.clone());
        }
        Some(TrafficSystemRelation {
            from: f.from,
            to: g.to,
            kind: TrafficRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<TrafficSystemRelation> {
        use TrafficRelationKind::*;
        use TrafficSystemElement::*;

        let mut m = Vec::new();

        for c in TrafficSystemElement::variants() {
            m.push(TrafficSystemRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // Signal composes into IntersectionState
        m.push(TrafficSystemRelation {
            from: Signal,
            to: IntersectionState,
            kind: ComposesInto,
        });
        // DirectionConflict composes into IntersectionState
        m.push(TrafficSystemRelation {
            from: DirectionConflict,
            to: IntersectionState,
            kind: ComposesInto,
        });
        // SignalAdvance changes IntersectionState
        m.push(TrafficSystemRelation {
            from: SignalAdvance,
            to: IntersectionState,
            kind: Changes,
        });
        // SafetyRule governs SignalAdvance
        m.push(TrafficSystemRelation {
            from: SafetyRule,
            to: SignalAdvance,
            kind: Governs,
        });
        // IntersectionState feeds back to CongestionFeedback
        m.push(TrafficSystemRelation {
            from: IntersectionState,
            to: CongestionFeedback,
            kind: FeedsBack,
        });
        // CongestionFeedback feeds back to SignalAdvance
        m.push(TrafficSystemRelation {
            from: CongestionFeedback,
            to: SignalAdvance,
            kind: FeedsBack,
        });
        // GreenWaveTiming stabilizes IntersectionState
        m.push(TrafficSystemRelation {
            from: GreenWaveTiming,
            to: IntersectionState,
            kind: Stabilizes,
        });
        // CongestionFeedback stabilizes via GreenWaveTiming
        m.push(TrafficSystemRelation {
            from: CongestionFeedback,
            to: GreenWaveTiming,
            kind: Stabilizes,
        });
        // FlowRate emerges from DirectionConflict
        m.push(TrafficSystemRelation {
            from: DirectionConflict,
            to: FlowRate,
            kind: ArisesFrom,
        });
        // SignalController regulates via SafetyRule
        m.push(TrafficSystemRelation {
            from: SignalController,
            to: SafetyRule,
            kind: Regulates,
        });
        // IntersectionBoundary separates Signal
        m.push(TrafficSystemRelation {
            from: IntersectionBoundary,
            to: Signal,
            kind: Separates,
        });
        // SignalAdvance modifies Signal (transition changes components)
        m.push(TrafficSystemRelation {
            from: SignalAdvance,
            to: Signal,
            kind: Changes,
        });
        // CongestionFeedback informs SignalController (feedback to regulator)
        m.push(TrafficSystemRelation {
            from: CongestionFeedback,
            to: SignalController,
            kind: FeedsBack,
        });

        // Transitive (mirrors SystemsCategory pattern)
        m.push(TrafficSystemRelation {
            from: IntersectionState,
            to: SignalAdvance,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: IntersectionState,
            to: GreenWaveTiming,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: IntersectionState,
            to: SignalController,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: IntersectionState,
            to: SafetyRule,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: IntersectionState,
            to: Signal,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: IntersectionState,
            to: DirectionConflict,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: IntersectionState,
            to: FlowRate,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: IntersectionState,
            to: IntersectionBoundary,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: CongestionFeedback,
            to: IntersectionState,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: SignalController,
            to: SignalAdvance,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: SignalController,
            to: IntersectionState,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: SignalController,
            to: Signal,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: SafetyRule,
            to: IntersectionState,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: SafetyRule,
            to: Signal,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: Signal,
            to: CongestionFeedback,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: DirectionConflict,
            to: CongestionFeedback,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: DirectionConflict,
            to: IntersectionState,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: IntersectionBoundary,
            to: IntersectionState,
            kind: Composed,
        });
        m.push(TrafficSystemRelation {
            from: DirectionConflict,
            to: FlowRate,
            kind: Composed,
        });

        m
    }
}

/// Functor from Traffic system to Systems Thinking.
///
/// This is THE PROOF that traffic is a system.
/// If the functor laws hold (identity preservation + composition preservation),
/// then traffic's structure IS systems thinking structure — not by analogy,
/// but by mathematical proof.
pub struct TrafficToSystems;

impl Functor for TrafficToSystems {
    type Source = TrafficSystemCategory;
    type Target = SystemsCategory;

    fn map_object(obj: &TrafficSystemElement) -> SystemConcept {
        match obj {
            TrafficSystemElement::Signal => SystemConcept::Component,
            TrafficSystemElement::DirectionConflict => SystemConcept::Interaction,
            TrafficSystemElement::IntersectionState => SystemConcept::State,
            TrafficSystemElement::SignalAdvance => SystemConcept::Transition,
            TrafficSystemElement::SafetyRule => SystemConcept::Constraint,
            TrafficSystemElement::CongestionFeedback => SystemConcept::Feedback,
            TrafficSystemElement::GreenWaveTiming => SystemConcept::Homeostasis,
            TrafficSystemElement::FlowRate => SystemConcept::Emergence,
            TrafficSystemElement::IntersectionBoundary => SystemConcept::Boundary,
            TrafficSystemElement::SignalController => SystemConcept::Controller,
        }
    }

    fn map_morphism(m: &TrafficSystemRelation) -> SystemRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            TrafficRelationKind::Identity => SystemRelationKind::Identity,
            TrafficRelationKind::ComposesInto => SystemRelationKind::ComposesInto,
            TrafficRelationKind::Changes => SystemRelationKind::Changes,
            TrafficRelationKind::Governs => SystemRelationKind::Governs,
            TrafficRelationKind::FeedsBack => SystemRelationKind::FeedsBack,
            TrafficRelationKind::Stabilizes => SystemRelationKind::Stabilizes,
            TrafficRelationKind::ArisesFrom => SystemRelationKind::ArisesFrom,
            TrafficRelationKind::Regulates => SystemRelationKind::Regulates,
            TrafficRelationKind::Separates => SystemRelationKind::Separates,
            TrafficRelationKind::Composed => SystemRelationKind::Composed,
        };
        SystemRelation { from, to, kind }
    }
}
pr4xis::register_functor!(TrafficToSystems);

use pr4xis::category::Arrow;
use pr4xis::category::entity::Concept;
use pr4xis::category::{Category, Functor};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::ontology::{SystemCategory, SystemConcept, SystemRelation, SystemRelationKind};

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

impl Concept for TrafficSystemElement {
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

impl Arrow for TrafficSystemRelation {
    type Object = TrafficSystemElement;
    type Kind = TrafficRelationKind;
    fn source(&self) -> TrafficSystemElement {
        self.from
    }
    fn target(&self) -> TrafficSystemElement {
        self.to
    }
    fn kind(&self) -> TrafficRelationKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new(format!("{:?}-[{:?}]-{:?}", self.from, self.kind, self.to)),
            description: Label::new(format!(
                "{:?} -[{:?}]-> {:?}",
                self.from, self.kind, self.to
            )),
            citation: Citation::parse_static(
                "Webster (1928) Highway Traffic Analysis; Robertson (1969) TRANSYT; Meadows (2008) Thinking in Systems",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
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
        // Same-kind composition under `Composed` only (#166 partial
        // category): heterogeneous source kinds have no declared
        // composition rule, matching the macro-generated target's
        // behaviour. Two `Composed` edges compose into another
        // `Composed` edge when the result is itself a declared morphism.
        if f.kind == TrafficRelationKind::Composed && g.kind == TrafficRelationKind::Composed {
            let candidate = TrafficSystemRelation {
                from: f.from,
                to: g.to,
                kind: TrafficRelationKind::Composed,
            };
            if Self::morphisms().contains(&candidate) {
                return Some(candidate);
            }
        }
        None
    }

    fn morphisms() -> Vec<TrafficSystemRelation> {
        use TrafficRelationKind::*;
        use TrafficSystemElement::*;
        use std::collections::HashSet;

        // Direct kinded edges of the traffic system.
        let direct: Vec<(
            TrafficSystemElement,
            TrafficSystemElement,
            TrafficRelationKind,
        )> = vec![
            (Signal, IntersectionState, ComposesInto),
            (DirectionConflict, IntersectionState, ComposesInto),
            (SignalAdvance, IntersectionState, Changes),
            (SafetyRule, SignalAdvance, Governs),
            (IntersectionState, CongestionFeedback, FeedsBack),
            (CongestionFeedback, SignalAdvance, FeedsBack),
            (GreenWaveTiming, IntersectionState, Stabilizes),
            (CongestionFeedback, GreenWaveTiming, Stabilizes),
            (DirectionConflict, FlowRate, ArisesFrom),
            (SignalController, SafetyRule, Regulates),
            (IntersectionBoundary, Signal, Separates),
            (SignalAdvance, Signal, Changes),
            (CongestionFeedback, SignalController, FeedsBack),
        ];

        let mut m: Vec<TrafficSystemRelation> = Vec::new();
        for c in TrafficSystemElement::variants() {
            m.push(TrafficSystemRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }
        for &(f, t, k) in &direct {
            m.push(TrafficSystemRelation {
                from: f,
                to: t,
                kind: k,
            });
        }

        // Warshall (1962) transitive closure under the `Composed` umbrella
        // kind, so AssociativityLaw holds (Mac Lane CWM Ch. I §1).
        let edges: HashSet<(TrafficSystemElement, TrafficSystemElement)> =
            direct.iter().map(|&(f, t, _)| (f, t)).collect();
        let mut closure = edges.clone();
        loop {
            let mut added = false;
            let snap: Vec<_> = closure.iter().cloned().collect();
            for &(a, b) in &snap {
                for &(b2, c) in &snap {
                    if b == b2 && !closure.contains(&(a, c)) {
                        closure.insert((a, c));
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }
        for (f, t) in closure {
            m.push(TrafficSystemRelation {
                from: f,
                to: t,
                kind: Composed,
            });
        }
        // Composed self-loops for every variant
        for c in TrafficSystemElement::variants() {
            let r = TrafficSystemRelation {
                from: c,
                to: c,
                kind: Composed,
            };
            if !m.contains(&r) {
                m.push(r);
            }
        }

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
    type Target = SystemCategory;

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
            // Composed source morphisms cover transitive paths. They must
            // map to a non-Identity target kind, otherwise target's
            // identity-aware compose treats them as identities — breaking
            // FunctorCompositionLaw when source(F(m)) ≠ target(F(m)).
            // `Subsumption` is the canonical Relations-kind always emitted
            // by the macro (Smith 2005 OBO-RO).
            TrafficRelationKind::Composed => SystemRelationKind::Subsumption,
        };
        SystemRelation { from, to, kind }
    }
}
pr4xis::register_functor!(TrafficToSystems);

use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Track lifecycle states.
///
/// A track goes through a defined lifecycle. The ontology enforces
/// that transitions are valid — you can't skip states.
///
/// Source: Bar-Shalom et al. (2001), Chapter 7.
///         Blackman & Popoli (1999), *Design and Analysis of Modern Tracking Systems*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackState {
    /// New track, not yet confirmed. Awaiting M-of-N confirmation.
    Tentative,
    /// Confirmed: sufficient consecutive detections (M hits in N scans).
    Confirmed,
    /// Coasting: missed detections, propagating with predict only.
    Coasting,
    /// Deleted: too many consecutive misses. Terminal state.
    Deleted,
}

impl Entity for TrackState {
    fn variants() -> Vec<Self> {
        vec![
            Self::Tentative,
            Self::Confirmed,
            Self::Coasting,
            Self::Deleted,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrackTransition {
    pub from: TrackState,
    pub to: TrackState,
}

impl Relationship for TrackTransition {
    type Object = TrackState;
    fn source(&self) -> TrackState {
        self.from
    }
    fn target(&self) -> TrackState {
        self.to
    }
}

/// Track lifecycle category.
///
/// Deleted is an absorbing state — no transitions out.
pub struct TrackLifecycleCategory;

impl Category for TrackLifecycleCategory {
    type Object = TrackState;
    type Morphism = TrackTransition;

    fn identity(obj: &TrackState) -> TrackTransition {
        TrackTransition {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &TrackTransition, g: &TrackTransition) -> Option<TrackTransition> {
        if f.to != g.from {
            return None;
        }
        Some(TrackTransition {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<TrackTransition> {
        use TrackState::*;
        let mut m = Vec::new();
        for s in TrackState::variants() {
            m.push(Self::identity(&s));
        }
        // Valid transitions
        m.push(TrackTransition {
            from: Tentative,
            to: Confirmed,
        }); // M-of-N success
        m.push(TrackTransition {
            from: Tentative,
            to: Deleted,
        }); // failed confirmation
        m.push(TrackTransition {
            from: Confirmed,
            to: Coasting,
        }); // missed detection
        m.push(TrackTransition {
            from: Confirmed,
            to: Deleted,
        }); // lost
        m.push(TrackTransition {
            from: Coasting,
            to: Confirmed,
        }); // re-detection
        m.push(TrackTransition {
            from: Coasting,
            to: Deleted,
        }); // too many misses
        // Transitive
        m.push(TrackTransition {
            from: Tentative,
            to: Coasting,
        });
        m.push(TrackTransition {
            from: Tentative,
            to: Tentative,
        });
        m.push(TrackTransition {
            from: Confirmed,
            to: Confirmed,
        });
        m.push(TrackTransition {
            from: Coasting,
            to: Coasting,
        });
        m
    }
}

#[derive(Debug, Clone)]
pub struct TrackStateDescription;

impl Quality for TrackStateDescription {
    type Individual = TrackState;
    type Value = &'static str;

    fn get(&self, s: &TrackState) -> Option<&'static str> {
        Some(match s {
            TrackState::Tentative => "new track, awaiting M-of-N confirmation",
            TrackState::Confirmed => "confirmed, actively tracked",
            TrackState::Coasting => "missed detections, predict-only",
            TrackState::Deleted => "terminated, absorbing state",
        })
    }
}

/// Axiom: Deleted is an absorbing state — no transitions out.
pub struct DeletedIsAbsorbing;

impl Axiom for DeletedIsAbsorbing {
    fn description(&self) -> &str {
        "Deleted is absorbing: once deleted, a track cannot return to any other state"
    }
    fn holds(&self) -> bool {
        let morphisms = TrackLifecycleCategory::morphisms();
        // No morphism from Deleted to any non-Deleted state
        !morphisms
            .iter()
            .any(|m| m.from == TrackState::Deleted && m.to != TrackState::Deleted)
    }
}

/// Axiom: every track starts as Tentative.
pub struct TrackStartsTentative;

impl Axiom for TrackStartsTentative {
    fn description(&self) -> &str {
        "every track begins in Tentative state"
    }
    fn holds(&self) -> bool {
        // No transitions TO Tentative from Confirmed/Coasting/Deleted
        let morphisms = TrackLifecycleCategory::morphisms();
        !morphisms
            .iter()
            .any(|m| m.to == TrackState::Tentative && m.from != TrackState::Tentative)
    }
}

/// Axiom: Coasting can return to Confirmed (re-detection).
pub struct ReDetectionPossible;

impl Axiom for ReDetectionPossible {
    fn description(&self) -> &str {
        "coasting track can return to confirmed on re-detection"
    }
    fn holds(&self) -> bool {
        let morphisms = TrackLifecycleCategory::morphisms();
        morphisms
            .iter()
            .any(|m| m.from == TrackState::Coasting && m.to == TrackState::Confirmed)
    }
}

pub struct MultiTargetOntology;

impl Ontology for MultiTargetOntology {
    type Cat = TrackLifecycleCategory;
    type Qual = TrackStateDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(DeletedIsAbsorbing),
            Box::new(TrackStartsTentative),
            Box::new(ReDetectionPossible),
        ]
    }
}

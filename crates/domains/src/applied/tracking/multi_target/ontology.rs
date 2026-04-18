#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Category, Concept};
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Track lifecycle states.
///
/// A track goes through a defined lifecycle. The ontology enforces
/// that transitions are valid — you can't skip states.
///
/// Source: Bar-Shalom et al. (2001), Chapter 7.
///         Blackman & Popoli (1999), *Design and Analysis of Modern Tracking Systems*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
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

define_ontology! {
    /// Track lifecycle category.
    ///
    /// Deleted is an absorbing state — no transitions out.
    pub MultiTargetOntologyMeta for TrackLifecycleCategory {
        concepts: TrackState,
        relation: TrackTransition,
        kind: TrackTransitionKind,
        kinds: [
            /// M-of-N confirmation success.
            Confirm,
            /// Missed detection — begin coasting.
            Miss,
            /// Re-detection during coasting.
            ReDetect,
            /// Track deletion (failed confirmation or too many misses).
            Delete,
        ],
        edges: [
            // Tentative -> Confirmed (M-of-N success)
            (Tentative, Confirmed, Confirm),
            // Tentative -> Deleted (failed confirmation)
            (Tentative, Deleted, Delete),
            // Confirmed -> Coasting (missed detection)
            (Confirmed, Coasting, Miss),
            // Confirmed -> Deleted (lost)
            (Confirmed, Deleted, Delete),
            // Coasting -> Confirmed (re-detection)
            (Coasting, Confirmed, ReDetect),
            // Coasting -> Deleted (too many misses)
            (Coasting, Deleted, Delete),
        ],
        composed: [
            // Tentative -> Coasting (through Confirmed)
            (Tentative, Coasting),
        ],
        being: Process,
        source: "Bar-Shalom et al. (2001)",
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
pr4xis::register_axiom!(DeletedIsAbsorbing);

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
pr4xis::register_axiom!(TrackStartsTentative);

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
pr4xis::register_axiom!(ReDetectionPossible);

pub struct MultiTargetOntology;

impl Ontology for MultiTargetOntology {
    type Cat = TrackLifecycleCategory;
    type Qual = TrackStateDescription;

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(DeletedIsAbsorbing),
            Box::new(TrackStartsTentative),
            Box::new(ReDetectionPossible),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<TrackLifecycleCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        MultiTargetOntology::validate().unwrap();
    }
}

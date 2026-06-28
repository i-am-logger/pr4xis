//! Multi-target tracking — track-state lifecycle ontology.
//!
//! A track moves through a defined lifecycle (tentative →
//! confirmed → coasting → deleted) governed by sensor-detection M-of-N
//! confirmation rules. The ontology enforces transition legality:
//! deleted is an absorbing state; only tentative tracks are entered;
//! coasting tracks may re-confirm.
//!
//! # Literature
//!
//! - **Bar-Shalom, Li & Kirubarajan (2001)** *Estimation with
//!   Applications to Tracking and Navigation*, Ch. 7 — multi-target
//!   tracking, the M-of-N confirmation rule, track-state automaton.
//! - **Blackman & Popoli (1999)** *Design and Analysis of Modern
//!   Tracking Systems* (Artech House) — the canonical reference for
//!   track-life-cycle management, confirmation, deletion, and coasting.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "MultiTarget",
    source: "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation Ch. 7; Blackman & Popoli (1999) Design and Analysis of Modern Tracking Systems",

    concepts: [
        // Bar-Shalom (2001) Ch. 7 / Blackman & Popoli (1999) track states.
        Tentative,
        Confirmed,
        Coasting,
        Deleted,
    ],

    labels: {
        Tentative: ("en", "Tentative",
            "Bar-Shalom (2001) §7.5: a new track that has not yet met the M-of-N confirmation criterion."),
        Confirmed: ("en", "Confirmed",
            "Bar-Shalom (2001) §7.5: a track that has satisfied M-of-N — actively tracked, with measurement updates."),
        Coasting: ("en", "Coasting",
            "Blackman & Popoli (1999): a track that has missed detections; the filter propagates with predict-only."),
        Deleted: ("en", "Deleted",
            "Bar-Shalom (2001) §7.5: a track that has failed confirmation or accumulated too many misses — terminal."),
    },

    // The track-lifecycle transition graph (Bar-Shalom 2001 Fig. 7.6 /
    // Blackman & Popoli 1999 Ch. 6) expressed as kinded edges. Custom
    // morphism-kinds capture the semantic kind of each transition.
    edges: [
        // Tentative -> Confirmed (M-of-N success)
        (Tentative, Confirmed, Confirm),
        // Tentative -> Deleted (failed confirmation)
        (Tentative, Deleted, Delete),
        // Confirmed -> Coasting (missed detection)
        (Confirmed, Coasting, Miss),
        // Confirmed -> Deleted (lost track)
        (Confirmed, Deleted, Delete),
        // Coasting -> Confirmed (re-detection)
        (Coasting, Confirmed, ReDetect),
        // Coasting -> Deleted (too many misses)
        (Coasting, Deleted, Delete),
    ],

    opposes: [
        // Confirmed (informative track) vs Deleted (terminated).
        (Confirmed, Deleted),
        (Deleted, Confirmed),
    ],
}

/// Quality: a one-line semantic description of each track state.
#[derive(Debug, Clone)]
pub struct TrackStateDescription;

impl Quality for TrackStateDescription {
    type Individual = MultiTargetConcept;
    type Value = &'static str;

    fn get(&self, s: &MultiTargetConcept) -> Option<&'static str> {
        Some(match s {
            MultiTargetConcept::Tentative => "new track, awaiting M-of-N confirmation",
            MultiTargetConcept::Confirmed => "confirmed, actively tracked",
            MultiTargetConcept::Coasting => "missed detections, predict-only",
            MultiTargetConcept::Deleted => "terminated, absorbing state",
        })
    }
}

impl Ontology for MultiTargetOntology {
    type Cat = MultiTargetCategory;
    type Qual = TrackStateDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(DeletedIsAbsorbing));
        axioms.push(Box::new(TrackStartsTentative));
        axioms.push(Box::new(ReDetectionPossible));
        axioms
    }
}

/// Axiom: Deleted is an absorbing state — no transitions out.
///
/// Bar-Shalom, Li & Kirubarajan (2001) §7.5: once a track is removed
/// from the active set, it does not return; resuming surveillance of
/// the same target initiates a new tentative track.
pub struct DeletedIsAbsorbing;

impl Axiom for DeletedIsAbsorbing {
    fn verify(&self) -> Verdict {
        // Absorbing per Bar-Shalom (2001) §7.5 applies to *track-lifecycle
        // transitions* (Confirm / Miss / Delete / ReDetect), not to
        // metalogical relations like Opposition / Subsumption / Parthood
        // / Causation, which carry no flow semantics for the tracking
        // automaton. Identity self-loops are also excluded.
        let absorbing = !MultiTargetCategory::morphisms().iter().any(|m| {
            !matches!(
                m.kind(),
                MultiTargetRelationKind::Identity
                    | MultiTargetRelationKind::Subsumption
                    | MultiTargetRelationKind::Parthood
                    | MultiTargetRelationKind::Causation
                    | MultiTargetRelationKind::Opposition
            ) && m.source() == MultiTargetConcept::Deleted
                && m.target() != MultiTargetConcept::Deleted
        });
        if absorbing {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DeletedIsAbsorbing",
        "Deleted is absorbing: no non-identity transition leaves Deleted",
        "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §7.5"
    );
}

pr4xis::register_axiom!(
    DeletedIsAbsorbing,
    "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §7.5"
);

/// Axiom: every track begins in the `Tentative` state — no transitions
/// arrive at `Tentative` from anywhere else.
///
/// Bar-Shalom (2001) §7.5 — track initiation is the only path that ends
/// at Tentative. Re-detection of a target after deletion starts a fresh
/// Tentative track, not a re-entry into the same one.
pub struct TrackStartsTentative;

impl Axiom for TrackStartsTentative {
    fn verify(&self) -> Verdict {
        // As in `DeletedIsAbsorbing`: filter to lifecycle transitions only
        // — metalogical kinds (Subsumption / Parthood / Causation /
        // Opposition / Identity) do not carry initiation semantics.
        let only_self_loops_to_tentative = !MultiTargetCategory::morphisms().iter().any(|m| {
            !matches!(
                m.kind(),
                MultiTargetRelationKind::Identity
                    | MultiTargetRelationKind::Subsumption
                    | MultiTargetRelationKind::Parthood
                    | MultiTargetRelationKind::Causation
                    | MultiTargetRelationKind::Opposition
            ) && m.target() == MultiTargetConcept::Tentative
                && m.source() != MultiTargetConcept::Tentative
        });
        if only_self_loops_to_tentative {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TrackStartsTentative",
        "every track begins in Tentative state — no non-identity edge ends at Tentative",
        "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §7.5"
    );
}

pr4xis::register_axiom!(
    TrackStartsTentative,
    "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §7.5"
);

/// Axiom: a coasting track may return to `Confirmed` upon re-detection.
///
/// Blackman & Popoli (1999) — coasting is a recoverable state; if the
/// sensor regains a detection within the deletion-threshold window the
/// track returns to confirmed status without restart.
pub struct ReDetectionPossible;

impl Axiom for ReDetectionPossible {
    fn verify(&self) -> Verdict {
        let has_edge = MultiTargetCategory::morphisms().iter().any(|m| {
            m.source() == MultiTargetConcept::Coasting
                && m.target() == MultiTargetConcept::Confirmed
        });
        if has_edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ReDetectionPossible",
        "coasting tracks may return to Confirmed on re-detection",
        "Blackman & Popoli (1999) Design and Analysis of Modern Tracking Systems Ch. 6"
    );
}

pr4xis::register_axiom!(
    ReDetectionPossible,
    "Blackman & Popoli (1999) Design and Analysis of Modern Tracking Systems Ch. 6"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<MultiTargetCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        MultiTargetOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn four_track_states() {
        assert_eq!(MultiTargetConcept::variants().len(), 4);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn deleted_is_absorbing_holds() {
        assert!(DeletedIsAbsorbing.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn track_starts_tentative_holds() {
        assert!(TrackStartsTentative.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn redetection_possible_holds() {
        assert!(ReDetectionPossible.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn confirm_edge_exists() {
        let has_confirm = MultiTargetCategory::morphisms().iter().any(|m| {
            m.kind() == MultiTargetRelationKind::Confirm
                && m.source() == MultiTargetConcept::Tentative
                && m.target() == MultiTargetConcept::Confirmed
        });
        assert!(has_confirm);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn description_total() {
        for c in MultiTargetConcept::variants() {
            assert!(TrackStateDescription.get(&c).is_some());
        }
    }

    fn arb_concept() -> impl Strategy<Value = MultiTargetConcept> {
        proptest::sample::select(MultiTargetConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in MultiTargetCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in MultiTargetOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_description_total(c in arb_concept()) {
            prop_assert!(TrackStateDescription.get(&c).is_some());
        }

        #[test]
        fn prop_no_edge_leaves_deleted(_seed in any::<u32>()) {
            // Absorbing-state invariant on the track-lifecycle transition
            // graph. Metalogical kinds (Opposition / Subsumption / Parthood
            // / Causation / Identity) are not transitions.
            for m in MultiTargetCategory::morphisms() {
                if !matches!(
                    m.kind(),
                    MultiTargetRelationKind::Identity
                        | MultiTargetRelationKind::Subsumption
                        | MultiTargetRelationKind::Parthood
                        | MultiTargetRelationKind::Causation
                        | MultiTargetRelationKind::Opposition
                ) && m.source() == MultiTargetConcept::Deleted
                {
                    prop_assert_eq!(m.target(), MultiTargetConcept::Deleted);
                }
            }
        }

        #[test]
        fn prop_no_edge_enters_tentative_from_elsewhere(_seed in any::<u32>()) {
            for m in MultiTargetCategory::morphisms() {
                if !matches!(
                    m.kind(),
                    MultiTargetRelationKind::Identity
                        | MultiTargetRelationKind::Subsumption
                        | MultiTargetRelationKind::Parthood
                        | MultiTargetRelationKind::Causation
                        | MultiTargetRelationKind::Opposition
                ) && m.target() == MultiTargetConcept::Tentative
                {
                    prop_assert_eq!(m.source(), MultiTargetConcept::Tentative);
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_description_total, Verifiable);
    pr4xis::register_praxis_value!(prop_no_edge_leaves_deleted, Verifiable);
    pr4xis::register_praxis_value!(prop_no_edge_enters_tentative_from_elsewhere, Verifiable);
}

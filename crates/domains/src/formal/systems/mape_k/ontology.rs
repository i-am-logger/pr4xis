//! MAPE-K — Monitor / Analyze / Plan / Execute over Knowledge.
//!
//! The canonical autonomic-computing control loop from:
//!
//! > Kephart, J. O. & Chess, D. M. (2003). *The Vision of Autonomic
//! > Computing*. IEEE Computer 36(1), 41–50.
//! > DOI: [10.1109/MC.2003.1160055](https://doi.org/10.1109/MC.2003.1160055)
//!
//! An autonomic system closes a four-phase cycle over a shared
//! knowledge base:
//!
//! ```text
//!   ┌── Monitor ── Analyze ── Plan ── Execute ──┐
//!   │       │        │        │        │       │
//!   │       └────────┴── Knowledge ─────┘       │
//!   │                                           │
//!   └───────────────── loop ────────────────────┘
//! ```
//!
//! Every phase *consults* the Knowledge base; Execute's side-effects
//! feed the next Monitor read. The loop closure (`Execute → Monitor`)
//! is what makes MAPE-K a **cycle**, not a linear pipeline.
//!
//! # Why this is pr4xis's chat pipeline
//!
//! The 13 existing `PipelineStep` variants map cleanly onto the four
//! phases — see `docs/research/pipeline-architecture-survey.md`. The
//! `PipelineStep → MapeK` cross-functor at `pipeline_step_functor.rs`
//! encodes the mapping as a verified structure-preserving arrow.
//!
//! # Related literature
//!
//! - IBM Autonomic Computing White Paper (2003), *An architectural
//!   blueprint for autonomic computing* — the original MAPE-K elaboration.
//! - Brun, Y. et al. (2009). *Engineering self-adaptive systems through
//!   feedback loops*. SEAMS. Surveys MAPE-K variants in self-adaptive
//!   software.
//! - Related pr4xis ontologies: `formal::systems` (Wiener cybernetics),
//!   `cognitive::cognition::metacognition` (second-order monitoring),
//!   `cognitive::linguistics::pipeline` (the chat flow itself).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "MapeK",
    source: "Kephart & Chess (2003), IEEE Computer 36(1)",

    concepts: [
        // === The four phases ===
        Monitor,
        Analyze,
        Plan,
        Execute,

        // === Shared substrate ===
        Knowledge,

        // === Abstract parent (for the phase set) ===
        MapeKPhase,
    ],

    labels: {
        Monitor: ("en", "Monitor", "The phase that observes the managed element and updates the knowledge base with sensed state. In pr4xis's chat pipeline this covers tokenisation, parsing, semantic interpretation, metacognition, and epistemic classification."),
        Analyze: ("en", "Analyze", "The phase that reasons over the current knowledge to diagnose what (if anything) needs to be changed. In pr4xis this covers entity lookup, taxonomy traversal, and common-ancestor computation."),
        Plan: ("en", "Plan", "The phase that constructs a plan of action from the analysis. In pr4xis this covers speech-act classification and response-frame selection (with Bratman 1987 BDI as the internal architecture of this phase)."),
        Execute: ("en", "Execute", "The phase that carries out the plan's actions, producing side-effects on the managed element. In pr4xis this covers content determination, document planning, and realisation (Reiter & Dale 2000 NLG pipeline as its internal structure)."),
        Knowledge: ("en", "Knowledge", "The shared knowledge base every MAPE phase consults and updates. In pr4xis this is the ontology substrate — every phase reads from and writes to the same knowledge graph."),
        MapeKPhase: ("en", "MAPE-K phase", "The abstract parent class of Monitor, Analyze, Plan, and Execute. Each phase has an input (consumes knowledge + prior-phase output) and an output (produces knowledge updates + next-phase input)."),
    },

    is_a: [
        // The four phases are all instances of the abstract MapeKPhase.
        (Monitor, MapeKPhase),
        (Analyze, MapeKPhase),
        (Plan, MapeKPhase),
        (Execute, MapeKPhase),
    ],

    edges: [
        // === The canonical four-phase cycle ===
        // Monitor → Analyze → Plan → Execute → Monitor (loop closure)
        (Monitor, Analyze, HandsOffTo),
        (Analyze, Plan, HandsOffTo),
        (Plan, Execute, HandsOffTo),
        (Execute, Monitor, HandsOffTo),

        // === Knowledge consultation (every phase reads + writes) ===
        (Monitor, Knowledge, Consults),
        (Analyze, Knowledge, Consults),
        (Plan, Knowledge, Consults),
        (Execute, Knowledge, Consults),
    ],

}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// The role a concept plays within the MAPE-K loop.
#[derive(Debug, Clone)]
pub struct MapeKRole;

impl Quality for MapeKRole {
    type Individual = MapeKConcept;
    type Value = &'static str;

    fn get(&self, c: &MapeKConcept) -> Option<&'static str> {
        use MapeKConcept as M;
        Some(match c {
            M::MapeKPhase => "abstract-phase",
            M::Monitor => "sense",
            M::Analyze => "diagnose",
            M::Plan => "decide",
            M::Execute => "act",
            M::Knowledge => "substrate",
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn direct_children_of(parent: MapeKConcept) -> Vec<MapeKConcept> {
    use pr4xis::category::{Arrow, Category};
    MapeKCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == MapeKRelationKind::Subsumption && m.target() == parent)
        .map(|m| m.source())
        .collect()
}

fn kinded_edge_exists(from: MapeKConcept, to: MapeKConcept, kind: MapeKRelationKind) -> bool {
    use pr4xis::category::{Arrow, Category};
    MapeKCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

// ---------------------------------------------------------------------------
// Domain axioms — separate `impl Axiom` blocks (new `verify` / `axiom_meta!`
// shape per #160 / #167).
// ---------------------------------------------------------------------------

/// Kephart & Chess (2003): the four phases are children of MapeKPhase.
pub struct FourPhaseCycle;

impl Axiom for FourPhaseCycle {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let actual = direct_children_of(MapeKConcept::MapeKPhase);
        let expected = [
            MapeKConcept::Monitor,
            MapeKConcept::Analyze,
            MapeKConcept::Plan,
            MapeKConcept::Execute,
        ];
        let ok = actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FourPhaseCycle",
        "direct children of MapeKPhase are exactly {Monitor, Analyze, Plan, Execute}",
        "Kephart & Chess (2003) IEEE Computer 36(1) \u{00a7}2"
    );
}
pr4xis::register_axiom!(
    FourPhaseCycle,
    "Kephart & Chess (2003) IEEE Computer 36(1) \u{00a7}2"
);

/// Kephart & Chess (2003): the four phases form a closed cycle M -> A -> P -> E -> M.
pub struct LoopIsClosed;

impl Axiom for LoopIsClosed {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = kinded_edge_exists(
            MapeKConcept::Monitor,
            MapeKConcept::Analyze,
            MapeKRelationKind::HandsOffTo,
        ) && kinded_edge_exists(
            MapeKConcept::Analyze,
            MapeKConcept::Plan,
            MapeKRelationKind::HandsOffTo,
        ) && kinded_edge_exists(
            MapeKConcept::Plan,
            MapeKConcept::Execute,
            MapeKRelationKind::HandsOffTo,
        ) && kinded_edge_exists(
            MapeKConcept::Execute,
            MapeKConcept::Monitor,
            MapeKRelationKind::HandsOffTo,
        );
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LoopIsClosed",
        "the four MAPE-K phases form a closed cycle: Monitor -> Analyze -> Plan -> Execute -> Monitor (HandsOffTo edges)",
        "Kephart & Chess (2003) IEEE Computer 36(1) \u{00a7}2"
    );
}
pr4xis::register_axiom!(
    LoopIsClosed,
    "Kephart & Chess (2003) IEEE Computer 36(1) \u{00a7}2"
);

/// Kephart & Chess (2003) — every MAPE phase has a Consults edge to Knowledge.
pub struct EveryPhaseConsultsKnowledge;

impl Axiom for EveryPhaseConsultsKnowledge {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let consults = |from: MapeKConcept| {
            kinded_edge_exists(from, MapeKConcept::Knowledge, MapeKRelationKind::Consults)
        };
        let ok = consults(MapeKConcept::Monitor)
            && consults(MapeKConcept::Analyze)
            && consults(MapeKConcept::Plan)
            && consults(MapeKConcept::Execute);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EveryPhaseConsultsKnowledge",
        "every MAPE phase carries a Consults edge to Knowledge (shared K substrate)",
        "Kephart & Chess (2003) IEEE Computer 36(1) \u{00a7}2"
    );
}
pr4xis::register_axiom!(
    EveryPhaseConsultsKnowledge,
    "Kephart & Chess (2003) IEEE Computer 36(1) \u{00a7}2"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for MapeKOntology {
    type Cat = MapeKCategory;
    type Qual = MapeKRole;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(FourPhaseCycle));
        axioms.push(Box::new(LoopIsClosed));
        axioms.push(Box::new(EveryPhaseConsultsKnowledge));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<MapeKCategory>();
    }

    #[test]
    fn ontology_validates() {
        MapeKOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn four_phase_cycle_holds() {
        assert!(FourPhaseCycle.verify().is_ok());
    }

    #[test]
    fn loop_is_closed_holds() {
        assert!(LoopIsClosed.verify().is_ok());
    }

    #[test]
    fn every_phase_consults_knowledge_holds() {
        assert!(EveryPhaseConsultsKnowledge.verify().is_ok());
    }
}

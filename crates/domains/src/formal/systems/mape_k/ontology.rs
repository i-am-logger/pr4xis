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

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "MapeK",
    source: "Kephart & Chess (2003), IEEE Computer 36(1)",
    being: Process,

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
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
    MapeKTaxonomy::relations()
        .into_iter()
        .filter_map(|(child, p)| if p == parent { Some(child) } else { None })
        .collect()
}

// ---------------------------------------------------------------------------
// Axioms — invariants of the MAPE-K architecture
// ---------------------------------------------------------------------------

/// Axiom: the direct children of `MapeKPhase` are exactly
/// `{Monitor, Analyze, Plan, Execute}` — Kephart & Chess (2003) §2.
/// Three-phase variants (e.g., MAP) are rejected; the full four are
/// required.
pub struct FourPhaseCycle;

impl Axiom for FourPhaseCycle {
    fn description(&self) -> &str {
        "the direct children of MapeKPhase are exactly {Monitor, Analyze, Plan, Execute} (Kephart & Chess 2003)"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(MapeKConcept::MapeKPhase);
        let expected = [
            MapeKConcept::Monitor,
            MapeKConcept::Analyze,
            MapeKConcept::Plan,
            MapeKConcept::Execute,
        ];
        actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c))
    }
}

/// Axiom: the phase-to-phase transitions form a closed 4-cycle:
/// `Monitor → Analyze → Plan → Execute → Monitor`. Without the
/// `Execute → Monitor` closing edge, MAPE-K would be a linear pipeline
/// rather than an autonomic loop.
pub struct LoopIsClosed;

impl Axiom for LoopIsClosed {
    fn description(&self) -> &str {
        "the four phases form a closed cycle M → A → P → E → M (Kephart & Chess 2003 §2)"
    }
    fn holds(&self) -> bool {
        use MapeKConcept as M;
        use MapeKRelationKind as K;
        let morphs = MapeKCategory::morphisms();
        let has = |from: M, to: M| {
            morphs
                .iter()
                .any(|r| r.from == from && r.to == to && r.kind == K::HandsOffTo)
        };
        has(M::Monitor, M::Analyze)
            && has(M::Analyze, M::Plan)
            && has(M::Plan, M::Execute)
            && has(M::Execute, M::Monitor)
    }
}

/// Axiom: every phase `Consults` `Knowledge` — the K in MAPE-K is
/// shared, not a phase's private state. Kephart & Chess (2003) §2:
/// "all four of these phases share a common knowledge base."
pub struct EveryPhaseConsultsKnowledge;

impl Axiom for EveryPhaseConsultsKnowledge {
    fn description(&self) -> &str {
        "every MAPE phase has a Consults edge to Knowledge (Kephart & Chess 2003 §2 — shared K)"
    }
    fn holds(&self) -> bool {
        use MapeKConcept as M;
        use MapeKRelationKind as K;
        let morphs = MapeKCategory::morphisms();
        let consults = |from: M| {
            morphs
                .iter()
                .any(|r| r.from == from && r.to == M::Knowledge && r.kind == K::Consults)
        };
        consults(M::Monitor) && consults(M::Analyze) && consults(M::Plan) && consults(M::Execute)
    }
}

impl Ontology for MapeKOntology {
    type Cat = MapeKCategory;
    type Qual = MapeKRole;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        MapeKOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(FourPhaseCycle),
            Box::new(LoopIsClosed),
            Box::new(EveryPhaseConsultsKnowledge),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<MapeKCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        MapeKOntology::validate().unwrap();
    }

    #[test]
    fn four_phase_cycle_holds() {
        assert!(FourPhaseCycle.holds(), "{}", FourPhaseCycle.description());
    }

    #[test]
    fn loop_is_closed_holds() {
        assert!(LoopIsClosed.holds(), "{}", LoopIsClosed.description());
    }

    #[test]
    fn every_phase_consults_knowledge_holds() {
        assert!(
            EveryPhaseConsultsKnowledge.holds(),
            "{}",
            EveryPhaseConsultsKnowledge.description()
        );
    }
}

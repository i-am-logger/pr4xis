//! Trace theory — operational / temporal view of derivations.
//!
//! The atemporal side is [`formal::logic::proof_theory`]; this is its
//! operational twin. Under Curry-Howard and the Hyland-Ong
//! characterisation of PCF, a trace IS a proof presented as a sequence
//! of moves. This ontology names the operational-semantics and
//! game-semantics vocabulary.
//!
//! # Literature
//!
//! - **Plotkin (1981)** *A Structural Approach to Operational Semantics*
//!   (DAIMI FN-19, Aarhus). SOS: reduction relations, small-step
//!   semantics, trace = reduction sequence.
//! - **Plotkin & Power (2002)** "Notions of Computation Determine Monads"
//!   (FoSSaCS). Computational effects and their traces.
//! - **Abramsky & Jung (1994)** *Domain Theory* (Handbook of Logic in
//!   Computer Science). Observational equivalence and trace semantics.
//! - **Hyland & Ong (2000)** "On Full Abstraction for PCF"
//!   (Information and Computation 163: 285–408). Game-semantic
//!   characterisation: innocent strategies = traces of play = proofs.
//! - **Abramsky (1996)** "Retracing some paths in process algebra"
//!   (CONCUR). Trace equivalence in process theory.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "TraceTheory",
    source: "Plotkin (1981) SOS; Plotkin-Power (2002) Notions of Computation; Abramsky-Jung (1994) Domain Theory; Hyland-Ong (2000) Full Abstraction for PCF; Abramsky (1996) process algebra",
    being: AbstractObject,

    concepts: [
        // === Genus ===
        Trace,

        // === Operational-semantics primitives (Plotkin 1981) ===
        ReductionStep,
        ReductionSequence,
        SmallStep,
        BigStep,
        Configuration,
        EvaluationContext,

        // === Observational ===
        ObservationalTrace,
        Observation,
        Event,

        // === Strategies / game semantics (Hyland-Ong 2000) ===
        Strategy,
        Play,
        Move,
        Position,
        InnocentStrategy,
        Interaction,

        // === Trace equivalence ===
        TraceEquivalence,
        BisimulationCandidate,
    ],

    labels: {
        Trace: ("en", "Trace",
            "The genus concept: an operational / temporal record of computation. A sequence of moves, reductions, or observations. Equivalent under Curry-Howard / Hyland-Ong to a proof."),

        ReductionStep: ("en", "Reduction step",
            "Plotkin (1981): a single rewriting move on a configuration — a one-step application of a reduction rule. The atomic move of operational semantics."),
        ReductionSequence: ("en", "Reduction sequence",
            "Plotkin: a chain of reduction steps. The most common trace shape — finite or infinite, possibly terminating in a normal form."),
        SmallStep: ("en", "Small-step semantics",
            "Plotkin (1981): a reduction relation where each step performs one atomic computation. Contrasts with big-step (natural semantics) which takes the whole computation as one rule."),
        BigStep: ("en", "Big-step semantics",
            "Kahn (1987) natural semantics: a rule reduces an expression directly to its final value. Less granular than small-step; trace captures only the top-level reduction."),
        Configuration: ("en", "Configuration",
            "A state of computation — the code being evaluated together with its context (environment, store, etc.). The domain of reduction steps."),
        EvaluationContext: ("en", "Evaluation context",
            "Felleisen (1987) evaluation context: a term with a hole, selecting where the next reduction happens. Formalises the call-by-value / call-by-name choice."),

        ObservationalTrace: ("en", "Observational trace",
            "A trace restricted to externally-visible events. Abramsky-Jung (1994): the basis for observational equivalence."),
        Observation: ("en", "Observation",
            "An externally-visible fact about a computation — an output, a side-effect, a termination signal. The granularity of observational traces."),
        Event: ("en", "Event",
            "A discrete happening in a trace — an input, an output, a state change, a message send. Process-algebra primitive (Abramsky 1996)."),

        Strategy: ("en", "Strategy",
            "Hyland-Ong (2000): a player's plan for responding to opponent moves in a game. Game-semantic dual of a program."),
        Play: ("en", "Play",
            "Hyland-Ong: a complete sequence of game moves — a legal trace of an interaction between Player and Opponent."),
        Move: ("en", "Move",
            "Hyland-Ong: an atomic game action — a question or an answer in a specific game position."),
        Position: ("en", "Position",
            "Hyland-Ong: the configuration of a game at a moment — whose turn it is, what moves are available."),
        InnocentStrategy: ("en", "Innocent strategy",
            "Hyland-Ong (2000): a strategy whose response depends only on the Player-view of the play, not the full history. THE equivalence class corresponding to PCF programs — and to proofs under Curry-Howard."),
        Interaction: ("en", "Interaction",
            "Hyland-Ong / Girard: the composition of two strategies along their common interface. The game-semantic analogue of cut elimination."),

        TraceEquivalence: ("en", "Trace equivalence",
            "Two systems are trace-equivalent iff they produce the same set of observational traces. Coarser than bisimulation but decidable in more cases. Abramsky (1996)."),
        BisimulationCandidate: ("en", "Bisimulation candidate",
            "Park-Milner (1981): a stronger equivalence than traces — systems must match moves step-by-step, not just produce the same traces. The basis for process-algebra reasoning."),
    },

    is_a: [
        // Reduction structure specialises Trace.
        (ReductionSequence, Trace),
        (ObservationalTrace, Trace),

        // Game-semantic specialisations.
        (Play, Trace),
        (InnocentStrategy, Strategy),

        // Reduction step varieties.
        (SmallStep, ReductionStep),
        (BigStep, ReductionStep),
    ],

    has_a: [
        // A trace is built from reduction steps (or moves, or events).
        (ReductionSequence, ReductionStep),
        (Trace, Event),
        (ObservationalTrace, Observation),

        // Game-semantic structure.
        (Play, Move),
        (Strategy, Move),
        (Interaction, Strategy),

        // Reductions operate on configurations via evaluation contexts.
        (ReductionStep, Configuration),
        (ReductionStep, EvaluationContext),
    ],

    opposes: [
        // Small-step vs big-step: two presentations of the same operational
        // content, but structurally opposed in granularity.
        (SmallStep, BigStep),

        // Trace equivalence is strictly coarser than bisimulation.
        (TraceEquivalence, BisimulationCandidate),
    ],
}

// -----------------------------------------------------------------------------
// Qualities
// -----------------------------------------------------------------------------

/// Literature tradition the concept comes from.
#[derive(Debug, Clone)]
pub struct TraceTradition;

impl Quality for TraceTradition {
    type Individual = TraceTheoryConcept;
    type Value = &'static str;

    fn get(&self, c: &TraceTheoryConcept) -> Option<&'static str> {
        use TraceTheoryConcept as T;
        Some(match c {
            T::ReductionStep
            | T::ReductionSequence
            | T::SmallStep
            | T::Configuration
            | T::EvaluationContext => "plotkin-sos-1981",
            T::BigStep => "kahn-natural-semantics-1987",
            T::ObservationalTrace | T::Observation | T::Event => "abramsky-jung-1994",
            T::Strategy
            | T::Play
            | T::Move
            | T::Position
            | T::InnocentStrategy
            | T::Interaction => "hyland-ong-2000",
            T::TraceEquivalence => "abramsky-1996",
            T::BisimulationCandidate => "park-milner-1981",
            T::Trace => "genus",
        })
    }
}

impl Ontology for TraceTheoryOntology {
    type Cat = TraceTheoryCategory;
    type Qual = TraceTradition;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        TraceTheoryOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        TraceTheoryOntology::generated_domain_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<TraceTheoryCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        TraceTheoryOntology::validate().unwrap();
    }

    #[test]
    fn reduction_sequence_is_a_trace() {
        use pr4xis::ontology::reasoning::taxonomy;
        assert!(taxonomy::is_a::<TraceTheoryTaxonomy>(
            &TraceTheoryConcept::ReductionSequence,
            &TraceTheoryConcept::Trace,
        ));
    }

    #[test]
    fn play_is_a_trace() {
        use pr4xis::ontology::reasoning::taxonomy;
        assert!(taxonomy::is_a::<TraceTheoryTaxonomy>(
            &TraceTheoryConcept::Play,
            &TraceTheoryConcept::Trace,
        ));
    }
}

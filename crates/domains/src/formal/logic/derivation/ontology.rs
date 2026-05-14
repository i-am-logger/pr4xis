//! Derivation — the genus concept unifying proof and trace.
//!
//! A **derivation** is a structured record of how a conclusion is reached
//! from starting data. Under Curry-Howard (1958), the Hyland–Ong (2000)
//! game-semantics characterisation of PCF, and the traced-monoidal-categories
//! framework of Joyal–Street–Verity (1996), "proof" (atemporal validity
//! witness) and "trace" (temporal computation record) are two presentations
//! of the same underlying structure. This ontology names that underlying
//! structure.
//!
//! # Why this ontology exists
//!
//! pr4xis has historically split derivation-like records across two places:
//! `pr4xis::engine::Trace` (runtime enforcement record) and
//! `pr4xis-domains::…::diagnostics::PipelineTrace` (MAPE-K pipeline record).
//! And axioms returned `bool`, throwing away the derivation entirely. This
//! ontology provides the single literature-grounded vocabulary that all
//! three concepts (engine trace, pipeline trace, axiom proof) instantiate.
//!
//! # Literature
//!
//! - **Joyal, Street & Verity (1996)** *Traced Monoidal Categories*
//!   (Math. Proc. Cambridge Phil. Soc. 119: 447–468). Defines trace as a
//!   categorical operation on a monoidal category, producing morphisms
//!   with feedback — the abstract shape of "derivation with cycles".
//! - **Plotkin (1981)** *A Structural Approach to Operational Semantics*
//!   (DAIMI FN-19, Aarhus). Reduction sequences as derivations of end
//!   states from start states.
//! - **Mac Lane (1971)** *Categories for the Working Mathematician*
//!   Ch. II §1. Morphisms ARE derivations in the deductive category
//!   reading (made explicit in Lambek 1968).
//! - **Lambek (1968)** "Deductive Systems and Categories I"
//!   (Math. Systems Theory 2: 287–318). Proofs as morphisms in a category.
//! - **Curry & Feys (1958)** *Combinatory Logic* Vol. I. The
//!   propositions-as-types correspondence; derivations as inhabitants.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Derivation",
    source: "Joyal-Street-Verity (1996) Traced Monoidal Categories; Plotkin (1981) SOS; Lambek (1968) Deductive Systems and Categories I; Mac Lane (1971) Ch. II §1; Curry & Feys (1958) Combinatory Logic",

    concepts: [
        // === Genus ===
        Derivation,

        // === Compositional structure ===
        DerivationStep,
        Sequence,
        Hypothesis,
        Consequence,

        // === Witness (what justifies the step) ===
        Witness,
        Evidence,
        Justification,

        // === Two canonical presentations (unified by this ontology) ===
        Proof,
        Trace,

        // === Operational aspect ===
        Reduction,
        Normalisation,
        Feedback,

        // === Composition / structure ===
        Composition,
        Cycle,
    ],

    labels: {
        Derivation: ("en", "Derivation",
            "The genus concept: a structured record by which a consequence is obtained from hypotheses. Both proofs and traces are derivations. Under Curry-Howard, Hyland-Ong, and traced-monoidal categories, a single underlying shape."),
        DerivationStep: ("en", "Derivation step",
            "A single move within a derivation — an application of an inference rule, a functor application, a reduction, or a model-check — with its own witness."),
        Sequence: ("en", "Sequence",
            "An ordered chain of steps. Plotkin (1981): a reduction sequence. Gentzen (1935): a sequent calculus proof. Joyal-Street-Verity (1996): a morphism composition in a traced category."),
        Hypothesis: ("en", "Hypothesis",
            "A starting datum of a derivation — an assumption, a premise, an input value, or a known fact. What the derivation takes as given."),
        Consequence: ("en", "Consequence",
            "The conclusion reached by a derivation — the derived fact, the proved theorem, or the computed output. What the derivation produces."),

        Witness: ("en", "Witness",
            "Evidence that a derivation step is justified. Can be a sub-derivation (proof-theoretic), a model fragment (model-theoretic), a trace entry (operational), or a functor application."),
        Evidence: ("en", "Evidence",
            "Concrete grounding for a witness — the actual data pointing to why a step is sound. Tarski (1936): the model-theoretic satisfaction relation."),
        Justification: ("en", "Justification",
            "The explanatory aspect of a witness — the 'because' of a step, expressed in ontology-level terms rather than raw computation."),

        Proof: ("en", "Proof",
            "A derivation whose witnesses establish logical validity. Atemporal view of the derivation-genus. Lambek (1968): a morphism in the deductive category."),
        Trace: ("en", "Trace",
            "A Proof with a temporal dimension layered on top — a Proof that additionally records WHEN each step fired. Under Curry-Howard (1958) and Hyland-Ong (2000), every trace IS a proof (of the input-output correspondence); it just carries more information — the run history. Plotkin (1981): a reduction sequence. Joyal-Street-Verity (1996): a morphism with feedback in a traced monoidal category."),

        Reduction: ("en", "Reduction",
            "An operational-semantics step that transforms one state to another. Plotkin (1981). Each reduction step is a DerivationStep; a sequence of reductions forms a Trace."),
        Normalisation: ("en", "Normalisation",
            "Prawitz (1965): a derivation is in normal form when all cuts are eliminated. For traces: the run has terminated. Curry-Howard (1958): normalisation = evaluation."),
        Feedback: ("en", "Feedback",
            "Joyal-Street-Verity (1996): the trace operator lets an arrow's output feed back into its input, producing a fixed-point / loop structure. The categorical foundation for recursion and iteration in traces."),

        Composition: ("en", "Composition",
            "Two derivations chain when the consequence of one matches the hypothesis of the next. The compositional structure of derivations is what makes them morphisms in a category (Mac Lane 1971)."),
        Cycle: ("en", "Cycle",
            "A derivation with feedback — Trace in the JSV sense. Distinct from a compositional chain in admitting self-reference."),
    },

    is_a: [
        // Proof is a derivation (atemporal presentation). Trace is a Proof
        // with a temporal dimension layered on top (Curry-Howard 1958;
        // Hyland-Ong 2000: every trace is a proof; additionally it tells
        // you WHEN each step fired). Transitively Trace is-a Derivation.
        (Proof, Derivation),
        (Trace, Proof),

        // A single step is part of a derivation (encoded via has_a below).
        // A sequence IS a derivation — the whole-chain view.
        (Sequence, Derivation),

        // Specific witness kinds.
        (Evidence, Witness),
        (Justification, Witness),

        // Normalisation is a kind of reduction-to-termination.
        (Normalisation, Reduction),

        // Cycle is a specific kind of composition structure.
        (Cycle, Composition),
    ],

    has_a: [
        // A derivation is structurally built from steps in a sequence.
        (Derivation, DerivationStep),
        (Sequence, DerivationStep),

        // Each step has a witness.
        (DerivationStep, Witness),

        // A derivation has a hypothesis (input side) and a consequence (output side).
        (Derivation, Hypothesis),
        (Derivation, Consequence),

        // Traces can carry feedback; proofs don't (classical) or do via fixed-point
        // combinators (type theory). The ontology permits it via Trace has_a Feedback.
        (Trace, Feedback),
    ],

    opposes: [
        // The canonical opposition: hypothesis vs consequence — what's given
        // vs what's derived. In any non-trivial derivation these are distinct.
        (Hypothesis, Consequence),
    ],
}

// -----------------------------------------------------------------------------
// Qualities
// -----------------------------------------------------------------------------

/// The presentation aspect — atemporal (proof) vs temporal (trace).
#[derive(Debug, Clone)]
pub struct DerivationAspect;

impl Quality for DerivationAspect {
    type Individual = DerivationConcept;
    type Value = &'static str;

    fn get(&self, c: &DerivationConcept) -> Option<&'static str> {
        use DerivationConcept as D;
        Some(match c {
            D::Proof | D::Normalisation => "atemporal",
            D::Trace | D::Reduction | D::Feedback | D::Cycle => "temporal",
            D::Derivation
            | D::DerivationStep
            | D::Sequence
            | D::Hypothesis
            | D::Consequence
            | D::Witness
            | D::Evidence
            | D::Justification
            | D::Composition => "neutral",
        })
    }
}

impl Ontology for DerivationOntology {
    type Cat = DerivationCategory;
    type Qual = DerivationAspect;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<DerivationCategory>();
    }

    #[test]
    fn ontology_validates() {
        DerivationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    /// Subsumption check via kinded-morphism filter (per_def `TaxonomyDef::is_a`
    /// is gone). Since Subsumption is transitive (per OBO-RO transitive_over),
    /// the proc macro emits closure edges into `morphisms()` — so this
    /// check covers both direct and transitive subsumption.
    fn is_a(child: DerivationConcept, parent: DerivationConcept) -> bool {
        use pr4xis::category::{Arrow, Category};
        DerivationCategory::morphisms().iter().any(|m| {
            m.kind() == DerivationRelationKind::Subsumption
                && m.source() == child
                && m.target() == parent
        })
    }

    #[test]
    fn proof_and_trace_are_both_derivations() {
        assert!(is_a(
            DerivationConcept::Proof,
            DerivationConcept::Derivation,
        ));
        // Trace is-a Proof is-a Derivation (transitive).
        assert!(is_a(
            DerivationConcept::Trace,
            DerivationConcept::Derivation,
        ));
    }

    #[test]
    fn trace_extends_proof_with_temporal_dimension() {
        // Curry-Howard (1958) / Hyland-Ong (2000): a trace IS a proof —
        // the temporal dimension is additive. Every trace is structurally
        // a proof; it just additionally records WHEN each step fired.
        assert!(is_a(DerivationConcept::Trace, DerivationConcept::Proof));
    }
}

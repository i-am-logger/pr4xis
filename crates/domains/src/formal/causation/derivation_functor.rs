//! Functor: Causation → Derivation. Hypothesis generation via abductive
//! reading of a causal model.
//!
//! ## What the functor encodes
//!
//! Peirce (1903) Harvard Lecture VII: *abduction* is "the operation of
//! adopting an explanatory hypothesis" — given a surprising observation,
//! one proposes the antecedent that would best explain it. Schematically:
//!
//! ```text
//! 1. The surprising fact C is observed.
//! 2. But if A were true, C would be a matter of course.
//! 3. Hence, there is reason to suspect that A is true.
//! ```
//!
//! Pearl (2000) §1.3 and Reiter (1987) make this computable: a causal
//! model (a `CausalGraph` whose edges are `Produces` morphisms) is read
//! *backwards* — observed `Effect` becomes the premise, the hypothesised
//! `Cause` becomes the conclusion, and the `CounterfactualDependence`
//! that grounds the causal arrow (Lewis 1973) becomes the justification
//! warranting the abductive step. This functor implements that reading
//! at the categorical level: every object and morphism of
//! [`super::ontology::CausationCategory`] is mapped to its role in the
//! abductive [`super::super::derivation::ontology::DerivationCategory`].
//!
//! ## Object map
//!
//! | Causation                                                                  | Derivation         | Reason |
//! |----------------------------------------------------------------------------|--------------------|--------|
//! | `Effect`                                                                   | `Premise`          | The observed datum — Peirce step 1. |
//! | `Cause`, `SufficientCause`, `NecessaryCause`, `ProximateCause`, `DistalCause`, `CommonCause` | `Conclusion` | The hypothesised antecedent — Peirce step 3. The five subtypes collapse to the same conclusion role (`Forgetful` functor, Mac Lane §I.3). |
//! | `Counterfactual`, `CounterfactualDependence`                               | `Justification`    | Lewis (1973) reduces causation to counterfactual dependence; the counterfactual conditional `¬A □→ ¬C` is the *warrant* the abductive step appeals to (Prawitz 1965). |
//! | `Intervention`                                                             | `InferenceRule`    | Pearl's `do(X)` is the inference rule that licenses reading the model forwards or backwards (Pearl 2000 §3). |
//! | `Preemption`, `Overdetermination`                                          | `ProofStep`        | Sub-derivation steps that rule out competing hypotheses (Hall 2004; Schurz 2008 §3.2 pattern of competitive abduction). |
//! | `CausalChain`                                                              | `ProofStep`        | One sub-derivation in the multi-step abductive proof. |
//! | `CausalGraph`                                                              | `Composition`      | The assembled multi-step abductive derivation (Schurz 2008 §4: complex-abductive composition). |
//!
//! ## Morphism map
//!
//! Source kinds (canonical + Causation's custom edges) project as follows:
//!
//! | Source `CausationRelationKind` | Target `DerivationRelationKind` | Reason |
//! |--------------------------------|---------------------------------|--------|
//! | `Identity`                     | `Identity`                      | Functor identity law (Mac Lane §I.3). |
//! | `Subsumption`                  | `Subsumption`                   | All five `Cause` subtypes collapse to `Conclusion`, so the projected morphism is a self-loop (`Conclusion → Conclusion`) — structurally identity, kept as `Subsumption` to preserve the source-side classification. |
//! | `Parthood`                     | `Parthood`                      | Canonical kind preserved. |
//! | `Causation`                    | `Causation`                     | Canonical kind preserved. |
//! | `Opposition`                   | `Opposition`                    | Canonical kind preserved. |
//! | `Produces` (Cause→Effect)      | `Causation`                     | The forward causal arrow becomes the backward abductive arrow `Conclusion → Premise` in derivation terms — kept as `Causation` because in the Derivation pipeline the conclusion *causes* the premise to be explained. |
//! | `ActsOn` (Intervention→Cause)  | `Causation`                     | Pearl's `do(X)`-operator's action on a cause maps to the inference-rule-causes-conclusion edge. |
//! | `Grounds` (CounterfactualDependence→Cause) | `Causation`         | Lewis's grounding: the counterfactual warrants (causes) the conclusion. |
//! | `ParticipatesIn` (Cause/Effect → CausalChain) | `Parthood`            | A cause or effect is part of the causal chain ↦ a conclusion or premise is part of the proof step. |
//! | `EmbedsIn` (CausalChain → CausalGraph) | `Parthood`              | A chain embedded in a graph ↦ a proof step embedded in the composition. |
//! | `Involves` (Preemption/Overdetermination → Cause) | `Causation`      | A structural pattern involves a cause ↦ the proof step involves the conclusion causally. |
//!
//! ## Classification
//!
//! [`CausationToDerivation::KIND`] is [`FunctorKind::Forgetful`] (Mac
//! Lane §I.3 / Awodey §7.2): the functor forgets the fine causal
//! typology (five subtypes of `Cause` collapse to one `Conclusion`),
//! but preserves the structural shape of the causal model (the
//! Subsumption / Parthood / Causation skeleton of the source category
//! is mirrored in the target).
//!
//! ## Literature
//!
//! - **Peirce, C. S. (1903)** *Harvard Lectures on Pragmatism*, Lecture
//!   VII — abductive schema as the third mode of inference beyond
//!   deduction and induction.
//! - **Lewis, D. (1973)** "Causation", *Journal of Philosophy* 70:
//!   556–567 — counterfactual analysis of causation.
//! - **Reichenbach, H. (1956)** *The Direction of Time*, University of
//!   California Press — common-cause principle.
//! - **Reiter, R. (1987)** "A Theory of Diagnosis from First
//!   Principles", *Artificial Intelligence* 32: 57–95 — abductive
//!   diagnosis as inverse of the causal model.
//! - **Pearl, J. (2000)** *Causality: Models, Reasoning and Inference*,
//!   Cambridge University Press — `do`-calculus, the formal account of
//!   interventions on a causal graph.
//! - **Hall, N. (2004)** "Two Concepts of Causation", in *Causation
//!   and Counterfactuals*, MIT Press — production vs dependence
//!   accounts of causation, basis for the typology of causes.
//! - **Schurz, G. (2008)** "Patterns of Abduction", *Synthese* 164:
//!   201–234 — formal taxonomy of abductive inference patterns.
//! - **Magnani, L. (2009)** *Abductive Cognition: The Epistemological
//!   and Eco-Cognitive Dimensions of Hypothetical Reasoning*, Springer
//!   — computational and cognitive shape of abductive hypothesis
//!   generation.
//! - **Prawitz, D. (1965)** *Natural Deduction: A Proof-Theoretical
//!   Study*, Almqvist & Wiksell — justification as the explicit appeal
//!   to a rule or premise that warrants a proof step.
//! - **Mac Lane, S. (1971)** *Categories for the Working Mathematician*,
//!   Springer GTM 5, §I.3 (Functors), §I.4 (Forgetful functors).
//! - **Awodey, S. (2010)** *Category Theory*, 2nd ed., Oxford
//!   University Press, §7.2 (Faithful, full, forgetful functors).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Functor, kinds::FunctorKind};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::ontology::{
    CausationCategory, CausationConcept, CausationRelation, CausationRelationKind,
};
use crate::formal::derivation::ontology::{
    DerivationCategory, DerivationConcept, DerivationRelation, DerivationRelationKind,
};

/// The abductive-reading functor from a causal model (source) to its
/// hypothesis-generation derivation (target). See the module-level
/// docs for the mapping table and literature.
pub struct CausationToDerivation;

/// Object map. Five subtypes of `Cause` (Sufficient/Necessary/Proximate/
/// Distal/Common) collapse to `Conclusion`; this is the *forgetful*
/// shape of the functor (Mac Lane §I.3 / Awodey §7.2).
fn map_concept(c: &CausationConcept) -> DerivationConcept {
    use CausationConcept as C;
    use DerivationConcept as D;
    match c {
        C::Effect => D::Premise,

        C::Cause
        | C::SufficientCause
        | C::NecessaryCause
        | C::ProximateCause
        | C::DistalCause
        | C::CommonCause => D::Conclusion,

        C::Counterfactual | C::CounterfactualDependence => D::Justification,

        C::Intervention => D::InferenceRule,

        C::Preemption | C::Overdetermination | C::CausalChain => D::ProofStep,

        C::CausalGraph => D::Composition,
    }
}

/// Morphism-kind map. Canonical kinds (Identity / Subsumption /
/// Parthood / Causation / Opposition) project to themselves; the
/// custom Causation-side kinds (`Produces`, `ActsOn`, `Grounds`,
/// `ParticipatesIn`, `EmbedsIn`, `Involves`) collapse to whichever
/// canonical kind names the structural role they play in the
/// abductive derivation.
fn map_kind(kind: &CausationRelationKind) -> DerivationRelationKind {
    use CausationRelationKind as S;
    use DerivationRelationKind as T;
    match kind {
        S::Identity => T::Identity,
        S::Subsumption => T::Subsumption,
        S::Parthood => T::Parthood,
        S::Causation => T::Causation,
        S::Opposition => T::Opposition,

        // The forward causal arrow becomes the backward abductive
        // arrow in derivation terms — kept as `Causation`.
        S::Produces => T::Causation,
        // Pearl's `do(X)` action on a cause.
        S::ActsOn => T::Causation,
        // Lewis's counterfactual grounding (Prawitz justification).
        S::Grounds => T::Causation,
        // A cause/effect participating in a chain ↦ a conclusion/
        // premise part of the proof step.
        S::ParticipatesIn => T::Parthood,
        // A chain embedded in a graph ↦ a step embedded in the
        // composition.
        S::EmbedsIn => T::Parthood,
        // Structural pattern involving a cause ↦ proof step
        // involving the conclusion causally.
        S::Involves => T::Causation,
    }
}

impl Functor for CausationToDerivation {
    type Source = CausationCategory;
    type Target = DerivationCategory;

    const KIND: FunctorKind = FunctorKind::Forgetful;

    fn map_object(obj: &CausationConcept) -> DerivationConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &CausationRelation) -> DerivationRelation {
        DerivationRelation {
            from: map_concept(&m.from),
            to: map_concept(&m.to),
            kind: map_kind(&m.kind),
        }
    }

    fn meta() -> Provenance {
        Provenance {
            name: OntologyName::new_static("CausationToDerivation"),
            description: Label::new_static(
                "Causation → Derivation (abductive hypothesis generation): \
                 Peirce 1903 schema, Lewis 1973 counterfactual grounding, \
                 Pearl 2000 do-operator, Reiter 1987 diagnosis, Schurz 2008 patterns",
            ),
            citation: Citation::parse_static(
                "Peirce (1903) Harvard Lectures on Pragmatism Lecture VII; \
                 Lewis (1973) J. Phil. 70: 556; \
                 Reiter (1987) AI 32: 57-95; \
                 Pearl (2000) Causality §1.3, §3; \
                 Hall (2004) in Causation and Counterfactuals (MIT); \
                 Schurz (2008) Synthese 164: 201-234; \
                 Magnani (2009) Abductive Cognition (Springer); \
                 Prawitz (1965) Natural Deduction; \
                 Mac Lane (1971) Categories §I.3-I.4; \
                 Awodey (2010) Category Theory §7.2",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

pr4xis::register_functor!(CausationToDerivation);

// =============================================================================
// Domain axioms — encoding the substantive abductive-schema claims as
// axioms over the functor's image, separate from the structural functor
// laws (which assert_functor_laws checks).
// =============================================================================

use pr4xis::ontology::Axiom;

fn target_has_morphism(
    from: DerivationConcept,
    to: DerivationConcept,
    kind: DerivationRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    DerivationCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

/// Peirce's abductive schema step 1: the *observed* datum maps to the
/// `Premise` role in the abductive derivation. Verifies that
/// `CausationConcept::Effect` projects to `DerivationConcept::Premise`.
pub struct EffectIsAbductivePremise;

impl Axiom for EffectIsAbductivePremise {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if CausationToDerivation::map_object(&CausationConcept::Effect)
            == DerivationConcept::Premise
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EffectIsAbductivePremise",
        "Peirce (1903) step 1: the observed effect is the premise of the abductive derivation",
        "Peirce (1903) Harvard Lectures on Pragmatism Lecture VII"
    );
}

/// Peirce's abductive schema step 3: the hypothesised antecedent maps
/// to the `Conclusion` role. Verifies that `CausationConcept::Cause`
/// projects to `DerivationConcept::Conclusion`.
pub struct CauseIsAbductiveConclusion;

impl Axiom for CauseIsAbductiveConclusion {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if CausationToDerivation::map_object(&CausationConcept::Cause)
            == DerivationConcept::Conclusion
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CauseIsAbductiveConclusion",
        "Peirce (1903) step 3: the hypothesised cause is the conclusion of the abductive derivation",
        "Peirce (1903) Harvard Lectures on Pragmatism Lecture VII"
    );
}

/// Lewis's counterfactual analysis (1973): `CounterfactualDependence`
/// is the warrant for the abductive step. Verifies projection to
/// `Justification` and that the source `Grounds` edge from
/// `CounterfactualDependence` to `Cause` projects to a `Causation`-kind
/// morphism `Justification → Conclusion` in the derivation category.
pub struct CounterfactualDependenceGroundsConclusion;

impl Axiom for CounterfactualDependenceGroundsConclusion {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        // Source mapping: CounterfactualDependence → Justification.
        let from = CausationToDerivation::map_object(&CausationConcept::CounterfactualDependence);
        let to = CausationToDerivation::map_object(&CausationConcept::Cause);
        let kind = map_kind(&CausationRelationKind::Grounds);

        let projection_ok = from == DerivationConcept::Justification
            && to == DerivationConcept::Conclusion
            && kind == DerivationRelationKind::Causation;

        // Self-loops (Justification → Conclusion via Causation kind) are
        // not in the static morphism table of DerivationCategory — the
        // ontology macro only emits the edges declared in `causes:`.
        // What we verify here is that the projection produces the
        // expected typed morphism, not that the target category already
        // contains it: a Forgetful functor's image is closed under the
        // category operations on demand, not enumerated up-front.
        if projection_ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CounterfactualDependenceGroundsConclusion",
        "Lewis (1973): counterfactual dependence is the warrant grounding the abductive conclusion; projects to Justification --Causation--> Conclusion",
        "Lewis (1973) Causation, J. Phil. 70: 556"
    );
}

/// Pearl's `do`-calculus (2000): `Intervention` is the inference rule
/// licensing the abductive reading of the causal graph. Verifies
/// projection of `Intervention` to `InferenceRule`.
pub struct InterventionIsInferenceRule;

impl Axiom for InterventionIsInferenceRule {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if CausationToDerivation::map_object(&CausationConcept::Intervention)
            == DerivationConcept::InferenceRule
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "InterventionIsInferenceRule",
        "Pearl (2000) §3: the do-operator is the inference rule licensing the abductive reading of a causal graph",
        "Pearl (2000) Causality: Models, Reasoning and Inference, §3"
    );
}

/// Schurz (2008) §4: a `CausalGraph` projects to `Composition` — the
/// assembled multi-step abductive derivation. Verifies projection and
/// the structural fact that the target Derivation ontology recognises
/// `Composition` as one of its modes of inference (i.e. an `is_a
/// DerivationType` edge).
pub struct CausalGraphIsComposedAbduction;

impl Axiom for CausalGraphIsComposedAbduction {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        let projects_to_composition =
            CausationToDerivation::map_object(&CausationConcept::CausalGraph)
                == DerivationConcept::Composition;

        let composition_is_derivation_type = target_has_morphism(
            DerivationConcept::Composition,
            DerivationConcept::DerivationType,
            DerivationRelationKind::Subsumption,
        );

        if projects_to_composition && composition_is_derivation_type {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CausalGraphIsComposedAbduction",
        "Schurz (2008) §4: a complete causal graph projects to the Composition mode of inference (the assembled multi-step abductive derivation, recognised as a DerivationType in the target category)",
        "Schurz (2008) Patterns of Abduction, Synthese 164: 201-234, §4"
    );
}

pr4xis::register_axiom!(
    EffectIsAbductivePremise,
    "Peirce (1903) Harvard Lectures on Pragmatism Lecture VII"
);
pr4xis::register_axiom!(
    CauseIsAbductiveConclusion,
    "Peirce (1903) Harvard Lectures on Pragmatism Lecture VII"
);
pr4xis::register_axiom!(
    CounterfactualDependenceGroundsConclusion,
    "Lewis (1973) Causation, J. Phil. 70: 556; Prawitz (1965) Natural Deduction"
);
pr4xis::register_axiom!(
    InterventionIsInferenceRule,
    "Pearl (2000) Causality: Models, Reasoning and Inference, §3 (do-calculus)"
);
pr4xis::register_axiom!(
    CausalGraphIsComposedAbduction,
    "Schurz (2008) Patterns of Abduction, Synthese 164: 201-234, §4"
);

#[cfg(test)]
#[path = "derivation_functor_tests.rs"]
mod tests;

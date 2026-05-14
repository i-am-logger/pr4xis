//! Causation — the theory of causes and effects (issue #152).
//!
//! pr4xis domain ontologies have long used a `causes:` clause that
//! emits a hardcoded `CausalDef` trait. This ontology is the *richer*
//! vocabulary that `causes:` semantically refers to — where pr4xis
//! domain code wants to express counterfactuals, interventions,
//! preemption, and common causes, this is the target.
//!
//! Four literature lineages supply the concepts:
//!
//! 1. **Counterfactual theory** — Lewis (1973) "Causation", J. Phil. 70;
//!    Lewis (1986) *Philosophical Papers Vol. II*. Source of
//!    `Counterfactual`, `CounterfactualDependence`, `Preemption`,
//!    `Overdetermination`.
//!
//! 2. **Structural-equation / interventionist** — Pearl (2000)
//!    *Causality: Models, Reasoning and Inference*; Woodward (2003)
//!    *Making Things Happen*. Source of `Intervention`,
//!    `CausalGraph`, `Cause` as a structural-equation variable.
//!
//! 3. **Screening-off** — Reichenbach (1956) *The Direction of Time*.
//!    Source of `CommonCause` (Reichenbach's principle: correlations
//!    demand a cause, either direct or a common ancestor).
//!
//! 4. **Typology of causes** — Hall (2004) "Two Concepts of Causation";
//!    Mackie (1974) *The Cement of the Universe* (INUS conditions).
//!    Source of `SufficientCause`, `NecessaryCause`, `ProximateCause`,
//!    `DistalCause`.
//!
//! Source: Lewis (1973, 1986); Pearl (2000); Reichenbach (1956); Woodward
//! (2003); Hall (2004); Mackie (1974).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Causation",
    source: "Lewis (1973) J. Phil. 70; Pearl (2000) Causality; Reichenbach (1956); Woodward (2003); Hall (2004); Mackie (1974) Cement of the Universe",

    concepts: [
        // === Roles ===
        Cause,
        Effect,

        // === Typology (Hall 2004 + Mackie 1974 INUS) ===
        SufficientCause,
        NecessaryCause,
        ProximateCause,
        DistalCause,

        // === Reichenbach (1956) — screening-off ===
        CommonCause,

        // === Lewis (1973) counterfactual theory ===
        Counterfactual,
        CounterfactualDependence,
        Preemption,
        Overdetermination,

        // === Pearl / Woodward interventionist ===
        Intervention,
        CausalChain,
        CausalGraph,
    ],

    labels: {
        Cause: ("en", "Cause",
            "The antecedent entity or event in a causal relation. Lewis (1973): that upon which the effect counterfactually depends. Pearl (2000): a variable in a structural equation model."),
        Effect: ("en", "Effect",
            "The consequent entity or event. Lewis: the counterfactually-dependent term. Pearl: the variable whose value is determined (in part) by interventions on causes."),

        SufficientCause: ("en", "Sufficient cause",
            "Mackie (1974) INUS: a condition whose presence alone is enough for the effect. Not necessary — other sufficient causes may also obtain."),
        NecessaryCause: ("en", "Necessary cause",
            "Mackie: a condition without which the effect cannot occur. Not sufficient — may require other conditions to actually produce the effect."),
        ProximateCause: ("en", "Proximate cause",
            "The immediate (most-direct) cause in a chain, closest in time/space to the effect. Cf. legal tort theory: the cause for which liability attaches."),
        DistalCause: ("en", "Distal cause",
            "A remote cause upstream in the chain. Mayr's ultimate-vs-proximate split: evolutionary (distal) vs mechanistic (proximate)."),

        CommonCause: ("en", "Common cause",
            "Reichenbach's common-cause principle (1956): if two events are correlated but neither causes the other, a common cause screens off the correlation. Foundation of causal inference from observational data."),

        Counterfactual: ("en", "Counterfactual",
            "A conditional whose antecedent is (or is assumed) false: 'if A had not occurred, B would not have occurred'. Lewis (1973): causation IS counterfactual dependence in the nearest-world sense."),
        CounterfactualDependence: ("en", "Counterfactual dependence",
            "The relation Lewis's theory reduces causation to: B counterfactually depends on A iff (¬A □→ ¬B). Non-trivial because cause and effect may be temporally separated but counterfactually linked."),
        Preemption: ("en", "Preemption",
            "Lewis + Hall: a backup cause is ready to produce the effect but is forestalled by the actual cause. Early preemption (the backup never runs) and late preemption (the backup starts but is superseded)."),
        Overdetermination: ("en", "Overdetermination",
            "Lewis (1973) §5: multiple sufficient causes simultaneously producing the same effect, each independently adequate. Complicates counterfactual accounts since removing one doesn't remove the effect."),

        Intervention: ("en", "Intervention",
            "Pearl's `do(X)` operator; Woodward's interventionist account. An action that sets X to a value independent of its usual causes — letting us read off the causal effect of X on its descendants in the causal graph."),
        CausalChain: ("en", "Causal chain",
            "A sequence A → B → C where each arrow denotes direct causation. Transitive in idealised cases; breakable by preemption or Lewis-chains."),
        CausalGraph: ("en", "Causal graph",
            "Pearl (2000): a directed acyclic graph whose nodes are variables and whose edges are direct causal influences. The computational core of structural causal models."),
    },

    is_a: [
        // Typology specialisations
        (SufficientCause, Cause),
        (NecessaryCause, Cause),
        (ProximateCause, Cause),
        (DistalCause, Cause),
        (CommonCause, Cause),

        // CounterfactualDependence is a kind of Counterfactual
        (CounterfactualDependence, Counterfactual),

        // Preemption and Overdetermination are causal-structure patterns
        (Preemption, CausalChain),
    ],

    edges: [
        // Cause produces Effect (the defining morphism)
        (Cause, Effect, Produces),

        // Intervention acts on a Cause to read off effects
        (Intervention, Cause, ActsOn),

        // CounterfactualDependence is the semantic ground of causation (Lewis)
        (CounterfactualDependence, Cause, Grounds),

        // CausalChain is built from Cause → Effect links
        (Cause, CausalChain, ParticipatesIn),
        (Effect, CausalChain, ParticipatesIn),

        // CausalGraph embeds CausalChains
        (CausalChain, CausalGraph, EmbedsIn),

        // Preemption and Overdetermination are about multiple Causes competing
        (Preemption, Cause, Involves),
        (Overdetermination, Cause, Involves),
    ],

}

// -----------------------------------------------------------------------------
// Domain axioms — declared as separate `impl Axiom` blocks (new `verify` /
// `axiom_meta!` shape per #160 / #167). Each axiom filters
// `CausationCategory::morphisms()` by relation kind, per the kinded-morphism
// canonical pattern (per_def traits are gone).
// -----------------------------------------------------------------------------

fn produces_edge_exists(from: CausationConcept, to: CausationConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    CausationCategory::morphisms().iter().any(|m| {
        m.source() == from && m.target() == to && m.kind() == CausationRelationKind::Produces
    })
}

fn subsumption_pair_exists(child: CausationConcept, parent: CausationConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    CausationCategory::morphisms().iter().any(|m| {
        m.source() == child
            && m.target() == parent
            && m.kind() == CausationRelationKind::Subsumption
    })
}

fn direct_children_of_cause() -> Vec<CausationConcept> {
    use pr4xis::category::{Arrow, Category};
    CausationCategory::morphisms()
        .iter()
        .filter(|m| {
            m.kind() == CausationRelationKind::Subsumption && m.target() == CausationConcept::Cause
        })
        .map(|m| m.source())
        .collect()
}

fn kinded_edge_exists(
    from: CausationConcept,
    to: CausationConcept,
    kind: CausationRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    CausationCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

/// Causes precede effects — the (Cause, Effect, Produces) edge encodes
/// Reichenbach's temporal-priority principle.
pub struct CausesPrecedeEffects;

impl Axiom for CausesPrecedeEffects {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if produces_edge_exists(CausationConcept::Cause, CausationConcept::Effect) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CausesPrecedeEffects",
        "(Cause, Effect, Produces) edge encodes Reichenbach's temporal-priority principle",
        "Reichenbach (1956) Direction of Time"
    );
}

/// Reichenbach's common-cause principle — CommonCause is a Cause.
pub struct CommonCauseScreening;

impl Axiom for CommonCauseScreening {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if subsumption_pair_exists(CausationConcept::CommonCause, CausationConcept::Cause) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CommonCauseScreening",
        "CommonCause is-a Cause (Reichenbach's principle: correlations between non-causally-related events demand a common ancestor)",
        "Reichenbach (1956) Direction of Time §19"
    );
}

/// Pearl's do-operator — Intervention acts on Cause.
pub struct InterventionActsOnCause;

impl Axiom for InterventionActsOnCause {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if kinded_edge_exists(
            CausationConcept::Intervention,
            CausationConcept::Cause,
            CausationRelationKind::ActsOn,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "InterventionActsOnCause",
        "(Intervention, Cause, ActsOn) edge encodes Pearl's do(X) operator",
        "Pearl (2000) Causality §1.3; Woodward (2003) Making Things Happen Ch. 3"
    );
}

/// Hall/Mackie typology — the five canonical Cause kinds.
pub struct FiveCauseKinds;

impl Axiom for FiveCauseKinds {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let expected = [
            CausationConcept::SufficientCause,
            CausationConcept::NecessaryCause,
            CausationConcept::ProximateCause,
            CausationConcept::DistalCause,
            CausationConcept::CommonCause,
        ];
        let actual = direct_children_of_cause();
        let ok = actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FiveCauseKinds",
        "direct children of Cause are exactly {SufficientCause, NecessaryCause, ProximateCause, DistalCause, CommonCause}",
        "Hall (2004) Two Concepts of Causation; Mackie (1974) Cement of the Universe (INUS)"
    );
}

/// Lewis's reduction — CounterfactualDependence grounds Causation.
pub struct CounterfactualDependenceGroundsCausation;

impl Axiom for CounterfactualDependenceGroundsCausation {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if kinded_edge_exists(
            CausationConcept::CounterfactualDependence,
            CausationConcept::Cause,
            CausationRelationKind::Grounds,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CounterfactualDependenceGroundsCausation",
        "(CounterfactualDependence, Cause, Grounds) edge encodes Lewis's reduction: causation IS counterfactual dependence in nearest-world semantics",
        "Lewis (1973) J. Phil. 70 — counterfactual analysis of causation"
    );
}

// -----------------------------------------------------------------------------
// CauseRole — which epistemic role a concept plays.
// -----------------------------------------------------------------------------

/// Quality: what role does this concept play in a causal analysis?
/// Sourced-from tag per Hall/Mackie/Lewis/Pearl/Reichenbach lineage.
#[derive(Debug, Clone)]
pub struct CauseRole;

impl Quality for CauseRole {
    type Individual = CausationConcept;
    type Value = &'static str;

    fn get(&self, c: &CausationConcept) -> Option<&'static str> {
        use CausationConcept as C;
        Some(match c {
            C::Cause | C::Effect => "role",
            C::SufficientCause | C::NecessaryCause => "mackie-inus",
            C::ProximateCause | C::DistalCause => "mayr-typology",
            C::CommonCause => "reichenbach",
            C::Counterfactual | C::CounterfactualDependence => "lewis",
            C::Preemption | C::Overdetermination => "lewis-hall",
            C::Intervention | C::CausalGraph => "pearl-woodward",
            C::CausalChain => "structural",
        })
    }
}

impl Ontology for CausationOntology {
    type Cat = CausationCategory;
    type Qual = CauseRole;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(CausesPrecedeEffects));
        axioms.push(Box::new(CommonCauseScreening));
        axioms.push(Box::new(InterventionActsOnCause));
        axioms.push(Box::new(FiveCauseKinds));
        axioms.push(Box::new(CounterfactualDependenceGroundsCausation));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<CausationCategory>();
    }

    #[test]
    fn ontology_validates() {
        CausationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn causes_precede_effects_holds() {
        assert!(CausesPrecedeEffects.verify().is_ok());
    }

    #[test]
    fn common_cause_screening_holds() {
        assert!(CommonCauseScreening.verify().is_ok());
    }

    #[test]
    fn intervention_acts_on_cause_holds() {
        assert!(InterventionActsOnCause.verify().is_ok());
    }

    #[test]
    fn five_cause_kinds_holds() {
        assert!(FiveCauseKinds.verify().is_ok());
    }
}

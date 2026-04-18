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
    being: AbstractObject,

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

    axioms: {
        CausesPrecedeEffects: {
            source: "Reichenbach (1956) Direction of Time — temporal priority of causes",
            description: "the edge (Cause, Effect, Produces) exists, encoding Reichenbach's temporal-priority principle: causes precede effects in the causal graph direction",
            holds: {
                use pr4xis::category::Category;
                let morphs = CausationCategory::morphisms();
                morphs.iter().any(|r| {
                    r.from == CausationConcept::Cause
                        && r.to == CausationConcept::Effect
                        && r.kind == CausationRelationKind::Produces
                })
            },
        },
        CommonCauseScreening: {
            source: "Reichenbach (1956) §19 — common-cause principle",
            description: "CommonCause is declared as a Cause (via is_a), encoding Reichenbach's principle that correlations between non-causally-related events demand a common ancestor",
            holds: {
                use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
                CausationTaxonomy::relations()
                    .iter()
                    .any(|(c, p)| {
                        *c == CausationConcept::CommonCause && *p == CausationConcept::Cause
                    })
            },
        },
        InterventionActsOnCause: {
            source: "Pearl (2000) Causality §1.3 do-operator; Woodward (2003) Making Things Happen Ch. 3",
            description: "the edge (Intervention, Cause, ActsOn) exists, encoding Pearl's do(X) operator: an intervention fixes a cause's value independent of its upstream causes",
            holds: {
                use pr4xis::category::Category;
                CausationCategory::morphisms().iter().any(|r| {
                    r.from == CausationConcept::Intervention
                        && r.to == CausationConcept::Cause
                        && r.kind == CausationRelationKind::ActsOn
                })
            },
        },
        FiveCauseKinds: {
            source: "Hall (2004) two concepts; Mackie (1974) INUS",
            description: "the direct children of Cause are exactly the five typology entries: SufficientCause, NecessaryCause, ProximateCause, DistalCause, CommonCause",
            holds: {
                use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
                let rels = CausationTaxonomy::relations();
                let expected = [
                    CausationConcept::SufficientCause,
                    CausationConcept::NecessaryCause,
                    CausationConcept::ProximateCause,
                    CausationConcept::DistalCause,
                    CausationConcept::CommonCause,
                ];
                let actual: Vec<_> = rels
                    .iter()
                    .filter_map(|(c, p)| if *p == CausationConcept::Cause { Some(*c) } else { None })
                    .collect();
                actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c))
            },
        },
        CounterfactualDependenceGroundsCausation: {
            source: "Lewis (1973) J. Phil. 70 — counterfactual analysis of causation",
            description: "the edge (CounterfactualDependence, Cause, Grounds) exists, encoding Lewis's reduction: causation IS counterfactual dependence in nearest-world semantics",
            holds: {
                use pr4xis::category::Category;
                CausationCategory::morphisms().iter().any(|r| {
                    r.from == CausationConcept::CounterfactualDependence
                        && r.to == CausationConcept::Cause
                        && r.kind == CausationRelationKind::Grounds
                })
            },
        },
    },
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

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        CausationOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        CausationOntology::generated_domain_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<CausationCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        CausationOntology::validate().unwrap();
    }

    #[test]
    fn causes_precede_effects_holds() {
        assert!(CausesPrecedeEffects.holds());
    }

    #[test]
    fn common_cause_screening_holds() {
        assert!(CommonCauseScreening.holds());
    }

    #[test]
    fn intervention_acts_on_cause_holds() {
        assert!(InterventionActsOnCause.holds());
    }

    #[test]
    fn five_cause_kinds_holds() {
        assert!(FiveCauseKinds.holds());
    }
}

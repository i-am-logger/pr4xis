//! Qualitative process theory — how physical systems change WITHOUT numeric
//! simulation: processes are active or not (by their preconditions), and an
//! active process's influence sets the SIGN of a quantity's change, never a
//! rate. Complements the quantitative `Physics`/`Kinematics` ontologies in
//! this same `natural::physics` branch with the commonsense-reasoning layer
//! those numeric laws cannot answer alone ("why doesn't the trophy fit in
//! the suitcase?" needs no differential equation).
//!
//! # Literature
//!
//! - **Forbus (1984)** *Qualitative Process Theory*, Artificial
//!   Intelligence 24(1-3) — processes, quantities, preconditions,
//!   influences (§2), the sign-based derivative abstraction (§2.1, §2.3).
//! - **Hayes (1979)** *The Naive Physics Manifesto*, in Michie (ed.)
//!   *Expert Systems in the Micro-Electronic Age* — the commonsense
//!   ontology program: containment, support, and the claim that an
//!   unsupported object falls.
//! - **Hayes (1985)** *Naive Physics I: Ontology for Liquids*, in Hobbs &
//!   Moore (eds.) *Formal Theories of the Commonsense World* — the
//!   container/content size constraint (§3).
//! - **de Kleer & Brown (1984)** *A Qualitative Physics Based on
//!   Confluences*, Artificial Intelligence 24(1-3) — envisioning: process
//!   theory's sibling approach, cited for the shared qualitative-state
//!   framing (this ontology does not build a full envisionment graph —
//!   honestly scoped to the direct-influence case, see `process.rs`).
//! - **Levesque, Davis & Morgenstern (2012)** *The Winograd Schema
//!   Challenge*, KR 2012 — the commonsense pronoun-resolution task this
//!   ontology's Size/Containment axiom answers directly (§1.5b).
//! - **Sakaguchi, Bras, Bhagavatula & Choi (2020)** *WinoGrande*,
//!   AAAI 2020 — the "twin sentence" (adjective-swap) methodology
//!   `too_big`/`too_small` realize.

use pr4xis::ontology::{Axiom, Ontology, Quality};

use super::containment::{self, Individual, Size};
use super::process::{
    self, InfluenceInstance, InfluenceSign, PreconditionInstance, ProcessInstance,
};

pr4xis::ontology! {
    name: "QualitativeProcess",
    source: "Forbus (1984) Qualitative Process Theory; Hayes (1979) The Naive Physics Manifesto; Hayes (1985) Naive Physics I: Ontology for Liquids; de Kleer & Brown (1984) A Qualitative Physics Based on Confluences",

    concepts: [Individual, Quantity, Process, Precondition, Influence, Containment, Support],

    labels: {
        Individual: ("en", "Individual",
            "A physical object or substance participating in a qualitative model. Forbus (1984) §2; Hayes (1985) §2's 'piece of stuff'."),
        Quantity: ("en", "Quantity",
            "A magnitude tracked by qualitative SIGN (increasing/steady/decreasing) against a QuantitySpace, never a numeric rate. Forbus (1984) §2.1."),
        Process: ("en", "Process",
            "An activity that, when active, drives one or more Quantities' derivatives via its Influences. Active exactly when every Precondition is satisfied. Forbus (1984) §2.2."),
        Precondition: ("en", "Precondition",
            "A condition gating whether a Process is active — part of the process's own definition, not an external check. Forbus (1984) §2.2."),
        Influence: ("en", "Influence",
            "The direct effect (I+ or I-) an active Process exerts on one Quantity's derivative sign. Forbus (1984) §2.3."),
        Containment: ("en", "Containment",
            "A container-holds-content relation between two Individuals, constrained by relative Size (a container must be at least as large as its content). Hayes (1985) §3."),
        Support: ("en", "Support",
            "A rests-on/holds-up relation between two Individuals; without it, the supported Individual falls. Hayes (1979), the naive-physics support principle."),
    },

    // A Process's definition constitutively includes its Preconditions and
    // Influences (Forbus 1984 §2.2: "a process has ... a set of
    // preconditions ... and a set of influences") — has-a, not is-a.
    has_a: [
        (Process, Precondition),
        (Process, Influence),
    ],

    // An active Process CAUSES the sign change in the Quantity it
    // influences (Forbus 1984 §2.3).
    causes: [
        (Process, Quantity),
    ],
}

/// Quality: which founding paper a concept's formal treatment originates
/// from — Forbus's process-theory primitives vs. Hayes's naive-physics
/// containment/support claims. A real, total, citable classification (not
/// a stand-in), since the two research programs are historically and
/// formally distinct even though this ontology composes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheoryOrigin {
    ForbusProcessTheory,
    HayesNaivePhysics,
}

#[derive(Debug, Clone)]
pub struct Origin;

impl Quality for Origin {
    type Individual = QualitativeProcessConcept;
    type Value = TheoryOrigin;

    fn get(&self, c: &QualitativeProcessConcept) -> Option<TheoryOrigin> {
        use QualitativeProcessConcept as C;
        use TheoryOrigin::*;
        Some(match c {
            C::Individual | C::Quantity | C::Process | C::Precondition | C::Influence => {
                ForbusProcessTheory
            }
            C::Containment | C::Support => HayesNaivePhysics,
        })
    }
}

impl Ontology for QualitativeProcessOntology {
    type Cat = QualitativeProcessCategory;
    type Qual = Origin;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ProcessActiveRequiresAllPreconditions));
        axioms.push(Box::new(ActiveProcessInfluencesQuantityDerivative));
        axioms.push(Box::new(ContainerSizeAtLeastContentSize));
        axioms.push(Box::new(UnsupportedIndividualsFall));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: a Process is active exactly when every one of its Preconditions
/// is satisfied. Forbus (1984) §2.2.
pub struct ProcessActiveRequiresAllPreconditions;

impl Axiom for ProcessActiveRequiresAllPreconditions {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let all_satisfied = ProcessInstance {
            name: "test".into(),
            preconditions: vec![
                PreconditionInstance {
                    description: "a".into(),
                    satisfied: true,
                },
                PreconditionInstance {
                    description: "b".into(),
                    satisfied: true,
                },
            ],
            influences: vec![],
        };
        let one_fails = ProcessInstance {
            preconditions: vec![
                PreconditionInstance {
                    description: "a".into(),
                    satisfied: true,
                },
                PreconditionInstance {
                    description: "b".into(),
                    satisfied: false,
                },
            ],
            ..all_satisfied.clone()
        };
        if process::is_active(&all_satisfied) && !process::is_active(&one_fails) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ProcessActiveRequiresAllPreconditions",
        "a process is active iff every one of its preconditions is satisfied",
        "Forbus (1984) Qualitative Process Theory §2.2"
    );
}

pr4xis::register_axiom!(
    ProcessActiveRequiresAllPreconditions,
    "Forbus (1984) Qualitative Process Theory §2.2"
);

/// Axiom: an active Process's direct Influence sets the SIGN of the
/// influenced Quantity's derivative — I+ increasing, I- decreasing. Forbus
/// (1984) §2.3.
pub struct ActiveProcessInfluencesQuantityDerivative;

impl Axiom for ActiveProcessInfluencesQuantityDerivative {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let heating = ProcessInstance {
            name: "heating".into(),
            preconditions: vec![],
            influences: vec![InfluenceInstance {
                quantity: "temperature".into(),
                sign: InfluenceSign::Positive,
            }],
        };
        let cooling = ProcessInstance {
            name: "cooling".into(),
            preconditions: vec![],
            influences: vec![InfluenceInstance {
                quantity: "temperature".into(),
                sign: InfluenceSign::Negative,
            }],
        };
        let heating_ok = process::predicted_derivative(&heating, "temperature")
            == Some(process::DerivativeSign::Increasing);
        let cooling_ok = process::predicted_derivative(&cooling, "temperature")
            == Some(process::DerivativeSign::Decreasing);
        if heating_ok && cooling_ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ActiveProcessInfluencesQuantityDerivative",
        "an active process's I+/I- influence sets the influenced quantity's derivative sign",
        "Forbus (1984) Qualitative Process Theory §2.3"
    );
}

pr4xis::register_axiom!(
    ActiveProcessInfluencesQuantityDerivative,
    "Forbus (1984) Qualitative Process Theory §2.3"
);

/// Axiom: a container's Size must be at least as large as its content's —
/// the physical constraint licensing "won't fit" reasoning. Hayes (1985)
/// *Naive Physics I: Ontology for Liquids* §3.
pub struct ContainerSizeAtLeastContentSize;

impl Axiom for ContainerSizeAtLeastContentSize {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let suitcase = Individual {
            name: "suitcase".into(),
            size: Size::Large,
        };
        let trophy = Individual {
            name: "trophy".into(),
            size: Size::Small,
        };
        let small_suitcase = Individual {
            name: "small suitcase".into(),
            size: Size::Small,
        };
        let big_trophy = Individual {
            name: "big trophy".into(),
            size: Size::Large,
        };
        let fits_ok = containment::fits(&suitcase, &trophy);
        let refuses_when_too_big = !containment::fits(&small_suitcase, &big_trophy);
        if fits_ok && refuses_when_too_big {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ContainerSizeAtLeastContentSize",
        "a container's size must be at least as large as what it holds",
        "Hayes (1985) Naive Physics I: Ontology for Liquids §3"
    );
}

pr4xis::register_axiom!(
    ContainerSizeAtLeastContentSize,
    "Hayes (1985) Naive Physics I: Ontology for Liquids §3"
);

/// Axiom: an Individual with no Support falls. Hayes (1979) *The Naive
/// Physics Manifesto* — the foundational commonsense support principle.
pub struct UnsupportedIndividualsFall;

impl Axiom for UnsupportedIndividualsFall {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if containment::falls_without_support(false) && !containment::falls_without_support(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "UnsupportedIndividualsFall",
        "an individual with no support falls",
        "Hayes (1979) The Naive Physics Manifesto"
    );
}

pr4xis::register_axiom!(
    UnsupportedIndividualsFall,
    "Hayes (1979) The Naive Physics Manifesto"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<QualitativeProcessCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        QualitativeProcessOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn seven_concepts() {
        assert_eq!(QualitativeProcessConcept::variants().len(), 7);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn origin_is_total_and_correctly_partitioned() {
        let q = Origin;
        for c in QualitativeProcessConcept::variants() {
            assert!(q.get(&c).is_some(), "{c:?} has no Origin");
        }
        assert_eq!(
            q.get(&QualitativeProcessConcept::Process),
            Some(TheoryOrigin::ForbusProcessTheory)
        );
        assert_eq!(
            q.get(&QualitativeProcessConcept::Containment),
            Some(TheoryOrigin::HayesNaivePhysics)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn process_active_requires_all_preconditions_holds() {
        assert!(ProcessActiveRequiresAllPreconditions.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn active_process_influences_quantity_derivative_holds() {
        assert!(ActiveProcessInfluencesQuantityDerivative.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn container_size_at_least_content_size_holds() {
        assert!(ContainerSizeAtLeastContentSize.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn unsupported_individuals_fall_holds() {
        assert!(UnsupportedIndividualsFall.verify().is_ok());
    }

    // -------------------------------------------------------------------
    // 1.5a — material physics: for a Process with a precondition-satisfied
    // instance, the run's predicted effect equals the axiom's predicted
    // effect, for a range of process/influence combinations.
    // -------------------------------------------------------------------

    fn arb_influence_sign() -> impl Strategy<Value = InfluenceSign> {
        prop_oneof![Just(InfluenceSign::Positive), Just(InfluenceSign::Negative),]
    }

    proptest! {
        #[test]
        fn prop_active_process_prediction_matches_influence_sign(sign in arb_influence_sign()) {
            let p = ProcessInstance {
                name: "p".into(),
                preconditions: vec![],
                influences: vec![InfluenceInstance { quantity: "q".into(), sign }],
            };
            let predicted = process::predicted_derivative(&p, "q");
            let expected = match sign {
                InfluenceSign::Positive => process::DerivativeSign::Increasing,
                InfluenceSign::Negative => process::DerivativeSign::Decreasing,
            };
            prop_assert_eq!(predicted, Some(expected));
        }

        #[test]
        fn prop_inactive_process_never_predicts(sign in arb_influence_sign()) {
            let p = ProcessInstance {
                name: "p".into(),
                preconditions: vec![PreconditionInstance { description: "x".into(), satisfied: false }],
                influences: vec![InfluenceInstance { quantity: "q".into(), sign }],
            };
            prop_assert_eq!(process::predicted_derivative(&p, "q"), None);
        }
    }

    pr4xis::register_praxis_value!(
        prop_active_process_prediction_matches_influence_sign,
        Verifiable
    );
    pr4xis::register_praxis_value!(prop_inactive_process_never_predicts, Honest);

    // -------------------------------------------------------------------
    // 1.5b — the Winograd pair: the antecedent (and its big/small swap)
    // falls directly out of ContainerSizeAtLeastContentSize's ordering,
    // for every strict size mismatch, both directions.
    // -------------------------------------------------------------------

    fn arb_size() -> impl Strategy<Value = Size> {
        prop_oneof![Just(Size::Small), Just(Size::Medium), Just(Size::Large)]
    }

    proptest! {
        #[test]
        fn prop_winograd_antecedent_is_whichever_individual_exceeds_the_other(
            container_size in arb_size(),
            content_size in arb_size(),
        ) {
            let container = Individual { name: "container".into(), size: container_size };
            let content = Individual { name: "content".into(), size: content_size };
            if content_size > container_size {
                // Fails to fit: "content" is too big, "container" is too small.
                prop_assert!(containment::too_big(&container, &content).is_some());
                prop_assert!(containment::too_small(&container, &content).is_some());
                prop_assert_eq!(
                    containment::too_big(&container, &content).map(|i| i.name.clone()),
                    Some(content.name.clone())
                );
                prop_assert_eq!(
                    containment::too_small(&container, &content).map(|i| i.name.clone()),
                    Some(container.name.clone())
                );
            } else {
                // Fits: no failure to explain, no antecedent.
                prop_assert_eq!(containment::too_big(&container, &content), None);
                prop_assert_eq!(containment::too_small(&container, &content), None);
            }
        }
    }

    pr4xis::register_praxis_value!(
        prop_winograd_antecedent_is_whichever_individual_exceeds_the_other,
        Verifiable
    );
}

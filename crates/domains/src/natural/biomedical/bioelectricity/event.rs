//! Bioelectric signal causal events.
//!
//! Events in the bioelectric signal causal chain, from ion-channel opening
//! to anatomical change. Extracted from the main bioelectricity ontology
//! into its own module so the causal chain is internal to one ontology
//! (`feedback_one_ontology_per_module`).
//!
//! # Literature
//!
//! - **Hodgkin & Huxley (1952)** "A Quantitative Description of Membrane
//!   Current and its Application to Conduction and Excitation in Nerve",
//!   *J. Physiol.* 117(4):500–544 — the canonical ion-channel-opening →
//!   ion-flux → Vmem-change chain.
//! - **Levin (2014)** "Molecular bioelectricity: how endogenous voltage
//!   potentials control cell behavior and instruct pattern regulation
//!   in vivo", *Molecular Biology of the Cell* 25(24):3835–3850 — the
//!   pattern-formation / morphogenetic-instruction / anatomical-change
//!   chain in developmental and regenerative contexts.
//! - **Levin (2019)** "The Computational Boundary of a 'Self'", *Front.
//!   Psychol.* 10:2688 — gap-junction propagation as the bridge between
//!   single-cell Vmem and tissue-level pattern.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "BioelectricEvent",
    source: "Hodgkin & Huxley (1952) J. Physiol. 117(4); Levin (2014) Mol. Biol. Cell 25(24); Levin (2019) Front. Psychol. 10:2688",

    concepts: [
        IonChannelOpening,
        IonFlux,
        VmemChange,
        GapJunctionPropagation,
        PatternFormation,
        MorphogeneticInstruction,
        AnatomicalChange,
    ],

    labels: {
        IonChannelOpening: ("en", "Ion channel opening",
            "Hodgkin & Huxley (1952): an ion channel in the plasma membrane transitions to its open conformation, permitting ion flux."),
        IonFlux: ("en", "Ion flux",
            "Hodgkin & Huxley (1952): directed movement of ions through an open channel down their electrochemical gradient."),
        VmemChange: ("en", "Vmem change",
            "Hodgkin & Huxley (1952): change in transmembrane potential resulting from net ion flux."),
        GapJunctionPropagation: ("en", "Gap-junction propagation",
            "Levin (2019): voltage change spreads to neighboring cells via connexin channels."),
        PatternFormation: ("en", "Pattern formation",
            "Levin (2014): a coordinated Vmem pattern emerges across tissue."),
        MorphogeneticInstruction: ("en", "Morphogenetic instruction",
            "Levin (2014): the bioelectric pattern instructs downstream morphogenetic machinery (gene expression, cytoskeletal rearrangement)."),
        AnatomicalChange: ("en", "Anatomical change",
            "Levin (2014): the resulting anatomical outcome — growth, regeneration, or differentiation."),
    },

    // Causal chain per Hodgkin & Huxley (1952) for the molecular steps and
    // Levin (2014, 2019) for the tissue-and-above steps.
    causes: [
        (IonChannelOpening, IonFlux),
        (IonFlux, VmemChange),
        (VmemChange, GapJunctionPropagation),
        (GapJunctionPropagation, PatternFormation),
        (PatternFormation, MorphogeneticInstruction),
        (MorphogeneticInstruction, AnatomicalChange),
    ],
}

/// Quality: at which TAME scale does this event operate?
///
/// Levin (2019) — events at the start of the chain are molecular; events
/// at the end are organism-scale. The scale label here is the string used
/// in the TAME ladder (`tame::CompetencyLevel`).
#[derive(Debug, Clone)]
pub struct EventScale;

impl Quality for EventScale {
    type Individual = BioelectricEventConcept;
    type Value = &'static str;

    fn get(&self, ev: &BioelectricEventConcept) -> Option<&'static str> {
        Some(match ev {
            BioelectricEventConcept::IonChannelOpening => "molecular",
            BioelectricEventConcept::IonFlux => "molecular",
            BioelectricEventConcept::VmemChange => "cellular",
            BioelectricEventConcept::GapJunctionPropagation => "tissue",
            BioelectricEventConcept::PatternFormation => "tissue",
            BioelectricEventConcept::MorphogeneticInstruction => "organ",
            BioelectricEventConcept::AnatomicalChange => "organism",
        })
    }
}

impl Ontology for BioelectricEventOntology {
    type Cat = BioelectricEventCategory;
    type Qual = EventScale;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(IonChannelOpeningReachesAnatomicalChange));
        axioms
    }
}

/// Helper: does a Causation edge (direct or transitive) exist from
/// `cause` to `effect`?
fn causes(cause: BioelectricEventConcept, effect: BioelectricEventConcept) -> bool {
    BioelectricEventCategory::morphisms().iter().any(|m| {
        m.kind() == BioelectricEventRelationKind::Causation
            && m.source() == cause
            && m.target() == effect
    })
}

/// Axiom: ion-channel opening transitively causes anatomical change — the
/// full Hodgkin–Huxley → Levin chain is complete (no broken link).
pub struct IonChannelOpeningReachesAnatomicalChange;

impl Axiom for IonChannelOpeningReachesAnatomicalChange {
    fn verify(&self) -> Verdict {
        if causes(
            BioelectricEventConcept::IonChannelOpening,
            BioelectricEventConcept::AnatomicalChange,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "IonChannelOpeningReachesAnatomicalChange",
        "ion-channel opening transitively causes anatomical change through the full bioelectric chain",
        "Hodgkin & Huxley (1952); Levin (2014) Mol. Biol. Cell 25(24); Levin (2019) Front. Psychol. 10:2688"
    );
}

pr4xis::register_axiom!(
    IonChannelOpeningReachesAnatomicalChange,
    "Hodgkin & Huxley (1952); Levin (2014) Mol. Biol. Cell 25(24)"
);

/// Backward-compatibility re-exports for existing call sites (supports
/// glob imports). The legacy names were emitted by the old
/// `define_ontology!` macro; the new proc macro emits only the kinded
/// category — there is no separate Causation struct.
pub use BioelectricEventCategory as BioelectricSignalCausalGraph;
pub use BioelectricEventConcept as BioelectricSignalEvent;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;
    use pr4xis::category::laws::assert_category_laws;
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<BioelectricEventCategory>();
    }

    #[test]
    fn ontology_validates() {
        BioelectricEventOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn has_seven_events() {
        assert_eq!(BioelectricEventConcept::variants().len(), 7);
    }

    #[test]
    fn full_chain_reachable() {
        assert!(IonChannelOpeningReachesAnatomicalChange.verify().is_ok());
    }

    #[test]
    fn vmem_change_reaches_pattern_formation() {
        assert!(causes(
            BioelectricEventConcept::VmemChange,
            BioelectricEventConcept::PatternFormation,
        ));
    }

    #[test]
    fn ion_channel_opening_reaches_anatomical_change() {
        assert!(causes(
            BioelectricEventConcept::IonChannelOpening,
            BioelectricEventConcept::AnatomicalChange,
        ));
    }

    fn arb_event() -> impl Strategy<Value = BioelectricEventConcept> {
        proptest::sample::select(BioelectricEventConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in BioelectricEventCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in BioelectricEventOntology::axioms() {
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
        fn prop_event_scale_total(c in arb_event()) {
            prop_assert!(EventScale.get(&c).is_some());
        }

        #[test]
        fn prop_causation_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = BioelectricEventConcept::variants();
            for m in BioelectricEventCategory::morphisms() {
                if m.kind() == BioelectricEventRelationKind::Causation {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }
    }
}

//! Bioelectric signal causal events.
//!
//! Events in the bioelectric signal causal chain, from ion channel opening
//! to anatomical change. Extracted from the main bioelectricity ontology
//! into its own module so the causal chain is internal to one ontology
//! (previously it was expressed via the old macro's `causation: X for Y`
//! clause that spanned two entity types).
//!
//! Source: Levin (2014); Levin (2019).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "BioelectricEvent",
    source: "Levin (2014); Levin (2019)",
    being: Event,

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
        IonChannelOpening: ("en", "Ion channel opening", "Ion channel in the cell membrane opens, allowing ion flux."),
        IonFlux: ("en", "Ion flux", "Directed movement of ions through an open channel."),
        VmemChange: ("en", "Vmem change", "Change in membrane potential resulting from ion flux."),
        GapJunctionPropagation: ("en", "Gap junction propagation", "Voltage change spreads to neighboring cells via gap junctions."),
        PatternFormation: ("en", "Pattern formation", "Coordinated Vmem pattern emerges across tissue."),
        MorphogeneticInstruction: ("en", "Morphogenetic instruction", "Bioelectric pattern instructs downstream morphogenetic machinery."),
        AnatomicalChange: ("en", "Anatomical change", "Resulting anatomical outcome — growth, regeneration, or differentiation."),
    },

    // Causal chain per Levin (2014).
    causes: [
        (IonChannelOpening, IonFlux),
        (IonFlux, VmemChange),
        (VmemChange, GapJunctionPropagation),
        (GapJunctionPropagation, PatternFormation),
        (PatternFormation, MorphogeneticInstruction),
        (MorphogeneticInstruction, AnatomicalChange),
    ],
}

/// Quality: at which TAME level does this event operate?
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

/// Axiom: the bioelectric causal graph is asymmetric.
pub struct CausalAsymmetric;

impl Axiom for CausalAsymmetric {
    fn description(&self) -> &str {
        "bioelectric signal causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<BioelectricEventCausation>::new().holds()
    }
}
pr4xis::register_axiom!(CausalAsymmetric, "Levin (2014); Levin (2019).");

/// Axiom: no bioelectric event directly causes itself.
pub struct NoSelfCausation;

impl Axiom for NoSelfCausation {
    fn description(&self) -> &str {
        "no bioelectric event directly causes itself"
    }
    fn holds(&self) -> bool {
        causation::NoSelfCausation::<BioelectricEventCausation>::new().holds()
    }
}
pr4xis::register_axiom!(NoSelfCausation, "Levin (2014); Levin (2019).");

impl Ontology for BioelectricEventOntology {
    type Cat = BioelectricEventCategory;
    type Qual = EventScale;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(CausalAsymmetric), Box::new(NoSelfCausation)]
    }
}

pub use BioelectricEventCausation as BioelectricSignalCausalGraph;
/// Backward-compatibility re-exports for existing callers (supports glob imports).
pub use BioelectricEventConcept as BioelectricSignalEvent;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;

    #[test]
    fn has_seven_events() {
        assert_eq!(BioelectricEventConcept::variants().len(), 7);
    }

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<BioelectricEventCategory>().unwrap();
    }

    #[test]
    fn causal_chain_is_asymmetric() {
        assert!(CausalAsymmetric.holds());
    }
}

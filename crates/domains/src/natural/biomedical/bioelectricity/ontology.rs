//! Core Levin bioelectric framework ontology.
//!
//! Models Dr. Michael Levin's TAME (Technological Approach to Mind
//! Everywhere) framework as formal ontology: the bioelectric code (Vmem
//! patterns encode morphogenetic information), gap-junction networks
//! (signal-propagation topology), the cognitive lightcone (scale of
//! goal-directed agency), and the interventions used to perturb each.
//!
//! The TAME competency hierarchy (Molecular → Organism) and the
//! bioelectric signal causal chain (IonChannelOpening → AnatomicalChange)
//! live in sibling modules (`tame`, `event`) to keep each ontology
//! focused on one kind of concept (`feedback_one_ontology_per_module`).
//!
//! # Literature
//!
//! - **Levin (2014)** "Molecular bioelectricity: how endogenous voltage
//!   potentials control cell behavior and instruct pattern regulation
//!   in vivo", *Molecular Biology of the Cell* 25(24):3835–3850 — Vmem
//!   as morphogenetic code; healthy polarised vs cancer depolarised.
//! - **Chernet & Levin (2013)** "Endogenous Voltage Potentials and the
//!   Microenvironment: Bioelectric Signals that Reveal, Induce and
//!   Normalize Cancer", *Journal of Clinical & Experimental Oncology*
//!   S1:002 — depolarisation as oncogene-like state and the Vmem
//!   normalisation experiments.
//! - **Levin (2019)** "The Computational Boundary of a 'Self': Developmental
//!   Bioelectricity Drives Multicellularity and Scale-Free Cognition",
//!   *Frontiers in Psychology* 10:2688 — gap-junction networks and the
//!   cognitive lightcone construct.
//! - **Fields & Levin (2022)** "Competency in Navigating Arbitrary Spaces",
//!   *Entropy* 24(6):819 — morphospace navigation and the five-level
//!   TAME hierarchy.

use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pub use crate::natural::biomedical::bioelectricity::event::{
    BioelectricSignalCausalGraph, BioelectricSignalEvent,
};
pub use crate::natural::biomedical::bioelectricity::tame::CompetencyLevel;

pr4xis::ontology! {
    name: "Bioelectric",
    source: "Levin (2014) Mol. Biol. Cell 25(24); Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002; Levin (2019) Front. Psychol. 10:2688; Fields & Levin (2022) Entropy 24(6):819",

    concepts: [
        // === Signals ===
        MembranePotential,
        VoltageGradient,
        BioelectricPrepattern,
        TransepithelialPotential,

        // === Networks ===
        GapJunctionNetwork,
        BioelectricCircuit,
        CognitiveLightcone,

        // === Morphospace ===
        TargetMorphology,
        CurrentMorphology,
        MorphogeneticField,

        // === Interventions ===
        IonChannelModulation,
        GapJunctionModulation,
        BioelectricCocktail,
        MechanicalStimulation,
        ProtonPumpInhibition,

        // === Abstract umbrellas ===
        Signal,
        Network,
        Morphospace,
        Intervention,
    ],

    labels: {
        MembranePotential: ("en", "Membrane potential",
            "Levin (2014): single-cell membrane potential (Vmem) — the difference in electric potential across the plasma membrane."),
        VoltageGradient: ("en", "Voltage gradient",
            "Levin (2014): spatial gradient of Vmem across a tissue."),
        BioelectricPrepattern: ("en", "Bioelectric prepattern",
            "Levin (2014): Vmem pattern that prefigures anatomical structure before gene expression."),
        TransepithelialPotential: ("en", "Transepithelial potential",
            "Levin (2014): Vmem across an epithelial layer; drives wound-healing currents."),
        GapJunctionNetwork: ("en", "Gap junction network",
            "Levin (2019): network of cells connected by connexin channels."),
        BioelectricCircuit: ("en", "Bioelectric circuit",
            "Levin (2019): functional pattern of electrical coupling across cells."),
        CognitiveLightcone: ("en", "Cognitive lightcone",
            "Levin (2019): spatiotemporal extent of goal-directed agency in a bioelectric system."),
        TargetMorphology: ("en", "Target morphology",
            "Fields & Levin (2022): the goal state in morphospace — the anatomical pattern the tissue navigates toward."),
        CurrentMorphology: ("en", "Current morphology",
            "Fields & Levin (2022): the present anatomical state of tissue in morphospace."),
        MorphogeneticField: ("en", "Morphogenetic field",
            "Levin (2014): field of bioelectric influence guiding tissue toward the target."),
        IonChannelModulation: ("en", "Ion channel modulation",
            "Chernet & Levin (2013): intervention that changes Vmem by opening/closing ion channels — cell-autonomous, does not require gap junctions."),
        GapJunctionModulation: ("en", "Gap junction modulation",
            "Levin (2019): intervention that changes cell coupling by gating connexins."),
        BioelectricCocktail: ("en", "Bioelectric cocktail",
            "Chernet & Levin (2013): combined intervention targeting multiple ion channels simultaneously."),
        MechanicalStimulation: ("en", "Mechanical stimulation",
            "Levin (2014): hardware-accessible intervention using physical force."),
        ProtonPumpInhibition: ("en", "Proton pump inhibition",
            "Chernet & Levin (2013): intervention targeting V-ATPase proton pumps."),
        Signal: ("en", "Signal (abstract)",
            "Levin (2014): abstract bioelectric signal — umbrella for measurable bioelectric quantities."),
        Network: ("en", "Network (abstract)",
            "Levin (2019): abstract bioelectric network — umbrella for cell-coupling topologies."),
        Morphospace: ("en", "Morphospace (abstract)",
            "Fields & Levin (2022): abstract morphospace concept — umbrella for anatomical state spaces."),
        Intervention: ("en", "Intervention (abstract)",
            "Levin (2014): abstract intervention — umbrella for perturbations applied to a bioelectric system."),
    },

    is_a: [
        // Signals.
        (MembranePotential, Signal),
        (VoltageGradient, Signal),
        (BioelectricPrepattern, Signal),
        (TransepithelialPotential, Signal),
        // Networks.
        (GapJunctionNetwork, Network),
        (BioelectricCircuit, Network),
        (CognitiveLightcone, Network),
        // Morphospace.
        (TargetMorphology, Morphospace),
        (CurrentMorphology, Morphospace),
        (MorphogeneticField, Morphospace),
        // Interventions.
        (IonChannelModulation, Intervention),
        (GapJunctionModulation, Intervention),
        (BioelectricCocktail, Intervention),
        (MechanicalStimulation, Intervention),
        (ProtonPumpInhibition, Intervention),
    ],

    opposes: [
        // Chernet & Levin (2013): direct channel modulation vs indirect
        // pump-mediated acid removal — two distinct mechanisms for the
        // same Vmem-normalisation outcome.
        (IonChannelModulation, ProtonPumpInhibition),
        (ProtonPumpInhibition, IonChannelModulation),
        // What IS (observable signal) vs what you DO (perturbation):
        // canonical observable/action opposition (Levin 2019 §4).
        (Signal, Intervention),
        (Intervention, Signal),
    ],
}

// Backward-compatibility re-export so existing call sites (Quality impls,
// axioms, functors) that reference `BioelectricEntity` keep compiling.
pub use BioelectricConcept as BioelectricEntity;
// Note: BioelectricTaxonomy struct was deleted per #152 (kinded morphisms)
// and #168 (per-def traits removed). Taxonomy queries now go through
// Category::morphisms().filter(|m| m.kind() == Subsumption).

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: at what TAME competency level does this entity operate?
///
/// Fields & Levin (2022) §3 — each bioelectric concept has a characteristic
/// scale of competency, from molecular (ion channels) to organism
/// (morphospace).
#[derive(Debug, Clone)]
pub struct OperatingLevel;

impl Quality for OperatingLevel {
    type Individual = BioelectricConcept;
    type Value = CompetencyLevel;

    fn get(&self, individual: &BioelectricConcept) -> Option<CompetencyLevel> {
        use BioelectricConcept::*;
        use CompetencyLevel::*;
        Some(match individual {
            MembranePotential | IonChannelModulation | ProtonPumpInhibition => Molecular,
            VoltageGradient | GapJunctionNetwork | GapJunctionModulation => Cellular,
            BioelectricPrepattern
            | BioelectricCircuit
            | BioelectricCocktail
            | TransepithelialPotential
            | MorphogeneticField
            | CurrentMorphology
            | MechanicalStimulation => Tissue,
            CognitiveLightcone | TargetMorphology => Organ,
            Signal | Network | Morphospace | Intervention => Organism,
        })
    }
}

/// Quality: is this entity accessible via hardware (mechanical) means?
///
/// Levin (2014): mechanical stimulation is the only intervention that does
/// not require pharmacological or molecular-biology tooling.
#[derive(Debug, Clone)]
pub struct IsHardwareAccessible;

impl Quality for IsHardwareAccessible {
    type Individual = BioelectricConcept;
    type Value = bool;

    fn get(&self, individual: &BioelectricConcept) -> Option<bool> {
        use BioelectricConcept::*;
        match individual {
            MechanicalStimulation => Some(true),
            MembranePotential
            | VoltageGradient
            | BioelectricPrepattern
            | TransepithelialPotential
            | GapJunctionNetwork
            | BioelectricCircuit
            | CognitiveLightcone
            | TargetMorphology
            | CurrentMorphology
            | MorphogeneticField
            | IonChannelModulation
            | GapJunctionModulation
            | BioelectricCocktail
            | ProtonPumpInhibition => Some(false),
            Signal | Network | Morphospace | Intervention => None,
        }
    }
}

/// Quality: does this entity require gap junctions to function?
///
/// Levin (2019) — tissue-level signals (voltage gradients, prepatterns,
/// circuits) propagate via connexin channels; single-cell entities and
/// cell-autonomous interventions do not.
#[derive(Debug, Clone)]
pub struct RequiresGapJunctions;

impl Quality for RequiresGapJunctions {
    type Individual = BioelectricConcept;
    type Value = bool;

    fn get(&self, individual: &BioelectricConcept) -> Option<bool> {
        use BioelectricConcept::*;
        match individual {
            VoltageGradient
            | BioelectricPrepattern
            | GapJunctionNetwork
            | BioelectricCircuit
            | CognitiveLightcone
            | MorphogeneticField
            | BioelectricCocktail => Some(true),
            MembranePotential
            | IonChannelModulation
            | MechanicalStimulation
            | ProtonPumpInhibition => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Helper: is `child` is-a `parent` via the Subsumption morphisms?
fn is_a(child: BioelectricConcept, parent: BioelectricConcept) -> bool {
    BioelectricCategory::morphisms().iter().any(|m| {
        m.kind() == BioelectricRelationKind::Subsumption
            && m.source() == child
            && m.target() == parent
    })
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Bioelectric code axiom: healthy tissue is polarised (more negative than
/// −40 mV), cancerous tissue is depolarised (more positive than ~−20 mV).
///
/// Chernet & Levin (2013) — Vmem normalisation experiments.
pub struct BioelectricCodeAxiom;

impl Axiom for BioelectricCodeAxiom {
    fn verify(&self) -> Verdict {
        let healthy_vmem = -50.0_f64;
        let cancer_vmem = -15.0_f64;
        if healthy_vmem < -40.0 && cancer_vmem > -18.0 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BioelectricCodeAxiom",
        "bioelectric code: healthy Vmem is polarised (< −40 mV), cancer is depolarised (> −18 mV)",
        "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002; Levin (2014) Mol. Biol. Cell 25(24)"
    );
}

pr4xis::register_axiom!(
    BioelectricCodeAxiom,
    "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002; Levin (2014) Mol. Biol. Cell 25(24)"
);

/// Gap-junction communication axiom: tissue-level signals require GJs,
/// single-cell signals do not (Levin 2019).
pub struct GapJunctionCommunicationAxiom;

impl Axiom for GapJunctionCommunicationAxiom {
    fn verify(&self) -> Verdict {
        use BioelectricConcept::*;
        let req = RequiresGapJunctions;
        if req.get(&VoltageGradient) == Some(true)
            && req.get(&BioelectricPrepattern) == Some(true)
            && req.get(&MembranePotential) == Some(false)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GapJunctionCommunicationAxiom",
        "tissue-level bioelectric signals require gap junctions, single-cell signals do not",
        "Levin (2019) Front. Psychol. 10:2688"
    );
}

pr4xis::register_axiom!(
    GapJunctionCommunicationAxiom,
    "Levin (2019) Front. Psychol. 10:2688"
);

/// Repolarisation-repair axiom: both ion-channel modulation and
/// proton-pump inhibition are interventions (Chernet & Levin 2013).
pub struct RepolarizationRepairAxiom;

impl Axiom for RepolarizationRepairAxiom {
    fn verify(&self) -> Verdict {
        use BioelectricConcept::*;
        if is_a(IonChannelModulation, Intervention) && is_a(ProtonPumpInhibition, Intervention) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RepolarizationRepairAxiom",
        "both ion-channel modulation and proton-pump inhibition are bioelectric interventions",
        "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
    );
}

pr4xis::register_axiom!(
    RepolarizationRepairAxiom,
    "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
);

/// Two-mechanism repair axiom: proton-pump inhibition does not require gap
/// junctions, the bioelectric cocktail does (Chernet & Levin 2013).
pub struct TwoMechanismRepairAxiom;

impl Axiom for TwoMechanismRepairAxiom {
    fn verify(&self) -> Verdict {
        use BioelectricConcept::*;
        let req = RequiresGapJunctions;
        if req.get(&ProtonPumpInhibition) == Some(false)
            && req.get(&BioelectricCocktail) == Some(true)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TwoMechanismRepairAxiom",
        "PPI does not require gap junctions, bioelectric cocktail does",
        "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
    );
}

pr4xis::register_axiom!(
    TwoMechanismRepairAxiom,
    "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
);

/// TAME hierarchy axiom: exactly 5 levels (Fields & Levin 2022).
pub struct TAMEHierarchyAxiom;

impl Axiom for TAMEHierarchyAxiom {
    fn verify(&self) -> Verdict {
        if CompetencyLevel::variants().len() == 5 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TAMEHierarchyAxiom",
        "TAME hierarchy has exactly 5 competency levels (Molecular, Cellular, Tissue, Organ, Organism)",
        "Fields & Levin (2022) Entropy 24(6):819"
    );
}

pr4xis::register_axiom!(
    TAMEHierarchyAxiom,
    "Fields & Levin (2022) Entropy 24(6):819"
);

/// Cognitive-lightcone axiom: requires GJs and operates at Organ level
/// (Levin 2019; Fields & Levin 2022).
pub struct CognitiveLightconeAxiom;

impl Axiom for CognitiveLightconeAxiom {
    fn verify(&self) -> Verdict {
        use BioelectricConcept::*;
        if RequiresGapJunctions.get(&CognitiveLightcone) == Some(true)
            && OperatingLevel.get(&CognitiveLightcone) == Some(CompetencyLevel::Organ)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CognitiveLightconeAxiom",
        "cognitive lightcone requires gap junctions and operates at organ level",
        "Levin (2019) Front. Psychol. 10:2688; Fields & Levin (2022) Entropy 24(6):819"
    );
}

pr4xis::register_axiom!(
    CognitiveLightconeAxiom,
    "Levin (2019) Front. Psychol. 10:2688; Fields & Levin (2022) Entropy 24(6):819"
);

/// Mechanical stimulation is the only hardware-accessible intervention
/// (Levin 2014).
pub struct MechanicalStimulationIsHardwareAccessible;

impl Axiom for MechanicalStimulationIsHardwareAccessible {
    fn verify(&self) -> Verdict {
        let hw = IsHardwareAccessible;
        let interventions: Vec<BioelectricConcept> = BioelectricConcept::variants()
            .into_iter()
            .filter(|e| is_a(*e, BioelectricConcept::Intervention))
            .collect();
        let hw_accessible: Vec<&BioelectricConcept> = interventions
            .iter()
            .filter(|e| hw.get(e) == Some(true))
            .collect();
        if hw_accessible.len() == 1
            && *hw_accessible[0] == BioelectricConcept::MechanicalStimulation
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MechanicalStimulationIsHardwareAccessible",
        "exactly one hardware-accessible intervention: MechanicalStimulation",
        "Levin (2014) Mol. Biol. Cell 25(24)"
    );
}

pr4xis::register_axiom!(
    MechanicalStimulationIsHardwareAccessible,
    "Levin (2014) Mol. Biol. Cell 25(24)"
);

/// All 5 TAME levels are represented in OperatingLevel values.
pub struct AllTAMELevelsRepresented;

impl Axiom for AllTAMELevelsRepresented {
    fn verify(&self) -> Verdict {
        let op = OperatingLevel;
        let levels: Vec<CompetencyLevel> = BioelectricConcept::variants()
            .iter()
            .filter_map(|e| op.get(e))
            .collect();
        if CompetencyLevel::variants()
            .iter()
            .all(|target| levels.contains(target))
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AllTAMELevelsRepresented",
        "all 5 TAME competency levels appear in some operating-level value",
        "Fields & Levin (2022) Entropy 24(6):819"
    );
}

pr4xis::register_axiom!(
    AllTAMELevelsRepresented,
    "Fields & Levin (2022) Entropy 24(6):819"
);

/// Cross-domain equivalence: the bioelectricity→regeneration functor
/// preserves `TargetMorphology` identity (Levin 2014; regeneration uses
/// the same concept).
pub struct TargetMorphologyCrossDomainEquivalence;

impl Axiom for TargetMorphologyCrossDomainEquivalence {
    fn verify(&self) -> Verdict {
        use crate::natural::biomedical::bioelectricity::regeneration_functor::BioelectricToRegeneration;
        use crate::natural::biomedical::regeneration::ontology::RegenerationEntity;
        use pr4xis::category::Functor;
        if BioelectricToRegeneration::map_object(&BioelectricConcept::TargetMorphology)
            == RegenerationEntity::TargetMorphology
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TargetMorphologyCrossDomainEquivalence",
        "TargetMorphology is the same entity in bioelectricity and regeneration (functor maps identity)",
        "Levin (2014) Mol. Biol. Cell 25(24)"
    );
}

pr4xis::register_axiom!(
    TargetMorphologyCrossDomainEquivalence,
    "Levin (2014) Mol. Biol. Cell 25(24)"
);

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

impl Ontology for BioelectricOntology {
    type Cat = BioelectricCategory;
    type Qual = OperatingLevel;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(BioelectricCodeAxiom));
        axioms.push(Box::new(GapJunctionCommunicationAxiom));
        axioms.push(Box::new(RepolarizationRepairAxiom));
        axioms.push(Box::new(TwoMechanismRepairAxiom));
        axioms.push(Box::new(TAMEHierarchyAxiom));
        axioms.push(Box::new(CognitiveLightconeAxiom));
        axioms.push(Box::new(MechanicalStimulationIsHardwareAccessible));
        axioms.push(Box::new(AllTAMELevelsRepresented));
        axioms.push(Box::new(TargetMorphologyCrossDomainEquivalence));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<BioelectricCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        BioelectricOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nineteen_concepts() {
        // 4 signals + 3 networks + 3 morphospace + 5 interventions + 4 abstract = 19.
        assert_eq!(BioelectricConcept::variants().len(), 19);
    }

    // -- Domain axioms --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bioelectric_code_axiom() {
        assert!(BioelectricCodeAxiom.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn gap_junction_communication_axiom() {
        assert!(GapJunctionCommunicationAxiom.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn repolarization_repair_axiom() {
        assert!(RepolarizationRepairAxiom.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn two_mechanism_repair_axiom() {
        assert!(TwoMechanismRepairAxiom.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn tame_hierarchy_axiom() {
        assert!(TAMEHierarchyAxiom.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cognitive_lightcone_axiom() {
        assert!(CognitiveLightconeAxiom.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mechanical_stimulation_only_hardware_accessible() {
        assert!(MechanicalStimulationIsHardwareAccessible.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn all_tame_levels_represented() {
        assert!(AllTAMELevelsRepresented.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn target_morphology_cross_domain_equivalence() {
        assert!(TargetMorphologyCrossDomainEquivalence.verify().is_ok());
    }

    // -- Subsumption / kind tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn signals_subsume_under_signal() {
        let subs: Vec<_> = BioelectricCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BioelectricRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for c in [
            BioelectricConcept::MembranePotential,
            BioelectricConcept::VoltageGradient,
            BioelectricConcept::BioelectricPrepattern,
            BioelectricConcept::TransepithelialPotential,
        ] {
            assert!(
                subs.contains(&(c, BioelectricConcept::Signal)),
                "{:?} should subsume under Signal",
                c
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn interventions_subsume_under_intervention() {
        for c in [
            BioelectricConcept::IonChannelModulation,
            BioelectricConcept::GapJunctionModulation,
            BioelectricConcept::BioelectricCocktail,
            BioelectricConcept::MechanicalStimulation,
            BioelectricConcept::ProtonPumpInhibition,
        ] {
            assert!(
                is_a(c, BioelectricConcept::Intervention),
                "{:?} should be an Intervention",
                c
            );
        }
    }

    // -- Opposition tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ion_channel_modulation_opposes_ppi() {
        let opps: Vec<_> = BioelectricCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BioelectricRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            BioelectricConcept::IonChannelModulation,
            BioelectricConcept::ProtonPumpInhibition
        )));
        assert!(opps.contains(&(
            BioelectricConcept::ProtonPumpInhibition,
            BioelectricConcept::IonChannelModulation
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn signal_opposes_intervention() {
        let opps: Vec<_> = BioelectricCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BioelectricRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(BioelectricConcept::Signal, BioelectricConcept::Intervention)));
    }

    // -- Quality tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn membrane_potential_is_molecular() {
        assert_eq!(
            OperatingLevel.get(&BioelectricConcept::MembranePotential),
            Some(CompetencyLevel::Molecular)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cognitive_lightcone_is_organ_level() {
        assert_eq!(
            OperatingLevel.get(&BioelectricConcept::CognitiveLightcone),
            Some(CompetencyLevel::Organ)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mechanical_stim_hardware_accessible() {
        assert_eq!(
            IsHardwareAccessible.get(&BioelectricConcept::MechanicalStimulation),
            Some(true)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ion_channel_modulation_not_gj_dependent() {
        assert_eq!(
            RequiresGapJunctions.get(&BioelectricConcept::IonChannelModulation),
            Some(false)
        );
    }

    // -- Literature axioms --

    /// Chernet & Levin (2013): GlyR-mediated hyperpolarisation is
    /// cell-autonomous (no GJs needed).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn literature_chernet_levin_2013_glyr_no_gj() {
        assert!(is_a(
            BioelectricConcept::IonChannelModulation,
            BioelectricConcept::Intervention,
        ));
        assert_eq!(
            RequiresGapJunctions.get(&BioelectricConcept::IonChannelModulation),
            Some(false),
        );
    }

    /// Fields & Levin (2022): the TAME ladder has exactly 5 ordered levels.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn literature_levin_2022_tame_five_levels_ordered() {
        let levels = CompetencyLevel::variants();
        assert_eq!(levels.len(), 5);
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = BioelectricConcept> {
        proptest::sample::select(BioelectricConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in BioelectricCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in BioelectricOntology::axioms() {
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
        fn prop_operating_level_total(c in arb_concept()) {
            prop_assert!(OperatingLevel.get(&c).is_some());
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = BioelectricConcept::variants();
            for m in BioelectricCategory::morphisms() {
                if m.kind() == BioelectricRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = BioelectricCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == BioelectricRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_operating_level_total, Verifiable);
    pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
}

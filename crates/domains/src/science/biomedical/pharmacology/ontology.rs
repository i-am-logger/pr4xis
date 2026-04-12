//! Bioelectric pharmacology ontology.
//!
//! Models bioelectric pharmacology as a formal ontology: drug classes, specific
//! agents, molecular targets, and bioelectric effects — plus the causal graph
//! from drug administration through Vmem shift to collective reprogramming.
//!
//! Key references:
//! - Kofman & Levin 2024: Bioelectric pharmacology of cancer
//! - Levin 2023: Morphoceuticals — drugs targeting anatomical outcomes
//! - Adams & Levin 2013: Vmem manipulation via ion channel/pump cocktails
//! - Chernet & Levin 2013: depolarization → oncogene-like transformation

use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Pharmacology Entity
// ---------------------------------------------------------------------------

/// Every entity in the bioelectric pharmacology domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PharmacologyEntity {
    // Drug classes
    IonChannelModulator,
    GapJunctionModulator,
    VoltageGatedBlocker,
    VoltageGatedOpener,
    MechanosensitiveModulator,
    ProtonPumpInhibitor,
    Morphoceutical,

    // Specific agents
    Ivermectin,    // GlyR agonist — Cl- influx → hyperpolarization (Levin)
    Decamethonium, // Depolarizing neuromuscular blocker
    Glibenclamide, // KATP blocker → depolarization
    Minoxidil,     // KATP opener → hyperpolarization
    Omeprazole,    // Proton pump inhibitor (acid suppression, not morphoceutical)

    // Targets
    IonChannel,
    GapJunction,
    Transporter,
    Receptor,

    // Effects
    Hyperpolarization,
    Depolarization,
    GapJunctionOpening,
    GapJunctionClosing,
    AntiInflammatory,

    // Abstract categories
    DrugClass,
    Agent,
    Target,
    Effect,
}

impl Entity for PharmacologyEntity {
    fn variants() -> Vec<Self> {
        use PharmacologyEntity::*;
        vec![
            // Drug classes
            IonChannelModulator,
            GapJunctionModulator,
            VoltageGatedBlocker,
            VoltageGatedOpener,
            MechanosensitiveModulator,
            ProtonPumpInhibitor,
            Morphoceutical,
            // Agents
            Ivermectin,
            Decamethonium,
            Glibenclamide,
            Minoxidil,
            Omeprazole,
            // Targets
            IonChannel,
            GapJunction,
            Transporter,
            Receptor,
            // Effects
            Hyperpolarization,
            Depolarization,
            GapJunctionOpening,
            GapJunctionClosing,
            AntiInflammatory,
            // Abstract
            DrugClass,
            Agent,
            Target,
            Effect,
        ]
    }
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for pharmacological entities.
///
/// Drug classes → DrugClass, agents → Agent, targets → Target, effects → Effect.
/// Specific agents also subsume under their drug class.
pub struct PharmacologyTaxonomy;

impl TaxonomyDef for PharmacologyTaxonomy {
    type Entity = PharmacologyEntity;

    fn relations() -> Vec<(PharmacologyEntity, PharmacologyEntity)> {
        use PharmacologyEntity::*;
        vec![
            // Drug classes is-a DrugClass
            (IonChannelModulator, DrugClass),
            (GapJunctionModulator, DrugClass),
            (VoltageGatedBlocker, DrugClass),
            (VoltageGatedOpener, DrugClass),
            (MechanosensitiveModulator, DrugClass),
            (ProtonPumpInhibitor, DrugClass),
            (Morphoceutical, DrugClass),
            // Agents is-a Agent
            (Ivermectin, Agent),
            (Decamethonium, Agent),
            (Glibenclamide, Agent),
            (Minoxidil, Agent),
            (Omeprazole, Agent),
            // Agents is-a their drug class
            (Ivermectin, IonChannelModulator),    // GlyR agonist
            (Decamethonium, IonChannelModulator), // nAChR agonist
            (Glibenclamide, VoltageGatedBlocker), // KATP blocker
            (Minoxidil, VoltageGatedOpener),      // KATP opener
            (Omeprazole, ProtonPumpInhibitor),    // H+/K+-ATPase inhibitor
            // Targets is-a Target
            (IonChannel, Target),
            (GapJunction, Target),
            (Transporter, Target),
            (Receptor, Target),
            // Effects is-a Effect
            (Hyperpolarization, Effect),
            (Depolarization, Effect),
            (GapJunctionOpening, Effect),
            (GapJunctionClosing, Effect),
            (AntiInflammatory, Effect),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph (PharmacologyEvent)
// ---------------------------------------------------------------------------

/// Causal events in bioelectric pharmacology.
///
/// Two causal chains:
/// 1. Drug → target binding → channel state → ion flux → Vmem shift → signaling
/// 2. GJ modulator → GJ state change → network change → collective reprogramming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PharmacologyEvent {
    DrugAdministration,
    TargetBinding,
    ChannelStateChange,
    IonFluxChange,
    VmemShift,
    DownstreamSignaling,
    GJModulatorBinding,
    GapJunctionStateChange,
    BioelectricNetworkChange,
    CollectiveReprogramming,
}

impl Entity for PharmacologyEvent {
    fn variants() -> Vec<Self> {
        use PharmacologyEvent::*;
        vec![
            DrugAdministration,
            TargetBinding,
            ChannelStateChange,
            IonFluxChange,
            VmemShift,
            DownstreamSignaling,
            GJModulatorBinding,
            GapJunctionStateChange,
            BioelectricNetworkChange,
            CollectiveReprogramming,
        ]
    }
}

/// Causal definition for pharmacological events.
pub struct PharmacologyCauses;

impl CausalDef for PharmacologyCauses {
    type Entity = PharmacologyEvent;

    fn relations() -> Vec<(PharmacologyEvent, PharmacologyEvent)> {
        use PharmacologyEvent::*;
        vec![
            // Chain 1: classical pharmacology → Vmem
            (DrugAdministration, TargetBinding),
            (TargetBinding, ChannelStateChange),
            (ChannelStateChange, IonFluxChange),
            (IonFluxChange, VmemShift),
            (VmemShift, DownstreamSignaling),
            // Chain 2: gap junction modulation → collective reprogramming
            (GJModulatorBinding, GapJunctionStateChange),
            (GapJunctionStateChange, BioelectricNetworkChange),
            (BioelectricNetworkChange, CollectiveReprogramming),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

/// A morphism in the pharmacology category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PharmacologyRelation {
    pub from: PharmacologyEntity,
    pub to: PharmacologyEntity,
}

impl Relationship for PharmacologyRelation {
    type Object = PharmacologyEntity;

    fn source(&self) -> PharmacologyEntity {
        self.from
    }

    fn target(&self) -> PharmacologyEntity {
        self.to
    }
}

/// Discrete category over pharmacology entities (full morphisms: all pairs).
pub struct PharmacologyCategory;

impl Category for PharmacologyCategory {
    type Object = PharmacologyEntity;
    type Morphism = PharmacologyRelation;

    fn identity(obj: &PharmacologyEntity) -> PharmacologyRelation {
        PharmacologyRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &PharmacologyRelation, g: &PharmacologyRelation) -> Option<PharmacologyRelation> {
        if f.to != g.from {
            return None;
        }
        Some(PharmacologyRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<PharmacologyRelation> {
        let variants = PharmacologyEntity::variants();
        let mut morphisms = Vec::new();
        for &from in &variants {
            for &to in &variants {
                morphisms.push(PharmacologyRelation { from, to });
            }
        }
        morphisms
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: what target does an agent act on?
#[derive(Debug, Clone)]
pub struct DrugTarget;

impl Quality for DrugTarget {
    type Individual = PharmacologyEntity;
    type Value = PharmacologyEntity;

    fn get(&self, individual: &PharmacologyEntity) -> Option<PharmacologyEntity> {
        use PharmacologyEntity::*;
        match individual {
            Ivermectin => Some(Receptor),      // GlyR (glycine receptor)
            Decamethonium => Some(Receptor),   // nAChR
            Glibenclamide => Some(IonChannel), // KATP channel
            Minoxidil => Some(IonChannel),     // KATP channel
            Omeprazole => Some(Transporter),   // H+/K+-ATPase
            _ => None,
        }
    }
}

/// Direction of Vmem effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VmemDirection {
    Hyperpolarizing,
    Depolarizing,
    Neutral,
}

/// Quality: does this agent hyperpolarize, depolarize, or neither?
#[derive(Debug, Clone)]
pub struct VmemEffect;

impl Quality for VmemEffect {
    type Individual = PharmacologyEntity;
    type Value = VmemDirection;

    fn get(&self, individual: &PharmacologyEntity) -> Option<VmemDirection> {
        use PharmacologyEntity::*;
        use VmemDirection::*;
        match individual {
            // Ivermectin: GlyR agonist → Cl- influx → hyperpolarization
            Ivermectin => Some(Hyperpolarizing),
            // Minoxidil: KATP opener → K+ efflux → hyperpolarization
            Minoxidil => Some(Hyperpolarizing),
            // Decamethonium: depolarizing blocker
            Decamethonium => Some(Depolarizing),
            // Glibenclamide: KATP blocker → depolarization
            Glibenclamide => Some(Depolarizing),
            // Omeprazole: acid suppression, Vmem-neutral
            Omeprazole => Some(Neutral),
            _ => None,
        }
    }
}

/// Quality: is this drug a morphoceutical (targets anatomical outcomes, not
/// just molecular pathways)?
///
/// Levin 2023: morphoceuticals are drugs whose therapeutic endpoint is a
/// specific anatomical structure, not merely a molecular target.
#[derive(Debug, Clone)]
pub struct IsMorphoceutical;

impl Quality for IsMorphoceutical {
    type Individual = PharmacologyEntity;
    type Value = bool;

    fn get(&self, individual: &PharmacologyEntity) -> Option<bool> {
        use PharmacologyEntity::*;
        match individual {
            // Ivermectin used as morphoceutical in Levin's planaria/frog work
            Ivermectin => Some(true),
            Minoxidil => Some(true),
            // Omeprazole targets acid secretion, not anatomical outcome
            Omeprazole => Some(false),
            Decamethonium => Some(false),
            Glibenclamide => Some(false),
            // The class itself
            Morphoceutical => Some(true),
            _ => None,
        }
    }
}

/// Quality: does this agent require a prescription?
#[derive(Debug, Clone)]
pub struct RequiresPrescription;

impl Quality for RequiresPrescription {
    type Individual = PharmacologyEntity;
    type Value = bool;

    fn get(&self, individual: &PharmacologyEntity) -> Option<bool> {
        use PharmacologyEntity::*;
        match individual {
            Ivermectin => Some(true),
            Decamethonium => Some(true),
            Glibenclamide => Some(true),
            Minoxidil => Some(false),  // OTC (topical)
            Omeprazole => Some(false), // OTC
            _ => None,
        }
    }
}

/// Quality: can the drug's bioelectric effect be achieved without the drug,
/// e.g., via mechanical stimulation or endogenous ion channel expression?
#[derive(Debug, Clone)]
pub struct IsEndogenouslyDerivable;

impl Quality for IsEndogenouslyDerivable {
    type Individual = PharmacologyEntity;
    type Value = bool;

    fn get(&self, individual: &PharmacologyEntity) -> Option<bool> {
        use PharmacologyEntity::*;
        match individual {
            // Mechanosensitive channels can be opened by vibration/pressure
            MechanosensitiveModulator => Some(true),
            // Ion channel drugs — not achievable without exogenous agent
            IonChannelModulator => Some(false),
            VoltageGatedBlocker => Some(false),
            VoltageGatedOpener => Some(false),
            ProtonPumpInhibitor => Some(false),
            GapJunctionModulator => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition (semantic contrasts)
// ---------------------------------------------------------------------------

/// Opposition pairs in the pharmacology domain.
///
/// - Hyperpolarization ↔ Depolarization: opposite Vmem effects
/// - GapJunctionOpening ↔ GapJunctionClosing: opposite GJ modulations
/// - VoltageGatedBlocker ↔ VoltageGatedOpener: opposite drug classes
pub struct PharmacologyOpposition;

impl OppositionDef for PharmacologyOpposition {
    type Entity = PharmacologyEntity;

    fn pairs() -> Vec<(PharmacologyEntity, PharmacologyEntity)> {
        use PharmacologyEntity::*;
        vec![
            (Hyperpolarization, Depolarization),
            (GapJunctionOpening, GapJunctionClosing),
            (VoltageGatedBlocker, VoltageGatedOpener),
        ]
    }
}

/// Axiom: pharmacology opposition is symmetric.
pub struct PharmacologyOppositionSymmetric;

impl Axiom for PharmacologyOppositionSymmetric {
    fn description(&self) -> &str {
        "pharmacology opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<PharmacologyOpposition>::new().holds()
    }
}

/// Axiom: pharmacology opposition is irreflexive (nothing opposes itself).
pub struct PharmacologyOppositionIrreflexive;

impl Axiom for PharmacologyOppositionIrreflexive {
    fn description(&self) -> &str {
        "pharmacology opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<PharmacologyOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Taxonomy is a DAG (no cycles).
pub struct PharmacologyTaxonomyIsDAG;

impl Axiom for PharmacologyTaxonomyIsDAG {
    fn description(&self) -> &str {
        "pharmacology taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<PharmacologyTaxonomy>::new().holds()
    }
}

/// Causal asymmetry: if A causes B then B does not cause A.
pub struct PharmacologyCausalAsymmetry;

impl Axiom for PharmacologyCausalAsymmetry {
    fn description(&self) -> &str {
        "pharmacology causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<PharmacologyCauses>::new().holds()
    }
}

/// Drug administration transitively causes VmemShift.
///
/// Chain: DrugAdministration → TargetBinding → ChannelStateChange →
///        IonFluxChange → VmemShift
pub struct DrugAdministrationCausesVmemShift;

impl Axiom for DrugAdministrationCausesVmemShift {
    fn description(&self) -> &str {
        "drug administration transitively causes Vmem shift"
    }

    fn holds(&self) -> bool {
        use PharmacologyEvent::*;
        let effects = causation::effects_of::<PharmacologyCauses>(&DrugAdministration);
        effects.contains(&VmemShift)
    }
}

/// GJ modulator binding causes collective reprogramming (Levin's network effect).
///
/// Chain: GJModulatorBinding → GapJunctionStateChange →
///        BioelectricNetworkChange → CollectiveReprogramming
pub struct GJModulatorCausesCollectiveReprogramming;

impl Axiom for GJModulatorCausesCollectiveReprogramming {
    fn description(&self) -> &str {
        "gap junction modulator causes collective reprogramming (Levin network effect)"
    }

    fn holds(&self) -> bool {
        use PharmacologyEvent::*;
        let effects = causation::effects_of::<PharmacologyCauses>(&GJModulatorBinding);
        effects.contains(&CollectiveReprogramming)
    }
}

/// Ivermectin is hyperpolarizing (GlyR agonist → Cl- influx).
pub struct IvermectinIsHyperpolarizing;

impl Axiom for IvermectinIsHyperpolarizing {
    fn description(&self) -> &str {
        "ivermectin is hyperpolarizing (GlyR agonist, Cl- influx)"
    }

    fn holds(&self) -> bool {
        VmemEffect.get(&PharmacologyEntity::Ivermectin) == Some(VmemDirection::Hyperpolarizing)
    }
}

/// Omeprazole is NOT a morphoceutical — it targets acid secretion, not anatomy.
pub struct OmeprazoleIsNotMorphoceutical;

impl Axiom for OmeprazoleIsNotMorphoceutical {
    fn description(&self) -> &str {
        "omeprazole is not a morphoceutical (targets acid, not anatomy)"
    }

    fn holds(&self) -> bool {
        IsMorphoceutical.get(&PharmacologyEntity::Omeprazole) == Some(false)
    }
}

/// Morphoceuticals target anatomical outcomes: the Morphoceutical class itself
/// has IsMorphoceutical = true, and it is a subset of DrugClass.
pub struct MorphoceuticalsTargetAnatomy;

impl Axiom for MorphoceuticalsTargetAnatomy {
    fn description(&self) -> &str {
        "morphoceuticals target anatomical outcomes (subset of drugs, Levin 2023)"
    }

    fn holds(&self) -> bool {
        use PharmacologyEntity::*;
        IsMorphoceutical.get(&Morphoceutical) == Some(true)
            && taxonomy::is_a::<PharmacologyTaxonomy>(&Morphoceutical, &DrugClass)
    }
}

/// MechanosensitiveModulator IS endogenously derivable — vibration/pressure
/// can activate mechanosensitive channels without drugs.
pub struct MechanosensitiveIsEndogenous;

impl Axiom for MechanosensitiveIsEndogenous {
    fn description(&self) -> &str {
        "mechanosensitive modulator is endogenously derivable (vibration suffices)"
    }

    fn holds(&self) -> bool {
        IsEndogenouslyDerivable.get(&PharmacologyEntity::MechanosensitiveModulator) == Some(true)
    }
}

/// Every agent has a target (DrugTarget quality is defined for all agents).
pub struct EveryAgentHasTarget;

impl Axiom for EveryAgentHasTarget {
    fn description(&self) -> &str {
        "every agent has a drug target"
    }

    fn holds(&self) -> bool {
        use PharmacologyEntity::*;
        let agents = [
            Ivermectin,
            Decamethonium,
            Glibenclamide,
            Minoxidil,
            Omeprazole,
        ];
        agents.iter().all(|a| DrugTarget.get(a).is_some())
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level ontology tying together the pharmacology category, qualities, and axioms.
pub struct PharmacologyOntology;

impl Ontology for PharmacologyOntology {
    type Cat = PharmacologyCategory;
    type Qual = IsMorphoceutical;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(PharmacologyTaxonomyIsDAG),
            Box::new(PharmacologyCausalAsymmetry),
            Box::new(DrugAdministrationCausesVmemShift),
            Box::new(GJModulatorCausesCollectiveReprogramming),
            Box::new(IvermectinIsHyperpolarizing),
            Box::new(OmeprazoleIsNotMorphoceutical),
            Box::new(MorphoceuticalsTargetAnatomy),
            Box::new(MechanosensitiveIsEndogenous),
            Box::new(EveryAgentHasTarget),
            Box::new(PharmacologyOppositionSymmetric),
            Box::new(PharmacologyOppositionIrreflexive),
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;

    // -- Entity count --

    #[test]
    fn test_entity_count() {
        assert_eq!(PharmacologyEntity::variants().len(), 25);
    }

    #[test]
    fn test_event_count() {
        assert_eq!(PharmacologyEvent::variants().len(), 10);
    }

    // -- Axiom tests --

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(
            PharmacologyTaxonomyIsDAG.holds(),
            "{}",
            PharmacologyTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_causal_asymmetry() {
        assert!(
            PharmacologyCausalAsymmetry.holds(),
            "{}",
            PharmacologyCausalAsymmetry.description()
        );
    }

    #[test]
    fn test_drug_administration_causes_vmem_shift() {
        assert!(
            DrugAdministrationCausesVmemShift.holds(),
            "{}",
            DrugAdministrationCausesVmemShift.description()
        );
    }

    #[test]
    fn test_gj_modulator_causes_collective_reprogramming() {
        assert!(
            GJModulatorCausesCollectiveReprogramming.holds(),
            "{}",
            GJModulatorCausesCollectiveReprogramming.description()
        );
    }

    #[test]
    fn test_ivermectin_is_hyperpolarizing() {
        assert!(
            IvermectinIsHyperpolarizing.holds(),
            "{}",
            IvermectinIsHyperpolarizing.description()
        );
    }

    #[test]
    fn test_omeprazole_is_not_morphoceutical() {
        assert!(
            OmeprazoleIsNotMorphoceutical.holds(),
            "{}",
            OmeprazoleIsNotMorphoceutical.description()
        );
    }

    #[test]
    fn test_morphoceuticals_target_anatomy() {
        assert!(
            MorphoceuticalsTargetAnatomy.holds(),
            "{}",
            MorphoceuticalsTargetAnatomy.description()
        );
    }

    #[test]
    fn test_mechanosensitive_is_endogenous() {
        assert!(
            MechanosensitiveIsEndogenous.holds(),
            "{}",
            MechanosensitiveIsEndogenous.description()
        );
    }

    #[test]
    fn test_every_agent_has_target() {
        assert!(
            EveryAgentHasTarget.holds(),
            "{}",
            EveryAgentHasTarget.description()
        );
    }

    // -- Opposition tests --

    #[test]
    fn test_pharmacology_opposition_symmetric() {
        assert!(
            PharmacologyOppositionSymmetric.holds(),
            "{}",
            PharmacologyOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_pharmacology_opposition_irreflexive() {
        assert!(
            PharmacologyOppositionIrreflexive.holds(),
            "{}",
            PharmacologyOppositionIrreflexive.description()
        );
    }

    #[test]
    fn test_hyperpolarization_opposes_depolarization() {
        use PharmacologyEntity::*;
        assert!(opposition::are_opposed::<PharmacologyOpposition>(
            &Hyperpolarization,
            &Depolarization
        ));
        assert!(opposition::are_opposed::<PharmacologyOpposition>(
            &Depolarization,
            &Hyperpolarization
        ));
    }

    #[test]
    fn test_gj_opening_opposes_gj_closing() {
        use PharmacologyEntity::*;
        assert!(opposition::are_opposed::<PharmacologyOpposition>(
            &GapJunctionOpening,
            &GapJunctionClosing
        ));
    }

    #[test]
    fn test_blocker_opposes_opener() {
        use PharmacologyEntity::*;
        assert!(opposition::are_opposed::<PharmacologyOpposition>(
            &VoltageGatedBlocker,
            &VoltageGatedOpener
        ));
    }

    #[test]
    fn test_hyperpolarization_does_not_oppose_gj_opening() {
        use PharmacologyEntity::*;
        assert!(!opposition::are_opposed::<PharmacologyOpposition>(
            &Hyperpolarization,
            &GapJunctionOpening
        ));
    }

    #[test]
    fn test_pharmacology_opposites_query() {
        use PharmacologyEntity::*;
        let opps = opposition::opposites::<PharmacologyOpposition>(&Hyperpolarization);
        assert_eq!(opps, vec![Depolarization]);
    }

    // -- Category law tests --

    #[test]
    fn test_pharmacology_category_laws() {
        check_category_laws::<PharmacologyCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<PharmacologyTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<PharmacologyCauses>>().unwrap();
    }

    // -- Taxonomy tests --

    #[test]
    fn test_agents_are_agents() {
        use PharmacologyEntity::*;
        for agent in [
            Ivermectin,
            Decamethonium,
            Glibenclamide,
            Minoxidil,
            Omeprazole,
        ] {
            assert!(
                taxonomy::is_a::<PharmacologyTaxonomy>(&agent, &Agent),
                "{:?} should be an Agent",
                agent
            );
        }
    }

    #[test]
    fn test_drug_classes_are_drug_classes() {
        use PharmacologyEntity::*;
        for class in [
            IonChannelModulator,
            GapJunctionModulator,
            VoltageGatedBlocker,
            VoltageGatedOpener,
            MechanosensitiveModulator,
            ProtonPumpInhibitor,
            Morphoceutical,
        ] {
            assert!(
                taxonomy::is_a::<PharmacologyTaxonomy>(&class, &DrugClass),
                "{:?} should be a DrugClass",
                class
            );
        }
    }

    #[test]
    fn test_ivermectin_is_ion_channel_modulator() {
        use PharmacologyEntity::*;
        assert!(taxonomy::is_a::<PharmacologyTaxonomy>(
            &Ivermectin,
            &IonChannelModulator
        ));
    }

    #[test]
    fn test_omeprazole_is_proton_pump_inhibitor() {
        use PharmacologyEntity::*;
        assert!(taxonomy::is_a::<PharmacologyTaxonomy>(
            &Omeprazole,
            &ProtonPumpInhibitor
        ));
    }

    #[test]
    fn test_ivermectin_transitively_is_drug_class() {
        use PharmacologyEntity::*;
        // Ivermectin → IonChannelModulator → DrugClass
        assert!(taxonomy::is_a::<PharmacologyTaxonomy>(
            &Ivermectin,
            &DrugClass
        ));
    }

    #[test]
    fn test_targets_are_targets() {
        use PharmacologyEntity::*;
        for target in [IonChannel, GapJunction, Transporter, Receptor] {
            assert!(
                taxonomy::is_a::<PharmacologyTaxonomy>(&target, &Target),
                "{:?} should be a Target",
                target
            );
        }
    }

    #[test]
    fn test_effects_are_effects() {
        use PharmacologyEntity::*;
        for effect in [
            Hyperpolarization,
            Depolarization,
            GapJunctionOpening,
            GapJunctionClosing,
            AntiInflammatory,
        ] {
            assert!(
                taxonomy::is_a::<PharmacologyTaxonomy>(&effect, &Effect),
                "{:?} should be an Effect",
                effect
            );
        }
    }

    // -- Causal chain tests --

    #[test]
    fn test_drug_administration_full_chain() {
        use PharmacologyEvent::*;
        let effects = causation::effects_of::<PharmacologyCauses>(&DrugAdministration);
        assert!(effects.contains(&TargetBinding));
        assert!(effects.contains(&ChannelStateChange));
        assert!(effects.contains(&IonFluxChange));
        assert!(effects.contains(&VmemShift));
        assert!(effects.contains(&DownstreamSignaling));
    }

    #[test]
    fn test_gj_modulator_full_chain() {
        use PharmacologyEvent::*;
        let effects = causation::effects_of::<PharmacologyCauses>(&GJModulatorBinding);
        assert!(effects.contains(&GapJunctionStateChange));
        assert!(effects.contains(&BioelectricNetworkChange));
        assert!(effects.contains(&CollectiveReprogramming));
    }

    #[test]
    fn test_vmem_shift_does_not_cause_drug_administration() {
        use PharmacologyEvent::*;
        let effects = causation::effects_of::<PharmacologyCauses>(&VmemShift);
        assert!(!effects.contains(&DrugAdministration));
    }

    // -- Quality tests --

    #[test]
    fn test_glibenclamide_is_depolarizing() {
        assert_eq!(
            VmemEffect.get(&PharmacologyEntity::Glibenclamide),
            Some(VmemDirection::Depolarizing)
        );
    }

    #[test]
    fn test_minoxidil_is_hyperpolarizing() {
        assert_eq!(
            VmemEffect.get(&PharmacologyEntity::Minoxidil),
            Some(VmemDirection::Hyperpolarizing)
        );
    }

    #[test]
    fn test_omeprazole_is_vmem_neutral() {
        assert_eq!(
            VmemEffect.get(&PharmacologyEntity::Omeprazole),
            Some(VmemDirection::Neutral)
        );
    }

    #[test]
    fn test_omeprazole_targets_transporter() {
        assert_eq!(
            DrugTarget.get(&PharmacologyEntity::Omeprazole),
            Some(PharmacologyEntity::Transporter)
        );
    }

    #[test]
    fn test_minoxidil_is_otc() {
        assert_eq!(
            RequiresPrescription.get(&PharmacologyEntity::Minoxidil),
            Some(false)
        );
    }

    #[test]
    fn test_ontology_validates() {
        PharmacologyOntology::validate().unwrap();
    }

    // -- Property-based tests (proptest) --

    use proptest::prelude::*;

    fn arb_pharmacology_entity() -> impl Strategy<Value = PharmacologyEntity> {
        (0..PharmacologyEntity::variants().len()).prop_map(|i| PharmacologyEntity::variants()[i])
    }

    proptest! {
        /// For any PharmacologyEntity that is an Agent, DrugTarget always returns Some
        /// (every drug has a target).
        #[test]
        fn prop_agent_has_drug_target(entity in arb_pharmacology_entity()) {
            if taxonomy::is_a::<PharmacologyTaxonomy>(
                &entity,
                &PharmacologyEntity::Agent,
            ) && entity != PharmacologyEntity::Agent
            {
                prop_assert!(
                    DrugTarget.get(&entity).is_some(),
                    "Agent {:?} must have a DrugTarget defined",
                    entity
                );
            }
        }
    }

    // -- Literature axioms --

    /// Kofman & Levin 2024: systematic review found ion channel drugs affect
    /// cancer phenotype. Every IonChannelModulator is a DrugClass.
    #[test]
    fn test_literature_kofman_levin_2024_ion_channel_drugs() {
        use PharmacologyEntity::*;
        // IonChannelModulator is-a DrugClass
        assert!(
            taxonomy::is_a::<PharmacologyTaxonomy>(&IonChannelModulator, &DrugClass),
            "Kofman & Levin 2024: IonChannelModulator must be a DrugClass"
        );
        // All agents that are IonChannelModulators are also transitively DrugClass
        let agents = [Ivermectin, Decamethonium];
        for agent in agents {
            assert!(
                taxonomy::is_a::<PharmacologyTaxonomy>(&agent, &IonChannelModulator),
                "Kofman & Levin 2024: {:?} must be an IonChannelModulator",
                agent
            );
            assert!(
                taxonomy::is_a::<PharmacologyTaxonomy>(&agent, &DrugClass),
                "Kofman & Levin 2024: {:?} must transitively be a DrugClass \
                 (ion channel drugs affect cancer phenotype)",
                agent
            );
        }
    }

    /// Chernet & Levin 2013: ivermectin (GlyR agonist) hyperpolarized cells by
    /// +19.4 mV, suppressing oncogene-induced tumors. Ivermectin is hyperpolarizing.
    #[test]
    fn test_literature_chernet_levin_2013_ivermectin_hyperpolarizing() {
        use PharmacologyEntity::*;
        assert_eq!(
            VmemEffect.get(&Ivermectin),
            Some(VmemDirection::Hyperpolarizing),
            "Chernet & Levin 2013: ivermectin (GlyR agonist) must be hyperpolarizing \
             (+19.4 mV shift, Cl- influx, suppresses oncogene-induced tumors)"
        );
        // Ivermectin targets a receptor (GlyR)
        assert_eq!(
            DrugTarget.get(&Ivermectin),
            Some(Receptor),
            "Chernet & Levin 2013: ivermectin targets GlyR (a receptor)"
        );
    }
}

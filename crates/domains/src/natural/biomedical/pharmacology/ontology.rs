//! Bioelectric pharmacology — drugs, targets, and bioelectric effects.
//!
//! Models drug classes (ion-channel modulators, gap-junction modulators,
//! voltage-gated openers/blockers, proton-pump inhibitors, morphoceuticals),
//! specific agents (ivermectin, decamethonium, glibenclamide, minoxidil,
//! omeprazole), molecular targets (ion channel, gap junction, transporter,
//! receptor), bioelectric effects (hyperpolarization, depolarization,
//! gap-junction opening/closing, anti-inflammatory), and the causal chains
//! drug → target binding → channel state change → ion flux → Vmem shift →
//! downstream signaling, and GJ modulator binding → gap-junction state
//! change → bioelectric network change → collective reprogramming. Per
//! `feedback_one_ontology_per_module` the original split between
//! `PharmacologyEntity` and `PharmacologyEvent` has been merged: events are
//! first-class concepts subsumed by the `PharmacologyEvent` umbrella.
//!
//! # Literature
//!
//! - **Goodman & Gilman (2018)** *The Pharmacological Basis of Therapeutics*,
//!   13th ed., McGraw-Hill — canonical reference for receptor-ligand
//!   pharmacology, ion-channel drugs (Kv openers/blockers, Na+/K+ blockers),
//!   proton-pump inhibitors, and drug-target classification.
//! - **Katzung (2018)** *Basic and Clinical Pharmacology*, 14th ed.,
//!   McGraw-Hill — class definitions for ion-channel modulators, gap-junction
//!   pharmacology, mechanosensitive-channel drugs, and prescription status.
//! - **Kofman & Levin (2024)** "Bioelectric pharmacology of cancer", review
//!   article — ion-channel drugs that reverse depolarised cancer Vmem.
//! - **Levin (2023)** "Morphoceuticals: drugs targeting anatomical
//!   outcomes" — drugs whose therapeutic endpoint is an anatomical
//!   structure rather than a molecular pathway.
//! - **Chernet & Levin (2013)** "Endogenous Voltage Potentials and the
//!   Microenvironment: Bioelectric Signals that Reveal, Induce and Normalize
//!   Cancer", *J. Clin. Exp. Oncol.* S1:002 — ivermectin (GlyR agonist) shifts
//!   Vmem by +19.4 mV and suppresses oncogene-induced tumors.
//! - **Adams & Levin (2013)** "Endogenous voltage gradients as mediators of
//!   cell-cell communication: strategies for investigating bioelectrical
//!   signals during pattern formation", *Cell Tissue Res.* 352(1):95–122 —
//!   ion channel / pump cocktails for Vmem manipulation.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Pharmacology",
    source: "Goodman & Gilman (2018) The Pharmacological Basis of Therapeutics 13th ed.; Katzung (2018) Basic and Clinical Pharmacology 14th ed.; Kofman & Levin (2024); Levin (2023) Morphoceuticals; Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002; Adams & Levin (2013) Cell Tissue Res. 352(1):95-122",

    concepts: [
        // === Drug classes (Goodman & Gilman 2018 Ch. 14, 30; Katzung 2018) ===
        IonChannelModulator,
        GapJunctionModulator,
        VoltageGatedBlocker,
        VoltageGatedOpener,
        MechanosensitiveModulator,
        ProtonPumpInhibitor,
        Morphoceutical,

        // === Specific agents (Goodman & Gilman 2018; Chernet & Levin 2013) ===
        Ivermectin,
        Decamethonium,
        Glibenclamide,
        Minoxidil,
        Omeprazole,

        // === Molecular targets (Goodman & Gilman 2018 Ch. 3) ===
        IonChannel,
        GapJunction,
        Transporter,
        Receptor,

        // === Bioelectric effects (Adams & Levin 2013; Chernet & Levin 2013) ===
        Hyperpolarization,
        Depolarization,
        GapJunctionOpening,
        GapJunctionClosing,
        AntiInflammatory,

        // === Abstract umbrellas ===
        DrugClass,
        Agent,
        Target,
        Effect,
        PharmacologyEvent,

        // === Causal events (Adams & Levin 2013; Kofman & Levin 2024) ===
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
    ],

    labels: {
        IonChannelModulator: ("en", "Ion channel modulator",
            "Goodman & Gilman (2018) Ch. 14: drug class acting on voltage- or ligand-gated ion channels to alter membrane excitability."),
        GapJunctionModulator: ("en", "Gap junction modulator",
            "Levin (2023): drug class that opens or closes connexin gap-junction channels and thus alters intercellular bioelectric coupling."),
        VoltageGatedBlocker: ("en", "Voltage-gated blocker",
            "Goodman & Gilman (2018) Ch. 14: agent that occludes a voltage-gated ion channel and prevents ion flux."),
        VoltageGatedOpener: ("en", "Voltage-gated opener",
            "Goodman & Gilman (2018) Ch. 14: agent that holds a voltage-gated ion channel open and increases ion flux."),
        MechanosensitiveModulator: ("en", "Mechanosensitive modulator",
            "Katzung (2018): drug class acting on mechanosensitive ion channels (Piezo-family); also activatable by direct mechanical stimulation."),
        ProtonPumpInhibitor: ("en", "Proton pump inhibitor",
            "Goodman & Gilman (2018) Ch. 49: drug class that irreversibly inhibits H+/K+-ATPase on parietal cells (e.g. omeprazole)."),
        Morphoceutical: ("en", "Morphoceutical",
            "Levin (2023): drug whose therapeutic endpoint is a specific anatomical structure rather than a molecular pathway."),

        Ivermectin: ("en", "Ivermectin",
            "Goodman & Gilman (2018) Ch. 54: glycine-receptor (GlyR) and glutamate-gated chloride-channel agonist; Chernet & Levin (2013) used it to hyperpolarise tumor cells by +19.4 mV."),
        Decamethonium: ("en", "Decamethonium",
            "Goodman & Gilman (2018) Ch. 11: depolarising neuromuscular blocker acting on the nicotinic acetylcholine receptor (nAChR)."),
        Glibenclamide: ("en", "Glibenclamide",
            "Goodman & Gilman (2018) Ch. 47: sulfonylurea that blocks pancreatic-beta-cell K_ATP channels, depolarising the cell to trigger insulin release."),
        Minoxidil: ("en", "Minoxidil",
            "Goodman & Gilman (2018) Ch. 27: K_ATP-channel opener; hyperpolarises vascular smooth muscle and is used topically for hair regrowth."),
        Omeprazole: ("en", "Omeprazole",
            "Goodman & Gilman (2018) Ch. 49: irreversible H+/K+-ATPase inhibitor used for acid suppression; Vmem-neutral."),

        IonChannel: ("en", "Ion channel",
            "Goodman & Gilman (2018) Ch. 3: transmembrane protein forming a selective conductance pathway for one or more ion species."),
        GapJunction: ("en", "Gap junction",
            "Levin (2023): cluster of connexin channels coupling the cytoplasm of adjacent cells; permits ionic and small-molecule diffusion."),
        Transporter: ("en", "Transporter",
            "Goodman & Gilman (2018) Ch. 3: ATPase or carrier protein that moves substrates across the membrane against or down a gradient."),
        Receptor: ("en", "Receptor",
            "Goodman & Gilman (2018) Ch. 3: protein that binds a ligand and transduces the binding event into a cellular response."),

        Hyperpolarization: ("en", "Hyperpolarization",
            "Adams & Levin (2013): shift of Vmem to a more negative value, typically via K+ efflux or Cl- influx."),
        Depolarization: ("en", "Depolarization",
            "Adams & Levin (2013): shift of Vmem to a less negative value, typically via Na+ influx or K+ blockade."),
        GapJunctionOpening: ("en", "Gap junction opening",
            "Levin (2023): increase in connexin channel permeability, expanding the bioelectric network."),
        GapJunctionClosing: ("en", "Gap junction closing",
            "Levin (2023): decrease in connexin channel permeability, electrically isolating cells from the network."),
        AntiInflammatory: ("en", "Anti-inflammatory effect",
            "Goodman & Gilman (2018) Ch. 38: damping of inflammatory signalling cascades (cytokines, prostaglandins, immune-cell recruitment)."),

        DrugClass: ("en", "Drug class",
            "Goodman & Gilman (2018): grouping of agents that share a mechanism of action or therapeutic endpoint."),
        Agent: ("en", "Agent",
            "Goodman & Gilman (2018): a specific chemical entity administered as a drug."),
        Target: ("en", "Target",
            "Goodman & Gilman (2018) Ch. 3: molecular entity to which a drug binds to produce its effect."),
        Effect: ("en", "Effect",
            "Goodman & Gilman (2018): physiological consequence produced when a drug engages its target."),
        PharmacologyEvent: ("en", "Pharmacology event",
            "Adams & Levin (2013): umbrella for time-extended pharmacological processes (administration, binding, state change, ion flux, Vmem shift, signalling)."),

        DrugAdministration: ("en", "Drug administration",
            "Goodman & Gilman (2018) Ch. 1: delivery of a drug to the organism (oral, parenteral, topical, etc.)."),
        TargetBinding: ("en", "Target binding",
            "Goodman & Gilman (2018) Ch. 3: physical engagement of the drug with its molecular target."),
        ChannelStateChange: ("en", "Channel state change",
            "Goodman & Gilman (2018) Ch. 14: transition of an ion-channel target between open, closed, or inactivated conformations."),
        IonFluxChange: ("en", "Ion flux change",
            "Adams & Levin (2013): change in transmembrane ionic current following channel state change."),
        VmemShift: ("en", "Vmem shift",
            "Adams & Levin (2013): change in membrane potential resulting from net ion-flux change."),
        DownstreamSignaling: ("en", "Downstream signaling",
            "Goodman & Gilman (2018) Ch. 3: cascade of intracellular events triggered by Vmem shift or receptor activation."),
        GJModulatorBinding: ("en", "Gap junction modulator binding",
            "Levin (2023): engagement of a connexin channel by a GJ-modulating drug."),
        GapJunctionStateChange: ("en", "Gap junction state change",
            "Levin (2023): transition of a connexin channel between open and closed states."),
        BioelectricNetworkChange: ("en", "Bioelectric network change",
            "Kofman & Levin (2024): alteration of the GJ-coupled bioelectric circuit topology."),
        CollectiveReprogramming: ("en", "Collective reprogramming",
            "Kofman & Levin (2024): tissue-scale change in cell fate or anatomy driven by altered bioelectric network state."),
    },

    is_a: [
        // Drug classes
        (IonChannelModulator, DrugClass),
        (GapJunctionModulator, DrugClass),
        (VoltageGatedBlocker, DrugClass),
        (VoltageGatedOpener, DrugClass),
        (MechanosensitiveModulator, DrugClass),
        (ProtonPumpInhibitor, DrugClass),
        (Morphoceutical, DrugClass),

        // Agents
        (Ivermectin, Agent),
        (Decamethonium, Agent),
        (Glibenclamide, Agent),
        (Minoxidil, Agent),
        (Omeprazole, Agent),

        // Agents within drug classes
        (Ivermectin, IonChannelModulator),
        (Decamethonium, IonChannelModulator),
        (Glibenclamide, VoltageGatedBlocker),
        (Minoxidil, VoltageGatedOpener),
        (Omeprazole, ProtonPumpInhibitor),

        // Targets
        (IonChannel, Target),
        (GapJunction, Target),
        (Transporter, Target),
        (Receptor, Target),

        // Effects
        (Hyperpolarization, Effect),
        (Depolarization, Effect),
        (GapJunctionOpening, Effect),
        (GapJunctionClosing, Effect),
        (AntiInflammatory, Effect),

        // Events under the PharmacologyEvent umbrella
        (DrugAdministration, PharmacologyEvent),
        (TargetBinding, PharmacologyEvent),
        (ChannelStateChange, PharmacologyEvent),
        (IonFluxChange, PharmacologyEvent),
        (VmemShift, PharmacologyEvent),
        (DownstreamSignaling, PharmacologyEvent),
        (GJModulatorBinding, PharmacologyEvent),
        (GapJunctionStateChange, PharmacologyEvent),
        (BioelectricNetworkChange, PharmacologyEvent),
        (CollectiveReprogramming, PharmacologyEvent),
    ],

    causes: [
        // Adams & Levin (2013): canonical bioelectric drug-action chain.
        (DrugAdministration, TargetBinding),
        (TargetBinding, ChannelStateChange),
        (ChannelStateChange, IonFluxChange),
        (IonFluxChange, VmemShift),
        (VmemShift, DownstreamSignaling),
        // Kofman & Levin (2024): gap-junction-modulator network effect.
        (GJModulatorBinding, GapJunctionStateChange),
        (GapJunctionStateChange, BioelectricNetworkChange),
        (BioelectricNetworkChange, CollectiveReprogramming),
    ],

    opposes: [
        // Hyperpolarization vs Depolarization — opposite Vmem effects
        // (Adams & Levin 2013).
        (Hyperpolarization, Depolarization),
        (Depolarization, Hyperpolarization),
        // GJ opening vs closing — opposite gap-junction modulations
        // (Levin 2023).
        (GapJunctionOpening, GapJunctionClosing),
        (GapJunctionClosing, GapJunctionOpening),
        // Blocker vs Opener — opposite drug-class actions on the same
        // channel (Goodman & Gilman 2018 Ch. 14).
        (VoltageGatedBlocker, VoltageGatedOpener),
        (VoltageGatedOpener, VoltageGatedBlocker),
    ],
}

// Backward-compatibility re-exports for sibling crates / partner functors
// that still reference the legacy `*Entity` / `*CategoryRelationKind` names.
pub use PharmacologyConcept as PharmacologyEntity;
pub use PharmacologyRelationKind as PharmacologyCategoryRelationKind;

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: what target does a given agent (or drug class) act on?
///
/// Goodman & Gilman (2018) Ch. 3 — every drug has a primary molecular target.
#[derive(Debug, Clone)]
pub struct DrugTarget;

impl Quality for DrugTarget {
    type Individual = PharmacologyConcept;
    type Value = PharmacologyConcept;

    fn get(&self, individual: &PharmacologyConcept) -> Option<PharmacologyConcept> {
        use PharmacologyConcept::*;
        match individual {
            Ivermectin => Some(Receptor),      // GlyR (glycine receptor)
            Decamethonium => Some(Receptor),   // nAChR
            Glibenclamide => Some(IonChannel), // K_ATP channel
            Minoxidil => Some(IonChannel),     // K_ATP channel
            Omeprazole => Some(Transporter),   // H+/K+-ATPase
            _ => None,
        }
    }
}

/// Direction of Vmem effect produced by an agent (Adams & Levin 2013).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VmemDirection {
    /// Vmem shifts to a more negative value (Adams & Levin 2013).
    Hyperpolarizing,
    /// Vmem shifts to a less negative value (Adams & Levin 2013).
    Depolarizing,
    /// Vmem is not perturbed by this agent.
    Neutral,
}

/// Quality: direction of Vmem effect.
#[derive(Debug, Clone)]
pub struct VmemEffect;

impl Quality for VmemEffect {
    type Individual = PharmacologyConcept;
    type Value = VmemDirection;

    fn get(&self, individual: &PharmacologyConcept) -> Option<VmemDirection> {
        use PharmacologyConcept::*;
        use VmemDirection::*;
        match individual {
            // Chernet & Levin (2013): ivermectin (GlyR agonist) hyperpolarises by +19.4 mV
            Ivermectin => Some(Hyperpolarizing),
            // Goodman & Gilman (2018) Ch. 27: minoxidil opens K_ATP -> K+ efflux -> hyperpolarisation
            Minoxidil => Some(Hyperpolarizing),
            // Goodman & Gilman (2018) Ch. 11: decamethonium is a depolarising NMJ blocker
            Decamethonium => Some(Depolarizing),
            // Goodman & Gilman (2018) Ch. 47: glibenclamide blocks K_ATP -> depolarisation
            Glibenclamide => Some(Depolarizing),
            // Goodman & Gilman (2018) Ch. 49: omeprazole inhibits H+/K+-ATPase, Vmem-neutral
            Omeprazole => Some(Neutral),
            _ => None,
        }
    }
}

/// Quality: is this drug a morphoceutical (targets anatomical outcomes)?
///
/// Levin (2023) — morphoceuticals are drugs whose therapeutic endpoint is
/// a specific anatomical structure, not merely a molecular target.
#[derive(Debug, Clone)]
pub struct IsMorphoceutical;

impl Quality for IsMorphoceutical {
    type Individual = PharmacologyConcept;
    type Value = bool;

    fn get(&self, individual: &PharmacologyConcept) -> Option<bool> {
        use PharmacologyConcept::*;
        match individual {
            // Ivermectin and minoxidil used as morphoceuticals in Levin's regeneration work
            Ivermectin => Some(true),
            Minoxidil => Some(true),
            // Omeprazole targets acid secretion, not anatomy
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
///
/// Goodman & Gilman (2018) and Katzung (2018) — drug-scheduling status.
#[derive(Debug, Clone)]
pub struct RequiresPrescription;

impl Quality for RequiresPrescription {
    type Individual = PharmacologyConcept;
    type Value = bool;

    fn get(&self, individual: &PharmacologyConcept) -> Option<bool> {
        use PharmacologyConcept::*;
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

/// Quality: can the bioelectric effect be achieved endogenously (without the drug)?
///
/// Mechanosensitive channels can be opened by vibration / pressure
/// (Katzung 2018; Coste et al. 2010 Piezo1 discovery).
#[derive(Debug, Clone)]
pub struct IsEndogenouslyDerivable;

impl Quality for IsEndogenouslyDerivable {
    type Individual = PharmacologyConcept;
    type Value = bool;

    fn get(&self, individual: &PharmacologyConcept) -> Option<bool> {
        use PharmacologyConcept::*;
        match individual {
            MechanosensitiveModulator => Some(true),
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
// Ontology + domain axioms
// ---------------------------------------------------------------------------

impl Ontology for PharmacologyOntology {
    type Cat = PharmacologyCategory;
    type Qual = IsMorphoceutical;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(DrugAdministrationCausesVmemShift));
        axioms.push(Box::new(GJModulatorCausesCollectiveReprogramming));
        axioms.push(Box::new(IvermectinIsHyperpolarizing));
        axioms.push(Box::new(OmeprazoleIsNotMorphoceutical));
        axioms.push(Box::new(MorphoceuticalsTargetAnatomy));
        axioms.push(Box::new(MechanosensitiveIsEndogenous));
        axioms.push(Box::new(EveryAgentHasTarget));
        axioms
    }
}

/// Helper: does a `Causation` edge exist from `cause` to `effect`?
fn causes(cause: PharmacologyConcept, effect: PharmacologyConcept) -> bool {
    PharmacologyCategory::morphisms().iter().any(|m| {
        m.kind() == PharmacologyRelationKind::Causation
            && m.source() == cause
            && m.target() == effect
    })
}

/// Helper: does a `Subsumption` edge exist from `child` to `parent`?
fn is_a(child: PharmacologyConcept, parent: PharmacologyConcept) -> bool {
    PharmacologyCategory::morphisms().iter().any(|m| {
        m.kind() == PharmacologyRelationKind::Subsumption
            && m.source() == child
            && m.target() == parent
    })
}

/// Axiom: DrugAdministration transitively causes VmemShift.
///
/// Adams & Levin (2013) — the canonical bioelectric-drug chain:
/// DrugAdministration → TargetBinding → ChannelStateChange →
/// IonFluxChange → VmemShift.
pub struct DrugAdministrationCausesVmemShift;

impl Axiom for DrugAdministrationCausesVmemShift {
    fn verify(&self) -> Verdict {
        // Walk the per-step causation chain (each edge is direct).
        let steps = [
            (
                PharmacologyConcept::DrugAdministration,
                PharmacologyConcept::TargetBinding,
            ),
            (
                PharmacologyConcept::TargetBinding,
                PharmacologyConcept::ChannelStateChange,
            ),
            (
                PharmacologyConcept::ChannelStateChange,
                PharmacologyConcept::IonFluxChange,
            ),
            (
                PharmacologyConcept::IonFluxChange,
                PharmacologyConcept::VmemShift,
            ),
        ];
        if steps.iter().all(|(c, e)| causes(*c, *e)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DrugAdministrationCausesVmemShift",
        "Drug administration transitively causes Vmem shift via the binding → channel-state → ion-flux chain",
        "Adams & Levin (2013) Cell Tissue Res. 352(1):95-122"
    );
}

pr4xis::register_axiom!(
    DrugAdministrationCausesVmemShift,
    "Adams & Levin (2013) Cell Tissue Res. 352(1):95-122"
);

/// Axiom: GJ modulator binding transitively causes collective reprogramming.
///
/// Kofman & Levin (2024) — the network-effect chain:
/// GJModulatorBinding → GapJunctionStateChange →
/// BioelectricNetworkChange → CollectiveReprogramming.
pub struct GJModulatorCausesCollectiveReprogramming;

impl Axiom for GJModulatorCausesCollectiveReprogramming {
    fn verify(&self) -> Verdict {
        let steps = [
            (
                PharmacologyConcept::GJModulatorBinding,
                PharmacologyConcept::GapJunctionStateChange,
            ),
            (
                PharmacologyConcept::GapJunctionStateChange,
                PharmacologyConcept::BioelectricNetworkChange,
            ),
            (
                PharmacologyConcept::BioelectricNetworkChange,
                PharmacologyConcept::CollectiveReprogramming,
            ),
        ];
        if steps.iter().all(|(c, e)| causes(*c, *e)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GJModulatorCausesCollectiveReprogramming",
        "Gap-junction modulator binding transitively causes collective tissue reprogramming",
        "Kofman & Levin (2024) Bioelectric pharmacology of cancer"
    );
}

pr4xis::register_axiom!(
    GJModulatorCausesCollectiveReprogramming,
    "Kofman & Levin (2024) Bioelectric pharmacology of cancer"
);

/// Axiom: Ivermectin is hyperpolarising.
///
/// Chernet & Levin (2013) — ivermectin (GlyR agonist) hyperpolarised tumor
/// cells by +19.4 mV and suppressed oncogene-induced tumors.
pub struct IvermectinIsHyperpolarizing;

impl Axiom for IvermectinIsHyperpolarizing {
    fn verify(&self) -> Verdict {
        if VmemEffect.get(&PharmacologyConcept::Ivermectin) == Some(VmemDirection::Hyperpolarizing)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "IvermectinIsHyperpolarizing",
        "Ivermectin (GlyR agonist) produces a hyperpolarising Vmem shift via Cl- influx",
        "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
    );
}

pr4xis::register_axiom!(
    IvermectinIsHyperpolarizing,
    "Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
);

/// Axiom: Omeprazole is not a morphoceutical.
///
/// Goodman & Gilman (2018) Ch. 49 — omeprazole targets acid secretion via
/// H+/K+-ATPase; its therapeutic endpoint is biochemical, not anatomical.
pub struct OmeprazoleIsNotMorphoceutical;

impl Axiom for OmeprazoleIsNotMorphoceutical {
    fn verify(&self) -> Verdict {
        if IsMorphoceutical.get(&PharmacologyConcept::Omeprazole) == Some(false) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "OmeprazoleIsNotMorphoceutical",
        "Omeprazole targets acid secretion (H+/K+-ATPase), not an anatomical outcome",
        "Goodman & Gilman (2018) The Pharmacological Basis of Therapeutics 13th ed. Ch. 49"
    );
}

pr4xis::register_axiom!(
    OmeprazoleIsNotMorphoceutical,
    "Goodman & Gilman (2018) The Pharmacological Basis of Therapeutics 13th ed. Ch. 49"
);

/// Axiom: Morphoceuticals target anatomical outcomes and are a DrugClass.
///
/// Levin (2023) — the Morphoceutical class is itself a morphoceutical and
/// it is subsumed under DrugClass in the taxonomy.
pub struct MorphoceuticalsTargetAnatomy;

impl Axiom for MorphoceuticalsTargetAnatomy {
    fn verify(&self) -> Verdict {
        let is_morpho = IsMorphoceutical.get(&PharmacologyConcept::Morphoceutical) == Some(true);
        let is_drug_class = is_a(
            PharmacologyConcept::Morphoceutical,
            PharmacologyConcept::DrugClass,
        );
        if is_morpho && is_drug_class {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MorphoceuticalsTargetAnatomy",
        "Morphoceuticals target anatomical outcomes and form a drug class",
        "Levin (2023) Morphoceuticals: drugs targeting anatomical outcomes"
    );
}

pr4xis::register_axiom!(
    MorphoceuticalsTargetAnatomy,
    "Levin (2023) Morphoceuticals: drugs targeting anatomical outcomes"
);

/// Axiom: MechanosensitiveModulator is endogenously derivable.
///
/// Katzung (2018) — mechanosensitive ion channels (Piezo family) can be
/// activated by direct mechanical stimulation (vibration, pressure) without
/// any pharmacological agent.
pub struct MechanosensitiveIsEndogenous;

impl Axiom for MechanosensitiveIsEndogenous {
    fn verify(&self) -> Verdict {
        if IsEndogenouslyDerivable.get(&PharmacologyConcept::MechanosensitiveModulator)
            == Some(true)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MechanosensitiveIsEndogenous",
        "Mechanosensitive-channel modulation can be achieved endogenously by mechanical stimulation",
        "Katzung (2018) Basic and Clinical Pharmacology 14th ed."
    );
}

pr4xis::register_axiom!(
    MechanosensitiveIsEndogenous,
    "Katzung (2018) Basic and Clinical Pharmacology 14th ed."
);

/// Axiom: every named agent has a defined molecular target.
///
/// Goodman & Gilman (2018) Ch. 3 — pharmacological drug-target principle.
pub struct EveryAgentHasTarget;

impl Axiom for EveryAgentHasTarget {
    fn verify(&self) -> Verdict {
        use PharmacologyConcept::*;
        let agents = [
            Ivermectin,
            Decamethonium,
            Glibenclamide,
            Minoxidil,
            Omeprazole,
        ];
        if agents.iter().all(|a| DrugTarget.get(a).is_some()) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EveryAgentHasTarget",
        "Every pharmacological agent has a defined molecular target",
        "Goodman & Gilman (2018) The Pharmacological Basis of Therapeutics 13th ed. Ch. 3"
    );
}

pr4xis::register_axiom!(
    EveryAgentHasTarget,
    "Goodman & Gilman (2018) The Pharmacological Basis of Therapeutics 13th ed. Ch. 3"
);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<PharmacologyCategory>();
    }

    #[test]
    fn ontology_validates() {
        PharmacologyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn concept_count() {
        // 7 drug classes + 5 agents + 4 targets + 5 effects
        // + 5 abstract umbrellas (DrugClass, Agent, Target, Effect, PharmacologyEvent)
        // + 10 events = 36.
        assert_eq!(PharmacologyConcept::variants().len(), 36);
    }

    // -- Domain axiom tests --

    #[test]
    fn drug_administration_causes_vmem_shift_axiom() {
        assert!(DrugAdministrationCausesVmemShift.verify().is_ok());
    }

    #[test]
    fn gj_modulator_causes_collective_reprogramming_axiom() {
        assert!(GJModulatorCausesCollectiveReprogramming.verify().is_ok());
    }

    #[test]
    fn ivermectin_is_hyperpolarizing_axiom() {
        assert!(IvermectinIsHyperpolarizing.verify().is_ok());
    }

    #[test]
    fn omeprazole_is_not_morphoceutical_axiom() {
        assert!(OmeprazoleIsNotMorphoceutical.verify().is_ok());
    }

    #[test]
    fn morphoceuticals_target_anatomy_axiom() {
        assert!(MorphoceuticalsTargetAnatomy.verify().is_ok());
    }

    #[test]
    fn mechanosensitive_is_endogenous_axiom() {
        assert!(MechanosensitiveIsEndogenous.verify().is_ok());
    }

    #[test]
    fn every_agent_has_target_axiom() {
        assert!(EveryAgentHasTarget.verify().is_ok());
    }

    // -- Subsumption-kind tests --

    #[test]
    fn agents_subsume_under_agent_umbrella() {
        use PharmacologyConcept::*;
        let subs: Vec<_> = PharmacologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PharmacologyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for agent in [
            Ivermectin,
            Decamethonium,
            Glibenclamide,
            Minoxidil,
            Omeprazole,
        ] {
            assert!(
                subs.contains(&(agent, Agent)),
                "{:?} should subsume under Agent",
                agent
            );
        }
    }

    #[test]
    fn drug_classes_subsume_under_drug_class_umbrella() {
        use PharmacologyConcept::*;
        let subs: Vec<_> = PharmacologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PharmacologyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for cls in [
            IonChannelModulator,
            GapJunctionModulator,
            VoltageGatedBlocker,
            VoltageGatedOpener,
            MechanosensitiveModulator,
            ProtonPumpInhibitor,
            Morphoceutical,
        ] {
            assert!(
                subs.contains(&(cls, DrugClass)),
                "{:?} should subsume under DrugClass",
                cls
            );
        }
    }

    #[test]
    fn events_subsume_under_pharmacology_event() {
        use PharmacologyConcept::*;
        let subs: Vec<_> = PharmacologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PharmacologyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for ev in [
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
        ] {
            assert!(
                subs.contains(&(ev, PharmacologyEvent)),
                "{:?} should subsume under PharmacologyEvent",
                ev
            );
        }
    }

    // -- Causation-kind tests --

    #[test]
    fn drug_administration_directly_causes_target_binding() {
        assert!(causes(
            PharmacologyConcept::DrugAdministration,
            PharmacologyConcept::TargetBinding
        ));
    }

    #[test]
    fn ion_flux_change_causes_vmem_shift() {
        assert!(causes(
            PharmacologyConcept::IonFluxChange,
            PharmacologyConcept::VmemShift
        ));
    }

    #[test]
    fn gj_modulator_binding_causes_state_change() {
        assert!(causes(
            PharmacologyConcept::GJModulatorBinding,
            PharmacologyConcept::GapJunctionStateChange
        ));
    }

    #[test]
    fn vmem_shift_does_not_cause_drug_administration() {
        assert!(!causes(
            PharmacologyConcept::VmemShift,
            PharmacologyConcept::DrugAdministration
        ));
    }

    // -- Opposition-kind tests --

    #[test]
    fn hyperpolarization_and_depolarization_oppose() {
        let opps: Vec<_> = PharmacologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PharmacologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            PharmacologyConcept::Hyperpolarization,
            PharmacologyConcept::Depolarization
        )));
        assert!(opps.contains(&(
            PharmacologyConcept::Depolarization,
            PharmacologyConcept::Hyperpolarization
        )));
    }

    #[test]
    fn gj_opening_and_closing_oppose() {
        let opps: Vec<_> = PharmacologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PharmacologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            PharmacologyConcept::GapJunctionOpening,
            PharmacologyConcept::GapJunctionClosing
        )));
    }

    #[test]
    fn blocker_and_opener_oppose() {
        let opps: Vec<_> = PharmacologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PharmacologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            PharmacologyConcept::VoltageGatedBlocker,
            PharmacologyConcept::VoltageGatedOpener
        )));
    }

    #[test]
    fn hyperpolarization_does_not_oppose_gj_opening() {
        let opps: Vec<_> = PharmacologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PharmacologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(!opps.contains(&(
            PharmacologyConcept::Hyperpolarization,
            PharmacologyConcept::GapJunctionOpening
        )));
    }

    // -- Quality tests --

    #[test]
    fn ivermectin_targets_receptor() {
        assert_eq!(
            DrugTarget.get(&PharmacologyConcept::Ivermectin),
            Some(PharmacologyConcept::Receptor)
        );
    }

    #[test]
    fn omeprazole_targets_transporter() {
        assert_eq!(
            DrugTarget.get(&PharmacologyConcept::Omeprazole),
            Some(PharmacologyConcept::Transporter)
        );
    }

    #[test]
    fn glibenclamide_is_depolarizing() {
        assert_eq!(
            VmemEffect.get(&PharmacologyConcept::Glibenclamide),
            Some(VmemDirection::Depolarizing)
        );
    }

    #[test]
    fn minoxidil_is_hyperpolarizing() {
        assert_eq!(
            VmemEffect.get(&PharmacologyConcept::Minoxidil),
            Some(VmemDirection::Hyperpolarizing)
        );
    }

    #[test]
    fn omeprazole_is_vmem_neutral() {
        assert_eq!(
            VmemEffect.get(&PharmacologyConcept::Omeprazole),
            Some(VmemDirection::Neutral)
        );
    }

    #[test]
    fn minoxidil_is_otc() {
        assert_eq!(
            RequiresPrescription.get(&PharmacologyConcept::Minoxidil),
            Some(false)
        );
    }

    #[test]
    fn ivermectin_requires_prescription() {
        assert_eq!(
            RequiresPrescription.get(&PharmacologyConcept::Ivermectin),
            Some(true)
        );
    }

    #[test]
    fn morphoceutical_class_is_morphoceutical() {
        assert_eq!(
            IsMorphoceutical.get(&PharmacologyConcept::Morphoceutical),
            Some(true)
        );
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = PharmacologyConcept> {
        proptest::sample::select(PharmacologyConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in PharmacologyCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in PharmacologyOntology::axioms() {
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
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = PharmacologyConcept::variants();
            for m in PharmacologyCategory::morphisms() {
                if m.kind() == PharmacologyRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = PharmacologyCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == PharmacologyRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(
                    opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} -> {:?} but not back",
                    a,
                    b
                );
            }
        }

        /// Every agent has a defined DrugTarget.
        #[test]
        fn prop_agents_have_drug_target(c in arb_concept()) {
            // Agents are direct subsumers of `Agent` in the taxonomy.
            let subs: Vec<_> = PharmacologyCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == PharmacologyRelationKind::Subsumption)
                .map(|m| (m.source(), m.target()))
                .collect();
            let is_agent =
                subs.contains(&(c, PharmacologyConcept::Agent)) && c != PharmacologyConcept::Agent;
            if is_agent {
                prop_assert!(
                    DrugTarget.get(&c).is_some(),
                    "Agent {:?} must have a DrugTarget defined",
                    c
                );
            }
        }
    }
}

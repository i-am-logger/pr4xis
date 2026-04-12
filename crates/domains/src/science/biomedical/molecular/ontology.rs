//! Molecular biology ontology.
//!
//! Entities: ions, channels, proteins, signaling molecules.
//! Taxonomy: channel type hierarchy (voltage-gated, mechanosensitive, etc.).
//! Causal graph: mechanical stress → channel activation → ion flux → Vmem shift.
//!
//! Key references:
//! - Coste 2010: Piezo1/Piezo2 discovery (2021 Nobel Prize)
//! - Mihara 2011: TRPV4 in esophageal epithelium
//! - Fukada & Yasuda 1957: collagen piezoelectricity
//! - Inose 2009: Cx26/Cx43 in esophagus
//! - Khalbuss 1995: acid effects on esophageal ion channels

use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::context::{self, ContextDef};
use pr4xis::ontology::reasoning::mereology::{self, MereologyDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every molecular entity in the esophageal repair domain.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum MolecularEntity {
    // Ions
    Sodium,
    Potassium,
    Calcium,
    Chloride,
    Proton,
    // Voltage-gated channels
    Nav,
    Kv,
    Cav,
    // Mechanosensitive channels
    Piezo1,
    Piezo2,
    TRPV4,
    // Ligand-gated channels
    GlyR,
    GABA_A,
    // Gap junctions
    Cx26,
    Cx43,
    // Structural proteins
    Collagen,
    Mucin,
    // Signaling molecules
    CalciumSignal,
    NitricOxide,
    // Abstract categories
    Ion,
    IonChannel,
    VoltageGated,
    Mechanosensitive,
    LigandGated,
    GapJunction,
    Protein,
    SignalingMolecule,
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for molecular entities.
pub struct MolecularTaxonomy;

impl TaxonomyDef for MolecularTaxonomy {
    type Entity = MolecularEntity;

    fn relations() -> Vec<(MolecularEntity, MolecularEntity)> {
        use MolecularEntity::*;
        vec![
            // Ions is-a Ion
            (Sodium, Ion),
            (Potassium, Ion),
            (Calcium, Ion),
            (Chloride, Ion),
            (Proton, Ion),
            // Channel subtypes is-a IonChannel
            (VoltageGated, IonChannel),
            (Mechanosensitive, IonChannel),
            (LigandGated, IonChannel),
            (GapJunction, IonChannel),
            // Voltage-gated channels
            (Nav, VoltageGated),
            (Kv, VoltageGated),
            (Cav, VoltageGated),
            // Mechanosensitive channels
            (Piezo1, Mechanosensitive),
            (Piezo2, Mechanosensitive),
            (TRPV4, Mechanosensitive),
            // Ligand-gated channels
            (GlyR, LigandGated),
            (GABA_A, LigandGated),
            // Gap junctions
            (Cx26, GapJunction),
            (Cx43, GapJunction),
            // Structural proteins
            (Collagen, Protein),
            (Mucin, Protein),
            // Signaling molecules
            (CalciumSignal, SignalingMolecule),
            (NitricOxide, SignalingMolecule),
        ]
    }
}

// ---------------------------------------------------------------------------
// Mereology (has-a / part-whole)
// ---------------------------------------------------------------------------

/// Part-whole relationships for molecular entities.
///
/// - IonChannel has-a Ion (channels conduct ions)
/// - VoltageGated has-a Nav, Kv, Cav
/// - Mechanosensitive has-a Piezo1, Piezo2, TRPV4
/// - LigandGated has-a GlyR, GABA_A
/// - GapJunction has-a Cx26, Cx43
pub struct MolecularMereology;

impl MereologyDef for MolecularMereology {
    type Entity = MolecularEntity;

    fn relations() -> Vec<(MolecularEntity, MolecularEntity)> {
        use MolecularEntity::*;
        vec![
            // IonChannel has-a Ion (channels conduct ions)
            (IonChannel, Ion),
            // VoltageGated has-a specific voltage-gated channels
            (VoltageGated, Nav),
            (VoltageGated, Kv),
            (VoltageGated, Cav),
            // Mechanosensitive has-a specific mechanosensitive channels
            (Mechanosensitive, Piezo1),
            (Mechanosensitive, Piezo2),
            (Mechanosensitive, TRPV4),
            // LigandGated has-a specific ligand-gated channels
            (LigandGated, GlyR),
            (LigandGated, GABA_A),
            // GapJunction has-a specific connexins
            (GapJunction, Cx26),
            (GapJunction, Cx43),
        ]
    }
}

/// Axiom: molecular mereology has no cycles.
pub struct MolecularMereologyNoCycles;

impl Axiom for MolecularMereologyNoCycles {
    fn description(&self) -> &str {
        "molecular mereology has no cycles"
    }

    fn holds(&self) -> bool {
        mereology::NoCycles::<MolecularMereology>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Causal events in the mechanotransduction pathway.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum CausalEvent {
    MechanicalStress,
    Piezo1Opening,
    TRPV4Opening,
    CollagenPiezoelectric,
    CalciumInflux,
    VmemShift,
    GeneExpression,
    MorphologicalChange,
    AcidExposure,
    KvInhibition,
    GlyRActivation,
    ChlorideInflux,
    Cx43Upregulation,
    GapJunctionFormation,
    BioelectricCoupling,
}

/// Causal graph: mechanotransduction and bioelectric signaling pathways.
pub struct MechanotransductionCausalGraph;

impl CausalDef for MechanotransductionCausalGraph {
    type Entity = CausalEvent;

    fn relations() -> Vec<(CausalEvent, CausalEvent)> {
        use CausalEvent::*;
        vec![
            // Mechanical stress triggers channel openings and piezoelectricity
            (MechanicalStress, Piezo1Opening),
            (MechanicalStress, TRPV4Opening),
            (MechanicalStress, CollagenPiezoelectric),
            // Channel openings cause calcium influx
            (Piezo1Opening, CalciumInflux),
            (TRPV4Opening, CalciumInflux),
            // Calcium influx shifts Vmem
            (CalciumInflux, VmemShift),
            // Vmem shift drives gene expression
            (VmemShift, GeneExpression),
            // Gene expression produces morphological change
            (GeneExpression, MorphologicalChange),
            // Acid pathway: acid inhibits Kv → depolarization
            (AcidExposure, KvInhibition),
            (KvInhibition, VmemShift),
            // Glycine receptor pathway: GlyR → Cl- → hyperpolarization
            (GlyRActivation, ChlorideInflux),
            (ChlorideInflux, VmemShift),
            // Gap junction pathway: Cx43 → GJ formation → bioelectric coupling → Vmem
            (Cx43Upregulation, GapJunctionFormation),
            (GapJunctionFormation, BioelectricCoupling),
            (BioelectricCoupling, VmemShift),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category (MolecularRelation morphism over MolecularEntity)
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over molecular entities.
    ///
    /// Every entity pair has a unique morphism; composition is transitive.
    pub MolecularCategory {
        entity: MolecularEntity,
        relation: MolecularRelation,
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Ionic charge (in elementary charge units).
#[derive(Debug, Clone)]
pub struct IonCharge;

impl Quality for IonCharge {
    type Individual = MolecularEntity;
    type Value = i32;

    fn get(&self, individual: &MolecularEntity) -> Option<i32> {
        use MolecularEntity::*;
        match individual {
            Sodium => Some(1),
            Potassium => Some(1),
            Calcium => Some(2),
            Chloride => Some(-1),
            Proton => Some(1),
            _ => None,
        }
    }
}

/// Nernst equilibrium potential (mV) for each ion species.
#[derive(Debug, Clone)]
pub struct EquilibriumPotential;

impl Quality for EquilibriumPotential {
    type Individual = MolecularEntity;
    type Value = f64;

    fn get(&self, individual: &MolecularEntity) -> Option<f64> {
        use MolecularEntity::*;
        match individual {
            Sodium => Some(67.0),
            Potassium => Some(-90.0),
            Calcium => Some(131.0), // [Ca2+]o=2mM, [Ca2+]i=100nM, T=37C
            Chloride => Some(-70.0),
            Proton => Some(-24.0), // H+ Nernst: pHi=7.0, pHo=7.4, T=37C
            _ => None,
        }
    }
}

/// Channel activation mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationMechanism {
    Voltage,
    Mechanical,
    Ligand,
    GapJunctionCoupling,
}

/// Quality: how is this channel activated?
#[derive(Debug, Clone)]
pub struct ChannelActivation;

impl Quality for ChannelActivation {
    type Individual = MolecularEntity;
    type Value = ActivationMechanism;

    fn get(&self, individual: &MolecularEntity) -> Option<ActivationMechanism> {
        use ActivationMechanism::*;
        use MolecularEntity::*;
        match individual {
            Nav | Kv | Cav => Some(Voltage),
            Piezo1 | Piezo2 | TRPV4 => Some(Mechanical),
            GlyR | GABA_A => Some(Ligand),
            Cx26 | Cx43 => Some(GapJunctionCoupling),
            _ => None,
        }
    }
}

/// Quality: which ion does this channel primarily conduct?
#[derive(Debug, Clone)]
pub struct IonSelectivity;

impl Quality for IonSelectivity {
    type Individual = MolecularEntity;
    type Value = MolecularEntity;

    fn get(&self, individual: &MolecularEntity) -> Option<MolecularEntity> {
        use MolecularEntity::*;
        match individual {
            Nav => Some(Sodium),
            Kv => Some(Potassium),
            Cav => Some(Calcium),
            Piezo1 | Piezo2 | TRPV4 => Some(Calcium),
            GlyR | GABA_A => Some(Chloride),
            Cx26 | Cx43 => Some(Calcium),
            _ => None,
        }
    }
}

/// Quality: is this entity expressed in the esophagus?
#[derive(Debug, Clone)]
pub struct ExpressedInEsophagus;

impl Quality for ExpressedInEsophagus {
    type Individual = MolecularEntity;
    type Value = bool;

    fn get(&self, individual: &MolecularEntity) -> Option<bool> {
        use MolecularEntity::*;
        match individual {
            Piezo1 | TRPV4 | Kv | Cx26 | Cx43 | Collagen | Mucin => Some(true),
            // Concrete entities not expressed (or unknown)
            Sodium | Potassium | Calcium | Chloride | Proton | Nav | Cav | Piezo2 | GlyR
            | GABA_A | CalciumSignal | NitricOxide => Some(false),
            // Abstract categories don't have expression
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition (semantic contrasts)
// ---------------------------------------------------------------------------

/// Opposition pairs in the molecular domain.
///
/// - Sodium ↔ Potassium: depolarizing vs repolarizing primary ions
/// - Calcium ↔ Chloride: excitatory vs inhibitory signaling ions
/// - Nav ↔ Kv: depolarization channel vs repolarization channel
pub struct MolecularOpposition;

impl OppositionDef for MolecularOpposition {
    type Entity = MolecularEntity;

    fn pairs() -> Vec<(MolecularEntity, MolecularEntity)> {
        use MolecularEntity::*;
        vec![(Sodium, Potassium), (Calcium, Chloride), (Nav, Kv)]
    }
}

/// Axiom: molecular opposition is symmetric.
pub struct MolecularOppositionSymmetric;

impl Axiom for MolecularOppositionSymmetric {
    fn description(&self) -> &str {
        "molecular opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<MolecularOpposition>::new().holds()
    }
}

/// Axiom: molecular opposition is irreflexive (nothing opposes itself).
pub struct MolecularOppositionIrreflexive;

impl Axiom for MolecularOppositionIrreflexive {
    fn description(&self) -> &str {
        "molecular opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<MolecularOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// The molecular taxonomy has no cycles (is a DAG).
pub struct MolecularTaxonomyIsDAG;

impl Axiom for MolecularTaxonomyIsDAG {
    fn description(&self) -> &str {
        "molecular taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<MolecularTaxonomy>::new().holds()
    }
}

/// Piezo1 is-a Mechanosensitive is-a IonChannel (two-level subsumption).
pub struct Piezo1IsMechanosensitiveChannel;

impl Axiom for Piezo1IsMechanosensitiveChannel {
    fn description(&self) -> &str {
        "Piezo1 is-a Mechanosensitive is-a IonChannel"
    }

    fn holds(&self) -> bool {
        use MolecularEntity::*;
        taxonomy::is_a::<MolecularTaxonomy>(&Piezo1, &Mechanosensitive)
            && taxonomy::is_a::<MolecularTaxonomy>(&Mechanosensitive, &IonChannel)
            && taxonomy::is_a::<MolecularTaxonomy>(&Piezo1, &IonChannel)
    }
}

/// TRPV4 is mechanosensitive AND expressed in the esophagus.
pub struct TRPV4InEsophagus;

impl Axiom for TRPV4InEsophagus {
    fn description(&self) -> &str {
        "TRPV4 is mechanosensitive and expressed in the esophagus"
    }

    fn holds(&self) -> bool {
        use MolecularEntity::*;
        taxonomy::is_a::<MolecularTaxonomy>(&TRPV4, &Mechanosensitive)
            && ExpressedInEsophagus.get(&TRPV4) == Some(true)
    }
}

/// All three mechanosensitive channels (Piezo1, Piezo2, TRPV4) pass calcium.
pub struct MechanosensitiveChannelsPassCalcium;

impl Axiom for MechanosensitiveChannelsPassCalcium {
    fn description(&self) -> &str {
        "all mechanosensitive channels conduct calcium"
    }

    fn holds(&self) -> bool {
        use MolecularEntity::*;
        [Piezo1, Piezo2, TRPV4]
            .iter()
            .all(|ch| IonSelectivity.get(ch) == Some(Calcium))
    }
}

/// The causal graph is asymmetric: if A causes B then B does not cause A.
pub struct CausalGraphIsAsymmetric;

impl Axiom for CausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<MechanotransductionCausalGraph>::new().holds()
    }
}

/// No event directly causes itself in the causal graph.
pub struct CausalGraphNoSelfCause;

impl Axiom for CausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no event directly causes itself"
    }

    fn holds(&self) -> bool {
        causation::NoSelfCausation::<MechanotransductionCausalGraph>::new().holds()
    }
}

/// Mechanical stress transitively causes morphological change.
pub struct MechanicalStressCausesMorphology;

impl Axiom for MechanicalStressCausesMorphology {
    fn description(&self) -> &str {
        "mechanical stress transitively causes morphological change"
    }

    fn holds(&self) -> bool {
        use CausalEvent::*;
        let effects = causation::effects_of::<MechanotransductionCausalGraph>(&MechanicalStress);
        effects.contains(&MorphologicalChange)
    }
}

/// Acid exposure causes Kv inhibition causes Vmem shift.
pub struct AcidCausesVmemShift;

impl Axiom for AcidCausesVmemShift {
    fn description(&self) -> &str {
        "acid exposure causes Vmem shift via Kv inhibition"
    }

    fn holds(&self) -> bool {
        use CausalEvent::*;
        let acid_effects = causation::effects_of::<MechanotransductionCausalGraph>(&AcidExposure);
        acid_effects.contains(&KvInhibition) && acid_effects.contains(&VmemShift)
    }
}

/// GlyR activation causes chloride influx causes Vmem shift (hyperpolarization).
pub struct GlyRCausesHyperpolarization;

impl Axiom for GlyRCausesHyperpolarization {
    fn description(&self) -> &str {
        "GlyR activation causes Vmem shift via chloride influx"
    }

    fn holds(&self) -> bool {
        use CausalEvent::*;
        let effects = causation::effects_of::<MechanotransductionCausalGraph>(&GlyRActivation);
        effects.contains(&ChlorideInflux) && effects.contains(&VmemShift)
    }
}

/// Nernst potentials are consistent: K < 0, Na > 0, Ca > 0, Cl < 0.
pub struct NernstPotentialsConsistent;

impl Axiom for NernstPotentialsConsistent {
    fn description(&self) -> &str {
        "Nernst equilibrium potentials have correct signs"
    }

    fn holds(&self) -> bool {
        use MolecularEntity::*;
        let e = EquilibriumPotential;
        e.get(&Potassium).unwrap() < 0.0
            && e.get(&Sodium).unwrap() > 0.0
            && e.get(&Calcium).unwrap() > 0.0
            && e.get(&Chloride).unwrap() < 0.0
            && e.get(&Proton).unwrap() < 0.0
    }
}

// ---------------------------------------------------------------------------
// Context — functional mode disambiguation
// ---------------------------------------------------------------------------
//
// Discovery: adjunction analysis (MolecularToBioelectric ⊣ BioelectricToMolecular)
// revealed that the counit collapses MembranePotential and IonChannelModulation
// to the same molecule (Kv). This means the molecular ontology was MISSING a
// distinction: molecules have two functional modes (constitutive vs therapeutic).
//
// ContextDef resolves this ambiguity. Same protein, different context.
//
// LITERATURE BASIS:
//   - Kv as resting Vmem setter: textbook electrophysiology
//   - Kv as drug target: Kofman & Levin 2024 (bioelectric pharmacology review)
//   - Piezo1 as sensor: Coste et al. 2010 (Nobel 2021)
//   - Piezo1 as therapeutic target: Lewis et al. 2017 (repetitive stimuli)
//   - Cx43 constitutive: Inose et al. 2009 (esophageal gap junctions)
//   - Cx43 as intervention target: Levin 2014 (bioelectric reprogramming)
//
// NOVEL CONTRIBUTION:
//   - Adjunction detecting the dual role automatically (counit collapse)
//   - ContextDef formalization of constitutive vs therapeutic modes
//
// HYPOTHESIS (testable):
//   - That every ion channel/connexin has exactly two functional modes
//   - Some may have more (e.g., Piezo1 in development vs adult homeostasis)

/// Functional context in which a molecular entity operates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum FunctionalContext {
    /// Constitutive mode: the molecule performs its baseline biological function.
    /// Kv maintains resting Vmem. Piezo1 senses mechanical environment.
    /// Cx43 connects cells in existing gap junction network.
    Constitutive,
    /// Therapeutic mode: the molecule is the target of an intervention.
    /// Kv is modulated by a drug to shift Vmem. Piezo1 is activated by
    /// deliberate vibration. Cx43 is upregulated to restore connectivity.
    Therapeutic,
}

/// Resolved functional role after disambiguation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum FunctionalRole {
    /// Passive homeostatic function (maintains resting state).
    PassiveHomeostatic,
    /// Active sensing function (detects environmental signals).
    MechanicalSensor,
    /// Structural component (part of existing tissue architecture).
    StructuralComponent,
    /// Communication channel (propagates signals between cells).
    InterCellularChannel,
    /// Therapeutic target (can be modulated to shift tissue state).
    TherapeuticTarget,
    /// Signal mediator (carries information downstream of activation).
    SignalMediator,
    /// Structural scaffold (provides mechanical support).
    StructuralScaffold,
    /// Protective barrier (shields tissue from damage).
    ProtectiveBarrier,
}

/// Context-dependent disambiguation of molecular entities.
///
/// Discovered via adjunction analysis: the counit of
/// MolecularToBioelectric ⊣ BioelectricToMolecular collapses
/// distinct bioelectric concepts to the same molecule, revealing
/// that molecules serve dual functional roles depending on context.
pub struct MolecularFunctionalContext;

impl ContextDef for MolecularFunctionalContext {
    type Entity = MolecularEntity;
    type Signal = FunctionalContext;
    type Resolution = FunctionalRole;

    fn resolutions() -> Vec<(MolecularEntity, FunctionalContext, FunctionalRole)> {
        use FunctionalContext::*;
        use FunctionalRole::*;
        use MolecularEntity::*;
        vec![
            // Kv: maintains resting Vmem vs therapeutic target for Vmem modulation
            (Kv, Constitutive, PassiveHomeostatic),
            (Kv, Therapeutic, TherapeuticTarget),
            // Nav: sets action potential threshold vs drug target
            (Nav, Constitutive, PassiveHomeostatic),
            (Nav, Therapeutic, TherapeuticTarget),
            // Cav: baseline calcium signaling vs modulation target
            (Cav, Constitutive, SignalMediator),
            (Cav, Therapeutic, TherapeuticTarget),
            // Piezo1: senses mechanical environment vs activated by deliberate vibration
            (Piezo1, Constitutive, MechanicalSensor),
            (Piezo1, Therapeutic, TherapeuticTarget),
            // Piezo2: proprioception vs therapeutic activation
            (Piezo2, Constitutive, MechanicalSensor),
            (Piezo2, Therapeutic, TherapeuticTarget),
            // TRPV4: osmotic/mechanical sensing vs therapeutic activation
            (TRPV4, Constitutive, MechanicalSensor),
            (TRPV4, Therapeutic, TherapeuticTarget),
            // GlyR: inhibitory neurotransmission vs Levin's hyperpolarization tool
            (GlyR, Constitutive, PassiveHomeostatic),
            (GlyR, Therapeutic, TherapeuticTarget),
            // GABA_A: inhibitory signaling vs drug target
            (GABA_A, Constitutive, PassiveHomeostatic),
            (GABA_A, Therapeutic, TherapeuticTarget),
            // Cx26: existing gap junction vs upregulation target
            (Cx26, Constitutive, InterCellularChannel),
            (Cx26, Therapeutic, TherapeuticTarget),
            // Cx43: existing gap junction vs connectivity modulation
            (Cx43, Constitutive, InterCellularChannel),
            (Cx43, Therapeutic, TherapeuticTarget),
            // Collagen: structural ECM component vs piezoelectric transducer
            (Collagen, Constitutive, StructuralScaffold),
            (Collagen, Therapeutic, MechanicalSensor),
            // Mucin: protective mucus layer vs barrier restoration target
            (Mucin, Constitutive, ProtectiveBarrier),
            (Mucin, Therapeutic, TherapeuticTarget),
        ]
    }
}

/// Axiom: molecular context resolution is deterministic.
/// Each (molecule, context) pair maps to exactly one functional role.
pub struct MolecularContextDeterministic;

impl Axiom for MolecularContextDeterministic {
    fn description(&self) -> &str {
        "molecular context resolution is deterministic: same molecule + same context = same role"
    }
    fn holds(&self) -> bool {
        context::Deterministic::<MolecularFunctionalContext>::default().holds()
    }
}

/// Axiom: all disambiguated molecules have true ambiguity.
/// Every molecule in the context map resolves to at least 2 distinct roles.
pub struct MolecularContextTrueAmbiguity;

impl Axiom for MolecularContextTrueAmbiguity {
    fn description(&self) -> &str {
        "every molecule in the context map has at least two distinct functional roles"
    }
    fn holds(&self) -> bool {
        context::TrueAmbiguity::<MolecularFunctionalContext>::default().holds()
    }
}

/// Axiom: Kv in constitutive mode is passive homeostatic (sets resting Vmem),
/// but in therapeutic mode is a target (drug shifts Vmem).
/// This is the counit collapse discovery: MembranePotential and
/// IonChannelModulation are the SAME molecule in different contexts.
pub struct KvDualRole;

impl Axiom for KvDualRole {
    fn description(&self) -> &str {
        "Kv has dual role: passive Vmem setter (constitutive) vs therapeutic target (adjunction discovery)"
    }
    fn holds(&self) -> bool {
        use FunctionalContext::*;
        use FunctionalRole::*;
        use MolecularEntity::*;
        context::resolve::<MolecularFunctionalContext>(&Kv, &Constitutive)
            == Some(PassiveHomeostatic)
            && context::resolve::<MolecularFunctionalContext>(&Kv, &Therapeutic)
                == Some(TherapeuticTarget)
    }
}

/// Axiom: Piezo1 in constitutive mode is a sensor, in therapeutic mode is a target.
/// Vibration therapy works by shifting Piezo1 from sensing to being actively driven.
pub struct Piezo1DualRole;

impl Axiom for Piezo1DualRole {
    fn description(&self) -> &str {
        "Piezo1 is a mechanical sensor (constitutive) and therapeutic target (vibration therapy)"
    }
    fn holds(&self) -> bool {
        use FunctionalContext::*;
        use FunctionalRole::*;
        use MolecularEntity::*;
        context::resolve::<MolecularFunctionalContext>(&Piezo1, &Constitutive)
            == Some(MechanicalSensor)
            && context::resolve::<MolecularFunctionalContext>(&Piezo1, &Therapeutic)
                == Some(TherapeuticTarget)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level ontology tying together the molecular category, qualities, and axioms.
pub struct MolecularOntology;

impl Ontology for MolecularOntology {
    type Cat = MolecularCategory;
    type Qual = IonSelectivity;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(MolecularTaxonomyIsDAG),
            Box::new(Piezo1IsMechanosensitiveChannel),
            Box::new(TRPV4InEsophagus),
            Box::new(MechanosensitiveChannelsPassCalcium),
            Box::new(CausalGraphIsAsymmetric),
            Box::new(CausalGraphNoSelfCause),
            Box::new(MechanicalStressCausesMorphology),
            Box::new(AcidCausesVmemShift),
            Box::new(GlyRCausesHyperpolarization),
            Box::new(NernstPotentialsConsistent),
            Box::new(MolecularOppositionSymmetric),
            Box::new(MolecularOppositionIrreflexive),
            Box::new(MolecularMereologyNoCycles),
            Box::new(MolecularContextDeterministic),
            Box::new(MolecularContextTrueAmbiguity),
            Box::new(KvDualRole),
            Box::new(Piezo1DualRole),
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::mereology::MereologyCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    // -- Axiom tests --

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(
            MolecularTaxonomyIsDAG.holds(),
            "{}",
            MolecularTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_piezo1_is_mechanosensitive_channel() {
        assert!(
            Piezo1IsMechanosensitiveChannel.holds(),
            "{}",
            Piezo1IsMechanosensitiveChannel.description()
        );
    }

    #[test]
    fn test_trpv4_in_esophagus() {
        assert!(
            TRPV4InEsophagus.holds(),
            "{}",
            TRPV4InEsophagus.description()
        );
    }

    #[test]
    fn test_mechanosensitive_channels_pass_calcium() {
        assert!(
            MechanosensitiveChannelsPassCalcium.holds(),
            "{}",
            MechanosensitiveChannelsPassCalcium.description()
        );
    }

    #[test]
    fn test_causal_graph_is_asymmetric() {
        assert!(
            CausalGraphIsAsymmetric.holds(),
            "{}",
            CausalGraphIsAsymmetric.description()
        );
    }

    #[test]
    fn test_causal_graph_no_self_cause() {
        assert!(
            CausalGraphNoSelfCause.holds(),
            "{}",
            CausalGraphNoSelfCause.description()
        );
    }

    #[test]
    fn test_mechanical_stress_causes_morphology() {
        assert!(
            MechanicalStressCausesMorphology.holds(),
            "{}",
            MechanicalStressCausesMorphology.description()
        );
    }

    #[test]
    fn test_acid_causes_vmem_shift() {
        assert!(
            AcidCausesVmemShift.holds(),
            "{}",
            AcidCausesVmemShift.description()
        );
    }

    #[test]
    fn test_glyr_causes_hyperpolarization() {
        assert!(
            GlyRCausesHyperpolarization.holds(),
            "{}",
            GlyRCausesHyperpolarization.description()
        );
    }

    #[test]
    fn test_nernst_potentials_consistent() {
        assert!(
            NernstPotentialsConsistent.holds(),
            "{}",
            NernstPotentialsConsistent.description()
        );
    }

    // -- Opposition tests --

    #[test]
    fn test_molecular_opposition_symmetric() {
        assert!(
            MolecularOppositionSymmetric.holds(),
            "{}",
            MolecularOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_molecular_opposition_irreflexive() {
        assert!(
            MolecularOppositionIrreflexive.holds(),
            "{}",
            MolecularOppositionIrreflexive.description()
        );
    }

    #[test]
    fn test_sodium_opposes_potassium() {
        use MolecularEntity::*;
        assert!(opposition::are_opposed::<MolecularOpposition>(
            &Sodium, &Potassium
        ));
        assert!(opposition::are_opposed::<MolecularOpposition>(
            &Potassium, &Sodium
        ));
    }

    #[test]
    fn test_calcium_opposes_chloride() {
        use MolecularEntity::*;
        assert!(opposition::are_opposed::<MolecularOpposition>(
            &Calcium, &Chloride
        ));
    }

    #[test]
    fn test_nav_opposes_kv() {
        use MolecularEntity::*;
        assert!(opposition::are_opposed::<MolecularOpposition>(&Nav, &Kv));
    }

    #[test]
    fn test_sodium_does_not_oppose_calcium() {
        use MolecularEntity::*;
        assert!(!opposition::are_opposed::<MolecularOpposition>(
            &Sodium, &Calcium
        ));
    }

    #[test]
    fn test_molecular_opposites_query() {
        use MolecularEntity::*;
        let opps = opposition::opposites::<MolecularOpposition>(&Sodium);
        assert_eq!(opps, vec![Potassium]);
    }

    // -- Category law tests --

    #[test]
    fn test_molecular_category_laws() {
        check_category_laws::<MolecularCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<MolecularTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<MechanotransductionCausalGraph>>().unwrap();
    }

    // -- Taxonomy depth tests --

    #[test]
    fn test_taxonomy_depth_piezo1() {
        use MolecularEntity::*;
        let ancestors = taxonomy::ancestors::<MolecularTaxonomy>(&Piezo1);
        // Piezo1 → Mechanosensitive → IonChannel (depth 2)
        assert!(ancestors.contains(&Mechanosensitive));
        assert!(ancestors.contains(&IonChannel));
        assert_eq!(ancestors.len(), 2);
    }

    #[test]
    fn test_taxonomy_depth_nav() {
        use MolecularEntity::*;
        let ancestors = taxonomy::ancestors::<MolecularTaxonomy>(&Nav);
        // Nav → VoltageGated → IonChannel (depth 2)
        assert!(ancestors.contains(&VoltageGated));
        assert!(ancestors.contains(&IonChannel));
        assert_eq!(ancestors.len(), 2);
    }

    #[test]
    fn test_taxonomy_ion_descendants() {
        use MolecularEntity::*;
        let descendants = taxonomy::descendants::<MolecularTaxonomy>(&Ion);
        assert_eq!(descendants.len(), 5);
        assert!(descendants.contains(&Sodium));
        assert!(descendants.contains(&Potassium));
        assert!(descendants.contains(&Calcium));
        assert!(descendants.contains(&Chloride));
        assert!(descendants.contains(&Proton));
    }

    // -- Channel selectivity consistency --

    #[test]
    fn test_channel_selectivity_consistency() {
        // Every channel with a selectivity should select an ion that is-a Ion
        for entity in MolecularEntity::variants() {
            if let Some(ion) = IonSelectivity.get(&entity) {
                assert!(
                    taxonomy::is_a::<MolecularTaxonomy>(&ion, &MolecularEntity::Ion),
                    "{:?} selects {:?} which is not an Ion",
                    entity,
                    ion
                );
            }
        }
    }

    #[test]
    fn test_channel_activation_consistency() {
        // Every channel with an activation mechanism should be is-a IonChannel
        for entity in MolecularEntity::variants() {
            if ChannelActivation.get(&entity).is_some() {
                assert!(
                    taxonomy::is_a::<MolecularTaxonomy>(&entity, &MolecularEntity::IonChannel),
                    "{:?} has an activation mechanism but is not an IonChannel",
                    entity
                );
            }
        }
    }

    // -- Causal chain length --

    #[test]
    fn test_causal_chain_mechanical_to_morphology() {
        use CausalEvent::*;
        // MechanicalStress has many transitive effects
        let effects = causation::effects_of::<MechanotransductionCausalGraph>(&MechanicalStress);
        // Should reach: Piezo1Opening, TRPV4Opening, CollagenPiezoelectric,
        //               CalciumInflux, VmemShift, GeneExpression, MorphologicalChange
        assert!(
            effects.len() >= 7,
            "mechanical stress should have at least 7 transitive effects, got {}",
            effects.len()
        );
    }

    #[test]
    fn test_causal_chain_gap_junction_to_vmem() {
        use CausalEvent::*;
        let effects = causation::effects_of::<MechanotransductionCausalGraph>(&Cx43Upregulation);
        // Cx43Upregulation → GapJunctionFormation → BioelectricCoupling → VmemShift
        //                     → GeneExpression → MorphologicalChange
        assert!(effects.contains(&GapJunctionFormation));
        assert!(effects.contains(&BioelectricCoupling));
        assert!(effects.contains(&VmemShift));
        assert!(effects.contains(&GeneExpression));
        assert!(effects.contains(&MorphologicalChange));
    }

    #[test]
    fn test_vmem_shift_has_multiple_causes() {
        use CausalEvent::*;
        let causes = causation::causes_of::<MechanotransductionCausalGraph>(&VmemShift);
        // VmemShift is caused by: CalciumInflux, KvInhibition, ChlorideInflux,
        //                          BioelectricCoupling, plus their transitive causes
        assert!(causes.contains(&CalciumInflux));
        assert!(causes.contains(&KvInhibition));
        assert!(causes.contains(&ChlorideInflux));
        assert!(causes.contains(&BioelectricCoupling));
    }

    // -- Quality tests --

    #[test]
    fn test_ion_charges() {
        use MolecularEntity::*;
        assert_eq!(IonCharge.get(&Sodium), Some(1));
        assert_eq!(IonCharge.get(&Potassium), Some(1));
        assert_eq!(IonCharge.get(&Calcium), Some(2));
        assert_eq!(IonCharge.get(&Chloride), Some(-1));
        assert_eq!(IonCharge.get(&Proton), Some(1));
        // Non-ions return None
        assert_eq!(IonCharge.get(&Nav), None);
    }

    #[test]
    fn test_equilibrium_potentials() {
        use MolecularEntity::*;
        assert_eq!(EquilibriumPotential.get(&Sodium), Some(67.0));
        assert_eq!(EquilibriumPotential.get(&Potassium), Some(-90.0));
        assert_eq!(EquilibriumPotential.get(&Calcium), Some(131.0));
        assert_eq!(EquilibriumPotential.get(&Chloride), Some(-70.0));
        assert_eq!(EquilibriumPotential.get(&Proton), Some(-24.0));
    }

    #[test]
    fn test_expressed_in_esophagus() {
        use MolecularEntity::*;
        let expressed: Vec<MolecularEntity> = MolecularEntity::variants()
            .into_iter()
            .filter(|e| ExpressedInEsophagus.get(e) == Some(true))
            .collect();
        assert!(expressed.contains(&Piezo1));
        assert!(expressed.contains(&TRPV4));
        assert!(expressed.contains(&Kv));
        assert!(expressed.contains(&Cx26));
        assert!(expressed.contains(&Cx43));
        assert!(expressed.contains(&Collagen));
        assert!(expressed.contains(&Mucin));
        assert_eq!(expressed.len(), 7);
    }

    #[test]
    fn test_entity_count() {
        assert_eq!(MolecularEntity::variants().len(), 27);
    }

    #[test]
    fn test_causal_event_count() {
        assert_eq!(CausalEvent::variants().len(), 15);
    }

    #[test]
    fn test_ontology_validates() {
        MolecularOntology::validate().unwrap();
    }

    fn arb_molecular_entity() -> impl Strategy<Value = MolecularEntity> {
        (0..MolecularEntity::variants().len()).prop_map(|i| MolecularEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_channel_has_selectivity_if_activated(entity in arb_molecular_entity()) {
            // Every entity with an activation mechanism also has ion selectivity
            if ChannelActivation.get(&entity).is_some() {
                prop_assert!(IonSelectivity.get(&entity).is_some());
            }
        }

        #[test]
        fn prop_ion_has_charge(entity in arb_molecular_entity()) {
            // Every ion has a defined charge
            if taxonomy::is_a::<MolecularTaxonomy>(&entity, &MolecularEntity::Ion) && entity != MolecularEntity::Ion {
                prop_assert!(IonCharge.get(&entity).is_some());
            }
        }
    }

    // -- Mereology tests --

    #[test]
    fn test_molecular_mereology_no_cycles() {
        assert!(
            MolecularMereologyNoCycles.holds(),
            "{}",
            MolecularMereologyNoCycles.description()
        );
    }

    #[test]
    fn test_mereology_category_laws() {
        check_category_laws::<MereologyCategory<MolecularMereology>>().unwrap();
    }

    #[test]
    fn test_ion_channel_has_ion() {
        use MolecularEntity::*;
        let parts = mereology::parts_of::<MolecularMereology>(&IonChannel);
        assert!(parts.contains(&Ion));
    }

    #[test]
    fn test_voltage_gated_has_nav_kv_cav() {
        use MolecularEntity::*;
        let parts = mereology::parts_of::<MolecularMereology>(&VoltageGated);
        assert!(parts.contains(&Nav));
        assert!(parts.contains(&Kv));
        assert!(parts.contains(&Cav));
    }

    #[test]
    fn test_mechanosensitive_has_piezo1_piezo2_trpv4() {
        use MolecularEntity::*;
        let parts = mereology::parts_of::<MolecularMereology>(&Mechanosensitive);
        assert!(parts.contains(&Piezo1));
        assert!(parts.contains(&Piezo2));
        assert!(parts.contains(&TRPV4));
    }

    #[test]
    fn test_ligand_gated_has_glyr_gaba_a() {
        use MolecularEntity::*;
        let parts = mereology::parts_of::<MolecularMereology>(&LigandGated);
        assert!(parts.contains(&GlyR));
        assert!(parts.contains(&GABA_A));
    }

    #[test]
    fn test_gap_junction_has_cx26_cx43() {
        use MolecularEntity::*;
        let parts = mereology::parts_of::<MolecularMereology>(&GapJunction);
        assert!(parts.contains(&Cx26));
        assert!(parts.contains(&Cx43));
    }

    #[test]
    fn test_ion_channel_transitively_has_nav() {
        use MolecularEntity::*;
        // IonChannel has-a VoltageGated (via taxonomy? No, via mereology: VoltageGated has-a Nav)
        // But IonChannel does not directly have VoltageGated in mereology
        // IonChannel has Ion, VoltageGated has Nav/Kv/Cav
        // These are independent mereology branches
        let parts = mereology::parts_of::<MolecularMereology>(&IonChannel);
        assert!(parts.contains(&Ion));
    }

    // -- Context tests (adjunction-discovered dual roles) --

    #[test]
    fn test_context_deterministic() {
        assert!(MolecularContextDeterministic.holds());
    }

    #[test]
    fn test_context_true_ambiguity() {
        assert!(MolecularContextTrueAmbiguity.holds());
    }

    #[test]
    fn test_kv_dual_role() {
        assert!(KvDualRole.holds());
    }

    #[test]
    fn test_piezo1_dual_role() {
        assert!(Piezo1DualRole.holds());
    }

    #[test]
    fn test_cx43_dual_role() {
        use FunctionalContext::*;
        use FunctionalRole::*;
        use MolecularEntity::*;
        // Cx43: gap junction component (constitutive) vs modulation target (therapeutic)
        assert_eq!(
            context::resolve::<MolecularFunctionalContext>(&Cx43, &Constitutive),
            Some(InterCellularChannel)
        );
        assert_eq!(
            context::resolve::<MolecularFunctionalContext>(&Cx43, &Therapeutic),
            Some(TherapeuticTarget)
        );
    }

    #[test]
    fn test_collagen_dual_role() {
        use FunctionalContext::*;
        use FunctionalRole::*;
        use MolecularEntity::*;
        // Collagen: structural scaffold (constitutive) vs piezoelectric sensor (therapeutic)
        // This is the Fukada & Yasuda 1957 finding: collagen IS structural AND piezoelectric
        assert_eq!(
            context::resolve::<MolecularFunctionalContext>(&Collagen, &Constitutive),
            Some(StructuralScaffold)
        );
        assert_eq!(
            context::resolve::<MolecularFunctionalContext>(&Collagen, &Therapeutic),
            Some(MechanicalSensor)
        );
    }

    #[test]
    fn test_ambiguous_entities() {
        // All molecules in the context map should be truly ambiguous (2+ roles)
        let ambiguous = context::ambiguous_entities::<MolecularFunctionalContext>();
        // Every entity in the context map has exactly 2 resolutions
        assert!(!ambiguous.is_empty());
        for entity in &ambiguous {
            let interps = context::interpretations::<MolecularFunctionalContext>(entity);
            assert!(
                interps.len() >= 2,
                "{:?} should have at least 2 interpretations, has {}",
                entity,
                interps.len()
            );
        }
    }
}

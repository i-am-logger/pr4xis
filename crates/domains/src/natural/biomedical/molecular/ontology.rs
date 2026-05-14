//! Molecular biology — ions, ion channels, structural proteins, and signaling
//! molecules for bioelectric and mechanotransduction modelling.
//!
//! Models the five physiologically relevant ions (Na⁺, K⁺, Ca²⁺, Cl⁻, H⁺),
//! the four families of ion channels (voltage-gated Nav/Kv/Cav,
//! mechanosensitive Piezo1/Piezo2/TRPV4, ligand-gated GlyR/GABA_A, gap
//! junctions Cx26/Cx43), structural proteins (collagen, mucin), and
//! signaling molecules (calcium signal, NO). Causal events span the
//! mechanotransduction pathway (mechanical stress → Piezo opening → Ca²⁺
//! influx → Vmem shift → gene expression → morphological change), the
//! acid → Kv inhibition → Vmem shift pathway, the GlyR → chloride influx →
//! Vmem shift pathway, and the Cx43 upregulation → gap-junction formation
//! → bioelectric coupling pathway. Per `feedback_one_ontology_per_module`
//! the original split between `MolecularEntity` and `CausalEvent` has been
//! merged: events are first-class concepts subsumed by the
//! `MechanotransductionEvent` umbrella.
//!
//! # Literature
//!
//! - **Alberts et al. (2015)** *Molecular Biology of the Cell*, 6th ed.,
//!   Garland Science — canonical reference for ions, ion channels, gap
//!   junctions, structural proteins, and signaling molecules.
//! - **Hille (2001)** *Ion Channels of Excitable Membranes*, 3rd ed.,
//!   Sinauer — canonical reference for Nernst equilibrium potentials,
//!   voltage-gated / ligand-gated / mechanosensitive channel classes, and
//!   ion selectivity.
//! - **Coste et al. (2010)** "Piezo1 and Piezo2 are essential components of
//!   distinct mechanically activated cation channels", *Science*
//!   330:55-60 — discovery of the Piezo family (2021 Nobel Prize to
//!   Patapoutian).
//! - **Mihara et al. (2011)** "Involvement of TRPV2 activation in
//!   intestinal movement", *J. Neurosci.* — TRPV4 expression in esophageal
//!   epithelium and its role in mechanosensation.
//! - **Fukada & Yasuda (1957)** "On the Piezoelectric Effect of Bone",
//!   *J. Phys. Soc. Japan* 12:1158-1162 — collagen as the piezoelectric
//!   substrate of bone matrix.
//! - **Inose et al. (2009)** "Connexin 26 and 43 expression in
//!   gastrointestinal epithelial cells" — Cx26/Cx43 in esophageal gap
//!   junctions.
//! - **Khalbuss et al. (1995)** "Acid-induced inhibition of K⁺ channels in
//!   esophageal epithelial cells" — H⁺-mediated Kv inhibition driving Vmem
//!   shifts.

// `GABA_A` is the published IUPHAR/BPS receptor nomenclature
// (Hille 2001 §6; Alexander et al. 2023, Br. J. Pharmacol. 180:S23–S144).
// Per Praxis literature-fidelity rule, keep the canonical name rather than
// rename it to `GabaA`.
#![allow(non_camel_case_types)]
use pr4xis::category::{Arrow, Category};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Molecular",
    source: "Alberts et al. (2015) Molecular Biology of the Cell 6th ed.; Hille (2001) Ion Channels of Excitable Membranes 3rd ed.; Coste et al. (2010) Science 330:55-60; Mihara et al. (2011); Fukada & Yasuda (1957) J. Phys. Soc. Japan 12:1158-1162; Inose et al. (2009); Khalbuss et al. (1995).",

    concepts: [
        // === Ions (Hille 2001 §1) ===
        Sodium,
        Potassium,
        Calcium,
        Chloride,
        Proton,

        // === Voltage-gated channels (Hille 2001 §2-5) ===
        Nav,
        Kv,
        Cav,

        // === Mechanosensitive channels (Coste et al. 2010; Mihara et al. 2011) ===
        Piezo1,
        Piezo2,
        TRPV4,

        // === Ligand-gated channels (Hille 2001 §6) ===
        GlyR,
        GABA_A,

        // === Gap junctions (Inose et al. 2009) ===
        Cx26,
        Cx43,

        // === Structural proteins (Alberts et al. 2015) ===
        Collagen,
        Mucin,

        // === Signaling molecules (Alberts et al. 2015 ch. 15) ===
        CalciumSignal,
        NitricOxide,

        // === Abstract umbrellas ===
        Ion,
        IonChannel,
        VoltageGated,
        Mechanosensitive,
        LigandGated,
        GapJunction,
        Protein,
        SignalingMolecule,
        MechanotransductionEvent,

        // === Causal events ===
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
    ],

    labels: {
        Sodium: ("en", "Sodium (Na⁺)",
            "Hille (2001) §1: monovalent cation, primary depolarizing ion; E_Na ≈ +67 mV at 37°C."),
        Potassium: ("en", "Potassium (K⁺)",
            "Hille (2001) §1: monovalent cation, primary determinant of resting potential; E_K ≈ -90 mV at 37°C."),
        Calcium: ("en", "Calcium (Ca²⁺)",
            "Hille (2001) §1; Alberts (2015) §15: divalent cation, intracellular second messenger; E_Ca ≈ +131 mV at 37°C."),
        Chloride: ("en", "Chloride (Cl⁻)",
            "Hille (2001) §1: monovalent anion, primary inhibitory carrier; E_Cl ≈ -70 mV at 37°C."),
        Proton: ("en", "Proton (H⁺)",
            "Hille (2001) §1: monovalent cation; E_H ≈ -24 mV at 37°C with pHᵢ=7.0, pHₒ=7.4."),

        Nav: ("en", "Voltage-gated Na⁺ channel (Nav)",
            "Hille (2001) §2: voltage-gated channel responsible for the depolarizing phase of the action potential."),
        Kv: ("en", "Voltage-gated K⁺ channel (Kv)",
            "Hille (2001) §5: voltage-gated channel responsible for action-potential repolarization and resting-Vmem setting."),
        Cav: ("en", "Voltage-gated Ca²⁺ channel (Cav)",
            "Hille (2001) §4: voltage-gated channel coupling membrane depolarization to intracellular Ca²⁺ signalling."),

        Piezo1: ("en", "Piezo1",
            "Coste et al. (2010): mechanically activated cation channel; primary mammalian mechanosensor."),
        Piezo2: ("en", "Piezo2",
            "Coste et al. (2010): mechanically activated cation channel; principal sensor for touch and proprioception."),
        TRPV4: ("en", "TRPV4",
            "Mihara et al. (2011): osmo- and mechanosensitive TRP-family cation channel expressed in esophageal epithelium."),

        GlyR: ("en", "Glycine receptor (GlyR)",
            "Hille (2001) §6: ligand-gated Cl⁻ channel; principal mediator of fast inhibitory neurotransmission in the spinal cord."),
        GABA_A: ("en", "GABA_A receptor",
            "Hille (2001) §6: ligand-gated Cl⁻ channel; principal mediator of fast inhibitory neurotransmission in the brain."),

        Cx26: ("en", "Connexin 26 (Cx26)",
            "Inose et al. (2009): gap-junction connexin expressed in esophageal epithelium."),
        Cx43: ("en", "Connexin 43 (Cx43)",
            "Inose et al. (2009): the most widely-expressed gap-junction connexin, central to bioelectric coupling."),

        Collagen: ("en", "Collagen",
            "Alberts et al. (2015): triple-helical extracellular-matrix protein; Fukada & Yasuda (1957) — piezoelectric."),
        Mucin: ("en", "Mucin",
            "Alberts et al. (2015): heavily glycosylated mucosal protein; protective epithelial barrier."),

        CalciumSignal: ("en", "Ca²⁺ signal",
            "Alberts et al. (2015) §15: intracellular Ca²⁺ rise functioning as a second messenger."),
        NitricOxide: ("en", "Nitric oxide (NO)",
            "Alberts et al. (2015) §15: diffusible gaseous signalling molecule; activates guanylate cyclase."),

        Ion: ("en", "Ion",
            "Hille (2001) §1: umbrella for the charged atomic / molecular species that carry ionic current."),
        IonChannel: ("en", "Ion channel",
            "Hille (2001) §2: umbrella for transmembrane proteins that catalyse selective passive ion flux."),
        VoltageGated: ("en", "Voltage-gated channel",
            "Hille (2001) §2-5: umbrella for ion channels gated by membrane potential."),
        Mechanosensitive: ("en", "Mechanosensitive channel",
            "Coste et al. (2010); Mihara et al. (2011): umbrella for ion channels gated by mechanical force."),
        LigandGated: ("en", "Ligand-gated channel",
            "Hille (2001) §6: umbrella for ion channels gated by binding of a specific ligand."),
        GapJunction: ("en", "Gap junction",
            "Inose et al. (2009); Alberts et al. (2015): umbrella for connexin-based intercellular channels."),
        Protein: ("en", "Protein",
            "Alberts et al. (2015): umbrella for the protein concepts in this ontology (channels and structural proteins)."),
        SignalingMolecule: ("en", "Signaling molecule",
            "Alberts et al. (2015) §15: umbrella for diffusible second-messenger species (Ca²⁺ signal, NO)."),
        MechanotransductionEvent: ("en", "Mechanotransduction event",
            "Coste et al. (2010); Hille (2001): umbrella for time-extended causal events in the mechanotransduction / bioelectric pathway."),

        MechanicalStress: ("en", "Mechanical stress",
            "Coste et al. (2010): force per unit area applied to a cell or tissue; the upstream trigger of mechanotransduction."),
        Piezo1Opening: ("en", "Piezo1 opening",
            "Coste et al. (2010): force-induced conformational change of Piezo1, permitting cation flux."),
        TRPV4Opening: ("en", "TRPV4 opening",
            "Mihara et al. (2011): osmotic/mechanical gating of TRPV4, permitting Ca²⁺ influx."),
        CollagenPiezoelectric: ("en", "Collagen piezoelectric response",
            "Fukada & Yasuda (1957): mechanical stress on collagen generates local electric polarization."),
        CalciumInflux: ("en", "Ca²⁺ influx",
            "Hille (2001) §4: net flow of Ca²⁺ into the cytosol through open channels."),
        VmemShift: ("en", "Vmem shift",
            "Hille (2001) §1: change in resting membrane potential away from its baseline."),
        GeneExpression: ("en", "Gene expression",
            "Alberts et al. (2015) §6: production of mRNA/protein from a gene, often triggered by Ca²⁺/Vmem-dependent signalling."),
        MorphologicalChange: ("en", "Morphological change",
            "Alberts et al. (2015) §19: alteration of cell or tissue shape downstream of bioelectric/genetic signalling."),
        AcidExposure: ("en", "Acid exposure",
            "Khalbuss et al. (1995): low-pH stimulation of an epithelial surface (e.g. esophageal reflux)."),
        KvInhibition: ("en", "Kv inhibition",
            "Khalbuss et al. (1995): proton-mediated block of voltage-gated K⁺ channels."),
        GlyRActivation: ("en", "GlyR activation",
            "Hille (2001) §6: glycine-bound opening of the glycine receptor Cl⁻ channel."),
        ChlorideInflux: ("en", "Cl⁻ influx",
            "Hille (2001) §1: net flow of Cl⁻ into the cytosol through open channels (typically hyperpolarizing)."),
        Cx43Upregulation: ("en", "Cx43 upregulation",
            "Inose et al. (2009): increased expression of connexin 43."),
        GapJunctionFormation: ("en", "Gap-junction formation",
            "Inose et al. (2009): assembly of paired connexons between adjacent cells creating an intercellular channel."),
        BioelectricCoupling: ("en", "Bioelectric coupling",
            "Inose et al. (2009): electrical continuity between cells via gap-junctional ion flow."),
    },

    is_a: [
        // Ions
        (Sodium, Ion),
        (Potassium, Ion),
        (Calcium, Ion),
        (Chloride, Ion),
        (Proton, Ion),

        // Channel family umbrellas
        (VoltageGated, IonChannel),
        (Mechanosensitive, IonChannel),
        (LigandGated, IonChannel),
        (GapJunction, IonChannel),

        // Voltage-gated
        (Nav, VoltageGated),
        (Kv, VoltageGated),
        (Cav, VoltageGated),

        // Mechanosensitive
        (Piezo1, Mechanosensitive),
        (Piezo2, Mechanosensitive),
        (TRPV4, Mechanosensitive),

        // Ligand-gated
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

        // Events under MechanotransductionEvent umbrella
        (MechanicalStress, MechanotransductionEvent),
        (Piezo1Opening, MechanotransductionEvent),
        (TRPV4Opening, MechanotransductionEvent),
        (CollagenPiezoelectric, MechanotransductionEvent),
        (CalciumInflux, MechanotransductionEvent),
        (VmemShift, MechanotransductionEvent),
        (GeneExpression, MechanotransductionEvent),
        (MorphologicalChange, MechanotransductionEvent),
        (AcidExposure, MechanotransductionEvent),
        (KvInhibition, MechanotransductionEvent),
        (GlyRActivation, MechanotransductionEvent),
        (ChlorideInflux, MechanotransductionEvent),
        (Cx43Upregulation, MechanotransductionEvent),
        (GapJunctionFormation, MechanotransductionEvent),
        (BioelectricCoupling, MechanotransductionEvent),
    ],

    causes: [
        // Coste et al. (2010): mechanical stress opens Piezo and TRPV4.
        (MechanicalStress, Piezo1Opening),
        (MechanicalStress, TRPV4Opening),
        // Fukada & Yasuda (1957): mechanical stress drives the collagen
        // piezoelectric response.
        (MechanicalStress, CollagenPiezoelectric),
        // Hille (2001) §4: Piezo/TRPV4 opening produces Ca²⁺ influx.
        (Piezo1Opening, CalciumInflux),
        (TRPV4Opening, CalciumInflux),
        // Hille (2001) §1: Ca²⁺ influx depolarizes Vmem.
        (CalciumInflux, VmemShift),
        // Alberts (2015) §6: Vmem-mediated signalling alters gene expression.
        (VmemShift, GeneExpression),
        // Alberts (2015) §19: altered gene expression drives morphological change.
        (GeneExpression, MorphologicalChange),
        // Khalbuss et al. (1995): acid → Kv inhibition → Vmem shift.
        (AcidExposure, KvInhibition),
        (KvInhibition, VmemShift),
        // Hille (2001) §6: GlyR opens Cl⁻ channels → Cl⁻ influx → Vmem shift.
        (GlyRActivation, ChlorideInflux),
        (ChlorideInflux, VmemShift),
        // Inose et al. (2009): Cx43 upregulation → gap-junction formation
        // → bioelectric coupling → Vmem shift.
        (Cx43Upregulation, GapJunctionFormation),
        (GapJunctionFormation, BioelectricCoupling),
        (BioelectricCoupling, VmemShift),
    ],

    opposes: [
        // Hille (2001) §1: Na⁺ and K⁺ are the depolarizing / repolarizing
        // primary ions of the resting and action potentials.
        (Sodium, Potassium),
        (Potassium, Sodium),
        // Hille (2001) §1: Ca²⁺ (excitatory/depolarizing) vs Cl⁻
        // (inhibitory/hyperpolarizing) signalling ions.
        (Calcium, Chloride),
        (Chloride, Calcium),
        // Hille (2001) §2/§5: Nav (depolarizing) vs Kv (repolarizing).
        (Nav, Kv),
        (Kv, Nav),
    ],
}

// ---------------------------------------------------------------------------
// Backward-compatibility aliases (transitional — pre-1.0)
// ---------------------------------------------------------------------------
//
// Many crate-internal modules (the formal/meta diagnostics tree, the
// adjunctions / composition_tests scaffolding, and the partner functor files
// that have not yet had their target side migrated) reference the old
// hand-rolled names `MolecularEntity` and `MolecularCategoryRelationKind`.
// These aliases keep those modules compiling. They will be removed once every
// consumer has been migrated.

/// Transitional alias for the proc-macro-generated `MolecularConcept`.
pub type MolecularEntity = MolecularConcept;
/// Transitional alias for the proc-macro-generated `MolecularRelationKind`.
pub type MolecularCategoryRelationKind = MolecularRelationKind;

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Ionic charge in elementary-charge units.
///
/// Hille (2001) §1: the canonical valences of the five physiological ions.
#[derive(Debug, Clone)]
pub struct IonCharge;

impl Quality for IonCharge {
    type Individual = MolecularConcept;
    type Value = i32;

    fn get(&self, c: &MolecularConcept) -> Option<i32> {
        use MolecularConcept::*;
        match c {
            Sodium | Potassium | Proton => Some(1),
            Calcium => Some(2),
            Chloride => Some(-1),
            _ => None,
        }
    }
}

/// Nernst equilibrium potential (mV) at 37 °C with physiological gradients.
///
/// Hille (2001) §1 table 1.1.
#[derive(Debug, Clone)]
pub struct EquilibriumPotential;

impl Quality for EquilibriumPotential {
    type Individual = MolecularConcept;
    type Value = f64;

    fn get(&self, c: &MolecularConcept) -> Option<f64> {
        use MolecularConcept::*;
        match c {
            Sodium => Some(67.0),
            Potassium => Some(-90.0),
            Calcium => Some(131.0), // [Ca²⁺]ₒ=2 mM, [Ca²⁺]ᵢ=100 nM, 37°C
            Chloride => Some(-70.0),
            Proton => Some(-24.0), // pHᵢ=7.0, pHₒ=7.4, 37°C
            _ => None,
        }
    }
}

/// Channel activation mechanism (Hille 2001 §2-6; Coste et al. 2010).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivationMechanism {
    Voltage,
    Mechanical,
    Ligand,
    GapJunctionCoupling,
}

/// Quality: how a channel is gated.
#[derive(Debug, Clone)]
pub struct ChannelActivation;

impl Quality for ChannelActivation {
    type Individual = MolecularConcept;
    type Value = ActivationMechanism;

    fn get(&self, c: &MolecularConcept) -> Option<ActivationMechanism> {
        use ActivationMechanism::*;
        use MolecularConcept::*;
        match c {
            Nav | Kv | Cav => Some(Voltage),
            Piezo1 | Piezo2 | TRPV4 => Some(Mechanical),
            GlyR | GABA_A => Some(Ligand),
            Cx26 | Cx43 => Some(GapJunctionCoupling),
            _ => None,
        }
    }
}

/// Quality: which ion the channel primarily conducts.
///
/// Hille (2001); Coste et al. (2010): Nav→Na, Kv→K, Cav/Piezo/TRPV4→Ca,
/// GlyR/GABA_A→Cl, connexins are non-selective but pass Ca.
#[derive(Debug, Clone)]
pub struct IonSelectivity;

impl Quality for IonSelectivity {
    type Individual = MolecularConcept;
    type Value = MolecularConcept;

    fn get(&self, c: &MolecularConcept) -> Option<MolecularConcept> {
        use MolecularConcept::*;
        match c {
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

/// Quality: whether the concept is expressed in esophageal epithelium.
///
/// Mihara et al. (2011); Inose et al. (2009); Khalbuss et al. (1995):
/// experimentally confirmed esophageal expression.
#[derive(Debug, Clone)]
pub struct ExpressedInEsophagus;

impl Quality for ExpressedInEsophagus {
    type Individual = MolecularConcept;
    type Value = bool;

    fn get(&self, c: &MolecularConcept) -> Option<bool> {
        use MolecularConcept::*;
        match c {
            Piezo1 | TRPV4 | Kv | Cx26 | Cx43 | Collagen | Mucin => Some(true),
            Sodium | Potassium | Calcium | Chloride | Proton | Nav | Cav | Piezo2 | GlyR
            | GABA_A | CalciumSignal | NitricOxide => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Ontology + domain axioms
// ---------------------------------------------------------------------------

impl Ontology for MolecularOntology {
    type Cat = MolecularCategory;
    type Qual = IonSelectivity;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(Piezo1IsMechanosensitiveChannel));
        axioms.push(Box::new(TRPV4InEsophagus));
        axioms.push(Box::new(MechanosensitiveChannelsPassCalcium));
        axioms.push(Box::new(MechanicalStressCausesMorphology));
        axioms.push(Box::new(AcidCausesVmemShift));
        axioms.push(Box::new(GlyRCausesHyperpolarization));
        axioms.push(Box::new(NernstPotentialsConsistent));
        axioms
    }
}

/// Helper: does there exist a direct `Subsumption` edge `child → parent`?
fn is_a(child: MolecularConcept, parent: MolecularConcept) -> bool {
    MolecularCategory::morphisms().iter().any(|m| {
        m.kind() == MolecularRelationKind::Subsumption
            && m.source() == child
            && m.target() == parent
    })
}

/// Helper: transitive closure of `Causation` from `cause`.
fn causal_effects(cause: MolecularConcept) -> Vec<MolecularConcept> {
    let direct: Vec<(MolecularConcept, MolecularConcept)> = MolecularCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == MolecularRelationKind::Causation)
        .map(|m| (m.source(), m.target()))
        .collect();
    let mut visited: Vec<MolecularConcept> = Vec::new();
    let mut frontier = vec![cause];
    while let Some(c) = frontier.pop() {
        for (s, t) in &direct {
            if *s == c && !visited.contains(t) {
                visited.push(*t);
                frontier.push(*t);
            }
        }
    }
    visited
}

/// Axiom: Piezo1 is-a Mechanosensitive is-a IonChannel.
///
/// Coste et al. (2010) — Piezo1 is the founding mammalian mechanosensitive
/// channel; the OBO-RO transitivity of subsumption then gives Piezo1 → IonChannel.
pub struct Piezo1IsMechanosensitiveChannel;

impl Axiom for Piezo1IsMechanosensitiveChannel {
    fn verify(&self) -> Verdict {
        use MolecularConcept::*;
        if is_a(Piezo1, Mechanosensitive) && is_a(Mechanosensitive, IonChannel) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "Piezo1IsMechanosensitiveChannel",
        "Piezo1 is-a Mechanosensitive is-a IonChannel",
        "Coste et al. (2010) Science 330:55-60"
    );
}
pr4xis::register_axiom!(
    Piezo1IsMechanosensitiveChannel,
    "Coste et al. (2010) Science 330:55-60"
);

/// Axiom: TRPV4 is mechanosensitive and expressed in the esophagus.
pub struct TRPV4InEsophagus;

impl Axiom for TRPV4InEsophagus {
    fn verify(&self) -> Verdict {
        use MolecularConcept::*;
        if is_a(TRPV4, Mechanosensitive) && ExpressedInEsophagus.get(&TRPV4) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TRPV4InEsophagus",
        "TRPV4 is mechanosensitive and is expressed in esophageal epithelium",
        "Mihara et al. (2011) Involvement of TRPV2 activation in intestinal movement"
    );
}
pr4xis::register_axiom!(
    TRPV4InEsophagus,
    "Mihara et al. (2011) Involvement of TRPV2 activation in intestinal movement"
);

/// Axiom: all three mechanosensitive channels conduct Ca²⁺.
///
/// Coste et al. (2010): Piezo1/Piezo2 are non-selective cation channels with
/// significant Ca²⁺ permeability; Mihara et al. (2011) — TRPV4 is Ca²⁺-permeable.
pub struct MechanosensitiveChannelsPassCalcium;

impl Axiom for MechanosensitiveChannelsPassCalcium {
    fn verify(&self) -> Verdict {
        use MolecularConcept::*;
        if [Piezo1, Piezo2, TRPV4]
            .iter()
            .all(|c| IonSelectivity.get(c) == Some(Calcium))
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MechanosensitiveChannelsPassCalcium",
        "All mechanosensitive channels (Piezo1, Piezo2, TRPV4) conduct calcium",
        "Coste et al. (2010) Science 330:55-60; Mihara et al. (2011)"
    );
}
pr4xis::register_axiom!(
    MechanosensitiveChannelsPassCalcium,
    "Coste et al. (2010) Science 330:55-60; Mihara et al. (2011)"
);

/// Axiom: mechanical stress transitively causes morphological change.
///
/// Coste et al. (2010); Alberts et al. (2015): the full mechanotransduction
/// chain — stress → Piezo opening → Ca²⁺ influx → Vmem shift → gene expression
/// → morphological change.
pub struct MechanicalStressCausesMorphology;

impl Axiom for MechanicalStressCausesMorphology {
    fn verify(&self) -> Verdict {
        if causal_effects(MolecularConcept::MechanicalStress)
            .contains(&MolecularConcept::MorphologicalChange)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MechanicalStressCausesMorphology",
        "Mechanical stress transitively causes morphological change via the mechanotransduction pathway",
        "Coste et al. (2010) Science 330:55-60; Alberts et al. (2015) Molecular Biology of the Cell 6th ed."
    );
}
pr4xis::register_axiom!(
    MechanicalStressCausesMorphology,
    "Coste et al. (2010) Science 330:55-60"
);

/// Axiom: acid exposure transitively causes Vmem shift via Kv inhibition.
pub struct AcidCausesVmemShift;

impl Axiom for AcidCausesVmemShift {
    fn verify(&self) -> Verdict {
        let effs = causal_effects(MolecularConcept::AcidExposure);
        if effs.contains(&MolecularConcept::KvInhibition)
            && effs.contains(&MolecularConcept::VmemShift)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AcidCausesVmemShift",
        "Acid exposure causes Vmem shift via Kv inhibition",
        "Khalbuss et al. (1995) Acid-induced inhibition of K⁺ channels in esophageal epithelial cells"
    );
}
pr4xis::register_axiom!(
    AcidCausesVmemShift,
    "Khalbuss et al. (1995) Acid-induced inhibition of K⁺ channels in esophageal epithelial cells"
);

/// Axiom: GlyR activation causes Vmem shift via chloride influx.
pub struct GlyRCausesHyperpolarization;

impl Axiom for GlyRCausesHyperpolarization {
    fn verify(&self) -> Verdict {
        let effs = causal_effects(MolecularConcept::GlyRActivation);
        if effs.contains(&MolecularConcept::ChlorideInflux)
            && effs.contains(&MolecularConcept::VmemShift)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GlyRCausesHyperpolarization",
        "GlyR activation causes Vmem shift via chloride influx",
        "Hille (2001) Ion Channels of Excitable Membranes §6"
    );
}
pr4xis::register_axiom!(
    GlyRCausesHyperpolarization,
    "Hille (2001) Ion Channels of Excitable Membranes §6"
);

/// Axiom: Nernst equilibrium potentials have the textbook signs.
///
/// Hille (2001) §1: E_K < 0, E_Na > 0, E_Ca > 0, E_Cl < 0, E_H < 0 at 37 °C
/// with mammalian gradients.
pub struct NernstPotentialsConsistent;

impl Axiom for NernstPotentialsConsistent {
    fn verify(&self) -> Verdict {
        use MolecularConcept::*;
        let e = EquilibriumPotential;
        let ok = e.get(&Potassium).unwrap_or(0.0) < 0.0
            && e.get(&Sodium).unwrap_or(0.0) > 0.0
            && e.get(&Calcium).unwrap_or(0.0) > 0.0
            && e.get(&Chloride).unwrap_or(0.0) < 0.0
            && e.get(&Proton).unwrap_or(0.0) < 0.0;
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NernstPotentialsConsistent",
        "Nernst equilibrium potentials have the textbook physiological signs (K<0, Na>0, Ca>0, Cl<0, H<0)",
        "Hille (2001) Ion Channels of Excitable Membranes §1"
    );
}
pr4xis::register_axiom!(
    NernstPotentialsConsistent,
    "Hille (2001) Ion Channels of Excitable Membranes §1"
);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<MolecularCategory>();
    }

    #[test]
    fn ontology_validates() {
        MolecularOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn concept_count() {
        // 5 ions + 3 voltage-gated + 3 mechanosensitive + 2 ligand-gated + 2 gap
        // + 2 structural + 2 signaling + 9 umbrellas + 15 events = 43
        assert_eq!(MolecularConcept::variants().len(), 43);
    }

    // -- Domain-axiom tests --

    #[test]
    fn piezo1_is_mechanosensitive_channel_axiom() {
        assert!(Piezo1IsMechanosensitiveChannel.verify().is_ok());
    }

    #[test]
    fn trpv4_in_esophagus_axiom() {
        assert!(TRPV4InEsophagus.verify().is_ok());
    }

    #[test]
    fn mechanosensitive_channels_pass_calcium_axiom() {
        assert!(MechanosensitiveChannelsPassCalcium.verify().is_ok());
    }

    #[test]
    fn mechanical_stress_causes_morphology_axiom() {
        assert!(MechanicalStressCausesMorphology.verify().is_ok());
    }

    #[test]
    fn acid_causes_vmem_shift_axiom() {
        assert!(AcidCausesVmemShift.verify().is_ok());
    }

    #[test]
    fn glyr_causes_hyperpolarization_axiom() {
        assert!(GlyRCausesHyperpolarization.verify().is_ok());
    }

    #[test]
    fn nernst_potentials_consistent_axiom() {
        assert!(NernstPotentialsConsistent.verify().is_ok());
    }

    // -- Subsumption-kind tests --

    fn subsumptions() -> Vec<(MolecularConcept, MolecularConcept)> {
        MolecularCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MolecularRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect()
    }

    #[test]
    fn ions_subsume_under_ion() {
        let subs = subsumptions();
        for c in [
            MolecularConcept::Sodium,
            MolecularConcept::Potassium,
            MolecularConcept::Calcium,
            MolecularConcept::Chloride,
            MolecularConcept::Proton,
        ] {
            assert!(
                subs.contains(&(c, MolecularConcept::Ion)),
                "{:?} should subsume under Ion",
                c
            );
        }
    }

    #[test]
    fn channel_families_subsume_under_ion_channel() {
        let subs = subsumptions();
        for c in [
            MolecularConcept::VoltageGated,
            MolecularConcept::Mechanosensitive,
            MolecularConcept::LigandGated,
            MolecularConcept::GapJunction,
        ] {
            assert!(
                subs.contains(&(c, MolecularConcept::IonChannel)),
                "{:?} should subsume under IonChannel",
                c
            );
        }
    }

    #[test]
    fn events_subsume_under_mechanotransduction_event() {
        let subs = subsumptions();
        for ev in [
            MolecularConcept::MechanicalStress,
            MolecularConcept::Piezo1Opening,
            MolecularConcept::TRPV4Opening,
            MolecularConcept::CalciumInflux,
            MolecularConcept::VmemShift,
            MolecularConcept::GeneExpression,
            MolecularConcept::MorphologicalChange,
            MolecularConcept::AcidExposure,
            MolecularConcept::KvInhibition,
            MolecularConcept::GlyRActivation,
            MolecularConcept::ChlorideInflux,
            MolecularConcept::Cx43Upregulation,
            MolecularConcept::GapJunctionFormation,
            MolecularConcept::BioelectricCoupling,
        ] {
            assert!(
                subs.contains(&(ev, MolecularConcept::MechanotransductionEvent)),
                "{:?} should subsume under MechanotransductionEvent",
                ev
            );
        }
    }

    // -- Causation-kind tests --

    fn direct_causes() -> Vec<(MolecularConcept, MolecularConcept)> {
        MolecularCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MolecularRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect()
    }

    #[test]
    fn mechanical_stress_causes_piezo1_opening() {
        assert!(direct_causes().contains(&(
            MolecularConcept::MechanicalStress,
            MolecularConcept::Piezo1Opening
        )));
    }

    #[test]
    fn calcium_influx_causes_vmem_shift() {
        assert!(
            direct_causes()
                .contains(&(MolecularConcept::CalciumInflux, MolecularConcept::VmemShift))
        );
    }

    #[test]
    fn causal_chain_mechanical_to_morphology_length() {
        let effs = causal_effects(MolecularConcept::MechanicalStress);
        // Should reach at minimum: Piezo1Opening, TRPV4Opening, CollagenPiezoelectric,
        // CalciumInflux, VmemShift, GeneExpression, MorphologicalChange = 7 events.
        assert!(
            effs.len() >= 7,
            "MechanicalStress should have at least 7 transitive effects, got {}",
            effs.len()
        );
    }

    #[test]
    fn vmem_shift_has_multiple_causes() {
        // VmemShift is reached transitively from CalciumInflux, KvInhibition,
        // ChlorideInflux, BioelectricCoupling, etc.
        let direct = direct_causes();
        let causes_of_vmem: Vec<MolecularConcept> = direct
            .iter()
            .filter(|(_, t)| *t == MolecularConcept::VmemShift)
            .map(|(s, _)| *s)
            .collect();
        assert!(causes_of_vmem.contains(&MolecularConcept::CalciumInflux));
        assert!(causes_of_vmem.contains(&MolecularConcept::KvInhibition));
        assert!(causes_of_vmem.contains(&MolecularConcept::ChlorideInflux));
        assert!(causes_of_vmem.contains(&MolecularConcept::BioelectricCoupling));
    }

    // -- Opposition-kind tests --

    fn oppositions() -> Vec<(MolecularConcept, MolecularConcept)> {
        MolecularCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MolecularRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect()
    }

    #[test]
    fn sodium_opposes_potassium() {
        let opps = oppositions();
        assert!(opps.contains(&(MolecularConcept::Sodium, MolecularConcept::Potassium)));
        assert!(opps.contains(&(MolecularConcept::Potassium, MolecularConcept::Sodium)));
    }

    #[test]
    fn calcium_opposes_chloride() {
        let opps = oppositions();
        assert!(opps.contains(&(MolecularConcept::Calcium, MolecularConcept::Chloride)));
    }

    #[test]
    fn nav_opposes_kv() {
        let opps = oppositions();
        assert!(opps.contains(&(MolecularConcept::Nav, MolecularConcept::Kv)));
    }

    // -- Quality tests --

    #[test]
    fn ion_charges() {
        use MolecularConcept::*;
        assert_eq!(IonCharge.get(&Sodium), Some(1));
        assert_eq!(IonCharge.get(&Potassium), Some(1));
        assert_eq!(IonCharge.get(&Calcium), Some(2));
        assert_eq!(IonCharge.get(&Chloride), Some(-1));
        assert_eq!(IonCharge.get(&Proton), Some(1));
        assert_eq!(IonCharge.get(&Nav), None);
    }

    #[test]
    fn equilibrium_potentials() {
        use MolecularConcept::*;
        assert_eq!(EquilibriumPotential.get(&Sodium), Some(67.0));
        assert_eq!(EquilibriumPotential.get(&Potassium), Some(-90.0));
        assert_eq!(EquilibriumPotential.get(&Calcium), Some(131.0));
        assert_eq!(EquilibriumPotential.get(&Chloride), Some(-70.0));
        assert_eq!(EquilibriumPotential.get(&Proton), Some(-24.0));
    }

    #[test]
    fn expressed_in_esophagus() {
        use MolecularConcept::*;
        let expressed: Vec<_> = MolecularConcept::variants()
            .into_iter()
            .filter(|c| ExpressedInEsophagus.get(c) == Some(true))
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
    fn channel_activation_consistency() {
        // Every concept with an activation mechanism must subsume under IonChannel.
        for c in MolecularConcept::variants() {
            if ChannelActivation.get(&c).is_some() {
                let to_channel = is_a(c, MolecularConcept::IonChannel);
                let umbrella = matches!(
                    c,
                    MolecularConcept::VoltageGated
                        | MolecularConcept::Mechanosensitive
                        | MolecularConcept::LigandGated
                        | MolecularConcept::GapJunction
                );
                assert!(
                    to_channel || umbrella,
                    "{:?} has activation mechanism but is not under IonChannel",
                    c
                );
            }
        }
    }

    #[test]
    fn channel_selectivity_consistency() {
        // Every concept with an ion selectivity selects an Ion.
        for c in MolecularConcept::variants() {
            if let Some(ion) = IonSelectivity.get(&c) {
                assert!(
                    is_a(ion, MolecularConcept::Ion),
                    "{:?} selects {:?} which is not subsumed under Ion",
                    c,
                    ion
                );
            }
        }
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = MolecularConcept> {
        proptest::sample::select(MolecularConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in MolecularCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in MolecularOntology::axioms() {
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
            let variants: Vec<_> = MolecularConcept::variants();
            for m in MolecularCategory::morphisms() {
                if m.kind() == MolecularRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = MolecularCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == MolecularRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(
                    opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back",
                    a,
                    b
                );
            }
        }

        #[test]
        fn prop_channel_has_selectivity_if_activated(c in arb_concept()) {
            if ChannelActivation.get(&c).is_some() {
                prop_assert!(IonSelectivity.get(&c).is_some());
            }
        }

        #[test]
        fn prop_ion_has_charge(c in arb_concept()) {
            if is_a(c, MolecularConcept::Ion) && c != MolecularConcept::Ion {
                prop_assert!(IonCharge.get(&c).is_some());
            }
        }
    }
}

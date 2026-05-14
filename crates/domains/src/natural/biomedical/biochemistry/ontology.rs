//! Biochemistry — signaling cascades and energy metabolism relevant to
//! bioelectric repair.
//!
//! Models the Ca²⁺ → Calmodulin → CaMKII → CREB → gene-expression pathway,
//! the PKC branch, the nitric-oxide / vasodilation branch, and ATP/ADP
//! energy metabolism. Per `feedback_one_ontology_per_module` the original
//! split between `BiochemistryEntity` and `BiochemistryCausalEvent` has
//! been merged: events are first-class concepts subsumed by the
//! `BiochemicalEvent` umbrella.
//!
//! # Literature
//!
//! - **Bhatt, Zhang & Snyder (2000)** *Annual Review of Neuroscience* 23,
//!   417–445 — calmodulin–CaMKII activation by Ca²⁺.
//! - **Sheng, Thompson & Greenberg (1990)** *Science* 252(5011), 1427–1430
//!   — Ca²⁺-CREB phosphorylation linking electrical activity to gene
//!   expression.
//! - **Bhargava (2012)** *Pharmacological Reviews* — cAMP and IP3 as
//!   intracellular second messengers.
//! - **Ignarro et al. (1987)** *PNAS* 84(24), 9265–9269 — endothelium-
//!   derived relaxing factor identified as nitric oxide.
//! - **Krebs (1957)** *Endeavour* 16, 125–129 — ATP / phosphorylation
//!   cascades.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Biochemistry",
    source: "Bhatt et al. (2000) CaMKII activation by calmodulin, Annu. Rev. Neurosci. 23; Sheng et al. (1990) Ca-CREB phosphorylation, Science 252(5011); Bhargava (2012) cAMP/IP3 as second messengers, Pharm. Rev.; Ignarro et al. (1987) NO as signaling molecule, PNAS 84(24); Krebs (1957) ATP in phosphorylation cascades, Endeavour 16",

    concepts: [
        // === Signaling molecules ===
        CalciumIon,
        Calmodulin,
        CaMKII,
        ProteinKinaseC,
        CREB,
        NitricOxide,

        // === Second messengers ===
        CAMP,
        IP3,

        // === Processes ===
        SignalTransduction,
        PhosphorylationCascade,
        GeneTranscription,
        ProteinSynthesis,
        SecondMessenger,

        // === Metabolic ===
        ATP,
        ADP,
        Glycolysis,
        OxidativePhosphorylation,

        // === Abstract umbrellas ===
        SignalingMolecule,
        BiochemicalProcess,
        EnergyMetabolite,
        BiochemicalEvent,

        // === Causal events (merged from BiochemistryCausalEvent) ===
        CalciumEntry,
        CalmodulinActivation,
        CaMKIIPhosphorylation,
        CREBActivation,
        GeneExpressionChange,
        ProteinSynthesisChange,
        PKCActivation,
        DownstreamSignaling,
        NOSynthaseActivation,
        NOProduction,
        ATPHydrolysis,
        EnergyRelease,
    ],

    labels: {
        CalciumIon: ("en", "Calcium ion",
            "Bhargava (2012): Ca²⁺ — the canonical intracellular second messenger; coordinates calmodulin and triggers CaMKII / PKC / NOS cascades."),
        Calmodulin: ("en", "Calmodulin",
            "Bhatt et al. (2000): Ca²⁺-binding regulatory protein that activates CaMKII and downstream signaling enzymes."),
        CaMKII: ("en", "CaMKII",
            "Bhatt et al. (2000): Ca²⁺/calmodulin-dependent protein kinase II; autophosphorylates on Thr286 to sustain activation."),
        ProteinKinaseC: ("en", "Protein kinase C",
            "Bhargava (2012): family of serine/threonine kinases activated by Ca²⁺ and diacylglycerol (DAG)."),
        CREB: ("en", "CREB",
            "Sheng et al. (1990): cAMP-response-element binding protein; Ca²⁺/CaMKII-phosphorylated CREB drives activity-dependent gene expression."),
        NitricOxide: ("en", "Nitric oxide",
            "Ignarro et al. (1987): diffusible gaseous signaling molecule produced by NO synthase; mediates vasodilation."),
        CAMP: ("en", "Cyclic AMP",
            "Bhargava (2012): 3',5'-cyclic AMP — intracellular second messenger downstream of Gs-coupled receptors."),
        IP3: ("en", "Inositol trisphosphate",
            "Bhargava (2012): IP3 — second messenger that releases Ca²⁺ from intracellular endoplasmic-reticulum stores."),

        SignalTransduction: ("en", "Signal transduction",
            "Bhatt et al. (2000): conversion of an extracellular signal into an intracellular response via receptor and effector cascades."),
        PhosphorylationCascade: ("en", "Phosphorylation cascade",
            "Krebs (1957): sequential ATP-driven phosphorylation of substrates; foundational mechanism of metabolic regulation."),
        GeneTranscription: ("en", "Gene transcription",
            "Sheng et al. (1990): synthesis of RNA from a DNA template; downstream effect of activity-dependent transcription factors like CREB."),
        ProteinSynthesis: ("en", "Protein synthesis",
            "Sheng et al. (1990): translation of mRNA into protein, completing the activity-dependent expression chain."),
        SecondMessenger: ("en", "Second messenger",
            "Bhargava (2012): intracellular relay molecule (Ca²⁺, cAMP, IP3, NO) that propagates a primary extracellular signal."),

        ATP: ("en", "ATP",
            "Krebs (1957): adenosine triphosphate — the cell's primary energy currency, charged form."),
        ADP: ("en", "ADP",
            "Krebs (1957): adenosine diphosphate — the discharged complement of ATP after hydrolysis of the γ-phosphate."),
        Glycolysis: ("en", "Glycolysis",
            "Krebs (1957) context: cytosolic anaerobic glucose-to-pyruvate pathway yielding net 2 ATP."),
        OxidativePhosphorylation: ("en", "Oxidative phosphorylation",
            "Krebs (1957) context: mitochondrial electron-transport-chain-coupled ATP synthesis under aerobic conditions."),

        SignalingMolecule: ("en", "Signaling molecule",
            "Bhatt et al. (2000): umbrella for molecules that carry intracellular or intercellular signals."),
        BiochemicalProcess: ("en", "Biochemical process",
            "Bhatt et al. (2000): umbrella for time-extended biochemical transformations (signaling, transcription, metabolism)."),
        EnergyMetabolite: ("en", "Energy metabolite",
            "Krebs (1957): umbrella for high-energy intermediates of cellular metabolism (ATP, ADP, ...)."),
        BiochemicalEvent: ("en", "Biochemical event",
            "Bhatt et al. (2000): umbrella for time-extended events in signaling cascades and energy turnover."),

        CalciumEntry: ("en", "Calcium entry",
            "Bhatt et al. (2000): Ca²⁺ enters the cytosol via voltage-gated or ligand-gated ion channels."),
        CalmodulinActivation: ("en", "Calmodulin activation",
            "Bhatt et al. (2000): Ca²⁺-loaded calmodulin adopts an activated conformation able to bind target enzymes."),
        CaMKIIPhosphorylation: ("en", "CaMKII phosphorylation",
            "Bhatt et al. (2000): Ca²⁺/calmodulin-dependent autophosphorylation of CaMKII on Thr286."),
        CREBActivation: ("en", "CREB activation",
            "Sheng et al. (1990): Ser133 phosphorylation of CREB enabling CBP recruitment."),
        GeneExpressionChange: ("en", "Gene expression change",
            "Sheng et al. (1990): activity-dependent change in transcript abundance downstream of CREB."),
        ProteinSynthesisChange: ("en", "Protein synthesis change",
            "Sheng et al. (1990): change in protein levels following altered transcription."),
        PKCActivation: ("en", "PKC activation",
            "Bhargava (2012): PKC activated by Ca²⁺ and DAG."),
        DownstreamSignaling: ("en", "Downstream signaling",
            "Bhargava (2012): PKC-driven phosphorylation of downstream substrates."),
        NOSynthaseActivation: ("en", "NO synthase activation",
            "Ignarro et al. (1987): Ca²⁺/calmodulin-dependent activation of NO synthase."),
        NOProduction: ("en", "Nitric oxide production",
            "Ignarro et al. (1987): NOS-catalysed synthesis of NO from L-arginine."),
        ATPHydrolysis: ("en", "ATP hydrolysis",
            "Krebs (1957): ATP → ADP + Pi releasing ~30.5 kJ/mol under standard conditions."),
        EnergyRelease: ("en", "Energy release",
            "Krebs (1957): the free-energy fraction made available by ATP hydrolysis for cellular work."),
    },

    is_a: [
        // Signaling molecules
        (CalciumIon, SignalingMolecule),
        (Calmodulin, SignalingMolecule),
        (CaMKII, SignalingMolecule),
        (ProteinKinaseC, SignalingMolecule),
        (CREB, SignalingMolecule),
        (NitricOxide, SignalingMolecule),
        (CAMP, SignalingMolecule),
        (IP3, SignalingMolecule),

        // Processes
        (SignalTransduction, BiochemicalProcess),
        (PhosphorylationCascade, BiochemicalProcess),
        (GeneTranscription, BiochemicalProcess),
        (ProteinSynthesis, BiochemicalProcess),
        (SecondMessenger, BiochemicalProcess),
        (Glycolysis, BiochemicalProcess),
        (OxidativePhosphorylation, BiochemicalProcess),

        // Metabolic
        (ATP, EnergyMetabolite),
        (ADP, EnergyMetabolite),

        // Events under the BiochemicalEvent umbrella
        (CalciumEntry, BiochemicalEvent),
        (CalmodulinActivation, BiochemicalEvent),
        (CaMKIIPhosphorylation, BiochemicalEvent),
        (CREBActivation, BiochemicalEvent),
        (GeneExpressionChange, BiochemicalEvent),
        (ProteinSynthesisChange, BiochemicalEvent),
        (PKCActivation, BiochemicalEvent),
        (DownstreamSignaling, BiochemicalEvent),
        (NOSynthaseActivation, BiochemicalEvent),
        (NOProduction, BiochemicalEvent),
        (ATPHydrolysis, BiochemicalEvent),
        (EnergyRelease, BiochemicalEvent),
    ],

    causes: [
        // Main cascade (Bhatt et al. 2000; Sheng et al. 1990):
        // Ca²⁺ entry → calmodulin → CaMKII → CREB → expression → translation.
        (CalciumEntry, CalmodulinActivation),
        (CalmodulinActivation, CaMKIIPhosphorylation),
        (CaMKIIPhosphorylation, CREBActivation),
        (CREBActivation, GeneExpressionChange),
        (GeneExpressionChange, ProteinSynthesisChange),
        // PKC branch (Bhargava 2012).
        (CalciumEntry, PKCActivation),
        (PKCActivation, DownstreamSignaling),
        // Nitric-oxide branch (Ignarro et al. 1987).
        (CalciumEntry, NOSynthaseActivation),
        (NOSynthaseActivation, NOProduction),
        // Energy metabolism (Krebs 1957).
        (ATPHydrolysis, EnergyRelease),
    ],

    opposes: [
        // ATP ↔ ADP — charged vs discharged energy currency (Krebs 1957).
        (ATP, ADP),
        (ADP, ATP),
        // Glycolysis ↔ Oxidative phosphorylation — anaerobic vs aerobic
        // ATP-producing pathways.
        (Glycolysis, OxidativePhosphorylation),
        (OxidativePhosphorylation, Glycolysis),
        // Fast covalent signaling vs slow transcriptional regulation.
        (PhosphorylationCascade, GeneTranscription),
        (GeneTranscription, PhosphorylationCascade),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: is the concept a second messenger?
///
/// Bhargava (2012) — Ca²⁺, cAMP, IP3, and NO are the canonical second
/// messengers carried by this ontology.
#[derive(Debug, Clone)]
pub struct IsSecondMessenger;

impl Quality for IsSecondMessenger {
    type Individual = BiochemistryConcept;
    type Value = bool;

    fn get(&self, c: &BiochemistryConcept) -> Option<bool> {
        use BiochemistryConcept::*;
        match c {
            CalciumIon | CAMP | IP3 | NitricOxide => Some(true),
            Calmodulin | CaMKII | ProteinKinaseC | CREB => Some(false),
            ATP | ADP => Some(false),
            _ => None,
        }
    }
}

/// Quality: is the concept a kinase?
///
/// Bhatt et al. (2000): CaMKII and PKC are serine/threonine kinases.
#[derive(Debug, Clone)]
pub struct IsKinase;

impl Quality for IsKinase {
    type Individual = BiochemistryConcept;
    type Value = bool;

    fn get(&self, c: &BiochemistryConcept) -> Option<bool> {
        use BiochemistryConcept::*;
        match c {
            CaMKII | ProteinKinaseC => Some(true),
            CalciumIon | Calmodulin | CREB | NitricOxide | CAMP | IP3 => Some(false),
            ATP | ADP => Some(false),
            _ => None,
        }
    }
}

/// Quality: does the process require ATP?
///
/// Krebs (1957): phosphorylation cascades and protein synthesis consume
/// ATP; gene transcription is energetically modest; canonical
/// signaling-relay events do not consume ATP themselves.
#[derive(Debug, Clone)]
pub struct RequiresATP;

impl Quality for RequiresATP {
    type Individual = BiochemistryConcept;
    type Value = bool;

    fn get(&self, c: &BiochemistryConcept) -> Option<bool> {
        use BiochemistryConcept::*;
        match c {
            PhosphorylationCascade | ProteinSynthesis => Some(true),
            SignalTransduction | GeneTranscription | SecondMessenger => Some(false),
            Glycolysis | OxidativePhosphorylation => Some(false),
            _ => None,
        }
    }
}

/// Characteristic time scale of biochemical processes.
///
/// Bhargava (2012) and Sheng et al. (1990): rapid Ca²⁺ signaling
/// (milliseconds), enzyme-cascade kinetics (seconds), metabolic flux
/// (minutes), and activity-dependent gene-expression / translation
/// (hours).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeScale {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
}

/// Quality: characteristic time scale of a process.
#[derive(Debug, Clone)]
pub struct ProcessTimeScale;

impl Quality for ProcessTimeScale {
    type Individual = BiochemistryConcept;
    type Value = TimeScale;

    fn get(&self, c: &BiochemistryConcept) -> Option<TimeScale> {
        use BiochemistryConcept::*;
        match c {
            SignalTransduction => Some(TimeScale::Milliseconds),
            PhosphorylationCascade => Some(TimeScale::Seconds),
            SecondMessenger => Some(TimeScale::Seconds),
            GeneTranscription => Some(TimeScale::Hours),
            ProteinSynthesis => Some(TimeScale::Hours),
            Glycolysis => Some(TimeScale::Minutes),
            OxidativePhosphorylation => Some(TimeScale::Minutes),
            _ => None,
        }
    }
}

/// Quality: is the process reversible on a short (signaling) time scale?
///
/// Bhargava (2012) and Sheng et al. (1990): kinase/phosphatase pairs make
/// phosphorylation reversible; second-messenger lifetimes are bounded by
/// phosphodiesterases; transcription and translation are difficult to
/// reverse on short timescales; committed metabolic steps are directional.
#[derive(Debug, Clone)]
pub struct IsReversible;

impl Quality for IsReversible {
    type Individual = BiochemistryConcept;
    type Value = bool;

    fn get(&self, c: &BiochemistryConcept) -> Option<bool> {
        use BiochemistryConcept::*;
        match c {
            PhosphorylationCascade => Some(true),
            SignalTransduction => Some(true),
            SecondMessenger => Some(true),
            GeneTranscription => Some(false),
            ProteinSynthesis => Some(false),
            Glycolysis => Some(false),
            OxidativePhosphorylation => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Ontology + domain axioms
// ---------------------------------------------------------------------------

impl Ontology for BiochemistryOntology {
    type Cat = BiochemistryCategory;
    type Qual = IsSecondMessenger;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(CalciumEntryCausesGeneExpression));
        axioms.push(Box::new(CalciumEntryCausesNOProduction));
        axioms.push(Box::new(CalciumIsSecondMessenger));
        axioms.push(Box::new(CaMKIIIsKinase));
        axioms.push(Box::new(PhosphorylationRequiresATP));
        axioms
    }
}

/// Helper: does a `Causation` edge (direct or transitive) exist from
/// `cause` to `effect`?
fn causes(cause: BiochemistryConcept, effect: BiochemistryConcept) -> bool {
    BiochemistryCategory::morphisms().iter().any(|m| {
        m.kind() == BiochemistryRelationKind::Causation
            && m.source() == cause
            && m.target() == effect
    })
}

/// Axiom: CalciumEntry transitively causes GeneExpressionChange via the
/// Bhatt (2000) and Sheng (1990) cascade.
pub struct CalciumEntryCausesGeneExpression;

impl Axiom for CalciumEntryCausesGeneExpression {
    fn verify(&self) -> Verdict {
        if causes(
            BiochemistryConcept::CalciumEntry,
            BiochemistryConcept::GeneExpressionChange,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CalciumEntryCausesGeneExpression",
        "Ca²⁺ entry transitively causes a CREB-mediated gene-expression change",
        "Bhatt et al. (2000) CaMKII activation by calmodulin, Annu. Rev. Neurosci. 23; Sheng et al. (1990) Ca-CREB phosphorylation, Science 252(5011)"
    );
}

pr4xis::register_axiom!(
    CalciumEntryCausesGeneExpression,
    "Bhatt et al. (2000) Annu. Rev. Neurosci. 23; Sheng et al. (1990) Science 252(5011)"
);

/// Axiom: CalciumEntry causes NO production through NOS activation
/// (Ignarro et al. 1987).
pub struct CalciumEntryCausesNOProduction;

impl Axiom for CalciumEntryCausesNOProduction {
    fn verify(&self) -> Verdict {
        if causes(
            BiochemistryConcept::CalciumEntry,
            BiochemistryConcept::NOSynthaseActivation,
        ) && causes(
            BiochemistryConcept::CalciumEntry,
            BiochemistryConcept::NOProduction,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CalciumEntryCausesNOProduction",
        "Ca²⁺ entry activates NO synthase and produces NO (vasodilation pathway)",
        "Ignarro et al. (1987) NO as signaling molecule, PNAS 84(24)"
    );
}

pr4xis::register_axiom!(
    CalciumEntryCausesNOProduction,
    "Ignarro et al. (1987) PNAS 84(24)"
);

/// Axiom: calcium ion is a second messenger (Bhargava 2012).
pub struct CalciumIsSecondMessenger;

impl Axiom for CalciumIsSecondMessenger {
    fn verify(&self) -> Verdict {
        if IsSecondMessenger.get(&BiochemistryConcept::CalciumIon) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CalciumIsSecondMessenger",
        "Calcium ion is the canonical intracellular second messenger",
        "Bhargava (2012) cAMP/IP3 as second messengers, Pharm. Rev."
    );
}

pr4xis::register_axiom!(
    CalciumIsSecondMessenger,
    "Bhargava (2012) cAMP/IP3 as second messengers, Pharm. Rev."
);

/// Axiom: CaMKII is a kinase (Bhatt et al. 2000).
pub struct CaMKIIIsKinase;

impl Axiom for CaMKIIIsKinase {
    fn verify(&self) -> Verdict {
        if IsKinase.get(&BiochemistryConcept::CaMKII) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CaMKIIIsKinase",
        "CaMKII is a Ca²⁺/calmodulin-dependent protein kinase",
        "Bhatt et al. (2000) CaMKII activation by calmodulin, Annu. Rev. Neurosci. 23"
    );
}

pr4xis::register_axiom!(
    CaMKIIIsKinase,
    "Bhatt et al. (2000) Annu. Rev. Neurosci. 23"
);

/// Axiom: phosphorylation cascades require ATP (Krebs 1957).
pub struct PhosphorylationRequiresATP;

impl Axiom for PhosphorylationRequiresATP {
    fn verify(&self) -> Verdict {
        if RequiresATP.get(&BiochemistryConcept::PhosphorylationCascade) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PhosphorylationRequiresATP",
        "Phosphorylation cascades transfer the γ-phosphate of ATP and therefore consume ATP",
        "Krebs (1957) ATP in phosphorylation cascades, Endeavour 16"
    );
}

pr4xis::register_axiom!(
    PhosphorylationRequiresATP,
    "Krebs (1957) ATP in phosphorylation cascades, Endeavour 16"
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
        assert_category_laws::<BiochemistryCategory>();
    }

    #[test]
    fn ontology_validates() {
        BiochemistryOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn concept_count() {
        // 6 signaling molecules + 2 second messengers + 5 processes
        // + 4 metabolic + 4 abstract umbrellas + 12 events = 33.
        assert_eq!(BiochemistryConcept::variants().len(), 33);
    }

    // -- Domain axiom tests --

    #[test]
    fn calcium_entry_causes_gene_expression_axiom() {
        assert!(CalciumEntryCausesGeneExpression.verify().is_ok());
    }

    #[test]
    fn calcium_entry_causes_no_production_axiom() {
        assert!(CalciumEntryCausesNOProduction.verify().is_ok());
    }

    #[test]
    fn calcium_is_second_messenger_axiom() {
        assert!(CalciumIsSecondMessenger.verify().is_ok());
    }

    #[test]
    fn camkii_is_kinase_axiom() {
        assert!(CaMKIIIsKinase.verify().is_ok());
    }

    #[test]
    fn phosphorylation_requires_atp_axiom() {
        assert!(PhosphorylationRequiresATP.verify().is_ok());
    }

    // -- Subsumption tests --

    #[test]
    fn signaling_molecules_subsume() {
        let subs: Vec<_> = BiochemistryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiochemistryRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for c in [
            BiochemistryConcept::CalciumIon,
            BiochemistryConcept::Calmodulin,
            BiochemistryConcept::CaMKII,
            BiochemistryConcept::CREB,
            BiochemistryConcept::NitricOxide,
        ] {
            assert!(
                subs.contains(&(c, BiochemistryConcept::SignalingMolecule)),
                "{:?} should subsume under SignalingMolecule",
                c
            );
        }
    }

    #[test]
    fn events_subsume_under_biochemical_event() {
        let subs: Vec<_> = BiochemistryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiochemistryRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for ev in [
            BiochemistryConcept::CalciumEntry,
            BiochemistryConcept::CalmodulinActivation,
            BiochemistryConcept::CREBActivation,
            BiochemistryConcept::ATPHydrolysis,
            BiochemistryConcept::NOProduction,
        ] {
            assert!(
                subs.contains(&(ev, BiochemistryConcept::BiochemicalEvent)),
                "{:?} should subsume under BiochemicalEvent",
                ev
            );
        }
    }

    // -- Causation tests --

    #[test]
    fn calcium_entry_full_main_cascade() {
        for c in [
            BiochemistryConcept::CalmodulinActivation,
            BiochemistryConcept::CaMKIIPhosphorylation,
            BiochemistryConcept::CREBActivation,
            BiochemistryConcept::GeneExpressionChange,
            BiochemistryConcept::ProteinSynthesisChange,
        ] {
            assert!(
                causes(BiochemistryConcept::CalciumEntry, c),
                "CalciumEntry should transitively cause {:?}",
                c
            );
        }
    }

    #[test]
    fn calcium_entry_pkc_branch() {
        assert!(causes(
            BiochemistryConcept::CalciumEntry,
            BiochemistryConcept::PKCActivation
        ));
        assert!(causes(
            BiochemistryConcept::CalciumEntry,
            BiochemistryConcept::DownstreamSignaling
        ));
    }

    #[test]
    fn atp_hydrolysis_causes_energy_release() {
        assert!(causes(
            BiochemistryConcept::ATPHydrolysis,
            BiochemistryConcept::EnergyRelease
        ));
    }

    // -- Opposition tests --

    #[test]
    fn atp_opposes_adp() {
        let opps: Vec<_> = BiochemistryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiochemistryRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(BiochemistryConcept::ATP, BiochemistryConcept::ADP)));
        assert!(opps.contains(&(BiochemistryConcept::ADP, BiochemistryConcept::ATP)));
    }

    #[test]
    fn glycolysis_opposes_oxphos() {
        let opps: Vec<_> = BiochemistryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiochemistryRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            BiochemistryConcept::Glycolysis,
            BiochemistryConcept::OxidativePhosphorylation,
        )));
    }

    // -- Quality tests --

    #[test]
    fn second_messengers_via_quality() {
        use BiochemistryConcept::*;
        assert_eq!(IsSecondMessenger.get(&CalciumIon), Some(true));
        assert_eq!(IsSecondMessenger.get(&CAMP), Some(true));
        assert_eq!(IsSecondMessenger.get(&IP3), Some(true));
        assert_eq!(IsSecondMessenger.get(&NitricOxide), Some(true));
        assert_eq!(IsSecondMessenger.get(&CaMKII), Some(false));
    }

    #[test]
    fn kinases_via_quality() {
        use BiochemistryConcept::*;
        assert_eq!(IsKinase.get(&CaMKII), Some(true));
        assert_eq!(IsKinase.get(&ProteinKinaseC), Some(true));
        assert_eq!(IsKinase.get(&CalciumIon), Some(false));
    }

    #[test]
    fn requires_atp_via_quality() {
        use BiochemistryConcept::*;
        assert_eq!(RequiresATP.get(&PhosphorylationCascade), Some(true));
        assert_eq!(RequiresATP.get(&ProteinSynthesis), Some(true));
        assert_eq!(RequiresATP.get(&SignalTransduction), Some(false));
    }

    #[test]
    fn time_scales_via_quality() {
        use BiochemistryConcept::*;
        assert_eq!(
            ProcessTimeScale.get(&SignalTransduction),
            Some(TimeScale::Milliseconds)
        );
        assert_eq!(
            ProcessTimeScale.get(&PhosphorylationCascade),
            Some(TimeScale::Seconds)
        );
        assert_eq!(
            ProcessTimeScale.get(&GeneTranscription),
            Some(TimeScale::Hours)
        );
        assert_eq!(ProcessTimeScale.get(&Glycolysis), Some(TimeScale::Minutes));
    }

    #[test]
    fn reversibility_via_quality() {
        use BiochemistryConcept::*;
        assert_eq!(IsReversible.get(&PhosphorylationCascade), Some(true));
        assert_eq!(IsReversible.get(&GeneTranscription), Some(false));
        assert_eq!(IsReversible.get(&ProteinSynthesis), Some(false));
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = BiochemistryConcept> {
        proptest::sample::select(BiochemistryConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in BiochemistryCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in BiochemistryOntology::axioms() {
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
            let variants: Vec<_> = BiochemistryConcept::variants();
            for m in BiochemistryCategory::morphisms() {
                if m.kind() == BiochemistryRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = BiochemistryCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == BiochemistryRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }

        #[test]
        fn prop_second_messenger_is_signaling_molecule(c in arb_concept()) {
            // Every second messenger should subsume under SignalingMolecule.
            if IsSecondMessenger.get(&c) == Some(true) {
                let subs: Vec<_> = BiochemistryCategory::morphisms()
                    .iter()
                    .filter(|m| m.kind() == BiochemistryRelationKind::Subsumption)
                    .map(|m| (m.source(), m.target()))
                    .collect();
                prop_assert!(
                    subs.contains(&(c, BiochemistryConcept::SignalingMolecule)),
                    "{:?} is a second messenger but does not subsume under SignalingMolecule",
                    c
                );
            }
        }

        #[test]
        fn prop_kinase_is_signaling_molecule(c in arb_concept()) {
            if IsKinase.get(&c) == Some(true) {
                let subs: Vec<_> = BiochemistryCategory::morphisms()
                    .iter()
                    .filter(|m| m.kind() == BiochemistryRelationKind::Subsumption)
                    .map(|m| (m.source(), m.target()))
                    .collect();
                prop_assert!(
                    subs.contains(&(c, BiochemistryConcept::SignalingMolecule)),
                    "{:?} is a kinase but does not subsume under SignalingMolecule",
                    c
                );
            }
        }
    }
}

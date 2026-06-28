//! Tissue-repair immunology ontology.
//!
//! Models the inflammatory cascade, macrophage polarisation (M1/M2),
//! cytokine signalling, and the causal chain from tissue injury to repair
//! or fibrosis. The key scientific insight modelled here: mechanical
//! stimulation (whole-body vibration) promotes M1→M2 macrophage transition,
//! shifting the immune response from pro-inflammatory to pro-repair
//! (Weinheimer-Haus 2014; Yu 2019).
//!
//! Per `feedback_one_ontology_per_module` the original split between
//! `ImmunologyEntity` and `ImmunologyEvent` has been merged into one
//! concept list, with events subsumed by the `ImmunologyEvent` umbrella.
//!
//! # Literature
//!
//! - **Murphy (2017)** *Janeway's Immunobiology*, 10th ed., Garland Science
//!   — canonical reference for immune-cell taxonomy (macrophage M1/M2,
//!   neutrophil, T cell, monocyte, mast cell), cytokine classification
//!   (pro- vs anti-inflammatory; TNFα, IL-6, IL-10, TGF-β), and the
//!   inflammatory-cascade time scales (acute = hours; chronic = weeks).
//! - **Abbas, Lichtman & Pillai (2021)** *Cellular and Molecular
//!   Immunology*, 10th ed. — companion reference for inflammation,
//!   resolution, and the macrophage polarisation states.
//! - **Weinheimer-Haus, Judex, Ennis, Koh (2014)** "Low-intensity vibration
//!   improves angiogenesis and wound healing in diabetic mice",
//!   *PLoS ONE* 9(3):e91355 — 45 Hz whole-body vibration shifts
//!   macrophage polarisation toward M2 in diabetic mouse wounds.
//! - **Yu, Touyz et al. (2019)** PMID:31247969 — whole-body vibration
//!   induces omental macrophage polarisation shift, confirming
//!   vibration modulates the immune response.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Immunology",
    source: "Murphy (2017) Janeway's Immunobiology 10th ed.; Abbas, Lichtman & Pillai (2021) Cellular and Molecular Immunology 10th ed.; Weinheimer-Haus et al. (2014) PLoS ONE 9(3):e91355; Yu et al. (2019) PMID:31247969",

    concepts: [
        // === Cells (Murphy 2017 §2 — leukocyte classification) ===
        MacrophageM1,
        MacrophageM2,
        Neutrophil,
        TCell,
        Monocyte,
        MastCell,
        Fibroblast,

        // === Inflammatory states (Murphy 2017 §3) ===
        AcuteInflammation,
        ChronicInflammation,
        Resolution,
        Fibrosis,
        TissueRepair,

        // === Cytokines (Murphy 2017 §3) ===
        ProInflammatoryCytokine,
        AntiInflammatoryCytokine,
        TNFAlpha,
        IL6,
        IL10,
        TGFBeta,

        // === Abstract umbrellas ===
        ImmuneCell,
        StromalCell,
        InflammatoryState,
        Cytokine,
        ImmunologyEvent,

        // === Causal events (merged from ImmunologyEvent — Murphy 2017
        //      §3 inflammatory cascade plus Weinheimer-Haus 2014 vibration
        //      intervention) ===
        TissueInjury,
        NeutrophilRecruitment,
        AcuteInflammationOnset,
        MonocyteRecruitment,
        M1Polarization,
        ProInflammatoryResponse,
        M1ToM2Transition,
        AntiInflammatoryResponse,
        TissueRemodeling,
        RepairCompletion,
        ChronicStimulus,
        FailedResolution,
        FibrosisProgression,
        MechanicalStimulation,
    ],

    labels: {
        MacrophageM1: ("en", "Macrophage (M1)",
            "Murphy (2017) §3: classically activated, pro-inflammatory macrophage phenotype."),
        MacrophageM2: ("en", "Macrophage (M2)",
            "Murphy (2017) §3: alternatively activated, pro-repair macrophage phenotype."),
        Neutrophil: ("en", "Neutrophil",
            "Murphy (2017) §2: first-responder polymorphonuclear leukocyte."),
        TCell: ("en", "T cell",
            "Murphy (2017) §2: thymus-derived adaptive lymphocyte."),
        Monocyte: ("en", "Monocyte",
            "Murphy (2017) §2: circulating mononuclear precursor of tissue macrophages."),
        MastCell: ("en", "Mast cell",
            "Murphy (2017) §2: tissue-resident granulocyte mediating allergic and inflammatory responses."),
        Fibroblast: ("en", "Fibroblast",
            "Abbas et al. (2021): stromal connective-tissue cell producing collagen during repair."),

        AcuteInflammation: ("en", "Acute inflammation",
            "Murphy (2017) §3: rapid-onset (hours) inflammatory state."),
        ChronicInflammation: ("en", "Chronic inflammation",
            "Murphy (2017) §3: prolonged (weeks) inflammatory state with persistent leukocyte infiltration."),
        Resolution: ("en", "Resolution",
            "Abbas et al. (2021): coordinated termination of inflammation and return to homeostasis."),
        Fibrosis: ("en", "Fibrosis",
            "Abbas et al. (2021): pathological replacement of normal tissue by collagenous scar."),
        TissueRepair: ("en", "Tissue repair",
            "Abbas et al. (2021): coordinated regenerative response restoring tissue integrity."),

        ProInflammatoryCytokine: ("en", "Pro-inflammatory cytokine",
            "Murphy (2017) §3: cytokine class amplifying inflammation."),
        AntiInflammatoryCytokine: ("en", "Anti-inflammatory cytokine",
            "Murphy (2017) §3: cytokine class dampening inflammation."),
        TNFAlpha: ("en", "TNF-α",
            "Murphy (2017) §3: canonical pro-inflammatory cytokine — Tumor Necrosis Factor alpha."),
        IL6: ("en", "IL-6",
            "Murphy (2017) §3: Interleukin 6 — acute-phase pro-inflammatory cytokine."),
        IL10: ("en", "IL-10",
            "Murphy (2017) §3: Interleukin 10 — canonical anti-inflammatory cytokine."),
        TGFBeta: ("en", "TGF-β",
            "Murphy (2017) §3: Transforming Growth Factor β — anti-inflammatory and pro-fibrotic."),

        ImmuneCell: ("en", "Immune cell (abstract)",
            "Murphy (2017) §2: umbrella for cells of the immune system."),
        StromalCell: ("en", "Stromal cell (abstract)",
            "Abbas et al. (2021): umbrella for connective-tissue support cells (fibroblasts) interacting with the immune system."),
        InflammatoryState: ("en", "Inflammatory state (abstract)",
            "Murphy (2017) §3: umbrella for tissue-level inflammatory or repair states."),
        Cytokine: ("en", "Cytokine (abstract)",
            "Murphy (2017) §3: umbrella for soluble protein signals between immune cells."),
        ImmunologyEvent: ("en", "Immunology event (abstract)",
            "Murphy (2017) §3: umbrella for time-extended processes in the inflammatory cascade."),

        TissueInjury: ("en", "Tissue injury",
            "Murphy (2017) §3: initiating tissue damage that triggers the inflammatory cascade."),
        NeutrophilRecruitment: ("en", "Neutrophil recruitment",
            "Murphy (2017) §3: rapid extravasation of neutrophils into the injured tissue."),
        AcuteInflammationOnset: ("en", "Acute inflammation onset",
            "Murphy (2017) §3: onset of the acute inflammatory state."),
        MonocyteRecruitment: ("en", "Monocyte recruitment",
            "Murphy (2017) §3: monocytes enter the tissue and differentiate into macrophages."),
        M1Polarization: ("en", "M1 polarisation",
            "Murphy (2017) §3: monocyte-derived macrophages polarise to the classical pro-inflammatory phenotype."),
        ProInflammatoryResponse: ("en", "Pro-inflammatory response",
            "Murphy (2017) §3: M1 macrophages produce TNFα/IL-6 and orchestrate the inflammatory phase."),
        M1ToM2Transition: ("en", "M1→M2 transition",
            "Weinheimer-Haus et al. (2014): macrophages switch from M1 (pro-inflammatory) to M2 (pro-repair) phenotype."),
        AntiInflammatoryResponse: ("en", "Anti-inflammatory response",
            "Murphy (2017) §3: M2 macrophages produce IL-10/TGF-β and drive resolution."),
        TissueRemodeling: ("en", "Tissue remodeling",
            "Abbas et al. (2021): orchestrated collagen deposition and matrix remodelling during repair."),
        RepairCompletion: ("en", "Repair completion",
            "Abbas et al. (2021): completion of tissue repair — return to functional state."),
        ChronicStimulus: ("en", "Chronic stimulus",
            "Murphy (2017) §3: persistent or recurrent injury driving chronic inflammation."),
        FailedResolution: ("en", "Failed resolution",
            "Abbas et al. (2021): failure of the resolution program — inflammation persists."),
        FibrosisProgression: ("en", "Fibrosis progression",
            "Abbas et al. (2021): pathological replacement of tissue by fibrotic scar."),
        MechanicalStimulation: ("en", "Mechanical stimulation",
            "Weinheimer-Haus et al. (2014): 45 Hz whole-body vibration that triggers the M1→M2 transition."),
    },

    is_a: [
        // Immune cells (Murphy 2017 §2).
        (MacrophageM1, ImmuneCell),
        (MacrophageM2, ImmuneCell),
        (Neutrophil, ImmuneCell),
        (TCell, ImmuneCell),
        (Monocyte, ImmuneCell),
        (MastCell, ImmuneCell),
        // Stromal cells.
        (Fibroblast, StromalCell),
        // Inflammatory states.
        (AcuteInflammation, InflammatoryState),
        (ChronicInflammation, InflammatoryState),
        (Resolution, InflammatoryState),
        (Fibrosis, InflammatoryState),
        (TissueRepair, InflammatoryState),
        // Cytokine taxonomy (Murphy 2017 §3 — disjoint branches).
        (TNFAlpha, ProInflammatoryCytokine),
        (IL6, ProInflammatoryCytokine),
        (IL10, AntiInflammatoryCytokine),
        (TGFBeta, AntiInflammatoryCytokine),
        (ProInflammatoryCytokine, Cytokine),
        (AntiInflammatoryCytokine, Cytokine),
        // Events under the ImmunologyEvent umbrella.
        (TissueInjury, ImmunologyEvent),
        (NeutrophilRecruitment, ImmunologyEvent),
        (AcuteInflammationOnset, ImmunologyEvent),
        (MonocyteRecruitment, ImmunologyEvent),
        (M1Polarization, ImmunologyEvent),
        (ProInflammatoryResponse, ImmunologyEvent),
        (M1ToM2Transition, ImmunologyEvent),
        (AntiInflammatoryResponse, ImmunologyEvent),
        (TissueRemodeling, ImmunologyEvent),
        (RepairCompletion, ImmunologyEvent),
        (ChronicStimulus, ImmunologyEvent),
        (FailedResolution, ImmunologyEvent),
        (FibrosisProgression, ImmunologyEvent),
        (MechanicalStimulation, ImmunologyEvent),
    ],

    causes: [
        // Normal healing cascade (Murphy 2017 §3).
        (TissueInjury, NeutrophilRecruitment),
        (NeutrophilRecruitment, AcuteInflammationOnset),
        (AcuteInflammationOnset, MonocyteRecruitment),
        (MonocyteRecruitment, M1Polarization),
        (M1Polarization, ProInflammatoryResponse),
        (ProInflammatoryResponse, M1ToM2Transition),
        (M1ToM2Transition, AntiInflammatoryResponse),
        (AntiInflammatoryResponse, TissueRemodeling),
        (TissueRemodeling, RepairCompletion),
        // Pathological path (Abbas et al. 2021).
        (ChronicStimulus, FailedResolution),
        (FailedResolution, FibrosisProgression),
        // Vibration intervention (Weinheimer-Haus et al. 2014).
        (MechanicalStimulation, M1ToM2Transition),
    ],

    opposes: [
        // M1 / M2 macrophage polarisation (Murphy 2017 §3).
        (MacrophageM1, MacrophageM2),
        (MacrophageM2, MacrophageM1),
        // Acute inflammation onset vs resolution.
        (AcuteInflammation, Resolution),
        (Resolution, AcuteInflammation),
        // Pathological persistence vs healing.
        (ChronicInflammation, TissueRepair),
        (TissueRepair, ChronicInflammation),
        // Pro- vs anti-inflammatory cytokine classes.
        (ProInflammatoryCytokine, AntiInflammatoryCytokine),
        (AntiInflammatoryCytokine, ProInflammatoryCytokine),
        // Canonical pro/anti pair: TNFα vs IL-10.
        (TNFAlpha, IL10),
        (IL10, TNFAlpha),
    ],
}

// Backward-compatibility re-exports.
pub use ImmunologyConcept as ImmunologyEntity;
pub use ImmunologyRelationKind as ImmunologyCategoryRelationKind;

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Macrophage polarisation state (Murphy 2017 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolarizationValue {
    M1Classical,
    M2Alternative,
    Unpolarized,
    NotApplicable,
}

/// Time scale of inflammatory processes (Murphy 2017 §3 — acute = hours,
/// chronic = weeks).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeScaleValue {
    Hours,
    Days,
    Weeks,
}

/// Quality: is this entity pro-inflammatory? (Murphy 2017 §3.)
#[derive(Debug, Clone)]
pub struct IsProInflammatory;

impl Quality for IsProInflammatory {
    type Individual = ImmunologyConcept;
    type Value = bool;

    fn get(&self, individual: &ImmunologyConcept) -> Option<bool> {
        use ImmunologyConcept::*;
        match individual {
            MacrophageM1 | Neutrophil | MastCell => Some(true),
            MacrophageM2 | TCell | Monocyte | Fibroblast => Some(false),
            TNFAlpha | IL6 | ProInflammatoryCytokine => Some(true),
            IL10 | TGFBeta | AntiInflammatoryCytokine => Some(false),
            AcuteInflammation | ChronicInflammation => Some(true),
            Resolution | TissueRepair | Fibrosis => Some(false),
            _ => None,
        }
    }
}

/// Quality: is this entity pro-repair? (Abbas et al. 2021.)
#[derive(Debug, Clone)]
pub struct IsProRepair;

impl Quality for IsProRepair {
    type Individual = ImmunologyConcept;
    type Value = bool;

    fn get(&self, individual: &ImmunologyConcept) -> Option<bool> {
        use ImmunologyConcept::*;
        match individual {
            MacrophageM1 => Some(false),
            MacrophageM2 | Fibroblast => Some(true),
            IL10 | TGFBeta | AntiInflammatoryCytokine => Some(true),
            TNFAlpha | IL6 | ProInflammatoryCytokine => Some(false),
            TissueRepair | Resolution => Some(true),
            Fibrosis | AcuteInflammation | ChronicInflammation => Some(false),
            _ => None,
        }
    }
}

/// Quality: macrophage polarisation state.
#[derive(Debug, Clone)]
pub struct PolarizationState;

impl Quality for PolarizationState {
    type Individual = ImmunologyConcept;
    type Value = PolarizationValue;

    fn get(&self, individual: &ImmunologyConcept) -> Option<PolarizationValue> {
        use ImmunologyConcept::*;
        match individual {
            MacrophageM1 => Some(PolarizationValue::M1Classical),
            MacrophageM2 => Some(PolarizationValue::M2Alternative),
            Monocyte => Some(PolarizationValue::Unpolarized),
            Neutrophil | TCell | MastCell | Fibroblast => Some(PolarizationValue::NotApplicable),
            _ => None,
        }
    }
}

/// Quality: time scale of inflammatory states (Murphy 2017 §3).
#[derive(Debug, Clone)]
pub struct TimeScale;

impl Quality for TimeScale {
    type Individual = ImmunologyConcept;
    type Value = TimeScaleValue;

    fn get(&self, individual: &ImmunologyConcept) -> Option<TimeScaleValue> {
        use ImmunologyConcept::*;
        match individual {
            AcuteInflammation => Some(TimeScaleValue::Hours),
            Resolution | TissueRepair => Some(TimeScaleValue::Days),
            ChronicInflammation | Fibrosis => Some(TimeScaleValue::Weeks),
            _ => None,
        }
    }
}

/// Quality: is this event modulable by mechanical stimulation
/// (Weinheimer-Haus et al. 2014)?
#[derive(Debug, Clone)]
pub struct IsModulableByVibration;

impl Quality for IsModulableByVibration {
    type Individual = ImmunologyConcept;
    type Value = bool;

    fn get(&self, individual: &ImmunologyConcept) -> Option<bool> {
        use ImmunologyConcept::*;
        match individual {
            M1ToM2Transition | AntiInflammatoryResponse | TissueRemodeling | RepairCompletion => {
                Some(true)
            }
            TissueInjury
            | NeutrophilRecruitment
            | AcuteInflammationOnset
            | MonocyteRecruitment
            | M1Polarization
            | ProInflammatoryResponse
            | ChronicStimulus
            | FailedResolution
            | FibrosisProgression
            | MechanicalStimulation => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_a(child: ImmunologyConcept, parent: ImmunologyConcept) -> bool {
    ImmunologyCategory::morphisms().iter().any(|m| {
        m.kind() == ImmunologyRelationKind::Subsumption
            && m.source() == child
            && m.target() == parent
    })
}

fn causes(cause: ImmunologyConcept, effect: ImmunologyConcept) -> bool {
    ImmunologyCategory::morphisms().iter().any(|m| {
        m.kind() == ImmunologyRelationKind::Causation && m.source() == cause && m.target() == effect
    })
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Tissue injury transitively causes repair completion (Murphy 2017 §3 —
/// normal healing cascade).
pub struct InjuryCausesRepair;

impl Axiom for InjuryCausesRepair {
    fn verify(&self) -> Verdict {
        if causes(
            ImmunologyConcept::TissueInjury,
            ImmunologyConcept::RepairCompletion,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "InjuryCausesRepair",
        "tissue injury transitively causes repair completion via the normal inflammatory cascade",
        "Murphy (2017) Janeway's Immunobiology 10th ed. §3"
    );
}

pr4xis::register_axiom!(
    InjuryCausesRepair,
    "Murphy (2017) Janeway's Immunobiology 10th ed. §3"
);

/// Chronic stimulus causes fibrosis progression, not repair completion
/// (Abbas et al. 2021 — pathological path).
pub struct ChronicStimulusCausesFibrosis;

impl Axiom for ChronicStimulusCausesFibrosis {
    fn verify(&self) -> Verdict {
        let cs = ImmunologyConcept::ChronicStimulus;
        if causes(cs, ImmunologyConcept::FibrosisProgression)
            && !causes(cs, ImmunologyConcept::RepairCompletion)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ChronicStimulusCausesFibrosis",
        "chronic stimulus transitively causes fibrosis progression; does not reach repair completion",
        "Abbas, Lichtman & Pillai (2021) Cellular and Molecular Immunology 10th ed."
    );
}

pr4xis::register_axiom!(
    ChronicStimulusCausesFibrosis,
    "Abbas, Lichtman & Pillai (2021) Cellular and Molecular Immunology 10th ed."
);

/// Mechanical stimulation causes M1→M2 transition (Weinheimer-Haus et al.
/// 2014: 45 Hz WBV shifts macrophage polarisation in diabetic mouse wounds).
pub struct VibrationCausesM1ToM2;

impl Axiom for VibrationCausesM1ToM2 {
    fn verify(&self) -> Verdict {
        if causes(
            ImmunologyConcept::MechanicalStimulation,
            ImmunologyConcept::M1ToM2Transition,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "VibrationCausesM1ToM2",
        "mechanical stimulation (45 Hz whole-body vibration) causes M1→M2 macrophage polarisation shift",
        "Weinheimer-Haus et al. (2014) PLoS ONE 9(3):e91355"
    );
}

pr4xis::register_axiom!(
    VibrationCausesM1ToM2,
    "Weinheimer-Haus et al. (2014) PLoS ONE 9(3):e91355"
);

/// M1 is pro-inflammatory (not pro-repair); M2 is pro-repair (not
/// pro-inflammatory). Mutually exclusive phenotypes (Murphy 2017 §3).
pub struct M1M2MutuallyExclusive;

impl Axiom for M1M2MutuallyExclusive {
    fn verify(&self) -> Verdict {
        use ImmunologyConcept::*;
        let pi = IsProInflammatory;
        let pr = IsProRepair;
        if pi.get(&MacrophageM1) == Some(true)
            && pr.get(&MacrophageM1) == Some(false)
            && pi.get(&MacrophageM2) == Some(false)
            && pr.get(&MacrophageM2) == Some(true)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "M1M2MutuallyExclusive",
        "M1 is pro-inflammatory (not pro-repair), M2 is pro-repair (not pro-inflammatory)",
        "Murphy (2017) Janeway's Immunobiology 10th ed. §3"
    );
}

pr4xis::register_axiom!(
    M1M2MutuallyExclusive,
    "Murphy (2017) Janeway's Immunobiology 10th ed. §3"
);

/// Pro-inflammatory and anti-inflammatory cytokine branches are disjoint
/// (Murphy 2017 §3 — canonical cytokine classification).
pub struct CytokineBranchesDisjoint;

impl Axiom for CytokineBranchesDisjoint {
    fn verify(&self) -> Verdict {
        use ImmunologyConcept::*;
        let ok = is_a(TNFAlpha, ProInflammatoryCytokine)
            && is_a(IL6, ProInflammatoryCytokine)
            && !is_a(TNFAlpha, AntiInflammatoryCytokine)
            && !is_a(IL6, AntiInflammatoryCytokine)
            && is_a(IL10, AntiInflammatoryCytokine)
            && is_a(TGFBeta, AntiInflammatoryCytokine)
            && !is_a(IL10, ProInflammatoryCytokine)
            && !is_a(TGFBeta, ProInflammatoryCytokine);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CytokineBranchesDisjoint",
        "pro-inflammatory and anti-inflammatory cytokines are disjoint taxonomy branches",
        "Murphy (2017) Janeway's Immunobiology 10th ed. §3"
    );
}

pr4xis::register_axiom!(
    CytokineBranchesDisjoint,
    "Murphy (2017) Janeway's Immunobiology 10th ed. §3"
);

/// M1→M2 transition transitively leads to repair completion (Murphy 2017
/// §3 — completion of the normal healing cascade).
pub struct M1ToM2LeadsToRepair;

impl Axiom for M1ToM2LeadsToRepair {
    fn verify(&self) -> Verdict {
        if causes(
            ImmunologyConcept::M1ToM2Transition,
            ImmunologyConcept::RepairCompletion,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "M1ToM2LeadsToRepair",
        "M1→M2 transition transitively leads to repair completion",
        "Murphy (2017) Janeway's Immunobiology 10th ed. §3"
    );
}

pr4xis::register_axiom!(
    M1ToM2LeadsToRepair,
    "Murphy (2017) Janeway's Immunobiology 10th ed. §3"
);

/// All concrete immune cells are subsumed by ImmuneCell; Fibroblast is
/// StromalCell (Murphy 2017 §2; Abbas et al. 2021).
pub struct AllImmuneCellsClassified;

impl Axiom for AllImmuneCellsClassified {
    fn verify(&self) -> Verdict {
        use ImmunologyConcept::*;
        let immune = [
            MacrophageM1,
            MacrophageM2,
            Neutrophil,
            TCell,
            Monocyte,
            MastCell,
        ];
        let ok = immune.iter().all(|c| is_a(*c, ImmuneCell))
            && is_a(Fibroblast, StromalCell)
            && !is_a(Fibroblast, ImmuneCell);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AllImmuneCellsClassified",
        "all concrete immune cells subsume under ImmuneCell; Fibroblast subsumes under StromalCell (not ImmuneCell)",
        "Murphy (2017) Janeway's Immunobiology 10th ed. §2; Abbas et al. (2021) §1"
    );
}

pr4xis::register_axiom!(
    AllImmuneCellsClassified,
    "Murphy (2017) §2; Abbas et al. (2021) §1"
);

/// Acute inflammation operates on Hours time scale, chronic on Weeks
/// (Murphy 2017 §3 — canonical clinical time scales).
pub struct InflammationTimeScales;

impl Axiom for InflammationTimeScales {
    fn verify(&self) -> Verdict {
        let ts = TimeScale;
        if ts.get(&ImmunologyConcept::AcuteInflammation) == Some(TimeScaleValue::Hours)
            && ts.get(&ImmunologyConcept::ChronicInflammation) == Some(TimeScaleValue::Weeks)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "InflammationTimeScales",
        "acute inflammation is Hours, chronic inflammation is Weeks",
        "Murphy (2017) Janeway's Immunobiology 10th ed. §3"
    );
}

pr4xis::register_axiom!(
    InflammationTimeScales,
    "Murphy (2017) Janeway's Immunobiology 10th ed. §3"
);

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

impl Ontology for ImmunologyOntology {
    type Cat = ImmunologyCategory;
    type Qual = IsProInflammatory;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(InjuryCausesRepair));
        axioms.push(Box::new(ChronicStimulusCausesFibrosis));
        axioms.push(Box::new(VibrationCausesM1ToM2));
        axioms.push(Box::new(M1M2MutuallyExclusive));
        axioms.push(Box::new(CytokineBranchesDisjoint));
        axioms.push(Box::new(M1ToM2LeadsToRepair));
        axioms.push(Box::new(AllImmuneCellsClassified));
        axioms.push(Box::new(InflammationTimeScales));
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
        assert_category_laws::<ImmunologyCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ImmunologyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    // -- Domain axiom tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn injury_causes_repair() {
        assert!(InjuryCausesRepair.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn chronic_stimulus_causes_fibrosis() {
        assert!(ChronicStimulusCausesFibrosis.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn vibration_causes_m1_to_m2() {
        assert!(VibrationCausesM1ToM2.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn m1_m2_mutually_exclusive() {
        assert!(M1M2MutuallyExclusive.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cytokine_branches_disjoint() {
        assert!(CytokineBranchesDisjoint.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn m1_to_m2_leads_to_repair() {
        assert!(M1ToM2LeadsToRepair.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn all_immune_cells_classified() {
        assert!(AllImmuneCellsClassified.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn inflammation_time_scales() {
        assert!(InflammationTimeScales.verify().is_ok());
    }

    // -- Subsumption / kind tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cells_classified_correctly() {
        use ImmunologyConcept::*;
        for c in [
            MacrophageM1,
            MacrophageM2,
            Neutrophil,
            TCell,
            Monocyte,
            MastCell,
        ] {
            assert!(is_a(c, ImmuneCell), "{:?} should be an ImmuneCell", c);
        }
        assert!(is_a(Fibroblast, StromalCell));
        assert!(!is_a(Fibroblast, ImmuneCell));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cytokines_are_cytokines() {
        use ImmunologyConcept::*;
        for c in [TNFAlpha, IL6, IL10, TGFBeta] {
            assert!(is_a(c, Cytokine));
        }
    }

    // -- Opposition tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn m1_opposes_m2() {
        let opps: Vec<_> = ImmunologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ImmunologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            ImmunologyConcept::MacrophageM1,
            ImmunologyConcept::MacrophageM2
        )));
        assert!(opps.contains(&(
            ImmunologyConcept::MacrophageM2,
            ImmunologyConcept::MacrophageM1
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn tnf_alpha_opposes_il10() {
        let opps: Vec<_> = ImmunologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ImmunologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(ImmunologyConcept::TNFAlpha, ImmunologyConcept::IL10)));
    }

    // -- Causal chain tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn full_healing_cascade() {
        use ImmunologyConcept::*;
        for e in [
            NeutrophilRecruitment,
            AcuteInflammationOnset,
            MonocyteRecruitment,
            M1Polarization,
            ProInflammatoryResponse,
            M1ToM2Transition,
            AntiInflammatoryResponse,
            TissueRemodeling,
            RepairCompletion,
        ] {
            assert!(causes(TissueInjury, e), "TissueInjury should cause {:?}", e);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fibrosis_path_does_not_reach_repair() {
        use ImmunologyConcept::*;
        assert!(causes(ChronicStimulus, FibrosisProgression));
        assert!(!causes(ChronicStimulus, RepairCompletion));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mechanical_stimulation_reaches_repair() {
        use ImmunologyConcept::*;
        assert!(causes(MechanicalStimulation, M1ToM2Transition));
        assert!(causes(MechanicalStimulation, RepairCompletion));
    }

    // -- Polarisation tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn polarization_states() {
        use ImmunologyConcept::*;
        let ps = PolarizationState;
        assert_eq!(ps.get(&MacrophageM1), Some(PolarizationValue::M1Classical));
        assert_eq!(
            ps.get(&MacrophageM2),
            Some(PolarizationValue::M2Alternative)
        );
        assert_eq!(ps.get(&Monocyte), Some(PolarizationValue::Unpolarized));
        assert_eq!(ps.get(&Neutrophil), Some(PolarizationValue::NotApplicable));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn vibration_modulable_events() {
        use ImmunologyConcept::*;
        let vm = IsModulableByVibration;
        assert_eq!(vm.get(&M1ToM2Transition), Some(true));
        assert_eq!(vm.get(&TissueInjury), Some(false));
        assert_eq!(vm.get(&ChronicStimulus), Some(false));
    }

    // -- Literature axioms --

    /// Weinheimer-Haus et al. (2014): 45 Hz WBV shifts macrophage
    /// polarisation toward M2 in diabetic mouse wounds.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn literature_weinheimer_haus_2014_wbv_m1_to_m2() {
        use ImmunologyConcept::*;
        assert!(causes(MechanicalStimulation, M1ToM2Transition));
        assert!(causes(MechanicalStimulation, RepairCompletion));
    }

    /// Yu et al. (2019) PMID:31247969: WBV induces omental macrophage
    /// polarisation shift — vibration modulates the immune response.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn literature_yu_2019_wbv_immune_modulation() {
        use ImmunologyConcept::*;
        assert!(causes(MechanicalStimulation, AntiInflammatoryResponse));
        assert_eq!(IsModulableByVibration.get(&M1ToM2Transition), Some(true));
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = ImmunologyConcept> {
        proptest::sample::select(ImmunologyConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in ImmunologyCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ImmunologyOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        /// Murphy (2017) §3: any ImmuneCell that has a polarisation cannot
        /// be both pro-inflammatory and pro-repair at the same time.
        #[test]
        fn prop_immune_cell_m1_m2_mutual_exclusivity(c in arb_concept()) {
            if is_a(c, ImmunologyConcept::ImmuneCell) && c != ImmunologyConcept::ImmuneCell {
                let pi = IsProInflammatory.get(&c);
                let pr = IsProRepair.get(&c);
                if let (Some(true), Some(true)) = (pi, pr) {
                    prop_assert!(false, "{:?} cannot be both pro-inflammatory and pro-repair", c);
                }
            }
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = ImmunologyConcept::variants();
            for m in ImmunologyCategory::morphisms() {
                if m.kind() == ImmunologyRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = ImmunologyCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == ImmunologyRelationKind::Opposition)
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
    pr4xis::register_praxis_value!(prop_immune_cell_m1_m2_mutual_exclusivity, Verifiable);
    pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
}

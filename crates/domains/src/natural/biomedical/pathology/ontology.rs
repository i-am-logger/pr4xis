//! Disease pathology — disease states, staging, classifications, and the
//! causal progression from tissue insult to neoplasia or stricture.
//!
//! Models the general pathology ontology (not organ-specific): normal tissue
//! through acute and chronic injury, metaplasia, dysplasia (low/high grade),
//! neoplasia, fibrosis, and stricture; benign/premalignant/malignant
//! classifications; inflammation, cellular adaptation, atypical growth, and
//! invasion processes; the canonical tissue-insult → acute-response →
//! chronic-adaptation → metaplastic → dysplastic → neoplastic chain plus the
//! fibrotic branch and dysplasia staging. Per
//! `feedback_one_ontology_per_module` the original split between
//! `PathologyEntity` and `PathologyCausalEvent` has been merged: events are
//! first-class concepts subsumed by the `PathologyEvent` umbrella.
//!
//! # Literature
//!
//! - **Kumar, Abbas, Aster (2020)** *Robbins & Cotran Pathologic Basis of
//!   Disease*, 10th ed., Elsevier — canonical reference for cellular injury
//!   and adaptation (Ch. 2), inflammation (Ch. 3), tissue repair and
//!   fibrosis (Ch. 4), neoplasia (Ch. 7), and the metaplasia / dysplasia /
//!   carcinoma-in-situ progression.
//! - **Chernet & Levin (2013)** "Endogenous Voltage Potentials and the
//!   Microenvironment: Bioelectric Signals that Reveal, Induce and Normalize
//!   Cancer", *J. Clin. Exp. Oncol.* S1:002 — depolarised Vmem correlates
//!   with neoplastic transformation; repolarisation suppresses tumor.
//! - **Levin (2014)** "Molecular bioelectricity: how endogenous voltage
//!   potentials control cell behavior and instruct pattern regulation in
//!   vivo", *Mol. Biol. Cell* 25(24):3835–3850 — normal polarised vs cancer
//!   depolarised Vmem.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Pathology",
    source: "Kumar, Abbas, Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed.; Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002; Levin (2014) Mol. Biol. Cell 25(24):3835-3850",

    concepts: [
        // === Disease states (Robbins & Cotran 2020 Ch. 2, 7) ===
        Normal,
        AcuteInjury,
        ChronicInjury,
        Metaplasia,
        Dysplasia,
        Neoplasia,
        Fibrosis,
        Stricture,

        // === Staging (Robbins & Cotran 2020 Ch. 7) ===
        LowGrade,
        HighGrade,

        // === Classifications (Robbins & Cotran 2020 Ch. 7) ===
        Benign,
        Premalignant,
        Malignant,

        // === Pathological processes (Robbins & Cotran 2020 Ch. 2-4, 7) ===
        Inflammation,
        CellularAdaptation,
        AtypicalGrowth,
        Invasion,

        // === Abstract umbrellas ===
        DiseaseState,
        Stage,
        Classification,
        PathologicalProcess,
        PathologyEvent,

        // === Causal events (Robbins & Cotran 2020 Ch. 2, 4, 7) ===
        TissueInsult,
        AcuteResponse,
        ChronicAdaptation,
        MetaplasticTransformation,
        DysplasticProgression,
        NeoplasticTransformation,
        FibroticRemodeling,
        StrictureFormation,
        LowGradeProgression,
        HighGradeProgression,
    ],

    labels: {
        Normal: ("en", "Normal",
            "Robbins & Cotran (2020) Ch. 2: healthy tissue with intact morphology, function, and (per Levin 2014) polarised membrane potential."),
        AcuteInjury: ("en", "Acute injury",
            "Robbins & Cotran (2020) Ch. 2: reversible cell injury of short duration following an exogenous or endogenous insult."),
        ChronicInjury: ("en", "Chronic injury",
            "Robbins & Cotran (2020) Ch. 2: sustained tissue damage from an ongoing insult."),
        Metaplasia: ("en", "Metaplasia",
            "Robbins & Cotran (2020) Ch. 2: reversible replacement of one differentiated cell type by another, generally in response to chronic stress."),
        Dysplasia: ("en", "Dysplasia",
            "Robbins & Cotran (2020) Ch. 7: disordered cell growth with loss of normal architectural and cytologic features; premalignant."),
        Neoplasia: ("en", "Neoplasia",
            "Robbins & Cotran (2020) Ch. 7: irreversible autonomous proliferation of cells (benign or malignant tumor)."),
        Fibrosis: ("en", "Fibrosis",
            "Robbins & Cotran (2020) Ch. 4: excessive extracellular-matrix deposition replacing functional tissue during chronic repair."),
        Stricture: ("en", "Stricture",
            "Robbins & Cotran (2020) Ch. 4: luminal narrowing of a hollow organ resulting from fibrotic remodelling."),

        LowGrade: ("en", "Low-grade dysplasia",
            "Robbins & Cotran (2020) Ch. 7: mild architectural distortion and nuclear atypia; lower progression risk."),
        HighGrade: ("en", "High-grade dysplasia",
            "Robbins & Cotran (2020) Ch. 7: severe architectural and cytologic atypia approaching carcinoma in situ."),

        Benign: ("en", "Benign",
            "Robbins & Cotran (2020) Ch. 7: tumor that lacks the capacity to invade or metastasise."),
        Premalignant: ("en", "Premalignant",
            "Robbins & Cotran (2020) Ch. 7: lesion with elevated risk of progression to invasive malignancy."),
        Malignant: ("en", "Malignant",
            "Robbins & Cotran (2020) Ch. 7: tumor with the capacity to invade adjacent tissue and metastasise."),

        Inflammation: ("en", "Inflammation",
            "Robbins & Cotran (2020) Ch. 3: stereotyped vascular and cellular response to injury, infection, or immune stimulus."),
        CellularAdaptation: ("en", "Cellular adaptation",
            "Robbins & Cotran (2020) Ch. 2: reversible change in cell size, number, phenotype, metabolic activity, or function in response to stress."),
        AtypicalGrowth: ("en", "Atypical growth",
            "Robbins & Cotran (2020) Ch. 7: disordered proliferation with cellular atypia preceding overt malignancy."),
        Invasion: ("en", "Invasion",
            "Robbins & Cotran (2020) Ch. 7: tumor cells breach the basement membrane — the defining hallmark of malignancy."),

        DiseaseState: ("en", "Disease state",
            "Robbins & Cotran (2020): umbrella for tissue states distinguished by structural and functional deviation from normal."),
        Stage: ("en", "Stage",
            "Robbins & Cotran (2020) Ch. 7: graded severity of dysplastic or neoplastic change."),
        Classification: ("en", "Classification",
            "Robbins & Cotran (2020) Ch. 7: categorical descriptor of a lesion's biological behaviour (benign / premalignant / malignant)."),
        PathologicalProcess: ("en", "Pathological process",
            "Robbins & Cotran (2020): umbrella for stereotyped mechanisms (inflammation, adaptation, atypical growth, invasion)."),
        PathologyEvent: ("en", "Pathology event",
            "Robbins & Cotran (2020): umbrella for time-extended pathology processes (insult, response, transformation, progression, remodelling)."),

        TissueInsult: ("en", "Tissue insult",
            "Robbins & Cotran (2020) Ch. 2: exogenous or endogenous stimulus that initiates cellular injury (chemical, physical, infectious, immunologic, genetic, nutritional)."),
        AcuteResponse: ("en", "Acute response",
            "Robbins & Cotran (2020) Ch. 3: acute inflammatory and repair response — rapid, neutrophil-predominant."),
        ChronicAdaptation: ("en", "Chronic adaptation",
            "Robbins & Cotran (2020) Ch. 2: long-term tissue remodelling in response to sustained insult (hypertrophy, hyperplasia, metaplasia)."),
        MetaplasticTransformation: ("en", "Metaplastic transformation",
            "Robbins & Cotran (2020) Ch. 2: phenotypic switch from one differentiated cell type to another (e.g. squamous to columnar)."),
        DysplasticProgression: ("en", "Dysplastic progression",
            "Robbins & Cotran (2020) Ch. 7: acquisition of dysplastic architectural and cytologic features."),
        NeoplasticTransformation: ("en", "Neoplastic transformation",
            "Robbins & Cotran (2020) Ch. 7: transition from dysplasia to autonomous, invasive neoplastic growth."),
        FibroticRemodeling: ("en", "Fibrotic remodeling",
            "Robbins & Cotran (2020) Ch. 4: excessive collagen deposition and scar formation replacing functional tissue."),
        StrictureFormation: ("en", "Stricture formation",
            "Robbins & Cotran (2020) Ch. 4: luminal narrowing produced by fibrotic remodelling."),
        LowGradeProgression: ("en", "Low-grade progression",
            "Robbins & Cotran (2020) Ch. 7: acquisition of mild dysplastic features."),
        HighGradeProgression: ("en", "High-grade progression",
            "Robbins & Cotran (2020) Ch. 7: acquisition of severe dysplastic features approaching carcinoma in situ."),
    },

    is_a: [
        // Disease states
        (Normal, DiseaseState),
        (AcuteInjury, DiseaseState),
        (ChronicInjury, DiseaseState),
        (Metaplasia, DiseaseState),
        (Dysplasia, DiseaseState),
        (Neoplasia, DiseaseState),
        (Fibrosis, DiseaseState),
        (Stricture, DiseaseState),

        // Staging
        (LowGrade, Stage),
        (HighGrade, Stage),

        // Classifications
        (Benign, Classification),
        (Premalignant, Classification),
        (Malignant, Classification),

        // Processes
        (Inflammation, PathologicalProcess),
        (CellularAdaptation, PathologicalProcess),
        (AtypicalGrowth, PathologicalProcess),
        (Invasion, PathologicalProcess),

        // Events under PathologyEvent
        (TissueInsult, PathologyEvent),
        (AcuteResponse, PathologyEvent),
        (ChronicAdaptation, PathologyEvent),
        (MetaplasticTransformation, PathologyEvent),
        (DysplasticProgression, PathologyEvent),
        (NeoplasticTransformation, PathologyEvent),
        (FibroticRemodeling, PathologyEvent),
        (StrictureFormation, PathologyEvent),
        (LowGradeProgression, PathologyEvent),
        (HighGradeProgression, PathologyEvent),
    ],

    causes: [
        // Robbins & Cotran (2020) Ch. 2, 3, 7 — canonical chronic-injury chain:
        // insult -> acute -> chronic -> metaplasia -> dysplasia -> neoplasia.
        (TissueInsult, AcuteResponse),
        (AcuteResponse, ChronicAdaptation),
        (ChronicAdaptation, MetaplasticTransformation),
        (MetaplasticTransformation, DysplasticProgression),
        (DysplasticProgression, NeoplasticTransformation),
        // Robbins & Cotran (2020) Ch. 4 — fibrotic branch:
        // chronic -> fibrotic remodelling -> stricture formation.
        (ChronicAdaptation, FibroticRemodeling),
        (FibroticRemodeling, StrictureFormation),
        // Robbins & Cotran (2020) Ch. 7 — dysplasia staging:
        // dysplastic -> low-grade -> high-grade -> neoplastic.
        (DysplasticProgression, LowGradeProgression),
        (LowGradeProgression, HighGradeProgression),
        (HighGradeProgression, NeoplasticTransformation),
    ],

    opposes: [
        // Normal vs Neoplasia: health vs disease endpoint (Robbins & Cotran
        // 2020 Ch. 7).
        (Normal, Neoplasia),
        (Neoplasia, Normal),
        // Benign vs Malignant: non-invasive vs invasive classification
        // (Robbins & Cotran 2020 Ch. 7).
        (Benign, Malignant),
        (Malignant, Benign),
        // LowGrade vs HighGrade: mild vs severe dysplasia (Robbins & Cotran
        // 2020 Ch. 7).
        (LowGrade, HighGrade),
        (HighGrade, LowGrade),
        // Inflammation vs CellularAdaptation: acute vs chronic response
        // (Robbins & Cotran 2020 Ch. 2-3).
        (Inflammation, CellularAdaptation),
        (CellularAdaptation, Inflammation),
    ],
}

// Backward-compatibility re-exports for partner functors / sibling crates
// that reference the legacy `*Entity` / `*CategoryRelationKind` names.
pub use PathologyConcept as PathologyEntity;
pub use PathologyRelationKind as PathologyCategoryRelationKind;

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: is this disease state reversible?
///
/// Robbins & Cotran (2020) Ch. 2 — acute injury, chronic injury, and
/// metaplasia are reversible; dysplasia is partially reversible at low
/// grade; neoplasia, fibrosis, and stricture are not.
#[derive(Debug, Clone)]
pub struct IsReversible;

impl Quality for IsReversible {
    type Individual = PathologyConcept;
    type Value = bool;

    fn get(&self, individual: &PathologyConcept) -> Option<bool> {
        use PathologyConcept::*;
        match individual {
            Normal => Some(true),
            AcuteInjury => Some(true),
            ChronicInjury => Some(true),
            Metaplasia => Some(true),
            Dysplasia => Some(true),  // low-grade can regress
            Neoplasia => Some(false), // irreversible malignant transformation
            Fibrosis => Some(false),  // scarring is largely permanent
            Stricture => Some(false), // structural narrowing
            _ => None,
        }
    }
}

/// Malignant-potential classification (Robbins & Cotran 2020 Ch. 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MalignantPotentialLevel {
    /// No malignant potential (Robbins & Cotran 2020 Ch. 7).
    None,
    /// Low malignant potential (Robbins & Cotran 2020 Ch. 7).
    Low,
    /// High malignant potential (Robbins & Cotran 2020 Ch. 7).
    High,
    /// Already malignant (Robbins & Cotran 2020 Ch. 7).
    IsMalignant,
}

/// Quality: malignant potential of a disease state.
#[derive(Debug, Clone)]
pub struct MalignantPotential;

impl Quality for MalignantPotential {
    type Individual = PathologyConcept;
    type Value = MalignantPotentialLevel;

    fn get(&self, individual: &PathologyConcept) -> Option<MalignantPotentialLevel> {
        use MalignantPotentialLevel::*;
        use PathologyConcept as P;
        match individual {
            P::Normal => Some(None),
            P::AcuteInjury => Some(None),
            P::ChronicInjury => Some(Low),
            P::Metaplasia => Some(Low),
            P::Dysplasia => Some(High),
            P::Neoplasia => Some(IsMalignant),
            P::Fibrosis => Some(None),
            P::Stricture => Some(None),
            _ => Option::None,
        }
    }
}

/// Quality: does this disease state require clinical intervention?
///
/// Robbins & Cotran (2020) Ch. 2, 7 — chronic injury onwards generally
/// requires active management.
#[derive(Debug, Clone)]
pub struct RequiresIntervention;

impl Quality for RequiresIntervention {
    type Individual = PathologyConcept;
    type Value = bool;

    fn get(&self, individual: &PathologyConcept) -> Option<bool> {
        use PathologyConcept::*;
        match individual {
            Normal => Some(false),
            AcuteInjury => Some(false),
            ChronicInjury => Some(true),
            Metaplasia => Some(true),
            Dysplasia => Some(true),
            Neoplasia => Some(true),
            Fibrosis => Some(true),
            Stricture => Some(true),
            _ => None,
        }
    }
}

/// Quality: bioelectric (Vmem) correlate in millivolts.
///
/// Levin (2014); Chernet & Levin (2013) — normal tissue is polarised
/// (~ −50 mV); dysplastic and neoplastic tissue is depolarised
/// (~ −15 to −10 mV).
#[derive(Debug, Clone)]
pub struct BioelectricCorrelate;

impl Quality for BioelectricCorrelate {
    type Individual = PathologyConcept;
    type Value = f64;

    fn get(&self, individual: &PathologyConcept) -> Option<f64> {
        use PathologyConcept::*;
        match individual {
            Normal => Some(-50.0),
            AcuteInjury => Some(-30.0),
            ChronicInjury => Some(-25.0),
            Metaplasia => Some(-25.0),
            Dysplasia => Some(-15.0),
            Neoplasia => Some(-10.0),
            Fibrosis => Some(-35.0),
            Stricture => Some(-35.0),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Ontology + domain axioms
// ---------------------------------------------------------------------------

impl Ontology for PathologyOntology {
    type Cat = PathologyCategory;
    type Qual = IsReversible;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(TissueInsultCausesNeoplasia));
        axioms.push(Box::new(TissueInsultCausesStricture));
        axioms.push(Box::new(DysplasiaIsPremalignant));
        axioms.push(Box::new(NormalHasNoMalignantPotential));
        axioms.push(Box::new(NeoplasiaIsMalignant));
        axioms.push(Box::new(MetaplasiaIsReversible));
        axioms.push(Box::new(AcuteReversibleNeoplasiaIrreversible));
        axioms.push(Box::new(NormalIsPolarized));
        axioms
    }
}

/// Helper: does a `Causation` edge exist from `cause` to `effect`?
fn causes(cause: PathologyConcept, effect: PathologyConcept) -> bool {
    PathologyCategory::morphisms().iter().any(|m| {
        m.kind() == PathologyRelationKind::Causation && m.source() == cause && m.target() == effect
    })
}

/// Axiom: TissueInsult transitively causes NeoplasticTransformation.
///
/// Robbins & Cotran (2020) Ch. 2, 7 — the chronic-injury chain leads from
/// tissue insult through acute / chronic adaptation / metaplasia /
/// dysplasia to overt neoplastic transformation.
pub struct TissueInsultCausesNeoplasia;

impl Axiom for TissueInsultCausesNeoplasia {
    fn verify(&self) -> Verdict {
        let steps = [
            (
                PathologyConcept::TissueInsult,
                PathologyConcept::AcuteResponse,
            ),
            (
                PathologyConcept::AcuteResponse,
                PathologyConcept::ChronicAdaptation,
            ),
            (
                PathologyConcept::ChronicAdaptation,
                PathologyConcept::MetaplasticTransformation,
            ),
            (
                PathologyConcept::MetaplasticTransformation,
                PathologyConcept::DysplasticProgression,
            ),
            (
                PathologyConcept::DysplasticProgression,
                PathologyConcept::NeoplasticTransformation,
            ),
        ];
        if steps.iter().all(|(c, e)| causes(*c, *e)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TissueInsultCausesNeoplasia",
        "Tissue insult transitively causes neoplastic transformation via the chronic-injury chain",
        "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 2, 7"
    );
}

pr4xis::register_axiom!(
    TissueInsultCausesNeoplasia,
    "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 2, 7"
);

/// Axiom: TissueInsult transitively causes StrictureFormation.
///
/// Robbins & Cotran (2020) Ch. 4 — the fibrotic branch:
/// insult → acute → chronic → fibrotic remodelling → stricture.
pub struct TissueInsultCausesStricture;

impl Axiom for TissueInsultCausesStricture {
    fn verify(&self) -> Verdict {
        let steps = [
            (
                PathologyConcept::TissueInsult,
                PathologyConcept::AcuteResponse,
            ),
            (
                PathologyConcept::AcuteResponse,
                PathologyConcept::ChronicAdaptation,
            ),
            (
                PathologyConcept::ChronicAdaptation,
                PathologyConcept::FibroticRemodeling,
            ),
            (
                PathologyConcept::FibroticRemodeling,
                PathologyConcept::StrictureFormation,
            ),
        ];
        if steps.iter().all(|(c, e)| causes(*c, *e)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TissueInsultCausesStricture",
        "Tissue insult transitively causes stricture formation via the fibrotic branch",
        "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 4"
    );
}

pr4xis::register_axiom!(
    TissueInsultCausesStricture,
    "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 4"
);

/// Axiom: Dysplasia has high malignant potential (premalignant).
///
/// Robbins & Cotran (2020) Ch. 7 — dysplasia is the canonical premalignant
/// lesion.
pub struct DysplasiaIsPremalignant;

impl Axiom for DysplasiaIsPremalignant {
    fn verify(&self) -> Verdict {
        if MalignantPotential.get(&PathologyConcept::Dysplasia)
            == Some(MalignantPotentialLevel::High)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DysplasiaIsPremalignant",
        "Dysplasia has high malignant potential (premalignant lesion)",
        "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 7"
    );
}

pr4xis::register_axiom!(
    DysplasiaIsPremalignant,
    "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 7"
);

/// Axiom: Normal tissue has no malignant potential.
pub struct NormalHasNoMalignantPotential;

impl Axiom for NormalHasNoMalignantPotential {
    fn verify(&self) -> Verdict {
        if MalignantPotential.get(&PathologyConcept::Normal) == Some(MalignantPotentialLevel::None)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NormalHasNoMalignantPotential",
        "Normal healthy tissue carries no malignant potential",
        "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 7"
    );
}

pr4xis::register_axiom!(
    NormalHasNoMalignantPotential,
    "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 7"
);

/// Axiom: Neoplasia is malignant.
pub struct NeoplasiaIsMalignant;

impl Axiom for NeoplasiaIsMalignant {
    fn verify(&self) -> Verdict {
        if MalignantPotential.get(&PathologyConcept::Neoplasia)
            == Some(MalignantPotentialLevel::IsMalignant)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NeoplasiaIsMalignant",
        "Neoplasia carries the malignant designation (invasive autonomous proliferation)",
        "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 7"
    );
}

pr4xis::register_axiom!(
    NeoplasiaIsMalignant,
    "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 7"
);

/// Axiom: Metaplasia is reversible.
///
/// Robbins & Cotran (2020) Ch. 2 — metaplasia is by definition a
/// reversible change in differentiated phenotype.
pub struct MetaplasiaIsReversible;

impl Axiom for MetaplasiaIsReversible {
    fn verify(&self) -> Verdict {
        if IsReversible.get(&PathologyConcept::Metaplasia) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MetaplasiaIsReversible",
        "Metaplasia is a reversible adaptive change in differentiated cell type",
        "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 2"
    );
}

pr4xis::register_axiom!(
    MetaplasiaIsReversible,
    "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 2"
);

/// Axiom: Acute injury is reversible; neoplasia is not.
pub struct AcuteReversibleNeoplasiaIrreversible;

impl Axiom for AcuteReversibleNeoplasiaIrreversible {
    fn verify(&self) -> Verdict {
        let acute = IsReversible.get(&PathologyConcept::AcuteInjury) == Some(true);
        let neo = IsReversible.get(&PathologyConcept::Neoplasia) == Some(false);
        if acute && neo {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AcuteReversibleNeoplasiaIrreversible",
        "Acute injury is reversible whereas neoplastic transformation is irreversible",
        "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 2, 7"
    );
}

pr4xis::register_axiom!(
    AcuteReversibleNeoplasiaIrreversible,
    "Kumar, Abbas & Aster (2020) Robbins & Cotran Pathologic Basis of Disease 10th ed. Ch. 2, 7"
);

/// Axiom: Normal tissue carries a polarised membrane potential.
///
/// Levin (2014); Chernet & Levin (2013) — normal tissue is polarised at
/// roughly −50 mV; depolarisation correlates with neoplastic transformation.
pub struct NormalIsPolarized;

impl Axiom for NormalIsPolarized {
    fn verify(&self) -> Verdict {
        match BioelectricCorrelate.get(&PathologyConcept::Normal) {
            Some(v) if v < -40.0 => Ok(Box::new(SimpleProof::new(self.meta()))),
            _ => Err(Box::new(SimpleCounterexample::new(self.meta()))),
        }
    }

    pr4xis::axiom_meta!(
        "NormalIsPolarized",
        "Normal healthy tissue carries a polarised Vmem (< -40 mV)",
        "Levin (2014) Mol. Biol. Cell 25(24):3835-3850; Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
    );
}

pr4xis::register_axiom!(
    NormalIsPolarized,
    "Levin (2014) Mol. Biol. Cell 25(24):3835-3850; Chernet & Levin (2013) J. Clin. Exp. Oncol. S1:002"
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
        assert_category_laws::<PathologyCategory>();
    }

    #[test]
    fn ontology_validates() {
        PathologyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn concept_count() {
        // 8 disease states + 2 stages + 3 classifications + 4 processes
        // + 5 abstract umbrellas (DiseaseState, Stage, Classification,
        //   PathologicalProcess, PathologyEvent)
        // + 10 events = 32.
        assert_eq!(PathologyConcept::variants().len(), 32);
    }

    // -- Domain axiom tests --

    #[test]
    fn tissue_insult_causes_neoplasia_axiom() {
        assert!(TissueInsultCausesNeoplasia.verify().is_ok());
    }

    #[test]
    fn tissue_insult_causes_stricture_axiom() {
        assert!(TissueInsultCausesStricture.verify().is_ok());
    }

    #[test]
    fn dysplasia_is_premalignant_axiom() {
        assert!(DysplasiaIsPremalignant.verify().is_ok());
    }

    #[test]
    fn normal_has_no_malignant_potential_axiom() {
        assert!(NormalHasNoMalignantPotential.verify().is_ok());
    }

    #[test]
    fn neoplasia_is_malignant_axiom() {
        assert!(NeoplasiaIsMalignant.verify().is_ok());
    }

    #[test]
    fn metaplasia_is_reversible_axiom() {
        assert!(MetaplasiaIsReversible.verify().is_ok());
    }

    #[test]
    fn acute_reversible_neoplasia_irreversible_axiom() {
        assert!(AcuteReversibleNeoplasiaIrreversible.verify().is_ok());
    }

    #[test]
    fn normal_is_polarized_axiom() {
        assert!(NormalIsPolarized.verify().is_ok());
    }

    // -- Subsumption-kind tests --

    #[test]
    fn disease_states_subsume_under_disease_state_umbrella() {
        use PathologyConcept::*;
        let subs: Vec<_> = PathologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PathologyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for state in [
            Normal,
            AcuteInjury,
            ChronicInjury,
            Metaplasia,
            Dysplasia,
            Neoplasia,
            Fibrosis,
            Stricture,
        ] {
            assert!(
                subs.contains(&(state, DiseaseState)),
                "{:?} should subsume under DiseaseState",
                state
            );
        }
    }

    #[test]
    fn events_subsume_under_pathology_event() {
        use PathologyConcept::*;
        let subs: Vec<_> = PathologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PathologyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for ev in [
            TissueInsult,
            AcuteResponse,
            ChronicAdaptation,
            MetaplasticTransformation,
            DysplasticProgression,
            NeoplasticTransformation,
            FibroticRemodeling,
            StrictureFormation,
            LowGradeProgression,
            HighGradeProgression,
        ] {
            assert!(
                subs.contains(&(ev, PathologyEvent)),
                "{:?} should subsume under PathologyEvent",
                ev
            );
        }
    }

    // -- Causation-kind tests --

    #[test]
    fn tissue_insult_directly_causes_acute_response() {
        assert!(causes(
            PathologyConcept::TissueInsult,
            PathologyConcept::AcuteResponse
        ));
    }

    #[test]
    fn chronic_adaptation_causes_fibrotic_remodeling() {
        assert!(causes(
            PathologyConcept::ChronicAdaptation,
            PathologyConcept::FibroticRemodeling
        ));
    }

    #[test]
    fn high_grade_progression_causes_neoplastic_transformation() {
        assert!(causes(
            PathologyConcept::HighGradeProgression,
            PathologyConcept::NeoplasticTransformation
        ));
    }

    #[test]
    fn neoplastic_transformation_does_not_cause_tissue_insult() {
        assert!(!causes(
            PathologyConcept::NeoplasticTransformation,
            PathologyConcept::TissueInsult
        ));
    }

    // -- Opposition-kind tests --

    #[test]
    fn normal_and_neoplasia_oppose() {
        let opps: Vec<_> = PathologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PathologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(PathologyConcept::Normal, PathologyConcept::Neoplasia)));
        assert!(opps.contains(&(PathologyConcept::Neoplasia, PathologyConcept::Normal)));
    }

    #[test]
    fn benign_and_malignant_oppose() {
        let opps: Vec<_> = PathologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PathologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(PathologyConcept::Benign, PathologyConcept::Malignant)));
    }

    #[test]
    fn lowgrade_and_highgrade_oppose() {
        let opps: Vec<_> = PathologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PathologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(PathologyConcept::LowGrade, PathologyConcept::HighGrade)));
    }

    #[test]
    fn normal_does_not_oppose_benign() {
        let opps: Vec<_> = PathologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PathologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(!opps.contains(&(PathologyConcept::Normal, PathologyConcept::Benign)));
    }

    // -- Quality tests --

    #[test]
    fn bioelectric_correlate_normal_polarized() {
        let vmem = BioelectricCorrelate.get(&PathologyConcept::Normal).unwrap();
        assert!(vmem < -40.0, "normal must be polarised, got {}", vmem);
    }

    #[test]
    fn bioelectric_correlate_dysplasia_depolarized() {
        let vmem = BioelectricCorrelate
            .get(&PathologyConcept::Dysplasia)
            .unwrap();
        assert!(vmem > -20.0, "dysplasia must be depolarised, got {}", vmem);
    }

    #[test]
    fn bioelectric_correlate_neoplasia_strongly_depolarized() {
        let vmem = BioelectricCorrelate
            .get(&PathologyConcept::Neoplasia)
            .unwrap();
        assert!(
            vmem > -15.0,
            "neoplasia must be strongly depolarised, got {}",
            vmem
        );
    }

    #[test]
    fn dysplasia_requires_intervention() {
        assert_eq!(
            RequiresIntervention.get(&PathologyConcept::Dysplasia),
            Some(true)
        );
    }

    #[test]
    fn acute_injury_self_resolving() {
        assert_eq!(
            RequiresIntervention.get(&PathologyConcept::AcuteInjury),
            Some(false)
        );
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = PathologyConcept> {
        proptest::sample::select(PathologyConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in PathologyCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in PathologyOntology::axioms() {
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
            let variants: Vec<_> = PathologyConcept::variants();
            for m in PathologyCategory::morphisms() {
                if m.kind() == PathologyRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = PathologyCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == PathologyRelationKind::Opposition)
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

        /// For every disease state, malignant potential is defined.
        #[test]
        fn prop_malignant_potential_defined_for_disease_states(c in arb_concept()) {
            let subs: Vec<_> = PathologyCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == PathologyRelationKind::Subsumption)
                .map(|m| (m.source(), m.target()))
                .collect();
            let is_disease =
                subs.contains(&(c, PathologyConcept::DiseaseState)) && c != PathologyConcept::DiseaseState;
            if is_disease {
                prop_assert!(
                    MalignantPotential.get(&c).is_some(),
                    "MalignantPotential should be defined for disease state {:?}",
                    c
                );
            }
        }
    }
}

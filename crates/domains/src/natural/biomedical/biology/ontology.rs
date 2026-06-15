//! Biological organisation ontology — Cell → Tissue → Organ → Organism.
//!
//! Models the canonical levels of biological organisation in mammalian
//! anatomy, focused on esophageal repair. Encodes both subsumption
//! (is-a) and part-whole (has-a) relationships, plus the developmental
//! and pathological causal chains.
//!
//! Per `feedback_one_ontology_per_module` the original split between
//! `BiologicalEntity` and `BiologicalCausalEvent` has been merged:
//! events are first-class concepts subsumed by the `BiologicalEvent`
//! umbrella.
//!
//! # Literature
//!
//! - **Schleiden & Schwann (1838–1839)** Cell Theory — every organism is
//!   composed of cells; the cell is the smallest unit of life.
//!   (Schleiden, "Beiträge zur Phytogenesis", *Müllers Archiv*, 1838;
//!   Schwann, *Mikroskopische Untersuchungen*, 1839.)
//! - **Virchow (1858)** *Die Cellularpathologie in ihrer Begründung auf
//!   physiologische und pathologische Gewebelehre* — "omnis cellula e
//!   cellula"; cells arise only from pre-existing cells.
//! - **Hooper (1956)** "Cell turnover in epithelial populations",
//!   *J. Histochem. Cytochem.* 4(6):531–540 — basal stem-cell
//!   differentiation in stratified squamous epithelium.
//! - **Piedrafita et al. (2020)** "A single-progenitor model as the
//!   unifying paradigm of epidermal and esophageal epithelial
//!   maintenance in mice", *Nature Communications* 11:1429 — modern
//!   single-progenitor formalisation of esophageal epithelial turnover.

use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Biology",
    source: "Schleiden & Schwann (1838–1839) Cell Theory; Virchow (1858) Cellularpathologie; Hooper (1956) J. Histochem. Cytochem. 4(6):531–540; Piedrafita et al. (2020) Nat. Commun. 11:1429",

    concepts: [
        // === Cells (Schleiden & Schwann 1838–1839) ===
        SquamousEpithelial,
        ColumnarEpithelial,
        GobletCell,
        BasalStemCell,
        Fibroblast,
        MacrophageM1,
        MacrophageM2,
        Osteocyte,

        // === Tissues ===
        SquamousEpithelium,
        ColumnarEpithelium,
        ConnectiveTissue,
        SmoothMuscle,
        NeuralTissue,
        BoneMatrix,

        // === Organs ===
        Esophagus,
        Heart,
        Lung,
        Brain,
        Bone,

        // === Abstract umbrellas ===
        Cell,
        Tissue,
        Organ,
        Organism,
        BiologicalEvent,

        // === Events (merged from BiologicalCausalEvent) ===
        StemCellDivision,
        CellDifferentiation,
        TissueFormation,
        OrganDevelopment,
        AcidDamage,
        InflammationOnset,
        MetaplasticChange,
        FibrosisOnset,
    ],

    labels: {
        SquamousEpithelial: ("en", "Squamous epithelial cell",
            "Hooper (1956): flat differentiated keratinocyte forming the surface of stratified squamous epithelium."),
        ColumnarEpithelial: ("en", "Columnar epithelial cell",
            "Hooper (1956): tall single-layer epithelial cell lining the gastric and intestinal mucosa."),
        GobletCell: ("en", "Goblet cell",
            "Mucus-secreting columnar epithelial cell."),
        BasalStemCell: ("en", "Basal stem cell",
            "Piedrafita et al. (2020): single-progenitor basal-layer stem cell that generates the suprabasal differentiated epithelium."),
        Fibroblast: ("en", "Fibroblast",
            "Connective-tissue cell producing collagen and extracellular matrix."),
        MacrophageM1: ("en", "Macrophage (M1)",
            "Classically activated pro-inflammatory macrophage phenotype."),
        MacrophageM2: ("en", "Macrophage (M2)",
            "Alternatively activated pro-repair macrophage phenotype."),
        Osteocyte: ("en", "Osteocyte",
            "Mechanosensitive bone-resident cell embedded in the mineralised bone matrix."),

        SquamousEpithelium: ("en", "Squamous epithelium",
            "Hooper (1956): stratified squamous epithelium — the canonical esophageal lining."),
        ColumnarEpithelium: ("en", "Columnar epithelium",
            "Single-layer columnar epithelium — gastric/intestinal lining; appears in the esophagus only as Barrett's metaplasia."),
        ConnectiveTissue: ("en", "Connective tissue",
            "Schwann (1839): tissue providing structural and metabolic support — composed of cells, fibres, and ground substance."),
        SmoothMuscle: ("en", "Smooth muscle",
            "Involuntary muscle tissue lining hollow viscera."),
        NeuralTissue: ("en", "Neural tissue",
            "Tissue composed of neurons and supporting glia."),
        BoneMatrix: ("en", "Bone matrix",
            "Mineralised extracellular matrix of bone, secreted by osteoblasts and inhabited by osteocytes."),

        Esophagus: ("en", "Esophagus",
            "Hollow muscular tube conveying food from pharynx to stomach; lined by stratified squamous epithelium."),
        Heart: ("en", "Heart", "Muscular pump driving systemic and pulmonary circulation."),
        Lung: ("en", "Lung", "Organ of gas exchange."),
        Brain: ("en", "Brain", "Central organ of the nervous system."),
        Bone: ("en", "Bone", "Mineralised organ of the skeletal system."),

        Cell: ("en", "Cell (abstract)",
            "Schleiden & Schwann (1838–1839): umbrella for the smallest unit of life — every concrete cell concept is-a Cell."),
        Tissue: ("en", "Tissue (abstract)",
            "Bichat (1801) / Schwann (1839): umbrella for an organised group of cells with a common function."),
        Organ: ("en", "Organ (abstract)",
            "Umbrella for an organised group of tissues with a coordinated function."),
        Organism: ("en", "Organism (abstract)",
            "Umbrella for the whole multicellular individual."),
        BiologicalEvent: ("en", "Biological event (abstract)",
            "Virchow (1858): umbrella for time-extended cellular and tissue-level processes — division, differentiation, damage, inflammation."),

        StemCellDivision: ("en", "Stem-cell division",
            "Virchow (1858) 'omnis cellula e cellula'; Piedrafita et al. (2020): basal stem cell divides, producing daughter cells that can self-renew or differentiate."),
        CellDifferentiation: ("en", "Cell differentiation",
            "Hooper (1956): a daughter basal cell exits the cycle and acquires a differentiated suprabasal phenotype."),
        TissueFormation: ("en", "Tissue formation",
            "Schwann (1839): coordinated proliferation and differentiation produces an organised tissue."),
        OrganDevelopment: ("en", "Organ development",
            "Coordinated formation of multiple tissues into an organ."),
        AcidDamage: ("en", "Acid damage",
            "Mucosal injury from gastric-acid exposure."),
        InflammationOnset: ("en", "Inflammation onset",
            "Virchow (1858): inflammatory response initiated by tissue injury."),
        MetaplasticChange: ("en", "Metaplastic change",
            "Replacement of one differentiated tissue type by another (e.g., squamous→columnar in Barrett's esophagus)."),
        FibrosisOnset: ("en", "Fibrosis onset",
            "Initiation of fibroblast proliferation and collagen deposition replacing normal tissue."),
    },

    is_a: [
        // Cells (Schleiden & Schwann 1838–1839).
        (SquamousEpithelial, Cell),
        (ColumnarEpithelial, Cell),
        (GobletCell, Cell),
        (BasalStemCell, Cell),
        (Fibroblast, Cell),
        (MacrophageM1, Cell),
        (MacrophageM2, Cell),
        (Osteocyte, Cell),

        // Tissues.
        (SquamousEpithelium, Tissue),
        (ColumnarEpithelium, Tissue),
        (ConnectiveTissue, Tissue),
        (SmoothMuscle, Tissue),
        (NeuralTissue, Tissue),
        (BoneMatrix, Tissue),

        // Organs.
        (Esophagus, Organ),
        (Heart, Organ),
        (Lung, Organ),
        (Brain, Organ),
        (Bone, Organ),

        // Events under the BiologicalEvent umbrella.
        (StemCellDivision, BiologicalEvent),
        (CellDifferentiation, BiologicalEvent),
        (TissueFormation, BiologicalEvent),
        (OrganDevelopment, BiologicalEvent),
        (AcidDamage, BiologicalEvent),
        (InflammationOnset, BiologicalEvent),
        (MetaplasticChange, BiologicalEvent),
        (FibrosisOnset, BiologicalEvent),
    ],

    has_a: [
        // Organism contains organs.
        (Organism, Esophagus),
        (Organism, Heart),
        (Organism, Lung),
        (Organism, Brain),
        (Organism, Bone),
        // Esophagus is composed of tissues.
        (Esophagus, SquamousEpithelium),
        (Esophagus, ConnectiveTissue),
        (Esophagus, SmoothMuscle),
        (Esophagus, NeuralTissue),
        // Bone is composed of bone matrix + connective tissue.
        (Bone, BoneMatrix),
        (Bone, ConnectiveTissue),
        // Tissues are composed of cells.
        (SquamousEpithelium, SquamousEpithelial),
        (SquamousEpithelium, BasalStemCell),
        (ColumnarEpithelium, ColumnarEpithelial),
        (ColumnarEpithelium, GobletCell),
        (ConnectiveTissue, Fibroblast),
        (BoneMatrix, Osteocyte),
    ],

    causes: [
        // Developmental chain (Hooper 1956; Piedrafita et al. 2020).
        (StemCellDivision, CellDifferentiation),
        (CellDifferentiation, TissueFormation),
        (TissueFormation, OrganDevelopment),
        // Pathological chain (Virchow 1858).
        (AcidDamage, InflammationOnset),
        (InflammationOnset, MetaplasticChange),
        (InflammationOnset, FibrosisOnset),
    ],

    opposes: [
        // Squamous vs columnar epithelial fate — the Barrett's-metaplasia
        // pair (Chernet & Levin 2013 in the bioelectric framing; the cell
        // fates themselves are Schleiden & Schwann 1838–1839 categories).
        (SquamousEpithelial, ColumnarEpithelial),
        (ColumnarEpithelial, SquamousEpithelial),
        // M1 vs M2 macrophage polarisation (the two opposed phenotypes
        // are clinical classics).
        (MacrophageM1, MacrophageM2),
        (MacrophageM2, MacrophageM1),
        // Cell vs Organism: micro vs macro scale.
        (Cell, Organism),
        (Organism, Cell),
    ],
}

// Backward-compatibility re-exports so existing functor and cross-domain
// consumers (biology↔bioelectric, biology↔molecular, immunology→biology,
// hematology→biology, regeneration→biology) keep compiling.
pub use BiologyCategory as _BiologyCategoryAlias;
pub use BiologyConcept as BiologicalEntity;
pub use BiologyRelation as BiologicalRelation;
pub use BiologyRelationKind as BiologyCategoryRelationKind;

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Level of biological organisation an entity belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrganizationLevel {
    Cellular,
    TissueLevel,
    OrganLevel,
    OrganismLevel,
    Abstract,
}

/// Quality: what organisational level does this entity represent?
///
/// Schleiden & Schwann (1838–1839) → Cell; Bichat / Schwann → Tissue;
/// Virchow (1858) → Organ; Organism is the whole multicellular individual.
#[derive(Debug, Clone)]
pub struct OrganizationLevelQuality;

impl Quality for OrganizationLevelQuality {
    type Individual = BiologyConcept;
    type Value = OrganizationLevel;

    fn get(&self, individual: &BiologyConcept) -> Option<OrganizationLevel> {
        use BiologyConcept::*;
        use OrganizationLevel::*;
        match individual {
            SquamousEpithelial | ColumnarEpithelial | GobletCell | BasalStemCell | Fibroblast
            | MacrophageM1 | MacrophageM2 | Osteocyte => Some(Cellular),
            SquamousEpithelium | ColumnarEpithelium | ConnectiveTissue | SmoothMuscle
            | NeuralTissue | BoneMatrix => Some(TissueLevel),
            Esophagus | Heart | Lung | Brain | Bone => Some(OrganLevel),
            Organism => Some(OrganismLevel),
            Cell | Tissue | Organ => Some(Abstract),
            // Events have no organisational level — they're processes.
            BiologicalEvent | StemCellDivision | CellDifferentiation | TissueFormation
            | OrganDevelopment | AcidDamage | InflammationOnset | MetaplasticChange
            | FibrosisOnset => None,
        }
    }
}

/// Quality: is this entity proliferative (capable of mitotic division)?
///
/// Virchow (1858) — only certain cell types retain proliferative capacity
/// after differentiation; basal stem cells, fibroblasts, and macrophages
/// can divide in adult tissue.
#[derive(Debug, Clone)]
pub struct IsProliferative;

impl Quality for IsProliferative {
    type Individual = BiologyConcept;
    type Value = bool;

    fn get(&self, individual: &BiologyConcept) -> Option<bool> {
        use BiologyConcept::*;
        match individual {
            SquamousEpithelial | ColumnarEpithelial | GobletCell | Osteocyte => Some(false),
            BasalStemCell | MacrophageM1 | MacrophageM2 | Fibroblast => Some(true),
            _ => None,
        }
    }
}

/// Quality: is this entity mechanosensitive?
#[derive(Debug, Clone)]
pub struct IsMechanosensitive;

impl Quality for IsMechanosensitive {
    type Individual = BiologyConcept;
    type Value = bool;

    fn get(&self, individual: &BiologyConcept) -> Option<bool> {
        use BiologyConcept::*;
        match individual {
            SquamousEpithelial | ColumnarEpithelial | Osteocyte | BoneMatrix => Some(true),
            GobletCell | BasalStemCell | Fibroblast | MacrophageM1 | MacrophageM2
            | SquamousEpithelium | ColumnarEpithelium | ConnectiveTissue | SmoothMuscle
            | NeuralTissue => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_a(child: BiologyConcept, parent: BiologyConcept) -> bool {
    BiologyCategory::morphisms().iter().any(|m| {
        m.kind() == BiologyRelationKind::Subsumption && m.source() == child && m.target() == parent
    })
}

fn parts_of(whole: BiologyConcept) -> Vec<BiologyConcept> {
    // Transitive parts: walk parthood edges (the macro emits transitive
    // closure for same-kind relations).
    BiologyCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == BiologyRelationKind::Parthood && m.target() == whole)
        .map(|m| m.source())
        .collect()
}

fn causes(cause: BiologyConcept, effect: BiologyConcept) -> bool {
    BiologyCategory::morphisms().iter().any(|m| {
        m.kind() == BiologyRelationKind::Causation && m.source() == cause && m.target() == effect
    })
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// All concrete cell types are subsumed by the abstract `Cell` (Schleiden
/// & Schwann 1838–1839: every cell is a cell).
pub struct AllCellsAreCell;

impl Axiom for AllCellsAreCell {
    fn verify(&self) -> Verdict {
        use BiologyConcept::*;
        let cells = [
            SquamousEpithelial,
            ColumnarEpithelial,
            GobletCell,
            BasalStemCell,
            Fibroblast,
            MacrophageM1,
            MacrophageM2,
            Osteocyte,
        ];
        if cells.iter().all(|c| is_a(*c, Cell)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AllCellsAreCell",
        "every concrete cell type is subsumed by the abstract Cell concept",
        "Schleiden & Schwann (1838–1839) Cell Theory"
    );
}

pr4xis::register_axiom!(
    AllCellsAreCell,
    "Schleiden & Schwann (1838–1839) Cell Theory"
);

/// The esophagus has squamous epithelium as a part (Hooper 1956 — the
/// canonical esophageal lining is stratified squamous).
pub struct EsophagusHasSquamousEpithelium;

impl Axiom for EsophagusHasSquamousEpithelium {
    fn verify(&self) -> Verdict {
        let parts = parts_of(BiologyConcept::Esophagus);
        if parts.contains(&BiologyConcept::SquamousEpithelium) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EsophagusHasSquamousEpithelium",
        "the esophagus has squamous epithelium as a part",
        "Hooper (1956) J. Histochem. Cytochem. 4(6):531–540"
    );
}

pr4xis::register_axiom!(
    EsophagusHasSquamousEpithelium,
    "Hooper (1956) J. Histochem. Cytochem. 4(6):531–540"
);

/// Squamous epithelium contains both squamous epithelial cells and basal
/// stem cells (Piedrafita et al. 2020 single-progenitor model).
pub struct EpitheliumHasStemCells;

impl Axiom for EpitheliumHasStemCells {
    fn verify(&self) -> Verdict {
        let parts = parts_of(BiologyConcept::SquamousEpithelium);
        if parts.contains(&BiologyConcept::SquamousEpithelial)
            && parts.contains(&BiologyConcept::BasalStemCell)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EpitheliumHasStemCells",
        "squamous epithelium contains both squamous epithelial cells and basal stem cells",
        "Piedrafita et al. (2020) Nat. Commun. 11:1429"
    );
}

pr4xis::register_axiom!(
    EpitheliumHasStemCells,
    "Piedrafita et al. (2020) Nat. Commun. 11:1429"
);

/// All four non-abstract organisational levels are represented in the
/// concept set (cellular, tissue, organ, organism).
pub struct AllLevelsRepresented;

impl Axiom for AllLevelsRepresented {
    fn verify(&self) -> Verdict {
        use OrganizationLevel::*;
        let q = OrganizationLevelQuality;
        let levels: Vec<OrganizationLevel> = BiologyConcept::variants()
            .iter()
            .filter_map(|e| q.get(e))
            .collect();
        let ok = [Cellular, TissueLevel, OrganLevel, OrganismLevel]
            .iter()
            .all(|t| levels.contains(t));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AllLevelsRepresented",
        "all four non-abstract organisational levels (Cellular, Tissue, Organ, Organism) are represented",
        "Schleiden & Schwann (1838–1839); Virchow (1858)"
    );
}

pr4xis::register_axiom!(
    AllLevelsRepresented,
    "Schleiden & Schwann (1838–1839); Virchow (1858)"
);

/// Mechanosensitivity is multi-scale: at least one cellular concept and at
/// least one tissue concept are mechanosensitive.
pub struct MechanosensitivityIsMultiscale;

impl Axiom for MechanosensitivityIsMultiscale {
    fn verify(&self) -> Verdict {
        let mech = IsMechanosensitive;
        let level = OrganizationLevelQuality;
        let mechano: Vec<BiologyConcept> = BiologyConcept::variants()
            .into_iter()
            .filter(|e| mech.get(e) == Some(true))
            .collect();
        let has_cellular = mechano
            .iter()
            .any(|e| level.get(e) == Some(OrganizationLevel::Cellular));
        let has_tissue = mechano
            .iter()
            .any(|e| level.get(e) == Some(OrganizationLevel::TissueLevel));
        if has_cellular && has_tissue {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MechanosensitivityIsMultiscale",
        "mechanosensitive entities exist at both cellular and tissue levels",
        "Hooper (1956); Piedrafita et al. (2020)"
    );
}

pr4xis::register_axiom!(
    MechanosensitivityIsMultiscale,
    "Hooper (1956); Piedrafita et al. (2020)"
);

/// Basal stem cells differentiate into squamous epithelial cells: both
/// coexist in the squamous epithelium; stem cells are proliferative,
/// differentiated cells are not (Hooper 1956; Piedrafita et al. 2020).
pub struct StemCellDifferentiation;

impl Axiom for StemCellDifferentiation {
    fn verify(&self) -> Verdict {
        use BiologyConcept::*;
        let parts = parts_of(SquamousEpithelium);
        let ok = parts.contains(&BasalStemCell)
            && parts.contains(&SquamousEpithelial)
            && IsProliferative.get(&BasalStemCell) == Some(true)
            && IsProliferative.get(&SquamousEpithelial) == Some(false);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StemCellDifferentiation",
        "basal stem cells and squamous epithelial cells coexist in squamous epithelium; only the former proliferate",
        "Hooper (1956) J. Histochem. Cytochem. 4(6):531–540; Piedrafita et al. (2020) Nat. Commun. 11:1429"
    );
}

pr4xis::register_axiom!(
    StemCellDifferentiation,
    "Hooper (1956); Piedrafita et al. (2020) Nat. Commun. 11:1429"
);

/// Acid damage transitively causes metaplastic change (Hooper 1956 ←
/// reflux/inflammation pathway).
pub struct AcidCausesMetaplasia;

impl Axiom for AcidCausesMetaplasia {
    fn verify(&self) -> Verdict {
        if causes(
            BiologyConcept::AcidDamage,
            BiologyConcept::MetaplasticChange,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AcidCausesMetaplasia",
        "acid damage transitively causes metaplastic change via inflammation",
        "Hooper (1956); Virchow (1858)"
    );
}

pr4xis::register_axiom!(AcidCausesMetaplasia, "Hooper (1956); Virchow (1858)");

/// Inflammation transitively causes fibrosis (Virchow 1858).
pub struct InflammationCausesFibrosis;

impl Axiom for InflammationCausesFibrosis {
    fn verify(&self) -> Verdict {
        if causes(
            BiologyConcept::InflammationOnset,
            BiologyConcept::FibrosisOnset,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "InflammationCausesFibrosis",
        "inflammation onset transitively causes fibrosis onset",
        "Virchow (1858) Cellularpathologie"
    );
}

pr4xis::register_axiom!(
    InflammationCausesFibrosis,
    "Virchow (1858) Cellularpathologie"
);

// -- Cross-domain equivalence axioms --

/// The immunology→biology functor preserves MacrophageM1 identity.
pub struct MacrophageM1CrossDomainEquivalence;

impl Axiom for MacrophageM1CrossDomainEquivalence {
    fn verify(&self) -> Verdict {
        use crate::natural::biomedical::immunology::biology_functor::ImmunologyToBiology;
        use crate::natural::biomedical::immunology::ontology::ImmunologyEntity;
        use pr4xis::category::Functor;
        if ImmunologyToBiology::map_object(&ImmunologyEntity::MacrophageM1)
            == BiologyConcept::MacrophageM1
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MacrophageM1CrossDomainEquivalence",
        "MacrophageM1 is the same entity in immunology and biology (functor maps identity)",
        "Murphy (2017) Janeway's Immunobiology 10th ed."
    );
}

pr4xis::register_axiom!(
    MacrophageM1CrossDomainEquivalence,
    "Murphy (2017) Janeway's Immunobiology 10th ed."
);

/// The immunology→biology functor preserves MacrophageM2 identity.
pub struct MacrophageM2CrossDomainEquivalence;

impl Axiom for MacrophageM2CrossDomainEquivalence {
    fn verify(&self) -> Verdict {
        use crate::natural::biomedical::immunology::biology_functor::ImmunologyToBiology;
        use crate::natural::biomedical::immunology::ontology::ImmunologyEntity;
        use pr4xis::category::Functor;
        if ImmunologyToBiology::map_object(&ImmunologyEntity::MacrophageM2)
            == BiologyConcept::MacrophageM2
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MacrophageM2CrossDomainEquivalence",
        "MacrophageM2 is the same entity in immunology and biology (functor maps identity)",
        "Murphy (2017) Janeway's Immunobiology 10th ed."
    );
}

pr4xis::register_axiom!(
    MacrophageM2CrossDomainEquivalence,
    "Murphy (2017) Janeway's Immunobiology 10th ed."
);

/// The immunology→biology functor preserves Fibroblast identity.
pub struct FibroblastCrossDomainEquivalence;

impl Axiom for FibroblastCrossDomainEquivalence {
    fn verify(&self) -> Verdict {
        use crate::natural::biomedical::immunology::biology_functor::ImmunologyToBiology;
        use crate::natural::biomedical::immunology::ontology::ImmunologyEntity;
        use pr4xis::category::Functor;
        if ImmunologyToBiology::map_object(&ImmunologyEntity::Fibroblast)
            == BiologyConcept::Fibroblast
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FibroblastCrossDomainEquivalence",
        "Fibroblast is the same entity in immunology and biology (functor maps identity)",
        "Murphy (2017) Janeway's Immunobiology 10th ed."
    );
}

pr4xis::register_axiom!(
    FibroblastCrossDomainEquivalence,
    "Murphy (2017) Janeway's Immunobiology 10th ed."
);

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

impl Ontology for BiologyOntology {
    type Cat = BiologyCategory;
    type Qual = OrganizationLevelQuality;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(AllCellsAreCell));
        axioms.push(Box::new(EsophagusHasSquamousEpithelium));
        axioms.push(Box::new(EpitheliumHasStemCells));
        axioms.push(Box::new(AllLevelsRepresented));
        axioms.push(Box::new(MechanosensitivityIsMultiscale));
        axioms.push(Box::new(StemCellDifferentiation));
        axioms.push(Box::new(AcidCausesMetaplasia));
        axioms.push(Box::new(InflammationCausesFibrosis));
        axioms.push(Box::new(MacrophageM1CrossDomainEquivalence));
        axioms.push(Box::new(MacrophageM2CrossDomainEquivalence));
        axioms.push(Box::new(FibroblastCrossDomainEquivalence));
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

    #[test]
    fn category_laws() {
        assert_category_laws::<BiologyCategory>();
    }

    #[test]
    fn ontology_validates() {
        BiologyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    // -- Domain axiom tests --

    #[test]
    fn all_cells_are_cell() {
        assert!(AllCellsAreCell.verify().is_ok());
    }

    #[test]
    fn esophagus_has_squamous_epithelium() {
        assert!(EsophagusHasSquamousEpithelium.verify().is_ok());
    }

    #[test]
    fn epithelium_has_stem_cells() {
        assert!(EpitheliumHasStemCells.verify().is_ok());
    }

    #[test]
    fn all_levels_represented() {
        assert!(AllLevelsRepresented.verify().is_ok());
    }

    #[test]
    fn mechanosensitivity_is_multiscale() {
        assert!(MechanosensitivityIsMultiscale.verify().is_ok());
    }

    #[test]
    fn stem_cell_differentiation_axiom() {
        assert!(StemCellDifferentiation.verify().is_ok());
    }

    #[test]
    fn acid_causes_metaplasia() {
        assert!(AcidCausesMetaplasia.verify().is_ok());
    }

    #[test]
    fn inflammation_causes_fibrosis() {
        assert!(InflammationCausesFibrosis.verify().is_ok());
    }

    // -- Cross-domain equivalence tests --

    #[test]
    fn macrophage_m1_cross_domain() {
        assert!(MacrophageM1CrossDomainEquivalence.verify().is_ok());
    }

    #[test]
    fn macrophage_m2_cross_domain() {
        assert!(MacrophageM2CrossDomainEquivalence.verify().is_ok());
    }

    #[test]
    fn fibroblast_cross_domain() {
        assert!(FibroblastCrossDomainEquivalence.verify().is_ok());
    }

    // -- Subsumption / kind tests --

    #[test]
    fn squamous_epithelial_is_a_cell() {
        assert!(is_a(
            BiologyConcept::SquamousEpithelial,
            BiologyConcept::Cell
        ));
    }

    #[test]
    fn osteocyte_is_a_cell() {
        assert!(is_a(BiologyConcept::Osteocyte, BiologyConcept::Cell));
    }

    #[test]
    fn esophagus_is_a_organ() {
        assert!(is_a(BiologyConcept::Esophagus, BiologyConcept::Organ));
    }

    #[test]
    fn cell_is_not_tissue() {
        assert!(!is_a(BiologyConcept::Cell, BiologyConcept::Tissue));
    }

    // -- Parthood / kind tests --

    #[test]
    fn organism_transitively_contains_squamous_epithelial() {
        let parts = parts_of(BiologyConcept::Organism);
        assert!(parts.contains(&BiologyConcept::SquamousEpithelial));
    }

    #[test]
    fn esophagus_transitively_contains_basal_stem_cell() {
        let parts = parts_of(BiologyConcept::Esophagus);
        assert!(parts.contains(&BiologyConcept::BasalStemCell));
    }

    #[test]
    fn bone_transitively_contains_osteocyte() {
        let parts = parts_of(BiologyConcept::Bone);
        assert!(parts.contains(&BiologyConcept::Osteocyte));
    }

    // -- Causation / kind tests --

    #[test]
    fn stem_cell_division_causes_organ_development() {
        assert!(causes(
            BiologyConcept::StemCellDivision,
            BiologyConcept::OrganDevelopment
        ));
    }

    #[test]
    fn acid_damage_causes_metaplastic_change() {
        assert!(causes(
            BiologyConcept::AcidDamage,
            BiologyConcept::MetaplasticChange
        ));
    }

    #[test]
    fn inflammation_causes_fibrosis_direct() {
        assert!(causes(
            BiologyConcept::InflammationOnset,
            BiologyConcept::FibrosisOnset
        ));
    }

    // -- Opposition tests --

    #[test]
    fn squamous_opposes_columnar() {
        let opps: Vec<_> = BiologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            BiologyConcept::SquamousEpithelial,
            BiologyConcept::ColumnarEpithelial
        )));
        assert!(opps.contains(&(
            BiologyConcept::ColumnarEpithelial,
            BiologyConcept::SquamousEpithelial
        )));
    }

    #[test]
    fn m1_opposes_m2() {
        let opps: Vec<_> = BiologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(BiologyConcept::MacrophageM1, BiologyConcept::MacrophageM2)));
    }

    #[test]
    fn cell_opposes_organism() {
        let opps: Vec<_> = BiologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiologyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(BiologyConcept::Cell, BiologyConcept::Organism)));
        assert!(opps.contains(&(BiologyConcept::Organism, BiologyConcept::Cell)));
    }

    // -- Quality tests --

    #[test]
    fn basal_stem_cell_is_proliferative() {
        assert_eq!(
            IsProliferative.get(&BiologyConcept::BasalStemCell),
            Some(true)
        );
    }

    #[test]
    fn osteocyte_is_mechanosensitive() {
        assert_eq!(
            IsMechanosensitive.get(&BiologyConcept::Osteocyte),
            Some(true)
        );
    }

    #[test]
    fn macrophage_not_mechanosensitive() {
        assert_eq!(
            IsMechanosensitive.get(&BiologyConcept::MacrophageM1),
            Some(false)
        );
    }

    #[test]
    fn organization_level_fibroblast_is_cellular() {
        assert_eq!(
            OrganizationLevelQuality.get(&BiologyConcept::Fibroblast),
            Some(OrganizationLevel::Cellular)
        );
    }

    #[test]
    fn organization_level_organism() {
        assert_eq!(
            OrganizationLevelQuality.get(&BiologyConcept::Organism),
            Some(OrganizationLevel::OrganismLevel)
        );
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = BiologyConcept> {
        proptest::sample::select(BiologyConcept::variants())
    }

    fn arb_anatomical_concept() -> impl Strategy<Value = BiologyConcept> {
        // Exclude event concepts which deliberately have no OrganizationLevel.
        proptest::sample::select(vec![
            BiologyConcept::SquamousEpithelial,
            BiologyConcept::ColumnarEpithelial,
            BiologyConcept::GobletCell,
            BiologyConcept::BasalStemCell,
            BiologyConcept::Fibroblast,
            BiologyConcept::MacrophageM1,
            BiologyConcept::MacrophageM2,
            BiologyConcept::Osteocyte,
            BiologyConcept::SquamousEpithelium,
            BiologyConcept::ColumnarEpithelium,
            BiologyConcept::ConnectiveTissue,
            BiologyConcept::SmoothMuscle,
            BiologyConcept::NeuralTissue,
            BiologyConcept::BoneMatrix,
            BiologyConcept::Esophagus,
            BiologyConcept::Heart,
            BiologyConcept::Lung,
            BiologyConcept::Brain,
            BiologyConcept::Bone,
            BiologyConcept::Cell,
            BiologyConcept::Tissue,
            BiologyConcept::Organ,
            BiologyConcept::Organism,
        ])
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in BiologyCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in BiologyOntology::axioms() {
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
        fn prop_organization_level_total_on_anatomy(c in arb_anatomical_concept()) {
            prop_assert!(OrganizationLevelQuality.get(&c).is_some());
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = BiologyConcept::variants();
            for m in BiologyCategory::morphisms() {
                if m.kind() == BiologyRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = BiologyCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == BiologyRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }

        #[test]
        fn prop_proliferative_partial(c in arb_concept()) {
            // IsProliferative is a partial function — never panics.
            let _ = IsProliferative.get(&c);
        }
    }
}

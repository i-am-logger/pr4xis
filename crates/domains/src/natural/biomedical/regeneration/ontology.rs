//! Regeneration biology — the science of how organisms restore lost or
//! damaged structures.
//!
//! Models regeneration types (epimorphic, morphallactic, compensatory,
//! stem-cell-mediated, epithelial restitution), pattern concepts (target
//! morphology, anatomical polarity, body axes, pattern memory, bistability),
//! structures (blastema, wound epithelium, nerve supply), and the causal
//! cascade from injury through wound closure, blastema formation, pattern
//! specification, differentiation, and morphological restoration — with
//! bioelectric, gap-junction, and nerve-derived signalling branches. Per
//! `feedback_one_ontology_per_module` the original split between
//! `RegenerationEntity` and `RegenerationEvent` has been merged: events are
//! first-class concepts subsumed by the `RegenerationEvent` umbrella.
//!
//! # Literature
//!
//! - **Atala, Lanza, Mikos, Nerem (eds.) (2019)** *Principles of Regenerative
//!   Medicine*, 3rd ed., Academic Press — canonical reference for blastema
//!   biology, epimorphic vs morphallactic regeneration, stem-cell-mediated
//!   repair, and compensatory hyperplasia.
//! - **Carlson (2007)** *Principles of Regenerative Biology*, Academic
//!   Press — comparative regeneration (salamander limb, planaria, liver),
//!   blastema and wound-epithelium roles, nerve dependence.
//! - **Tanaka & Reddien (2011)** "The Cellular Basis for Animal
//!   Regeneration", *Developmental Cell* 21(1):172–185 — cellular
//!   mechanisms of vertebrate and invertebrate regeneration, blastema
//!   composition, neural and bioelectric requirements.
//! - **Levin (2012)** "Molecular bioelectricity in developmental biology",
//!   *Dev. Biol.* 369(1):1–4 — Vmem patterns instruct pattern formation.
//! - **Beane et al. / Levin (2017)** *NPJ Regen. Med.* (PMID:28538159) —
//!   bistable bioelectric pattern memory in planarian polarity.
//! - **Singer (1952)** "The influence of the nerve in regeneration of the
//!   amphibian extremity", *Q. Rev. Biol.* 27(2):169–200 — classical
//!   denervation experiments establishing nerve dependence.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Regeneration",
    source: "Atala et al. eds. (2019) Principles of Regenerative Medicine 3rd ed.; Carlson (2007) Principles of Regenerative Biology; Tanaka & Reddien (2011) Dev. Cell 21(1):172-185; Levin (2012) Dev. Biol. 369(1):1-4; Singer (1952) Q. Rev. Biol. 27(2):169-200",

    concepts: [
        // === Regeneration types (Carlson 2007 Ch. 1; Tanaka & Reddien 2011) ===
        Epimorphic,
        Morphallactic,
        Compensatory,
        StemCellMediated,
        EpithelialRestitution,

        // === Pattern concepts (Levin 2012; Beane/Levin 2017) ===
        TargetMorphology,
        AnatomicalPolarity,
        AnteriorPosteriorAxis,
        DorsalVentralAxis,
        LeftRightAxis,
        PatternMemory,
        Bistability,

        // === Structures (Atala et al. 2019; Tanaka & Reddien 2011) ===
        Blastema,
        WoundEpithelium,
        NerveSupply,

        // === Abstract umbrellas ===
        RegenerationType,
        BodyAxis,
        PatternConcept,
        Structure,
        RegenerationEvent,

        // === Causal events (Tanaka & Reddien 2011; Singer 1952; Levin 2012) ===
        Injury,
        WoundClosure,
        BlastemaFormation,
        PatternSpecification,
        Differentiation,
        MorphologicalRestoration,
        BioelectricSignal,
        PolarityDetermination,
        GapJunctionCommunication,
        CollectiveDecision,
        NerveSignaling,
    ],

    labels: {
        Epimorphic: ("en", "Epimorphic regeneration",
            "Carlson (2007) Ch. 1: regrowth of a lost structure via blastema formation (e.g. salamander limb)."),
        Morphallactic: ("en", "Morphallactic regeneration",
            "Carlson (2007) Ch. 1: remodelling of existing tissue without new growth, repolarising the surviving body plan (e.g. planaria)."),
        Compensatory: ("en", "Compensatory regeneration",
            "Atala et al. (2019): organ regrowth by proliferation of surviving differentiated cells (e.g. mammalian liver)."),
        StemCellMediated: ("en", "Stem-cell-mediated regeneration",
            "Atala et al. (2019): tissue replacement driven by resident or recruited stem cells (e.g. haematopoiesis, intestinal crypts)."),
        EpithelialRestitution: ("en", "Epithelial restitution",
            "Atala et al. (2019): rapid wound coverage by cell migration of pre-existing epithelial cells, without proliferation or bioelectric pattern specification (e.g. gut epithelium)."),

        TargetMorphology: ("en", "Target morphology",
            "Levin (2012): the anatomical goal-state that regenerating tissue navigates toward."),
        AnatomicalPolarity: ("en", "Anatomical polarity",
            "Beane/Levin (2017) PMID:28538159: directional information (head-vs-tail, etc.) encoded as a bistable bioelectric state."),
        AnteriorPosteriorAxis: ("en", "Anterior-posterior axis",
            "Carlson (2007): the head-to-tail body axis."),
        DorsalVentralAxis: ("en", "Dorsal-ventral axis",
            "Carlson (2007): the back-to-belly body axis."),
        LeftRightAxis: ("en", "Left-right axis",
            "Carlson (2007): the body's left-right asymmetry axis."),
        PatternMemory: ("en", "Pattern memory",
            "Beane/Levin (2017) PMID:28538159: stored morphogenetic information that persists across regeneration cycles."),
        Bistability: ("en", "Bistability",
            "Beane/Levin (2017) PMID:28538159: two stable bioelectric attractor states (e.g. planarian head-vs-tail polarity) that can be stochastically switched by gap-junction modulation."),

        Blastema: ("en", "Blastema",
            "Tanaka & Reddien (2011): proliferative cell mass at a wound site that gives rise to the regenerated structure (epimorphic regeneration)."),
        WoundEpithelium: ("en", "Wound epithelium",
            "Tanaka & Reddien (2011): epithelial sheet covering the wound that signals to underlying mesenchyme and is essential for blastema formation."),
        NerveSupply: ("en", "Nerve supply",
            "Singer (1952): nerve fibres innervating the wound; nerve-derived trophic factors are required for blastema maintenance and growth."),

        RegenerationType: ("en", "Regeneration type",
            "Carlson (2007): umbrella for distinct modes of regenerative repair (epimorphic, morphallactic, compensatory, stem-cell-mediated, restitution)."),
        BodyAxis: ("en", "Body axis",
            "Carlson (2007): umbrella for the three orthogonal axes (AP, DV, LR) of bilaterian body plan."),
        PatternConcept: ("en", "Pattern concept",
            "Levin (2012): umbrella for concepts describing morphogenetic patterning (target morphology, polarity, body axes, pattern memory, bistability)."),
        Structure: ("en", "Structure",
            "Tanaka & Reddien (2011): umbrella for physical structures participating in regeneration (blastema, wound epithelium, nerve supply)."),
        RegenerationEvent: ("en", "Regeneration event",
            "Tanaka & Reddien (2011): umbrella for time-extended regenerative processes (injury, closure, blastema formation, patterning, differentiation, restoration)."),

        Injury: ("en", "Injury",
            "Tanaka & Reddien (2011): tissue damage that initiates the regeneration cascade."),
        WoundClosure: ("en", "Wound closure",
            "Tanaka & Reddien (2011): rapid epithelial migration sealing the wound."),
        BlastemaFormation: ("en", "Blastema formation",
            "Tanaka & Reddien (2011): assembly of the proliferative cell mass at the wound site."),
        PatternSpecification: ("en", "Pattern specification",
            "Levin (2012); Tanaka & Reddien (2011): determination of what anatomical structure to build."),
        Differentiation: ("en", "Differentiation",
            "Tanaka & Reddien (2011): terminal commitment of blastema cells to target tissue types."),
        MorphologicalRestoration: ("en", "Morphological restoration",
            "Tanaka & Reddien (2011): completion of regeneration with restoration of the original morphology."),
        BioelectricSignal: ("en", "Bioelectric signal",
            "Levin (2012): Vmem-pattern signal that encodes pattern information during regeneration."),
        PolarityDetermination: ("en", "Polarity determination",
            "Beane/Levin (2017): assignment of tissue polarity (e.g. head-vs-tail) via bioelectric pattern."),
        GapJunctionCommunication: ("en", "Gap junction communication",
            "Levin (2012): cell-cell signalling via connexin channels enabling tissue-level pattern computation."),
        CollectiveDecision: ("en", "Collective decision",
            "Levin (2012): tissue-level consensus on target morphology emerging from gap-junction-coupled cells."),
        NerveSignaling: ("en", "Nerve signaling",
            "Singer (1952); Tanaka & Reddien (2011): nerve-derived trophic signalling required for blastema maintenance."),
    },

    is_a: [
        // Regeneration types
        (Epimorphic, RegenerationType),
        (Morphallactic, RegenerationType),
        (Compensatory, RegenerationType),
        (StemCellMediated, RegenerationType),
        (EpithelialRestitution, RegenerationType),

        // Body axes nested under BodyAxis, then PatternConcept (axes are
        // patterning concepts in Carlson 2007 / Levin 2012).
        (AnteriorPosteriorAxis, BodyAxis),
        (DorsalVentralAxis, BodyAxis),
        (LeftRightAxis, BodyAxis),
        (BodyAxis, PatternConcept),

        // Pattern concepts
        (TargetMorphology, PatternConcept),
        (AnatomicalPolarity, PatternConcept),
        (PatternMemory, PatternConcept),
        (Bistability, PatternConcept),

        // Structures
        (Blastema, Structure),
        (WoundEpithelium, Structure),
        (NerveSupply, Structure),

        // Events under RegenerationEvent
        (Injury, RegenerationEvent),
        (WoundClosure, RegenerationEvent),
        (BlastemaFormation, RegenerationEvent),
        (PatternSpecification, RegenerationEvent),
        (Differentiation, RegenerationEvent),
        (MorphologicalRestoration, RegenerationEvent),
        (BioelectricSignal, RegenerationEvent),
        (PolarityDetermination, RegenerationEvent),
        (GapJunctionCommunication, RegenerationEvent),
        (CollectiveDecision, RegenerationEvent),
        (NerveSignaling, RegenerationEvent),
    ],

    causes: [
        // Tanaka & Reddien (2011): canonical epimorphic chain.
        (Injury, WoundClosure),
        (WoundClosure, BlastemaFormation),
        (BlastemaFormation, PatternSpecification),
        (PatternSpecification, Differentiation),
        (Differentiation, MorphologicalRestoration),
        // Levin (2012): bioelectric branch into pattern specification.
        (BioelectricSignal, PolarityDetermination),
        (PolarityDetermination, PatternSpecification),
        // Levin (2012): gap-junction branch into pattern specification.
        (GapJunctionCommunication, CollectiveDecision),
        (CollectiveDecision, PatternSpecification),
        // Singer (1952): nerve branch into blastema formation.
        (NerveSignaling, BlastemaFormation),
    ],

    opposes: [
        // Epimorphic vs EpithelialRestitution — most complex vs simplest
        // regenerative repair (Carlson 2007; Atala et al. 2019).
        (Epimorphic, EpithelialRestitution),
        (EpithelialRestitution, Epimorphic),
        // Blastema vs WoundEpithelium — proliferative core vs protective
        // boundary at the regeneration site (Tanaka & Reddien 2011).
        (Blastema, WoundEpithelium),
        (WoundEpithelium, Blastema),
        // TargetMorphology vs Bistability — single attractor vs multiple
        // stable states in morphospace (Beane/Levin 2017).
        (TargetMorphology, Bistability),
        (Bistability, TargetMorphology),
    ],
}

// Backward-compatibility re-exports for partner functors / sibling crates
// that reference the legacy `*Entity` / `*CategoryRelationKind` names.
pub use RegenerationConcept as RegenerationEntity;
pub use RegenerationRelationKind as RegenerationCategoryRelationKind;

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: does this regeneration type require blastema formation?
///
/// Tanaka & Reddien (2011) — epimorphic regeneration is defined by blastema
/// formation; morphallactic / compensatory / stem-cell-mediated / restitution
/// do not require a blastema.
#[derive(Debug, Clone)]
pub struct RequiresBlastema;

impl Quality for RequiresBlastema {
    type Individual = RegenerationConcept;
    type Value = bool;

    fn get(&self, individual: &RegenerationConcept) -> Option<bool> {
        use RegenerationConcept::*;
        match individual {
            Epimorphic => Some(true),
            Morphallactic | Compensatory | StemCellMediated | EpithelialRestitution => Some(false),
            _ => None,
        }
    }
}

/// Quality: does this regeneration type require nerve supply?
///
/// Singer (1952) — denervation experiments showed amphibian limb
/// regeneration fails without nerves; other regeneration types do not.
#[derive(Debug, Clone)]
pub struct RequiresNerveSupply;

impl Quality for RequiresNerveSupply {
    type Individual = RegenerationConcept;
    type Value = bool;

    fn get(&self, individual: &RegenerationConcept) -> Option<bool> {
        use RegenerationConcept::*;
        match individual {
            Epimorphic => Some(true),
            Morphallactic | Compensatory | StemCellMediated | EpithelialRestitution => Some(false),
            _ => None,
        }
    }
}

/// Quality: is the pattern reversible by bioelectric manipulation?
///
/// Beane/Levin (2017) — planarian head/tail polarity is reversible by gap-
/// junction modulation; pattern memory and target morphology are not.
#[derive(Debug, Clone)]
pub struct IsReversible;

impl Quality for IsReversible {
    type Individual = RegenerationConcept;
    type Value = bool;

    fn get(&self, individual: &RegenerationConcept) -> Option<bool> {
        use RegenerationConcept::*;
        match individual {
            AnatomicalPolarity | Bistability => Some(true),
            TargetMorphology | PatternMemory => Some(false),
            AnteriorPosteriorAxis | DorsalVentralAxis | LeftRightAxis => Some(true),
            _ => None,
        }
    }
}

/// Characteristic timescale for regeneration.
///
/// Carlson (2007); Tanaka & Reddien (2011) — empirical timescales.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegenerationTimescale {
    /// Hours — epithelial restitution in gut (Atala et al. 2019).
    Hours,
    /// Days — planarian head regeneration (~7 days, Tanaka & Reddien 2011).
    Days,
    /// Weeks — salamander limb regeneration (~4-8 weeks, Carlson 2007).
    Weeks,
    /// Months — mammalian liver compensatory regrowth (~2-3 months, Atala et al. 2019).
    Months,
}

/// Quality: how fast does this type of regeneration occur?
#[derive(Debug, Clone)]
pub struct RegenerationSpeed;

impl Quality for RegenerationSpeed {
    type Individual = RegenerationConcept;
    type Value = RegenerationTimescale;

    fn get(&self, individual: &RegenerationConcept) -> Option<RegenerationTimescale> {
        use RegenerationConcept::*;
        use RegenerationTimescale::*;
        match individual {
            EpithelialRestitution => Some(Hours),
            Morphallactic => Some(Days),
            Epimorphic | StemCellMediated => Some(Weeks),
            Compensatory => Some(Months),
            _ => None,
        }
    }
}

/// Quality: does this regeneration type require bioelectric signalling?
///
/// Levin (2012) — pattern specification requires bioelectric signals in
/// every regeneration type except epithelial restitution (which is purely
/// cell-migration-based wound healing).
#[derive(Debug, Clone)]
pub struct RequiresBioelectricSignal;

impl Quality for RequiresBioelectricSignal {
    type Individual = RegenerationConcept;
    type Value = bool;

    fn get(&self, individual: &RegenerationConcept) -> Option<bool> {
        use RegenerationConcept::*;
        match individual {
            Epimorphic | Morphallactic | Compensatory | StemCellMediated => Some(true),
            EpithelialRestitution => Some(false),
            _ => None,
        }
    }
}

/// Model organism in which a given regeneration type is best characterised.
///
/// Carlson (2007); Atala et al. (2019).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelOrganism {
    /// Axolotl / newt (Carlson 2007 Ch. 7).
    Salamander,
    /// Planarian flatworm (Tanaka & Reddien 2011).
    Planarian,
    /// Zebrafish (Atala et al. 2019).
    Zebrafish,
    /// Mouse (Atala et al. 2019).
    Mouse,
    /// Human (Atala et al. 2019).
    Human,
}

/// Quality: primary model organism in which a given regeneration type is
/// best characterised in the literature.
#[derive(Debug, Clone)]
pub struct PrimaryModelOrganism;

impl Quality for PrimaryModelOrganism {
    type Individual = RegenerationConcept;
    type Value = ModelOrganism;

    fn get(&self, entity: &RegenerationConcept) -> Option<ModelOrganism> {
        use RegenerationConcept::*;
        match entity {
            Epimorphic => Some(ModelOrganism::Salamander),
            Morphallactic => Some(ModelOrganism::Planarian),
            Compensatory => Some(ModelOrganism::Zebrafish),
            StemCellMediated => Some(ModelOrganism::Mouse),
            EpithelialRestitution => Some(ModelOrganism::Human),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Ontology + domain axioms
// ---------------------------------------------------------------------------

impl Ontology for RegenerationOntology {
    type Cat = RegenerationCategory;
    type Qual = RequiresBioelectricSignal;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(InjuryCausesRestoration));
        axioms.push(Box::new(BioelectricCausesPattern));
        axioms.push(Box::new(GapJunctionCausesCollectiveDecision));
        axioms.push(Box::new(EpimorphicRequiresBlastemaAndNerve));
        axioms.push(Box::new(EpithelialRestitutionNoBioelectric));
        axioms.push(Box::new(BistabilityIsReversiblePatternConcept));
        axioms
    }
}

/// Helper: does a `Causation` edge exist from `cause` to `effect`?
fn causes(cause: RegenerationConcept, effect: RegenerationConcept) -> bool {
    RegenerationCategory::morphisms().iter().any(|m| {
        m.kind() == RegenerationRelationKind::Causation
            && m.source() == cause
            && m.target() == effect
    })
}

/// Helper: does a `Subsumption` edge exist from `child` to `parent`?
fn is_a(child: RegenerationConcept, parent: RegenerationConcept) -> bool {
    RegenerationCategory::morphisms().iter().any(|m| {
        m.kind() == RegenerationRelationKind::Subsumption
            && m.source() == child
            && m.target() == parent
    })
}

/// Axiom: Injury transitively causes MorphologicalRestoration.
///
/// Tanaka & Reddien (2011) — the complete regeneration cascade.
pub struct InjuryCausesRestoration;

impl Axiom for InjuryCausesRestoration {
    fn verify(&self) -> Verdict {
        let steps = [
            (
                RegenerationConcept::Injury,
                RegenerationConcept::WoundClosure,
            ),
            (
                RegenerationConcept::WoundClosure,
                RegenerationConcept::BlastemaFormation,
            ),
            (
                RegenerationConcept::BlastemaFormation,
                RegenerationConcept::PatternSpecification,
            ),
            (
                RegenerationConcept::PatternSpecification,
                RegenerationConcept::Differentiation,
            ),
            (
                RegenerationConcept::Differentiation,
                RegenerationConcept::MorphologicalRestoration,
            ),
        ];
        if steps.iter().all(|(c, e)| causes(*c, *e)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "InjuryCausesRestoration",
        "Injury transitively causes morphological restoration via the wound-closure / blastema / pattern / differentiation chain",
        "Tanaka & Reddien (2011) Dev. Cell 21(1):172-185"
    );
}

pr4xis::register_axiom!(
    InjuryCausesRestoration,
    "Tanaka & Reddien (2011) Dev. Cell 21(1):172-185"
);

/// Axiom: BioelectricSignal causes PatternSpecification.
///
/// Levin (2012) — Vmem patterns instruct anatomical pattern formation.
pub struct BioelectricCausesPattern;

impl Axiom for BioelectricCausesPattern {
    fn verify(&self) -> Verdict {
        let steps = [
            (
                RegenerationConcept::BioelectricSignal,
                RegenerationConcept::PolarityDetermination,
            ),
            (
                RegenerationConcept::PolarityDetermination,
                RegenerationConcept::PatternSpecification,
            ),
        ];
        if steps.iter().all(|(c, e)| causes(*c, *e)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BioelectricCausesPattern",
        "Bioelectric signals cause pattern specification via polarity determination",
        "Levin (2012) Dev. Biol. 369(1):1-4"
    );
}

pr4xis::register_axiom!(
    BioelectricCausesPattern,
    "Levin (2012) Dev. Biol. 369(1):1-4"
);

/// Axiom: GapJunctionCommunication causes CollectiveDecision.
///
/// Levin (2012) — gap-junction-coupled cells perform tissue-level pattern
/// computation that resolves into a target-morphology decision.
pub struct GapJunctionCausesCollectiveDecision;

impl Axiom for GapJunctionCausesCollectiveDecision {
    fn verify(&self) -> Verdict {
        if causes(
            RegenerationConcept::GapJunctionCommunication,
            RegenerationConcept::CollectiveDecision,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GapJunctionCausesCollectiveDecision",
        "Gap-junction communication causes the tissue-level collective decision on target morphology",
        "Levin (2012) Dev. Biol. 369(1):1-4"
    );
}

pr4xis::register_axiom!(
    GapJunctionCausesCollectiveDecision,
    "Levin (2012) Dev. Biol. 369(1):1-4"
);

/// Axiom: Epimorphic regeneration requires both blastema and nerve supply.
///
/// Singer (1952); Tanaka & Reddien (2011) — both blastema (proliferative
/// mass) and nerve-derived trophic factors are required.
pub struct EpimorphicRequiresBlastemaAndNerve;

impl Axiom for EpimorphicRequiresBlastemaAndNerve {
    fn verify(&self) -> Verdict {
        let blast = RequiresBlastema.get(&RegenerationConcept::Epimorphic) == Some(true);
        let nerve = RequiresNerveSupply.get(&RegenerationConcept::Epimorphic) == Some(true);
        if blast && nerve {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EpimorphicRequiresBlastemaAndNerve",
        "Epimorphic regeneration requires both a blastema and nerve-derived trophic signalling",
        "Singer (1952) Q. Rev. Biol. 27(2):169-200; Tanaka & Reddien (2011) Dev. Cell 21(1):172-185"
    );
}

pr4xis::register_axiom!(
    EpimorphicRequiresBlastemaAndNerve,
    "Singer (1952) Q. Rev. Biol. 27(2):169-200; Tanaka & Reddien (2011) Dev. Cell 21(1):172-185"
);

/// Axiom: EpithelialRestitution does not require bioelectric signalling.
///
/// Atala et al. (2019) — restitution is rapid cell-migration-based wound
/// closure without pattern specification.
pub struct EpithelialRestitutionNoBioelectric;

impl Axiom for EpithelialRestitutionNoBioelectric {
    fn verify(&self) -> Verdict {
        if RequiresBioelectricSignal.get(&RegenerationConcept::EpithelialRestitution) == Some(false)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EpithelialRestitutionNoBioelectric",
        "Epithelial restitution proceeds by cell migration without bioelectric pattern specification",
        "Atala, Lanza, Mikos, Nerem eds. (2019) Principles of Regenerative Medicine 3rd ed."
    );
}

pr4xis::register_axiom!(
    EpithelialRestitutionNoBioelectric,
    "Atala, Lanza, Mikos, Nerem eds. (2019) Principles of Regenerative Medicine 3rd ed."
);

/// Axiom: Bistability is a PatternConcept and is reversible.
///
/// Beane/Levin (2017) — planarian polarity is stored as a bistable
/// bioelectric state that can be stochastically edited.
pub struct BistabilityIsReversiblePatternConcept;

impl Axiom for BistabilityIsReversiblePatternConcept {
    fn verify(&self) -> Verdict {
        let is_pc = is_a(
            RegenerationConcept::Bistability,
            RegenerationConcept::PatternConcept,
        );
        let rev = IsReversible.get(&RegenerationConcept::Bistability) == Some(true);
        if is_pc && rev {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BistabilityIsReversiblePatternConcept",
        "Bistability is a pattern concept and is reversible via bioelectric manipulation",
        "Beane/Levin (2017) NPJ Regen. Med. (PMID:28538159)"
    );
}

pr4xis::register_axiom!(
    BistabilityIsReversiblePatternConcept,
    "Beane/Levin (2017) NPJ Regen. Med. (PMID:28538159)"
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

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<RegenerationCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        RegenerationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn concept_count() {
        // 5 reg types + 7 pattern concepts + 3 structures
        // + 5 abstract umbrellas (RegenerationType, BodyAxis, PatternConcept,
        //   Structure, RegenerationEvent)
        // + 11 events = 31.
        assert_eq!(RegenerationConcept::variants().len(), 31);
    }

    // -- Domain axiom tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn injury_causes_restoration_axiom() {
        assert!(InjuryCausesRestoration.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bioelectric_causes_pattern_axiom() {
        assert!(BioelectricCausesPattern.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn gap_junction_causes_collective_decision_axiom() {
        assert!(GapJunctionCausesCollectiveDecision.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn epimorphic_requires_blastema_and_nerve_axiom() {
        assert!(EpimorphicRequiresBlastemaAndNerve.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn epithelial_restitution_no_bioelectric_axiom() {
        assert!(EpithelialRestitutionNoBioelectric.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bistability_is_reversible_pattern_concept_axiom() {
        assert!(BistabilityIsReversiblePatternConcept.verify().is_ok());
    }

    // -- Subsumption-kind tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn regeneration_types_subsume_under_umbrella() {
        use RegenerationConcept::*;
        for t in [
            Epimorphic,
            Morphallactic,
            Compensatory,
            StemCellMediated,
            EpithelialRestitution,
        ] {
            assert!(
                is_a(t, RegenerationType),
                "{:?} should subsume under RegenerationType",
                t
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn body_axes_subsume_under_body_axis() {
        use RegenerationConcept::*;
        for ax in [AnteriorPosteriorAxis, DorsalVentralAxis, LeftRightAxis] {
            assert!(is_a(ax, BodyAxis), "{:?} should subsume under BodyAxis", ax);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn structures_subsume_under_structure() {
        use RegenerationConcept::*;
        for s in [Blastema, WoundEpithelium, NerveSupply] {
            assert!(is_a(s, Structure), "{:?} should subsume under Structure", s);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn events_subsume_under_regeneration_event() {
        use RegenerationConcept::*;
        for ev in [
            Injury,
            WoundClosure,
            BlastemaFormation,
            PatternSpecification,
            Differentiation,
            MorphologicalRestoration,
            BioelectricSignal,
            PolarityDetermination,
            GapJunctionCommunication,
            CollectiveDecision,
            NerveSignaling,
        ] {
            assert!(
                is_a(ev, RegenerationEvent),
                "{:?} should subsume under RegenerationEvent",
                ev
            );
        }
    }

    // -- Causation-kind tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn injury_directly_causes_wound_closure() {
        assert!(causes(
            RegenerationConcept::Injury,
            RegenerationConcept::WoundClosure
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nerve_signaling_causes_blastema_formation() {
        assert!(causes(
            RegenerationConcept::NerveSignaling,
            RegenerationConcept::BlastemaFormation
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bioelectric_signal_to_polarity_determination() {
        assert!(causes(
            RegenerationConcept::BioelectricSignal,
            RegenerationConcept::PolarityDetermination
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn morphological_restoration_does_not_cause_injury() {
        assert!(!causes(
            RegenerationConcept::MorphologicalRestoration,
            RegenerationConcept::Injury
        ));
    }

    // -- Opposition-kind tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn epimorphic_and_epithelial_restitution_oppose() {
        let opps: Vec<_> = RegenerationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == RegenerationRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            RegenerationConcept::Epimorphic,
            RegenerationConcept::EpithelialRestitution
        )));
        assert!(opps.contains(&(
            RegenerationConcept::EpithelialRestitution,
            RegenerationConcept::Epimorphic
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn blastema_and_wound_epithelium_oppose() {
        let opps: Vec<_> = RegenerationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == RegenerationRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            RegenerationConcept::Blastema,
            RegenerationConcept::WoundEpithelium
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn epimorphic_does_not_oppose_morphallactic() {
        let opps: Vec<_> = RegenerationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == RegenerationRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(!opps.contains(&(
            RegenerationConcept::Epimorphic,
            RegenerationConcept::Morphallactic
        )));
    }

    // -- Quality tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn epimorphic_requires_blastema() {
        assert_eq!(
            RequiresBlastema.get(&RegenerationConcept::Epimorphic),
            Some(true)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn morphallactic_no_blastema() {
        assert_eq!(
            RequiresBlastema.get(&RegenerationConcept::Morphallactic),
            Some(false)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn compensatory_no_nerve_supply() {
        assert_eq!(
            RequiresNerveSupply.get(&RegenerationConcept::Compensatory),
            Some(false)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn anatomical_polarity_is_reversible() {
        assert_eq!(
            IsReversible.get(&RegenerationConcept::AnatomicalPolarity),
            Some(true)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn epithelial_restitution_is_fastest() {
        assert_eq!(
            RegenerationSpeed.get(&RegenerationConcept::EpithelialRestitution),
            Some(RegenerationTimescale::Hours)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn epimorphic_requires_bioelectric_signal() {
        assert_eq!(
            RequiresBioelectricSignal.get(&RegenerationConcept::Epimorphic),
            Some(true)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn all_regeneration_types_have_model_organism() {
        use RegenerationConcept::*;
        for t in [
            Epimorphic,
            Morphallactic,
            Compensatory,
            StemCellMediated,
            EpithelialRestitution,
        ] {
            assert!(
                PrimaryModelOrganism.get(&t).is_some(),
                "{:?} should have a primary model organism",
                t
            );
        }
    }

    // -- Proptests --

    fn arb_concept() -> impl Strategy<Value = RegenerationConcept> {
        proptest::sample::select(RegenerationConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in RegenerationCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in RegenerationOntology::axioms() {
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
            let variants: Vec<_> = RegenerationConcept::variants();
            for m in RegenerationCategory::morphisms() {
                if m.kind() == RegenerationRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = RegenerationCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == RegenerationRelationKind::Opposition)
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

        /// Every regeneration type has a defined speed.
        #[test]
        fn prop_regeneration_type_has_speed(c in arb_concept()) {
            if is_a(c, RegenerationConcept::RegenerationType)
                && c != RegenerationConcept::RegenerationType
            {
                prop_assert!(RegenerationSpeed.get(&c).is_some());
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
    pr4xis::register_praxis_value!(prop_regeneration_type_has_speed, Verifiable);
}

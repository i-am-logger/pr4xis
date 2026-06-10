//! Auditory anatomy — structural hierarchy of the human auditory system.
//!
//! Models the discrete concept set spanning outer / middle / inner ear
//! and the central auditory pathway, plus the canonical taxonomy (e.g.
//! Malleus is-a Ossicle) and mereology (e.g. MiddleEar has-a Malleus).
//!
//! # Literature
//!
//! - **Pickles (2012)** *An Introduction to the Physiology of Hearing*
//!   (4th ed.) — comprehensive reference for auditory-system anatomy
//!   and the outer/middle/inner-ear taxonomy.
//! - **Raphael & Altschuler (2003)** "Structure and innervation of the
//!   cochlea", *Brain Research Bulletin* 60(5–6):397–422.
//! - **Dallos, Popper & Fay (1996)** *The Cochlea* (Springer Handbook of
//!   Auditory Research, vol. 8) — three-scalae structure, hair-cell
//!   anatomy.
//! - **von Békésy (1960)** *Experiments in Hearing* — basilar-membrane
//!   travelling wave, cochlear mechanics.
//! - **Hudspeth (2014)** "Integrating the active process of hair cells
//!   with cochlear function", *Nature Reviews Neuroscience* 15(9):600–614.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::FinitelyGenerated;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Anatomy",
    source: "Pickles (2012) Physiology of Hearing; Raphael & Altschuler (2003); Dallos, Popper & Fay (1996) The Cochlea; von Bekesy (1960) Experiments in Hearing; Hudspeth (2014)",

    concepts: [
        // Outer-ear structures
        Pinna, EarCanal, TympanicMembrane,
        // Middle-ear structures
        Malleus, Incus, Stapes,
        OvalWindow, RoundWindow, EustachianTube,
        TensorTympani, Stapedius,
        // Inner-ear structures
        Cochlea, BasilarMembrane, OrganOfCorti, TectorialMembrane,
        ScalaVestibuli, ScalaMedia, ScalaTympani,
        Endolymph, Perilymph, StriVascularis, ReissnersMembrane,
        Vestibule, SemicircularCanals,
        // Cellular components
        InnerHairCell, OuterHairCell, SupportingCell, SpiralGanglionNeuron,
        // Central auditory pathway
        AuditoryNerve, CochlearNucleus, SuperiorOlivaryComplex,
        InferiorColliculus, MedialGeniculateBody, AuditoryCortex,
        // Abstract groupings
        Ear, OuterEar, MiddleEar, InnerEar,
        Ossicle, HairCell, CochlearFluid, CochlearMembrane, AuditoryNucleus,
    ],

    labels: {
        Pinna: ("en", "Pinna",
            "Pickles (2012): visible cartilaginous outer-ear structure that funnels sound into the ear canal."),
        EarCanal: ("en", "Ear canal",
            "Pickles (2012): external auditory meatus — the ~2.5 cm tube terminating at the tympanic membrane."),
        TympanicMembrane: ("en", "Tympanic membrane",
            "Pickles (2012): the eardrum — first mechanical transducer in air conduction."),
        Malleus: ("en", "Malleus",
            "Pickles (2012): the first ossicle, attached to the tympanic membrane."),
        Incus: ("en", "Incus",
            "Pickles (2012): the middle ossicle articulating malleus to stapes."),
        Stapes: ("en", "Stapes",
            "Pickles (2012): the third ossicle, coupling to the oval window."),
        OvalWindow: ("en", "Oval window",
            "Pickles (2012): the inner-ear-facing membrane driven by the stapes footplate."),
        RoundWindow: ("en", "Round window",
            "Pickles (2012): pressure-relief membrane at the base of scala tympani."),
        EustachianTube: ("en", "Eustachian tube",
            "Pickles (2012): middle-ear → nasopharynx connection that equalises pressure."),
        TensorTympani: ("en", "Tensor tympani",
            "Pickles (2012): middle-ear muscle attached to the malleus."),
        Stapedius: ("en", "Stapedius",
            "Pickles (2012): middle-ear muscle attached to the stapes; acoustic reflex."),
        Cochlea: ("en", "Cochlea",
            "Dallos et al. (1996): the spiral inner-ear organ of hearing."),
        BasilarMembrane: ("en", "Basilar membrane",
            "von Bekesy (1960): the tonotopically-tuned membrane in the cochlea."),
        OrganOfCorti: ("en", "Organ of Corti",
            "Pickles (2012): the sensory epithelium on the basilar membrane housing hair cells."),
        TectorialMembrane: ("en", "Tectorial membrane",
            "Pickles (2012): the acellular membrane overlying the organ of Corti."),
        ScalaVestibuli: ("en", "Scala vestibuli",
            "Dallos et al. (1996): the upper cochlear duct, perilymph-filled."),
        ScalaMedia: ("en", "Scala media",
            "Dallos et al. (1996): the middle cochlear duct, endolymph-filled."),
        ScalaTympani: ("en", "Scala tympani",
            "Dallos et al. (1996): the lower cochlear duct, perilymph-filled."),
        Endolymph: ("en", "Endolymph",
            "Dallos et al. (1996): K+-rich fluid filling scala media."),
        Perilymph: ("en", "Perilymph",
            "Dallos et al. (1996): Na+-rich fluid filling scala vestibuli and scala tympani."),
        StriVascularis: ("en", "Stria vascularis",
            "Pickles (2012): vascular tissue producing the endocochlear potential."),
        ReissnersMembrane: ("en", "Reissner's membrane",
            "Pickles (2012): separates scala vestibuli from scala media."),
        Vestibule: ("en", "Vestibule",
            "Pickles (2012): central inner-ear cavity housing utricle and saccule."),
        SemicircularCanals: ("en", "Semicircular canals",
            "Pickles (2012): three orthogonal canals encoding rotational acceleration."),
        InnerHairCell: ("en", "Inner hair cell",
            "Hudspeth (2014): primary mechanoreceptor; ~3500 per cochlea; afferent."),
        OuterHairCell: ("en", "Outer hair cell",
            "Hudspeth (2014): electromotile amplifier; ~12000 per cochlea."),
        SupportingCell: ("en", "Supporting cell",
            "Pickles (2012): non-sensory cells in the organ of Corti."),
        SpiralGanglionNeuron: ("en", "Spiral ganglion neuron",
            "Raphael & Altschuler (2003): primary afferent neurons of the auditory nerve."),
        AuditoryNerve: ("en", "Auditory nerve",
            "Pickles (2012): CN VIII cochlear branch carrying spiral-ganglion axons."),
        CochlearNucleus: ("en", "Cochlear nucleus",
            "Pickles (2012): first central auditory nucleus in the brainstem."),
        SuperiorOlivaryComplex: ("en", "Superior olivary complex",
            "Pickles (2012): brainstem nucleus performing interaural-time-difference computation."),
        InferiorColliculus: ("en", "Inferior colliculus",
            "Pickles (2012): midbrain auditory integration centre."),
        MedialGeniculateBody: ("en", "Medial geniculate body",
            "Pickles (2012): auditory thalamic relay."),
        AuditoryCortex: ("en", "Auditory cortex",
            "Pickles (2012): primary auditory cortical region (A1) in the temporal lobe."),
        Ear: ("en", "Ear",
            "Pickles (2012): the entire peripheral auditory organ."),
        OuterEar: ("en", "Outer ear",
            "Pickles (2012): pinna + ear canal — pre-tympanic-membrane structures."),
        MiddleEar: ("en", "Middle ear",
            "Pickles (2012): air-filled cavity between tympanic membrane and oval window."),
        InnerEar: ("en", "Inner ear",
            "Pickles (2012): fluid-filled labyrinth containing cochlea and vestibular system."),
        Ossicle: ("en", "Ossicle",
            "Pickles (2012): one of the three middle-ear bones (malleus, incus, stapes)."),
        HairCell: ("en", "Hair cell",
            "Hudspeth (2014): mechanosensory cell with stereociliary bundle."),
        CochlearFluid: ("en", "Cochlear fluid",
            "Dallos et al. (1996): generic term for endolymph or perilymph."),
        CochlearMembrane: ("en", "Cochlear membrane",
            "Pickles (2012): generic term for basilar / tectorial / Reissner's membranes."),
        AuditoryNucleus: ("en", "Auditory nucleus",
            "Pickles (2012): generic term for a central auditory nucleus."),
    },

    is_a: [
        // Ear taxonomy
        (OuterEar, Ear), (MiddleEar, Ear), (InnerEar, Ear),
        (Pinna, OuterEar), (EarCanal, OuterEar), (TympanicMembrane, OuterEar),
        (Ossicle, MiddleEar),
        (Malleus, Ossicle), (Incus, Ossicle), (Stapes, Ossicle),
        (TensorTympani, MiddleEar), (Stapedius, MiddleEar),
        (OvalWindow, MiddleEar), (RoundWindow, MiddleEar), (EustachianTube, MiddleEar),
        (Cochlea, InnerEar), (Vestibule, InnerEar), (SemicircularCanals, InnerEar),
        // Membrane / fluid taxonomy
        (BasilarMembrane, CochlearMembrane),
        (TectorialMembrane, CochlearMembrane),
        (ReissnersMembrane, CochlearMembrane),
        (Endolymph, CochlearFluid), (Perilymph, CochlearFluid),
        // Hair-cell taxonomy
        (InnerHairCell, HairCell), (OuterHairCell, HairCell),
        // Central-nucleus taxonomy
        (CochlearNucleus, AuditoryNucleus),
        (SuperiorOlivaryComplex, AuditoryNucleus),
        (InferiorColliculus, AuditoryNucleus),
        (MedialGeniculateBody, AuditoryNucleus),
    ],

    has_a: [
        // Ear mereology
        (Ear, OuterEar), (Ear, MiddleEar), (Ear, InnerEar),
        (OuterEar, Pinna), (OuterEar, EarCanal), (OuterEar, TympanicMembrane),
        (MiddleEar, Malleus), (MiddleEar, Incus), (MiddleEar, Stapes),
        (MiddleEar, OvalWindow), (MiddleEar, RoundWindow), (MiddleEar, EustachianTube),
        (MiddleEar, TensorTympani), (MiddleEar, Stapedius),
        (InnerEar, Cochlea), (InnerEar, Vestibule), (InnerEar, SemicircularCanals),
        // Cochlear mereology
        (Cochlea, BasilarMembrane), (Cochlea, OrganOfCorti), (Cochlea, TectorialMembrane),
        (Cochlea, ScalaVestibuli), (Cochlea, ScalaMedia), (Cochlea, ScalaTympani),
        (Cochlea, ReissnersMembrane), (Cochlea, StriVascularis),
        (ScalaVestibuli, Perilymph), (ScalaTympani, Perilymph), (ScalaMedia, Endolymph),
        (OrganOfCorti, InnerHairCell), (OrganOfCorti, OuterHairCell), (OrganOfCorti, SupportingCell),
        (Cochlea, SpiralGanglionNeuron),
    ],

    opposes: [
        // OuterEar ↔ InnerEar — extremes of the auditory periphery.
        (OuterEar, InnerEar), (InnerEar, OuterEar),
        // Endolymph (K+-rich) ↔ Perilymph (Na+-rich) — Dallos (1996).
        (Endolymph, Perilymph), (Perilymph, Endolymph),
        // IHC (afferent transduction) ↔ OHC (electromotile amplification)
        // — Hudspeth (2014).
        (InnerHairCell, OuterHairCell), (OuterHairCell, InnerHairCell),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Anatomical-region tag for an auditory entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnatomicalRegion {
    External,
    MiddleEarRegion,
    InnerEarRegion,
    Neural,
    Abstract,
}

#[derive(Debug, Clone)]
pub struct RegionQuality;
impl Quality for RegionQuality {
    type Individual = AnatomyConcept;
    type Value = AnatomicalRegion;
    fn get(&self, individual: &AnatomyConcept) -> Option<AnatomicalRegion> {
        use AnatomicalRegion::*;
        use AnatomyConcept::*;
        Some(match individual {
            Pinna | EarCanal | TympanicMembrane => External,
            Malleus | Incus | Stapes | OvalWindow | RoundWindow | EustachianTube
            | TensorTympani | Stapedius => MiddleEarRegion,
            Cochlea | BasilarMembrane | OrganOfCorti | TectorialMembrane | ScalaVestibuli
            | ScalaMedia | ScalaTympani | Endolymph | Perilymph | StriVascularis
            | ReissnersMembrane | Vestibule | SemicircularCanals | InnerHairCell
            | OuterHairCell | SupportingCell | SpiralGanglionNeuron => InnerEarRegion,
            AuditoryNerve
            | CochlearNucleus
            | SuperiorOlivaryComplex
            | InferiorColliculus
            | MedialGeniculateBody
            | AuditoryCortex => Neural,
            Ear | OuterEar | MiddleEar | InnerEar | Ossicle | HairCell | CochlearFluid
            | CochlearMembrane | AuditoryNucleus => Abstract,
        })
    }
}

#[derive(Debug, Clone)]
pub struct IsMechanicallyActive;
impl Quality for IsMechanicallyActive {
    type Individual = AnatomyConcept;
    type Value = bool;
    fn get(&self, individual: &AnatomyConcept) -> Option<bool> {
        use AnatomyConcept::*;
        Some(matches!(
            individual,
            TympanicMembrane
                | Malleus
                | Incus
                | Stapes
                | OvalWindow
                | RoundWindow
                | BasilarMembrane
                | TectorialMembrane
                | InnerHairCell
                | OuterHairCell
        ))
    }
}

/// Characteristic resonant frequency (Hz) for cavity / mechanical resonators.
/// Pickles (2012) §3 for ear-canal and tympanic-membrane resonances.
#[derive(Debug, Clone)]
pub struct CharacteristicFrequency;
impl Quality for CharacteristicFrequency {
    type Individual = AnatomyConcept;
    type Value = f64;
    fn get(&self, individual: &AnatomyConcept) -> Option<f64> {
        use AnatomyConcept::*;
        match individual {
            Pinna => Some(2700.0),
            EarCanal => Some(3000.0),
            TympanicMembrane => Some(1000.0),
            Stapes => Some(1000.0),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers — taxonomy / mereology queries via kinded-morphism filtering.
// ---------------------------------------------------------------------------

/// Whether `child` is-a `parent` (Subsumption-kinded morphism in the
/// category, transitively closed by the proc macro).
fn is_a(child: AnatomyConcept, parent: AnatomyConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    if child == parent {
        return true;
    }
    AnatomyCategory::morphisms().iter().any(|m| {
        m.kind() == AnatomyRelationKind::Subsumption && m.source() == child && m.target() == parent
    })
}

/// Transitive parts of `whole` (Parthood-kinded morphisms, transitively
/// closed by the proc macro).
fn parts_of(whole: AnatomyConcept) -> Vec<AnatomyConcept> {
    use pr4xis::category::{Arrow, Category};
    AnatomyCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == AnatomyRelationKind::Parthood && m.source() == whole)
        .map(|m| m.target())
        .collect()
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: the three middle-ear ossicles (Pickles 2012 §4) are taxonomically Ossicle.
pub struct ThreeOssicles;
impl Axiom for ThreeOssicles {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AnatomyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if is_a(Malleus, Ossicle) && is_a(Incus, Ossicle) && is_a(Stapes, Ossicle) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ThreeOssicles",
        "malleus, incus, and stapes are ossicles",
        "Pickles (2012) Physiology of Hearing §4"
    );
}
pr4xis::register_axiom!(ThreeOssicles, "Pickles (2012) Physiology of Hearing §4");

/// Axiom: cochlea transitively contains both inner and outer hair cells.
pub struct CochleaContainsHairCells;
impl Axiom for CochleaContainsHairCells {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AnatomyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts = parts_of(Cochlea);
        if parts.contains(&InnerHairCell) && parts.contains(&OuterHairCell) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "CochleaContainsHairCells",
        "cochlea transitively contains both inner and outer hair cells",
        "Pickles (2012); Hudspeth (2014)"
    );
}
pr4xis::register_axiom!(CochleaContainsHairCells, "Pickles (2012); Hudspeth (2014)");

/// Axiom: the ear (Pickles 2012) transitively contains hair cells.
pub struct EarContainsHairCells;
impl Axiom for EarContainsHairCells {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AnatomyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts = parts_of(Ear);
        if parts.contains(&InnerHairCell) && parts.contains(&OuterHairCell) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "EarContainsHairCells",
        "ear transitively contains inner and outer hair cells",
        "Pickles (2012) Physiology of Hearing"
    );
}
pr4xis::register_axiom!(EarContainsHairCells, "Pickles (2012) Physiology of Hearing");

/// Axiom: the cochlea has three scalae — vestibuli, media, tympani.
/// Dallos et al. (1996) — canonical three-fluid-chamber cochlear anatomy.
pub struct CochleaHasThreeScalae;
impl Axiom for CochleaHasThreeScalae {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AnatomyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts = parts_of(Cochlea);
        if parts.contains(&ScalaVestibuli)
            && parts.contains(&ScalaMedia)
            && parts.contains(&ScalaTympani)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "CochleaHasThreeScalae",
        "cochlea contains scala vestibuli, scala media, and scala tympani",
        "Dallos, Popper & Fay (1996) The Cochlea"
    );
}
pr4xis::register_axiom!(
    CochleaHasThreeScalae,
    "Dallos, Popper & Fay (1996) The Cochlea"
);

/// Axiom: all four non-abstract anatomical regions are represented in the
/// concept set (External, MiddleEarRegion, InnerEarRegion, Neural).
pub struct AllRegionsRepresented;
impl Axiom for AllRegionsRepresented {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AnatomicalRegion::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let quality = RegionQuality;
        let all = AnatomyConcept::variants();
        let regions: Vec<AnatomicalRegion> = all.iter().filter_map(|e| quality.get(e)).collect();
        if [External, MiddleEarRegion, InnerEarRegion, Neural]
            .iter()
            .all(|target| regions.contains(target))
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "AllRegionsRepresented",
        "all four non-abstract anatomical regions are represented",
        "Pickles (2012) Physiology of Hearing"
    );
}
pr4xis::register_axiom!(
    AllRegionsRepresented,
    "Pickles (2012) Physiology of Hearing"
);

/// Axiom: both hair-cell subtypes are mechanically active. Hudspeth (2014).
pub struct HairCellsAreMechanicallyActive;
impl Axiom for HairCellsAreMechanicallyActive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AnatomyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if IsMechanicallyActive.get(&InnerHairCell) == Some(true)
            && IsMechanicallyActive.get(&OuterHairCell) == Some(true)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "HairCellsAreMechanicallyActive",
        "both inner and outer hair cells are mechanically active",
        "Hudspeth (2014) Nature Reviews Neuroscience 15:600"
    );
}
pr4xis::register_axiom!(
    HairCellsAreMechanicallyActive,
    "Hudspeth (2014) Nature Reviews Neuroscience 15:600"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for AnatomyOntology {
    type Cat = AnatomyCategory;
    type Qual = RegionQuality;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ThreeOssicles));
        axioms.push(Box::new(CochleaContainsHairCells));
        axioms.push(Box::new(EarContainsHairCells));
        axioms.push(Box::new(CochleaHasThreeScalae));
        axioms.push(Box::new(AllRegionsRepresented));
        axioms.push(Box::new(HairCellsAreMechanicallyActive));
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
        assert_category_laws::<AnatomyCategory>();
    }

    #[test]
    fn ontology_validates() {
        AnatomyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn forty_three_concepts() {
        assert_eq!(AnatomyConcept::variants().len(), 43);
    }

    #[test]
    fn three_ossicles_holds() {
        assert!(ThreeOssicles.verify().is_ok());
    }
    #[test]
    fn cochlea_contains_hair_cells_holds() {
        assert!(CochleaContainsHairCells.verify().is_ok());
    }
    #[test]
    fn ear_contains_hair_cells_holds() {
        assert!(EarContainsHairCells.verify().is_ok());
    }
    #[test]
    fn cochlea_has_three_scalae_holds() {
        assert!(CochleaHasThreeScalae.verify().is_ok());
    }
    #[test]
    fn all_regions_represented_holds() {
        assert!(AllRegionsRepresented.verify().is_ok());
    }
    #[test]
    fn hair_cells_mechanically_active_holds() {
        assert!(HairCellsAreMechanicallyActive.verify().is_ok());
    }

    #[test]
    fn outer_ear_opposes_inner_ear() {
        let opposed: Vec<_> = AnatomyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AnatomyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opposed.contains(&(AnatomyConcept::OuterEar, AnatomyConcept::InnerEar)));
        assert!(opposed.contains(&(AnatomyConcept::Endolymph, AnatomyConcept::Perilymph)));
    }

    #[test]
    fn malleus_is_ossicle() {
        assert!(is_a(AnatomyConcept::Malleus, AnatomyConcept::Ossicle));
    }
    #[test]
    fn inner_hair_cell_is_hair_cell() {
        assert!(is_a(
            AnatomyConcept::InnerHairCell,
            AnatomyConcept::HairCell
        ));
    }
    #[test]
    fn cochlea_is_inner_ear() {
        assert!(is_a(AnatomyConcept::Cochlea, AnatomyConcept::InnerEar));
    }
    #[test]
    fn endolymph_is_cochlear_fluid() {
        assert!(is_a(
            AnatomyConcept::Endolymph,
            AnatomyConcept::CochlearFluid
        ));
    }

    #[test]
    fn ear_contains_cochlea_transitively() {
        let parts = parts_of(AnatomyConcept::Ear);
        assert!(parts.contains(&AnatomyConcept::Cochlea));
    }
    #[test]
    fn cochlea_contains_basilar_membrane() {
        let parts = parts_of(AnatomyConcept::Cochlea);
        assert!(parts.contains(&AnatomyConcept::BasilarMembrane));
    }
    #[test]
    fn cochlea_contains_organ_of_corti() {
        let parts = parts_of(AnatomyConcept::Cochlea);
        assert!(parts.contains(&AnatomyConcept::OrganOfCorti));
    }
    #[test]
    fn organ_of_corti_contains_ihc() {
        let parts = parts_of(AnatomyConcept::OrganOfCorti);
        assert!(parts.contains(&AnatomyConcept::InnerHairCell));
    }
    #[test]
    fn middle_ear_contains_stapes() {
        let parts = parts_of(AnatomyConcept::MiddleEar);
        assert!(parts.contains(&AnatomyConcept::Stapes));
    }

    #[test]
    fn pinna_is_external() {
        assert_eq!(
            RegionQuality.get(&AnatomyConcept::Pinna),
            Some(AnatomicalRegion::External)
        );
    }
    #[test]
    fn stapes_is_middle_ear_region() {
        assert_eq!(
            RegionQuality.get(&AnatomyConcept::Stapes),
            Some(AnatomicalRegion::MiddleEarRegion)
        );
    }
    #[test]
    fn cochlea_is_inner_ear_region() {
        assert_eq!(
            RegionQuality.get(&AnatomyConcept::Cochlea),
            Some(AnatomicalRegion::InnerEarRegion)
        );
    }
    #[test]
    fn auditory_cortex_is_neural() {
        assert_eq!(
            RegionQuality.get(&AnatomyConcept::AuditoryCortex),
            Some(AnatomicalRegion::Neural)
        );
    }
    #[test]
    fn tympanic_membrane_is_mechanically_active() {
        assert_eq!(
            IsMechanicallyActive.get(&AnatomyConcept::TympanicMembrane),
            Some(true)
        );
    }
    #[test]
    fn eustachian_tube_is_not_mechanically_active() {
        assert_eq!(
            IsMechanicallyActive.get(&AnatomyConcept::EustachianTube),
            Some(false)
        );
    }
    #[test]
    fn ear_canal_resonance() {
        assert_eq!(
            CharacteristicFrequency.get(&AnatomyConcept::EarCanal),
            Some(3000.0)
        );
    }

    fn arb_auditory() -> impl Strategy<Value = AnatomyConcept> {
        proptest::sample::select(AnatomyConcept::variants())
    }
    proptest! {
        #[test]
        fn prop_is_a_reflexive(entity in arb_auditory()) {
            prop_assert!(is_a(entity, entity));
        }
        #[test]
        fn prop_region_is_total(entity in arb_auditory()) {
            prop_assert!(RegionQuality.get(&entity).is_some());
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in AnatomyOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
}

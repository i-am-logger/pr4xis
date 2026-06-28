//! Vestibular system — balance and spatial orientation sharing the
//! inner ear with the cochlea.
//!
//! # Literature
//!
//! - **Goldberg et al. (2012)** *The Vestibular System: A Sixth Sense*,
//!   Oxford University Press.
//! - **Angelaki & Cullen (2008)** "Vestibular system: the many facets of
//!   a multimodal sense", *Annu. Rev. Neurosci.* 31:125-150.
//! - **Rabbitt, Damiano & Grant (2004)** "Biomechanics of the
//!   semicircular canals and otolith organs", in *The Vestibular System*.
//! - **Fernandez & Goldberg (1971)** "Physiology of peripheral neurons
//!   innervating semicircular canals of the squirrel monkey", *J.
//!   Neurophysiol.* 34(4):661-675.
//! - **Hudspeth & Corey (1977)** "Sensitivity, polarity, and conductance
//!   change in the response of vertebrate hair cells to controlled
//!   mechanical stimuli", *PNAS* 74(6):2407-2411.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Vestibular",
    source: "Goldberg et al. (2012) The Vestibular System; Angelaki & Cullen (2008) Annu. Rev. Neurosci. 31:125; Rabbitt et al. (2004); Fernandez & Goldberg (1971) J. Neurophysiol. 34(4):661; Hudspeth & Corey (1977) PNAS 74(6):2407",

    concepts: [
        LateralCanal, AnteriorCanal, PosteriorCanal,
        Ampulla, Cupula, CrisaAmpullaris,
        Utricle, Saccule, Macula, Otoconia, OtolithMembrane,
        StriolarRegion, ExtrastriolarRegion,
        TypeIHairCell, TypeIIHairCell, CalyxEnding, BoutonEnding,
        VestibularNerve, ScarpaGanglion,
        VestibularNuclei, MedialVestibularNucleus, LateralVestibularNucleus,
        SuperiorVestibularNucleus,
        CerebellumVestibular,
        VestibuloOcularReflex, VestibuloSpinalReflex, VestibuloColicReflex,
        AngularAcceleration, LinearAcceleration, GravityVector, HeadTilt,
        BPPV, VestibularNeuritis, Vertigo,
        // Umbrellas
        SemicircularCanal, OtolithOrgan, VestibularHairCell, VestibularReflex,
        VestibularStimulus, VestibularDisorder,
        // Events
        HeadRotation, EndolymphFlow, CupulaDeflection,
        CanalHairCellActivation, HeadLinearMotion, OtoconiaShear,
        MaculaHairCellActivation, VestibularAfferentFiring,
        VestibularNucleiProcessing, VORActivation,
        EyeMovementCompensation, PosturalAdjustment,
        VestibularEvent,
    ],

    labels: {
        LateralCanal: ("en", "Lateral canal",
            "Goldberg et al. (2012): horizontal semicircular canal — yaw axis."),
        AnteriorCanal: ("en", "Anterior canal",
            "Goldberg et al. (2012): anterior vertical canal — pitch axis."),
        PosteriorCanal: ("en", "Posterior canal",
            "Goldberg et al. (2012): posterior vertical canal — roll axis."),
        Ampulla: ("en", "Ampulla",
            "Goldberg et al. (2012): canal enlargement housing the cupula."),
        Cupula: ("en", "Cupula",
            "Goldberg et al. (2012): gelatinous diaphragm sealing the ampulla."),
        CrisaAmpullaris: ("en", "Crista ampullaris",
            "Goldberg et al. (2012): hair-cell-bearing crest in the ampulla."),
        Utricle: ("en", "Utricle",
            "Goldberg et al. (2012): otolith organ sensitive to horizontal linear acceleration."),
        Saccule: ("en", "Saccule",
            "Goldberg et al. (2012): otolith organ sensitive to vertical linear acceleration."),
        Macula: ("en", "Macula",
            "Goldberg et al. (2012): hair-cell-bearing sensory epithelium in the otolith organs."),
        Otoconia: ("en", "Otoconia",
            "Goldberg et al. (2012): calcium-carbonate crystals on the otolith membrane."),
        OtolithMembrane: ("en", "Otolith membrane",
            "Goldberg et al. (2012): gelatinous layer carrying the otoconia."),
        StriolarRegion: ("en", "Striolar region",
            "Goldberg et al. (2012): central macular region of opposing hair-cell polarities."),
        ExtrastriolarRegion: ("en", "Extrastriolar region",
            "Goldberg et al. (2012): peripheral macular region."),
        TypeIHairCell: ("en", "Type I hair cell",
            "Hudspeth & Corey (1977): flask-shaped vestibular hair cell with calyx innervation."),
        TypeIIHairCell: ("en", "Type II hair cell",
            "Hudspeth & Corey (1977): cylindrical vestibular hair cell with bouton innervation."),
        CalyxEnding: ("en", "Calyx ending",
            "Goldberg et al. (2012): cup-shaped afferent terminal on Type I cells."),
        BoutonEnding: ("en", "Bouton ending",
            "Goldberg et al. (2012): button-shaped afferent terminal on Type II cells."),
        VestibularNerve: ("en", "Vestibular nerve",
            "Goldberg et al. (2012): CN VIII vestibular branch."),
        ScarpaGanglion: ("en", "Scarpa's ganglion",
            "Goldberg et al. (2012): primary-afferent cell bodies of CN VIII vestibular branch."),
        VestibularNuclei: ("en", "Vestibular nuclei",
            "Goldberg et al. (2012): brainstem complex (medial / lateral / superior / descending)."),
        MedialVestibularNucleus: ("en", "Medial vestibular nucleus",
            "Goldberg et al. (2012): VOR-driving nucleus."),
        LateralVestibularNucleus: ("en", "Lateral vestibular nucleus",
            "Goldberg et al. (2012): vestibulospinal-tract-driving nucleus."),
        SuperiorVestibularNucleus: ("en", "Superior vestibular nucleus",
            "Goldberg et al. (2012): VOR-pathway nucleus."),
        CerebellumVestibular: ("en", "Vestibulocerebellum",
            "Goldberg et al. (2012): cerebellar regions modulating vestibular processing."),
        VestibuloOcularReflex: ("en", "Vestibulo-ocular reflex",
            "Goldberg et al. (2012): gaze stabilisation reflex compensating head rotation."),
        VestibuloSpinalReflex: ("en", "Vestibulo-spinal reflex",
            "Goldberg et al. (2012): postural stabilisation reflex."),
        VestibuloColicReflex: ("en", "Vestibulo-colic reflex",
            "Goldberg et al. (2012): head-on-trunk stabilisation reflex."),
        AngularAcceleration: ("en", "Angular acceleration",
            "Rabbitt et al. (2004): rotational acceleration encoded by the canals."),
        LinearAcceleration: ("en", "Linear acceleration",
            "Goldberg et al. (2012): translational acceleration encoded by the otolith organs."),
        GravityVector: ("en", "Gravity vector",
            "Goldberg et al. (2012): static gravitational reference encoded by otoliths."),
        HeadTilt: ("en", "Head tilt",
            "Goldberg et al. (2012): static-head-orientation stimulus."),
        BPPV: ("en", "BPPV",
            "Goldberg et al. (2012): benign paroxysmal positional vertigo — otoconial displacement."),
        VestibularNeuritis: ("en", "Vestibular neuritis",
            "Goldberg et al. (2012): acute unilateral vestibular-nerve inflammation."),
        Vertigo: ("en", "Vertigo",
            "Goldberg et al. (2012): symptom of perceived rotation."),
        SemicircularCanal: ("en", "Semicircular canal",
            "Goldberg et al. (2012): umbrella for the three rotational sensors."),
        OtolithOrgan: ("en", "Otolith organ",
            "Goldberg et al. (2012): umbrella for utricle + saccule."),
        VestibularHairCell: ("en", "Vestibular hair cell",
            "Hudspeth & Corey (1977): umbrella for Type I + Type II hair cells."),
        VestibularReflex: ("en", "Vestibular reflex",
            "Goldberg et al. (2012): umbrella for vestibulo-* reflexes."),
        VestibularStimulus: ("en", "Vestibular stimulus",
            "Goldberg et al. (2012): umbrella for vestibular inputs."),
        VestibularDisorder: ("en", "Vestibular disorder",
            "Goldberg et al. (2012): umbrella for vestibular pathologies."),
        HeadRotation: ("en", "Head rotation",
            "Rabbitt et al. (2004): event — angular head movement stimulating canals."),
        EndolymphFlow: ("en", "Endolymph flow",
            "Rabbitt et al. (2004): event — fluid flow within a canal from rotation."),
        CupulaDeflection: ("en", "Cupula deflection",
            "Rabbitt et al. (2004): event — cupula bowing from endolymph drag."),
        CanalHairCellActivation: ("en", "Canal hair cell activation",
            "Hudspeth & Corey (1977): event — bundle deflection in the crista."),
        HeadLinearMotion: ("en", "Head linear motion",
            "Goldberg et al. (2012): event — translational head movement."),
        OtoconiaShear: ("en", "Otoconia shear",
            "Goldberg et al. (2012): event — otoconia inertial shear over the macula."),
        MaculaHairCellActivation: ("en", "Macula hair cell activation",
            "Hudspeth & Corey (1977): event — bundle deflection in the macula."),
        VestibularAfferentFiring: ("en", "Vestibular afferent firing",
            "Fernandez & Goldberg (1971) J. Neurophysiol. 34(4):661 — event — primary afferent spike train."),
        VestibularNucleiProcessing: ("en", "Vestibular nuclei processing",
            "Goldberg et al. (2012): event — brainstem integration."),
        VORActivation: ("en", "VOR activation",
            "Goldberg et al. (2012): event — VOR loop engagement."),
        EyeMovementCompensation: ("en", "Eye movement compensation",
            "Goldberg et al. (2012): terminal event — gaze-stabilising eye movement."),
        PosturalAdjustment: ("en", "Postural adjustment",
            "Goldberg et al. (2012): terminal event — postural correction."),
        VestibularEvent: ("en", "Vestibular event",
            "Goldberg et al. (2012): umbrella concept for vestibular perdurants."),
    },

    is_a: [
        (LateralCanal, SemicircularCanal), (AnteriorCanal, SemicircularCanal),
        (PosteriorCanal, SemicircularCanal),
        (Utricle, OtolithOrgan), (Saccule, OtolithOrgan),
        (TypeIHairCell, VestibularHairCell), (TypeIIHairCell, VestibularHairCell),
        (VestibuloOcularReflex, VestibularReflex),
        (VestibuloSpinalReflex, VestibularReflex),
        (VestibuloColicReflex, VestibularReflex),
        (AngularAcceleration, VestibularStimulus),
        (LinearAcceleration, VestibularStimulus),
        (GravityVector, VestibularStimulus), (HeadTilt, VestibularStimulus),
        (BPPV, VestibularDisorder), (VestibularNeuritis, VestibularDisorder),
        (Vertigo, VestibularDisorder),
        (HeadRotation, VestibularEvent), (EndolymphFlow, VestibularEvent),
        (CupulaDeflection, VestibularEvent), (CanalHairCellActivation, VestibularEvent),
        (HeadLinearMotion, VestibularEvent), (OtoconiaShear, VestibularEvent),
        (MaculaHairCellActivation, VestibularEvent),
        (VestibularAfferentFiring, VestibularEvent),
        (VestibularNucleiProcessing, VestibularEvent),
        (VORActivation, VestibularEvent),
        (EyeMovementCompensation, VestibularEvent),
        (PosturalAdjustment, VestibularEvent),
    ],

    has_a: [
        (LateralCanal, Ampulla), (AnteriorCanal, Ampulla), (PosteriorCanal, Ampulla),
        (Ampulla, CrisaAmpullaris), (Ampulla, Cupula),
        (CrisaAmpullaris, TypeIHairCell), (CrisaAmpullaris, TypeIIHairCell),
        (Utricle, Macula), (Saccule, Macula),
        (Macula, Otoconia), (Macula, OtolithMembrane),
        (Macula, TypeIHairCell), (Macula, TypeIIHairCell),
        (Macula, StriolarRegion), (Macula, ExtrastriolarRegion),
    ],

    causes: [
        (HeadRotation, EndolymphFlow),
        (EndolymphFlow, CupulaDeflection),
        (CupulaDeflection, CanalHairCellActivation),
        (HeadLinearMotion, OtoconiaShear),
        (OtoconiaShear, MaculaHairCellActivation),
        (CanalHairCellActivation, VestibularAfferentFiring),
        (MaculaHairCellActivation, VestibularAfferentFiring),
        (VestibularAfferentFiring, VestibularNucleiProcessing),
        (VestibularNucleiProcessing, VORActivation),
        (VORActivation, EyeMovementCompensation),
        (VestibularNucleiProcessing, PosturalAdjustment),
    ],

    opposes: [
        (AngularAcceleration, LinearAcceleration),
        (LinearAcceleration, AngularAcceleration),
        (TypeIHairCell, TypeIIHairCell), (TypeIIHairCell, TypeIHairCell),
        (VestibuloOcularReflex, VestibuloSpinalReflex),
        (VestibuloSpinalReflex, VestibuloOcularReflex),
    ],
}

#[derive(Debug, Clone)]
pub struct TimeConstant;
impl Quality for TimeConstant {
    type Individual = VestibularConcept;
    type Value = f64;
    fn get(&self, individual: &VestibularConcept) -> Option<f64> {
        use VestibularConcept::*;
        match individual {
            Cupula => Some(6.0),
            LateralCanal => Some(6.0),
            VestibularNuclei => Some(17.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VORGain;
impl Quality for VORGain {
    type Individual = VestibularConcept;
    type Value = f64;
    fn get(&self, individual: &VestibularConcept) -> Option<f64> {
        match individual {
            VestibularConcept::VestibuloOcularReflex => Some(1.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CanalSensitivity;
impl Quality for CanalSensitivity {
    type Individual = VestibularConcept;
    type Value = &'static str;
    fn get(&self, individual: &VestibularConcept) -> Option<&'static str> {
        use VestibularConcept::*;
        match individual {
            LateralCanal => Some("horizontal/yaw"),
            AnteriorCanal => Some("sagittal/pitch"),
            PosteriorCanal => Some("coronal/roll"),
            _ => None,
        }
    }
}

fn is_a(child: VestibularConcept, parent: VestibularConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    if child == parent {
        return true;
    }
    VestibularCategory::morphisms().iter().any(|m| {
        m.kind() == VestibularRelationKind::Subsumption
            && m.source() == child
            && m.target() == parent
    })
}

fn parts_of(whole: VestibularConcept) -> Vec<VestibularConcept> {
    use pr4xis::category::{Arrow, Category};
    VestibularCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == VestibularRelationKind::Parthood && m.target() == whole)
        .map(|m| m.source())
        .collect()
}

fn effects_of(cause: VestibularConcept) -> Vec<VestibularConcept> {
    use pr4xis::category::{Arrow, Category};
    VestibularCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == VestibularRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

pub struct ThreeCanals;
impl Axiom for ThreeCanals {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use VestibularConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = [LateralCanal, AnteriorCanal, PosteriorCanal]
            .iter()
            .all(|c| is_a(*c, SemicircularCanal));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ThreeCanals",
        "three semicircular canals are classified",
        "Goldberg et al. (2012) The Vestibular System"
    );
}
pr4xis::register_axiom!(ThreeCanals, "Goldberg et al. (2012) The Vestibular System");

pub struct TwoOtolithOrgans;
impl Axiom for TwoOtolithOrgans {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use VestibularConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if is_a(Utricle, OtolithOrgan) && is_a(Saccule, OtolithOrgan) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "TwoOtolithOrgans",
        "utricle and saccule are otolith organs",
        "Goldberg et al. (2012) The Vestibular System"
    );
}
pr4xis::register_axiom!(
    TwoOtolithOrgans,
    "Goldberg et al. (2012) The Vestibular System"
);

pub struct RotationCausesVOR;
impl Axiom for RotationCausesVOR {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use VestibularConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(HeadRotation).contains(&EyeMovementCompensation) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "RotationCausesVOR",
        "head rotation transitively causes eye movement compensation",
        "Goldberg et al. (2012) The Vestibular System"
    );
}
pr4xis::register_axiom!(
    RotationCausesVOR,
    "Goldberg et al. (2012) The Vestibular System"
);

pub struct CanalsContainHairCells;
impl Axiom for CanalsContainHairCells {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use VestibularConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts = parts_of(LateralCanal);
        if parts.contains(&TypeIHairCell) && parts.contains(&TypeIIHairCell) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "CanalsContainHairCells",
        "semicircular canals transitively contain Type I and Type II hair cells",
        "Hudspeth & Corey (1977) PNAS 74(6):2407"
    );
}
pr4xis::register_axiom!(
    CanalsContainHairCells,
    "Hudspeth & Corey (1977) PNAS 74(6):2407"
);

pub struct VORGainIsUnity;
impl Axiom for VORGainIsUnity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if VORGain.get(&VestibularConcept::VestibuloOcularReflex) == Some(1.0) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "VORGainIsUnity",
        "ideal VOR gain is 1.0 (perfect gaze stabilisation)",
        "Goldberg et al. (2012) The Vestibular System"
    );
}
pr4xis::register_axiom!(
    VORGainIsUnity,
    "Goldberg et al. (2012) The Vestibular System"
);

impl Ontology for VestibularOntology {
    type Cat = VestibularCategory;
    type Qual = TimeConstant;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(ThreeCanals));
        a.push(Box::new(TwoOtolithOrgans));
        a.push(Box::new(RotationCausesVOR));
        a.push(Box::new(CanalsContainHairCells));
        a.push(Box::new(VORGainIsUnity));
        a
    }
}

// Back-compat aliases.
pub use VestibularConcept as VestibularEntity;
pub use VestibularRelationKind as VestibularCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<VestibularCategory>();
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        VestibularOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn three_canals() {
        assert!(ThreeCanals.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn two_otolith_organs() {
        assert!(TwoOtolithOrgans.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn rotation_causes_vor() {
        assert!(RotationCausesVOR.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn canals_contain_hair_cells() {
        assert!(CanalsContainHairCells.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn vor_gain_unity() {
        assert!(VORGainIsUnity.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in VestibularCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in VestibularOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}

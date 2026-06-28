//! Bone conduction — sound reaching the cochlea via skull vibration
//! rather than air-conducted tympanic-membrane drive.
//!
//! Three primary mechanisms (Tonndorf 1966; Stenfelt 2011):
//!   1. **Osseotympanic**: ear-canal-wall vibration → eardrum vibration.
//!   2. **Inertial**: skull vibration → ossicular-chain inertia → oval window.
//!   3. **Compressional**: skull vibration → cochlear-wall compression →
//!      differential fluid motion.
//!
//! Plus the **distortional** low-frequency mode (skull deformation).
//!
//! # Literature
//!
//! - **Tonndorf (1966)** "Bone Conduction: Studies in Experimental
//!   Animals", *Acta Otolaryngol. Suppl.* 213:1-132 — original
//!   classification of BC mechanisms.
//! - **Stenfelt & Goode (2005)** "Bone-Conducted Sound", *Otology &
//!   Neurotology* 26(6):1245-1261.
//! - **Stenfelt (2011)** "Acoustic and Physiologic Aspects of Bone
//!   Conduction Hearing", *Adv. Otorhinolaryngol.* 71:10-21.
//! - **Stenfelt (2015)** "Inner ear contribution to bone conduction
//!   hearing in the human", *Hearing Research* 329:41-51.
//! - **Reinfeldt et al. (2015)** "New developments in bone-conduction
//!   hearing implants", *Med. Devices* 8:79-93.
//! - **von Bekesy (1960)** *Experiments in Hearing*.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "BoneCond",
    source: "Tonndorf (1966) Acta Otolaryngol. Suppl. 213:1; Stenfelt & Goode (2005) Otology & Neurotology 26(6):1245; Stenfelt (2011) Adv. Otorhinolaryngol. 71:10; Stenfelt (2015) Hearing Research 329:41; von Bekesy (1960) Experiments in Hearing",

    concepts: [
        // Mechanisms
        OsseotympanicBC, InertialBC, CompressionalBC, DistortionalBC,
        // Physical events / processes (as concept-nouns)
        SkullVibration, EarCanalWallVibration, OssicularInertia,
        CochlearWallCompression, FluidInertia, SkullDeformation,
        SoundRadiation,
        // Transducers
        BoneAnchoredDevice, PercutaneousImplant, TranscutaneousDevice,
        SkinDriveTransducer, PiezoelectricTransducer, ElectromagneticTransducer,
        // Application sites
        Mastoid, Forehead, TemporalBone, Vertex, Teeth,
        // Phenomena
        OcclusionEffect, TranscranialAttenuation, SkullResonance, ForceLevel,
        // Umbrellas
        BCMechanism, BCTransducer, ApplicationSite, BCPhenomenon,
        // Causal events
        TransducerActivation, SkullCoupling, SkullWavePropagation,
        EarCanalWallMotion, TympanicMembraneResponse, OsseotympanicStimulation,
        OssicularLag, StapesDisplacement, OvalWindowDrive,
        CochlearBoneCompression, DifferentialFluidFlow, BasilarMembraneExcitation,
        SkullModeDeformation, InnerEarDistortion, CochlearResponse,
        BCEvent,
    ],

    labels: {
        OsseotympanicBC: ("en", "Osseotympanic BC",
            "Tonndorf (1966): canal-wall vibration radiating into the canal and driving the eardrum."),
        InertialBC: ("en", "Inertial BC",
            "Tonndorf (1966): differential motion between skull and ossicles drives stapes via inertia."),
        CompressionalBC: ("en", "Compressional BC",
            "Tonndorf (1966): cochlear-wall compression drives fluid asymmetrically across windows."),
        DistortionalBC: ("en", "Distortional BC",
            "Stenfelt (2015): low-frequency skull-mode deformation of the inner ear capsule."),
        SkullVibration: ("en", "Skull vibration",
            "Stenfelt (2011): mechanical oscillation of the cranial vault from a BC transducer."),
        EarCanalWallVibration: ("en", "Ear-canal-wall vibration",
            "Tonndorf (1966): vibration of canal walls coupling acoustic energy into the canal."),
        OssicularInertia: ("en", "Ossicular inertia",
            "Tonndorf (1966): mass-driven lag of ossicles relative to the skull."),
        CochlearWallCompression: ("en", "Cochlear-wall compression",
            "Stenfelt (2015): cyclic compression of the otic capsule by skull-borne waves."),
        FluidInertia: ("en", "Fluid inertia",
            "Stenfelt (2015): mass-driven lag of perilymph/endolymph relative to bone."),
        SkullDeformation: ("en", "Skull deformation",
            "Stenfelt (2015): bending modes of the cranial vault at low frequencies."),
        SoundRadiation: ("en", "Sound radiation",
            "Tonndorf (1966): re-radiation of bone-borne energy into the ear canal."),
        BoneAnchoredDevice: ("en", "Bone-anchored hearing device",
            "Tjellstrom et al. (1981) Am. J. Otol. 2(4):304 — osseointegrated percutaneous BC implant."),
        PercutaneousImplant: ("en", "Percutaneous implant",
            "Reinfeldt et al. (2015): skin-penetrating abutment-based BC implant."),
        TranscutaneousDevice: ("en", "Transcutaneous device",
            "Reinfeldt et al. (2015): subdermal BC implant with magnetic coupling."),
        SkinDriveTransducer: ("en", "Skin-drive transducer",
            "Hakansson et al. (2010): transducer pressed against intact skin."),
        PiezoelectricTransducer: ("en", "Piezoelectric transducer",
            "Hakansson et al. (2010): piezo-crystal BC actuator."),
        ElectromagneticTransducer: ("en", "Electromagnetic transducer",
            "Hakansson et al. (2010): moving-coil BC actuator."),
        Mastoid: ("en", "Mastoid",
            "Tonndorf (1966): bony prominence behind the pinna; canonical BC application site."),
        Forehead: ("en", "Forehead",
            "Tonndorf (1966): midline frontal-bone site with symmetric coupling."),
        TemporalBone: ("en", "Temporal bone",
            "Tonndorf (1966): lateral cranial bone housing the otic capsule."),
        Vertex: ("en", "Vertex",
            "Tonndorf (1966): superior midline cranial site."),
        Teeth: ("en", "Teeth",
            "Reinfeldt et al. (2015): intra-oral BC coupling site."),
        OcclusionEffect: ("en", "Occlusion effect",
            "Tonndorf (1966): low-frequency level rise when the canal is occluded."),
        TranscranialAttenuation: ("en", "Transcranial attenuation",
            "Stenfelt (2011): inter-aural BC level difference."),
        SkullResonance: ("en", "Skull resonance",
            "Stenfelt (2011): cranial vibration mode peaks."),
        ForceLevel: ("en", "Force level",
            "Hakansson et al. (2010): BC stimulus intensity in dB re 1 µN."),
        BCMechanism: ("en", "BC mechanism",
            "Tonndorf (1966): umbrella concept for BC physical pathways."),
        BCTransducer: ("en", "BC transducer",
            "Hakansson et al. (2010): umbrella for BC actuator types."),
        ApplicationSite: ("en", "Application site",
            "Tonndorf (1966): umbrella for stimulus locations on the skull."),
        BCPhenomenon: ("en", "BC phenomenon",
            "Tonndorf (1966): umbrella for emergent BC effects."),
        TransducerActivation: ("en", "Transducer activation",
            "Hakansson et al. (2010): event of BC-driver motion onset."),
        SkullCoupling: ("en", "Skull coupling",
            "Tonndorf (1966): event of force transfer transducer → skull."),
        SkullWavePropagation: ("en", "Skull wave propagation",
            "Stenfelt (2011): event of vibration spreading through cranial bone."),
        EarCanalWallMotion: ("en", "Ear-canal-wall motion",
            "Tonndorf (1966): event of canal-wall vibration arising from skull-borne energy."),
        TympanicMembraneResponse: ("en", "Tympanic-membrane response",
            "Tonndorf (1966): event of TM vibration from radiated canal sound."),
        OsseotympanicStimulation: ("en", "Osseotympanic stimulation",
            "Tonndorf (1966): event of cochlear stimulation via osseotympanic pathway."),
        OssicularLag: ("en", "Ossicular lag",
            "Tonndorf (1966): event of inertial ossicular displacement vs skull."),
        StapesDisplacement: ("en", "Stapes displacement",
            "Tonndorf (1966): event of stapes motion driving the oval window."),
        OvalWindowDrive: ("en", "Oval-window drive",
            "Tonndorf (1966): event of cochlear fluid drive via the oval window."),
        CochlearBoneCompression: ("en", "Cochlear-bone compression",
            "Stenfelt (2015): event of cyclic compression of the otic capsule."),
        DifferentialFluidFlow: ("en", "Differential fluid flow",
            "Stenfelt (2015): event of asymmetric perilymph displacement across windows."),
        BasilarMembraneExcitation: ("en", "Basilar-membrane excitation",
            "von Bekesy (1960): event of BM travelling-wave initiation."),
        SkullModeDeformation: ("en", "Skull-mode deformation",
            "Stenfelt (2015): event of low-frequency cranial bending."),
        InnerEarDistortion: ("en", "Inner-ear distortion",
            "Stenfelt (2015): event of otic-capsule deformation modulating fluid."),
        CochlearResponse: ("en", "Cochlear response",
            "Tonndorf (1966): terminal event — cochlear excitation by BC stimulus."),
        BCEvent: ("en", "BC event",
            "Stenfelt (2011): umbrella concept for any BC perdurant."),
    },

    is_a: [
        (OsseotympanicBC, BCMechanism), (InertialBC, BCMechanism),
        (CompressionalBC, BCMechanism), (DistortionalBC, BCMechanism),
        (BoneAnchoredDevice, BCTransducer), (PercutaneousImplant, BCTransducer),
        (TranscutaneousDevice, BCTransducer), (SkinDriveTransducer, BCTransducer),
        (PiezoelectricTransducer, BCTransducer), (ElectromagneticTransducer, BCTransducer),
        (Mastoid, ApplicationSite), (Forehead, ApplicationSite),
        (TemporalBone, ApplicationSite), (Vertex, ApplicationSite), (Teeth, ApplicationSite),
        (OcclusionEffect, BCPhenomenon), (TranscranialAttenuation, BCPhenomenon),
        (SkullResonance, BCPhenomenon),
        (TransducerActivation, BCEvent), (SkullCoupling, BCEvent),
        (SkullWavePropagation, BCEvent), (EarCanalWallMotion, BCEvent),
        (TympanicMembraneResponse, BCEvent), (OsseotympanicStimulation, BCEvent),
        (OssicularLag, BCEvent), (StapesDisplacement, BCEvent),
        (OvalWindowDrive, BCEvent), (CochlearBoneCompression, BCEvent),
        (DifferentialFluidFlow, BCEvent), (BasilarMembraneExcitation, BCEvent),
        (SkullModeDeformation, BCEvent), (InnerEarDistortion, BCEvent),
        (CochlearResponse, BCEvent),
    ],

    causes: [
        (TransducerActivation, SkullCoupling), (SkullCoupling, SkullWavePropagation),
        (SkullWavePropagation, EarCanalWallMotion),
        (EarCanalWallMotion, TympanicMembraneResponse),
        (TympanicMembraneResponse, OsseotympanicStimulation),
        (OsseotympanicStimulation, CochlearResponse),
        (SkullWavePropagation, OssicularLag),
        (OssicularLag, StapesDisplacement), (StapesDisplacement, OvalWindowDrive),
        (OvalWindowDrive, CochlearResponse),
        (SkullWavePropagation, CochlearBoneCompression),
        (CochlearBoneCompression, DifferentialFluidFlow),
        (DifferentialFluidFlow, BasilarMembraneExcitation),
        (BasilarMembraneExcitation, CochlearResponse),
        (SkullWavePropagation, SkullModeDeformation),
        (SkullModeDeformation, InnerEarDistortion),
        (InnerEarDistortion, CochlearResponse),
    ],

    opposes: [
        (OsseotympanicBC, CompressionalBC), (CompressionalBC, OsseotympanicBC),
        (PercutaneousImplant, TranscutaneousDevice),
        (TranscutaneousDevice, PercutaneousImplant),
        (Mastoid, Forehead), (Forehead, Mastoid),
    ],
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrequencyRange {
    pub low: f64,
    pub high: f64,
}

#[derive(Debug, Clone)]
pub struct DominantFrequencyRange;
impl Quality for DominantFrequencyRange {
    type Individual = BoneCondConcept;
    type Value = FrequencyRange;
    fn get(&self, individual: &BoneCondConcept) -> Option<FrequencyRange> {
        use BoneCondConcept::*;
        match individual {
            OsseotympanicBC => Some(FrequencyRange {
                low: 20.0,
                high: 1000.0,
            }),
            InertialBC => Some(FrequencyRange {
                low: 100.0,
                high: 3000.0,
            }),
            CompressionalBC => Some(FrequencyRange {
                low: 4000.0,
                high: 10000.0,
            }),
            DistortionalBC => Some(FrequencyRange {
                low: 20.0,
                high: 400.0,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranscranialAttenuationDB;
impl Quality for TranscranialAttenuationDB {
    type Individual = BoneCondConcept;
    type Value = f64;
    fn get(&self, individual: &BoneCondConcept) -> Option<f64> {
        use BoneCondConcept::*;
        match individual {
            Mastoid => Some(10.0),
            Forehead => Some(0.0),
            TemporalBone => Some(12.0),
            Vertex => Some(0.0),
            Teeth => Some(5.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkullResonanceFrequency;
impl Quality for SkullResonanceFrequency {
    type Individual = BoneCondConcept;
    type Value = f64;
    fn get(&self, individual: &BoneCondConcept) -> Option<f64> {
        use BoneCondConcept::*;
        match individual {
            Mastoid => Some(200.0),
            Forehead => Some(800.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequiresSurgery;
impl Quality for RequiresSurgery {
    type Individual = BoneCondConcept;
    type Value = bool;
    fn get(&self, individual: &BoneCondConcept) -> Option<bool> {
        use BoneCondConcept::*;
        match individual {
            BoneAnchoredDevice | PercutaneousImplant | TranscutaneousDevice => Some(true),
            SkinDriveTransducer | PiezoelectricTransducer | ElectromagneticTransducer => {
                Some(false)
            }
            _ => None,
        }
    }
}

fn effects_of(cause: BoneCondConcept) -> Vec<BoneCondConcept> {
    use pr4xis::category::{Arrow, Category};
    BoneCondCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == BoneCondRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

pub struct FourBCMechanisms;
impl Axiom for FourBCMechanisms {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use BoneCondConcept::*;
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let is_a = |child: BoneCondConcept, parent: BoneCondConcept| {
            child == parent
                || BoneCondCategory::morphisms().iter().any(|m| {
                    m.kind() == BoneCondRelationKind::Subsumption
                        && m.source() == child
                        && m.target() == parent
                })
        };
        let ok = [OsseotympanicBC, InertialBC, CompressionalBC, DistortionalBC]
            .iter()
            .all(|m| is_a(*m, BCMechanism));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "FourBCMechanisms",
        "all four BC mechanisms (osseotympanic, inertial, compressional, distortional) are classified",
        "Tonndorf (1966) Acta Otolaryngol. Suppl. 213:1"
    );
}
pr4xis::register_axiom!(
    FourBCMechanisms,
    "Tonndorf (1966) Acta Otolaryngol. Suppl. 213:1"
);

pub struct TransducerCausesCochlearResponse;
impl Axiom for TransducerCausesCochlearResponse {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use BoneCondConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(TransducerActivation).contains(&CochlearResponse) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "TransducerCausesCochlearResponse",
        "transducer activation transitively causes cochlear response",
        "Stenfelt (2011) Adv. Otorhinolaryngol. 71:10"
    );
}
pr4xis::register_axiom!(
    TransducerCausesCochlearResponse,
    "Stenfelt (2011) Adv. Otorhinolaryngol. 71:10"
);

pub struct AllPathwaysConverge;
impl Axiom for AllPathwaysConverge {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use BoneCondConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = effects_of(OsseotympanicStimulation).contains(&CochlearResponse)
            && effects_of(OvalWindowDrive).contains(&CochlearResponse)
            && effects_of(BasilarMembraneExcitation).contains(&CochlearResponse);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "AllPathwaysConverge",
        "osseotympanic, inertial, and compressional pathways all reach cochlear response",
        "Tonndorf (1966) Acta Otolaryngol. Suppl. 213:1"
    );
}
pr4xis::register_axiom!(
    AllPathwaysConverge,
    "Tonndorf (1966) Acta Otolaryngol. Suppl. 213:1"
);

pub struct ForeheadResonanceHigherThanMastoid;
impl Axiom for ForeheadResonanceHigherThanMastoid {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use BoneCondConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let f = SkullResonanceFrequency.get(&Forehead).unwrap_or(0.0);
        let m = SkullResonanceFrequency.get(&Mastoid).unwrap_or(0.0);
        if f > m {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ForeheadResonanceHigherThanMastoid",
        "forehead skull resonance frequency is higher than mastoid",
        "Stenfelt (2011) Adv. Otorhinolaryngol. 71:10"
    );
}
pr4xis::register_axiom!(
    ForeheadResonanceHigherThanMastoid,
    "Stenfelt (2011) Adv. Otorhinolaryngol. 71:10"
);

impl Ontology for BoneCondOntology {
    type Cat = BoneCondCategory;
    type Qual = TranscranialAttenuationDB;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(FourBCMechanisms));
        a.push(Box::new(TransducerCausesCochlearResponse));
        a.push(Box::new(AllPathwaysConverge));
        a.push(Box::new(ForeheadResonanceHigherThanMastoid));
        a
    }
}

// Back-compat aliases used by sibling functors and `adjunctions.rs`.
pub use BoneCondCategory as BoneConductionCategory;
pub use BoneCondConcept as BoneCondEntity;
pub use BoneCondOntology as BoneConductionOntology;
pub use BoneCondRelationKind as BoneConductionCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<BoneCondCategory>();
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        BoneCondOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn four_bc_mechanisms() {
        assert!(FourBCMechanisms.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn transducer_causes_cochlear() {
        assert!(TransducerCausesCochlearResponse.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn all_pathways_converge() {
        assert!(AllPathwaysConverge.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in BoneCondCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in BoneCondOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}

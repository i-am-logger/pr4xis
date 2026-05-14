//! Hearing devices — assistive technology and diagnostic equipment.
//!
//! # Literature
//!
//! - **Dillon (2012)** *Hearing Aids* (2nd ed.), Thieme.
//! - **Zeng, Popper & Fay (2008)** *Cochlear Implants*, Springer.
//! - **Tjellstrom, Hakansson & Lindstrom (1981)** "Bone-Anchored
//!   Hearing Aid", *Am. J. Otol.* 2(4):304-310.
//! - **Hakansson, Tjellstrom & Carlsson (2010)** "Bone-Conduction
//!   Hearing Devices", *Adv. Otorhinolaryngol.* 71:51-58.
//! - **Chasin (2006)** *Hearing Loss in Musicians*, Plural Publishing.
//!
//! # Design
//!
//! Per `feedback_one_ontology_per_module`, the dual-enum has been merged.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Device",
    source: "Dillon (2012) Hearing Aids 2nd ed.; Zeng et al. (2008) Cochlear Implants; Tjellstrom et al. (1981) Am. J. Otol. 2(4):304; Hakansson et al. (2010) Adv. Otorhinolaryngol. 71:51",

    concepts: [
        BehindTheEar, InTheEar, CompletelyInCanal, ReceiverInCanal,
        CROS, BiCROS,
        CochlearImplant, BoneAnchoredHearingAid, MiddleEarImplant, AuditoryBrainstemImplant,
        BoneConductionHeadphone, SoftbandBAHA, AdhesiveBC,
        DirectionalMicrophone, NoiseSuppression, FeedbackCancellation,
        FrequencyCompression, WideAdaptiveDynamicRange, Telecoil, BluetoothStreaming,
        Audiometer, Tympanometer, OAEProbe, ABRSystem, RealEarMeasurement,
        Microphone, Amplifier, Receiver, ElectrodeArray, SpeechProcessor,
        HearingAid, ImplantableDevice, BCDevice, SignalProcessingFeature,
        DiagnosticEquipment, DeviceComponent,
        // Events
        HearingLossDiagnosis, DeviceSelection, CustomMolding, InitialFitting,
        RealEarVerificationEvent, FineTuning, OutcomeImprovement,
        DeviceEvent,
    ],

    labels: {
        BehindTheEar: ("en", "Behind-the-ear hearing aid",
            "Dillon (2012) §1: post-auricular shell with custom earmold or thin tube."),
        InTheEar: ("en", "In-the-ear hearing aid",
            "Dillon (2012) §1: concha-shell custom-fit hearing aid."),
        CompletelyInCanal: ("en", "Completely-in-canal hearing aid",
            "Dillon (2012) §1: deep canal hearing aid."),
        ReceiverInCanal: ("en", "Receiver-in-canal hearing aid",
            "Dillon (2012) §1: receiver in the canal, electronics behind ear."),
        CROS: ("en", "CROS",
            "Dillon (2012) §15: contralateral routing of signal for unilateral loss."),
        BiCROS: ("en", "BiCROS",
            "Dillon (2012) §15: bilateral CROS with better-ear aid."),
        CochlearImplant: ("en", "Cochlear implant",
            "Zeng et al. (2008): intracochlear electrode array stimulating the auditory nerve."),
        BoneAnchoredHearingAid: ("en", "Bone-anchored hearing aid",
            "Tjellstrom et al. (1981) Am. J. Otol. 2(4):304 — osseointegrated BC implant."),
        MiddleEarImplant: ("en", "Middle-ear implant",
            "Dillon (2012) §15: ossicle-coupled mechanical actuator."),
        AuditoryBrainstemImplant: ("en", "Auditory brainstem implant",
            "Zeng et al. (2008): cochlear-nucleus surface electrode array."),
        BoneConductionHeadphone: ("en", "Bone-conduction headphone",
            "Hakansson et al. (2010): consumer skin-drive BC transducer."),
        SoftbandBAHA: ("en", "Softband BAHA",
            "Hakansson et al. (2010): non-implanted headband-mounted BC device."),
        AdhesiveBC: ("en", "Adhesive BC",
            "Reinfeldt et al. (2015) Med. Devices 8:79 — adhesive-mounted BC actuator."),
        DirectionalMicrophone: ("en", "Directional microphone",
            "Dillon (2012) §7: polar-pattern microphone for SNR improvement."),
        NoiseSuppression: ("en", "Noise suppression",
            "Dillon (2012) §8: spectral-subtraction-style noise reduction."),
        FeedbackCancellation: ("en", "Feedback cancellation",
            "Dillon (2012) §6: adaptive filter cancelling acoustic feedback."),
        FrequencyCompression: ("en", "Frequency compression",
            "Dillon (2012) §10: nonlinear frequency lowering for severe high-frequency loss."),
        WideAdaptiveDynamicRange: ("en", "Wide-adaptive dynamic range",
            "Dillon (2012) §10: WDRC compression mapping wide input range to narrow output."),
        Telecoil: ("en", "Telecoil",
            "Dillon (2012) §15: inductive pickup for loop systems."),
        BluetoothStreaming: ("en", "Bluetooth streaming",
            "Dillon (2012) §15: wireless audio input to hearing aids."),
        Audiometer: ("en", "Audiometer",
            "Katz et al. (2015): pure-tone / speech threshold instrument."),
        Tympanometer: ("en", "Tympanometer",
            "Jerger (1970): admittance measurement instrument."),
        OAEProbe: ("en", "OAE probe",
            "Kemp (1978): otoacoustic-emission probe assembly."),
        ABRSystem: ("en", "ABR system",
            "Jewett & Williston (1971): brainstem-response evoked-potential system."),
        RealEarMeasurement: ("en", "Real-ear measurement",
            "Dillon (2012) §10: probe-microphone-based hearing-aid verification."),
        Microphone: ("en", "Microphone",
            "Dillon (2012) §5: acousto-electric transducer."),
        Amplifier: ("en", "Amplifier",
            "Dillon (2012) §6: analogue or digital signal gain stage."),
        Receiver: ("en", "Receiver",
            "Dillon (2012) §5: electro-acoustic output transducer."),
        ElectrodeArray: ("en", "Electrode array",
            "Zeng et al. (2008): intracochlear stimulating contacts."),
        SpeechProcessor: ("en", "Speech processor",
            "Zeng et al. (2008): CI external processing unit."),
        HearingAid: ("en", "Hearing aid",
            "Dillon (2012): umbrella concept for acoustic-output devices."),
        ImplantableDevice: ("en", "Implantable device",
            "Zeng et al. (2008): umbrella concept for surgically-implanted hearing devices."),
        BCDevice: ("en", "BC device",
            "Hakansson et al. (2010): umbrella concept for bone-conduction devices."),
        SignalProcessingFeature: ("en", "Signal-processing feature",
            "Dillon (2012): umbrella for in-aid DSP features."),
        DiagnosticEquipment: ("en", "Diagnostic equipment",
            "Katz et al. (2015): umbrella concept for clinical instruments."),
        DeviceComponent: ("en", "Device component",
            "Dillon (2012): umbrella for hardware subcomponents."),
        HearingLossDiagnosis: ("en", "Hearing-loss diagnosis",
            "Katz et al. (2015): event initiating the fitting pathway."),
        DeviceSelection: ("en", "Device selection",
            "Dillon (2012): event of choosing a device type."),
        CustomMolding: ("en", "Custom molding",
            "Dillon (2012): event of fabricating earmold or shell."),
        InitialFitting: ("en", "Initial fitting",
            "Dillon (2012): event of first device insertion + programming."),
        RealEarVerificationEvent: ("en", "Real-ear verification",
            "Dillon (2012): event of in-situ output measurement."),
        FineTuning: ("en", "Fine tuning",
            "Dillon (2012): event of programming adjustment."),
        OutcomeImprovement: ("en", "Outcome improvement",
            "Dillon (2012): terminal event — measurable benefit achieved."),
        DeviceEvent: ("en", "Device event",
            "Dillon (2012): umbrella for fitting-pathway perdurants."),
    },

    is_a: [
        (BehindTheEar, HearingAid), (InTheEar, HearingAid),
        (CompletelyInCanal, HearingAid), (ReceiverInCanal, HearingAid),
        (CROS, HearingAid), (BiCROS, HearingAid),
        (CochlearImplant, ImplantableDevice), (BoneAnchoredHearingAid, ImplantableDevice),
        (MiddleEarImplant, ImplantableDevice), (AuditoryBrainstemImplant, ImplantableDevice),
        (BoneConductionHeadphone, BCDevice), (SoftbandBAHA, BCDevice),
        (AdhesiveBC, BCDevice), (BoneAnchoredHearingAid, BCDevice),
        (DirectionalMicrophone, SignalProcessingFeature),
        (NoiseSuppression, SignalProcessingFeature),
        (FeedbackCancellation, SignalProcessingFeature),
        (FrequencyCompression, SignalProcessingFeature),
        (WideAdaptiveDynamicRange, SignalProcessingFeature),
        (Telecoil, SignalProcessingFeature),
        (BluetoothStreaming, SignalProcessingFeature),
        (Audiometer, DiagnosticEquipment), (Tympanometer, DiagnosticEquipment),
        (OAEProbe, DiagnosticEquipment), (ABRSystem, DiagnosticEquipment),
        (RealEarMeasurement, DiagnosticEquipment),
        (Microphone, DeviceComponent), (Amplifier, DeviceComponent),
        (Receiver, DeviceComponent), (ElectrodeArray, DeviceComponent),
        (SpeechProcessor, DeviceComponent),
        (HearingLossDiagnosis, DeviceEvent), (DeviceSelection, DeviceEvent),
        (CustomMolding, DeviceEvent), (InitialFitting, DeviceEvent),
        (RealEarVerificationEvent, DeviceEvent), (FineTuning, DeviceEvent),
        (OutcomeImprovement, DeviceEvent),
    ],

    has_a: [
        (BehindTheEar, Microphone), (BehindTheEar, Amplifier), (BehindTheEar, Receiver),
        (CochlearImplant, ElectrodeArray), (CochlearImplant, SpeechProcessor),
        (CochlearImplant, Microphone),
        (HearingAid, DirectionalMicrophone), (HearingAid, NoiseSuppression),
        (HearingAid, FeedbackCancellation),
    ],

    causes: [
        (HearingLossDiagnosis, DeviceSelection),
        (DeviceSelection, CustomMolding),
        (CustomMolding, InitialFitting),
        (InitialFitting, RealEarVerificationEvent),
        (RealEarVerificationEvent, FineTuning),
        (FineTuning, OutcomeImprovement),
    ],

    opposes: [
        (CochlearImplant, HearingAid), (HearingAid, CochlearImplant),
        (BehindTheEar, CompletelyInCanal), (CompletelyInCanal, BehindTheEar),
        (DirectionalMicrophone, Telecoil), (Telecoil, DirectionalMicrophone),
    ],
}

#[derive(Debug, Clone)]
pub struct MaxGainDB;
impl Quality for MaxGainDB {
    type Individual = DeviceConcept;
    type Value = f64;
    fn get(&self, individual: &DeviceConcept) -> Option<f64> {
        use DeviceConcept::*;
        match individual {
            CompletelyInCanal => Some(40.0),
            InTheEar => Some(55.0),
            BehindTheEar => Some(75.0),
            ReceiverInCanal => Some(60.0),
            CochlearImplant => Some(120.0),
            BoneAnchoredHearingAid => Some(45.0),
            BoneConductionHeadphone => Some(30.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatteryLifeDays;
impl Quality for BatteryLifeDays {
    type Individual = DeviceConcept;
    type Value = f64;
    fn get(&self, individual: &DeviceConcept) -> Option<f64> {
        use DeviceConcept::*;
        match individual {
            BehindTheEar => Some(7.0),
            CochlearImplant => Some(1.0),
            CompletelyInCanal => Some(5.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequiresSurgery;
impl Quality for RequiresSurgery {
    type Individual = DeviceConcept;
    type Value = bool;
    fn get(&self, individual: &DeviceConcept) -> Option<bool> {
        use DeviceConcept::*;
        match individual {
            CochlearImplant
            | BoneAnchoredHearingAid
            | MiddleEarImplant
            | AuditoryBrainstemImplant => Some(true),
            BehindTheEar | InTheEar | CompletelyInCanal | ReceiverInCanal => Some(false),
            BoneConductionHeadphone | SoftbandBAHA | AdhesiveBC => Some(false),
            CROS | BiCROS => Some(false),
            _ => None,
        }
    }
}

fn parts_of(whole: DeviceConcept) -> Vec<DeviceConcept> {
    use pr4xis::category::{Arrow, Category};
    DeviceCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == DeviceRelationKind::Parthood && m.source() == whole)
        .map(|m| m.target())
        .collect()
}

pub struct BTEContainsComponents;
impl Axiom for BTEContainsComponents {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use DeviceConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts = parts_of(BehindTheEar);
        if parts.contains(&Microphone) && parts.contains(&Amplifier) && parts.contains(&Receiver) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "BTEContainsComponents",
        "BTE hearing aid contains microphone, amplifier, and receiver",
        "Dillon (2012) Hearing Aids §5"
    );
}
pr4xis::register_axiom!(BTEContainsComponents, "Dillon (2012) Hearing Aids §5");

pub struct CIHighestGain;
impl Axiom for CIHighestGain {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use DeviceConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ci = MaxGainDB.get(&CochlearImplant).unwrap_or(0.0);
        let bte = MaxGainDB.get(&BehindTheEar).unwrap_or(0.0);
        if ci > bte {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "CIHighestGain",
        "cochlear implant provides highest effective gain",
        "Zeng et al. (2008) Cochlear Implants"
    );
}
pr4xis::register_axiom!(CIHighestGain, "Zeng et al. (2008) Cochlear Implants");

pub struct ImplantablesRequireSurgery;
impl Axiom for ImplantablesRequireSurgery {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use DeviceConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = [
            CochlearImplant,
            BoneAnchoredHearingAid,
            MiddleEarImplant,
            AuditoryBrainstemImplant,
        ]
        .iter()
        .all(|d| RequiresSurgery.get(d) == Some(true));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ImplantablesRequireSurgery",
        "all implantable devices require surgery",
        "Zeng et al. (2008) Cochlear Implants; Tjellstrom et al. (1981) Am. J. Otol. 2(4):304"
    );
}
pr4xis::register_axiom!(
    ImplantablesRequireSurgery,
    "Zeng et al. (2008) Cochlear Implants; Tjellstrom et al. (1981) Am. J. Otol. 2(4):304"
);

impl Ontology for DeviceOntology {
    type Cat = DeviceCategory;
    type Qual = MaxGainDB;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(BTEContainsComponents));
        a.push(Box::new(CIHighestGain));
        a.push(Box::new(ImplantablesRequireSurgery));
        a
    }
}

// Back-compat aliases used by sibling functors.
pub use DeviceConcept as DeviceEntity;
pub use DeviceRelationKind as DeviceCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<DeviceCategory>();
    }
    #[test]
    fn ontology_validates() {
        DeviceOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[test]
    fn bte_contains_components() {
        assert!(BTEContainsComponents.verify().is_ok());
    }
    #[test]
    fn ci_highest_gain() {
        assert!(CIHighestGain.verify().is_ok());
    }
    #[test]
    fn implantables_require_surgery() {
        assert!(ImplantablesRequireSurgery.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in DeviceCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in DeviceOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
}

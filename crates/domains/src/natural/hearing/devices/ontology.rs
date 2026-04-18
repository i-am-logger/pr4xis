//! Hearing devices ontology.
//!
//! Models hearing assistive technology and audiometric equipment.
//!
//! Key references:
//! - Dillon 2012: Hearing Aids (2nd ed.)
//! - Zeng et al. 2008: Cochlear Implants
//! - Tjellström et al. 1981: bone-anchored hearing aid
//! - Håkansson et al. 2010: BC hearing devices review
//! - Chasin 2006: Musicians and the Prevention of Hearing Loss

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::mereology;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum DeviceEntity {
    BehindTheEar,
    InTheEar,
    CompletelyInCanal,
    ReceiverInCanal,
    CROS,
    BiCROS,
    CochlearImplant,
    BoneAnchoredHearingAid,
    MiddleEarImplant,
    AuditoryBrainstemImplant,
    BoneConductionHeadphone,
    SoftbandBAHA,
    AdhesiveBC,
    DirectionalMicrophone,
    NoiseSuppression,
    FeedbackCancellation,
    FrequencyCompression,
    WideAdaptiveDynamicRange,
    Telecoil,
    BluetoothStreaming,
    Audiometer,
    Tympanometer,
    OAEProbe,
    ABRSystem,
    RealEarMeasurement,
    Microphone,
    Amplifier,
    Receiver,
    ElectrodeArray,
    SpeechProcessor,
    HearingAid,
    ImplantableDevice,
    BCDevice,
    SignalProcessingFeature,
    DiagnosticEquipment,
    DeviceComponent,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum DeviceCausalEvent {
    HearingLossDiagnosis,
    DeviceSelection,
    CustomMolding,
    InitialFitting,
    RealEarVerification,
    FineTuning,
    OutcomeImprovement,
}

define_ontology! {
    /// Discrete category over hearing device entities.
    pub DeviceOntology for DeviceCategory {
        entity: DeviceEntity, relation: DeviceRelation,
        being: PhysicalEndurant,
        source: "Dillon (2012); Zeng et al. (2008)",
        taxonomy: DeviceTaxonomy [
            (BehindTheEar, HearingAid), (InTheEar, HearingAid), (CompletelyInCanal, HearingAid), (ReceiverInCanal, HearingAid), (CROS, HearingAid), (BiCROS, HearingAid),
            (CochlearImplant, ImplantableDevice), (BoneAnchoredHearingAid, ImplantableDevice), (MiddleEarImplant, ImplantableDevice), (AuditoryBrainstemImplant, ImplantableDevice),
            (BoneConductionHeadphone, BCDevice), (SoftbandBAHA, BCDevice), (AdhesiveBC, BCDevice), (BoneAnchoredHearingAid, BCDevice),
            (DirectionalMicrophone, SignalProcessingFeature), (NoiseSuppression, SignalProcessingFeature), (FeedbackCancellation, SignalProcessingFeature), (FrequencyCompression, SignalProcessingFeature), (WideAdaptiveDynamicRange, SignalProcessingFeature), (Telecoil, SignalProcessingFeature), (BluetoothStreaming, SignalProcessingFeature),
            (Audiometer, DiagnosticEquipment), (Tympanometer, DiagnosticEquipment), (OAEProbe, DiagnosticEquipment), (ABRSystem, DiagnosticEquipment), (RealEarMeasurement, DiagnosticEquipment),
            (Microphone, DeviceComponent), (Amplifier, DeviceComponent), (Receiver, DeviceComponent), (ElectrodeArray, DeviceComponent), (SpeechProcessor, DeviceComponent),
        ],
        mereology: DeviceMereology [
            (BehindTheEar, Microphone), (BehindTheEar, Amplifier), (BehindTheEar, Receiver),
            (CochlearImplant, ElectrodeArray), (CochlearImplant, SpeechProcessor), (CochlearImplant, Microphone),
            (HearingAid, DirectionalMicrophone), (HearingAid, NoiseSuppression), (HearingAid, FeedbackCancellation),
        ],
        causation: DeviceCausalGraph for DeviceCausalEvent [
            (HearingLossDiagnosis, DeviceSelection), (DeviceSelection, CustomMolding), (CustomMolding, InitialFitting), (InitialFitting, RealEarVerification), (RealEarVerification, FineTuning), (FineTuning, OutcomeImprovement),
        ],
        opposition: DeviceOpposition [
            (CochlearImplant, HearingAid), (BehindTheEar, CompletelyInCanal), (DirectionalMicrophone, Telecoil),
        ],
    }
}

#[derive(Debug, Clone)]
pub struct MaxGainDB;
impl Quality for MaxGainDB {
    type Individual = DeviceEntity;
    type Value = f64;
    fn get(&self, individual: &DeviceEntity) -> Option<f64> {
        use DeviceEntity::*;
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
    type Individual = DeviceEntity;
    type Value = f64;
    fn get(&self, individual: &DeviceEntity) -> Option<f64> {
        use DeviceEntity::*;
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
    type Individual = DeviceEntity;
    type Value = bool;
    fn get(&self, individual: &DeviceEntity) -> Option<bool> {
        use DeviceEntity::*;
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

pub struct BTEContainsComponents;
impl Axiom for BTEContainsComponents {
    fn description(&self) -> &str {
        "BTE hearing aid contains microphone, amplifier, and receiver"
    }
    fn holds(&self) -> bool {
        use DeviceEntity::*;
        let parts = mereology::parts_of::<DeviceMereology>(&BehindTheEar);
        parts.contains(&Microphone) && parts.contains(&Amplifier) && parts.contains(&Receiver)
    }
}
pr4xis::register_axiom!(BTEContainsComponents);
pub struct CIHighestGain;
impl Axiom for CIHighestGain {
    fn description(&self) -> &str {
        "cochlear implant provides highest gain"
    }
    fn holds(&self) -> bool {
        use DeviceEntity::*;
        MaxGainDB.get(&CochlearImplant).unwrap() > MaxGainDB.get(&BehindTheEar).unwrap()
    }
}
pr4xis::register_axiom!(CIHighestGain);
pub struct ImplantablesRequireSurgery;
impl Axiom for ImplantablesRequireSurgery {
    fn description(&self) -> &str {
        "all implantable devices require surgery"
    }
    fn holds(&self) -> bool {
        use DeviceEntity::*;
        [
            CochlearImplant,
            BoneAnchoredHearingAid,
            MiddleEarImplant,
            AuditoryBrainstemImplant,
        ]
        .iter()
        .all(|d| RequiresSurgery.get(d) == Some(true))
    }
}
pr4xis::register_axiom!(ImplantablesRequireSurgery);
pub struct HearingAidsNoSurgery;
impl Axiom for HearingAidsNoSurgery {
    fn description(&self) -> &str {
        "conventional hearing aids do not require surgery"
    }
    fn holds(&self) -> bool {
        use DeviceEntity::*;
        [BehindTheEar, InTheEar, CompletelyInCanal, ReceiverInCanal]
            .iter()
            .all(|d| RequiresSurgery.get(d) == Some(false))
    }
}
pr4xis::register_axiom!(HearingAidsNoSurgery);
pub struct BAHADualClassification;
impl Axiom for BAHADualClassification {
    fn description(&self) -> &str {
        "BAHA is classified as both implantable and BC device"
    }
    fn holds(&self) -> bool {
        use DeviceEntity::*;
        taxonomy::is_a::<DeviceTaxonomy>(&BoneAnchoredHearingAid, &ImplantableDevice)
            && taxonomy::is_a::<DeviceTaxonomy>(&BoneAnchoredHearingAid, &BCDevice)
    }
}
pr4xis::register_axiom!(BAHADualClassification);
pub struct BTELongestBattery;
impl Axiom for BTELongestBattery {
    fn description(&self) -> &str {
        "BTE has longest battery life among compared devices"
    }
    fn holds(&self) -> bool {
        use DeviceEntity::*;
        let bte = BatteryLifeDays.get(&BehindTheEar).unwrap();
        let ci = BatteryLifeDays.get(&CochlearImplant).unwrap();
        let cic = BatteryLifeDays.get(&CompletelyInCanal).unwrap();
        bte > ci && bte > cic
    }
}
pr4xis::register_axiom!(BTELongestBattery);
pub struct DiagnosisCausesOutcome;
impl Axiom for DiagnosisCausesOutcome {
    fn description(&self) -> &str {
        "hearing loss diagnosis transitively causes outcome improvement"
    }
    fn holds(&self) -> bool {
        use DeviceCausalEvent::*;
        causation::effects_of::<DeviceCausalGraph>(&HearingLossDiagnosis)
            .contains(&OutcomeImprovement)
    }
}
pr4xis::register_axiom!(DiagnosisCausesOutcome);

impl Ontology for DeviceOntology {
    type Cat = DeviceCategory;
    type Qual = MaxGainDB;
    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }
    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(BTEContainsComponents),
            Box::new(CIHighestGain),
            Box::new(ImplantablesRequireSurgery),
            Box::new(HearingAidsNoSurgery),
            Box::new(BAHADualClassification),
            Box::new(BTELongestBattery),
            Box::new(DiagnosisCausesOutcome),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::mereology::MereologyCategory;
    use pr4xis::ontology::reasoning::opposition;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;
    #[test]
    fn test_bte_contains_components() {
        assert!(BTEContainsComponents.holds());
    }
    #[test]
    fn test_ci_highest_gain() {
        assert!(CIHighestGain.holds());
    }
    #[test]
    fn test_implantables_surgery() {
        assert!(ImplantablesRequireSurgery.holds());
    }
    #[test]
    fn test_ha_no_surgery() {
        assert!(HearingAidsNoSurgery.holds());
    }
    #[test]
    fn test_baha_dual() {
        assert!(BAHADualClassification.holds());
    }
    #[test]
    fn test_diagnosis_causes_outcome() {
        assert!(DiagnosisCausesOutcome.holds());
    }
    #[test]
    fn test_ci_opposes_hearing_aid() {
        assert!(opposition::are_opposed::<DeviceOpposition>(
            &DeviceEntity::CochlearImplant,
            &DeviceEntity::HearingAid
        ));
    }
    #[test]
    fn test_category_laws() {
        check_category_laws::<DeviceCategory>().unwrap();
    }
    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<DeviceTaxonomy>>().unwrap();
    }
    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<DeviceCausalGraph>>().unwrap();
    }
    #[test]
    fn test_mereology_category_laws() {
        check_category_laws::<MereologyCategory<DeviceMereology>>().unwrap();
    }
    #[test]
    fn test_bte_longest_battery() {
        assert!(BTELongestBattery.holds());
    }
    #[test]
    fn test_bte_battery_life() {
        assert_eq!(BatteryLifeDays.get(&DeviceEntity::BehindTheEar), Some(7.0));
    }
    #[test]
    fn test_ci_battery_life() {
        assert_eq!(
            BatteryLifeDays.get(&DeviceEntity::CochlearImplant),
            Some(1.0)
        );
    }
    #[test]
    fn test_entity_count() {
        assert_eq!(DeviceEntity::variants().len(), 36);
    }
    #[test]
    fn test_ontology_validates() {
        DeviceOntology::validate().unwrap();
    }
    fn arb_entity() -> impl Strategy<Value = DeviceEntity> {
        (0..DeviceEntity::variants().len()).prop_map(|i| DeviceEntity::variants()[i])
    }
    proptest! { #[test] fn prop_taxonomy_reflexive(entity in arb_entity()) { prop_assert!(taxonomy::is_a::<DeviceTaxonomy>(&entity, &entity)); } }
}

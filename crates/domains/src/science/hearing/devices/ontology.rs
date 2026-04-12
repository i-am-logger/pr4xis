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
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::mereology::{self, MereologyDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum DeviceEntity {
    // Hearing aids
    BehindTheEar,
    InTheEar,
    CompletelyInCanal,
    ReceiverInCanal,
    CROS,
    BiCROS,
    // Implantable devices
    CochlearImplant,
    BoneAnchoredHearingAid,
    MiddleEarImplant,
    AuditoryBrainstemImplant,
    // Non-surgical BC devices
    BoneConductionHeadphone,
    SoftbandBAHA,
    AdhesiveBC,
    // Signal processing features
    DirectionalMicrophone,
    NoiseSuppression,
    FeedbackCancellation,
    FrequencyCompression,
    WideAdaptiveDynamicRange,
    Telecoil,
    BluetoothStreaming,
    // Audiometric equipment
    Audiometer,
    Tympanometer,
    OAEProbe,
    ABRSystem,
    RealEarMeasurement,
    // Components
    Microphone,
    Amplifier,
    Receiver,
    ElectrodeArray,
    SpeechProcessor,
    // Abstract categories
    HearingAid,
    ImplantableDevice,
    BCDevice,
    SignalProcessingFeature,
    DiagnosticEquipment,
    DeviceComponent,
}

// ---------------------------------------------------------------------------
// Taxonomy
// ---------------------------------------------------------------------------

pub struct DeviceTaxonomy;

impl TaxonomyDef for DeviceTaxonomy {
    type Entity = DeviceEntity;

    fn relations() -> Vec<(DeviceEntity, DeviceEntity)> {
        use DeviceEntity::*;
        vec![
            // Hearing aid types
            (BehindTheEar, HearingAid),
            (InTheEar, HearingAid),
            (CompletelyInCanal, HearingAid),
            (ReceiverInCanal, HearingAid),
            (CROS, HearingAid),
            (BiCROS, HearingAid),
            // Implantable devices
            (CochlearImplant, ImplantableDevice),
            (BoneAnchoredHearingAid, ImplantableDevice),
            (MiddleEarImplant, ImplantableDevice),
            (AuditoryBrainstemImplant, ImplantableDevice),
            // BC devices
            (BoneConductionHeadphone, BCDevice),
            (SoftbandBAHA, BCDevice),
            (AdhesiveBC, BCDevice),
            (BoneAnchoredHearingAid, BCDevice),
            // Signal processing features
            (DirectionalMicrophone, SignalProcessingFeature),
            (NoiseSuppression, SignalProcessingFeature),
            (FeedbackCancellation, SignalProcessingFeature),
            (FrequencyCompression, SignalProcessingFeature),
            (WideAdaptiveDynamicRange, SignalProcessingFeature),
            (Telecoil, SignalProcessingFeature),
            (BluetoothStreaming, SignalProcessingFeature),
            // Diagnostic equipment
            (Audiometer, DiagnosticEquipment),
            (Tympanometer, DiagnosticEquipment),
            (OAEProbe, DiagnosticEquipment),
            (ABRSystem, DiagnosticEquipment),
            (RealEarMeasurement, DiagnosticEquipment),
            // Components
            (Microphone, DeviceComponent),
            (Amplifier, DeviceComponent),
            (Receiver, DeviceComponent),
            (ElectrodeArray, DeviceComponent),
            (SpeechProcessor, DeviceComponent),
        ]
    }
}

// ---------------------------------------------------------------------------
// Mereology (has-a / part-whole)
// ---------------------------------------------------------------------------

/// Part-whole relationships for hearing devices.
///
/// Devices are composed of their functional components:
/// - BTE hearing aids contain microphone, amplifier, and receiver
/// - Cochlear implants contain electrode array, speech processor, and microphone
/// - Hearing aids (abstract) contain signal processing features
///
/// Dillon 2012; Zeng et al. 2008.
pub struct DeviceMereology;

impl MereologyDef for DeviceMereology {
    type Entity = DeviceEntity;

    fn relations() -> Vec<(DeviceEntity, DeviceEntity)> {
        use DeviceEntity::*;
        vec![
            // BTE hearing aid components
            (BehindTheEar, Microphone),
            (BehindTheEar, Amplifier),
            (BehindTheEar, Receiver),
            // Cochlear implant components
            (CochlearImplant, ElectrodeArray),
            (CochlearImplant, SpeechProcessor),
            (CochlearImplant, Microphone),
            // Hearing aid signal processing features
            (HearingAid, DirectionalMicrophone),
            (HearingAid, NoiseSuppression),
            (HearingAid, FeedbackCancellation),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Causal events in the hearing device fitting pipeline.
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

/// Causal graph for the hearing device fitting pipeline.
pub struct DeviceCausalGraph;

impl CausalDef for DeviceCausalGraph {
    type Entity = DeviceCausalEvent;

    fn relations() -> Vec<(DeviceCausalEvent, DeviceCausalEvent)> {
        use DeviceCausalEvent::*;
        vec![
            // Diagnosis determines device selection
            (HearingLossDiagnosis, DeviceSelection),
            // Device selection leads to custom ear mold
            (DeviceSelection, CustomMolding),
            // Custom mold enables initial fitting
            (CustomMolding, InitialFitting),
            // Initial fitting verified with real-ear measurement
            (InitialFitting, RealEarVerification),
            // Verification guides fine-tuning adjustments
            (RealEarVerification, FineTuning),
            // Fine-tuning produces outcome improvement
            (FineTuning, OutcomeImprovement),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over hearing device entities.
    pub DeviceCategory {
        entity: DeviceEntity,
        relation: DeviceRelation,
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Maximum gain (dB) typically provided by device type.
///
/// Dillon 2012, Table 1.1.
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
            CochlearImplant => Some(120.0), // bypasses hair cells entirely
            BoneAnchoredHearingAid => Some(45.0),
            BoneConductionHeadphone => Some(30.0),
            _ => None,
        }
    }
}

/// Battery life in days for hearing devices.
///
/// - BehindTheEar: ~7 days (disposable zinc-air)
/// - CochlearImplant: ~1 day (rechargeable)
/// - CompletelyInCanal: ~5 days (smaller battery)
///
/// Dillon 2012, Ch. 2.
#[derive(Debug, Clone)]
pub struct BatteryLifeDays;

impl Quality for BatteryLifeDays {
    type Individual = DeviceEntity;
    type Value = f64;

    fn get(&self, individual: &DeviceEntity) -> Option<f64> {
        use DeviceEntity::*;
        match individual {
            BehindTheEar => Some(7.0),
            CochlearImplant => Some(1.0), // rechargeable, daily charge
            CompletelyInCanal => Some(5.0),
            _ => None,
        }
    }
}

/// Whether the device requires surgery.
#[derive(Debug, Clone)]
pub struct RequiresSurgery;

impl Quality for RequiresSurgery {
    type Individual = DeviceEntity;
    type Value = bool;

    fn get(&self, individual: &DeviceEntity) -> Option<bool> {
        use DeviceEntity::*;
        match individual {
            CochlearImplant => Some(true),
            BoneAnchoredHearingAid => Some(true),
            MiddleEarImplant => Some(true),
            AuditoryBrainstemImplant => Some(true),
            BehindTheEar | InTheEar | CompletelyInCanal | ReceiverInCanal => Some(false),
            BoneConductionHeadphone | SoftbandBAHA | AdhesiveBC => Some(false),
            CROS | BiCROS => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in hearing devices.
///
/// - CochlearImplant vs HearingAid: bypass vs amplify (different treatment categories)
/// - BehindTheEar vs CompletelyInCanal: largest vs smallest form factor
/// - DirectionalMicrophone vs Telecoil: spatial vs electromagnetic input
pub struct DeviceOpposition;

impl OppositionDef for DeviceOpposition {
    type Entity = DeviceEntity;

    fn pairs() -> Vec<(DeviceEntity, DeviceEntity)> {
        use DeviceEntity::*;
        vec![
            (CochlearImplant, HearingAid),
            (BehindTheEar, CompletelyInCanal),
            (DirectionalMicrophone, Telecoil),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

pub struct DeviceTaxonomyIsDAG;
impl Axiom for DeviceTaxonomyIsDAG {
    fn description(&self) -> &str {
        "device taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        taxonomy::NoCycles::<DeviceTaxonomy>::new().holds()
    }
}

/// Mereology is a DAG.
pub struct DeviceMereologyIsDAG;
impl Axiom for DeviceMereologyIsDAG {
    fn description(&self) -> &str {
        "device mereology is a DAG"
    }
    fn holds(&self) -> bool {
        mereology::NoCycles::<DeviceMereology>::new().holds()
    }
}

/// BTE hearing aid contains microphone, amplifier, and receiver.
///
/// Dillon 2012, Ch. 1.
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

/// CI provides highest gain (bypasses damaged hair cells).
pub struct CIHighestGain;
impl Axiom for CIHighestGain {
    fn description(&self) -> &str {
        "cochlear implant provides highest gain"
    }
    fn holds(&self) -> bool {
        use DeviceEntity::*;
        let ci = MaxGainDB.get(&CochlearImplant).unwrap();
        let bte = MaxGainDB.get(&BehindTheEar).unwrap();
        ci > bte
    }
}

/// All implantable devices require surgery.
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

/// No conventional hearing aid requires surgery.
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

/// BAHA is both implantable AND a BC device.
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

/// BTE has longest battery life among compared devices (Dillon 2012).
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

/// Causal graph is asymmetric.
pub struct DeviceCausalGraphIsAsymmetric;
impl Axiom for DeviceCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "device causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<DeviceCausalGraph>::new().holds()
    }
}

/// No event causes itself.
pub struct DeviceCausalGraphNoSelfCause;
impl Axiom for DeviceCausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no device fitting event causes itself"
    }
    fn holds(&self) -> bool {
        causation::NoSelfCausation::<DeviceCausalGraph>::new().holds()
    }
}

/// Opposition is symmetric.
pub struct DeviceOppositionSymmetric;
impl Axiom for DeviceOppositionSymmetric {
    fn description(&self) -> &str {
        "device opposition is symmetric"
    }
    fn holds(&self) -> bool {
        opposition::Symmetric::<DeviceOpposition>::new().holds()
    }
}

/// Opposition is irreflexive.
pub struct DeviceOppositionIrreflexive;
impl Axiom for DeviceOppositionIrreflexive {
    fn description(&self) -> &str {
        "device opposition is irreflexive"
    }
    fn holds(&self) -> bool {
        opposition::Irreflexive::<DeviceOpposition>::new().holds()
    }
}

/// Diagnosis transitively causes outcome improvement.
pub struct DiagnosisCausesOutcome;
impl Axiom for DiagnosisCausesOutcome {
    fn description(&self) -> &str {
        "hearing loss diagnosis transitively causes outcome improvement"
    }
    fn holds(&self) -> bool {
        use DeviceCausalEvent::*;
        let effects = causation::effects_of::<DeviceCausalGraph>(&HearingLossDiagnosis);
        effects.contains(&OutcomeImprovement)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

pub struct DeviceOntology;

impl Ontology for DeviceOntology {
    type Cat = DeviceCategory;
    type Qual = MaxGainDB;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(DeviceTaxonomyIsDAG),
            Box::new(DeviceMereologyIsDAG),
            Box::new(BTEContainsComponents),
            Box::new(CIHighestGain),
            Box::new(ImplantablesRequireSurgery),
            Box::new(HearingAidsNoSurgery),
            Box::new(BAHADualClassification),
            Box::new(BTELongestBattery),
            Box::new(DeviceCausalGraphIsAsymmetric),
            Box::new(DeviceCausalGraphNoSelfCause),
            Box::new(DiagnosisCausesOutcome),
            Box::new(DeviceOppositionSymmetric),
            Box::new(DeviceOppositionIrreflexive),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::mereology::MereologyCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(DeviceTaxonomyIsDAG.holds());
    }
    #[test]
    fn test_mereology_is_dag() {
        assert!(DeviceMereologyIsDAG.holds());
    }
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
    fn test_causal_graph_asymmetric() {
        assert!(DeviceCausalGraphIsAsymmetric.holds());
    }
    #[test]
    fn test_causal_graph_no_self_cause() {
        assert!(DeviceCausalGraphNoSelfCause.holds());
    }
    #[test]
    fn test_diagnosis_causes_outcome() {
        assert!(DiagnosisCausesOutcome.holds());
    }
    #[test]
    fn test_opposition_symmetric() {
        assert!(DeviceOppositionSymmetric.holds());
    }
    #[test]
    fn test_opposition_irreflexive() {
        assert!(DeviceOppositionIrreflexive.holds());
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
    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<DeviceTaxonomy>(&entity, &entity));
        }
    }
}

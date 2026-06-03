//! Functor: PathologyCategory → DeviceCategory.
//!
//! Maps hearing disorders to their treatment devices.
//!
//! Citation: Dillon (2012) *Hearing Aids* 2nd ed.; Zeng et al. (2008)
//! *Cochlear Implants*; Tjellstrom et al. (1981) *Am. J. Otol.* 2(4):304
//! — the disorder→device fitting decisions are clinical-textbook content.

use crate::natural::hearing::devices::ontology::*;
use crate::natural::hearing::pathology::ontology::*;
use pr4xis::category::{Arrow, Functor};

pub struct PathologyToDevices;

impl Functor for PathologyToDevices {
    type Source = PathologyCategory;
    type Target = DeviceCategory;

    fn map_object(obj: &PathologyEntity) -> DeviceEntity {
        use DeviceEntity::*;
        use PathologyEntity as P;
        match obj {
            P::ConductiveHearingLoss
            | P::Otosclerosis
            | P::OtitisMedia
            | P::TympanicPerforation
            | P::Cholesteatoma
            | P::OssicularFixation => BoneAnchoredHearingAid,
            P::SensorineuralHearingLoss
            | P::Presbycusis
            | P::NoiseInducedHearingLoss
            | P::SuddenSensorineuralLoss
            | P::HairCellLoss
            | P::StereociliaDamage
            | P::OxidativeStress
            | P::Excitotoxicity
            | P::StriaDysfunction => BehindTheEar,
            P::MixedHearingLoss => BehindTheEar,
            P::AuditoryNeuropathy | P::DemyelinationVIII => CochlearImplant,
            P::CentralAuditoryProcessingDisorder => NoiseSuppression,
            P::MenieresDisease | P::EndolymphaticHydrops => BehindTheEar,
            P::AcousticNeuroma => AuditoryBrainstemImplant,
            P::Tinnitus | P::PhantomPercept => NoiseSuppression,
            P::Hyperacusis => FrequencyCompression,
            P::SynapticRibbonLoss => CochlearImplant,
            P::ElevatedThreshold => BehindTheEar,
            P::ReducedFrequencySelectivity => FrequencyCompression,
            P::LoudnessRecruitment => WideAdaptiveDynamicRange,
            P::PoorSpeechInNoise => DirectionalMicrophone,
            P::ReducedTemporalResolution => CochlearImplant,
            P::AbnormalBinauralProcessing => CROS,
            P::Audiogram | P::PureToneAverage | P::SpeechReceptionThreshold => Audiometer,
            P::OtoacousticEmission => OAEProbe,
            P::AuditoryBrainstemResponse => ABRSystem,
            P::HearingLoss => HearingAid,
            P::PeripheralPathology => HearingAid,
            P::CentralPathology => HearingAid,
            P::DamageMechanism => HearingAid,
            P::PerceptualDeficit => SignalProcessingFeature,
            P::ClinicalMeasure => DiagnosticEquipment,
            // Pathology events → device events.
            P::NoiseExposure | P::AgingDegeneration => HearingLossDiagnosis,
            P::Infection | P::Autoimmune | P::GeneticMutation => HearingLossDiagnosis,
            P::OHCDamage | P::IHCDamage | P::SynapseLoss => DeviceSelection,
            P::StriDegeneration | P::MiddleEarDysfunction | P::NeuralDegeneration => CustomMolding,
            P::ThresholdShift => InitialFitting,
            P::FrequencyResolutionLoss => FineTuning,
            P::TemporalSmearing => RealEarVerificationEvent,
            P::TinnitusGeneration => DeviceSelection,
            P::CommunicationDifficulty => OutcomeImprovement,
            P::PathologyEvent => DeviceEvent,
        }
    }

    fn map_morphism(m: &PathologyRelation) -> DeviceRelation {
        use DeviceRelationKind as Tk;
        use PathologyCategoryRelationKind as Sk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
            Sk::Parthood => Tk::Parthood,
        };
        DeviceRelation { from, to, kind }
    }
}
pr4xis::register_functor!(PathologyToDevices);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[test]
    fn functor_laws() {
        assert_functor_laws::<PathologyToDevices>();
    }
    #[test]
    fn conductive_maps_to_baha() {
        assert_eq!(
            PathologyToDevices::map_object(&PathologyEntity::ConductiveHearingLoss),
            DeviceEntity::BoneAnchoredHearingAid
        );
    }
    #[test]
    fn neuropathy_maps_to_ci() {
        assert_eq!(
            PathologyToDevices::map_object(&PathologyEntity::AuditoryNeuropathy),
            DeviceEntity::CochlearImplant
        );
    }
    #[test]
    fn every_entity_maps_valid() {
        let targets = DeviceEntity::variants();
        for obj in PathologyEntity::variants() {
            assert!(targets.contains(&PathologyToDevices::map_object(&obj)));
        }
    }
}

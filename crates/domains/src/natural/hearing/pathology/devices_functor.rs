//! Functor: PathologyCategory -> DeviceCategory.
//!
//! Maps hearing disorders to their treatment devices.

use crate::natural::hearing::devices::ontology::*;
use crate::natural::hearing::pathology::ontology::*;
use pr4xis::category::{Category, Functor, Relationship};

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
        }
    }

    fn map_morphism(m: &PathologyRelation) -> DeviceRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind {
            PathologyCategoryRelationKind::Identity => DeviceCategory::identity(&from),
            _ => DeviceRelation {
                from,
                to,
                kind: DeviceCategoryRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(PathologyToDevices);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<PathologyToDevices>().unwrap();
    }
    #[test]
    fn test_analogy_validates() {
        Analogy::<PathologyToDevices>::validate().unwrap();
    }
    #[test]
    fn test_conductive_maps_to_baha() {
        assert_eq!(
            PathologyToDevices::map_object(&PathologyEntity::ConductiveHearingLoss),
            DeviceEntity::BoneAnchoredHearingAid
        );
    }
    #[test]
    fn test_neuropathy_maps_to_ci() {
        assert_eq!(
            PathologyToDevices::map_object(&PathologyEntity::AuditoryNeuropathy),
            DeviceEntity::CochlearImplant
        );
    }
    #[test]
    fn test_every_entity_maps_valid() {
        let targets = DeviceEntity::variants();
        for obj in PathologyEntity::variants() {
            assert!(targets.contains(&PathologyToDevices::map_object(&obj)));
        }
    }

    use pr4xis::category::Category;
    use proptest::prelude::*;

    fn arb_pathology_entity() -> impl Strategy<Value = PathologyEntity> {
        (0..PathologyEntity::variants().len()).prop_map(|i| PathologyEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_pathology_entity()) {
            let mapped = PathologyToDevices::map_object(&entity);
            prop_assert!(DeviceEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_pathology_entity()) {
            let id_src = PathologyCategory::identity(&entity);
            let mapped_id = PathologyToDevices::map_morphism(&id_src);
            let id_tgt = DeviceCategory::identity(&PathologyToDevices::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}

//! Functor: PathologyCategory -> AudiologyCategory.
//!
//! Maps hearing disorders to the clinical tests used to diagnose them.

use crate::science::hearing::audiology::ontology::*;
use crate::science::hearing::pathology::ontology::*;
use praxis::category::{Functor, Relationship};

pub struct PathologyToAudiology;

impl Functor for PathologyToAudiology {
    type Source = PathologyCategory;
    type Target = AudiologyCategory;

    fn map_object(obj: &PathologyEntity) -> AudiologyEntity {
        use AudiologyEntity::*;
        use PathologyEntity as P;
        match obj {
            P::ConductiveHearingLoss | P::OssicularFixation | P::Otosclerosis => AirBoneGap,
            P::SensorineuralHearingLoss
            | P::Presbycusis
            | P::NoiseInducedHearingLoss
            | P::SuddenSensorineuralLoss
            | P::HairCellLoss
            | P::StereociliaDamage
            | P::OxidativeStress
            | P::Excitotoxicity
            | P::StriaDysfunction => PureToneAudiometry,
            P::MixedHearingLoss => PureToneAudiometry,
            P::AuditoryNeuropathy | P::DemyelinationVIII => AuditoryBrainstemResponse,
            P::CentralAuditoryProcessingDisorder => SpeechInNoiseTest,
            P::MenieresDisease | P::EndolymphaticHydrops => ElectroCochleography,
            P::AcousticNeuroma => AuditoryBrainstemResponse,
            P::Tinnitus | P::PhantomPercept | P::Hyperacusis => PureToneAudiometry,
            P::SynapticRibbonLoss => AuditoryBrainstemResponse,
            P::OtitisMedia | P::TympanicPerforation | P::Cholesteatoma => Tympanometry,
            P::ElevatedThreshold => PureToneAverage,
            P::ReducedFrequencySelectivity | P::LoudnessRecruitment => PureToneAudiometry,
            P::PoorSpeechInNoise => QuickSIN,
            P::ReducedTemporalResolution => SpeechInNoiseTest,
            P::AbnormalBinauralProcessing => SpeechInNoiseTest,
            P::Audiogram => PureToneAudiometry,
            P::PureToneAverage => PureToneAverage,
            P::SpeechReceptionThreshold => SpeechRecognitionThreshold,
            P::OtoacousticEmission => DistortionProductOAE,
            P::AuditoryBrainstemResponse => AuditoryBrainstemResponse,
            P::HearingLoss | P::PeripheralPathology | P::CentralPathology => DiagnosticTest,
            P::DamageMechanism => DiagnosticTest,
            P::PerceptualDeficit => SpeechTest,
            P::ClinicalMeasure => DiagnosticTest,
        }
    }

    fn map_morphism(m: &PathologyRelation) -> AudiologyRelation {
        AudiologyRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::Entity;
    use praxis::category::validate::check_functor_laws;
    use praxis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<PathologyToAudiology>().unwrap();
    }
    #[test]
    fn test_analogy_validates() {
        Analogy::<PathologyToAudiology>::validate().unwrap();
    }
    #[test]
    fn test_conductive_maps_to_air_bone_gap() {
        assert_eq!(
            PathologyToAudiology::map_object(&PathologyEntity::ConductiveHearingLoss),
            AudiologyEntity::AirBoneGap
        );
    }
    #[test]
    fn test_every_entity_maps_valid() {
        let targets = AudiologyEntity::variants();
        for obj in PathologyEntity::variants() {
            assert!(targets.contains(&PathologyToAudiology::map_object(&obj)));
        }
    }

    use praxis::category::Category;
    use proptest::prelude::*;

    fn arb_pathology_entity() -> impl Strategy<Value = PathologyEntity> {
        (0..PathologyEntity::variants().len()).prop_map(|i| PathologyEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_pathology_entity()) {
            let mapped = PathologyToAudiology::map_object(&entity);
            prop_assert!(AudiologyEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_pathology_entity()) {
            let id_src = PathologyCategory::identity(&entity);
            let mapped_id = PathologyToAudiology::map_morphism(&id_src);
            let id_tgt = AudiologyCategory::identity(&PathologyToAudiology::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}

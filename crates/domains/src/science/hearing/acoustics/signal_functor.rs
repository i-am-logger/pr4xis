//! Functor: AcousticsCategory -> SignalProcessingCategory.
//!
//! Maps acoustic physics to computational analysis tools.

use crate::science::hearing::acoustics::ontology::*;
use crate::science::hearing::signal_processing::ontology::*;
use praxis::category::{Functor, Relationship};

pub struct AcousticsToSignalProcessing;

impl Functor for AcousticsToSignalProcessing {
    type Source = AcousticsCategory;
    type Target = SignalProcessingCategory;

    fn map_object(obj: &AcousticEntity) -> SignalEntity {
        use AcousticEntity as A;
        use SignalEntity::*;
        match obj {
            A::Frequency | A::WaveProperty => FrequencyDomain,
            A::Amplitude | A::Intensity => PowerSpectralDensity,
            A::Wavelength => FourierTransform,
            A::Phase => HilbertTransform,
            A::SoundWave | A::LongitudinalWave | A::Wave => TimeDomain,
            A::TransverseWave | A::ShearWave => TimeDomain,
            A::Air | A::Water | A::SoftTissue | A::Cartilage | A::Fluid | A::Medium => Sampling,
            A::CorticalBone | A::CancellousBone | A::Solid | A::BoneTissue => Sampling,
            A::Resonance => BandPassFilter,
            A::Reflection => Autocorrelation,
            A::Refraction => WindowFunction,
            A::Diffraction => WaveletTransform,
            A::Absorption => LowPassFilter,
            A::Attenuation => LowPassFilter,
            A::ImpedanceMismatch => HighPassFilter,
            A::AcousticPhenomenon => Transform,
        }
    }

    fn map_morphism(m: &AcousticRelation) -> SignalRelation {
        SignalRelation {
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
        check_functor_laws::<AcousticsToSignalProcessing>().unwrap();
    }
    #[test]
    fn test_analogy_validates() {
        Analogy::<AcousticsToSignalProcessing>::validate().unwrap();
    }
    #[test]
    fn test_frequency_maps_to_freq_domain() {
        assert_eq!(
            AcousticsToSignalProcessing::map_object(&AcousticEntity::Frequency),
            SignalEntity::FrequencyDomain
        );
    }
    #[test]
    fn test_every_entity_maps_valid() {
        let targets = SignalEntity::variants();
        for obj in AcousticEntity::variants() {
            assert!(targets.contains(&AcousticsToSignalProcessing::map_object(&obj)));
        }
    }

    use praxis::category::Category;
    use proptest::prelude::*;

    fn arb_acoustic_entity() -> impl Strategy<Value = AcousticEntity> {
        (0..AcousticEntity::variants().len()).prop_map(|i| AcousticEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_acoustic_entity()) {
            let mapped = AcousticsToSignalProcessing::map_object(&entity);
            prop_assert!(SignalEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_acoustic_entity()) {
            let id_src = AcousticsCategory::identity(&entity);
            let mapped_id = AcousticsToSignalProcessing::map_morphism(&id_src);
            let id_tgt = SignalProcessingCategory::identity(&AcousticsToSignalProcessing::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}

//! Functor: AcousticsCategory -> BiophysicsCategory.
//!
//! Proves that acoustics has a structure-preserving map into biophysics.
//! Each acoustic entity maps to its biophysical substrate:
//! SoundWave -> MechanicalWave, AcousticPressure -> MechanicalStress,
//! BoneConduction -> BoneMatrix, Air -> FluidMedium, etc.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Functor, Relationship};

use crate::natural::biomedical::acoustics::ontology::{
    AcousticsCategory, AcousticsEntity, AcousticsRelation,
};
use crate::natural::biomedical::biophysics::ontology::{
    BiophysicsCategory, BiophysicsEntity, BiophysicsRelation,
};

/// Structure-preserving map from acoustics entities to their biophysical substrate.
pub struct AcousticsToBiophysics;

impl Functor for AcousticsToBiophysics {
    type Source = AcousticsCategory;
    type Target = BiophysicsCategory;

    fn map_object(obj: &AcousticsEntity) -> BiophysicsEntity {
        use AcousticsEntity as A;
        use BiophysicsEntity as BP;
        match obj {
            // Wave properties -> biophysics wave/mechanical
            A::SoundWave => BP::MechanicalWave,
            A::AcousticPressure => BP::MechanicalStress,
            A::AcousticIntensity => BP::MechanicalStress,
            A::AcousticFrequency => BP::Frequency,
            A::AcousticWavelength => BP::Wavelength,
            A::AcousticAmplitude => BP::MechanicalStress,
            A::Waveform => BP::MechanicalWave,

            // Impedance -> biophysics acoustic impedance
            A::AcousticImpedance => BP::AcousticImpedance,
            A::ImpedanceMismatch => BP::AcousticImpedance,
            A::ReflectionCoefficient => BP::AcousticImpedance,
            A::TransmissionCoefficient => BP::AcousticImpedance,

            // Conduction paths -> biophysics media
            A::BoneConduction => BP::BoneMatrix,
            A::AirConduction => BP::FluidMedium,
            A::SoftTissueConduction => BP::SoftTissue,

            // Transducers -> mechanical wave (they generate waves)
            A::ElectroacousticTransducer => BP::MechanicalWave,
            A::PiezoelectricTransducer => BP::MechanicalWave,
            A::ElectromagneticTransducer => BP::MechanicalWave,

            // Media -> biophysics media
            A::Air => BP::FluidMedium,
            A::Bone => BP::BoneMatrix,
            A::SoftTissue => BP::SoftTissue,
            A::Fluid => BP::FluidMedium,

            // Abstract categories
            A::WaveProperty => BP::WaveProperty,
            A::ImpedanceProperty => BP::WaveProperty,
            A::ConductionPath => BP::BiologicalMedium,
            A::TransducerType => BP::MechanicalProperty,
            A::AcousticMedium => BP::BiologicalMedium,
        }
    }

    fn map_morphism(m: &AcousticsRelation) -> BiophysicsRelation {
        BiophysicsRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}
pr4xis::register_functor!(AcousticsToBiophysics);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<AcousticsToBiophysics>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<AcousticsToBiophysics>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in AcousticsEntity::variants() {
            let id_src = AcousticsCategory::identity(&obj);
            let mapped_id = AcousticsToBiophysics::map_morphism(&id_src);
            let id_tgt = BiophysicsCategory::identity(&AcousticsToBiophysics::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        let objs = AcousticsEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = AcousticsRelation { from: a, to: b };
                    let g = AcousticsRelation { from: b, to: c };
                    let composed = AcousticsCategory::compose(&f, &g).unwrap();
                    let mapped_composed = AcousticsToBiophysics::map_morphism(&composed);
                    let composed_mapped = BiophysicsCategory::compose(
                        &AcousticsToBiophysics::map_morphism(&f),
                        &AcousticsToBiophysics::map_morphism(&g),
                    )
                    .unwrap();
                    assert_eq!(
                        mapped_composed, composed_mapped,
                        "composition law failed for {:?} -> {:?} -> {:?}",
                        a, b, c
                    );
                }
            }
        }
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = BiophysicsEntity::variants();
        for obj in AcousticsEntity::variants() {
            let mapped = AcousticsToBiophysics::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BiophysicsEntity",
                obj,
                mapped
            );
        }
    }

    // -- Specific mapping tests --

    #[test]
    fn test_sound_wave_maps_to_mechanical_wave() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsEntity::SoundWave),
            BiophysicsEntity::MechanicalWave,
        );
    }

    #[test]
    fn test_acoustic_pressure_maps_to_mechanical_stress() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsEntity::AcousticPressure),
            BiophysicsEntity::MechanicalStress,
        );
    }

    #[test]
    fn test_bone_conduction_maps_to_bone_matrix() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsEntity::BoneConduction),
            BiophysicsEntity::BoneMatrix,
        );
    }

    #[test]
    fn test_air_conduction_maps_to_fluid_medium() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsEntity::AirConduction),
            BiophysicsEntity::FluidMedium,
        );
    }

    #[test]
    fn test_air_maps_to_fluid_medium() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsEntity::Air),
            BiophysicsEntity::FluidMedium,
        );
    }

    #[test]
    fn test_bone_maps_to_bone_matrix() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsEntity::Bone),
            BiophysicsEntity::BoneMatrix,
        );
    }

    #[test]
    fn test_piezoelectric_transducer_maps_to_mechanical_wave() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsEntity::PiezoelectricTransducer),
            BiophysicsEntity::MechanicalWave,
        );
    }

    #[test]
    fn test_acoustic_frequency_maps_to_frequency() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsEntity::AcousticFrequency),
            BiophysicsEntity::Frequency,
        );
    }

    #[test]
    fn test_conduction_path_maps_to_biological_medium() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsEntity::ConductionPath),
            BiophysicsEntity::BiologicalMedium,
        );
    }
}

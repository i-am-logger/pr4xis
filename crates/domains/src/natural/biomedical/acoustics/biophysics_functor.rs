//! Functor: AcousticsCategory -> BiophysicsCategory.
//!
//! Proves that acoustics has a structure-preserving map into biophysics.
//! Each acoustic entity maps to its biophysical substrate:
//! SoundWave -> MechanicalWave, AcousticPressure -> MechanicalStress,
//! BoneConduction -> BoneMatrix, Air -> FluidMedium, etc.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Arrow, Category, Functor};

use crate::natural::biomedical::acoustics::ontology::{
    AcousticsCategory, AcousticsConcept, AcousticsRelation, AcousticsRelationKind,
};
use crate::natural::biomedical::biophysics::ontology::{
    BiophysicsCategory, BiophysicsConcept as BiophysicsEntity, BiophysicsRelation,
    BiophysicsRelationKind,
};

/// Structure-preserving map from acoustics entities to their biophysical substrate.
pub struct AcousticsToBiophysics;

impl Functor for AcousticsToBiophysics {
    type Source = AcousticsCategory;
    type Target = BiophysicsCategory;

    fn map_object(obj: &AcousticsConcept) -> BiophysicsEntity {
        use AcousticsConcept as A;
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

            // Abstract umbrellas and the AcousticEvent root
            A::WaveProperty => BP::WaveProperty,
            A::ImpedanceProperty => BP::WaveProperty,
            A::ConductionPath => BP::BiologicalMedium,
            A::TransducerType => BP::MechanicalProperty,
            A::AcousticMedium => BP::BiologicalMedium,
            A::AcousticEvent => BP::MechanicalWave,

            // Causal events — merged into the concept enum.
            // Each event maps to the biophysical substrate that carries it.
            A::ElectricalSignalInput => BP::MechanicalStress,
            A::TransducerActivation => BP::MechanicalWave,
            A::SurfaceOscillation => BP::MechanicalWave,
            A::AcousticWaveGeneration => BP::MechanicalWave,
            A::MediumPropagation => BP::MechanicalWave,
            A::ImpedanceBoundary => BP::AcousticImpedance,
            A::PartialReflection => BP::AcousticImpedance,
            A::PartialTransmission => BP::AcousticImpedance,
            A::BoneCoupledTransmission => BP::BoneMatrix,
            A::DeepTissuePenetration => BP::SoftTissue,
        }
    }

    fn map_morphism(m: &AcousticsRelation) -> BiophysicsRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Preserve source's Identity → target's identity; everything else
        // maps to Composed in the (dense) Biophysics target so that
        // F(g∘f) == F(g)∘F(f) holds under collapse.
        // Identity preserved; other source kinds collapse to Subsumption in
        // the (kinded, partial) Biophysics target. Same-kind preservation
        // keeps F(g∘f) = F(g)∘F(f) under #166 (no Composed kind).
        match m.kind {
            AcousticsRelationKind::Identity => BiophysicsCategory::identity(&from),
            _ => BiophysicsRelation {
                from,
                to,
                kind: BiophysicsRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(AcousticsToBiophysics);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        assert_functor_laws::<AcousticsToBiophysics>();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<AcousticsToBiophysics>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in AcousticsConcept::variants() {
            let id_src = AcousticsCategory::identity(&obj);
            let mapped_id = AcousticsToBiophysics::map_morphism(&id_src);
            let id_tgt = BiophysicsCategory::identity(&AcousticsToBiophysics::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation_on_subsumption() {
        // The migrated Acoustics category is kinded and partial (per OBO-RO,
        // #166): compose only succeeds for same-kind transitive relations.
        // Exercise composition along Subsumption chains and verify that the
        // functor preserves the composite.
        for m in AcousticsCategory::morphisms() {
            if m.kind() != AcousticsRelationKind::Subsumption {
                continue;
            }
            for n in AcousticsCategory::morphisms() {
                if n.kind() != AcousticsRelationKind::Subsumption {
                    continue;
                }
                if m.target() != n.source() {
                    continue;
                }
                let composed = match AcousticsCategory::compose(&m, &n) {
                    Some(c) => c,
                    None => continue,
                };
                let mapped_composed = AcousticsToBiophysics::map_morphism(&composed);
                let composed_mapped = BiophysicsCategory::compose(
                    &AcousticsToBiophysics::map_morphism(&m),
                    &AcousticsToBiophysics::map_morphism(&n),
                )
                .expect("target composition is total");
                assert_eq!(
                    mapped_composed, composed_mapped,
                    "composition law failed for {:?} ∘ {:?}",
                    m, n
                );
            }
        }
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = BiophysicsEntity::variants();
        for obj in AcousticsConcept::variants() {
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
            AcousticsToBiophysics::map_object(&AcousticsConcept::SoundWave),
            BiophysicsEntity::MechanicalWave,
        );
    }

    #[test]
    fn test_acoustic_pressure_maps_to_mechanical_stress() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsConcept::AcousticPressure),
            BiophysicsEntity::MechanicalStress,
        );
    }

    #[test]
    fn test_bone_conduction_maps_to_bone_matrix() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsConcept::BoneConduction),
            BiophysicsEntity::BoneMatrix,
        );
    }

    #[test]
    fn test_air_conduction_maps_to_fluid_medium() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsConcept::AirConduction),
            BiophysicsEntity::FluidMedium,
        );
    }

    #[test]
    fn test_air_maps_to_fluid_medium() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsConcept::Air),
            BiophysicsEntity::FluidMedium,
        );
    }

    #[test]
    fn test_bone_maps_to_bone_matrix() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsConcept::Bone),
            BiophysicsEntity::BoneMatrix,
        );
    }

    #[test]
    fn test_piezoelectric_transducer_maps_to_mechanical_wave() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsConcept::PiezoelectricTransducer),
            BiophysicsEntity::MechanicalWave,
        );
    }

    #[test]
    fn test_acoustic_frequency_maps_to_frequency() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsConcept::AcousticFrequency),
            BiophysicsEntity::Frequency,
        );
    }

    #[test]
    fn test_conduction_path_maps_to_biological_medium() {
        assert_eq!(
            AcousticsToBiophysics::map_object(&AcousticsConcept::ConductionPath),
            BiophysicsEntity::BiologicalMedium,
        );
    }
}

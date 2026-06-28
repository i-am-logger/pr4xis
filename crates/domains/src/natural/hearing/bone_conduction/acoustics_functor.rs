//! Functor: AcousticsCategory → BoneConductionCategory.
//!
//! Maps acoustic entities to their bone-conduction role. Sound waves
//! become skull vibrations, media become application contexts, phenomena
//! become BC-specific effects.
//!
//! Citation: Stenfelt & Goode (2005) *Otology & Neurotology* 26(6):1245 —
//! the canonical BC review grounds these correspondences.

use pr4xis::category::{Arrow, Functor};

use crate::natural::hearing::acoustics::ontology::{
    AcousticEntity, AcousticRelation, AcousticsCategory, AcousticsCategoryRelationKind,
};
use crate::natural::hearing::bone_conduction::ontology::{
    BoneCondEntity, BoneCondRelation, BoneCondRelationKind, BoneConductionCategory,
};

pub struct AcousticsToBoneConduction;

impl Functor for AcousticsToBoneConduction {
    type Source = AcousticsCategory;
    type Target = BoneConductionCategory;

    fn map_object(obj: &AcousticEntity) -> BoneCondEntity {
        use AcousticEntity as A;
        use BoneCondEntity::*;
        match obj {
            A::SoundWave | A::LongitudinalWave | A::TransverseWave | A::ShearWave | A::Wave => {
                SkullVibration
            }
            A::Frequency
            | A::Amplitude
            | A::Wavelength
            | A::Phase
            | A::Intensity
            | A::WaveProperty => ForceLevel,
            A::CorticalBone => Mastoid,
            A::CancellousBone => TemporalBone,
            A::Air
            | A::Water
            | A::SoftTissue
            | A::Cartilage
            | A::Fluid
            | A::Medium
            | A::Solid
            | A::BoneTissue => SkinDriveTransducer,
            A::Resonance => SkullResonance,
            A::ImpedanceMismatch => TranscranialAttenuation,
            A::Reflection | A::Refraction | A::Diffraction => SkullResonance,
            A::Absorption | A::Attenuation => TranscranialAttenuation,
            A::AcousticPhenomenon => BCPhenomenon,
            // Acoustic events → BC events.
            A::SourceVibration => TransducerActivation,
            A::MediumCoupling => SkullCoupling,
            A::WavePropagation => SkullWavePropagation,
            A::BoundaryEncounter | A::ImpedanceTransition => OssicularLag,
            A::EnergyReflection => InnerEarDistortion,
            A::EnergyTransmission => OvalWindowDrive,
            A::EnergyAbsorption => CochlearBoneCompression,
            A::WaveAttenuation => DifferentialFluidFlow,
            A::ResonantAmplification => BasilarMembraneExcitation,
            A::ReceiverExcitation => CochlearResponse,
            A::AcousticEvent => BCEvent,
        }
    }

    fn map_morphism(m: &AcousticRelation) -> BoneCondRelation {
        use AcousticsCategoryRelationKind as Sk;
        use BoneCondRelationKind as Tk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
            // Canonical kinds always emitted; unreachable when source has no
            // edges of this kind (acoustics has Parthood; bone has too).
            Sk::Parthood => Tk::Parthood,
        };
        BoneCondRelation { from, to, kind }
    }
}
pr4xis::register_functor!(AcousticsToBoneConduction);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Category, FinitelyGenerated};

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws() {
        assert_functor_laws::<AcousticsToBoneConduction>();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn identity_preservation() {
        for obj in AcousticEntity::variants() {
            let id_src = AcousticsCategory::identity(&obj);
            let mapped_id = AcousticsToBoneConduction::map_morphism(&id_src);
            let id_tgt =
                BoneConductionCategory::identity(&AcousticsToBoneConduction::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn sound_wave_maps_to_skull_vibration() {
        assert_eq!(
            AcousticsToBoneConduction::map_object(&AcousticEntity::SoundWave),
            BoneCondEntity::SkullVibration
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn cortical_bone_maps_to_mastoid() {
        assert_eq!(
            AcousticsToBoneConduction::map_object(&AcousticEntity::CorticalBone),
            BoneCondEntity::Mastoid
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn every_entity_maps_valid() {
        let targets = BoneCondEntity::variants();
        for obj in AcousticEntity::variants() {
            assert!(targets.contains(&AcousticsToBoneConduction::map_object(&obj)));
        }
    }
}

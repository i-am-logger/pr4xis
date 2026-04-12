//! Functor: TransductionCategory -> PsychoacousticsCategory.
//!
//! Maps molecular/cellular transduction entities to their perceptual role.
//! Hair cell mechanics become perceived sound qualities.

use pr4xis::category::{Functor, Relationship};

use crate::natural::hearing::psychoacoustics::ontology::{
    PsychoacousticEntity, PsychoacousticRelation, PsychoacousticsCategory,
};
use crate::natural::hearing::transduction::ontology::{
    TransductionCategory, TransductionEntity, TransductionRelation,
};

/// Structure-preserving map from transduction entities to perceptual role.
pub struct TransductionToPsychoacoustics;

impl Functor for TransductionToPsychoacoustics {
    type Source = TransductionCategory;
    type Target = PsychoacousticsCategory;

    fn map_object(obj: &TransductionEntity) -> PsychoacousticEntity {
        use PsychoacousticEntity::*;
        use TransductionEntity as T;
        match obj {
            // Stereocilia and mechanical → frequency selectivity (place code → pitch)
            T::Stereocilium
            | T::StereociliaBundle
            | T::CuticularPlate
            | T::Kinocilium
            | T::StereociliaDeflection => FrequencySelectivity,
            // Tip links → auditory filter mechanics
            T::TipLink | T::Cadherin23 | T::Protocadherin15 | T::TipLinkTension => AuditoryFilter,
            // MET channel complex → critical band (frequency resolution)
            T::METChannel
            | T::TMC1
            | T::TMC2
            | T::TMIE
            | T::LHFPL5
            | T::METChannelOpening
            | T::METComponent => CriticalBand,
            // Ion influx → loudness (intensity coding)
            T::PotassiumInflux | T::Potassium => Loudness,
            T::CalciumInflux | T::Calcium => Loudness,
            // Depolarization / receptor potential → loudness coding
            T::Depolarization | T::ReceptorPotential | T::EndocochlearPotential => Loudness,
            // Glutamate release → temporal resolution (synaptic timing)
            T::GlutamateRelease | T::Glutamate => TemporalResolution,
            // Action potential → pitch perception (temporal coding)
            T::ActionPotential => Pitch,
            // OHC electromotility → cochlear amplifier → frequency selectivity
            T::Prestin | T::Electromotility | T::CochlearAmplification => FrequencySelectivity,
            // Ion channels → loudness (gain control)
            T::KCNQ4 | T::CaV1_3 | T::BKChannel | T::IonChannel => Loudness,
            // Abstract
            T::CellularSignal | T::TipLinkProtein => AuditoryFilter,
        }
    }

    fn map_morphism(m: &TransductionRelation) -> PsychoacousticRelation {
        PsychoacousticRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<TransductionToPsychoacoustics>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<TransductionToPsychoacoustics>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in TransductionEntity::variants() {
            let id_src = TransductionCategory::identity(&obj);
            let mapped_id = TransductionToPsychoacoustics::map_morphism(&id_src);
            let id_tgt =
                PsychoacousticsCategory::identity(&TransductionToPsychoacoustics::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_action_potential_maps_to_pitch() {
        assert_eq!(
            TransductionToPsychoacoustics::map_object(&TransductionEntity::ActionPotential),
            PsychoacousticEntity::Pitch,
        );
    }

    #[test]
    fn test_potassium_influx_maps_to_loudness() {
        assert_eq!(
            TransductionToPsychoacoustics::map_object(&TransductionEntity::PotassiumInflux),
            PsychoacousticEntity::Loudness,
        );
    }

    #[test]
    fn test_prestin_maps_to_frequency_selectivity() {
        assert_eq!(
            TransductionToPsychoacoustics::map_object(&TransductionEntity::Prestin),
            PsychoacousticEntity::FrequencySelectivity,
        );
    }

    #[test]
    fn test_met_channel_maps_to_critical_band() {
        assert_eq!(
            TransductionToPsychoacoustics::map_object(&TransductionEntity::METChannel),
            PsychoacousticEntity::CriticalBand,
        );
    }

    #[test]
    fn test_glutamate_release_maps_to_temporal_resolution() {
        assert_eq!(
            TransductionToPsychoacoustics::map_object(&TransductionEntity::GlutamateRelease),
            PsychoacousticEntity::TemporalResolution,
        );
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = PsychoacousticEntity::variants();
        for obj in TransductionEntity::variants() {
            let mapped = TransductionToPsychoacoustics::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not valid",
                obj,
                mapped
            );
        }
    }

    use proptest::prelude::*;

    fn arb_transduction_entity() -> impl Strategy<Value = TransductionEntity> {
        (0..TransductionEntity::variants().len()).prop_map(|i| TransductionEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_transduction_entity()) {
            let mapped = TransductionToPsychoacoustics::map_object(&entity);
            prop_assert!(PsychoacousticEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_transduction_entity()) {
            let id_src = TransductionCategory::identity(&entity);
            let mapped_id = TransductionToPsychoacoustics::map_morphism(&id_src);
            let id_tgt = PsychoacousticsCategory::identity(&TransductionToPsychoacoustics::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}

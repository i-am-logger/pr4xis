//! Functor: TransductionCategory -> NeuroscienceCategory.
//!
//! Maps hair cell transduction events to their neural processing roles.

use crate::science::hearing::auditory_neuroscience::ontology::*;
use crate::science::hearing::transduction::ontology::*;
use praxis::category::{Functor, Relationship};

pub struct TransductionToNeuroscience;

impl Functor for TransductionToNeuroscience {
    type Source = TransductionCategory;
    type Target = NeuroscienceCategory;

    fn map_object(obj: &TransductionEntity) -> NeuralEntity {
        use NeuralEntity::*;
        use TransductionEntity as T;
        match obj {
            // Action potential → auditory nerve fiber
            T::ActionPotential => AuditoryNerveFiber,
            // Glutamate release → spike timing (rate coding)
            T::GlutamateRelease | T::Glutamate => RateCoding,
            // Depolarization / receptor potential → rate-level function
            T::Depolarization | T::ReceptorPotential => RateLevelFunction,
            // MET channel → frequency tuning curve
            T::METChannel
            | T::TMC1
            | T::TMC2
            | T::TMIE
            | T::LHFPL5
            | T::METChannelOpening
            | T::METComponent => FrequencyTuningCurve,
            // Stereocilia → place coding (tonotopic position)
            T::Stereocilium
            | T::StereociliaBundle
            | T::CuticularPlate
            | T::Kinocilium
            | T::StereociliaDeflection => PlaceCoding,
            // Tip links → characteristic frequency
            T::TipLink
            | T::Cadherin23
            | T::Protocadherin15
            | T::TipLinkTension
            | T::TipLinkProtein => CharacteristicFrequency,
            // OHC amplification → dynamic range
            T::Prestin | T::Electromotility | T::CochlearAmplification => DynamicRange,
            // Ion channels / currents → spontaneous rate
            T::PotassiumInflux
            | T::CalciumInflux
            | T::Potassium
            | T::Calcium
            | T::KCNQ4
            | T::CaV1_3
            | T::BKChannel
            | T::IonChannel => SpontaneousRate,
            // Endocochlear potential → adaptation
            T::EndocochlearPotential => Adaptation,
            // Cellular signal → response property
            T::CellularSignal => OnsetResponse,
        }
    }

    fn map_morphism(m: &TransductionRelation) -> NeuralRelation {
        NeuralRelation {
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
        check_functor_laws::<TransductionToNeuroscience>().unwrap();
    }
    #[test]
    fn test_analogy_validates() {
        Analogy::<TransductionToNeuroscience>::validate().unwrap();
    }

    #[test]
    fn test_action_potential_maps_to_an_fiber() {
        assert_eq!(
            TransductionToNeuroscience::map_object(&TransductionEntity::ActionPotential),
            NeuralEntity::AuditoryNerveFiber,
        );
    }

    #[test]
    fn test_every_entity_maps_valid() {
        let targets = NeuralEntity::variants();
        for obj in TransductionEntity::variants() {
            assert!(targets.contains(&TransductionToNeuroscience::map_object(&obj)));
        }
    }

    use praxis::category::Category;
    use proptest::prelude::*;

    fn arb_transduction_entity() -> impl Strategy<Value = TransductionEntity> {
        (0..TransductionEntity::variants().len()).prop_map(|i| TransductionEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_transduction_entity()) {
            let mapped = TransductionToNeuroscience::map_object(&entity);
            prop_assert!(NeuralEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_transduction_entity()) {
            let id_src = TransductionCategory::identity(&entity);
            let mapped_id = TransductionToNeuroscience::map_morphism(&id_src);
            let id_tgt = NeuroscienceCategory::identity(&TransductionToNeuroscience::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}

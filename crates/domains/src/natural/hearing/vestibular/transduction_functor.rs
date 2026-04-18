//! Functor: TransductionCategory -> VestibularCategory.
//!
//! Maps hair cell transduction to vestibular function — same molecular
//! machinery, different sensory modality.

use crate::natural::hearing::transduction::ontology::*;
use crate::natural::hearing::vestibular::ontology::*;
use pr4xis::category::{Category, Functor, Relationship};

pub struct TransductionToVestibular;

impl Functor for TransductionToVestibular {
    type Source = TransductionCategory;
    type Target = VestibularCategory;

    fn map_object(obj: &TransductionEntity) -> VestibularEntity {
        use TransductionEntity as T;
        use VestibularEntity::*;
        match obj {
            T::Stereocilium
            | T::StereociliaBundle
            | T::CuticularPlate
            | T::Kinocilium
            | T::StereociliaDeflection => CrisaAmpullaris,
            T::TipLink
            | T::Cadherin23
            | T::Protocadherin15
            | T::TipLinkTension
            | T::TipLinkProtein => TypeIHairCell,
            T::METChannel
            | T::TMC1
            | T::TMC2
            | T::TMIE
            | T::LHFPL5
            | T::METChannelOpening
            | T::METComponent => TypeIHairCell,
            T::PotassiumInflux
            | T::Potassium
            | T::CalciumInflux
            | T::Calcium
            | T::Depolarization
            | T::ReceptorPotential
            | T::EndocochlearPotential => TypeIHairCell,
            T::KCNQ4 | T::CaV1_3 | T::BKChannel | T::IonChannel => TypeIIHairCell,
            T::GlutamateRelease | T::Glutamate => ScarpaGanglion,
            T::ActionPotential => VestibularNerve,
            T::Prestin | T::Electromotility | T::CochlearAmplification => Cupula, // no prestin analog in vestibular
            T::CellularSignal => VestibularNuclei,
        }
    }

    fn map_morphism(m: &TransductionRelation) -> VestibularRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind {
            TransductionCategoryRelationKind::Identity => VestibularCategory::identity(&from),
            _ => VestibularRelation {
                from,
                to,
                kind: VestibularCategoryRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(TransductionToVestibular);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<TransductionToVestibular>().unwrap();
    }
    #[test]
    fn test_analogy_validates() {
        Analogy::<TransductionToVestibular>::validate().unwrap();
    }

    #[test]
    fn test_action_potential_maps_to_nerve() {
        assert_eq!(
            TransductionToVestibular::map_object(&TransductionEntity::ActionPotential),
            VestibularEntity::VestibularNerve,
        );
    }

    #[test]
    fn test_every_entity_maps_valid() {
        let targets = VestibularEntity::variants();
        for obj in TransductionEntity::variants() {
            assert!(targets.contains(&TransductionToVestibular::map_object(&obj)));
        }
    }

    use pr4xis::category::Category;
    use proptest::prelude::*;

    fn arb_transduction_entity() -> impl Strategy<Value = TransductionEntity> {
        (0..TransductionEntity::variants().len()).prop_map(|i| TransductionEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_transduction_entity()) {
            let mapped = TransductionToVestibular::map_object(&entity);
            prop_assert!(VestibularEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_transduction_entity()) {
            let id_src = TransductionCategory::identity(&entity);
            let mapped_id = TransductionToVestibular::map_morphism(&id_src);
            let id_tgt = VestibularCategory::identity(&TransductionToVestibular::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}

//! Functor: AnatomyCategory -> VestibularCategory.
//!
//! Maps auditory anatomy to vestibular structures (shared inner ear).

use crate::natural::hearing::anatomy::ontology::*;
use crate::natural::hearing::vestibular::ontology::*;
use pr4xis::category::{Category, Functor, Relationship};

pub struct AnatomyToVestibular;

impl Functor for AnatomyToVestibular {
    type Source = AnatomyCategory;
    type Target = VestibularCategory;

    fn map_object(obj: &AuditoryEntity) -> VestibularEntity {
        use AuditoryEntity as A;
        use VestibularEntity::*;
        match obj {
            // Vestibular structures map directly
            A::Vestibule => Utricle,
            A::SemicircularCanals => LateralCanal,
            // Inner ear shared structures
            A::Endolymph => Cupula,
            A::Perilymph => OtolithMembrane,
            // Hair cells → vestibular hair cells
            A::InnerHairCell => TypeIHairCell,
            A::OuterHairCell => TypeIIHairCell,
            A::SupportingCell => CrisaAmpullaris,
            // Neural pathway
            A::SpiralGanglionNeuron => ScarpaGanglion,
            A::AuditoryNerve => VestibularNerve,
            A::CochlearNucleus
            | A::SuperiorOlivaryComplex
            | A::InferiorColliculus
            | A::MedialGeniculateBody => VestibularNuclei,
            A::AuditoryCortex => CerebellumVestibular,
            // Cochlea → otolith (both fluid-filled sensory organs)
            A::Cochlea | A::BasilarMembrane | A::OrganOfCorti | A::TectorialMembrane => Macula,
            A::ScalaVestibuli | A::ScalaMedia | A::ScalaTympani => Ampulla,
            A::StriVascularis | A::ReissnersMembrane => StriolarRegion,
            // Outer/middle ear → no vestibular analog, map to generic
            A::Pinna | A::EarCanal | A::TympanicMembrane => Cupula,
            A::Malleus
            | A::Incus
            | A::Stapes
            | A::OvalWindow
            | A::RoundWindow
            | A::EustachianTube
            | A::TensorTympani
            | A::Stapedius => Otoconia,
            // Abstract
            A::Ear
            | A::OuterEar
            | A::MiddleEar
            | A::InnerEar
            | A::Ossicle
            | A::HairCell
            | A::CochlearFluid
            | A::CochlearMembrane
            | A::AuditoryNucleus => VestibularNuclei,
        }
    }

    fn map_morphism(m: &AuditoryRelation) -> VestibularRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind {
            AnatomyCategoryRelationKind::Identity => VestibularCategory::identity(&from),
            _ => VestibularRelation {
                from,
                to,
                kind: VestibularCategoryRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(AnatomyToVestibular);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<AnatomyToVestibular>().unwrap();
    }
    #[test]
    fn test_analogy_validates() {
        Analogy::<AnatomyToVestibular>::validate().unwrap();
    }
    #[test]
    fn test_ihc_maps_to_type_i() {
        assert_eq!(
            AnatomyToVestibular::map_object(&AuditoryEntity::InnerHairCell),
            VestibularEntity::TypeIHairCell
        );
    }
    #[test]
    fn test_every_entity_maps_valid() {
        let targets = VestibularEntity::variants();
        for obj in AuditoryEntity::variants() {
            assert!(targets.contains(&AnatomyToVestibular::map_object(&obj)));
        }
    }

    use pr4xis::category::Category;
    use proptest::prelude::*;

    fn arb_auditory_entity() -> impl Strategy<Value = AuditoryEntity> {
        (0..AuditoryEntity::variants().len()).prop_map(|i| AuditoryEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_auditory_entity()) {
            let mapped = AnatomyToVestibular::map_object(&entity);
            prop_assert!(VestibularEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_auditory_entity()) {
            let id_src = AnatomyCategory::identity(&entity);
            let mapped_id = AnatomyToVestibular::map_morphism(&id_src);
            let id_tgt = VestibularCategory::identity(&AnatomyToVestibular::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}

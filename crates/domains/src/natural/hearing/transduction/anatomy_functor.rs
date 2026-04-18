//! Functor: AnatomyCategory -> TransductionCategory.
//!
//! Maps anatomical structures to their functional roles in mechanotransduction.
//! The cochlea's anatomy becomes the molecular machinery of sound-to-nerve conversion.

use pr4xis::category::{Functor, Relationship};

use crate::natural::hearing::anatomy::ontology::{
    AnatomyCategory, AuditoryEntity, AuditoryRelation,
};
use crate::natural::hearing::transduction::ontology::{
    TransductionCategory, TransductionEntity, TransductionRelation,
};

/// Structure-preserving map from auditory anatomy to transduction role.
pub struct AnatomyToTransduction;

impl Functor for AnatomyToTransduction {
    type Source = AnatomyCategory;
    type Target = TransductionCategory;

    fn map_object(obj: &AuditoryEntity) -> TransductionEntity {
        use AuditoryEntity as A;
        use TransductionEntity::*;
        match obj {
            // Hair cells → their transduction role
            A::InnerHairCell => GlutamateRelease, // IHC primary output
            A::OuterHairCell => Electromotility,  // OHC primary output
            // Cochlear membranes → mechanical events
            A::BasilarMembrane => StereociliaDeflection, // BM motion drives stereocilia
            A::TectorialMembrane => TipLinkTension,      // TM shear drives tip links
            A::OrganOfCorti => METChannel,               // organ = transduction apparatus
            // Fluids → driving potentials
            A::Endolymph => EndocochlearPotential, // endolymph carries EP
            A::Perilymph => Potassium,             // perilymph is the return path
            A::ScalaMedia => EndocochlearPotential, // scala media contains endolymph
            A::StriVascularis => EndocochlearPotential, // stria generates EP
            // Neural
            A::SpiralGanglionNeuron => ActionPotential,
            A::AuditoryNerve => ActionPotential,
            A::CochlearNucleus => ActionPotential,
            A::SuperiorOlivaryComplex => ActionPotential,
            A::InferiorColliculus => ActionPotential,
            A::MedialGeniculateBody => ActionPotential,
            A::AuditoryCortex => ActionPotential,
            // Cochlea overall → the stereocilia bundle (site of transduction)
            A::Cochlea => StereociliaBundle,
            // Ossicular chain → mechanical input
            A::Stapes | A::OvalWindow => StereociliaDeflection,
            A::Malleus | A::Incus => StereociliaDeflection,
            // All other structures → stereocilia bundle (generic mechanical)
            A::Pinna
            | A::EarCanal
            | A::TympanicMembrane
            | A::RoundWindow
            | A::EustachianTube
            | A::TensorTympani
            | A::Stapedius
            | A::ScalaVestibuli
            | A::ScalaTympani
            | A::ReissnersMembrane
            | A::Vestibule
            | A::SemicircularCanals
            | A::SupportingCell => StereociliaBundle,
            // Abstract categories
            A::Ear
            | A::OuterEar
            | A::MiddleEar
            | A::InnerEar
            | A::Ossicle
            | A::HairCell
            | A::CochlearFluid
            | A::CochlearMembrane
            | A::AuditoryNucleus => StereociliaBundle,
        }
    }

    fn map_morphism(m: &AuditoryRelation) -> TransductionRelation {
        TransductionRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}
pr4xis::register_functor!(AnatomyToTransduction);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<AnatomyToTransduction>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<AnatomyToTransduction>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in AuditoryEntity::variants() {
            let id_src = AnatomyCategory::identity(&obj);
            let mapped_id = AnatomyToTransduction::map_morphism(&id_src);
            let id_tgt = TransductionCategory::identity(&AnatomyToTransduction::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_ihc_maps_to_glutamate_release() {
        assert_eq!(
            AnatomyToTransduction::map_object(&AuditoryEntity::InnerHairCell),
            TransductionEntity::GlutamateRelease,
        );
    }

    #[test]
    fn test_ohc_maps_to_electromotility() {
        assert_eq!(
            AnatomyToTransduction::map_object(&AuditoryEntity::OuterHairCell),
            TransductionEntity::Electromotility,
        );
    }

    #[test]
    fn test_endolymph_maps_to_ep() {
        assert_eq!(
            AnatomyToTransduction::map_object(&AuditoryEntity::Endolymph),
            TransductionEntity::EndocochlearPotential,
        );
    }

    #[test]
    fn test_auditory_nerve_maps_to_action_potential() {
        assert_eq!(
            AnatomyToTransduction::map_object(&AuditoryEntity::AuditoryNerve),
            TransductionEntity::ActionPotential,
        );
    }

    #[test]
    fn test_basilar_membrane_maps_to_stereocilia_deflection() {
        assert_eq!(
            AnatomyToTransduction::map_object(&AuditoryEntity::BasilarMembrane),
            TransductionEntity::StereociliaDeflection,
        );
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = TransductionEntity::variants();
        for obj in AuditoryEntity::variants() {
            let mapped = AnatomyToTransduction::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid TransductionEntity",
                obj,
                mapped
            );
        }
    }

    use proptest::prelude::*;

    fn arb_auditory_entity() -> impl Strategy<Value = AuditoryEntity> {
        (0..AuditoryEntity::variants().len()).prop_map(|i| AuditoryEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_auditory_entity()) {
            let mapped = AnatomyToTransduction::map_object(&entity);
            prop_assert!(TransductionEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_auditory_entity()) {
            let id_src = AnatomyCategory::identity(&entity);
            let mapped_id = AnatomyToTransduction::map_morphism(&id_src);
            let id_tgt = TransductionCategory::identity(&AnatomyToTransduction::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}

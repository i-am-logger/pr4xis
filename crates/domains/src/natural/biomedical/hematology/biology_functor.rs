//! Functor: HematologyCategory -> BiologyCategory.
//!
//! Proves that the hematology domain has a structure-preserving map into
//! biological organization. Blood is a connective tissue; blood cells map
//! to Cell; plasma proteins map to Fibroblast (the primary protein-producing
//! cell); electrolytes and properties map to Cell or Tissue.

use pr4xis::category::{Arrow, Functor};

use crate::natural::biomedical::biology::ontology::{
    BiologicalEntity, BiologicalRelation, BiologyCategory, BiologyRelationKind,
};
use crate::natural::biomedical::hematology::ontology::{
    HematologyCategory, HematologyEntity, HematologyRelation, HematologyRelationKind,
};

/// Structure-preserving map from hematology entities to biological organization.
pub struct HematologyToBiology;

impl Functor for HematologyToBiology {
    type Source = HematologyCategory;
    type Target = BiologyCategory;

    fn map_object(obj: &HematologyEntity) -> BiologicalEntity {
        use BiologicalEntity as B;
        use HematologyEntity as H;
        match obj {
            // Blood cells -> Cell
            H::RedBloodCell => B::Cell,
            H::WhiteBloodCell => B::Cell,
            H::Platelet => B::Cell,

            // Blood plasma and whole blood -> ConnectiveTissue
            // (blood is a connective tissue)
            H::BloodPlasma => B::ConnectiveTissue,
            H::WholeBlood => B::ConnectiveTissue,
            H::Serum => B::ConnectiveTissue,

            // Plasma proteins -> Fibroblast (protein producers)
            H::Albumin => B::Fibroblast,
            H::Globulin => B::Fibroblast,
            H::Fibrinogen => B::Fibroblast,
            H::Immunoglobulin => B::Fibroblast,

            // Electrolytes -> Cell (intracellular/extracellular ion balance)
            H::SodiumPlasma => B::Cell,
            H::PotassiumPlasma => B::Cell,
            H::CalciumPlasma => B::Cell,
            H::ChloridePlasma => B::Cell,
            H::BicarbonatePlasma => B::Cell,

            // Properties -> Tissue (tissue-level measurements)
            H::OsmoticPressure => B::Tissue,
            H::OncoticPressure => B::Tissue,
            H::BloodPH => B::Tissue,
            H::Hematocrit => B::Tissue,
            H::Viscosity => B::Tissue,

            // Abstract categories
            H::BloodComponent => B::ConnectiveTissue,
            H::PlasmaProtein => B::Cell,
            H::PlasmaElectrolyte => B::Cell,
            H::BloodProperty => B::Tissue,

            // Events (merged into the source concept enum per
            // `feedback_one_ontology_per_module`) — all hematology
            // events map to ConnectiveTissue (blood is a connective
            // tissue, where every hematology event occurs).
            H::HematologyEvent
            | H::Hemorrhage
            | H::PlasmaVolumeLoss
            | H::ElectrolyteImbalance
            | H::Inflammation
            | H::AcutePhaseResponse
            | H::AlbuminDecrease
            | H::AcidBaseDisturbance
            | H::BicarbonateBuffering
            | H::PHCorrection
            | H::CoagulationCascade
            | H::FibrinFormation => B::ConnectiveTissue,
        }
    }

    fn map_morphism(m: &HematologyRelation) -> BiologicalRelation {
        use BiologyRelationKind as Tk;
        use HematologyRelationKind as Sk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        BiologicalRelation { from, to, kind }
    }
}
pr4xis::register_functor!(HematologyToBiology);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    /// Daubert prong 2 — verify identity + composition preservation.
    #[test]
    fn functor_laws() {
        assert_functor_laws::<HematologyToBiology>();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<HematologyToBiology>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in HematologyEntity::variants() {
            let id_src = HematologyCategory::identity(&obj);
            let mapped_id = HematologyToBiology::map_morphism(&id_src);
            let id_tgt = BiologyCategory::identity(&HematologyToBiology::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }
    // NOTE: test_composition_preservation removed — pending the final
    // adjunctions/composition_tests batch (the source is now a kinded
    // partial category per OBO-RO; `Composed` no longer exists).

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = BiologicalEntity::variants();
        for obj in HematologyEntity::variants() {
            let mapped = HematologyToBiology::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BiologicalEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_rbc_maps_to_cell() {
        assert_eq!(
            HematologyToBiology::map_object(&HematologyEntity::RedBloodCell),
            BiologicalEntity::Cell,
        );
    }

    #[test]
    fn test_wbc_maps_to_cell() {
        assert_eq!(
            HematologyToBiology::map_object(&HematologyEntity::WhiteBloodCell),
            BiologicalEntity::Cell,
        );
    }

    #[test]
    fn test_platelet_maps_to_cell() {
        assert_eq!(
            HematologyToBiology::map_object(&HematologyEntity::Platelet),
            BiologicalEntity::Cell,
        );
    }

    #[test]
    fn test_blood_plasma_maps_to_connective_tissue() {
        assert_eq!(
            HematologyToBiology::map_object(&HematologyEntity::BloodPlasma),
            BiologicalEntity::ConnectiveTissue,
        );
    }

    #[test]
    fn test_whole_blood_maps_to_connective_tissue() {
        assert_eq!(
            HematologyToBiology::map_object(&HematologyEntity::WholeBlood),
            BiologicalEntity::ConnectiveTissue,
        );
    }

    #[test]
    fn test_albumin_maps_to_fibroblast() {
        assert_eq!(
            HematologyToBiology::map_object(&HematologyEntity::Albumin),
            BiologicalEntity::Fibroblast,
        );
    }

    #[test]
    fn test_sodium_maps_to_cell() {
        assert_eq!(
            HematologyToBiology::map_object(&HematologyEntity::SodiumPlasma),
            BiologicalEntity::Cell,
        );
    }

    #[test]
    fn test_osmotic_pressure_maps_to_tissue() {
        assert_eq!(
            HematologyToBiology::map_object(&HematologyEntity::OsmoticPressure),
            BiologicalEntity::Tissue,
        );
    }

    #[test]
    fn test_blood_component_abstract_maps_to_connective_tissue() {
        assert_eq!(
            HematologyToBiology::map_object(&HematologyEntity::BloodComponent),
            BiologicalEntity::ConnectiveTissue,
        );
    }
}

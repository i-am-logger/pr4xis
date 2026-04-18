//! Functor: HematologyCategory -> BiologyCategory.
//!
//! Proves that the hematology domain has a structure-preserving map into
//! biological organization. Blood is a connective tissue; blood cells map
//! to Cell; plasma proteins map to Fibroblast (the primary protein-producing
//! cell); electrolytes and properties map to Cell or Tissue.

use pr4xis::category::{Category, Functor, Relationship};

use crate::natural::biomedical::biology::ontology::{
    BiologicalEntity, BiologicalRelation, BiologyCategory, BiologyCategoryRelationKind,
};
use crate::natural::biomedical::hematology::ontology::{
    HematologyCategory, HematologyCategoryRelationKind, HematologyEntity, HematologyRelation,
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
        }
    }

    fn map_morphism(m: &HematologyRelation) -> BiologicalRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind {
            HematologyCategoryRelationKind::Identity => BiologyCategory::identity(&from),
            _ => BiologicalRelation {
                from,
                to,
                kind: BiologyCategoryRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(HematologyToBiology);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<HematologyToBiology>().unwrap();
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

    #[test]
    fn test_composition_preservation() {
        let objs = HematologyEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = HematologyRelation {
                        from: a,
                        to: b,
                        kind: HematologyCategoryRelationKind::Composed,
                    };
                    let g = HematologyRelation {
                        from: b,
                        to: c,
                        kind: HematologyCategoryRelationKind::Composed,
                    };
                    let composed = HematologyCategory::compose(&f, &g).unwrap();
                    let mapped_composed = HematologyToBiology::map_morphism(&composed);
                    let composed_mapped = BiologyCategory::compose(
                        &HematologyToBiology::map_morphism(&f),
                        &HematologyToBiology::map_morphism(&g),
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

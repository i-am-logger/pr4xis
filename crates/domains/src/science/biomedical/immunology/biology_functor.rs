//! Functor: ImmunologyCategory -> BiologyCategory.
//!
//! Proves that the immunology domain has a structure-preserving map into
//! biological organization. Immune cells map to their biological counterparts:
//! MacrophageM1/M2 map directly (same entities in both ontologies), other
//! immune cells abstract to Cell, inflammatory states map to Tissue (tissue-level
//! phenomena), and cytokines map to Cell (produced by cells).
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use praxis::category::{Functor, Relationship};

use crate::science::biomedical::biology::ontology::{
    BiologicalEntity, BiologicalRelation, BiologyCategory,
};
use crate::science::biomedical::immunology::ontology::{
    ImmunologyCategory, ImmunologyEntity, ImmunologyRelation,
};

/// Structure-preserving map from immunology entities to biological organization.
pub struct ImmunologyToBiology;

impl Functor for ImmunologyToBiology {
    type Source = ImmunologyCategory;
    type Target = BiologyCategory;

    fn map_object(obj: &ImmunologyEntity) -> BiologicalEntity {
        use BiologicalEntity as B;
        use ImmunologyEntity as I;
        match obj {
            // Direct mappings -- same entities in both ontologies
            I::MacrophageM1 => B::MacrophageM1,
            I::MacrophageM2 => B::MacrophageM2,
            I::Fibroblast => B::Fibroblast,

            // Abstract immune cells -> Cell
            I::Neutrophil | I::TCell | I::Monocyte | I::MastCell => B::Cell,

            // Inflammatory states are tissue-level phenomena
            I::AcuteInflammation | I::ChronicInflammation => B::Tissue,
            I::Resolution | I::TissueRepair => B::Tissue,
            I::Fibrosis => B::Tissue,

            // Cytokines are produced by and act on cells
            I::ProInflammatoryCytokine | I::AntiInflammatoryCytokine => B::Cell,
            I::TNFAlpha | I::IL6 | I::IL10 | I::TGFBeta => B::Cell,

            // Abstract categories
            I::ImmuneCell => B::Cell,
            I::StromalCell => B::Cell,
            I::InflammatoryState => B::Tissue,
            I::Cytokine => B::Cell,
        }
    }

    fn map_morphism(m: &ImmunologyRelation) -> BiologicalRelation {
        BiologicalRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::validate::check_functor_laws;
    use praxis::category::{Category, Entity};
    use praxis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<ImmunologyToBiology>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<ImmunologyToBiology>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in ImmunologyEntity::variants() {
            let id_src = ImmunologyCategory::identity(&obj);
            let mapped_id = ImmunologyToBiology::map_morphism(&id_src);
            let id_tgt = BiologyCategory::identity(&ImmunologyToBiology::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        let objs = ImmunologyEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = ImmunologyRelation { from: a, to: b };
                    let g = ImmunologyRelation { from: b, to: c };
                    let composed = ImmunologyCategory::compose(&f, &g).unwrap();
                    let mapped_composed = ImmunologyToBiology::map_morphism(&composed);
                    let composed_mapped = BiologyCategory::compose(
                        &ImmunologyToBiology::map_morphism(&f),
                        &ImmunologyToBiology::map_morphism(&g),
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
        for obj in ImmunologyEntity::variants() {
            let mapped = ImmunologyToBiology::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BiologicalEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_macrophage_m1_maps_directly() {
        assert_eq!(
            ImmunologyToBiology::map_object(&ImmunologyEntity::MacrophageM1),
            BiologicalEntity::MacrophageM1,
        );
    }

    #[test]
    fn test_macrophage_m2_maps_directly() {
        assert_eq!(
            ImmunologyToBiology::map_object(&ImmunologyEntity::MacrophageM2),
            BiologicalEntity::MacrophageM2,
        );
    }

    #[test]
    fn test_fibroblast_maps_directly() {
        assert_eq!(
            ImmunologyToBiology::map_object(&ImmunologyEntity::Fibroblast),
            BiologicalEntity::Fibroblast,
        );
    }

    #[test]
    fn test_neutrophil_maps_to_cell() {
        assert_eq!(
            ImmunologyToBiology::map_object(&ImmunologyEntity::Neutrophil),
            BiologicalEntity::Cell,
        );
    }

    #[test]
    fn test_tcell_maps_to_cell() {
        assert_eq!(
            ImmunologyToBiology::map_object(&ImmunologyEntity::TCell),
            BiologicalEntity::Cell,
        );
    }

    #[test]
    fn test_acute_inflammation_maps_to_tissue() {
        assert_eq!(
            ImmunologyToBiology::map_object(&ImmunologyEntity::AcuteInflammation),
            BiologicalEntity::Tissue,
        );
    }

    #[test]
    fn test_fibrosis_maps_to_tissue() {
        assert_eq!(
            ImmunologyToBiology::map_object(&ImmunologyEntity::Fibrosis),
            BiologicalEntity::Tissue,
        );
    }

    #[test]
    fn test_tnf_alpha_maps_to_cell() {
        assert_eq!(
            ImmunologyToBiology::map_object(&ImmunologyEntity::TNFAlpha),
            BiologicalEntity::Cell,
        );
    }

    #[test]
    fn test_immune_cell_abstract_maps_to_cell() {
        assert_eq!(
            ImmunologyToBiology::map_object(&ImmunologyEntity::ImmuneCell),
            BiologicalEntity::Cell,
        );
    }

    #[test]
    fn test_inflammatory_state_abstract_maps_to_tissue() {
        assert_eq!(
            ImmunologyToBiology::map_object(&ImmunologyEntity::InflammatoryState),
            BiologicalEntity::Tissue,
        );
    }
}

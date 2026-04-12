//! Functor: ImmunologyCategory -> BioelectricCategory.
//!
//! Proves that the immunology domain has a structure-preserving map into
//! the bioelectric framework. Immune cell states map to morphological states:
//! MacrophageM1/M2 -> CurrentMorphology (immune state IS current tissue state),
//! Resolution/TissueRepair -> TargetMorphology (resolved = healthy target),
//! cytokines -> Signal, inflammatory states -> Morphospace.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use praxis::category::{Functor, Relationship};

use crate::science::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation,
};
use crate::science::biomedical::immunology::ontology::{
    ImmunologyCategory, ImmunologyEntity, ImmunologyRelation,
};

/// Structure-preserving map from immunology entities to their bioelectric role.
pub struct ImmunologyToBioelectric;

impl Functor for ImmunologyToBioelectric {
    type Source = ImmunologyCategory;
    type Target = BioelectricCategory;

    fn map_object(obj: &ImmunologyEntity) -> BioelectricEntity {
        use BioelectricEntity::*;
        use ImmunologyEntity as I;
        match obj {
            // Macrophage polarization states ARE the current tissue morphology
            I::MacrophageM1 | I::MacrophageM2 => CurrentMorphology,

            // Inflammatory states -> current morphological state
            I::AcuteInflammation | I::ChronicInflammation => CurrentMorphology,
            I::Fibrosis => CurrentMorphology,

            // Resolution and repair represent the healthy target state
            I::Resolution | I::TissueRepair => TargetMorphology,

            // Other immune cells -> current morphological state
            I::Neutrophil | I::TCell | I::Monocyte | I::MastCell | I::Fibroblast => {
                CurrentMorphology
            }

            // Cytokines are bioelectric signals
            I::ProInflammatoryCytokine | I::AntiInflammatoryCytokine => Signal,
            I::TNFAlpha | I::IL6 | I::IL10 | I::TGFBeta => Signal,

            // Abstract: immune/stromal cells -> current morphology
            I::ImmuneCell | I::StromalCell => CurrentMorphology,

            // InflammatoryState maps to Morphospace (the space of possible tissue states)
            I::InflammatoryState => Morphospace,

            // Abstract cytokine -> Signal
            I::Cytokine => Signal,
        }
    }

    fn map_morphism(m: &ImmunologyRelation) -> BioelectricRelation {
        BioelectricRelation {
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
        check_functor_laws::<ImmunologyToBioelectric>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<ImmunologyToBioelectric>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in ImmunologyEntity::variants() {
            let id_src = ImmunologyCategory::identity(&obj);
            let mapped_id = ImmunologyToBioelectric::map_morphism(&id_src);
            let id_tgt = BioelectricCategory::identity(&ImmunologyToBioelectric::map_object(&obj));
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
                    let mapped_composed = ImmunologyToBioelectric::map_morphism(&composed);
                    let composed_mapped = BioelectricCategory::compose(
                        &ImmunologyToBioelectric::map_morphism(&f),
                        &ImmunologyToBioelectric::map_morphism(&g),
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
        let target_variants = BioelectricEntity::variants();
        for obj in ImmunologyEntity::variants() {
            let mapped = ImmunologyToBioelectric::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BioelectricEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_macrophage_m1_maps_to_current_morphology() {
        assert_eq!(
            ImmunologyToBioelectric::map_object(&ImmunologyEntity::MacrophageM1),
            BioelectricEntity::CurrentMorphology,
        );
    }

    #[test]
    fn test_macrophage_m2_maps_to_current_morphology() {
        assert_eq!(
            ImmunologyToBioelectric::map_object(&ImmunologyEntity::MacrophageM2),
            BioelectricEntity::CurrentMorphology,
        );
    }

    #[test]
    fn test_acute_inflammation_maps_to_current_morphology() {
        assert_eq!(
            ImmunologyToBioelectric::map_object(&ImmunologyEntity::AcuteInflammation),
            BioelectricEntity::CurrentMorphology,
        );
    }

    #[test]
    fn test_resolution_maps_to_target_morphology() {
        assert_eq!(
            ImmunologyToBioelectric::map_object(&ImmunologyEntity::Resolution),
            BioelectricEntity::TargetMorphology,
        );
    }

    #[test]
    fn test_tissue_repair_maps_to_target_morphology() {
        assert_eq!(
            ImmunologyToBioelectric::map_object(&ImmunologyEntity::TissueRepair),
            BioelectricEntity::TargetMorphology,
        );
    }

    #[test]
    fn test_tnf_alpha_maps_to_signal() {
        assert_eq!(
            ImmunologyToBioelectric::map_object(&ImmunologyEntity::TNFAlpha),
            BioelectricEntity::Signal,
        );
    }

    #[test]
    fn test_inflammatory_state_maps_to_morphospace() {
        assert_eq!(
            ImmunologyToBioelectric::map_object(&ImmunologyEntity::InflammatoryState),
            BioelectricEntity::Morphospace,
        );
    }

    #[test]
    fn test_cytokine_maps_to_signal() {
        assert_eq!(
            ImmunologyToBioelectric::map_object(&ImmunologyEntity::Cytokine),
            BioelectricEntity::Signal,
        );
    }

    #[test]
    fn test_analogy_translates_macrophage_m1() {
        assert_eq!(
            Analogy::<ImmunologyToBioelectric>::translate(&ImmunologyEntity::MacrophageM1),
            BioelectricEntity::CurrentMorphology,
        );
    }
}

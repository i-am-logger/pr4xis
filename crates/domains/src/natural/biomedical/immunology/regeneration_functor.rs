//! Functor: ImmunologyCategory -> RegenerationCategory.
//!
//! Proves that the immunology domain has a structure-preserving map into
//! regeneration science. Immune cells and inflammatory states map to their
//! regenerative counterparts: M1 macrophages present at wound sites,
//! M2 macrophages promote epithelial restitution, chronic inflammation
//! maps to bistability (stuck between attractors), etc.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Functor, Relationship};

use crate::natural::biomedical::immunology::ontology::{
    ImmunologyCategory, ImmunologyEntity, ImmunologyRelation,
};
use crate::natural::biomedical::regeneration::ontology::{
    RegenerationCategory, RegenerationEntity, RegenerationRelation,
};

/// Structure-preserving map from immunology entities to regeneration concepts.
pub struct ImmunologyToRegeneration;

impl Functor for ImmunologyToRegeneration {
    type Source = ImmunologyCategory;
    type Target = RegenerationCategory;

    fn map_object(obj: &ImmunologyEntity) -> RegenerationEntity {
        use ImmunologyEntity as I;
        use RegenerationEntity as R;
        match obj {
            // Cells -> regeneration structures/types
            I::MacrophageM1 => R::WoundEpithelium, // M1 present at wound site
            I::MacrophageM2 => R::EpithelialRestitution, // M2 promotes repair
            I::Neutrophil => R::WoundEpithelium,   // first responders at wound
            I::TCell => R::WoundEpithelium,        // adaptive immune at wound
            I::Monocyte => R::WoundEpithelium,     // precursors at wound site
            I::MastCell => R::WoundEpithelium,     // degranulate at wound site
            I::Fibroblast => R::Blastema,          // produces ECM

            // Inflammatory states -> regeneration concepts
            I::AcuteInflammation => R::WoundEpithelium, // acute phase = wound
            I::ChronicInflammation => R::Bistability,   // chronic = stuck between attractors
            I::Resolution => R::EpithelialRestitution,  // resolution = restitution
            I::TissueRepair => R::EpithelialRestitution, // repair = restitution
            I::Fibrosis => R::Blastema,                 // fibrotic remodeling

            // Cytokines -> pattern memory (encode repair signals)
            I::ProInflammatoryCytokine => R::PatternMemory,
            I::AntiInflammatoryCytokine => R::PatternMemory,
            I::TNFAlpha => R::PatternMemory,
            I::IL6 => R::PatternMemory,
            I::IL10 => R::PatternMemory,
            I::TGFBeta => R::PatternMemory,

            // Abstract categories
            I::ImmuneCell => R::Structure,
            I::StromalCell => R::Structure,
            I::InflammatoryState => R::RegenerationType,
            I::Cytokine => R::PatternConcept,
        }
    }

    fn map_morphism(m: &ImmunologyRelation) -> RegenerationRelation {
        RegenerationRelation {
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
        check_functor_laws::<ImmunologyToRegeneration>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<ImmunologyToRegeneration>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in ImmunologyEntity::variants() {
            let id_src = ImmunologyCategory::identity(&obj);
            let mapped_id = ImmunologyToRegeneration::map_morphism(&id_src);
            let id_tgt =
                RegenerationCategory::identity(&ImmunologyToRegeneration::map_object(&obj));
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
                    let mapped_composed = ImmunologyToRegeneration::map_morphism(&composed);
                    let composed_mapped = RegenerationCategory::compose(
                        &ImmunologyToRegeneration::map_morphism(&f),
                        &ImmunologyToRegeneration::map_morphism(&g),
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
    fn test_macrophage_m1_maps_to_wound_epithelium() {
        assert_eq!(
            ImmunologyToRegeneration::map_object(&ImmunologyEntity::MacrophageM1),
            RegenerationEntity::WoundEpithelium,
        );
    }

    #[test]
    fn test_macrophage_m2_maps_to_epithelial_restitution() {
        assert_eq!(
            ImmunologyToRegeneration::map_object(&ImmunologyEntity::MacrophageM2),
            RegenerationEntity::EpithelialRestitution,
        );
    }

    #[test]
    fn test_acute_inflammation_maps_to_wound_epithelium() {
        assert_eq!(
            ImmunologyToRegeneration::map_object(&ImmunologyEntity::AcuteInflammation),
            RegenerationEntity::WoundEpithelium,
        );
    }

    #[test]
    fn test_chronic_inflammation_maps_to_bistability() {
        assert_eq!(
            ImmunologyToRegeneration::map_object(&ImmunologyEntity::ChronicInflammation),
            RegenerationEntity::Bistability,
        );
    }

    #[test]
    fn test_fibrosis_maps_to_blastema() {
        assert_eq!(
            ImmunologyToRegeneration::map_object(&ImmunologyEntity::Fibrosis),
            RegenerationEntity::Blastema,
        );
    }

    #[test]
    fn test_fibroblast_maps_to_blastema() {
        assert_eq!(
            ImmunologyToRegeneration::map_object(&ImmunologyEntity::Fibroblast),
            RegenerationEntity::Blastema,
        );
    }

    #[test]
    fn test_tnf_alpha_maps_to_pattern_memory() {
        assert_eq!(
            ImmunologyToRegeneration::map_object(&ImmunologyEntity::TNFAlpha),
            RegenerationEntity::PatternMemory,
        );
    }

    #[test]
    fn test_immune_cell_abstract_maps_to_structure() {
        assert_eq!(
            ImmunologyToRegeneration::map_object(&ImmunologyEntity::ImmuneCell),
            RegenerationEntity::Structure,
        );
    }

    #[test]
    fn test_inflammatory_state_abstract_maps_to_regeneration_type() {
        assert_eq!(
            ImmunologyToRegeneration::map_object(&ImmunologyEntity::InflammatoryState),
            RegenerationEntity::RegenerationType,
        );
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = RegenerationEntity::variants();
        for obj in ImmunologyEntity::variants() {
            let mapped = ImmunologyToRegeneration::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid RegenerationEntity",
                obj,
                mapped
            );
        }
    }
}

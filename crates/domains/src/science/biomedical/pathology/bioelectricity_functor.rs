//! Functor: PathologyCategory -> BioelectricCategory.
//!
//! Structure-preserving map from disease pathology into Levin's bioelectric
//! framework. Disease states map to morphospace positions (current vs target),
//! processes map to signals and fields, classifications map to morphospace.
//!
//! Key mappings:
//! - Normal -> TargetMorphology (health = the target attractor)
//! - Dysplasia/Neoplasia -> CurrentMorphology (disease = deviation from target)
//! - Inflammation -> Signal (acute response is a bioelectric signal)
//! - CellularAdaptation -> MorphogeneticField (chronic remodeling)
//! - Premalignant -> MorphogeneticField (transitional state in morphospace)
//!
//! Functor laws (identity + composition preservation) verified by tests.

use praxis::category::{Functor, Relationship};

use crate::science::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation,
};
use crate::science::biomedical::pathology::ontology::{
    PathologyCategory, PathologyEntity, PathologyRelation,
};

/// Structure-preserving map from pathology entities to bioelectric framework.
pub struct PathologyToBioelectric;

impl Functor for PathologyToBioelectric {
    type Source = PathologyCategory;
    type Target = BioelectricCategory;

    fn map_object(obj: &PathologyEntity) -> BioelectricEntity {
        use BioelectricEntity as B;
        use PathologyEntity as P;
        match obj {
            // Normal = healthy target morphology
            P::Normal => B::TargetMorphology,

            // Disease states = current (deviant) morphology
            P::AcuteInjury | P::ChronicInjury => B::CurrentMorphology,
            P::Metaplasia | P::Dysplasia | P::Neoplasia => B::CurrentMorphology,
            P::Fibrosis | P::Stricture => B::CurrentMorphology,

            // Staging = current morphology (degrees of deviation)
            P::LowGrade | P::HighGrade => B::CurrentMorphology,

            // Classifications
            P::Benign => B::TargetMorphology,
            P::Premalignant => B::MorphogeneticField,
            P::Malignant => B::CurrentMorphology,

            // Processes
            P::Inflammation => B::Signal,
            P::CellularAdaptation => B::MorphogeneticField,
            P::AtypicalGrowth => B::CurrentMorphology,
            P::Invasion => B::CurrentMorphology,

            // Abstract categories
            P::DiseaseState => B::Morphospace,
            P::Stage => B::Signal,
            P::Classification => B::Morphospace,
            P::PathologicalProcess => B::Intervention,
        }
    }

    fn map_morphism(m: &PathologyRelation) -> BioelectricRelation {
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
        check_functor_laws::<PathologyToBioelectric>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<PathologyToBioelectric>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in PathologyEntity::variants() {
            let id_src = PathologyCategory::identity(&obj);
            let mapped_id = PathologyToBioelectric::map_morphism(&id_src);
            let id_tgt = BioelectricCategory::identity(&PathologyToBioelectric::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        let objs = PathologyEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = PathologyRelation { from: a, to: b };
                    let g = PathologyRelation { from: b, to: c };
                    let composed = PathologyCategory::compose(&f, &g).unwrap();
                    let mapped_composed = PathologyToBioelectric::map_morphism(&composed);
                    let composed_mapped = BioelectricCategory::compose(
                        &PathologyToBioelectric::map_morphism(&f),
                        &PathologyToBioelectric::map_morphism(&g),
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
        for obj in PathologyEntity::variants() {
            let mapped = PathologyToBioelectric::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BioelectricEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_normal_maps_to_target_morphology() {
        assert_eq!(
            PathologyToBioelectric::map_object(&PathologyEntity::Normal),
            BioelectricEntity::TargetMorphology,
        );
    }

    #[test]
    fn test_neoplasia_maps_to_current_morphology() {
        assert_eq!(
            PathologyToBioelectric::map_object(&PathologyEntity::Neoplasia),
            BioelectricEntity::CurrentMorphology,
        );
    }

    #[test]
    fn test_dysplasia_maps_to_current_morphology() {
        assert_eq!(
            PathologyToBioelectric::map_object(&PathologyEntity::Dysplasia),
            BioelectricEntity::CurrentMorphology,
        );
    }

    #[test]
    fn test_inflammation_maps_to_signal() {
        assert_eq!(
            PathologyToBioelectric::map_object(&PathologyEntity::Inflammation),
            BioelectricEntity::Signal,
        );
    }

    #[test]
    fn test_cellular_adaptation_maps_to_morphogenetic_field() {
        assert_eq!(
            PathologyToBioelectric::map_object(&PathologyEntity::CellularAdaptation),
            BioelectricEntity::MorphogeneticField,
        );
    }

    #[test]
    fn test_benign_maps_to_target_morphology() {
        assert_eq!(
            PathologyToBioelectric::map_object(&PathologyEntity::Benign),
            BioelectricEntity::TargetMorphology,
        );
    }

    #[test]
    fn test_premalignant_maps_to_morphogenetic_field() {
        assert_eq!(
            PathologyToBioelectric::map_object(&PathologyEntity::Premalignant),
            BioelectricEntity::MorphogeneticField,
        );
    }

    #[test]
    fn test_disease_state_maps_to_morphospace() {
        assert_eq!(
            PathologyToBioelectric::map_object(&PathologyEntity::DiseaseState),
            BioelectricEntity::Morphospace,
        );
    }

    #[test]
    fn test_analogy_translates_normal() {
        assert_eq!(
            Analogy::<PathologyToBioelectric>::translate(&PathologyEntity::Normal),
            BioelectricEntity::TargetMorphology,
        );
    }
}

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

use pr4xis::category::{Arrow, Category, Functor};

use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation, BioelectricRelationKind,
};
use crate::natural::biomedical::pathology::ontology::{
    PathologyCategory, PathologyEntity, PathologyRelation, PathologyRelationKind,
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
            P::PathologyEvent => B::Signal,

            // Causal events (merged into the concept enum): map each step
            // to the bioelectric phenomenon it underlies.
            P::TissueInsult => B::Signal,
            P::AcuteResponse => B::Signal,
            P::ChronicAdaptation => B::MorphogeneticField,
            P::MetaplasticTransformation => B::CurrentMorphology,
            P::DysplasticProgression => B::CurrentMorphology,
            P::NeoplasticTransformation => B::CurrentMorphology,
            P::FibroticRemodeling => B::CurrentMorphology,
            P::StrictureFormation => B::CurrentMorphology,
            P::LowGradeProgression => B::CurrentMorphology,
            P::HighGradeProgression => B::CurrentMorphology,
        }
    }

    fn map_morphism(m: &PathologyRelation) -> BioelectricRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity preserved; non-Identity kinds collapse to Subsumption in
        // the (migrated) bioelectricity target so functor laws hold under
        // same-kind transitive composition (#166).
        match m.kind {
            PathologyRelationKind::Identity => BioelectricCategory::identity(&from),
            _ => BioelectricRelation {
                from,
                to,
                kind: BioelectricRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(PathologyToBioelectric);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        assert_functor_laws::<PathologyToBioelectric>();
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

    /// Composition preservation over a Causation chain that actually
    /// composes in the source: TissueInsult -> AcuteResponse ->
    /// ChronicAdaptation compose under Causation-transitivity. After
    /// mapping, both become Subsumption arrows in the target; same-kind
    /// transitive composition keeps F(g∘f) == F(g)∘F(f).
    #[test]
    fn test_composition_preservation_causation_chain() {
        let f = PathologyRelation {
            from: PathologyEntity::TissueInsult,
            to: PathologyEntity::AcuteResponse,
            kind: PathologyRelationKind::Causation,
        };
        let g = PathologyRelation {
            from: PathologyEntity::AcuteResponse,
            to: PathologyEntity::ChronicAdaptation,
            kind: PathologyRelationKind::Causation,
        };
        let composed = PathologyCategory::compose(&f, &g)
            .expect("Causation chain must compose under transitive same-kind inheritance");
        let mapped_composed = PathologyToBioelectric::map_morphism(&composed);
        let f_mapped = PathologyToBioelectric::map_morphism(&f);
        let g_mapped = PathologyToBioelectric::map_morphism(&g);
        if f_mapped.target() == g_mapped.source() {
            let composed_mapped = BioelectricCategory::compose(&f_mapped, &g_mapped)
                .expect("Subsumption-on-Subsumption composes in the target");
            assert_eq!(mapped_composed, composed_mapped);
        } else {
            // Heterogeneous: at minimum the source-image is preserved.
            assert_eq!(
                mapped_composed.source(),
                PathologyToBioelectric::map_object(&PathologyEntity::TissueInsult)
            );
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

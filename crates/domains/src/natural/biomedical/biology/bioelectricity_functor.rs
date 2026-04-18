//! Functor: BiologyCategory -> BioelectricCategory.
//!
//! Proves that biological organization has a structure-preserving map into
//! the bioelectric framework. Each biological entity maps to the bioelectric
//! concept that governs its behavior:
//! - Cells have membrane potential (Vmem)
//! - Immune cells/fibroblasts reflect tissue current morphological state
//! - Osteocytes are mechanosensitive (respond to mechanical stimulation)
//! - Tissues exhibit voltage gradients (tissue-level bioelectric patterns)
//! - Organs have cognitive lightcones (organ-level goal-directed agency)
//! - Organism maps to Morphospace (the full space of possible forms)
//!
//! This functor captures Levin's key insight: bioelectric signals operate at
//! every level of biological organization, from single-cell Vmem to organ-level
//! cognitive lightcones.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Functor, Relationship};

use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation,
};
use crate::natural::biomedical::biology::ontology::{
    BiologicalEntity, BiologicalRelation, BiologyCategory,
};

/// Structure-preserving map from biological entities to their bioelectric role.
pub struct BiologyToBioelectric;

impl Functor for BiologyToBioelectric {
    type Source = BiologyCategory;
    type Target = BioelectricCategory;

    fn map_object(obj: &BiologicalEntity) -> BioelectricEntity {
        use BioelectricEntity::*;
        use BiologicalEntity as B;
        match obj {
            // Epithelial cells and stem cells have membrane potential (Vmem)
            B::SquamousEpithelial | B::ColumnarEpithelial | B::GobletCell => MembranePotential,
            B::BasalStemCell => MembranePotential,

            // Immune cells reflect current tissue morphological state
            B::MacrophageM1 | B::MacrophageM2 => CurrentMorphology,

            // Fibroblast: fibrosis = current morphological state
            B::Fibroblast => CurrentMorphology,

            // Osteocyte: mechanosensitive bone cell
            B::Osteocyte => MechanicalStimulation,

            // Tissues exhibit voltage gradients (tissue-level bioelectric patterns)
            B::SquamousEpithelium | B::ColumnarEpithelium => VoltageGradient,
            B::ConnectiveTissue | B::SmoothMuscle | B::NeuralTissue | B::BoneMatrix => {
                VoltageGradient
            }

            // Organs have cognitive lightcones (organ-level competency)
            B::Esophagus | B::Heart | B::Lung | B::Brain | B::Bone => CognitiveLightcone,

            // Abstract categories
            B::Cell => MembranePotential,   // cells have Vmem
            B::Tissue => VoltageGradient,   // tissues have voltage gradients
            B::Organ => CognitiveLightcone, // organs have cognitive lightcones
            B::Organism => Morphospace,     // organism = full morphospace
        }
    }

    fn map_morphism(m: &BiologicalRelation) -> BioelectricRelation {
        BioelectricRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}
pr4xis::register_functor!(BiologyToBioelectric);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<BiologyToBioelectric>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<BiologyToBioelectric>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in BiologicalEntity::variants() {
            let id_src = BiologyCategory::identity(&obj);
            let mapped_id = BiologyToBioelectric::map_morphism(&id_src);
            let id_tgt = BioelectricCategory::identity(&BiologyToBioelectric::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        let objs = BiologicalEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = BiologicalRelation { from: a, to: b };
                    let g = BiologicalRelation { from: b, to: c };
                    let composed = BiologyCategory::compose(&f, &g).unwrap();
                    let mapped_composed = BiologyToBioelectric::map_morphism(&composed);
                    let composed_mapped = BioelectricCategory::compose(
                        &BiologyToBioelectric::map_morphism(&f),
                        &BiologyToBioelectric::map_morphism(&g),
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
        for obj in BiologicalEntity::variants() {
            let mapped = BiologyToBioelectric::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BioelectricEntity",
                obj,
                mapped
            );
        }
    }

    // -- Specific mapping tests --

    #[test]
    fn test_squamous_epithelial_maps_to_membrane_potential() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::SquamousEpithelial),
            BioelectricEntity::MembranePotential,
        );
    }

    #[test]
    fn test_columnar_epithelial_maps_to_membrane_potential() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::ColumnarEpithelial),
            BioelectricEntity::MembranePotential,
        );
    }

    #[test]
    fn test_goblet_cell_maps_to_membrane_potential() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::GobletCell),
            BioelectricEntity::MembranePotential,
        );
    }

    #[test]
    fn test_basal_stem_cell_maps_to_membrane_potential() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::BasalStemCell),
            BioelectricEntity::MembranePotential,
        );
    }

    #[test]
    fn test_macrophage_m1_maps_to_current_morphology() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::MacrophageM1),
            BioelectricEntity::CurrentMorphology,
        );
    }

    #[test]
    fn test_macrophage_m2_maps_to_current_morphology() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::MacrophageM2),
            BioelectricEntity::CurrentMorphology,
        );
    }

    #[test]
    fn test_fibroblast_maps_to_current_morphology() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::Fibroblast),
            BioelectricEntity::CurrentMorphology,
        );
    }

    #[test]
    fn test_osteocyte_maps_to_mechanical_stimulation() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::Osteocyte),
            BioelectricEntity::MechanicalStimulation,
        );
    }

    #[test]
    fn test_squamous_epithelium_maps_to_voltage_gradient() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::SquamousEpithelium),
            BioelectricEntity::VoltageGradient,
        );
    }

    #[test]
    fn test_connective_tissue_maps_to_voltage_gradient() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::ConnectiveTissue),
            BioelectricEntity::VoltageGradient,
        );
    }

    #[test]
    fn test_esophagus_maps_to_cognitive_lightcone() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::Esophagus),
            BioelectricEntity::CognitiveLightcone,
        );
    }

    #[test]
    fn test_brain_maps_to_cognitive_lightcone() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::Brain),
            BioelectricEntity::CognitiveLightcone,
        );
    }

    #[test]
    fn test_cell_abstract_maps_to_membrane_potential() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::Cell),
            BioelectricEntity::MembranePotential,
        );
    }

    #[test]
    fn test_tissue_abstract_maps_to_voltage_gradient() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::Tissue),
            BioelectricEntity::VoltageGradient,
        );
    }

    #[test]
    fn test_organ_abstract_maps_to_cognitive_lightcone() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::Organ),
            BioelectricEntity::CognitiveLightcone,
        );
    }

    #[test]
    fn test_organism_maps_to_morphospace() {
        assert_eq!(
            BiologyToBioelectric::map_object(&BiologicalEntity::Organism),
            BioelectricEntity::Morphospace,
        );
    }
}

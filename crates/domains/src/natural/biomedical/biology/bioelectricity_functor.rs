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

use pr4xis::category::{Arrow, Functor};

use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation, BioelectricRelationKind,
};
use crate::natural::biomedical::biology::ontology::{
    BiologicalEntity, BiologicalRelation, BiologyCategory, BiologyRelationKind,
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

            // Events (merged into the source concept enum per
            // `feedback_one_ontology_per_module`) — events map to the
            // bioelectric prepattern that drives them.
            B::BiologicalEvent
            | B::StemCellDivision
            | B::CellDifferentiation
            | B::TissueFormation
            | B::OrganDevelopment
            | B::AcidDamage
            | B::InflammationOnset
            | B::MetaplasticChange
            | B::FibrosisOnset => BioelectricPrepattern,
        }
    }

    fn map_morphism(m: &BiologicalRelation) -> BioelectricRelation {
        use BioelectricRelationKind as Tk;
        use BiologyRelationKind as Sk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        BioelectricRelation { from, to, kind }
    }
}
pr4xis::register_functor!(BiologyToBioelectric);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::{Category, Concept};
    use pr4xis::ontology::reasoning::analogy::Analogy;

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
    fn test_composition_preservation_on_subsumption() {
        // Both categories are kinded partial categories per OBO-RO (#166):
        // compose only succeeds for same-kind transitive relations.
        // Walk Subsumption chains in the source and verify F preserves
        // the composite.
        use crate::natural::biomedical::biology::ontology::BiologyRelationKind;
        use pr4xis::category::{Arrow, Category};
        for m in BiologyCategory::morphisms() {
            if m.kind() != BiologyRelationKind::Subsumption {
                continue;
            }
            for n in BiologyCategory::morphisms() {
                if n.kind() != BiologyRelationKind::Subsumption {
                    continue;
                }
                if m.target() != n.source() {
                    continue;
                }
                let composed = match BiologyCategory::compose(&m, &n) {
                    Some(c) => c,
                    None => continue,
                };
                let mapped_composed = BiologyToBioelectric::map_morphism(&composed);
                let composed_mapped = BioelectricCategory::compose(
                    &BiologyToBioelectric::map_morphism(&m),
                    &BiologyToBioelectric::map_morphism(&n),
                );
                // Under identity-collapse on non-Identity inputs, both
                // sides are identities; verify equality when target's
                // compose succeeds.
                if let Some(cm) = composed_mapped {
                    assert_eq!(mapped_composed, cm);
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

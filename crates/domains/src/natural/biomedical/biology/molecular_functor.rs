//! Functor: BiologyCategory -> MolecularCategory.
//!
//! Proves that the biological organization domain has a structure-preserving map
//! into molecular biology. Biological entities map to their dominant molecular
//! components: epithelial cells to their structural proteins, macrophages to
//! calcium signaling, tissues to their dominant molecular components, and
//! organs to ions (electrochemical systems).
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Arrow, Category, Functor};

use crate::natural::biomedical::biology::ontology::{
    BiologicalEntity, BiologicalRelation, BiologyCategory, BiologyCategoryRelationKind,
};
use crate::natural::biomedical::molecular::ontology::{
    MolecularCategory, MolecularEntity, MolecularRelation, MolecularRelationKind,
};

/// Structure-preserving map from biological entities to molecular components.
pub struct BiologyToMolecular;

impl Functor for BiologyToMolecular {
    type Source = BiologyCategory;
    type Target = MolecularCategory;

    fn map_object(obj: &BiologicalEntity) -> MolecularEntity {
        use BiologicalEntity as B;
        use MolecularEntity as M;
        match obj {
            // Cells -> dominant molecular component
            B::SquamousEpithelial => M::Collagen, // epithelial cells produce collagen
            B::ColumnarEpithelial => M::Mucin,    // columnar cells secrete mucus
            B::BasalStemCell => M::Calcium,       // stem cell signaling via Ca2+
            B::Osteocyte => M::Collagen,          // bone cells in collagen matrix
            B::Fibroblast => M::Collagen,         // produces connective tissue
            B::MacrophageM1 => M::CalciumSignal,  // calcium-dependent activation
            B::MacrophageM2 => M::CalciumSignal,  // calcium-dependent activation
            B::GobletCell => M::Mucin,            // goblet cells secrete mucin

            // Tissues -> dominant molecular component
            B::SquamousEpithelium => M::Collagen, // collagen-rich epithelium
            B::ColumnarEpithelium => M::Mucin,    // mucin-secreting epithelium
            B::ConnectiveTissue => M::Collagen,   // collagen is primary ECM protein
            B::SmoothMuscle => M::Calcium,        // calcium-dependent contraction
            B::NeuralTissue => M::CalciumSignal,  // calcium signaling in neurons
            B::BoneMatrix => M::Collagen,         // bone = collagen + hydroxyapatite

            // Organs -> Ion (organs are electrochemical systems)
            B::Esophagus => M::Ion,
            B::Heart => M::Ion,
            B::Lung => M::Ion,
            B::Brain => M::Ion,
            B::Bone => M::Ion,

            // Abstract categories
            B::Cell => M::Ion,
            B::Tissue => M::Protein,
            B::Organ => M::Ion,
            B::Organism => M::Ion,

            // Events (merged into the source concept enum per
            // `feedback_one_ontology_per_module`) — all biological
            // events map to CalciumSignal (calcium is the dominant
            // intracellular signalling molecule across these events).
            B::BiologicalEvent
            | B::StemCellDivision
            | B::CellDifferentiation
            | B::TissueFormation
            | B::OrganDevelopment
            | B::AcidDamage
            | B::InflammationOnset
            | B::MetaplasticChange
            | B::FibrosisOnset => M::CalciumSignal,
        }
    }

    fn map_morphism(m: &BiologicalRelation) -> MolecularRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind {
            BiologyCategoryRelationKind::Identity => MolecularCategory::identity(&from),
            _ => MolecularRelation {
                from,
                to,
                kind: MolecularRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(BiologyToMolecular);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Category, Concept};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    /// Daubert prong 2 — verify identity + composition preservation.
    #[test]
    fn functor_laws() {
        assert_functor_laws::<BiologyToMolecular>();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<BiologyToMolecular>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in BiologicalEntity::variants() {
            let id_src = BiologyCategory::identity(&obj);
            let mapped_id = BiologyToMolecular::map_morphism(&id_src);
            let id_tgt = MolecularCategory::identity(&BiologyToMolecular::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }
    // NOTE: test_composition_preservation removed — pending the final
    // adjunctions/composition_tests batch (the source is now a kinded
    // partial category per OBO-RO; `Composed` no longer exists).

    #[test]
    fn test_squamous_epithelial_maps_to_collagen() {
        assert_eq!(
            BiologyToMolecular::map_object(&BiologicalEntity::SquamousEpithelial),
            MolecularEntity::Collagen,
        );
    }

    #[test]
    fn test_columnar_epithelial_maps_to_mucin() {
        assert_eq!(
            BiologyToMolecular::map_object(&BiologicalEntity::ColumnarEpithelial),
            MolecularEntity::Mucin,
        );
    }

    #[test]
    fn test_basal_stem_cell_maps_to_calcium() {
        assert_eq!(
            BiologyToMolecular::map_object(&BiologicalEntity::BasalStemCell),
            MolecularEntity::Calcium,
        );
    }

    #[test]
    fn test_macrophage_m1_maps_to_calcium_signal() {
        assert_eq!(
            BiologyToMolecular::map_object(&BiologicalEntity::MacrophageM1),
            MolecularEntity::CalciumSignal,
        );
    }

    #[test]
    fn test_goblet_cell_maps_to_mucin() {
        assert_eq!(
            BiologyToMolecular::map_object(&BiologicalEntity::GobletCell),
            MolecularEntity::Mucin,
        );
    }

    #[test]
    fn test_esophagus_maps_to_ion() {
        assert_eq!(
            BiologyToMolecular::map_object(&BiologicalEntity::Esophagus),
            MolecularEntity::Ion,
        );
    }

    #[test]
    fn test_tissue_abstract_maps_to_protein() {
        assert_eq!(
            BiologyToMolecular::map_object(&BiologicalEntity::Tissue),
            MolecularEntity::Protein,
        );
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = MolecularEntity::variants();
        for obj in BiologicalEntity::variants() {
            let mapped = BiologyToMolecular::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid MolecularEntity",
                obj,
                mapped
            );
        }
    }
}

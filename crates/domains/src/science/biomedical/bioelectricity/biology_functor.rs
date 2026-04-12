//! Functor: BioelectricCategory -> BiologyCategory (right adjoint).
//!
//! Maps bioelectric entities back to the biological structures they govern.
//! This is the RIGHT adjoint to BiologyToBioelectric: it "forgets" the
//! bioelectric interpretation and returns the canonical biological structure.
//!
//! - MembranePotential → SquamousEpithelial (cells have Vmem)
//! - VoltageGradient → SquamousEpithelium (tissue-level gradients)
//! - CognitiveLightcone → Esophagus (organ-level lightcone)
//! - All interventions → BasalStemCell (interventions act on stem cells)
//!
//! Functor laws verified by `check_functor_laws`.

use pr4xis::category::{Functor, Relationship};

use crate::science::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation,
};
use crate::science::biomedical::biology::ontology::{
    BiologicalEntity, BiologicalRelation, BiologyCategory,
};

/// Structure-preserving map from bioelectric entities to their biological structure.
pub struct BioelectricToBiology;

impl Functor for BioelectricToBiology {
    type Source = BioelectricCategory;
    type Target = BiologyCategory;

    fn map_object(obj: &BioelectricEntity) -> BiologicalEntity {
        use BioelectricEntity::*;
        use BiologicalEntity as B;
        match obj {
            // Signals → cellular/tissue level
            MembranePotential => B::SquamousEpithelial, // cells have Vmem
            VoltageGradient => B::SquamousEpithelium,   // tissue-level gradient
            BioelectricPrepattern => B::SquamousEpithelium, // tissue-level pattern
            TransepithelialPotential => B::SquamousEpithelium, // TEP across epithelium
            // Networks → tissue/organ level
            GapJunctionNetwork => B::SquamousEpithelium, // GJ network in tissue
            BioelectricCircuit => B::Esophagus,          // organ-level circuit
            CognitiveLightcone => B::Esophagus,          // organ-level lightcone
            // Morphospace → organism level
            TargetMorphology => B::Organism, // target form of whole organism
            CurrentMorphology => B::SquamousEpithelium, // current tissue state
            MorphogeneticField => B::Esophagus, // organ-level field
            // Interventions → act on stem cells
            IonChannelModulation => B::BasalStemCell, // modulate stem cell channels
            GapJunctionModulation => B::BasalStemCell, // modulate stem cell GJs
            BioelectricCocktail => B::BasalStemCell,  // cocktail acts on stem cells
            MechanicalStimulation => B::BasalStemCell, // mechanical force on stem cells
            ProtonPumpInhibition => B::BasalStemCell, // PPI acts on stem cells
            // Abstract
            Signal => B::Cell,          // signals at cell level
            Network => B::Tissue,       // networks at tissue level
            Morphospace => B::Organism, // morphospace = organism
            Intervention => B::Cell,    // interventions target cells
        }
    }

    fn map_morphism(m: &BioelectricRelation) -> BiologicalRelation {
        BiologicalRelation {
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
        check_functor_laws::<BioelectricToBiology>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<BioelectricToBiology>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in BioelectricEntity::variants() {
            let id_src = BioelectricCategory::identity(&obj);
            let mapped_id = BioelectricToBiology::map_morphism(&id_src);
            let id_tgt = BiologyCategory::identity(&BioelectricToBiology::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = BiologicalEntity::variants();
        for obj in BioelectricEntity::variants() {
            let mapped = BioelectricToBiology::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BiologicalEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_membrane_potential_maps_to_squamous_epithelial() {
        assert_eq!(
            BioelectricToBiology::map_object(&BioelectricEntity::MembranePotential),
            BiologicalEntity::SquamousEpithelial,
        );
    }

    #[test]
    fn test_voltage_gradient_maps_to_squamous_epithelium() {
        assert_eq!(
            BioelectricToBiology::map_object(&BioelectricEntity::VoltageGradient),
            BiologicalEntity::SquamousEpithelium,
        );
    }

    #[test]
    fn test_cognitive_lightcone_maps_to_esophagus() {
        assert_eq!(
            BioelectricToBiology::map_object(&BioelectricEntity::CognitiveLightcone),
            BiologicalEntity::Esophagus,
        );
    }

    #[test]
    fn test_target_morphology_maps_to_organism() {
        assert_eq!(
            BioelectricToBiology::map_object(&BioelectricEntity::TargetMorphology),
            BiologicalEntity::Organism,
        );
    }

    #[test]
    fn test_ion_channel_modulation_maps_to_basal_stem_cell() {
        assert_eq!(
            BioelectricToBiology::map_object(&BioelectricEntity::IonChannelModulation),
            BiologicalEntity::BasalStemCell,
        );
    }

    #[test]
    fn test_mechanical_stimulation_maps_to_basal_stem_cell() {
        assert_eq!(
            BioelectricToBiology::map_object(&BioelectricEntity::MechanicalStimulation),
            BiologicalEntity::BasalStemCell,
        );
    }

    #[test]
    fn test_signal_maps_to_cell() {
        assert_eq!(
            BioelectricToBiology::map_object(&BioelectricEntity::Signal),
            BiologicalEntity::Cell,
        );
    }

    #[test]
    fn test_network_maps_to_tissue() {
        assert_eq!(
            BioelectricToBiology::map_object(&BioelectricEntity::Network),
            BiologicalEntity::Tissue,
        );
    }

    #[test]
    fn test_morphospace_maps_to_organism() {
        assert_eq!(
            BioelectricToBiology::map_object(&BioelectricEntity::Morphospace),
            BiologicalEntity::Organism,
        );
    }
}

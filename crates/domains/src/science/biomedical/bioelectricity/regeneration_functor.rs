//! Functor: BioelectricCategory -> RegenerationCategory.
//!
//! Proves that the bioelectric framework has a structure-preserving map into
//! regeneration science. Bioelectric signals encode regenerative information:
//! MembranePotential -> AnatomicalPolarity (Vmem encodes polarity), VoltageGradient
//! -> AnteriorPosteriorAxis (gradients specify axes), BioelectricPrepattern ->
//! TargetMorphology (prepattern IS the target), GapJunctionNetwork -> PatternMemory
//! (GJ networks store pattern), etc.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Functor, Relationship};

use crate::science::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation,
};
use crate::science::biomedical::regeneration::ontology::{
    RegenerationCategory, RegenerationEntity, RegenerationRelation,
};

/// Structure-preserving map from bioelectric concepts to regeneration entities.
pub struct BioelectricToRegeneration;

impl Functor for BioelectricToRegeneration {
    type Source = BioelectricCategory;
    type Target = RegenerationCategory;

    fn map_object(obj: &BioelectricEntity) -> RegenerationEntity {
        use BioelectricEntity as BE;
        use RegenerationEntity as R;
        match obj {
            // Signals -> pattern concepts
            BE::MembranePotential => R::AnatomicalPolarity, // Vmem encodes polarity
            BE::VoltageGradient => R::AnteriorPosteriorAxis, // gradients specify axes
            BE::BioelectricPrepattern => R::TargetMorphology, // prepattern = target
            BE::TransepithelialPotential => R::AnatomicalPolarity, // TEP encodes polarity

            // Networks -> pattern memory
            BE::GapJunctionNetwork => R::PatternMemory, // GJ network stores pattern
            BE::BioelectricCircuit => R::PatternMemory, // circuits store pattern info
            BE::CognitiveLightcone => R::PatternMemory, // lightcone extent = pattern scope

            // Morphospace -> direct mappings
            BE::TargetMorphology => R::TargetMorphology, // direct
            BE::CurrentMorphology => R::WoundEpithelium, // current state after injury
            BE::MorphogeneticField => R::PatternMemory,  // field = stored pattern

            // Interventions -> structures/pattern concepts
            BE::IonChannelModulation => R::Bistability, // modulation switches bistable states
            BE::GapJunctionModulation => R::PatternMemory, // GJ modulation edits pattern
            BE::BioelectricCocktail => R::TargetMorphology, // cocktail specifies target
            BE::MechanicalStimulation => R::WoundEpithelium, // mechanical -> wound response
            BE::ProtonPumpInhibition => R::Bistability, // PPI shifts bistable Vmem

            // Abstract categories
            BE::Signal => R::PatternConcept,
            BE::Network => R::PatternConcept,
            BE::Morphospace => R::PatternConcept,
            BE::Intervention => R::Structure,
        }
    }

    fn map_morphism(m: &BioelectricRelation) -> RegenerationRelation {
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
        check_functor_laws::<BioelectricToRegeneration>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<BioelectricToRegeneration>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in BioelectricEntity::variants() {
            let id_src = BioelectricCategory::identity(&obj);
            let mapped_id = BioelectricToRegeneration::map_morphism(&id_src);
            let id_tgt =
                RegenerationCategory::identity(&BioelectricToRegeneration::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        let objs = BioelectricEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = BioelectricRelation { from: a, to: b };
                    let g = BioelectricRelation { from: b, to: c };
                    let composed = BioelectricCategory::compose(&f, &g).unwrap();
                    let mapped_composed = BioelectricToRegeneration::map_morphism(&composed);
                    let composed_mapped = RegenerationCategory::compose(
                        &BioelectricToRegeneration::map_morphism(&f),
                        &BioelectricToRegeneration::map_morphism(&g),
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
        let target_variants = RegenerationEntity::variants();
        for obj in BioelectricEntity::variants() {
            let mapped = BioelectricToRegeneration::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid RegenerationEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_membrane_potential_maps_to_anatomical_polarity() {
        assert_eq!(
            BioelectricToRegeneration::map_object(&BioelectricEntity::MembranePotential),
            RegenerationEntity::AnatomicalPolarity,
        );
    }

    #[test]
    fn test_voltage_gradient_maps_to_anterior_posterior_axis() {
        assert_eq!(
            BioelectricToRegeneration::map_object(&BioelectricEntity::VoltageGradient),
            RegenerationEntity::AnteriorPosteriorAxis,
        );
    }

    #[test]
    fn test_bioelectric_prepattern_maps_to_target_morphology() {
        assert_eq!(
            BioelectricToRegeneration::map_object(&BioelectricEntity::BioelectricPrepattern),
            RegenerationEntity::TargetMorphology,
        );
    }

    #[test]
    fn test_gap_junction_network_maps_to_pattern_memory() {
        assert_eq!(
            BioelectricToRegeneration::map_object(&BioelectricEntity::GapJunctionNetwork),
            RegenerationEntity::PatternMemory,
        );
    }

    #[test]
    fn test_cognitive_lightcone_maps_to_pattern_memory() {
        assert_eq!(
            BioelectricToRegeneration::map_object(&BioelectricEntity::CognitiveLightcone),
            RegenerationEntity::PatternMemory,
        );
    }

    #[test]
    fn test_target_morphology_maps_directly() {
        assert_eq!(
            BioelectricToRegeneration::map_object(&BioelectricEntity::TargetMorphology),
            RegenerationEntity::TargetMorphology,
        );
    }

    #[test]
    fn test_current_morphology_maps_to_wound_epithelium() {
        assert_eq!(
            BioelectricToRegeneration::map_object(&BioelectricEntity::CurrentMorphology),
            RegenerationEntity::WoundEpithelium,
        );
    }

    #[test]
    fn test_morphogenetic_field_maps_to_pattern_memory() {
        assert_eq!(
            BioelectricToRegeneration::map_object(&BioelectricEntity::MorphogeneticField),
            RegenerationEntity::PatternMemory,
        );
    }

    #[test]
    fn test_ion_channel_modulation_maps_to_bistability() {
        assert_eq!(
            BioelectricToRegeneration::map_object(&BioelectricEntity::IonChannelModulation),
            RegenerationEntity::Bistability,
        );
    }

    #[test]
    fn test_signal_abstract_maps_to_pattern_concept() {
        assert_eq!(
            BioelectricToRegeneration::map_object(&BioelectricEntity::Signal),
            RegenerationEntity::PatternConcept,
        );
    }
}

//! Tests for functor composition across domains.
//!
//! Verifies that chaining functors between domains produces well-defined
//! composed maps that preserve identity and composition laws.

#[cfg(test)]
mod tests {
    use pr4xis::category::{Category, Entity, Functor};

    // -----------------------------------------------------------------------
    // Test 1: pharmacology -> molecular -> bioelectricity
    // -----------------------------------------------------------------------

    #[test]
    fn test_pharma_to_molecular_to_bioelectric_composition() {
        use crate::science::biomedical::bioelectricity::ontology::*;
        use crate::science::biomedical::molecular::bioelectricity_functor::MolecularToBioelectric;
        use crate::science::biomedical::pharmacology::molecular_functor::PharmacologyToMolecular;
        use crate::science::biomedical::pharmacology::ontology::*;

        // For every pharmacology entity, the composed map should be well-defined
        for entity in PharmacologyEntity::variants() {
            let molecular = PharmacologyToMolecular::map_object(&entity);
            let bioelectric = MolecularToBioelectric::map_object(&molecular);
            // Verify the composed map exists and maps to a valid entity
            assert!(
                BioelectricEntity::variants().contains(&bioelectric),
                "pharma->mol->bioelectric: {:?} -> {:?} -> {:?} is not a valid BioelectricEntity",
                entity,
                molecular,
                bioelectric
            );
        }

        // Verify composition preserves identity
        for entity in PharmacologyEntity::variants() {
            let id_pharma = PharmacologyCategory::identity(&entity);
            let mapped_once = PharmacologyToMolecular::map_morphism(&id_pharma);
            let mapped_twice = MolecularToBioelectric::map_morphism(&mapped_once);
            let direct_id = BioelectricCategory::identity(&MolecularToBioelectric::map_object(
                &PharmacologyToMolecular::map_object(&entity),
            ));
            assert_eq!(
                mapped_twice, direct_id,
                "composition identity failed for {:?}",
                entity
            );
        }

        // Verify a sample morphism composition
        let objs = PharmacologyEntity::variants();
        let a = objs[0];
        let b = objs[5];
        let c = objs[10];
        let f = PharmacologyRelation { from: a, to: b };
        let g = PharmacologyRelation { from: b, to: c };
        let composed = PharmacologyCategory::compose(&f, &g).unwrap();
        // Map composed morphism through both functors
        let mapped_composed =
            MolecularToBioelectric::map_morphism(&PharmacologyToMolecular::map_morphism(&composed));
        // Map individual morphisms and compose in target
        let f_mapped =
            MolecularToBioelectric::map_morphism(&PharmacologyToMolecular::map_morphism(&f));
        let g_mapped =
            MolecularToBioelectric::map_morphism(&PharmacologyToMolecular::map_morphism(&g));
        let composed_mapped = BioelectricCategory::compose(&f_mapped, &g_mapped).unwrap();
        assert_eq!(
            mapped_composed, composed_mapped,
            "pharma->mol->bioelectric morphism composition failed"
        );
    }

    // -----------------------------------------------------------------------
    // Test 2: pharmacology -> immunology -> biology
    // -----------------------------------------------------------------------

    #[test]
    fn test_pharma_to_immunology_to_biology_composition() {
        use crate::science::biomedical::biology::ontology::*;
        use crate::science::biomedical::immunology::biology_functor::ImmunologyToBiology;
        use crate::science::biomedical::pharmacology::immunology_functor::PharmacologyToImmunology;
        use crate::science::biomedical::pharmacology::ontology::*;

        // For every pharmacology entity, the composed map should be well-defined
        for entity in PharmacologyEntity::variants() {
            let immunology = PharmacologyToImmunology::map_object(&entity);
            let biology = ImmunologyToBiology::map_object(&immunology);
            assert!(
                BiologicalEntity::variants().contains(&biology),
                "pharma->immuno->bio: {:?} -> {:?} -> {:?} is not valid",
                entity,
                immunology,
                biology
            );
        }

        // Verify composition preserves identity
        for entity in PharmacologyEntity::variants() {
            let id_pharma = PharmacologyCategory::identity(&entity);
            let mapped_once = PharmacologyToImmunology::map_morphism(&id_pharma);
            let mapped_twice = ImmunologyToBiology::map_morphism(&mapped_once);
            let direct_id = BiologyCategory::identity(&ImmunologyToBiology::map_object(
                &PharmacologyToImmunology::map_object(&entity),
            ));
            assert_eq!(
                mapped_twice, direct_id,
                "composition identity failed for {:?}",
                entity
            );
        }

        // Verify a sample morphism composition
        let objs = PharmacologyEntity::variants();
        let a = objs[0];
        let b = objs[5];
        let c = objs[10];
        let f = PharmacologyRelation { from: a, to: b };
        let g = PharmacologyRelation { from: b, to: c };
        let composed = PharmacologyCategory::compose(&f, &g).unwrap();
        let mapped_composed =
            ImmunologyToBiology::map_morphism(&PharmacologyToImmunology::map_morphism(&composed));
        let f_mapped =
            ImmunologyToBiology::map_morphism(&PharmacologyToImmunology::map_morphism(&f));
        let g_mapped =
            ImmunologyToBiology::map_morphism(&PharmacologyToImmunology::map_morphism(&g));
        let composed_mapped = BiologyCategory::compose(&f_mapped, &g_mapped).unwrap();
        assert_eq!(
            mapped_composed, composed_mapped,
            "pharma->immuno->biology morphism composition failed"
        );
    }

    // -----------------------------------------------------------------------
    // Test 3: pharmacology -> immunology -> bioelectricity
    // -----------------------------------------------------------------------

    #[test]
    fn test_pharma_to_immunology_to_bioelectric_composition() {
        use crate::science::biomedical::bioelectricity::ontology::*;
        use crate::science::biomedical::immunology::bioelectricity_functor::ImmunologyToBioelectric;
        use crate::science::biomedical::pharmacology::immunology_functor::PharmacologyToImmunology;
        use crate::science::biomedical::pharmacology::ontology::*;

        // For every pharmacology entity, the composed map should be well-defined
        for entity in PharmacologyEntity::variants() {
            let immunology = PharmacologyToImmunology::map_object(&entity);
            let bioelectric = ImmunologyToBioelectric::map_object(&immunology);
            assert!(
                BioelectricEntity::variants().contains(&bioelectric),
                "pharma->immuno->bioelectric: {:?} -> {:?} -> {:?} is not valid",
                entity,
                immunology,
                bioelectric
            );
        }

        // Verify composition preserves identity
        for entity in PharmacologyEntity::variants() {
            let id_pharma = PharmacologyCategory::identity(&entity);
            let mapped_once = PharmacologyToImmunology::map_morphism(&id_pharma);
            let mapped_twice = ImmunologyToBioelectric::map_morphism(&mapped_once);
            let direct_id = BioelectricCategory::identity(&ImmunologyToBioelectric::map_object(
                &PharmacologyToImmunology::map_object(&entity),
            ));
            assert_eq!(
                mapped_twice, direct_id,
                "composition identity failed for {:?}",
                entity
            );
        }

        // Verify a sample morphism composition
        let objs = PharmacologyEntity::variants();
        let a = objs[1];
        let b = objs[6];
        let c = objs[12];
        let f = PharmacologyRelation { from: a, to: b };
        let g = PharmacologyRelation { from: b, to: c };
        let composed = PharmacologyCategory::compose(&f, &g).unwrap();
        let mapped_composed = ImmunologyToBioelectric::map_morphism(
            &PharmacologyToImmunology::map_morphism(&composed),
        );
        let f_mapped =
            ImmunologyToBioelectric::map_morphism(&PharmacologyToImmunology::map_morphism(&f));
        let g_mapped =
            ImmunologyToBioelectric::map_morphism(&PharmacologyToImmunology::map_morphism(&g));
        let composed_mapped = BioelectricCategory::compose(&f_mapped, &g_mapped).unwrap();
        assert_eq!(
            mapped_composed, composed_mapped,
            "pharma->immuno->bioelectric morphism composition failed"
        );
    }

    // -----------------------------------------------------------------------
    // Test 4: bioelectricity -> regeneration -> biology
    // -----------------------------------------------------------------------

    #[test]
    fn test_bioelectric_to_regeneration_to_biology_composition() {
        use crate::science::biomedical::bioelectricity::ontology::*;
        use crate::science::biomedical::bioelectricity::regeneration_functor::BioelectricToRegeneration;
        use crate::science::biomedical::biology::ontology::*;
        use crate::science::biomedical::regeneration::biology_functor::RegenerationToBiology;

        // For every bioelectric entity, the composed map should be well-defined
        for entity in BioelectricEntity::variants() {
            let regeneration = BioelectricToRegeneration::map_object(&entity);
            let biology = RegenerationToBiology::map_object(&regeneration);
            assert!(
                BiologicalEntity::variants().contains(&biology),
                "bioelectric->regen->bio: {:?} -> {:?} -> {:?} is not valid",
                entity,
                regeneration,
                biology
            );
        }

        // Verify composition preserves identity
        for entity in BioelectricEntity::variants() {
            let id_be = BioelectricCategory::identity(&entity);
            let mapped_once = BioelectricToRegeneration::map_morphism(&id_be);
            let mapped_twice = RegenerationToBiology::map_morphism(&mapped_once);
            let direct_id = BiologyCategory::identity(&RegenerationToBiology::map_object(
                &BioelectricToRegeneration::map_object(&entity),
            ));
            assert_eq!(
                mapped_twice, direct_id,
                "composition identity failed for {:?}",
                entity
            );
        }

        // Verify a sample morphism composition
        let objs = BioelectricEntity::variants();
        let a = objs[0];
        let b = objs[5];
        let c = objs[10];
        let f = BioelectricRelation { from: a, to: b };
        let g = BioelectricRelation { from: b, to: c };
        let composed = BioelectricCategory::compose(&f, &g).unwrap();
        let mapped_composed = RegenerationToBiology::map_morphism(
            &BioelectricToRegeneration::map_morphism(&composed),
        );
        let f_mapped =
            RegenerationToBiology::map_morphism(&BioelectricToRegeneration::map_morphism(&f));
        let g_mapped =
            RegenerationToBiology::map_morphism(&BioelectricToRegeneration::map_morphism(&g));
        let composed_mapped = BiologyCategory::compose(&f_mapped, &g_mapped).unwrap();
        assert_eq!(
            mapped_composed, composed_mapped,
            "bioelectric->regen->biology morphism composition failed"
        );
    }

    // -----------------------------------------------------------------------
    // Test 5: electrophysiology -> bioelectricity -> regeneration (3-step)
    // -----------------------------------------------------------------------

    #[test]
    fn test_electrophysiology_to_bioelectric_to_regeneration_composition() {
        use crate::science::biomedical::bioelectricity::regeneration_functor::BioelectricToRegeneration;
        use crate::science::biomedical::electrophysiology::bioelectricity_functor::ElectrophysiologyToBioelectric;
        use crate::science::biomedical::electrophysiology::ontology::*;
        use crate::science::biomedical::regeneration::ontology::*;

        // For every electrophysiology entity, the composed map should be well-defined
        for entity in ElectrophysiologyEntity::variants() {
            let bioelectric = ElectrophysiologyToBioelectric::map_object(&entity);
            let regeneration = BioelectricToRegeneration::map_object(&bioelectric);
            assert!(
                RegenerationEntity::variants().contains(&regeneration),
                "ephys->bioelectric->regen: {:?} -> {:?} -> {:?} is not valid",
                entity,
                bioelectric,
                regeneration
            );
        }

        // Verify composition preserves identity
        for entity in ElectrophysiologyEntity::variants() {
            let id_ephys = ElectrophysiologyCategory::identity(&entity);
            let mapped_once = ElectrophysiologyToBioelectric::map_morphism(&id_ephys);
            let mapped_twice = BioelectricToRegeneration::map_morphism(&mapped_once);
            let direct_id = RegenerationCategory::identity(&BioelectricToRegeneration::map_object(
                &ElectrophysiologyToBioelectric::map_object(&entity),
            ));
            assert_eq!(
                mapped_twice, direct_id,
                "composition identity failed for {:?}",
                entity
            );
        }

        // Verify a sample morphism composition
        let objs = ElectrophysiologyEntity::variants();
        let a = objs[0];
        let b = objs[5];
        let c = objs[10];
        let f = ElectrophysiologyRelation { from: a, to: b };
        let g = ElectrophysiologyRelation { from: b, to: c };
        let composed = ElectrophysiologyCategory::compose(&f, &g).unwrap();
        let mapped_composed = BioelectricToRegeneration::map_morphism(
            &ElectrophysiologyToBioelectric::map_morphism(&composed),
        );
        let f_mapped = BioelectricToRegeneration::map_morphism(
            &ElectrophysiologyToBioelectric::map_morphism(&f),
        );
        let g_mapped = BioelectricToRegeneration::map_morphism(
            &ElectrophysiologyToBioelectric::map_morphism(&g),
        );
        let composed_mapped = RegenerationCategory::compose(&f_mapped, &g_mapped).unwrap();
        assert_eq!(
            mapped_composed, composed_mapped,
            "ephys->bioelectric->regen morphism composition failed"
        );
    }
}

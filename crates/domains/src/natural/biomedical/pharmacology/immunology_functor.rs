//! Functor: PharmacologyCategory -> ImmunologyCategory.
//!
//! Proves that pharmacological entities have a structure-preserving map to
//! immunological outcomes. Each drug maps through its immune effect:
//! AntiInflammatory -> Resolution (anti-inflammatory drugs resolve inflammation),
//! Hyperpolarization -> TissueRepair (healthy cells are polarized),
//! Depolarization -> AcuteInflammation (depolarized = inflamed), etc.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Category, Functor, Relationship};

use crate::natural::biomedical::immunology::ontology::{
    ImmunologyCategory, ImmunologyCategoryRelationKind, ImmunologyEntity, ImmunologyRelation,
};
use crate::natural::biomedical::pharmacology::ontology::{
    PharmacologyCategory, PharmacologyCategoryRelationKind, PharmacologyEntity,
    PharmacologyRelation,
};

/// Structure-preserving map from pharmacology entities to their immunological outcomes.
pub struct PharmacologyToImmunology;

impl Functor for PharmacologyToImmunology {
    type Source = PharmacologyCategory;
    type Target = ImmunologyCategory;

    fn map_object(obj: &PharmacologyEntity) -> ImmunologyEntity {
        use ImmunologyEntity as I;
        use PharmacologyEntity as P;
        match obj {
            // Effects -> immunological outcomes
            P::AntiInflammatory => I::Resolution, // anti-inflammatory drugs resolve inflammation
            P::Hyperpolarization => I::TissueRepair, // healthy cells are polarized
            P::Depolarization => I::AcuteInflammation, // depolarized = inflamed
            P::GapJunctionOpening => I::TissueRepair, // GJ opening promotes repair
            P::GapJunctionClosing => I::ChronicInflammation, // GJ closing -> chronic state

            // Specific agents -> map through their effects
            P::Ivermectin => I::TissueRepair, // hyperpolarizing -> repair
            P::Minoxidil => I::TissueRepair,  // hyperpolarizing -> repair
            P::Decamethonium => I::AcuteInflammation, // depolarizing -> inflammation
            P::Glibenclamide => I::AcuteInflammation, // depolarizing -> inflammation
            P::Omeprazole => I::Resolution,   // PPI removes damage source

            // Drug classes
            P::IonChannelModulator => I::ImmuneCell, // modulates immune cells
            P::GapJunctionModulator => I::ImmuneCell, // modulates cells via GJ
            P::VoltageGatedBlocker => I::ImmuneCell, // targets immune cell channels
            P::VoltageGatedOpener => I::ImmuneCell,  // targets immune cell channels
            P::MechanosensitiveModulator => I::ImmuneCell, // modulates mechanosensitive cells
            P::ProtonPumpInhibitor => I::Resolution, // removes damage source
            P::Morphoceutical => I::TissueRepair,    // targets anatomy -> repair

            // Targets -> ImmuneCell
            P::IonChannel | P::GapJunction | P::Transporter | P::Receptor => I::ImmuneCell,

            // Abstract categories
            P::DrugClass => I::ImmuneCell,
            P::Agent => I::ImmuneCell,
            P::Target => I::ImmuneCell,
            P::Effect => I::InflammatoryState,
        }
    }

    fn map_morphism(m: &PharmacologyRelation) -> ImmunologyRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind {
            PharmacologyCategoryRelationKind::Identity => ImmunologyCategory::identity(&from),
            _ => ImmunologyRelation {
                from,
                to,
                kind: ImmunologyCategoryRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(PharmacologyToImmunology);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<PharmacologyToImmunology>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<PharmacologyToImmunology>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in PharmacologyEntity::variants() {
            let id_src = PharmacologyCategory::identity(&obj);
            let mapped_id = PharmacologyToImmunology::map_morphism(&id_src);
            let id_tgt = ImmunologyCategory::identity(&PharmacologyToImmunology::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        let objs = PharmacologyEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = PharmacologyRelation {
                        from: a,
                        to: b,
                        kind: PharmacologyCategoryRelationKind::Composed,
                    };
                    let g = PharmacologyRelation {
                        from: b,
                        to: c,
                        kind: PharmacologyCategoryRelationKind::Composed,
                    };
                    let composed = PharmacologyCategory::compose(&f, &g).unwrap();
                    let mapped_composed = PharmacologyToImmunology::map_morphism(&composed);
                    let composed_mapped = ImmunologyCategory::compose(
                        &PharmacologyToImmunology::map_morphism(&f),
                        &PharmacologyToImmunology::map_morphism(&g),
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
        let target_variants = ImmunologyEntity::variants();
        for obj in PharmacologyEntity::variants() {
            let mapped = PharmacologyToImmunology::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid ImmunologyEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_anti_inflammatory_maps_to_resolution() {
        assert_eq!(
            PharmacologyToImmunology::map_object(&PharmacologyEntity::AntiInflammatory),
            ImmunologyEntity::Resolution,
        );
    }

    #[test]
    fn test_hyperpolarization_maps_to_tissue_repair() {
        assert_eq!(
            PharmacologyToImmunology::map_object(&PharmacologyEntity::Hyperpolarization),
            ImmunologyEntity::TissueRepair,
        );
    }

    #[test]
    fn test_depolarization_maps_to_acute_inflammation() {
        assert_eq!(
            PharmacologyToImmunology::map_object(&PharmacologyEntity::Depolarization),
            ImmunologyEntity::AcuteInflammation,
        );
    }

    #[test]
    fn test_ion_channel_modulator_maps_to_immune_cell() {
        assert_eq!(
            PharmacologyToImmunology::map_object(&PharmacologyEntity::IonChannelModulator),
            ImmunologyEntity::ImmuneCell,
        );
    }

    #[test]
    fn test_morphoceutical_maps_to_tissue_repair() {
        assert_eq!(
            PharmacologyToImmunology::map_object(&PharmacologyEntity::Morphoceutical),
            ImmunologyEntity::TissueRepair,
        );
    }

    #[test]
    fn test_proton_pump_inhibitor_maps_to_resolution() {
        assert_eq!(
            PharmacologyToImmunology::map_object(&PharmacologyEntity::ProtonPumpInhibitor),
            ImmunologyEntity::Resolution,
        );
    }

    #[test]
    fn test_gap_junction_opening_maps_to_tissue_repair() {
        assert_eq!(
            PharmacologyToImmunology::map_object(&PharmacologyEntity::GapJunctionOpening),
            ImmunologyEntity::TissueRepair,
        );
    }

    #[test]
    fn test_gap_junction_closing_maps_to_chronic_inflammation() {
        assert_eq!(
            PharmacologyToImmunology::map_object(&PharmacologyEntity::GapJunctionClosing),
            ImmunologyEntity::ChronicInflammation,
        );
    }

    #[test]
    fn test_ivermectin_maps_to_tissue_repair() {
        assert_eq!(
            PharmacologyToImmunology::map_object(&PharmacologyEntity::Ivermectin),
            ImmunologyEntity::TissueRepair,
        );
    }

    #[test]
    fn test_decamethonium_maps_to_acute_inflammation() {
        assert_eq!(
            PharmacologyToImmunology::map_object(&PharmacologyEntity::Decamethonium),
            ImmunologyEntity::AcuteInflammation,
        );
    }

    #[test]
    fn test_analogy_translates_anti_inflammatory() {
        assert_eq!(
            Analogy::<PharmacologyToImmunology>::translate(&PharmacologyEntity::AntiInflammatory),
            ImmunologyEntity::Resolution,
        );
    }
}

//! Functor: MolecularCategory -> BioelectricCategory.
//!
//! Proves that the molecular biology domain has a structure-preserving map into
//! the bioelectric framework. Each molecular entity maps to the bioelectric role
//! it plays: mechanosensitive channels ARE the mechanism of MechanicalStimulation,
//! voltage-gated channels are modulated (IonChannelModulation), connexins modulate
//! gap junctions (GapJunctionModulation), calcium is a bioelectric signal, etc.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Category, Functor, Relationship};

use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation, BioelectricRelationKind,
};
use crate::natural::biomedical::molecular::ontology::{
    MolecularCategory, MolecularCategoryRelationKind, MolecularEntity, MolecularRelation,
};

/// Structure-preserving map from molecular entities to their bioelectric role.
pub struct MolecularToBioelectric;

impl Functor for MolecularToBioelectric {
    type Source = MolecularCategory;
    type Target = BioelectricCategory;

    fn map_object(obj: &MolecularEntity) -> BioelectricEntity {
        use BioelectricEntity::*;
        use MolecularEntity as M;
        match obj {
            // Mechanosensitive channels ARE mechanical stimulation
            M::Piezo1 | M::Piezo2 | M::TRPV4 => MechanicalStimulation,
            // Voltage-gated channels are modulated
            M::Nav | M::Kv | M::Cav => IonChannelModulation,
            // Ligand-gated channels used for modulation
            M::GlyR | M::GABA_A => IonChannelModulation,
            // Connexins are gap junction modulators
            M::Cx26 | M::Cx43 => GapJunctionModulation,
            // Calcium is a bioelectric signal
            M::Calcium | M::CalciumSignal => Signal,
            // Collagen piezoelectricity = mechanical -> bioelectric
            M::Collagen => MechanicalStimulation,
            // All other ions/proteins participate in signaling
            M::Sodium | M::Potassium | M::Chloride | M::Proton => Signal,
            M::Mucin | M::NitricOxide => Signal,
            // Abstract categories -> Signal
            M::Ion
            | M::IonChannel
            | M::VoltageGated
            | M::Mechanosensitive
            | M::LigandGated
            | M::GapJunction
            | M::Protein
            | M::SignalingMolecule => Signal,
        }
    }

    fn map_morphism(m: &MolecularRelation) -> BioelectricRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity morphisms must map to identity (functor law). Other kinds
        // collapse to Composed in the target — matching how the target's
        // compose produces Composed morphisms for non-Identity inputs (so
        // F(g∘f) == F(g)∘F(f) holds under collapse).
        match m.kind {
            MolecularCategoryRelationKind::Identity => BioelectricCategory::identity(&from),
            _ => BioelectricRelation {
                from,
                to,
                kind: BioelectricRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(MolecularToBioelectric);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<MolecularToBioelectric>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<MolecularToBioelectric>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        // For every molecular entity, mapping the identity morphism should yield
        // the identity morphism on the mapped object.
        for obj in MolecularEntity::variants() {
            let id_src = MolecularCategory::identity(&obj);
            let mapped_id = MolecularToBioelectric::map_morphism(&id_src);
            let id_tgt = BioelectricCategory::identity(&MolecularToBioelectric::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        // For composable morphism pairs, mapping the composition must equal
        // composing the mapped morphisms.
        let objs = MolecularEntity::variants();
        // Test a representative sample (full product is large)
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = MolecularRelation {
                        from: a,
                        to: b,
                        kind: MolecularCategoryRelationKind::Composed,
                    };
                    let g = MolecularRelation {
                        from: b,
                        to: c,
                        kind: MolecularCategoryRelationKind::Composed,
                    };
                    let composed = MolecularCategory::compose(&f, &g).unwrap();
                    let mapped_composed = MolecularToBioelectric::map_morphism(&composed);
                    let composed_mapped = BioelectricCategory::compose(
                        &MolecularToBioelectric::map_morphism(&f),
                        &MolecularToBioelectric::map_morphism(&g),
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
        for obj in MolecularEntity::variants() {
            let mapped = MolecularToBioelectric::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BioelectricEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_piezo1_maps_to_mechanical_stimulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Piezo1),
            BioelectricEntity::MechanicalStimulation,
        );
    }

    #[test]
    fn test_piezo2_maps_to_mechanical_stimulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Piezo2),
            BioelectricEntity::MechanicalStimulation,
        );
    }

    #[test]
    fn test_trpv4_maps_to_mechanical_stimulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::TRPV4),
            BioelectricEntity::MechanicalStimulation,
        );
    }

    #[test]
    fn test_nav_maps_to_ion_channel_modulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Nav),
            BioelectricEntity::IonChannelModulation,
        );
    }

    #[test]
    fn test_glyr_maps_to_ion_channel_modulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::GlyR),
            BioelectricEntity::IonChannelModulation,
        );
    }

    #[test]
    fn test_cx26_maps_to_gap_junction_modulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Cx26),
            BioelectricEntity::GapJunctionModulation,
        );
    }

    #[test]
    fn test_cx43_maps_to_gap_junction_modulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Cx43),
            BioelectricEntity::GapJunctionModulation,
        );
    }

    #[test]
    fn test_calcium_maps_to_signal() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Calcium),
            BioelectricEntity::Signal,
        );
    }

    #[test]
    fn test_collagen_maps_to_mechanical_stimulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Collagen),
            BioelectricEntity::MechanicalStimulation,
        );
    }
}

//! Functor: MechanobiologyCategory -> MolecularCategory.
//!
//! Proves that mechanobiology has a structure-preserving map into molecular biology.
//! Each mechanobiological entity maps to its molecular substrate:
//! MechanosensitiveChannel -> Piezo1, CalciumTransient -> Calcium,
//! MembraneTension -> Mechanosensitive, SubstrateStiffness -> Collagen, etc.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Arrow, Category, Functor};

use crate::natural::biomedical::mechanobiology::ontology::{
    MechanobiologyCategory, MechanobiologyEntity, MechanobiologyRelation,
    MechanobiologyRelationKind,
};
use crate::natural::biomedical::molecular::ontology::{
    MolecularCategory, MolecularEntity, MolecularRelation, MolecularRelationKind,
};

/// Structure-preserving map from mechanobiology entities to their molecular substrate.
pub struct MechanobiologyToMolecular;

impl Functor for MechanobiologyToMolecular {
    type Source = MechanobiologyCategory;
    type Target = MolecularCategory;

    fn map_object(obj: &MechanobiologyEntity) -> MolecularEntity {
        use MechanobiologyEntity as MB;
        use MolecularEntity as M;
        match obj {
            // Forces -> Mechanosensitive (forces activate mechanosensors)
            MB::MembraneTension => M::Mechanosensitive,
            MB::ShearStress => M::Mechanosensitive,
            MB::CompressiveStress => M::Mechanosensitive,
            MB::TensileStress => M::Mechanosensitive,
            MB::SubstrateStiffness => M::Collagen, // ECM stiffness = collagen content

            // Mechanotransduction -> Piezo1 (canonical mechanosensitive channel)
            MB::MechanosensitiveChannel => M::Piezo1,
            MB::ChannelConformation => M::Piezo1,
            MB::OpenState => M::Piezo1,
            MB::ClosedState => M::Piezo1,
            MB::InactivatedState => M::Piezo1,

            // Frequency response -> Mechanosensitive (frequency filtering is a
            // property of mechanosensitive channels, Lewis 2017)
            MB::FrequencyFiltering => M::Mechanosensitive,
            MB::ActivationThreshold => M::Mechanosensitive,
            MB::InactivationKinetics => M::Mechanosensitive,
            MB::RecoveryTime => M::Mechanosensitive,

            // Cellular responses
            MB::CalciumTransient => M::Calcium,
            MB::CytoskeletalRemodeling => M::Protein,
            MB::FocalAdhesion => M::Protein,
            MB::Mechanoadaptation => M::Protein,

            // Abstract categories
            MB::MechanicalForce => M::Mechanosensitive,
            MB::ChannelState => M::Piezo1,
            MB::FrequencyProperty => M::Mechanosensitive,
            MB::CellularResponse => M::Calcium,
            MB::MechanobiologyEvent => M::Mechanosensitive,

            // Causal events (merged into the concept enum): each step maps
            // to the molecular substrate involved.
            MB::MechanicalLoad => M::Mechanosensitive,
            MB::MembraneDeformation => M::Mechanosensitive,
            MB::ChannelGating => M::Piezo1,
            MB::IonInflux => M::Calcium,
            MB::IntracellularSignaling => M::Calcium,
            MB::RepetitiveStimulus => M::Mechanosensitive,
            MB::ChannelInactivation => M::Piezo1,
            MB::FrequencyDependentResponse => M::Piezo1,
            MB::SustainedForce => M::Mechanosensitive,
            MB::ThresholdShift => M::Mechanosensitive,
        }
    }

    fn map_morphism(m: &MechanobiologyRelation) -> MolecularRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity preserved; non-Identity kinds collapse to Subsumption in
        // the (migrated) molecular target so functor laws hold under
        // same-kind transitive composition (#166).
        match m.kind {
            MechanobiologyRelationKind::Identity => MolecularCategory::identity(&from),
            _ => MolecularRelation {
                from,
                to,
                kind: MolecularRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(MechanobiologyToMolecular);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        assert_functor_laws::<MechanobiologyToMolecular>();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<MechanobiologyToMolecular>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in MechanobiologyEntity::variants() {
            let id_src = MechanobiologyCategory::identity(&obj);
            let mapped_id = MechanobiologyToMolecular::map_morphism(&id_src);
            let id_tgt = MolecularCategory::identity(&MechanobiologyToMolecular::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    /// Composition preservation over a Causation chain that actually
    /// composes: MechanicalLoad -> MembraneDeformation -> ChannelGating
    /// compose under Causation-transitivity.
    #[test]
    fn test_composition_preservation_causation_chain() {
        let f = MechanobiologyRelation {
            from: MechanobiologyEntity::MechanicalLoad,
            to: MechanobiologyEntity::MembraneDeformation,
            kind: MechanobiologyRelationKind::Causation,
        };
        let g = MechanobiologyRelation {
            from: MechanobiologyEntity::MembraneDeformation,
            to: MechanobiologyEntity::ChannelGating,
            kind: MechanobiologyRelationKind::Causation,
        };
        let composed = MechanobiologyCategory::compose(&f, &g)
            .expect("Causation chain must compose under transitive same-kind inheritance");
        let mapped_composed = MechanobiologyToMolecular::map_morphism(&composed);
        let composed_mapped = MolecularCategory::compose(
            &MechanobiologyToMolecular::map_morphism(&f),
            &MechanobiologyToMolecular::map_morphism(&g),
        )
        .expect("functor-mapped morphisms must compose in the target category");
        assert_eq!(mapped_composed, composed_mapped);
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = MolecularEntity::variants();
        for obj in MechanobiologyEntity::variants() {
            let mapped = MechanobiologyToMolecular::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid MolecularEntity",
                obj,
                mapped
            );
        }
    }

    // -- Specific mapping tests --

    #[test]
    fn test_mechanosensitive_channel_maps_to_piezo1() {
        assert_eq!(
            MechanobiologyToMolecular::map_object(&MechanobiologyEntity::MechanosensitiveChannel),
            MolecularEntity::Piezo1,
        );
    }

    #[test]
    fn test_calcium_transient_maps_to_calcium() {
        assert_eq!(
            MechanobiologyToMolecular::map_object(&MechanobiologyEntity::CalciumTransient),
            MolecularEntity::Calcium,
        );
    }

    #[test]
    fn test_membrane_tension_maps_to_mechanosensitive() {
        assert_eq!(
            MechanobiologyToMolecular::map_object(&MechanobiologyEntity::MembraneTension),
            MolecularEntity::Mechanosensitive,
        );
    }

    #[test]
    fn test_substrate_stiffness_maps_to_collagen() {
        assert_eq!(
            MechanobiologyToMolecular::map_object(&MechanobiologyEntity::SubstrateStiffness),
            MolecularEntity::Collagen,
        );
    }

    #[test]
    fn test_open_state_maps_to_piezo1() {
        assert_eq!(
            MechanobiologyToMolecular::map_object(&MechanobiologyEntity::OpenState),
            MolecularEntity::Piezo1,
        );
    }

    #[test]
    fn test_closed_state_maps_to_piezo1() {
        assert_eq!(
            MechanobiologyToMolecular::map_object(&MechanobiologyEntity::ClosedState),
            MolecularEntity::Piezo1,
        );
    }

    #[test]
    fn test_focal_adhesion_maps_to_protein() {
        assert_eq!(
            MechanobiologyToMolecular::map_object(&MechanobiologyEntity::FocalAdhesion),
            MolecularEntity::Protein,
        );
    }

    #[test]
    fn test_frequency_filtering_maps_to_mechanosensitive() {
        assert_eq!(
            MechanobiologyToMolecular::map_object(&MechanobiologyEntity::FrequencyFiltering),
            MolecularEntity::Mechanosensitive,
        );
    }

    #[test]
    fn test_channel_state_maps_to_piezo1() {
        assert_eq!(
            MechanobiologyToMolecular::map_object(&MechanobiologyEntity::ChannelState),
            MolecularEntity::Piezo1,
        );
    }

    #[test]
    fn test_cellular_response_maps_to_calcium() {
        assert_eq!(
            MechanobiologyToMolecular::map_object(&MechanobiologyEntity::CellularResponse),
            MolecularEntity::Calcium,
        );
    }
}

//! Functor: PharmacologyCategory -> MolecularCategory.
//!
//! Proves that pharmacological entities have a structure-preserving map to their
//! molecular targets. Each drug maps to the molecular entity it acts on:
//! Ivermectin -> GlyR (glycine receptor agonist), Omeprazole -> Proton (proton
//! pump inhibitor), etc.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Arrow, Category, Functor};

use crate::natural::biomedical::molecular::ontology::{
    MolecularCategory, MolecularEntity, MolecularRelation, MolecularRelationKind,
};
use crate::natural::biomedical::pharmacology::ontology::{
    PharmacologyCategory, PharmacologyEntity, PharmacologyRelation, PharmacologyRelationKind,
};

/// Structure-preserving map from pharmacology entities to their molecular targets.
pub struct PharmacologyToMolecular;

impl Functor for PharmacologyToMolecular {
    type Source = PharmacologyCategory;
    type Target = MolecularCategory;

    fn map_object(obj: &PharmacologyEntity) -> MolecularEntity {
        use MolecularEntity as M;
        use PharmacologyEntity as P;
        match obj {
            // Specific agents -> their molecular targets
            P::Ivermectin => M::GlyR,   // GlyR agonist
            P::Omeprazole => M::Proton, // proton pump inhibitor
            P::Minoxidil => M::Kv,      // KATP opener
            P::Glibenclamide => M::Kv,  // KATP blocker
            P::Decamethonium => M::Nav, // depolarizing agent

            // Drug classes -> abstract molecular entities
            P::IonChannelModulator => M::IonChannel,
            P::GapJunctionModulator => M::GapJunction,
            P::VoltageGatedBlocker => M::VoltageGated,
            P::VoltageGatedOpener => M::VoltageGated,
            P::MechanosensitiveModulator => M::Mechanosensitive,
            P::ProtonPumpInhibitor => M::Proton,
            P::Morphoceutical => M::IonChannel, // targets channels for anatomical outcomes

            // Targets -> molecular targets
            P::IonChannel => M::IonChannel,
            P::GapJunction => M::GapJunction,
            P::Transporter => M::Protein, // transporters are proteins
            P::Receptor => M::Protein,    // receptors are proteins

            // Effects -> relevant ions/molecules
            P::Hyperpolarization => M::Potassium, // K+ efflux -> hyperpolarization
            P::Depolarization => M::Sodium,       // Na+ influx -> depolarization
            P::GapJunctionOpening => M::Cx43,     // Cx43 is the major GJ protein
            P::GapJunctionClosing => M::Cx43,
            P::AntiInflammatory => M::NitricOxide, // NO is anti-inflammatory mediator

            // Abstract categories -> abstract molecular entities
            P::DrugClass => M::Protein, // drugs target proteins
            P::Agent => M::Protein,
            P::Target => M::Protein,
            P::Effect => M::Ion, // effects are mediated by ions
            P::PharmacologyEvent => M::Protein,

            // Causal events (merged into the concept enum; map to the
            // molecular substrate involved in each step).
            P::DrugAdministration => M::Protein,
            P::TargetBinding => M::Protein,
            P::ChannelStateChange => M::IonChannel,
            P::IonFluxChange => M::Ion,
            P::VmemShift => M::Ion,
            P::DownstreamSignaling => M::Protein,
            P::GJModulatorBinding => M::Cx43,
            P::GapJunctionStateChange => M::Cx43,
            P::BioelectricNetworkChange => M::GapJunction,
            P::CollectiveReprogramming => M::Protein,
        }
    }

    fn map_morphism(m: &PharmacologyRelation) -> MolecularRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity preserved; non-Identity kinds collapse to Subsumption in
        // the (migrated) molecular target so functor laws hold under
        // same-kind transitive composition (#166).
        match m.kind {
            PharmacologyRelationKind::Identity => MolecularCategory::identity(&from),
            _ => MolecularRelation {
                from,
                to,
                kind: MolecularRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(PharmacologyToMolecular);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        assert_functor_laws::<PharmacologyToMolecular>();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<PharmacologyToMolecular>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in PharmacologyEntity::variants() {
            let id_src = PharmacologyCategory::identity(&obj);
            let mapped_id = PharmacologyToMolecular::map_morphism(&id_src);
            let id_tgt = MolecularCategory::identity(&PharmacologyToMolecular::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    /// Composition preservation over a Subsumption chain that actually
    /// composes in the new partial-category API: Ivermectin -> IonChannelModulator
    /// (declared) and IonChannelModulator -> DrugClass (declared) compose to
    /// Ivermectin -> DrugClass under Subsumption-transitivity.
    #[test]
    fn test_composition_preservation_subsumption_chain() {
        let f = PharmacologyRelation {
            from: PharmacologyEntity::Ivermectin,
            to: PharmacologyEntity::IonChannelModulator,
            kind: PharmacologyRelationKind::Subsumption,
        };
        let g = PharmacologyRelation {
            from: PharmacologyEntity::IonChannelModulator,
            to: PharmacologyEntity::DrugClass,
            kind: PharmacologyRelationKind::Subsumption,
        };
        let composed = PharmacologyCategory::compose(&f, &g)
            .expect("Subsumption chain Ivermectin -> ICM -> DrugClass must compose");
        let mapped_composed = PharmacologyToMolecular::map_morphism(&composed);
        let composed_mapped = MolecularCategory::compose(
            &PharmacologyToMolecular::map_morphism(&f),
            &PharmacologyToMolecular::map_morphism(&g),
        )
        .expect("functor-mapped morphisms must compose in the target category");
        assert_eq!(mapped_composed, composed_mapped);
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = MolecularEntity::variants();
        for obj in PharmacologyEntity::variants() {
            let mapped = PharmacologyToMolecular::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid MolecularEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_ivermectin_maps_to_glyr() {
        assert_eq!(
            PharmacologyToMolecular::map_object(&PharmacologyEntity::Ivermectin),
            MolecularEntity::GlyR,
        );
    }

    #[test]
    fn test_omeprazole_maps_to_proton() {
        assert_eq!(
            PharmacologyToMolecular::map_object(&PharmacologyEntity::Omeprazole),
            MolecularEntity::Proton,
        );
    }

    #[test]
    fn test_minoxidil_maps_to_kv() {
        assert_eq!(
            PharmacologyToMolecular::map_object(&PharmacologyEntity::Minoxidil),
            MolecularEntity::Kv,
        );
    }

    #[test]
    fn test_glibenclamide_maps_to_kv() {
        assert_eq!(
            PharmacologyToMolecular::map_object(&PharmacologyEntity::Glibenclamide),
            MolecularEntity::Kv,
        );
    }

    #[test]
    fn test_decamethonium_maps_to_nav() {
        assert_eq!(
            PharmacologyToMolecular::map_object(&PharmacologyEntity::Decamethonium),
            MolecularEntity::Nav,
        );
    }

    #[test]
    fn test_ion_channel_modulator_maps_to_ion_channel() {
        assert_eq!(
            PharmacologyToMolecular::map_object(&PharmacologyEntity::IonChannelModulator),
            MolecularEntity::IonChannel,
        );
    }

    #[test]
    fn test_gap_junction_modulator_maps_to_gap_junction() {
        assert_eq!(
            PharmacologyToMolecular::map_object(&PharmacologyEntity::GapJunctionModulator),
            MolecularEntity::GapJunction,
        );
    }

    #[test]
    fn test_morphoceutical_maps_to_ion_channel() {
        assert_eq!(
            PharmacologyToMolecular::map_object(&PharmacologyEntity::Morphoceutical),
            MolecularEntity::IonChannel,
        );
    }

    #[test]
    fn test_hyperpolarization_maps_to_potassium() {
        assert_eq!(
            PharmacologyToMolecular::map_object(&PharmacologyEntity::Hyperpolarization),
            MolecularEntity::Potassium,
        );
    }

    #[test]
    fn test_depolarization_maps_to_sodium() {
        assert_eq!(
            PharmacologyToMolecular::map_object(&PharmacologyEntity::Depolarization),
            MolecularEntity::Sodium,
        );
    }
}

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

use pr4xis::category::{Arrow, Category, Functor};

use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation, BioelectricRelationKind,
};
use crate::natural::biomedical::molecular::ontology::{
    MolecularCategory, MolecularEntity, MolecularRelation, MolecularRelationKind,
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

            // Mechanotransduction events (umbrella + Piezo1Opening etc.) —
            // these are dynamic molecular events; in the bioelectric view
            // they map to Signal.
            _ => Signal,
        }
    }

    fn map_morphism(m: &MolecularRelation) -> BioelectricRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity preserved; non-Identity kinds collapse to a single target
        // kind so F(g∘f) = F(g)∘F(f) holds for same-kind transitive composition
        // under #166 (no `Composed` variant in proc-macro-generated enums).
        match m.kind {
            MolecularRelationKind::Identity => BioelectricCategory::identity(&from),
            _ => BioelectricRelation {
                from,
                to,
                kind: BioelectricRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(MolecularToBioelectric);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_functor_laws() {
        assert_functor_laws::<MolecularToBioelectric>();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_analogy_validates() {
        Analogy::<MolecularToBioelectric>::validate().unwrap();
    }

    #[pr4xis::praxis_value(Extensible)]
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

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_composition_preservation_on_subsumption() {
        // The migrated Molecular category is kinded and partial (#166):
        // compose only succeeds for same-kind transitive relations. Exercise
        // composition along Subsumption chains and verify the functor
        // preserves the composite.
        for m in MolecularCategory::morphisms() {
            if m.kind() != MolecularRelationKind::Subsumption {
                continue;
            }
            for n in MolecularCategory::morphisms() {
                if n.kind() != MolecularRelationKind::Subsumption {
                    continue;
                }
                if m.target() != n.source() {
                    continue;
                }
                let composed = match MolecularCategory::compose(&m, &n) {
                    Some(c) => c,
                    None => continue,
                };
                let mapped_composed = MolecularToBioelectric::map_morphism(&composed);
                let composed_mapped = BioelectricCategory::compose(
                    &MolecularToBioelectric::map_morphism(&m),
                    &MolecularToBioelectric::map_morphism(&n),
                )
                .expect("target composition is total for same-kind");
                assert_eq!(
                    mapped_composed, composed_mapped,
                    "composition law failed for {:?} ∘ {:?}",
                    m, n
                );
            }
        }
    }

    #[pr4xis::praxis_value(Extensible)]
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

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_piezo1_maps_to_mechanical_stimulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Piezo1),
            BioelectricEntity::MechanicalStimulation,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_piezo2_maps_to_mechanical_stimulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Piezo2),
            BioelectricEntity::MechanicalStimulation,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_trpv4_maps_to_mechanical_stimulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::TRPV4),
            BioelectricEntity::MechanicalStimulation,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_nav_maps_to_ion_channel_modulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Nav),
            BioelectricEntity::IonChannelModulation,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_glyr_maps_to_ion_channel_modulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::GlyR),
            BioelectricEntity::IonChannelModulation,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_cx26_maps_to_gap_junction_modulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Cx26),
            BioelectricEntity::GapJunctionModulation,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_cx43_maps_to_gap_junction_modulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Cx43),
            BioelectricEntity::GapJunctionModulation,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_calcium_maps_to_signal() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Calcium),
            BioelectricEntity::Signal,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_collagen_maps_to_mechanical_stimulation() {
        assert_eq!(
            MolecularToBioelectric::map_object(&MolecularEntity::Collagen),
            BioelectricEntity::MechanicalStimulation,
        );
    }
}

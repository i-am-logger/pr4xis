//! Functor: BiochemistryCategory -> MolecularCategory.
//!
//! Proves that the biochemistry signaling domain has a structure-preserving map
//! into molecular biology. Signaling molecules map to their molecular counterparts:
//! CalciumIon to Calcium, proteins (Calmodulin/CaMKII/PKC/CREB) to Protein,
//! second messengers (cAMP/IP3) to CalciumSignal, energy carriers (ATP/ADP) to Ion,
//! and processes to their dominant molecular participants.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Functor, Relationship};

use crate::science::biomedical::biochemistry::ontology::{
    BiochemistryCategory, BiochemistryEntity, BiochemistryRelation,
};
use crate::science::biomedical::molecular::ontology::{
    MolecularCategory, MolecularEntity, MolecularRelation,
};

/// Structure-preserving map from biochemistry entities to molecular components.
pub struct BiochemistryToMolecular;

impl Functor for BiochemistryToMolecular {
    type Source = BiochemistryCategory;
    type Target = MolecularCategory;

    fn map_object(obj: &BiochemistryEntity) -> MolecularEntity {
        use BiochemistryEntity as B;
        use MolecularEntity as M;
        match obj {
            // Direct ion mapping
            B::CalciumIon => M::Calcium,

            // Nitric oxide maps directly
            B::NitricOxide => M::NitricOxide,

            // Proteins: calmodulin, CaMKII, PKC, CREB are all proteins
            B::Calmodulin => M::Protein,
            B::CaMKII => M::Protein,
            B::ProteinKinaseC => M::Protein,
            B::CREB => M::Protein,

            // Second messengers cAMP and IP3 are calcium signaling components
            B::CAMP => M::CalciumSignal,
            B::IP3 => M::CalciumSignal,

            // Energy carriers are ions (electrochemical energy)
            B::ATP => M::Ion,
            B::ADP => M::Ion,

            // Processes map to their dominant molecular participants
            B::SignalTransduction => M::SignalingMolecule,
            B::PhosphorylationCascade => M::Protein, // kinases are proteins
            B::GeneTranscription => M::Protein,      // transcription factors are proteins
            B::ProteinSynthesis => M::Protein,       // output is protein
            B::SecondMessenger => M::CalciumSignal,  // second messengers relay signals

            // Metabolic processes
            B::Glycolysis => M::Ion, // produces ionic intermediates
            B::OxidativePhosphorylation => M::Ion, // electron transport chain

            // Abstract categories map to molecular abstracts
            B::SignalingMolecule => M::SignalingMolecule,
            B::BiochemicalProcess => M::Protein, // processes act on proteins
            B::EnergyMetabolite => M::Ion,       // energy currency
        }
    }

    fn map_morphism(m: &BiochemistryRelation) -> MolecularRelation {
        MolecularRelation {
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
        check_functor_laws::<BiochemistryToMolecular>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<BiochemistryToMolecular>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in BiochemistryEntity::variants() {
            let id_src = BiochemistryCategory::identity(&obj);
            let mapped_id = BiochemistryToMolecular::map_morphism(&id_src);
            let id_tgt = MolecularCategory::identity(&BiochemistryToMolecular::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        let objs = BiochemistryEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = BiochemistryRelation { from: a, to: b };
                    let g = BiochemistryRelation { from: b, to: c };
                    let composed = BiochemistryCategory::compose(&f, &g).unwrap();
                    let mapped_composed = BiochemistryToMolecular::map_morphism(&composed);
                    let composed_mapped = MolecularCategory::compose(
                        &BiochemistryToMolecular::map_morphism(&f),
                        &BiochemistryToMolecular::map_morphism(&g),
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
    fn test_calcium_ion_maps_to_calcium() {
        assert_eq!(
            BiochemistryToMolecular::map_object(&BiochemistryEntity::CalciumIon),
            MolecularEntity::Calcium,
        );
    }

    #[test]
    fn test_nitric_oxide_maps_to_nitric_oxide() {
        assert_eq!(
            BiochemistryToMolecular::map_object(&BiochemistryEntity::NitricOxide),
            MolecularEntity::NitricOxide,
        );
    }

    #[test]
    fn test_calmodulin_maps_to_protein() {
        assert_eq!(
            BiochemistryToMolecular::map_object(&BiochemistryEntity::Calmodulin),
            MolecularEntity::Protein,
        );
    }

    #[test]
    fn test_camkii_maps_to_protein() {
        assert_eq!(
            BiochemistryToMolecular::map_object(&BiochemistryEntity::CaMKII),
            MolecularEntity::Protein,
        );
    }

    #[test]
    fn test_camp_maps_to_calcium_signal() {
        assert_eq!(
            BiochemistryToMolecular::map_object(&BiochemistryEntity::CAMP),
            MolecularEntity::CalciumSignal,
        );
    }

    #[test]
    fn test_atp_maps_to_ion() {
        assert_eq!(
            BiochemistryToMolecular::map_object(&BiochemistryEntity::ATP),
            MolecularEntity::Ion,
        );
    }

    #[test]
    fn test_signaling_molecule_maps_to_signaling_molecule() {
        assert_eq!(
            BiochemistryToMolecular::map_object(&BiochemistryEntity::SignalingMolecule),
            MolecularEntity::SignalingMolecule,
        );
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = MolecularEntity::variants();
        for obj in BiochemistryEntity::variants() {
            let mapped = BiochemistryToMolecular::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid MolecularEntity",
                obj,
                mapped
            );
        }
    }
}

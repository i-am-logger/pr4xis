//! Functor: BiochemistryCategory -> BioelectricCategory.
//!
//! Maps biochemical signaling entities to their bioelectric roles.
//! This functor bridges the gap between molecular mechanisms and
//! tissue-level bioelectric phenomena.
//!
//! The hypothesis: routing molecular → biochemistry → bioelectricity
//! (two hops) should have lower information loss than the direct
//! molecular → bioelectricity functor (one hop).

use pr4xis::category::{Category, Functor, Relationship};

use crate::natural::biomedical::biochemistry::ontology::{
    BiochemistryCategory, BiochemistryCategoryRelationKind, BiochemistryEntity,
    BiochemistryRelation,
};
use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation, BioelectricRelationKind,
};

pub struct BiochemistryToBioelectric;

impl Functor for BiochemistryToBioelectric {
    type Source = BiochemistryCategory;
    type Target = BioelectricCategory;

    fn map_object(obj: &BiochemistryEntity) -> BioelectricEntity {
        use BiochemistryEntity as B;
        use BioelectricEntity as E;
        match obj {
            // Calcium ion IS the primary bioelectric signal carrier
            B::CalciumIon => E::MembranePotential,
            // Calmodulin/CaMKII/PKC/CREB = intracellular signal transduction
            // These are the mechanism BEHIND bioelectric pattern formation
            B::Calmodulin => E::Signal,
            B::CaMKII => E::Signal,
            B::ProteinKinaseC => E::Signal,
            B::CREB => E::MorphogeneticField, // CREB = gene expression = morphogenesis
            // Nitric oxide = intercellular signaling (vasodilation, paracrine)
            B::NitricOxide => E::Signal,
            // Second messengers propagate bioelectric information
            B::CAMP => E::Signal,
            B::IP3 => E::MembranePotential, // IP3 releases Ca2+ → Vmem shift
            // Biochemical processes map to bioelectric phenomena
            B::SignalTransduction => E::BioelectricCircuit,
            B::PhosphorylationCascade => E::Signal,
            B::GeneTranscription => E::MorphogeneticField,
            B::ProteinSynthesis => E::TargetMorphology,
            B::SecondMessenger => E::Signal,
            // Energy metabolism
            B::ATP => E::MembranePotential, // ATP drives ion pumps → Vmem
            B::ADP => E::MembranePotential,
            B::Glycolysis => E::CurrentMorphology, // metabolic state = current state
            B::OxidativePhosphorylation => E::CurrentMorphology,
            // Abstract categories
            B::SignalingMolecule => E::Signal,
            B::BiochemicalProcess => E::Network,
            B::EnergyMetabolite => E::Signal,
        }
    }

    fn map_morphism(m: &BiochemistryRelation) -> BioelectricRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity morphisms must map to identity (functor law). Other kinds
        // collapse to Composed in the target — matching how the target's
        // compose produces Composed morphisms for non-Identity inputs (so
        // F(g∘f) == F(g)∘F(f) holds under collapse).
        match m.kind {
            BiochemistryCategoryRelationKind::Identity => BioelectricCategory::identity(&from),
            _ => BioelectricRelation {
                from,
                to,
                kind: BioelectricRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(BiochemistryToBioelectric);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<BiochemistryToBioelectric>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<BiochemistryToBioelectric>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in BiochemistryEntity::variants() {
            let id_src = BiochemistryCategory::identity(&obj);
            let mapped = BiochemistryToBioelectric::map_morphism(&id_src);
            let id_tgt =
                BioelectricCategory::identity(&BiochemistryToBioelectric::map_object(&obj));
            assert_eq!(mapped, id_tgt, "identity failed for {:?}", obj);
        }
    }

    #[test]
    fn test_every_entity_maps() {
        let target_variants = BioelectricEntity::variants();
        for obj in BiochemistryEntity::variants() {
            let mapped = BiochemistryToBioelectric::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to invalid {:?}",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_calcium_maps_to_membrane_potential() {
        assert_eq!(
            BiochemistryToBioelectric::map_object(&BiochemistryEntity::CalciumIon),
            BioelectricEntity::MembranePotential
        );
    }

    #[test]
    fn test_creb_maps_to_morphogenetic_field() {
        assert_eq!(
            BiochemistryToBioelectric::map_object(&BiochemistryEntity::CREB),
            BioelectricEntity::MorphogeneticField
        );
    }

    #[test]
    fn test_gene_transcription_maps_to_morphogenetic_field() {
        assert_eq!(
            BiochemistryToBioelectric::map_object(&BiochemistryEntity::GeneTranscription),
            BioelectricEntity::MorphogeneticField
        );
    }

    #[test]
    fn test_atp_maps_to_membrane_potential() {
        // ATP drives Na/K-ATPase which sets Vmem
        assert_eq!(
            BiochemistryToBioelectric::map_object(&BiochemistryEntity::ATP),
            BioelectricEntity::MembranePotential
        );
    }
}

//! Functor: ChemistryCategory -> MolecularCategory.
//!
//! Proves that foundational chemistry has a structure-preserving map into
//! molecular biology. Electrolytes and ions map to Ion, bonds map to Protein
//! (covalent/hydrogen/VanDerWaals/metallic) or Ion (ionic), states of matter
//! map to their biological molecular constituents, and physical properties
//! map to Ion (electrochemical basis).
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Arrow, Category, Functor};

use crate::natural::biomedical::chemistry::ontology::{
    ChemistryCategory, ChemistryConcept, ChemistryRelation, ChemistryRelationKind,
};
use crate::natural::biomedical::molecular::ontology::{
    MolecularCategory, MolecularEntity, MolecularRelation, MolecularRelationKind,
};

/// Structure-preserving map from chemistry entities to molecular components.
pub struct ChemistryToMolecular;

impl Functor for ChemistryToMolecular {
    type Source = ChemistryCategory;
    type Target = MolecularCategory;

    fn map_object(obj: &ChemistryConcept) -> MolecularEntity {
        use ChemistryConcept as C;
        use MolecularEntity as M;
        match obj {
            // Solution components -> Ion (electrolyte/ionic basis)
            C::Electrolyte => M::Ion,
            C::Buffer => M::Ion,
            C::Solvent => M::Sodium, // water is the ionic medium; Na+ is primary osmolyte
            C::Solute => M::Ion,

            // Bonding -> Protein or Ion
            C::IonicBond => M::Ion,
            C::CovalentBond => M::Protein,
            C::HydrogenBond => M::Protein, // protein folding
            C::VanDerWaals => M::Protein,
            C::Metallic => M::Protein,

            // Physical properties -> Ion (electrochemical basis)
            C::PH => M::Proton,
            C::Concentration => M::Ion,
            C::Osmolarity => M::Sodium, // Na+ is the primary osmolyte
            C::Temperature => M::Ion,
            C::Pressure => M::Ion,

            // States of matter -> biological molecular constituents
            C::Solid => M::Collagen, // structural solid
            C::Liquid => M::Mucin,   // biological fluids
            C::Gel => M::Mucin,      // biological gel
            C::Colloid => M::Mucin,  // biological colloid
            C::Gas => M::Ion,        // dissolved gases as ions
            C::Plasma => M::Calcium, // ionized plasma; Ca2+ is key plasma ion

            // Abstract categories and ChemicalEvent umbrella
            C::StateOfMatter => M::Ion,
            C::ChemicalBond => M::Protein,
            C::PhysicalProperty => M::Ion,
            C::SolutionComponent => M::Ion,
            C::ChemicalEvent => M::Ion,

            // Causal events — merged into the concept enum.
            // Aqueous-phase reactions are predominantly ionic processes.
            C::Dissolution => M::Ion,
            C::IonDissociation => M::Ion,
            C::ElectrolyteFormation => M::Ion,
            C::AcidBaseReaction => M::Proton,
            C::PHChange => M::Proton,
            C::ProteinDenaturation => M::Protein,
            C::TemperatureChange => M::Ion,
            C::PhaseTransition => M::Ion,
            C::ConcentrationGradient => M::Ion,
            C::Diffusion => M::Ion,
        }
    }

    fn map_morphism(m: &ChemistryRelation) -> MolecularRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind {
            ChemistryRelationKind::Identity => MolecularCategory::identity(&from),
            _ => MolecularRelation {
                from,
                to,
                kind: MolecularRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(ChemistryToMolecular);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        assert_functor_laws::<ChemistryToMolecular>();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<ChemistryToMolecular>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in ChemistryConcept::variants() {
            let id_src = ChemistryCategory::identity(&obj);
            let mapped_id = ChemistryToMolecular::map_morphism(&id_src);
            let id_tgt = MolecularCategory::identity(&ChemistryToMolecular::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation_on_subsumption() {
        // The migrated Chemistry category is kinded and partial (per OBO-RO,
        // #166): compose only succeeds for same-kind transitive relations.
        // Exercise composition along Subsumption chains and verify that the
        // functor preserves the composite.
        for m in ChemistryCategory::morphisms() {
            if m.kind() != ChemistryRelationKind::Subsumption {
                continue;
            }
            for n in ChemistryCategory::morphisms() {
                if n.kind() != ChemistryRelationKind::Subsumption {
                    continue;
                }
                if m.target() != n.source() {
                    continue;
                }
                let composed = match ChemistryCategory::compose(&m, &n) {
                    Some(c) => c,
                    None => continue,
                };
                let mapped_composed = ChemistryToMolecular::map_morphism(&composed);
                let composed_mapped = MolecularCategory::compose(
                    &ChemistryToMolecular::map_morphism(&m),
                    &ChemistryToMolecular::map_morphism(&n),
                )
                .expect("target composition is total");
                assert_eq!(
                    mapped_composed, composed_mapped,
                    "composition law failed for {:?} ∘ {:?}",
                    m, n
                );
            }
        }
    }

    // -- Specific mapping tests --

    #[test]
    fn test_electrolyte_maps_to_ion() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::Electrolyte),
            MolecularEntity::Ion,
        );
    }

    #[test]
    fn test_buffer_maps_to_ion() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::Buffer),
            MolecularEntity::Ion,
        );
    }

    #[test]
    fn test_solvent_maps_to_sodium() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::Solvent),
            MolecularEntity::Sodium,
        );
    }

    #[test]
    fn test_ionic_bond_maps_to_ion() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::IonicBond),
            MolecularEntity::Ion,
        );
    }

    #[test]
    fn test_covalent_bond_maps_to_protein() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::CovalentBond),
            MolecularEntity::Protein,
        );
    }

    #[test]
    fn test_hydrogen_bond_maps_to_protein() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::HydrogenBond),
            MolecularEntity::Protein,
        );
    }

    #[test]
    fn test_ph_maps_to_proton() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::PH),
            MolecularEntity::Proton,
        );
    }

    #[test]
    fn test_osmolarity_maps_to_sodium() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::Osmolarity),
            MolecularEntity::Sodium,
        );
    }

    #[test]
    fn test_solid_maps_to_collagen() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::Solid),
            MolecularEntity::Collagen,
        );
    }

    #[test]
    fn test_liquid_maps_to_mucin() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::Liquid),
            MolecularEntity::Mucin,
        );
    }

    #[test]
    fn test_gel_maps_to_mucin() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::Gel),
            MolecularEntity::Mucin,
        );
    }

    #[test]
    fn test_plasma_maps_to_calcium() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryConcept::Plasma),
            MolecularEntity::Calcium,
        );
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = MolecularEntity::variants();
        for obj in ChemistryConcept::variants() {
            let mapped = ChemistryToMolecular::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid MolecularEntity",
                obj,
                mapped
            );
        }
    }
}

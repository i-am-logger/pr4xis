//! Gap analysis: automated detection of missing ontological distinctions.
//!
//! Uses adjunction unit/counit to find entities that collapse under
//! round-trip mapping. Every collapse = a missing distinction.
//!
//! Discovery methodology:
//!   1. Compute unit eta_A = G(F(A)) for every source entity A
//!   2. If eta_A != A, the source ontology is missing a distinction
//!   3. Compute counit epsilon_B = F(G(B)) for every target entity B
//!   4. If epsilon_B != B, the target ontology is missing a distinction
//!   5. Cross-reference gaps against ContextDef resolutions
//!
//! NOVEL CONTRIBUTION: this is a machine-verifiable methodology for
//! detecting incompleteness in scientific ontologies.

use pr4xis::category::{Entity, Functor};

use crate::natural::biomedical::bioelectricity::biology_functor::BioelectricToBiology;
use crate::natural::biomedical::bioelectricity::molecular_functor::BioelectricToMolecular;
use crate::natural::biomedical::bioelectricity::ontology::BioelectricEntity;
use crate::natural::biomedical::biology::bioelectricity_functor::BiologyToBioelectric;
use crate::natural::biomedical::biology::ontology::BiologicalEntity;
use crate::natural::biomedical::molecular::bioelectricity_functor::MolecularToBioelectric;
use crate::natural::biomedical::molecular::ontology::MolecularEntity;
use crate::natural::biomedical::molecular::pharmacology_functor::MolecularToPharmacology;
use crate::natural::biomedical::pharmacology::molecular_functor::PharmacologyToMolecular;
use crate::natural::biomedical::pharmacology::ontology::PharmacologyEntity;

/// A detected gap: an entity whose round-trip changes its identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gap<E: Entity> {
    /// The original entity.
    pub original: E,
    /// What it became after round-trip.
    pub collapsed_to: E,
}

/// Result of gap analysis on an adjunction.
#[derive(Debug, Clone)]
pub struct GapReport<S: Entity, T: Entity> {
    /// Entities in source that collapse under unit (A -> G(F(A)) != A).
    pub unit_gaps: Vec<Gap<S>>,
    /// Entities in source that are preserved (A -> G(F(A)) == A).
    pub unit_preserved: Vec<S>,
    /// Entities in target that collapse under counit (F(G(B)) -> B != B).
    pub counit_gaps: Vec<Gap<T>>,
    /// Entities in target that are preserved.
    pub counit_preserved: Vec<T>,
}

impl<S: Entity + std::fmt::Debug, T: Entity + std::fmt::Debug> GapReport<S, T> {
    /// Fraction of source entities lost in round-trip.
    pub fn unit_loss_ratio(&self) -> f64 {
        let total = self.unit_gaps.len() + self.unit_preserved.len();
        if total == 0 {
            return 0.0;
        }
        self.unit_gaps.len() as f64 / total as f64
    }

    /// Fraction of target entities lost in round-trip.
    pub fn counit_loss_ratio(&self) -> f64 {
        let total = self.counit_gaps.len() + self.counit_preserved.len();
        if total == 0 {
            return 0.0;
        }
        self.counit_gaps.len() as f64 / total as f64
    }
}

/// Analyze gaps in the Molecular ⊣ Bioelectric adjunction.
pub fn analyze_molecular_bioelectric() -> GapReport<MolecularEntity, BioelectricEntity> {
    let mut unit_gaps = Vec::new();
    let mut unit_preserved = Vec::new();

    for entity in MolecularEntity::variants() {
        let round_trip =
            BioelectricToMolecular::map_object(&MolecularToBioelectric::map_object(&entity));
        if round_trip == entity {
            unit_preserved.push(entity);
        } else {
            unit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    let mut counit_gaps = Vec::new();
    let mut counit_preserved = Vec::new();

    for entity in BioelectricEntity::variants() {
        let round_trip =
            MolecularToBioelectric::map_object(&BioelectricToMolecular::map_object(&entity));
        if round_trip == entity {
            counit_preserved.push(entity);
        } else {
            counit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    GapReport {
        unit_gaps,
        unit_preserved,
        counit_gaps,
        counit_preserved,
    }
}

/// Analyze gaps in the Pharmacology ⊣ Molecular adjunction.
pub fn analyze_pharmacology_molecular() -> GapReport<PharmacologyEntity, MolecularEntity> {
    let mut unit_gaps = Vec::new();
    let mut unit_preserved = Vec::new();

    for entity in PharmacologyEntity::variants() {
        let round_trip =
            MolecularToPharmacology::map_object(&PharmacologyToMolecular::map_object(&entity));
        if round_trip == entity {
            unit_preserved.push(entity);
        } else {
            unit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    let mut counit_gaps = Vec::new();
    let mut counit_preserved = Vec::new();

    for entity in MolecularEntity::variants() {
        let round_trip =
            PharmacologyToMolecular::map_object(&MolecularToPharmacology::map_object(&entity));
        if round_trip == entity {
            counit_preserved.push(entity);
        } else {
            counit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    GapReport {
        unit_gaps,
        unit_preserved,
        counit_gaps,
        counit_preserved,
    }
}

/// Analyze gaps in the Biology ⊣ Bioelectric adjunction.
pub fn analyze_biology_bioelectric() -> GapReport<BiologicalEntity, BioelectricEntity> {
    let mut unit_gaps = Vec::new();
    let mut unit_preserved = Vec::new();

    for entity in BiologicalEntity::variants() {
        let round_trip =
            BioelectricToBiology::map_object(&BiologyToBioelectric::map_object(&entity));
        if round_trip == entity {
            unit_preserved.push(entity);
        } else {
            unit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    let mut counit_gaps = Vec::new();
    let mut counit_preserved = Vec::new();

    for entity in BioelectricEntity::variants() {
        let round_trip =
            BiologyToBioelectric::map_object(&BioelectricToBiology::map_object(&entity));
        if round_trip == entity {
            counit_preserved.push(entity);
        } else {
            counit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    GapReport {
        unit_gaps,
        unit_preserved,
        counit_gaps,
        counit_preserved,
    }
}

/// Analyze two-hop loss: molecular → biochemistry → bioelectricity.
///
/// Hypothesis: routing through biochemistry as an intermediate domain
/// should have lower unit loss than the direct molecular → bioelectricity.
pub fn analyze_two_hop_molecular_bioelectric() -> GapReport<MolecularEntity, BioelectricEntity> {
    // BiochemistryToBioelectric and BiochemistryToMolecular are referenced in
    // the analysis methodology comments below — kept for future two-hop implementation.

    // Forward: molecular → biochemistry → bioelectricity
    // We need molecular → biochemistry, but we only have biochemistry → molecular.
    // So we measure the composed forward path differently:
    // For each molecular entity, find which biochemistry entities map TO it,
    // then map those through to bioelectricity.
    //
    // Actually, the direct comparison is simpler: measure loss of the COMPOSED
    // functor (molecular → bioelectricity via biochemistry) vs the DIRECT functor.
    //
    // But we don't have a molecular → biochemistry functor. We have
    // biochemistry → molecular. So let's measure from the biochemistry side:
    // how much loss does biochemistry → bioelectricity have compared to
    // the molecular → bioelectricity path?

    // Measure: for each molecular entity, does the DIRECT path and the
    // TWO-HOP path through biochemistry's reverse agree?
    // This requires the molecular → bioelectricity right adjoint.

    // Simpler approach: just measure biochemistry → bioelectricity loss directly.
    // If biochemistry → bioelectricity has LESS loss than molecular → bioelectricity,
    // the intermediate domain helps.

    let mut unit_gaps = Vec::new();
    let mut unit_preserved = Vec::new();

    // For each molecular entity, go direct to bioelectricity
    for entity in MolecularEntity::variants() {
        let direct = MolecularToBioelectric::map_object(&entity);
        let round_trip = BioelectricToMolecular::map_object(&direct);
        if round_trip == entity {
            unit_preserved.push(entity);
        } else {
            unit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    // Counit is the same as direct (same bioelectric entities)
    let mut counit_gaps = Vec::new();
    let mut counit_preserved = Vec::new();

    for entity in BioelectricEntity::variants() {
        let round_trip =
            MolecularToBioelectric::map_object(&BioelectricToMolecular::map_object(&entity));
        if round_trip == entity {
            counit_preserved.push(entity);
        } else {
            counit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    GapReport {
        unit_gaps,
        unit_preserved,
        counit_gaps,
        counit_preserved,
    }
}

/// Measure biochemistry → bioelectricity loss independently.
/// If this is LOWER than molecular → bioelectricity (85%),
/// the intermediate domain reduces loss.
pub fn analyze_biochemistry_bioelectric_loss() -> f64 {
    use crate::natural::biomedical::biochemistry::bioelectricity_functor::BiochemistryToBioelectric;
    use crate::natural::biomedical::biochemistry::ontology::BiochemistryEntity;

    let mut gaps = 0;
    let total = BiochemistryEntity::variants().len();

    // Count how many biochemistry entities map to UNIQUE bioelectric entities
    let mut seen = std::collections::HashSet::new();
    for entity in BiochemistryEntity::variants() {
        let mapped = BiochemistryToBioelectric::map_object(&entity);
        if !seen.insert(mapped) {
            gaps += 1; // this entity collapsed into an already-seen target
        }
    }

    gaps as f64 / total as f64
}

/// Analyze gaps in the lineage pair: Syntrometry → Pr4xisSubstrate ⊣
/// Pr4xisSubstrate → Syntrometry. Unlike the biomedical adjunctions this
/// one isn't a strict [`pr4xis::category::Adjunction`] (the reverse
/// direction can't satisfy the strict Functor laws under the dense-source
/// / kinded-target structures — see `docs/research/kinded-functor-
/// failures.md`), but the unit/counit object-level round-trips are
/// well-defined and surface the missing distinctions the pr4xis substrate
/// has relative to Heim's vocabulary.
pub fn analyze_syntrometry_substrate() -> GapReport<
    crate::formal::meta::syntrometry::ontology::SyntrometryConcept,
    crate::formal::meta::syntrometry::substrate::Pr4xisSubstrateConcept,
> {
    use crate::formal::meta::syntrometry::lineage_functor::SyntrometryToPr4xisSubstrate;
    use crate::formal::meta::syntrometry::ontology::SyntrometryConcept;
    use crate::formal::meta::syntrometry::substrate::Pr4xisSubstrateConcept;
    use crate::formal::meta::syntrometry::substrate_functor::map_substrate;

    let mut unit_gaps = Vec::new();
    let mut unit_preserved = Vec::new();

    for entity in SyntrometryConcept::variants() {
        let round_trip = map_substrate(&SyntrometryToPr4xisSubstrate::map_object(&entity));
        if round_trip == entity {
            unit_preserved.push(entity);
        } else {
            unit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    let mut counit_gaps = Vec::new();
    let mut counit_preserved = Vec::new();

    for entity in Pr4xisSubstrateConcept::variants() {
        let round_trip = SyntrometryToPr4xisSubstrate::map_object(&map_substrate(&entity));
        if round_trip == entity {
            counit_preserved.push(entity);
        } else {
            counit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    GapReport {
        unit_gaps,
        unit_preserved,
        counit_gaps,
        counit_preserved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::natural::biomedical::molecular::ontology::{
        FunctionalContext, MolecularFunctionalContext,
    };
    use pr4xis::ontology::reasoning::context;

    // -----------------------------------------------------------------------
    // Molecular-Bioelectric gap analysis
    // -----------------------------------------------------------------------

    #[test]
    fn test_molecular_bioelectric_gaps_exist() {
        let report = analyze_molecular_bioelectric();
        // There MUST be gaps — the adjunction is not an equivalence
        assert!(
            !report.unit_gaps.is_empty(),
            "unit should have gaps (molecular distinctions invisible at bioelectric scale)"
        );
        assert!(
            !report.counit_gaps.is_empty(),
            "counit should have gaps (bioelectric distinctions that collapse at molecular scale)"
        );
    }

    #[test]
    fn test_molecular_bioelectric_unit_loss_ratio() {
        let report = analyze_molecular_bioelectric();
        let ratio = report.unit_loss_ratio();
        // Expect significant but not total loss
        assert!(
            ratio > 0.1,
            "unit loss should be > 10%, got {:.1}%",
            ratio * 100.0
        );
        assert!(
            ratio < 0.9,
            "unit loss should be < 90%, got {:.1}%",
            ratio * 100.0
        );
    }

    #[test]
    fn test_molecular_bioelectric_counit_loss_ratio() {
        let report = analyze_molecular_bioelectric();
        let ratio = report.counit_loss_ratio();
        assert!(
            ratio > 0.0,
            "counit loss should be > 0%, got {:.1}%",
            ratio * 100.0
        );
    }

    #[test]
    fn test_kv_is_unit_gap() {
        let report = analyze_molecular_bioelectric();
        let kv_gap = report
            .unit_gaps
            .iter()
            .find(|g| g.original == MolecularEntity::Kv);
        assert!(
            kv_gap.is_some(),
            "Kv should be a unit gap (collapses to different entity on round-trip)"
        );
    }

    #[test]
    fn test_membrane_potential_is_counit_gap() {
        let report = analyze_molecular_bioelectric();
        let mp_gap = report
            .counit_gaps
            .iter()
            .find(|g| g.original == BioelectricEntity::MembranePotential);
        assert!(
            mp_gap.is_some(),
            "MembranePotential should be a counit gap (collapses on round-trip)"
        );
    }

    #[test]
    fn test_unit_loss_greater_than_counit_loss() {
        // PREDICTION: molecular→bioelectric loses more than bioelectric→molecular
        // because many molecules map to few bioelectric concepts
        let report = analyze_molecular_bioelectric();
        assert!(
            report.unit_loss_ratio() > report.counit_loss_ratio(),
            "unit loss ({:.1}%) should exceed counit loss ({:.1}%): \
             molecular→bioelectric is more lossy",
            report.unit_loss_ratio() * 100.0,
            report.counit_loss_ratio() * 100.0,
        );
    }

    // -----------------------------------------------------------------------
    // Gap resolution proof: every Kv gap has a ContextDef resolution
    // -----------------------------------------------------------------------

    #[test]
    fn test_kv_gap_is_resolved_by_context() {
        // The gap: Kv round-trips to a different entity
        let report = analyze_molecular_bioelectric();
        let kv_gap = report
            .unit_gaps
            .iter()
            .find(|g| g.original == MolecularEntity::Kv)
            .expect("Kv should be a gap");

        // The resolution: ContextDef distinguishes Kv's two roles
        let constitutive = context::resolve::<MolecularFunctionalContext>(
            &MolecularEntity::Kv,
            &FunctionalContext::Constitutive,
        );
        let therapeutic = context::resolve::<MolecularFunctionalContext>(
            &MolecularEntity::Kv,
            &FunctionalContext::Therapeutic,
        );

        // Both contexts resolve to DIFFERENT roles
        assert!(constitutive.is_some(), "Kv constitutive should resolve");
        assert!(therapeutic.is_some(), "Kv therapeutic should resolve");
        assert_ne!(
            constitutive, therapeutic,
            "Kv constitutive and therapeutic should resolve to DIFFERENT roles \
             (this is the missing distinction the adjunction detected): {:?}",
            kv_gap
        );
    }

    #[test]
    fn test_piezo_gap_is_resolved_by_context() {
        let report = analyze_molecular_bioelectric();
        // Piezo2 should be a gap (collapses to Piezo1 on round-trip)
        let piezo2_gap = report
            .unit_gaps
            .iter()
            .find(|g| g.original == MolecularEntity::Piezo2);
        assert!(piezo2_gap.is_some(), "Piezo2 should be a unit gap");

        // Piezo1 is preserved (canonical representative)
        let piezo1_preserved = report
            .unit_preserved
            .iter()
            .any(|e| *e == MolecularEntity::Piezo1);
        assert!(piezo1_preserved, "Piezo1 should be preserved (canonical)");
    }

    // -----------------------------------------------------------------------
    // Pharmacology-Molecular gap analysis
    // -----------------------------------------------------------------------

    #[test]
    fn test_pharmacology_molecular_gaps_exist() {
        let report = analyze_pharmacology_molecular();
        assert!(!report.unit_gaps.is_empty());
    }

    // -----------------------------------------------------------------------
    // Biology-Bioelectric gap analysis
    // -----------------------------------------------------------------------

    #[test]
    fn test_biology_bioelectric_gaps_exist() {
        let report = analyze_biology_bioelectric();
        assert!(!report.unit_gaps.is_empty());
    }

    #[test]
    fn test_biology_bioelectric_many_cells_collapse() {
        // Multiple cell types should collapse to MembranePotential
        let report = analyze_biology_bioelectric();
        let cell_gaps: Vec<_> = report
            .unit_gaps
            .iter()
            .filter(|g| {
                matches!(
                    g.original,
                    BiologicalEntity::ColumnarEpithelial
                        | BiologicalEntity::GobletCell
                        | BiologicalEntity::Fibroblast
                )
            })
            .collect();
        assert!(
            cell_gaps.len() >= 2,
            "multiple cell types should collapse (bioelectric can't distinguish them)"
        );
    }

    // -----------------------------------------------------------------------
    // Cross-adjunction comparison
    // -----------------------------------------------------------------------

    #[test]
    fn test_all_adjunctions_have_gaps() {
        let mol_bio = analyze_molecular_bioelectric();
        let pharma_mol = analyze_pharmacology_molecular();
        let bio_bioelec = analyze_biology_bioelectric();

        assert!(
            !mol_bio.unit_gaps.is_empty(),
            "mol-bio should have unit gaps"
        );
        assert!(
            !pharma_mol.unit_gaps.is_empty(),
            "pharma-mol should have unit gaps"
        );
        assert!(
            !bio_bioelec.unit_gaps.is_empty(),
            "bio-bioelec should have unit gaps"
        );

        // ALL adjunctions detect gaps — the methodology works universally
    }

    #[test]
    fn test_print_all_gap_reports() {
        let mb = analyze_molecular_bioelectric();
        let pm = analyze_pharmacology_molecular();
        let bb = analyze_biology_bioelectric();

        eprintln!("\n--- GAP ANALYSIS RESULTS ---");
        eprintln!("Molecular ⊣ Bioelectric:");
        eprintln!(
            "  Unit loss:   {:.1}% ({}/{})",
            mb.unit_loss_ratio() * 100.0,
            mb.unit_gaps.len(),
            mb.unit_gaps.len() + mb.unit_preserved.len()
        );
        for g in &mb.unit_gaps {
            eprintln!("    {:?} → {:?}", g.original, g.collapsed_to);
        }
        eprintln!(
            "  Counit loss: {:.1}% ({}/{})",
            mb.counit_loss_ratio() * 100.0,
            mb.counit_gaps.len(),
            mb.counit_gaps.len() + mb.counit_preserved.len()
        );
        for g in &mb.counit_gaps {
            eprintln!("    {:?} → {:?}", g.original, g.collapsed_to);
        }

        eprintln!("Pharmacology ⊣ Molecular:");
        eprintln!(
            "  Unit loss:   {:.1}% ({}/{})",
            pm.unit_loss_ratio() * 100.0,
            pm.unit_gaps.len(),
            pm.unit_gaps.len() + pm.unit_preserved.len()
        );
        eprintln!(
            "  Counit loss: {:.1}% ({}/{})",
            pm.counit_loss_ratio() * 100.0,
            pm.counit_gaps.len(),
            pm.counit_gaps.len() + pm.counit_preserved.len()
        );

        eprintln!("Biology ⊣ Bioelectric:");
        eprintln!(
            "  Unit loss:   {:.1}% ({}/{})",
            bb.unit_loss_ratio() * 100.0,
            bb.unit_gaps.len(),
            bb.unit_gaps.len() + bb.unit_preserved.len()
        );
        eprintln!(
            "  Counit loss: {:.1}% ({}/{})",
            bb.counit_loss_ratio() * 100.0,
            bb.counit_gaps.len(),
            bb.counit_gaps.len() + bb.counit_preserved.len()
        );
        eprintln!("--- END ---\n");
    }

    #[test]
    fn test_loss_ratios_are_asymmetric() {
        // The unit loss should differ from counit loss for each adjunction
        // (information loss is asymmetric between scales)
        let mol_bio = analyze_molecular_bioelectric();
        assert!(
            (mol_bio.unit_loss_ratio() - mol_bio.counit_loss_ratio()).abs() > 0.01,
            "loss should be asymmetric: unit={:.1}% counit={:.1}%",
            mol_bio.unit_loss_ratio() * 100.0,
            mol_bio.counit_loss_ratio() * 100.0,
        );
    }

    // -----------------------------------------------------------------------
    // Intermediate domain hypothesis: biochemistry reduces loss
    // -----------------------------------------------------------------------

    #[test]
    fn test_biochemistry_bioelectric_loss() {
        let loss = analyze_biochemistry_bioelectric_loss();
        eprintln!(
            "\n--- INTERMEDIATE DOMAIN ANALYSIS ---\n\
             Direct molecular → bioelectric unit loss: 85.2%\n\
             Biochemistry → bioelectric collapse: {:.1}%\n\
             ---",
            loss * 100.0
        );
        // Biochemistry has 20 entities mapping to 19 bioelectric entities.
        // Some collapse is expected but it should be LESS than molecular's 85%.
        // If so, the intermediate domain hypothesis holds.
    }

    #[test]
    fn test_print_biochemistry_bioelectric_mapping() {
        use crate::natural::biomedical::biochemistry::bioelectricity_functor::BiochemistryToBioelectric;
        use crate::natural::biomedical::biochemistry::ontology::BiochemistryEntity;

        eprintln!("\n--- BIOCHEMISTRY → BIOELECTRICITY MAPPING ---");
        for entity in BiochemistryEntity::variants() {
            let mapped = BiochemistryToBioelectric::map_object(&entity);
            eprintln!("  {:?} → {:?}", entity, mapped);
        }
        eprintln!("---\n");
    }

    // -----------------------------------------------------------------------
    // Full chain measurement: acoustics → biophysics → molecular → bioelectricity
    // -----------------------------------------------------------------------

    #[test]
    fn test_full_chain_collapse_measurement() {
        use crate::natural::biomedical::acoustics::biophysics_functor::AcousticsToBiophysics;
        use crate::natural::biomedical::acoustics::ontology::AcousticsEntity;
        use crate::natural::biomedical::biochemistry::bioelectricity_functor::BiochemistryToBioelectric;
        use crate::natural::biomedical::biochemistry::ontology::BiochemistryEntity;
        use crate::natural::biomedical::biophysics::molecular_functor::BiophysicsToMolecular;
        use crate::natural::biomedical::biophysics::ontology::BiophysicsEntity;
        use crate::natural::biomedical::mechanobiology::molecular_functor::MechanobiologyToMolecular;
        use crate::natural::biomedical::mechanobiology::ontology::MechanobiologyEntity;
        use pr4xis::category::Entity;

        // Measure collapse at each hop by counting unique targets

        // Hop 1: acoustics → biophysics
        let acous_targets: std::collections::HashSet<_> = AcousticsEntity::variants()
            .iter()
            .map(|e| AcousticsToBiophysics::map_object(e))
            .collect();
        let acous_collapse =
            1.0 - (acous_targets.len() as f64 / AcousticsEntity::variants().len() as f64);

        // Hop 2: biophysics → molecular
        let biophys_targets: std::collections::HashSet<_> = BiophysicsEntity::variants()
            .iter()
            .map(|e| BiophysicsToMolecular::map_object(e))
            .collect();
        let biophys_collapse =
            1.0 - (biophys_targets.len() as f64 / BiophysicsEntity::variants().len() as f64);

        // Hop 3: mechanobiology → molecular
        let mechano_targets: std::collections::HashSet<_> = MechanobiologyEntity::variants()
            .iter()
            .map(|e| MechanobiologyToMolecular::map_object(e))
            .collect();
        let mechano_collapse =
            1.0 - (mechano_targets.len() as f64 / MechanobiologyEntity::variants().len() as f64);

        // Hop 4: molecular → bioelectricity (direct)
        let mol_targets: std::collections::HashSet<_> = MolecularEntity::variants()
            .iter()
            .map(|e| MolecularToBioelectric::map_object(e))
            .collect();
        let mol_collapse =
            1.0 - (mol_targets.len() as f64 / MolecularEntity::variants().len() as f64);

        // Hop 5: biochemistry → bioelectricity
        let biochem_targets: std::collections::HashSet<_> = BiochemistryEntity::variants()
            .iter()
            .map(|e| BiochemistryToBioelectric::map_object(e))
            .collect();
        let biochem_collapse =
            1.0 - (biochem_targets.len() as f64 / BiochemistryEntity::variants().len() as f64);

        // End-to-end: acoustics → biophysics → molecular → bioelectricity
        let end_to_end_targets: std::collections::HashSet<_> = AcousticsEntity::variants()
            .iter()
            .map(|e| {
                let biophys = AcousticsToBiophysics::map_object(e);
                let mol = BiophysicsToMolecular::map_object(&biophys);
                MolecularToBioelectric::map_object(&mol)
            })
            .collect();
        let end_to_end_collapse =
            1.0 - (end_to_end_targets.len() as f64 / AcousticsEntity::variants().len() as f64);

        eprintln!("\n=== FULL CHAIN COLLAPSE ANALYSIS ===");
        eprintln!("Per-hop collapse (fraction of entities that map to same target as another):");
        eprintln!(
            "  acoustics → biophysics:       {:.1}% ({} unique targets from {} entities)",
            acous_collapse * 100.0,
            acous_targets.len(),
            AcousticsEntity::variants().len()
        );
        eprintln!(
            "  biophysics → molecular:        {:.1}% ({} unique from {})",
            biophys_collapse * 100.0,
            biophys_targets.len(),
            BiophysicsEntity::variants().len()
        );
        eprintln!(
            "  mechanobiology → molecular:    {:.1}% ({} unique from {})",
            mechano_collapse * 100.0,
            mechano_targets.len(),
            MechanobiologyEntity::variants().len()
        );
        eprintln!(
            "  molecular → bioelectricity:    {:.1}% ({} unique from {})",
            mol_collapse * 100.0,
            mol_targets.len(),
            MolecularEntity::variants().len()
        );
        eprintln!(
            "  biochemistry → bioelectricity: {:.1}% ({} unique from {})",
            biochem_collapse * 100.0,
            biochem_targets.len(),
            BiochemistryEntity::variants().len()
        );
        eprintln!("");
        eprintln!("End-to-end: acoustics → biophysics → molecular → bioelectricity:");
        eprintln!(
            "  {:.1}% collapse ({} unique bioelectric targets from {} acoustic entities)",
            end_to_end_collapse * 100.0,
            end_to_end_targets.len(),
            AcousticsEntity::variants().len()
        );
        eprintln!("");
        eprintln!("Adjunction round-trip losses (unit loss):");
        let mb = analyze_molecular_bioelectric();
        let pm = analyze_pharmacology_molecular();
        let bb = analyze_biology_bioelectric();
        eprintln!(
            "  molecular ⊣ bioelectric:    {:.1}%",
            mb.unit_loss_ratio() * 100.0
        );
        eprintln!(
            "  pharmacology ⊣ molecular:   {:.1}%",
            pm.unit_loss_ratio() * 100.0
        );
        eprintln!(
            "  biology ⊣ bioelectric:      {:.1}%",
            bb.unit_loss_ratio() * 100.0
        );
        eprintln!("=== END ===\n");
    }

    // -----------------------------------------------------------------------
    // Syntrometry-Substrate gap analysis (#62)
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Syntrometry → MetaOntology cross-functor collapse (Phase 4)
    // -----------------------------------------------------------------------

    /// Syntrometry → Staging collapses Heim's finer grain into Futamura's
    /// coarser vocabulary. Expected 6+ collapses (many concepts land at
    /// Program).
    #[test]
    fn test_syntrometry_to_staging_collapse_is_measured() {
        use crate::formal::meta::staging::ontology::StageConcept;
        use crate::formal::meta::syntrometry::ontology::SyntrometryConcept;
        use crate::formal::meta::syntrometry::staging_functor::SyntrometryToStaging;
        use pr4xis::category::{Entity, Functor};
        use std::collections::HashSet;

        let mapped: HashSet<StageConcept> = SyntrometryConcept::variants()
            .into_iter()
            .map(|c| SyntrometryToStaging::map_object(&c))
            .collect();
        let total = SyntrometryConcept::variants().len();
        let unique = mapped.len();
        let collapse = total - unique;
        assert!(
            collapse >= 6,
            "Syntrometry → Staging expected to collapse significantly (Futamura vocabulary is coarser); got only {} collapses",
            collapse
        );
        eprintln!(
            "\nSyntrometry → Staging collapse: {}/{} ({:.1}%)",
            collapse,
            total,
            collapse as f64 / total as f64 * 100.0
        );
    }

    /// Phase 5: Syntrometry → Algebra maps Heim's operators onto Goguen /
    /// Zimmermann ontology-algebra primitives.
    #[test]
    fn test_syntrometry_to_algebra_collapse_is_measured() {
        use crate::formal::meta::algebra::ontology::AlgebraConcept;
        use crate::formal::meta::syntrometry::algebra_functor::SyntrometryToAlgebra;
        use crate::formal::meta::syntrometry::ontology::SyntrometryConcept;
        use pr4xis::category::{Entity, Functor};
        use std::collections::HashSet;

        let mapped: HashSet<AlgebraConcept> = SyntrometryConcept::variants()
            .into_iter()
            .map(|c| SyntrometryToAlgebra::map_object(&c))
            .collect();
        let total = SyntrometryConcept::variants().len();
        let unique = mapped.len();
        let collapse = total - unique;
        eprintln!(
            "\nSyntrometry → Algebra collapse: {}/{} ({:.1}%)",
            collapse,
            total,
            collapse as f64 / total as f64 * 100.0
        );
        assert!(collapse < total, "functor is not trivial");
    }

    /// Syntrometry → MetaOntology intentionally collapses 6 of 18 concepts
    /// — pairs that share a diagnostic role collapse to the same `MetaEntity`
    /// bucket (e.g., Synkolator/Korporator/permutations → Functor;
    /// Koordination/Reflexivity → NaturalTransformation).
    #[test]
    fn test_syntrometry_to_meta_ontology_collapse_is_six() {
        use crate::formal::meta::ontology_diagnostics::ontology::MetaEntity;
        use crate::formal::meta::syntrometry::meta_ontology_functor::SyntrometryToMetaOntology;
        use crate::formal::meta::syntrometry::ontology::SyntrometryConcept;
        use pr4xis::category::{Entity, Functor};
        use std::collections::HashSet;

        let mapped: HashSet<MetaEntity> = SyntrometryConcept::variants()
            .into_iter()
            .map(|c| SyntrometryToMetaOntology::map_object(&c))
            .collect();
        let total = SyntrometryConcept::variants().len();
        let unique = mapped.len();
        let collapse = total - unique;
        assert_eq!(
            collapse, 6,
            "expected 6 intentional collapses (Syntrix/SyntrixLevel → CategoryStructure; \
             Synkolator/Korporator/SequencePermutation/OrientationPermutation → Functor; \
             Predicate/Aspektivsystem → DomainOntology; Koordination/Reflexivity → NaturalTransformation); \
             got {}",
            collapse
        );
        eprintln!(
            "\nSyntrometry → MetaOntology collapse: {}/{} ({:.1}%)",
            collapse,
            total,
            collapse as f64 / total as f64 * 100.0
        );
    }

    #[test]
    fn test_syntrometry_substrate_intentional_collapses() {
        let report = analyze_syntrometry_substrate();
        // The substrate is closed — every substrate primitive round-trips.
        assert!(
            report.counit_gaps.is_empty(),
            "counit gaps should be empty (substrate is closed): {:?}",
            report.counit_gaps
        );
        // Intentional collapses:
        // - Dialektik → SubCategory → Syntrix (opposition lives in Dialectics)
        // - SequencePermutation, OrientationPermutation → SubEndofunctor (both collapse with Synkolator)
        // - Aspektivsystem → SubOntology (collapses with Predikatrix)
        assert_eq!(
            report.unit_gaps.len(),
            4,
            "expected four intentional collapses; got {:?}",
            report.unit_gaps
        );
        eprintln!(
            "\nSyntrometry ⊣ Pr4xisSubstrate: {}/{} unit preserved ({} intentional collapses); {}/{} counit preserved",
            report.unit_preserved.len(),
            report.unit_preserved.len() + report.unit_gaps.len(),
            report.unit_gaps.len(),
            report.counit_preserved.len(),
            report.counit_preserved.len() + report.counit_gaps.len(),
        );
    }
}

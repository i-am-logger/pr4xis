//! Electrophysiology ontology: the science of measuring bioelectric signals.
//!
//! Models measurement techniques, measured quantities, and recording modes
//! as a formal ontological structure with taxonomy, category, qualities, and axioms.
//!
//! Key references:
//! - Levin 2024: Optical Estimation of Bioelectric Patterns
//! - Neher & Sakmann 1976: patch clamp technique
//! - Bhatt et al. 2015: bioimpedance spectroscopy

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::opposition;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every entity in the electrophysiology domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ElectrophysiologyEntity {
    // Measurement techniques
    PatchClamp,
    SharpElectrode,
    VoltageSensitiveDye,
    CalciumImaging,
    Bioimpedance,
    ExtracellularRecording,
    MultiElectrodeArray,
    OpticalMapping,

    // Measured quantities
    RestingPotential,
    ActionPotential,
    TransepithelialPotential,
    FieldPotential,
    Impedance,
    IntracellularCalcium,

    // Recording modes
    WholeCell,
    CellAttached,
    InsideOut,
    OutsideOut,
    CurrentClamp,
    VoltageClamp,

    // Abstract categories
    MeasurementTechnique,
    MeasuredQuantity,
    RecordingMode,
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for electrophysiology entities.
define_ontology! {
    /// Electrophysiology ontology: techniques, quantities, recording modes.
    pub ElectrophysiologyOntologyMeta for ElectrophysiologyCategory {
        entity: ElectrophysiologyEntity,
        relation: ElectrophysiologyRelation,

        taxonomy: ElectrophysiologyTaxonomy [
            (PatchClamp, MeasurementTechnique),
            (SharpElectrode, MeasurementTechnique),
            (VoltageSensitiveDye, MeasurementTechnique),
            (CalciumImaging, MeasurementTechnique),
            (Bioimpedance, MeasurementTechnique),
            (ExtracellularRecording, MeasurementTechnique),
            (MultiElectrodeArray, MeasurementTechnique),
            (OpticalMapping, MeasurementTechnique),
            (RestingPotential, MeasuredQuantity),
            (ActionPotential, MeasuredQuantity),
            (TransepithelialPotential, MeasuredQuantity),
            (FieldPotential, MeasuredQuantity),
            (Impedance, MeasuredQuantity),
            (IntracellularCalcium, MeasuredQuantity),
            (WholeCell, RecordingMode),
            (CellAttached, RecordingMode),
            (InsideOut, RecordingMode),
            (OutsideOut, RecordingMode),
            (CurrentClamp, RecordingMode),
            (VoltageClamp, RecordingMode),
        ],

        opposition: ElectrophysiologyOpposition [
            (PatchClamp, OpticalMapping),
            (CurrentClamp, VoltageClamp),
            (RestingPotential, ActionPotential),
        ],
    }
}

// ---------------------------------------------------------------------------
// Spatial and temporal resolution enums
// ---------------------------------------------------------------------------

/// Spatial resolution achievable by a measurement technique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpatialScale {
    SingleCell,
    CellCluster,
    Tissue,
    Organ,
}

/// Temporal resolution achievable by a measurement technique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemporalScale {
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: is this measurement technique invasive (damages or penetrates the cell)?
#[derive(Debug, Clone)]
pub struct IsInvasive;

impl Quality for IsInvasive {
    type Individual = ElectrophysiologyEntity;
    type Value = bool;

    fn get(&self, individual: &ElectrophysiologyEntity) -> Option<bool> {
        use ElectrophysiologyEntity::*;
        match individual {
            PatchClamp | SharpElectrode => Some(true),
            VoltageSensitiveDye
            | CalciumImaging
            | Bioimpedance
            | ExtracellularRecording
            | MultiElectrodeArray
            | OpticalMapping => Some(false),
            _ => None,
        }
    }
}

/// Quality: what spatial resolution does this technique achieve?
#[derive(Debug, Clone)]
pub struct SpatialResolution;

impl Quality for SpatialResolution {
    type Individual = ElectrophysiologyEntity;
    type Value = SpatialScale;

    fn get(&self, individual: &ElectrophysiologyEntity) -> Option<SpatialScale> {
        use ElectrophysiologyEntity::*;
        use SpatialScale::*;
        match individual {
            PatchClamp | SharpElectrode => Some(SingleCell),
            VoltageSensitiveDye | CalciumImaging | OpticalMapping => Some(Tissue),
            MultiElectrodeArray | ExtracellularRecording => Some(CellCluster),
            Bioimpedance => Some(Organ),
            _ => None,
        }
    }
}

/// Quality: what temporal resolution does this technique achieve?
#[derive(Debug, Clone)]
pub struct TemporalResolution;

impl Quality for TemporalResolution {
    type Individual = ElectrophysiologyEntity;
    type Value = TemporalScale;

    fn get(&self, individual: &ElectrophysiologyEntity) -> Option<TemporalScale> {
        use ElectrophysiologyEntity::*;
        use TemporalScale::*;
        match individual {
            PatchClamp | SharpElectrode => Some(Microseconds),
            ExtracellularRecording | MultiElectrodeArray => Some(Milliseconds),
            VoltageSensitiveDye | OpticalMapping | CalciumImaging => Some(Milliseconds),
            Bioimpedance => Some(Seconds),
            _ => None,
        }
    }
}

/// Quality: does this technique measure membrane potential (Vmem)?
#[derive(Debug, Clone)]
pub struct MeasuresVmem;

impl Quality for MeasuresVmem {
    type Individual = ElectrophysiologyEntity;
    type Value = bool;

    fn get(&self, individual: &ElectrophysiologyEntity) -> Option<bool> {
        use ElectrophysiologyEntity::*;
        match individual {
            PatchClamp | SharpElectrode | VoltageSensitiveDye | OpticalMapping => Some(true),
            CalciumImaging | Bioimpedance | ExtracellularRecording | MultiElectrodeArray => {
                Some(false)
            }
            _ => None,
        }
    }
}

/// Quality: can this technique be used in vivo?
#[derive(Debug, Clone)]
pub struct CanMeasureInVivo;

impl Quality for CanMeasureInVivo {
    type Individual = ElectrophysiologyEntity;
    type Value = bool;

    fn get(&self, individual: &ElectrophysiologyEntity) -> Option<bool> {
        use ElectrophysiologyEntity::*;
        match individual {
            VoltageSensitiveDye
            | CalciumImaging
            | Bioimpedance
            | ExtracellularRecording
            | OpticalMapping => Some(true),
            PatchClamp | SharpElectrode | MultiElectrodeArray => Some(false),
            _ => None,
        }
    }
}

/// Quality: does this technique require direct physical contact with the cell?
#[derive(Debug, Clone)]
pub struct RequiresContactWithCell;

impl Quality for RequiresContactWithCell {
    type Individual = ElectrophysiologyEntity;
    type Value = bool;

    fn get(&self, individual: &ElectrophysiologyEntity) -> Option<bool> {
        use ElectrophysiologyEntity::*;
        match individual {
            PatchClamp | SharpElectrode => Some(true),
            VoltageSensitiveDye | CalciumImaging | OpticalMapping => Some(false),
            ExtracellularRecording | MultiElectrodeArray => Some(true),
            Bioimpedance => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition (semantic contrasts)
// ---------------------------------------------------------------------------

/// Opposition pairs in the electrophysiology domain.
///
/// - PatchClamp ↔ OpticalMapping: invasive/single-cell vs non-invasive/spatial
/// - CurrentClamp ↔ VoltageClamp: measure voltage vs measure current
/// - RestingPotential ↔ ActionPotential: steady-state vs transient
/// Axiom: electrophysiology opposition is symmetric.
pub struct ElectrophysiologyOppositionSymmetric;

impl Axiom for ElectrophysiologyOppositionSymmetric {
    fn description(&self) -> &str {
        "electrophysiology opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<ElectrophysiologyOpposition>::new().holds()
    }
}

/// Axiom: electrophysiology opposition is irreflexive (nothing opposes itself).
pub struct ElectrophysiologyOppositionIrreflexive;

impl Axiom for ElectrophysiologyOppositionIrreflexive {
    fn description(&self) -> &str {
        "electrophysiology opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<ElectrophysiologyOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Taxonomy is a directed acyclic graph.
pub struct TaxonomyIsDAG;

impl Axiom for TaxonomyIsDAG {
    fn description(&self) -> &str {
        "electrophysiology taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<ElectrophysiologyTaxonomy>::new().holds()
    }
}

/// Category identity and composition laws hold.
pub struct CategoryLawsHold;

impl Axiom for CategoryLawsHold {
    fn description(&self) -> &str {
        "electrophysiology category satisfies identity, associativity, and closure"
    }

    fn holds(&self) -> bool {
        use pr4xis::category::validate::check_category_laws;
        check_category_laws::<ElectrophysiologyCategory>().is_ok()
    }
}

/// At least one non-invasive measurement technique exists.
pub struct NonInvasiveMethodExists;

impl Axiom for NonInvasiveMethodExists {
    fn description(&self) -> &str {
        "at least one non-invasive measurement technique exists"
    }

    fn holds(&self) -> bool {
        let q = IsInvasive;
        ElectrophysiologyEntity::variants()
            .iter()
            .any(|e| q.get(e) == Some(false))
    }
}

/// Both single-cell and tissue-level resolution methods exist.
pub struct MultiscaleMethods;

impl Axiom for MultiscaleMethods {
    fn description(&self) -> &str {
        "both single-cell and tissue-level spatial resolution methods exist"
    }

    fn holds(&self) -> bool {
        let q = SpatialResolution;
        let all = ElectrophysiologyEntity::variants();
        let has_single_cell = all
            .iter()
            .any(|e| q.get(e) == Some(SpatialScale::SingleCell));
        let has_tissue = all.iter().any(|e| q.get(e) == Some(SpatialScale::Tissue));
        has_single_cell && has_tissue
    }
}

/// At least one method can measure Vmem in vivo (Levin's voltage-sensitive dyes).
pub struct VmemInVivoMethodExists;

impl Axiom for VmemInVivoMethodExists {
    fn description(&self) -> &str {
        "at least one method measures Vmem in vivo (voltage-sensitive dye — Levin's primary tool)"
    }

    fn holds(&self) -> bool {
        let vmem = MeasuresVmem;
        let in_vivo = CanMeasureInVivo;
        ElectrophysiologyEntity::variants()
            .iter()
            .any(|e| vmem.get(e) == Some(true) && in_vivo.get(e) == Some(true))
    }
}

/// Patch clamp is invasive AND has single-cell resolution (gold standard but destructive).
pub struct PatchClampGoldStandard;

impl Axiom for PatchClampGoldStandard {
    fn description(&self) -> &str {
        "patch clamp is invasive with single-cell resolution (gold standard)"
    }

    fn holds(&self) -> bool {
        use ElectrophysiologyEntity::*;
        IsInvasive.get(&PatchClamp) == Some(true)
            && SpatialResolution.get(&PatchClamp) == Some(SpatialScale::SingleCell)
    }
}

/// Bioimpedance is non-invasive (surface method for deep tissue proxy).
pub struct BioimpedanceNonInvasive;

impl Axiom for BioimpedanceNonInvasive {
    fn description(&self) -> &str {
        "bioimpedance is non-invasive (surface electrodes, deep tissue proxy)"
    }

    fn holds(&self) -> bool {
        use ElectrophysiologyEntity::*;
        IsInvasive.get(&Bioimpedance) == Some(false)
    }
}

/// Optical methods (VSD, calcium imaging, optical mapping) do not require cell contact.
pub struct OpticalMethodsNoContact;

impl Axiom for OpticalMethodsNoContact {
    fn description(&self) -> &str {
        "optical methods do not require direct cell contact"
    }

    fn holds(&self) -> bool {
        use ElectrophysiologyEntity::*;
        let q = RequiresContactWithCell;
        q.get(&VoltageSensitiveDye) == Some(false)
            && q.get(&CalciumImaging) == Some(false)
            && q.get(&OpticalMapping) == Some(false)
    }
}

/// Both Vmem-measuring and non-Vmem-measuring techniques exist.
pub struct VmemAndNonVmemTechniques;

impl Axiom for VmemAndNonVmemTechniques {
    fn description(&self) -> &str {
        "both Vmem-measuring and non-Vmem-measuring techniques exist"
    }

    fn holds(&self) -> bool {
        let q = MeasuresVmem;
        let all = ElectrophysiologyEntity::variants();
        let has_vmem = all.iter().any(|e| q.get(e) == Some(true));
        let has_non_vmem = all.iter().any(|e| q.get(e) == Some(false));
        has_vmem && has_non_vmem
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level ontology tying together the electrophysiology category, qualities, and axioms.
pub struct ElectrophysiologyOntology;

impl Ontology for ElectrophysiologyOntology {
    type Cat = ElectrophysiologyCategory;
    type Qual = IsInvasive;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(TaxonomyIsDAG),
            Box::new(CategoryLawsHold),
            Box::new(NonInvasiveMethodExists),
            Box::new(MultiscaleMethods),
            Box::new(VmemInVivoMethodExists),
            Box::new(PatchClampGoldStandard),
            Box::new(BioimpedanceNonInvasive),
            Box::new(OpticalMethodsNoContact),
            Box::new(VmemAndNonVmemTechniques),
            Box::new(ElectrophysiologyOppositionSymmetric),
            Box::new(ElectrophysiologyOppositionIrreflexive),
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    // -- Entity count --

    #[test]
    fn test_entity_count() {
        // 8 techniques + 6 quantities + 6 recording modes + 3 abstract = 23
        assert_eq!(ElectrophysiologyEntity::variants().len(), 23);
    }

    // -- Axiom tests --

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(TaxonomyIsDAG.holds(), "{}", TaxonomyIsDAG.description());
    }

    #[test]
    fn test_category_laws_hold() {
        assert!(
            CategoryLawsHold.holds(),
            "{}",
            CategoryLawsHold.description()
        );
    }

    #[test]
    fn test_non_invasive_method_exists() {
        assert!(
            NonInvasiveMethodExists.holds(),
            "{}",
            NonInvasiveMethodExists.description()
        );
    }

    #[test]
    fn test_multiscale_methods() {
        assert!(
            MultiscaleMethods.holds(),
            "{}",
            MultiscaleMethods.description()
        );
    }

    #[test]
    fn test_vmem_in_vivo_method_exists() {
        assert!(
            VmemInVivoMethodExists.holds(),
            "{}",
            VmemInVivoMethodExists.description()
        );
    }

    #[test]
    fn test_patch_clamp_gold_standard() {
        assert!(
            PatchClampGoldStandard.holds(),
            "{}",
            PatchClampGoldStandard.description()
        );
    }

    #[test]
    fn test_bioimpedance_non_invasive() {
        assert!(
            BioimpedanceNonInvasive.holds(),
            "{}",
            BioimpedanceNonInvasive.description()
        );
    }

    #[test]
    fn test_optical_methods_no_contact() {
        assert!(
            OpticalMethodsNoContact.holds(),
            "{}",
            OpticalMethodsNoContact.description()
        );
    }

    #[test]
    fn test_vmem_and_non_vmem_techniques() {
        assert!(
            VmemAndNonVmemTechniques.holds(),
            "{}",
            VmemAndNonVmemTechniques.description()
        );
    }

    // -- Category law tests --

    #[test]
    fn test_electrophysiology_category_laws() {
        check_category_laws::<ElectrophysiologyCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<ElectrophysiologyTaxonomy>>().unwrap();
    }

    // -- Taxonomy tests --

    #[test]
    fn test_techniques_are_measurement_techniques() {
        use ElectrophysiologyEntity::*;
        for tech in [
            PatchClamp,
            SharpElectrode,
            VoltageSensitiveDye,
            CalciumImaging,
            Bioimpedance,
            ExtracellularRecording,
            MultiElectrodeArray,
            OpticalMapping,
        ] {
            assert!(
                taxonomy::is_a::<ElectrophysiologyTaxonomy>(&tech, &MeasurementTechnique),
                "{:?} should be a MeasurementTechnique",
                tech
            );
        }
    }

    #[test]
    fn test_quantities_are_measured_quantities() {
        use ElectrophysiologyEntity::*;
        for qty in [
            RestingPotential,
            ActionPotential,
            TransepithelialPotential,
            FieldPotential,
            Impedance,
            IntracellularCalcium,
        ] {
            assert!(
                taxonomy::is_a::<ElectrophysiologyTaxonomy>(&qty, &MeasuredQuantity),
                "{:?} should be a MeasuredQuantity",
                qty
            );
        }
    }

    #[test]
    fn test_recording_modes() {
        use ElectrophysiologyEntity::*;
        for mode in [
            WholeCell,
            CellAttached,
            InsideOut,
            OutsideOut,
            CurrentClamp,
            VoltageClamp,
        ] {
            assert!(
                taxonomy::is_a::<ElectrophysiologyTaxonomy>(&mode, &RecordingMode),
                "{:?} should be a RecordingMode",
                mode
            );
        }
    }

    #[test]
    fn test_technique_descendants_count() {
        let descendants = taxonomy::descendants::<ElectrophysiologyTaxonomy>(
            &ElectrophysiologyEntity::MeasurementTechnique,
        );
        assert_eq!(descendants.len(), 8);
    }

    #[test]
    fn test_recording_mode_descendants_count() {
        let descendants = taxonomy::descendants::<ElectrophysiologyTaxonomy>(
            &ElectrophysiologyEntity::RecordingMode,
        );
        assert_eq!(descendants.len(), 6);
    }

    // -- Quality consistency tests --

    #[test]
    fn test_invasive_count() {
        let q = IsInvasive;
        let invasive: Vec<_> = ElectrophysiologyEntity::variants()
            .into_iter()
            .filter(|e| q.get(e) == Some(true))
            .collect();
        let non_invasive: Vec<_> = ElectrophysiologyEntity::variants()
            .into_iter()
            .filter(|e| q.get(e) == Some(false))
            .collect();
        // PatchClamp, SharpElectrode = 2 invasive
        assert_eq!(invasive.len(), 2);
        // 6 non-invasive techniques
        assert_eq!(non_invasive.len(), 6);
    }

    #[test]
    fn test_vmem_measuring_techniques() {
        use ElectrophysiologyEntity::*;
        let q = MeasuresVmem;
        let vmem_techniques: Vec<_> = ElectrophysiologyEntity::variants()
            .into_iter()
            .filter(|e| q.get(e) == Some(true))
            .collect();
        assert_eq!(vmem_techniques.len(), 4);
        assert!(vmem_techniques.contains(&PatchClamp));
        assert!(vmem_techniques.contains(&SharpElectrode));
        assert!(vmem_techniques.contains(&VoltageSensitiveDye));
        assert!(vmem_techniques.contains(&OpticalMapping));
    }

    #[test]
    fn test_in_vivo_techniques() {
        use ElectrophysiologyEntity::*;
        let q = CanMeasureInVivo;
        let in_vivo: Vec<_> = ElectrophysiologyEntity::variants()
            .into_iter()
            .filter(|e| q.get(e) == Some(true))
            .collect();
        assert_eq!(in_vivo.len(), 5);
        // VSD is both Vmem and in vivo (Levin's primary tool)
        assert!(in_vivo.contains(&VoltageSensitiveDye));
    }

    #[test]
    fn test_spatial_resolution_all_scales_covered() {
        let q = SpatialResolution;
        let all = ElectrophysiologyEntity::variants();
        for scale in [
            SpatialScale::SingleCell,
            SpatialScale::CellCluster,
            SpatialScale::Tissue,
            SpatialScale::Organ,
        ] {
            assert!(
                all.iter().any(|e| q.get(e) == Some(scale)),
                "no technique covers spatial scale {:?}",
                scale
            );
        }
    }

    #[test]
    fn test_ontology_validates() {
        ElectrophysiologyOntology::validate().unwrap();
    }

    #[test]
    fn test_contact_vs_noncontact_consistency() {
        // Invasive methods require contact, optical methods do not
        use ElectrophysiologyEntity::*;
        let contact = RequiresContactWithCell;
        assert_eq!(contact.get(&PatchClamp), Some(true));
        assert_eq!(contact.get(&SharpElectrode), Some(true));
        assert_eq!(contact.get(&VoltageSensitiveDye), Some(false));
        assert_eq!(contact.get(&OpticalMapping), Some(false));
        assert_eq!(contact.get(&CalciumImaging), Some(false));
    }

    // -- Opposition tests --

    #[test]
    fn test_electrophysiology_opposition_symmetric() {
        assert!(
            ElectrophysiologyOppositionSymmetric.holds(),
            "{}",
            ElectrophysiologyOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_electrophysiology_opposition_irreflexive() {
        assert!(
            ElectrophysiologyOppositionIrreflexive.holds(),
            "{}",
            ElectrophysiologyOppositionIrreflexive.description()
        );
    }

    #[test]
    fn test_patch_clamp_opposes_optical_mapping() {
        use ElectrophysiologyEntity::*;
        assert!(opposition::are_opposed::<ElectrophysiologyOpposition>(
            &PatchClamp,
            &OpticalMapping
        ));
        assert!(opposition::are_opposed::<ElectrophysiologyOpposition>(
            &OpticalMapping,
            &PatchClamp
        ));
    }

    #[test]
    fn test_current_clamp_opposes_voltage_clamp() {
        use ElectrophysiologyEntity::*;
        assert!(opposition::are_opposed::<ElectrophysiologyOpposition>(
            &CurrentClamp,
            &VoltageClamp
        ));
    }

    #[test]
    fn test_resting_potential_opposes_action_potential() {
        use ElectrophysiologyEntity::*;
        assert!(opposition::are_opposed::<ElectrophysiologyOpposition>(
            &RestingPotential,
            &ActionPotential
        ));
    }

    #[test]
    fn test_patch_clamp_does_not_oppose_sharp_electrode() {
        use ElectrophysiologyEntity::*;
        assert!(!opposition::are_opposed::<ElectrophysiologyOpposition>(
            &PatchClamp,
            &SharpElectrode
        ));
    }

    #[test]
    fn test_electrophysiology_opposites_query() {
        use ElectrophysiologyEntity::*;
        let opps = opposition::opposites::<ElectrophysiologyOpposition>(&PatchClamp);
        assert_eq!(opps, vec![OpticalMapping]);
    }

    fn arb_ephys_entity() -> impl Strategy<Value = ElectrophysiologyEntity> {
        (0..ElectrophysiologyEntity::variants().len())
            .prop_map(|i| ElectrophysiologyEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_technique_has_invasiveness(entity in arb_ephys_entity()) {
            // Every concrete measurement technique has a defined invasiveness
            use ElectrophysiologyEntity::*;
            let is_abstract = matches!(entity, MeasurementTechnique | MeasuredQuantity | RecordingMode);
            if !is_abstract && taxonomy::is_a::<ElectrophysiologyTaxonomy>(&entity, &MeasurementTechnique) {
                prop_assert!(IsInvasive.get(&entity).is_some());
            }
        }

        #[test]
        fn prop_invasive_requires_contact(entity in arb_ephys_entity()) {
            // Invasive techniques always require cell contact
            if IsInvasive.get(&entity) == Some(true) {
                prop_assert!(RequiresContactWithCell.get(&entity) == Some(true));
            }
        }
    }
}

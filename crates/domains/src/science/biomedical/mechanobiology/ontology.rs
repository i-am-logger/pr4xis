//! Mechanobiology ontology.
//!
//! Entities: mechanical forces, mechanosensitive channels, channel states,
//! frequency response properties, and cellular responses.
//! Taxonomy: property type hierarchy (force, channel state, frequency, response).
//! Causal graph: mechanical load -> membrane deformation -> channel gating ->
//!   ion influx -> intracellular signaling; repetitive stimulus -> channel
//!   inactivation -> frequency-dependent response.
//!
//! Key references:
//! - Lewis et al 2017 (PMID:28636944): Piezo1/2 frequency filtering
//! - PMID:37459546 (2023): Piezo1 membrane stretch threshold lambda=1.9
//! - Coste 2010: Piezo1 discovery (2021 Nobel Prize)

use praxis::category::{Category, Entity, Relationship};
use praxis::ontology::reasoning::causation::{self, CausalDef};
use praxis::ontology::reasoning::opposition::{self, OppositionDef};
use praxis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use praxis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every entity in the mechanobiology domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MechanobiologyEntity {
    // Forces
    MembraneTension,
    ShearStress,
    CompressiveStress,
    TensileStress,
    SubstrateStiffness,
    // Mechanotransduction
    MechanosensitiveChannel,
    ChannelConformation,
    OpenState,
    ClosedState,
    InactivatedState,
    // Frequency response
    FrequencyFiltering,
    ActivationThreshold,
    InactivationKinetics,
    RecoveryTime,
    // Cellular responses
    CalciumTransient,
    CytoskeletalRemodeling,
    FocalAdhesion,
    Mechanoadaptation,
    // Abstract categories
    MechanicalForce,
    ChannelState,
    FrequencyProperty,
    CellularResponse,
}

impl Entity for MechanobiologyEntity {
    fn variants() -> Vec<Self> {
        use MechanobiologyEntity::*;
        vec![
            MembraneTension,
            ShearStress,
            CompressiveStress,
            TensileStress,
            SubstrateStiffness,
            MechanosensitiveChannel,
            ChannelConformation,
            OpenState,
            ClosedState,
            InactivatedState,
            FrequencyFiltering,
            ActivationThreshold,
            InactivationKinetics,
            RecoveryTime,
            CalciumTransient,
            CytoskeletalRemodeling,
            FocalAdhesion,
            Mechanoadaptation,
            MechanicalForce,
            ChannelState,
            FrequencyProperty,
            CellularResponse,
        ]
    }
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for mechanobiology entities.
pub struct MechanobiologyTaxonomy;

impl TaxonomyDef for MechanobiologyTaxonomy {
    type Entity = MechanobiologyEntity;

    fn relations() -> Vec<(MechanobiologyEntity, MechanobiologyEntity)> {
        use MechanobiologyEntity::*;
        vec![
            // Forces is-a MechanicalForce
            (MembraneTension, MechanicalForce),
            (ShearStress, MechanicalForce),
            (CompressiveStress, MechanicalForce),
            (TensileStress, MechanicalForce),
            (SubstrateStiffness, MechanicalForce),
            // Channel states is-a ChannelState
            (OpenState, ChannelState),
            (ClosedState, ChannelState),
            (InactivatedState, ChannelState),
            // Mechanotransduction components -> ChannelState (channel-related)
            (MechanosensitiveChannel, ChannelState),
            (ChannelConformation, ChannelState),
            // Frequency properties is-a FrequencyProperty
            (FrequencyFiltering, FrequencyProperty),
            (ActivationThreshold, FrequencyProperty),
            (InactivationKinetics, FrequencyProperty),
            (RecoveryTime, FrequencyProperty),
            // Cellular responses is-a CellularResponse
            (CalciumTransient, CellularResponse),
            (CytoskeletalRemodeling, CellularResponse),
            (FocalAdhesion, CellularResponse),
            (Mechanoadaptation, CellularResponse),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

/// A morphism in the mechanobiology category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanobiologyRelation {
    pub from: MechanobiologyEntity,
    pub to: MechanobiologyEntity,
}

impl Relationship for MechanobiologyRelation {
    type Object = MechanobiologyEntity;

    fn source(&self) -> MechanobiologyEntity {
        self.from
    }

    fn target(&self) -> MechanobiologyEntity {
        self.to
    }
}

/// Discrete category over mechanobiology entities.
pub struct MechanobiologyCategory;

impl Category for MechanobiologyCategory {
    type Object = MechanobiologyEntity;
    type Morphism = MechanobiologyRelation;

    fn identity(obj: &MechanobiologyEntity) -> MechanobiologyRelation {
        MechanobiologyRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(
        f: &MechanobiologyRelation,
        g: &MechanobiologyRelation,
    ) -> Option<MechanobiologyRelation> {
        if f.to != g.from {
            return None;
        }
        Some(MechanobiologyRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<MechanobiologyRelation> {
        let variants = MechanobiologyEntity::variants();
        let mut morphisms = Vec::new();
        for &from in &variants {
            for &to in &variants {
                morphisms.push(MechanobiologyRelation { from, to });
            }
        }
        morphisms
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Events in the mechanobiology causal chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MechanobiologyCausalEvent {
    /// External mechanical load applied to tissue
    MechanicalLoad,
    /// Cell membrane deforms under load
    MembraneDeformation,
    /// Mechanosensitive channel gates open
    ChannelGating,
    /// Ions flow through open channel
    IonInflux,
    /// Intracellular signaling cascade activates
    IntracellularSignaling,
    /// Repeated mechanical stimulus over time
    RepetitiveStimulus,
    /// Channel enters inactivated state (cannot reopen immediately)
    ChannelInactivation,
    /// Response depends on stimulus frequency (Lewis 2017)
    FrequencyDependentResponse,
    /// Sustained mechanical force applied chronically
    SustainedForce,
    /// Threshold shifts with chronic loading
    ThresholdShift,
}

impl Entity for MechanobiologyCausalEvent {
    fn variants() -> Vec<Self> {
        use MechanobiologyCausalEvent::*;
        vec![
            MechanicalLoad,
            MembraneDeformation,
            ChannelGating,
            IonInflux,
            IntracellularSignaling,
            RepetitiveStimulus,
            ChannelInactivation,
            FrequencyDependentResponse,
            SustainedForce,
            ThresholdShift,
        ]
    }
}

/// Causal graph for mechanobiology.
///
/// MechanicalLoad -> MembraneDeformation -> ChannelGating -> IonInflux ->
///   IntracellularSignaling
/// RepetitiveStimulus -> ChannelInactivation -> FrequencyDependentResponse
///   (Lewis 2017: Piezo channels are frequency filters)
/// SustainedForce -> ThresholdShift (mechanoadaptation)
pub struct MechanobiologyCauses;

impl CausalDef for MechanobiologyCauses {
    type Entity = MechanobiologyCausalEvent;

    fn relations() -> Vec<(MechanobiologyCausalEvent, MechanobiologyCausalEvent)> {
        use MechanobiologyCausalEvent::*;
        vec![
            // Main chain: load -> deformation -> gating -> influx -> signaling
            (MechanicalLoad, MembraneDeformation),
            (MembraneDeformation, ChannelGating),
            (ChannelGating, IonInflux),
            (IonInflux, IntracellularSignaling),
            // Frequency filtering path (Lewis 2017)
            (RepetitiveStimulus, ChannelInactivation),
            (ChannelInactivation, FrequencyDependentResponse),
            // Mechanoadaptation path
            (SustainedForce, MembraneDeformation),
            (SustainedForce, ThresholdShift),
        ]
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: activation threshold in mN/m for mechanosensitive channel opening.
///
/// Piezo1 requires ~1-5 mN/m membrane tension for activation
/// (PMID:37459546, Coste 2010).
#[derive(Debug, Clone)]
pub struct ActivationThresholdValue;

impl Quality for ActivationThresholdValue {
    type Individual = MechanobiologyEntity;
    type Value = f64;

    fn get(&self, individual: &MechanobiologyEntity) -> Option<f64> {
        use MechanobiologyEntity::*;
        match individual {
            MembraneTension => Some(3.0),         // ~3 mN/m midpoint for Piezo1
            MechanosensitiveChannel => Some(3.0), // canonical Piezo1 threshold
            ActivationThreshold => Some(3.0),     // the threshold concept itself
            _ => None,
        }
    }
}

/// Quality: is this entity frequency-dependent?
///
/// Lewis et al 2017 (PMID:28636944): mechanosensitive channels are frequency
/// filters -- Piezo1/2 inactivation kinetics determine which frequencies the
/// channel can follow.
#[derive(Debug, Clone)]
pub struct IsFrequencyDependent;

impl Quality for IsFrequencyDependent {
    type Individual = MechanobiologyEntity;
    type Value = bool;

    fn get(&self, individual: &MechanobiologyEntity) -> Option<bool> {
        use MechanobiologyEntity::*;
        match individual {
            MechanosensitiveChannel => Some(true), // Lewis 2017
            FrequencyFiltering => Some(true),
            InactivationKinetics => Some(true),
            RecoveryTime => Some(true),
            OpenState => Some(false), // state, not frequency-dependent
            ClosedState => Some(false),
            _ => None,
        }
    }
}

/// Quality: inactivation time in milliseconds.
///
/// Piezo1 inactivation ~15-30 ms (Lewis 2017). This determines the maximum
/// stimulus frequency the channel can follow: ~33-67 Hz.
#[derive(Debug, Clone)]
pub struct InactivationTimeMs;

impl Quality for InactivationTimeMs {
    type Individual = MechanobiologyEntity;
    type Value = f64;

    fn get(&self, individual: &MechanobiologyEntity) -> Option<f64> {
        use MechanobiologyEntity::*;
        match individual {
            MechanosensitiveChannel => Some(20.0), // ~20 ms midpoint for Piezo1
            InactivationKinetics => Some(20.0),
            _ => None,
        }
    }
}

/// Quality: does this process require membrane tension?
#[derive(Debug, Clone)]
pub struct RequiresMembraneTension;

impl Quality for RequiresMembraneTension {
    type Individual = MechanobiologyEntity;
    type Value = bool;

    fn get(&self, individual: &MechanobiologyEntity) -> Option<bool> {
        use MechanobiologyEntity::*;
        match individual {
            OpenState => Some(true), // channel opening requires tension
            MechanosensitiveChannel => Some(true),
            ChannelConformation => Some(true),
            CytoskeletalRemodeling => Some(false), // cytoskeleton remodels without direct tension
            FocalAdhesion => Some(false),          // adhesion is structural
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition (semantic contrasts)
// ---------------------------------------------------------------------------

/// Opposition pairs in the mechanobiology domain.
///
/// - OpenState <-> ClosedState: channel is either open or closed
/// - ActivationThreshold <-> Mechanoadaptation: threshold vs adaptation
///   (sustained force shifts the threshold -- mechanoadaptation)
pub struct MechanobiologyOpposition;

impl OppositionDef for MechanobiologyOpposition {
    type Entity = MechanobiologyEntity;

    fn pairs() -> Vec<(MechanobiologyEntity, MechanobiologyEntity)> {
        use MechanobiologyEntity::*;
        vec![
            (OpenState, ClosedState),
            (ActivationThreshold, Mechanoadaptation),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: mechanobiology taxonomy is a DAG.
pub struct MechanobiologyTaxonomyIsDAG;

impl Axiom for MechanobiologyTaxonomyIsDAG {
    fn description(&self) -> &str {
        "mechanobiology taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<MechanobiologyTaxonomy>::new().holds()
    }
}

/// Axiom: mechanobiology causal graph is asymmetric.
pub struct MechanobiologyCausalAsymmetric;

impl Axiom for MechanobiologyCausalAsymmetric {
    fn description(&self) -> &str {
        "mechanobiology causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<MechanobiologyCauses>::new().holds()
    }
}

/// Axiom: mechanical load transitively causes intracellular signaling.
pub struct MechanicalLoadCausesSignaling;

impl Axiom for MechanicalLoadCausesSignaling {
    fn description(&self) -> &str {
        "mechanical load transitively causes intracellular signaling \
         (full chain: load -> deformation -> gating -> influx -> signaling)"
    }

    fn holds(&self) -> bool {
        use MechanobiologyCausalEvent::*;
        let effects = causation::effects_of::<MechanobiologyCauses>(&MechanicalLoad);
        effects.contains(&IntracellularSignaling)
    }
}

/// Axiom: repetitive stimulus causes frequency-dependent response (Lewis 2017).
pub struct RepetitiveStimulusCausesFrequencyResponse;

impl Axiom for RepetitiveStimulusCausesFrequencyResponse {
    fn description(&self) -> &str {
        "repetitive stimulus causes frequency-dependent response \
         (Lewis 2017: Piezo channels are frequency filters)"
    }

    fn holds(&self) -> bool {
        use MechanobiologyCausalEvent::*;
        let effects = causation::effects_of::<MechanobiologyCauses>(&RepetitiveStimulus);
        effects.contains(&FrequencyDependentResponse)
    }
}

/// Axiom: mechanosensitive channel is frequency-dependent (Lewis 2017).
pub struct MechanosensitiveChannelIsFrequencyDependent;

impl Axiom for MechanosensitiveChannelIsFrequencyDependent {
    fn description(&self) -> &str {
        "mechanosensitive channel is frequency-dependent (Lewis 2017)"
    }

    fn holds(&self) -> bool {
        IsFrequencyDependent.get(&MechanobiologyEntity::MechanosensitiveChannel) == Some(true)
    }
}

/// Axiom: channel gating requires membrane tension.
pub struct ChannelGatingRequiresTension;

impl Axiom for ChannelGatingRequiresTension {
    fn description(&self) -> &str {
        "channel gating (open state) requires membrane tension"
    }

    fn holds(&self) -> bool {
        RequiresMembraneTension.get(&MechanobiologyEntity::OpenState) == Some(true)
    }
}

/// Axiom: sustained force causes mechanoadaptation (threshold shifts with chronic loading).
pub struct SustainedForceCausesAdaptation;

impl Axiom for SustainedForceCausesAdaptation {
    fn description(&self) -> &str {
        "sustained force causes threshold shift (mechanoadaptation)"
    }

    fn holds(&self) -> bool {
        use MechanobiologyCausalEvent::*;
        let effects = causation::effects_of::<MechanobiologyCauses>(&SustainedForce);
        effects.contains(&ThresholdShift)
    }
}

/// Axiom: OpenState opposes ClosedState.
pub struct OpenOpposesClosedState;

impl Axiom for OpenOpposesClosedState {
    fn description(&self) -> &str {
        "open state opposes closed state (mutually exclusive channel conformations)"
    }

    fn holds(&self) -> bool {
        use MechanobiologyEntity::*;
        opposition::are_opposed::<MechanobiologyOpposition>(&OpenState, &ClosedState)
    }
}

/// Axiom: mechanobiology opposition is symmetric.
pub struct MechanobiologyOppositionSymmetric;

impl Axiom for MechanobiologyOppositionSymmetric {
    fn description(&self) -> &str {
        "mechanobiology opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<MechanobiologyOpposition>::new().holds()
    }
}

/// Axiom: mechanobiology opposition is irreflexive.
pub struct MechanobiologyOppositionIrreflexive;

impl Axiom for MechanobiologyOppositionIrreflexive {
    fn description(&self) -> &str {
        "mechanobiology opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<MechanobiologyOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level mechanobiology ontology tying together category, qualities, and axioms.
pub struct MechanobiologyOntology;

impl Ontology for MechanobiologyOntology {
    type Cat = MechanobiologyCategory;
    type Qual = ActivationThresholdValue;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(MechanobiologyTaxonomyIsDAG),
            Box::new(MechanobiologyCausalAsymmetric),
            Box::new(MechanicalLoadCausesSignaling),
            Box::new(RepetitiveStimulusCausesFrequencyResponse),
            Box::new(MechanosensitiveChannelIsFrequencyDependent),
            Box::new(ChannelGatingRequiresTension),
            Box::new(SustainedForceCausesAdaptation),
            Box::new(OpenOpposesClosedState),
            Box::new(MechanobiologyOppositionSymmetric),
            Box::new(MechanobiologyOppositionIrreflexive),
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::validate::check_category_laws;
    use praxis::ontology::reasoning::taxonomy::TaxonomyCategory;

    // -- Axiom tests --

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(
            MechanobiologyTaxonomyIsDAG.holds(),
            "{}",
            MechanobiologyTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_causal_asymmetric() {
        assert!(
            MechanobiologyCausalAsymmetric.holds(),
            "{}",
            MechanobiologyCausalAsymmetric.description()
        );
    }

    #[test]
    fn test_mechanical_load_causes_signaling() {
        assert!(
            MechanicalLoadCausesSignaling.holds(),
            "{}",
            MechanicalLoadCausesSignaling.description()
        );
    }

    #[test]
    fn test_repetitive_stimulus_causes_frequency_response() {
        assert!(
            RepetitiveStimulusCausesFrequencyResponse.holds(),
            "{}",
            RepetitiveStimulusCausesFrequencyResponse.description()
        );
    }

    #[test]
    fn test_mechanosensitive_channel_is_frequency_dependent() {
        assert!(
            MechanosensitiveChannelIsFrequencyDependent.holds(),
            "{}",
            MechanosensitiveChannelIsFrequencyDependent.description()
        );
    }

    #[test]
    fn test_channel_gating_requires_tension() {
        assert!(
            ChannelGatingRequiresTension.holds(),
            "{}",
            ChannelGatingRequiresTension.description()
        );
    }

    #[test]
    fn test_sustained_force_causes_adaptation() {
        assert!(
            SustainedForceCausesAdaptation.holds(),
            "{}",
            SustainedForceCausesAdaptation.description()
        );
    }

    #[test]
    fn test_open_opposes_closed() {
        assert!(
            OpenOpposesClosedState.holds(),
            "{}",
            OpenOpposesClosedState.description()
        );
    }

    #[test]
    fn test_opposition_symmetric() {
        assert!(
            MechanobiologyOppositionSymmetric.holds(),
            "{}",
            MechanobiologyOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_opposition_irreflexive() {
        assert!(
            MechanobiologyOppositionIrreflexive.holds(),
            "{}",
            MechanobiologyOppositionIrreflexive.description()
        );
    }

    // -- Category law tests --

    #[test]
    fn test_mechanobiology_category_laws() {
        check_category_laws::<MechanobiologyCategory>().unwrap();
    }

    #[test]
    fn test_mechanobiology_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<MechanobiologyTaxonomy>>().unwrap();
    }

    #[test]
    fn test_mechanobiology_causal_category_laws() {
        use praxis::ontology::reasoning::causation::CausalCategory;
        check_category_laws::<CausalCategory<MechanobiologyCauses>>().unwrap();
    }

    // -- Taxonomy tests --

    #[test]
    fn test_forces_are_mechanical_force() {
        use MechanobiologyEntity::*;
        for entity in [
            MembraneTension,
            ShearStress,
            CompressiveStress,
            TensileStress,
            SubstrateStiffness,
        ] {
            assert!(
                taxonomy::is_a::<MechanobiologyTaxonomy>(&entity, &MechanicalForce),
                "{:?} should be a MechanicalForce",
                entity
            );
        }
    }

    #[test]
    fn test_channel_states_are_channel_state() {
        use MechanobiologyEntity::*;
        for entity in [
            OpenState,
            ClosedState,
            InactivatedState,
            MechanosensitiveChannel,
            ChannelConformation,
        ] {
            assert!(
                taxonomy::is_a::<MechanobiologyTaxonomy>(&entity, &ChannelState),
                "{:?} should be a ChannelState",
                entity
            );
        }
    }

    #[test]
    fn test_frequency_properties_are_frequency() {
        use MechanobiologyEntity::*;
        for entity in [
            FrequencyFiltering,
            ActivationThreshold,
            InactivationKinetics,
            RecoveryTime,
        ] {
            assert!(
                taxonomy::is_a::<MechanobiologyTaxonomy>(&entity, &FrequencyProperty),
                "{:?} should be a FrequencyProperty",
                entity
            );
        }
    }

    #[test]
    fn test_cellular_responses_are_response() {
        use MechanobiologyEntity::*;
        for entity in [
            CalciumTransient,
            CytoskeletalRemodeling,
            FocalAdhesion,
            Mechanoadaptation,
        ] {
            assert!(
                taxonomy::is_a::<MechanobiologyTaxonomy>(&entity, &CellularResponse),
                "{:?} should be a CellularResponse",
                entity
            );
        }
    }

    #[test]
    fn test_entity_count() {
        assert_eq!(MechanobiologyEntity::variants().len(), 22);
    }

    #[test]
    fn test_causal_event_count() {
        assert_eq!(MechanobiologyCausalEvent::variants().len(), 10);
    }

    // -- Opposition tests --

    #[test]
    fn test_open_opposes_closed_state() {
        use MechanobiologyEntity::*;
        assert!(opposition::are_opposed::<MechanobiologyOpposition>(
            &OpenState,
            &ClosedState
        ));
        assert!(opposition::are_opposed::<MechanobiologyOpposition>(
            &ClosedState,
            &OpenState
        ));
    }

    #[test]
    fn test_threshold_opposes_adaptation() {
        use MechanobiologyEntity::*;
        assert!(opposition::are_opposed::<MechanobiologyOpposition>(
            &ActivationThreshold,
            &Mechanoadaptation
        ));
    }

    #[test]
    fn test_open_does_not_oppose_inactivated() {
        use MechanobiologyEntity::*;
        assert!(!opposition::are_opposed::<MechanobiologyOpposition>(
            &OpenState,
            &InactivatedState
        ));
    }

    // -- Quality tests --

    #[test]
    fn test_membrane_tension_threshold() {
        let t = ActivationThresholdValue.get(&MechanobiologyEntity::MembraneTension);
        assert_eq!(t, Some(3.0));
    }

    #[test]
    fn test_channel_threshold() {
        let t = ActivationThresholdValue.get(&MechanobiologyEntity::MechanosensitiveChannel);
        assert_eq!(t, Some(3.0));
    }

    #[test]
    fn test_channel_is_frequency_dependent() {
        assert_eq!(
            IsFrequencyDependent.get(&MechanobiologyEntity::MechanosensitiveChannel),
            Some(true)
        );
    }

    #[test]
    fn test_open_state_not_frequency_dependent() {
        assert_eq!(
            IsFrequencyDependent.get(&MechanobiologyEntity::OpenState),
            Some(false)
        );
    }

    #[test]
    fn test_inactivation_time() {
        let t = InactivationTimeMs.get(&MechanobiologyEntity::MechanosensitiveChannel);
        assert_eq!(t, Some(20.0));
    }

    #[test]
    fn test_open_state_requires_tension() {
        assert_eq!(
            RequiresMembraneTension.get(&MechanobiologyEntity::OpenState),
            Some(true)
        );
    }

    #[test]
    fn test_cytoskeletal_remodeling_no_tension() {
        assert_eq!(
            RequiresMembraneTension.get(&MechanobiologyEntity::CytoskeletalRemodeling),
            Some(false)
        );
    }

    // -- Causal chain tests --

    #[test]
    fn test_mechanical_load_full_chain() {
        use MechanobiologyCausalEvent::*;
        let effects = causation::effects_of::<MechanobiologyCauses>(&MechanicalLoad);
        assert!(effects.contains(&MembraneDeformation));
        assert!(effects.contains(&ChannelGating));
        assert!(effects.contains(&IonInflux));
        assert!(effects.contains(&IntracellularSignaling));
    }

    #[test]
    fn test_repetitive_stimulus_frequency_path() {
        use MechanobiologyCausalEvent::*;
        let effects = causation::effects_of::<MechanobiologyCauses>(&RepetitiveStimulus);
        assert!(effects.contains(&ChannelInactivation));
        assert!(effects.contains(&FrequencyDependentResponse));
    }

    #[test]
    fn test_sustained_force_causes_threshold_shift() {
        use MechanobiologyCausalEvent::*;
        let effects = causation::effects_of::<MechanobiologyCauses>(&SustainedForce);
        assert!(effects.contains(&ThresholdShift));
    }

    #[test]
    fn test_sustained_force_also_causes_signaling() {
        use MechanobiologyCausalEvent::*;
        let effects = causation::effects_of::<MechanobiologyCauses>(&SustainedForce);
        // SustainedForce -> MembraneDeformation -> ChannelGating -> IonInflux -> Signaling
        assert!(effects.contains(&IntracellularSignaling));
    }

    // -- Ontology validation --

    #[test]
    fn test_ontology_validates() {
        MechanobiologyOntology::validate().unwrap();
    }

    // -- Property-based tests (proptest) --

    use proptest::prelude::*;

    fn arb_mechanobiology_entity() -> impl Strategy<Value = MechanobiologyEntity> {
        (0..MechanobiologyEntity::variants().len())
            .prop_map(|i| MechanobiologyEntity::variants()[i])
    }

    fn arb_mechanobiology_causal_event() -> impl Strategy<Value = MechanobiologyCausalEvent> {
        (0..MechanobiologyCausalEvent::variants().len())
            .prop_map(|i| MechanobiologyCausalEvent::variants()[i])
    }

    proptest! {
        /// For any MechanobiologyEntity, taxonomy is-a is reflexive.
        #[test]
        fn prop_taxonomy_is_a_reflexive(entity in arb_mechanobiology_entity()) {
            prop_assert!(
                taxonomy::is_a::<MechanobiologyTaxonomy>(&entity, &entity),
                "is-a should be reflexive for {:?}",
                entity
            );
        }

        /// ActivationThresholdValue is always positive when defined.
        #[test]
        fn prop_threshold_always_positive(entity in arb_mechanobiology_entity()) {
            if let Some(t) = ActivationThresholdValue.get(&entity) {
                prop_assert!(t > 0.0, "activation threshold must be positive for {:?}", entity);
            }
        }

        /// InactivationTimeMs is always positive when defined.
        #[test]
        fn prop_inactivation_time_positive(entity in arb_mechanobiology_entity()) {
            if let Some(t) = InactivationTimeMs.get(&entity) {
                prop_assert!(t > 0.0, "inactivation time must be positive for {:?}", entity);
            }
        }

        /// No causal event causes itself (checked via proptest).
        #[test]
        fn prop_no_self_causation(event in arb_mechanobiology_causal_event()) {
            let relations = MechanobiologyCauses::relations();
            let self_caused = relations.iter().any(|(a, b)| *a == event && *b == event);
            prop_assert!(
                !self_caused,
                "event {:?} should not directly cause itself",
                event
            );
        }
    }
}

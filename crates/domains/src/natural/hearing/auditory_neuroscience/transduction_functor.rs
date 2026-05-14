//! Functor: TransductionCategory → NeuroscienceCategory.
//!
//! Maps hair-cell-transduction mechanisms to their neural-processing
//! roles.
//!
//! Citation: Hudspeth (2014) *Nat. Rev. Neurosci.* 15(9):600 — the
//! transduction-to-neural pipeline; Schnupp et al. (2011) *Auditory
//! Neuroscience* — neural-side framing.

use crate::natural::hearing::auditory_neuroscience::ontology::*;
use crate::natural::hearing::transduction::ontology::*;
use pr4xis::category::{Arrow, Functor};

pub struct TransductionToNeuroscience;

impl Functor for TransductionToNeuroscience {
    type Source = TransductionCategory;
    type Target = NeuroscienceCategory;

    fn map_object(obj: &TransductionEntity) -> NeuralEntity {
        use NeuralEntity::*;
        use TransductionEntity as T;
        match obj {
            T::ActionPotential => AuditoryNerveFiber,
            T::GlutamateRelease | T::Glutamate => RateCoding,
            T::Depolarization | T::ReceptorPotential => RateLevelFunction,
            T::METChannel
            | T::TMC1
            | T::TMC2
            | T::TMIE
            | T::LHFPL5
            | T::METChannelOpening
            | T::METComponent => FrequencyTuningCurve,
            T::Stereocilium
            | T::StereociliaBundle
            | T::CuticularPlate
            | T::Kinocilium
            | T::StereociliaDeflection => PlaceCoding,
            T::TipLink
            | T::Cadherin23
            | T::Protocadherin15
            | T::TipLinkTension
            | T::TipLinkProtein => CharacteristicFrequency,
            T::Prestin | T::Electromotility | T::CochlearAmplification => DynamicRange,
            T::PotassiumInflux
            | T::CalciumInflux
            | T::Potassium
            | T::Calcium
            | T::KCNQ4
            | T::CaV1_3
            | T::BKChannel
            | T::IonChannel => SpontaneousRate,
            T::EndocochlearPotential => Adaptation,
            T::CellularSignal => OnsetResponse,
            // Transduction events → neural events.
            T::BasilarMembraneMotion => AuditoryNerveInput,
            T::StereociliaBundleDeflection | T::TipLinkStretch => CochlearNucleusIntegration,
            T::METChannelGating | T::PotassiumEntry => BinauralConvergence,
            T::CellDepolarization => LemniscalRelay,
            T::CalciumEntry => MultisensoryIntegration,
            T::VesicleRelease => ThalamicGating,
            T::NerveActivation => CorticalAnalysis,
            T::PrestiConformationChange | T::CellLengthChange => StreamFormation,
            T::BasilarMembraneAmplification => PerceptualBinding,
            T::SlowAdaptation | T::FastAdaptation => Adaptation,
            T::TransductionEvent => NeuralEvent,
        }
    }

    fn map_morphism(m: &TransductionRelation) -> NeuralRelation {
        use NeuralRelationKind as Tk;
        use TransductionCategoryRelationKind as Sk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        NeuralRelation { from, to, kind }
    }
}
pr4xis::register_functor!(TransductionToNeuroscience);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;
    use pr4xis::category::laws::assert_functor_laws;

    #[test]
    fn functor_laws() {
        assert_functor_laws::<TransductionToNeuroscience>();
    }
    #[test]
    fn action_potential_maps_to_an_fiber() {
        assert_eq!(
            TransductionToNeuroscience::map_object(&TransductionEntity::ActionPotential),
            NeuralEntity::AuditoryNerveFiber
        );
    }
    #[test]
    fn every_entity_maps_valid() {
        let targets = NeuralEntity::variants();
        for obj in TransductionEntity::variants() {
            assert!(targets.contains(&TransductionToNeuroscience::map_object(&obj)));
        }
    }
}

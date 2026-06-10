//! Functor: AcousticsCategory → SignalProcessingCategory.
//!
//! Maps acoustic physics to computational analysis representations.
//!
//! Citation: Oppenheim & Schafer (2010) *Discrete-Time Signal Processing*
//! grounds the spectral / windowing concepts; Kinsler et al. (2000)
//! *Fundamentals of Acoustics* grounds the acoustic side.
//!
//! `map_morphism` preserves the canonical relation kind (Identity /
//! Subsumption / Parthood / Causation / Opposition) per Mac Lane (1971)
//! CWM Ch. II §1: `F(g∘f) = F(g)∘F(f)`. Same-kind transitive composition
//! in source (OBO-RO `transitive_over`, Smith 2005) inherits the kind in
//! target, so the law holds for every composable pair.

use crate::natural::hearing::acoustics::ontology::*;
use crate::natural::hearing::signal_processing::ontology::*;
use pr4xis::category::{Arrow, Functor};

pub struct AcousticsToSignalProcessing;

impl Functor for AcousticsToSignalProcessing {
    type Source = AcousticsCategory;
    type Target = SignalProcessingCategory;

    fn map_object(obj: &AcousticEntity) -> SignalEntity {
        use AcousticEntity as A;
        use SignalEntity::*;
        match obj {
            A::Frequency | A::WaveProperty => FrequencyDomain,
            A::Amplitude | A::Intensity => PowerSpectralDensity,
            A::Wavelength => FourierTransform,
            A::Phase => HilbertTransform,
            A::SoundWave | A::LongitudinalWave | A::Wave => TimeDomain,
            A::TransverseWave | A::ShearWave => TimeDomain,
            A::Air | A::Water | A::SoftTissue | A::Cartilage | A::Fluid | A::Medium => Sampling,
            A::CorticalBone | A::CancellousBone | A::Solid | A::BoneTissue => Sampling,
            A::Resonance => BandPassFilter,
            A::Reflection => Autocorrelation,
            A::Refraction => WindowFunction,
            A::Diffraction => WaveletTransform,
            A::Absorption | A::Attenuation => LowPassFilter,
            A::ImpedanceMismatch => HighPassFilter,
            A::AcousticPhenomenon => Transform,
            // Acoustic events → signal-pipeline events.
            A::SourceVibration | A::MediumCoupling => RawSignal,
            A::WavePropagation | A::BoundaryEncounter | A::ImpedanceTransition => SpectralTransform,
            A::EnergyReflection | A::EnergyTransmission | A::EnergyAbsorption => FeatureExtraction,
            A::WaveAttenuation => Decimation,
            A::ResonantAmplification => Interpolation,
            A::ReceiverExcitation => PatternClassification,
            A::AcousticEvent => SignalEvent,
        }
    }

    fn map_morphism(m: &AcousticRelation) -> SignalRelation {
        use AcousticsCategoryRelationKind as Sk;
        use SignalRelationKind as Tk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        SignalRelation { from, to, kind }
    }
}
pr4xis::register_functor!(AcousticsToSignalProcessing);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[test]
    fn functor_laws() {
        assert_functor_laws::<AcousticsToSignalProcessing>();
    }

    #[test]
    fn frequency_maps_to_freq_domain() {
        assert_eq!(
            AcousticsToSignalProcessing::map_object(&AcousticEntity::Frequency),
            SignalEntity::FrequencyDomain
        );
    }

    #[test]
    fn every_entity_maps_valid() {
        let targets = SignalEntity::variants();
        for obj in AcousticEntity::variants() {
            assert!(targets.contains(&AcousticsToSignalProcessing::map_object(&obj)));
        }
    }
}

//! Adjunctions between pssst domains.
//!
//! Each adjunction captures a pair of "optimally inverse" functors.
//! The unit η and counit ε capture information loss in the round trip.
//!
//! Four adjunctions model fundamental dualities in hearing science:
//!
//! 1. Analysis ⊣ Synthesis: Acoustics ↔ Signal Processing
//!    Decomposing sound into components vs reconstructing from components.
//!    Information lost: phase relationships, transient details.
//!
//! 2. Health ⊣ Disease: Anatomy ↔ Pathology
//!    Normal structure vs pathological state.
//!    Information lost: individual variation, comorbidities.
//!
//! 3. Bottom-up ⊣ Top-down: Psychoacoustics ↔ Music Perception
//!    Stimulus-driven perception vs expectation-driven cognition.
//!    Information lost: context (bottom-up), detail (top-down).
//!
//! 4. Diagnosis ⊣ Treatment: Pathology ↔ Devices
//!    Identifying disorder vs applying intervention.
//!    Information lost: exact etiology (diagnosis), complete correction (treatment).
//!
//! References:
//! - Mac Lane 1971: Categories for the Working Mathematician, Ch. IV
//! - Awodey 2010: Category Theory, Ch. 9

use pr4xis::category::{Adjunction, Functor, Relationship};

// =============================================================================
// 1. Analysis ⊣ Synthesis (Acoustics ↔ Signal Processing)
// =============================================================================

use crate::science::hearing::acoustics::ontology::*;
use crate::science::hearing::signal_processing::ontology::*;

// Left adjoint: Acoustics → Signal Processing (analysis / decomposition)
// Already defined as AcousticsToSignalProcessing in acoustics/signal_functor.rs
use crate::science::hearing::acoustics::signal_functor::AcousticsToSignalProcessing;

/// Right adjoint: Signal Processing → Acoustics (synthesis / reconstruction).
///
/// Reconstructs acoustic concepts from signal processing representations.
/// Lossy: reconstructed sound ≠ original (phase, transients lost).
pub struct SignalProcessingToAcoustics;

impl Functor for SignalProcessingToAcoustics {
    type Source = SignalProcessingCategory;
    type Target = AcousticsCategory;

    fn map_object(obj: &SignalEntity) -> AcousticEntity {
        use AcousticEntity::*;
        use SignalEntity as S;
        match obj {
            // Transforms → wave phenomena
            S::FourierTransform | S::FFT | S::InverseFFT | S::Transform => SoundWave,
            S::ShortTimeFourierTransform | S::Spectrogram => SoundWave,
            S::WaveletTransform | S::HilbertTransform | S::CepstralAnalysis => SoundWave,
            // Representations → wave properties
            S::PowerSpectralDensity => Intensity,
            S::Autocorrelation | S::Correlation => Phase,
            S::Cepstrum | S::MelFrequencyCepstrum | S::Representation => Frequency,
            // Filters → acoustic phenomena
            S::LowPassFilter
            | S::HighPassFilter
            | S::BandPassFilter
            | S::BandStopFilter
            | S::FIRFilter
            | S::IIRFilter
            | S::GammatoneFilter
            | S::Filter => Resonance,
            // Sampling → medium
            S::Sampling
            | S::NyquistFrequency
            | S::Aliasing
            | S::Quantization
            | S::SamplingConcept => Air,
            // Windows → wave properties
            S::WindowFunction
            | S::HannWindow
            | S::HammingWindow
            | S::BlackmanWindow
            | S::RectangularWindow => Amplitude,
            // Operations → phenomena
            S::Convolution | S::SignalOperation => Resonance,
            S::Decimation | S::Interpolation => Attenuation,
            // Domains → wave types
            S::TimeDomain => SoundWave,
            S::FrequencyDomain => Frequency,
            S::AnalysisDomain => Wave,
        }
    }

    fn map_morphism(m: &SignalRelation) -> AcousticRelation {
        AcousticRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

/// Analysis ⊣ Synthesis adjunction.
///
/// Left (Analysis): decompose sound into spectral components.
/// Right (Synthesis): reconstruct sound from spectral components.
///
/// Unit η_A: A → Synthesize(Analyze(A))
///   Round-trip: sound → spectrum → reconstructed sound.
///   Not identity: phase information may be lost (magnitude-only spectrogram).
///
/// Counit ε_B: Analyze(Synthesize(B)) → B
///   Round-trip: spectrum → sound → re-analyzed spectrum.
///   Close to identity when using complete (complex) representation.
pub struct AnalysisSynthesis;

impl Adjunction for AnalysisSynthesis {
    type Left = AcousticsToSignalProcessing;
    type Right = SignalProcessingToAcoustics;

    fn unit(obj: &AcousticEntity) -> AcousticRelation {
        // η_A: A → G(F(A)) — the acoustic entity embeds into its round-trip
        let analyzed = AcousticsToSignalProcessing::map_object(obj);
        let reconstructed = SignalProcessingToAcoustics::map_object(&analyzed);
        AcousticRelation {
            from: *obj,
            to: reconstructed,
        }
    }

    fn counit(obj: &SignalEntity) -> SignalRelation {
        // ε_B: F(G(B)) → B — the signal entity projects from its round-trip
        let synthesized = SignalProcessingToAcoustics::map_object(obj);
        let reanalyzed = AcousticsToSignalProcessing::map_object(&synthesized);
        SignalRelation {
            from: reanalyzed,
            to: *obj,
        }
    }
}

// =============================================================================
// 2. Health ⊣ Disease (Anatomy ↔ Pathology)
// =============================================================================

use crate::science::hearing::anatomy::ontology::*;
use crate::science::hearing::pathology::ontology::*;

/// Left adjoint: Anatomy → Pathology (what can go wrong with this structure).
pub struct AnatomyToPathology;

impl Functor for AnatomyToPathology {
    type Source = AnatomyCategory;
    type Target = PathologyCategory;

    fn map_object(obj: &AuditoryEntity) -> PathologyEntity {
        use AuditoryEntity as A;
        use PathologyEntity::*;
        match obj {
            // Outer ear → conductive
            A::Pinna | A::EarCanal => ConductiveHearingLoss,
            A::TympanicMembrane => TympanicPerforation,
            // Middle ear → conductive
            A::Malleus | A::Incus | A::Stapes | A::Ossicle => Otosclerosis,
            A::OvalWindow | A::RoundWindow => OssicularFixation,
            A::EustachianTube | A::TensorTympani | A::Stapedius => OtitisMedia,
            // Inner ear hair cells → SNHL
            A::InnerHairCell | A::OuterHairCell | A::HairCell => HairCellLoss,
            A::OrganOfCorti | A::BasilarMembrane | A::TectorialMembrane | A::CochlearMembrane => {
                StereociliaDamage
            }
            // Cochlear fluids/structures → Ménière's
            A::Endolymph | A::ScalaMedia | A::CochlearFluid => MenieresDisease,
            A::Perilymph | A::ScalaVestibuli | A::ScalaTympani => EndolymphaticHydrops,
            A::StriVascularis => StriaDysfunction,
            A::ReissnersMembrane | A::Cochlea => SensorineuralHearingLoss,
            // Neural → neuropathy
            A::SpiralGanglionNeuron | A::AuditoryNerve => AuditoryNeuropathy,
            A::CochlearNucleus
            | A::SuperiorOlivaryComplex
            | A::InferiorColliculus
            | A::MedialGeniculateBody
            | A::AuditoryCortex
            | A::AuditoryNucleus => CentralAuditoryProcessingDisorder,
            // Vestibular
            A::Vestibule | A::SemicircularCanals => MenieresDisease,
            A::SupportingCell => SynapticRibbonLoss,
            // Abstract
            A::Ear | A::OuterEar => ConductiveHearingLoss,
            A::MiddleEar => ConductiveHearingLoss,
            A::InnerEar => SensorineuralHearingLoss,
        }
    }

    fn map_morphism(m: &AuditoryRelation) -> PathologyRelation {
        PathologyRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

/// Right adjoint: Pathology → Anatomy (which structure is affected).
pub struct PathologyToAnatomy;

impl Functor for PathologyToAnatomy {
    type Source = PathologyCategory;
    type Target = AnatomyCategory;

    fn map_object(obj: &PathologyEntity) -> AuditoryEntity {
        use AuditoryEntity::*;
        use PathologyEntity as P;
        match obj {
            P::ConductiveHearingLoss | P::OtitisMedia | P::Cholesteatoma => MiddleEar,
            P::SensorineuralHearingLoss
            | P::Presbycusis
            | P::NoiseInducedHearingLoss
            | P::SuddenSensorineuralLoss => Cochlea,
            P::MixedHearingLoss => Ear,
            P::AuditoryNeuropathy | P::DemyelinationVIII => AuditoryNerve,
            P::CentralAuditoryProcessingDisorder => AuditoryCortex,
            P::Otosclerosis | P::OssicularFixation => Stapes,
            P::MenieresDisease | P::EndolymphaticHydrops => Endolymph,
            P::AcousticNeuroma => AuditoryNerve,
            P::Tinnitus | P::Hyperacusis | P::PhantomPercept => Cochlea,
            P::TympanicPerforation => TympanicMembrane,
            P::HairCellLoss | P::StereociliaDamage => OuterHairCell,
            P::SynapticRibbonLoss | P::Excitotoxicity => InnerHairCell,
            P::StriaDysfunction => StriVascularis,
            P::OxidativeStress => OrganOfCorti,
            P::ElevatedThreshold | P::ReducedFrequencySelectivity | P::LoudnessRecruitment => {
                Cochlea
            }
            P::PoorSpeechInNoise
            | P::ReducedTemporalResolution
            | P::AbnormalBinauralProcessing
            | P::PerceptualDeficit => AuditoryCortex,
            P::Audiogram
            | P::PureToneAverage
            | P::SpeechReceptionThreshold
            | P::OtoacousticEmission
            | P::AuditoryBrainstemResponse
            | P::ClinicalMeasure => Ear,
            P::HearingLoss | P::PeripheralPathology => Cochlea,
            P::CentralPathology => AuditoryCortex,
            P::DamageMechanism => OrganOfCorti,
        }
    }

    fn map_morphism(m: &PathologyRelation) -> AuditoryRelation {
        AuditoryRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

/// Health ⊣ Disease adjunction.
///
/// Left (Health→Disease): normal anatomy maps to what can go wrong.
/// Right (Disease→Health): pathology maps to affected structure.
///
/// Unit η: Structure → AffectedBy(DiseasedIn(Structure))
///   e.g., OHC → HairCellLoss → OuterHairCell (recovers the cell type)
///
/// Counit ε: DiseasedIn(AffectedBy(Disease)) → Disease
///   e.g., SNHL → Cochlea → SNHL (recovers the disorder)
pub struct HealthDisease;

impl Adjunction for HealthDisease {
    type Left = AnatomyToPathology;
    type Right = PathologyToAnatomy;

    fn unit(obj: &AuditoryEntity) -> AuditoryRelation {
        let diseased = AnatomyToPathology::map_object(obj);
        let recovered = PathologyToAnatomy::map_object(&diseased);
        AuditoryRelation {
            from: *obj,
            to: recovered,
        }
    }

    fn counit(obj: &PathologyEntity) -> PathologyRelation {
        let structure = PathologyToAnatomy::map_object(obj);
        let re_diseased = AnatomyToPathology::map_object(&structure);
        PathologyRelation {
            from: re_diseased,
            to: *obj,
        }
    }
}

// =============================================================================
// 3. Bottom-up ⊣ Top-down (Psychoacoustics ↔ Music Perception)
// =============================================================================

use crate::science::hearing::music_perception::ontology::*;
use crate::science::hearing::psychoacoustics::ontology::*;

// Left adjoint already defined: PsychoacousticsToMusic
use crate::science::hearing::psychoacoustics::music_functor::PsychoacousticsToMusic;

/// Right adjoint: Music Perception → Psychoacoustics (top-down influence).
///
/// Musical expectations and schemas modulate low-level perception.
/// Huron 2006: Sweet Anticipation.
pub struct MusicToPsychoacoustics;

impl Functor for MusicToPsychoacoustics {
    type Source = MusicPerceptionCategory;
    type Target = PsychoacousticsCategory;

    fn map_object(obj: &MusicEntity) -> PsychoacousticEntity {
        use MusicEntity as M;
        use PsychoacousticEntity::*;
        match obj {
            // Pitch percepts → pitch perception
            M::PitchHeight
            | M::PitchChroma
            | M::OctaveEquivalence
            | M::AbsolutePitch
            | M::RelativePitch
            | M::MelodicContour
            | M::IntervalPerception
            | M::PitchPercept => Pitch,
            // Harmonic → frequency selectivity
            M::Consonance
            | M::Dissonance
            | M::RoughnessModel
            | M::HarmonicSeries
            | M::VirtualPitchPercept
            | M::MissingFundamental
            | M::Chord
            | M::Tonality
            | M::KeySense
            | M::HarmonicPercept => FrequencySelectivity,
            // Rhythmic → temporal
            M::Beat
            | M::Meter
            | M::Tempo
            | M::Syncopation
            | M::Groove
            | M::Entrainment
            | M::TemporalExpectation
            | M::RhythmicPercept => TemporalResolution,
            // Timbre → timbre
            M::SpectralCentroid
            | M::AttackTime
            | M::SpectralFlux
            | M::InstrumentIdentification
            | M::TimbrePercept => Timbre,
            // Affective / expectation → loudness (arousal proxy)
            M::MusicalExpectation
            | M::Surprise
            | M::Tension
            | M::Resolution
            | M::MusicalEmotion
            | M::AffectiveResponse => Loudness,
            // Memory
            M::EarWorm | M::MusicalMemory | M::TonalSchemaMemory => Pitch,
        }
    }

    fn map_morphism(m: &MusicRelation) -> PsychoacousticRelation {
        PsychoacousticRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

/// Bottom-up ⊣ Top-down adjunction.
///
/// Left (Bottom-up): stimulus percepts → musical cognition.
/// Right (Top-down): musical schemas → modulated perception.
///
/// The unit captures how bottom-up features are interpreted musically,
/// then that musical interpretation feeds back to shape perception.
/// Not identity: expectations can override physical stimulus (e.g.,
/// missing fundamental, subjective rhythm in ambiguous patterns).
pub struct BottomUpTopDown;

impl Adjunction for BottomUpTopDown {
    type Left = PsychoacousticsToMusic;
    type Right = MusicToPsychoacoustics;

    fn unit(obj: &PsychoacousticEntity) -> PsychoacousticRelation {
        let musical = PsychoacousticsToMusic::map_object(obj);
        let feedback = MusicToPsychoacoustics::map_object(&musical);
        PsychoacousticRelation {
            from: *obj,
            to: feedback,
        }
    }

    fn counit(obj: &MusicEntity) -> MusicRelation {
        let percept = MusicToPsychoacoustics::map_object(obj);
        let re_musical = PsychoacousticsToMusic::map_object(&percept);
        MusicRelation {
            from: re_musical,
            to: *obj,
        }
    }
}

// =============================================================================
// 4. Diagnosis ⊣ Treatment (Pathology ↔ Devices)
// =============================================================================

use crate::science::hearing::devices::ontology::*;

// Left adjoint already defined: PathologyToDevices
use crate::science::hearing::pathology::devices_functor::PathologyToDevices;

/// Right adjoint: Devices → Pathology (what condition does this device treat).
pub struct DevicesToPathology;

impl Functor for DevicesToPathology {
    type Source = DeviceCategory;
    type Target = PathologyCategory;

    fn map_object(obj: &DeviceEntity) -> PathologyEntity {
        use DeviceEntity as D;
        use PathologyEntity::*;
        match obj {
            // Hearing aids → SNHL
            D::BehindTheEar
            | D::InTheEar
            | D::CompletelyInCanal
            | D::ReceiverInCanal
            | D::HearingAid => SensorineuralHearingLoss,
            // CROS → unilateral loss
            D::CROS | D::BiCROS => AbnormalBinauralProcessing,
            // Implantable → severe/profound
            D::CochlearImplant => SensorineuralHearingLoss,
            D::BoneAnchoredHearingAid => ConductiveHearingLoss,
            D::MiddleEarImplant => Otosclerosis,
            D::AuditoryBrainstemImplant => AcousticNeuroma,
            D::ImplantableDevice => SensorineuralHearingLoss,
            // BC devices → conductive
            D::BoneConductionHeadphone | D::SoftbandBAHA | D::AdhesiveBC | D::BCDevice => {
                ConductiveHearingLoss
            }
            // Signal processing → specific deficits
            D::DirectionalMicrophone => PoorSpeechInNoise,
            D::NoiseSuppression => Tinnitus,
            D::FeedbackCancellation => ElevatedThreshold,
            D::FrequencyCompression => ReducedFrequencySelectivity,
            D::WideAdaptiveDynamicRange => LoudnessRecruitment,
            D::Telecoil | D::BluetoothStreaming | D::SignalProcessingFeature => PoorSpeechInNoise,
            // Diagnostic → clinical measure
            D::Audiometer => Audiogram,
            D::Tympanometer => OtoacousticEmission,
            D::OAEProbe => OtoacousticEmission,
            D::ABRSystem => AuditoryBrainstemResponse,
            D::RealEarMeasurement | D::DiagnosticEquipment => Audiogram,
            // Components
            D::Microphone
            | D::Amplifier
            | D::Receiver
            | D::ElectrodeArray
            | D::SpeechProcessor
            | D::DeviceComponent => HearingLoss,
        }
    }

    fn map_morphism(m: &DeviceRelation) -> PathologyRelation {
        PathologyRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

/// Diagnosis ⊣ Treatment adjunction.
///
/// Left (Diagnosis): disorder → recommended device.
/// Right (Treatment): device → condition it treats.
///
/// Unit η: Disorder → TreatedBy(DeviceFor(Disorder))
///   e.g., SNHL → BTE → SNHL (recovers the condition)
///
/// Counit ε: DeviceFor(TreatedBy(Device)) → Device
///   e.g., BTE → SNHL → BTE (recovers the device type)
///
/// Not exact inverses: one device may treat multiple conditions,
/// one condition may be treated by multiple devices.
pub struct DiagnosisTreatment;

impl Adjunction for DiagnosisTreatment {
    type Left = PathologyToDevices;
    type Right = DevicesToPathology;

    fn unit(obj: &PathologyEntity) -> PathologyRelation {
        let device = PathologyToDevices::map_object(obj);
        let condition = DevicesToPathology::map_object(&device);
        PathologyRelation {
            from: *obj,
            to: condition,
        }
    }

    fn counit(obj: &DeviceEntity) -> DeviceRelation {
        let condition = DevicesToPathology::map_object(obj);
        let device = PathologyToDevices::map_object(&condition);
        DeviceRelation {
            from: device,
            to: *obj,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    // -- Right adjoint functor law tests --

    #[test]
    fn test_signal_to_acoustics_functor_laws() {
        check_functor_laws::<SignalProcessingToAcoustics>().unwrap();
    }
    #[test]
    fn test_signal_to_acoustics_analogy() {
        Analogy::<SignalProcessingToAcoustics>::validate().unwrap();
    }

    #[test]
    fn test_pathology_to_anatomy_functor_laws() {
        check_functor_laws::<PathologyToAnatomy>().unwrap();
    }
    #[test]
    fn test_pathology_to_anatomy_analogy() {
        Analogy::<PathologyToAnatomy>::validate().unwrap();
    }

    #[test]
    fn test_music_to_psychoacoustics_functor_laws() {
        check_functor_laws::<MusicToPsychoacoustics>().unwrap();
    }
    #[test]
    fn test_music_to_psychoacoustics_analogy() {
        Analogy::<MusicToPsychoacoustics>::validate().unwrap();
    }

    #[test]
    fn test_devices_to_pathology_functor_laws() {
        check_functor_laws::<DevicesToPathology>().unwrap();
    }
    #[test]
    fn test_devices_to_pathology_analogy() {
        Analogy::<DevicesToPathology>::validate().unwrap();
    }

    #[test]
    fn test_anatomy_to_pathology_functor_laws() {
        check_functor_laws::<AnatomyToPathology>().unwrap();
    }
    #[test]
    fn test_anatomy_to_pathology_analogy() {
        Analogy::<AnatomyToPathology>::validate().unwrap();
    }

    // -- Adjunction unit/counit well-formedness --

    #[test]
    fn test_analysis_synthesis_unit() {
        // For every acoustic entity, unit produces a valid morphism
        for obj in AcousticEntity::variants() {
            let morphism = AnalysisSynthesis::unit(&obj);
            assert_eq!(morphism.from, obj);
            assert!(AcousticEntity::variants().contains(&morphism.to));
        }
    }

    #[test]
    fn test_analysis_synthesis_counit() {
        for obj in SignalEntity::variants() {
            let morphism = AnalysisSynthesis::counit(&obj);
            assert_eq!(morphism.to, obj);
            assert!(SignalEntity::variants().contains(&morphism.from));
        }
    }

    #[test]
    fn test_health_disease_unit() {
        for obj in AuditoryEntity::variants() {
            let morphism = HealthDisease::unit(&obj);
            assert_eq!(morphism.from, obj);
            assert!(AuditoryEntity::variants().contains(&morphism.to));
        }
    }

    #[test]
    fn test_health_disease_counit() {
        for obj in PathologyEntity::variants() {
            let morphism = HealthDisease::counit(&obj);
            assert_eq!(morphism.to, obj);
            assert!(PathologyEntity::variants().contains(&morphism.from));
        }
    }

    #[test]
    fn test_bottom_up_top_down_unit() {
        for obj in PsychoacousticEntity::variants() {
            let morphism = BottomUpTopDown::unit(&obj);
            assert_eq!(morphism.from, obj);
            assert!(PsychoacousticEntity::variants().contains(&morphism.to));
        }
    }

    #[test]
    fn test_bottom_up_top_down_counit() {
        for obj in MusicEntity::variants() {
            let morphism = BottomUpTopDown::counit(&obj);
            assert_eq!(morphism.to, obj);
            assert!(MusicEntity::variants().contains(&morphism.from));
        }
    }

    #[test]
    fn test_diagnosis_treatment_unit() {
        for obj in PathologyEntity::variants() {
            let morphism = DiagnosisTreatment::unit(&obj);
            assert_eq!(morphism.from, obj);
            assert!(PathologyEntity::variants().contains(&morphism.to));
        }
    }

    #[test]
    fn test_diagnosis_treatment_counit() {
        for obj in DeviceEntity::variants() {
            let morphism = DiagnosisTreatment::counit(&obj);
            assert_eq!(morphism.to, obj);
            assert!(DeviceEntity::variants().contains(&morphism.from));
        }
    }

    // -- Semantic validation --

    #[test]
    fn test_ohc_round_trips_through_health_disease() {
        // OHC → HairCellLoss → OuterHairCell
        let unit = HealthDisease::unit(&AuditoryEntity::OuterHairCell);
        assert_eq!(
            unit.to,
            AuditoryEntity::OuterHairCell,
            "OHC should recover through round-trip"
        );
    }

    #[test]
    fn test_snhl_round_trips_through_diagnosis_treatment() {
        // SNHL → BTE → SNHL
        let unit = DiagnosisTreatment::unit(&PathologyEntity::SensorineuralHearingLoss);
        assert_eq!(unit.to, PathologyEntity::SensorineuralHearingLoss);
    }

    #[test]
    fn test_pitch_round_trips_through_bottom_up_top_down() {
        // Pitch → PitchHeight → Pitch
        let unit = BottomUpTopDown::unit(&PsychoacousticEntity::Pitch);
        assert_eq!(unit.to, PsychoacousticEntity::Pitch);
    }

    // -- Property-based tests --

    use proptest::prelude::*;

    fn arb_acoustic_entity() -> impl Strategy<Value = AcousticEntity> {
        (0..AcousticEntity::variants().len()).prop_map(|i| AcousticEntity::variants()[i])
    }

    fn arb_signal_entity() -> impl Strategy<Value = SignalEntity> {
        (0..SignalEntity::variants().len()).prop_map(|i| SignalEntity::variants()[i])
    }

    fn arb_auditory_entity() -> impl Strategy<Value = AuditoryEntity> {
        (0..AuditoryEntity::variants().len()).prop_map(|i| AuditoryEntity::variants()[i])
    }

    fn arb_pathology_entity() -> impl Strategy<Value = PathologyEntity> {
        (0..PathologyEntity::variants().len()).prop_map(|i| PathologyEntity::variants()[i])
    }

    fn arb_psychoacoustic_entity() -> impl Strategy<Value = PsychoacousticEntity> {
        (0..PsychoacousticEntity::variants().len())
            .prop_map(|i| PsychoacousticEntity::variants()[i])
    }

    fn arb_music_entity() -> impl Strategy<Value = MusicEntity> {
        (0..MusicEntity::variants().len()).prop_map(|i| MusicEntity::variants()[i])
    }

    fn arb_device_entity() -> impl Strategy<Value = DeviceEntity> {
        (0..DeviceEntity::variants().len()).prop_map(|i| DeviceEntity::variants()[i])
    }

    proptest! {
        // -- Right adjoint functor property tests --

        #[test]
        fn prop_signal_to_acoustics_maps_valid(entity in arb_signal_entity()) {
            let mapped = SignalProcessingToAcoustics::map_object(&entity);
            prop_assert!(AcousticEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_signal_to_acoustics_preserves_identity(entity in arb_signal_entity()) {
            let id_src = SignalProcessingCategory::identity(&entity);
            let mapped_id = SignalProcessingToAcoustics::map_morphism(&id_src);
            let id_tgt = AcousticsCategory::identity(&SignalProcessingToAcoustics::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }

        #[test]
        fn prop_pathology_to_anatomy_maps_valid(entity in arb_pathology_entity()) {
            let mapped = PathologyToAnatomy::map_object(&entity);
            prop_assert!(AuditoryEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_pathology_to_anatomy_preserves_identity(entity in arb_pathology_entity()) {
            let id_src = PathologyCategory::identity(&entity);
            let mapped_id = PathologyToAnatomy::map_morphism(&id_src);
            let id_tgt = AnatomyCategory::identity(&PathologyToAnatomy::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }

        #[test]
        fn prop_music_to_psychoacoustics_maps_valid(entity in arb_music_entity()) {
            let mapped = MusicToPsychoacoustics::map_object(&entity);
            prop_assert!(PsychoacousticEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_music_to_psychoacoustics_preserves_identity(entity in arb_music_entity()) {
            let id_src = MusicPerceptionCategory::identity(&entity);
            let mapped_id = MusicToPsychoacoustics::map_morphism(&id_src);
            let id_tgt = PsychoacousticsCategory::identity(&MusicToPsychoacoustics::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }

        #[test]
        fn prop_devices_to_pathology_maps_valid(entity in arb_device_entity()) {
            let mapped = DevicesToPathology::map_object(&entity);
            prop_assert!(PathologyEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_devices_to_pathology_preserves_identity(entity in arb_device_entity()) {
            let id_src = DeviceCategory::identity(&entity);
            let mapped_id = DevicesToPathology::map_morphism(&id_src);
            let id_tgt = PathologyCategory::identity(&DevicesToPathology::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }

        #[test]
        fn prop_anatomy_to_pathology_maps_valid(entity in arb_auditory_entity()) {
            let mapped = AnatomyToPathology::map_object(&entity);
            prop_assert!(PathologyEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_anatomy_to_pathology_preserves_identity(entity in arb_auditory_entity()) {
            let id_src = AnatomyCategory::identity(&entity);
            let mapped_id = AnatomyToPathology::map_morphism(&id_src);
            let id_tgt = PathologyCategory::identity(&AnatomyToPathology::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }

        // -- Adjunction unit/counit property tests --

        #[test]
        fn prop_analysis_synthesis_unit_wellformed(entity in arb_acoustic_entity()) {
            let m = AnalysisSynthesis::unit(&entity);
            prop_assert_eq!(m.from, entity);
            prop_assert!(AcousticEntity::variants().contains(&m.to));
        }

        #[test]
        fn prop_analysis_synthesis_counit_wellformed(entity in arb_signal_entity()) {
            let m = AnalysisSynthesis::counit(&entity);
            prop_assert_eq!(m.to, entity);
            prop_assert!(SignalEntity::variants().contains(&m.from));
        }

        #[test]
        fn prop_health_disease_unit_wellformed(entity in arb_auditory_entity()) {
            let m = HealthDisease::unit(&entity);
            prop_assert_eq!(m.from, entity);
            prop_assert!(AuditoryEntity::variants().contains(&m.to));
        }

        #[test]
        fn prop_health_disease_counit_wellformed(entity in arb_pathology_entity()) {
            let m = HealthDisease::counit(&entity);
            prop_assert_eq!(m.to, entity);
            prop_assert!(PathologyEntity::variants().contains(&m.from));
        }

        #[test]
        fn prop_bottom_up_top_down_unit_wellformed(entity in arb_psychoacoustic_entity()) {
            let m = BottomUpTopDown::unit(&entity);
            prop_assert_eq!(m.from, entity);
            prop_assert!(PsychoacousticEntity::variants().contains(&m.to));
        }

        #[test]
        fn prop_bottom_up_top_down_counit_wellformed(entity in arb_music_entity()) {
            let m = BottomUpTopDown::counit(&entity);
            prop_assert_eq!(m.to, entity);
            prop_assert!(MusicEntity::variants().contains(&m.from));
        }

        #[test]
        fn prop_diagnosis_treatment_unit_wellformed(entity in arb_pathology_entity()) {
            let m = DiagnosisTreatment::unit(&entity);
            prop_assert_eq!(m.from, entity);
            prop_assert!(PathologyEntity::variants().contains(&m.to));
        }

        #[test]
        fn prop_diagnosis_treatment_counit_wellformed(entity in arb_device_entity()) {
            let m = DiagnosisTreatment::counit(&entity);
            prop_assert_eq!(m.to, entity);
            prop_assert!(DeviceEntity::variants().contains(&m.from));
        }
    }
}

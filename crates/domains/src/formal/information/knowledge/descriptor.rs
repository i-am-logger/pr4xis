use pr4xis::ontology::Vocabulary;
use pr4xis::ontology::upper::being::Being;

pub fn describe_knowledge_base() -> Vec<Vocabulary> {
    static CACHE: std::sync::OnceLock<Vec<Vocabulary>> = std::sync::OnceLock::new();
    CACHE.get_or_init(build_descriptors).clone()
}

fn build_descriptors() -> Vec<Vocabulary> {
    vec![
        // =================================================================
        // Cognitive — Cognition (define_ontology!)
        // =================================================================
        crate::cognitive::cognition::distinction::DistinctionOntology::vocabulary(),
        crate::cognitive::cognition::epistemics::EpistemicOntology::vocabulary(),
        crate::cognitive::cognition::metacognition::MetaCognitionOntology::vocabulary(),
        crate::cognitive::cognition::self_model::SelfModelOntology::vocabulary(),
        // =================================================================
        // Cognitive — Linguistics (mixed: define_ontology! + manual)
        // =================================================================
        // Lexicon: manual — POS-specific modification rules in morphisms()
        manual::<
            crate::cognitive::linguistics::lexicon::ontology::LexicalCategory,
            crate::cognitive::linguistics::lexicon::pos::PosTag,
        >("Lexicon", "pr4xis_domains::cognitive::linguistics::lexicon", "Lambek (1958); Chiarcos & Sukhareva OLiA (2015)", Being::SocialObject),
        // Tense: manual — computes 3x4 tense-aspect grid shifts algorithmically
        manual::<
            crate::cognitive::linguistics::morphology::tense::TenseCategory,
            crate::cognitive::linguistics::morphology::tense::TenseAspect,
        >("Tense & Aspect", "pr4xis_domains::cognitive::linguistics::morphology::tense", "Reichenbach (1947); Comrie (1976)", Being::AbstractObject),
        // SpellingErrors: manual — specific error chain morphisms (etiology->operation->observation)
        manual::<
            crate::cognitive::linguistics::orthography::distance::SpellingErrorCategory,
            crate::cognitive::linguistics::orthography::distance::SpellingErrorConcept,
        >("Spelling Errors", "pr4xis_domains::cognitive::linguistics::orthography::distance", "Damerau (1964); Brill & Moore (2000)", Being::Quality),
        crate::cognitive::linguistics::orthography::channel::ChannelOntology::vocabulary(),
        crate::cognitive::linguistics::pragmatics::reference::ReferenceOntology::vocabulary(),
        crate::cognitive::linguistics::pragmatics::generation::ProductionOntology::vocabulary(),
        crate::cognitive::linguistics::pragmatics::nlg::NlgOntology::vocabulary(),
        crate::cognitive::linguistics::pragmatics::response::ResponseOntology::vocabulary(),
        // =================================================================
        // Formal — Mathematics (macro-generated)
        // =================================================================
        crate::formal::math::ontology::MathOntology::vocabulary(),
        crate::formal::math::geometry::ontology::EuclideanGeometryOntology::vocabulary(),
        crate::formal::math::linear_algebra::ontology::LinearAlgebraOntology::vocabulary(),
        crate::formal::math::probability::ontology::ProbabilityOntology::vocabulary(),
        crate::formal::math::quantity::ontology::QuantityOntology::vocabulary(),
        crate::formal::math::rotation::ontology::RotationOntology::vocabulary(),
        crate::formal::math::signal_processing::ontology::SignalProcessingOntology::vocabulary(),
        crate::formal::math::statistics::ontology::StatisticsOntology::vocabulary(),
        crate::formal::math::temporal::ontology::TimeOntology::vocabulary(),
        crate::formal::math::control_theory::ontology::ControlTheoryOntology::vocabulary(),
        // =================================================================
        // Formal — Meta (macro-generated)
        // =================================================================
        crate::formal::meta::artifact_identity::ontology::ArtifactIdentityOntology::vocabulary(),
        crate::formal::meta::staging::ontology::StagingOntology::vocabulary(),
        crate::formal::meta::ontology_diagnostics::ontology::MetaOntology::vocabulary(),
        // =================================================================
        // Formal — Information (macro-generated)
        // =================================================================
        crate::formal::information::ontology::InformationOntology::vocabulary(),
        crate::formal::information::communication::ontology::CommunicationOntology::vocabulary(),
        crate::formal::information::dialogue::ontology::DialogueOntology::vocabulary(),
        crate::formal::information::events::ontology::EventOntology::vocabulary(),
        crate::formal::information::concurrency::ontology::ConcurrencyOntology::vocabulary(),
        crate::formal::information::provenance::ontology::ProvenanceOntology::vocabulary(),
        crate::formal::information::knowledge::ontology::KnowledgeOntology::vocabulary(),
        crate::formal::information::diagnostics::ontology::DiagnosticOntology::vocabulary(),
        crate::formal::information::measurement::ontology::MeasurementOntology::vocabulary(),
        crate::formal::information::schema::ontology::SchemaOntology::vocabulary(),
        crate::formal::information::storage::ontology::StorageOntology::vocabulary(),
        // =================================================================
        // Formal — Systems (macro-generated)
        // =================================================================
        crate::formal::systems::ontology::SystemsOntology::vocabulary(),
        crate::formal::systems::control::ControlOntology::vocabulary(),
        // =================================================================
        // Formal — Other (macro-generated)
        // =================================================================
        crate::formal::analytical_methods::ontology::AnalyticalMethodsOntology::vocabulary(),
        crate::formal::derivation::ontology::DerivationOntology::vocabulary(),
        crate::formal::optimization::ontology::OptimizationOntology::vocabulary(),
        crate::formal::recommendation::ontology::RecommendationOntology::vocabulary(),
        // =================================================================
        // Applied — Data Provisioning (macro-generated)
        // =================================================================
        crate::applied::data_provisioning::ontology::DataProvisioningOntology::vocabulary(),
        // =================================================================
        // Applied — HMI (manual: bright_variant_of() method drives morphisms)
        // =================================================================
        manual::<
            crate::applied::hmi::theming::ontology::ThemingCategory,
            crate::applied::hmi::theming::base16::ColorSlot,
        >("Theming", "pr4xis_domains::applied::hmi::theming", "Base16 styling spec; WCAG 2.1", Being::Quality),
        // =================================================================
        // Applied — Navigation (macro-generated)
        // =================================================================
        crate::applied::navigation::ahrs::ontology::AhrsOntology::vocabulary(),
        crate::applied::navigation::celestial::ontology::CelestialOntology::vocabulary(),
        crate::applied::navigation::gnss::ontology::GnssOntology::vocabulary(),
        crate::applied::navigation::imu::ontology::ImuOntology::vocabulary(),
        crate::applied::navigation::ins_gnss::ontology::InsGnssOntology::vocabulary(),
        crate::applied::navigation::odometry::ontology::OdometryOntology::vocabulary(),
        // =================================================================
        // Applied — Sensor Fusion (macro-generated)
        // =================================================================
        crate::applied::sensor_fusion::fusion::ontology::FusionOntology::vocabulary(),
        crate::applied::sensor_fusion::observation::ontology::ObservationOntology::vocabulary(),
        crate::applied::sensor_fusion::sensor::ontology::SensorOntology::vocabulary(),
        crate::applied::sensor_fusion::state::ontology::StateEstimationOntology::vocabulary(),
        crate::applied::sensor_fusion::time::ontology::SensorTimeOntology::vocabulary(),
        // =================================================================
        // Applied — Space (macro-generated)
        // =================================================================
        crate::applied::space::attitude::ontology::AttitudeOntology::vocabulary(),
        crate::applied::space::orbit::ontology::OrbitOntology::vocabulary(),
        // =================================================================
        // Applied — Tracking (macro-generated)
        // =================================================================
        crate::applied::tracking::single_target::ontology::SingleTargetOntology::vocabulary(),
        // =================================================================
        // Applied — Underwater (macro-generated)
        // =================================================================
        crate::applied::underwater::acoustic::ontology::AcousticOntology::vocabulary(),
        crate::applied::underwater::auv::ontology::AuvOntology::vocabulary(),
        // =================================================================
        // Applied — Industrial (macro-generated)
        // =================================================================
        crate::applied::industrial::process::ontology::ProcessOntology::vocabulary(),
        crate::applied::industrial::structural::ontology::StructuralOntology::vocabulary(),
        // =================================================================
        // Applied — Localization (macro-generated)
        // =================================================================
        crate::applied::localization::terrain::ontology::TerrainOntology::vocabulary(),
        // =================================================================
        // Applied — Perception (macro-generated)
        // =================================================================
        crate::applied::perception::occupancy::ontology::OccupancyOntology::vocabulary(),
        // =================================================================
        // Applied — Hardware (define_ontology!)
        // =================================================================
        crate::applied::hardware::traffic::ontology::TrafficOntology::vocabulary(),
        crate::applied::hardware::elevator::ontology::ElevatorOntology::vocabulary(),
        // =================================================================
        // Natural — Physics (macro-generated)
        // =================================================================
        crate::natural::physics::ontology::PhysicsOntology::vocabulary(),
        crate::natural::physics::kinematics::ontology::KinematicsOntology::vocabulary(),
        // =================================================================
        // Natural — Geodesy (macro-generated)
        // =================================================================
        crate::natural::geodesy::ontology::GeodesyOntology::vocabulary(),
        // =================================================================
        // Natural — Colors (macro-generated)
        // =================================================================
        crate::natural::colors::ontology::ColorOntology::vocabulary(),
        // =================================================================
        // Natural — Biomedical (macro-generated)
        // =================================================================
        crate::natural::biomedical::acoustics::ontology::AcousticsOntologyMeta::vocabulary(),
        crate::natural::biomedical::biochemistry::ontology::BiochemistryOntologyMeta::vocabulary(),
        crate::natural::biomedical::bioelectricity::ontology::BioelectricOntologyMeta::vocabulary(),
        crate::natural::biomedical::bioelectricity::morphospace::MorphospaceOntologyMeta::vocabulary(),
        crate::natural::biomedical::biology::ontology::BiologyOntologyMeta::vocabulary(),
        crate::natural::biomedical::biophysics::ontology::BiophysicsOntologyMeta::vocabulary(),
        crate::natural::biomedical::chemistry::ontology::ChemistryOntologyMeta::vocabulary(),
        crate::natural::biomedical::electrophysiology::ontology::ElectrophysiologyOntologyMeta::vocabulary(),
        crate::natural::biomedical::hematology::ontology::HematologyOntologyMeta::vocabulary(),
        crate::natural::biomedical::immunology::ontology::ImmunologyOntologyMeta::vocabulary(),
        crate::natural::biomedical::mechanobiology::ontology::MechanobiologyOntologyMeta::vocabulary(),
        crate::natural::biomedical::molecular::ontology::MolecularOntologyMeta::vocabulary(),
        crate::natural::biomedical::pathology::ontology::PathologyOntologyMeta::vocabulary(),
        crate::natural::biomedical::pharmacology::ontology::PharmacologyOntologyMeta::vocabulary(),
        crate::natural::biomedical::regeneration::ontology::RegenerationOntologyMeta::vocabulary(),
        // =================================================================
        // Natural — Hearing (macro-generated)
        // =================================================================
        crate::natural::hearing::acoustics::ontology::AcousticsOntology::vocabulary(),
        crate::natural::hearing::anatomy::ontology::AnatomyOntology::vocabulary(),
        crate::natural::hearing::audiology::ontology::AudiologyOntology::vocabulary(),
        crate::natural::hearing::auditory_neuroscience::ontology::NeuroscienceOntology::vocabulary(),
        crate::natural::hearing::bone_conduction::ontology::BoneConductionOntology::vocabulary(),
        crate::natural::hearing::devices::ontology::DeviceOntology::vocabulary(),
        crate::natural::hearing::environmental_acoustics::ontology::EnvironmentalAcousticsOntology::vocabulary(),
        crate::natural::hearing::music_perception::ontology::MusicPerceptionOntology::vocabulary(),
        crate::natural::hearing::pathology::ontology::PathologyOntology::vocabulary(),
        crate::natural::hearing::psychoacoustics::ontology::PsychoacousticsOntology::vocabulary(),
        crate::natural::hearing::signal_processing::ontology::SignalProcessingOntology::vocabulary(),
        crate::natural::hearing::speech::ontology::SpeechOntology::vocabulary(),
        crate::natural::hearing::transduction::ontology::TransductionOntology::vocabulary(),
        crate::natural::hearing::vestibular::ontology::VestibularOntology::vocabulary(),
        // =================================================================
        // Social — Games (define_ontology!)
        // =================================================================
        crate::social::games::chess::ontology::ChessOntology::vocabulary(),
        crate::social::games::rubik::ontology::RubikOntology::vocabulary(),
        // =================================================================
        // Social — Judicial (manual: uses valid_transitions() + algorithmic closure)
        // =================================================================
        manual::<
            crate::social::judicial::ontology::CaseLifecycleCategory,
            crate::social::judicial::PhaseTag,
        >("Judicial", "pr4xis_domains::social::judicial", "Hart (1961); Sartor (2005)", Being::Process),
        // =================================================================
        // Social — Military (macro-generated)
        // =================================================================
        crate::social::military::electronic_warfare::ontology::EwOntology::vocabulary(),
        // =================================================================
        // Social — Software (mixed: define_ontology! + manual)
        // =================================================================
        crate::social::software::protocols::http::ontology::HttpMethodOntology::vocabulary(),
        // Markup/XML/RDF/OWL: manual — complex containment/structural rules
        // with custom morphisms() per W3C spec constraints
        manual::<
            crate::social::software::markup::ontology::MarkupCategory,
            crate::social::software::markup::ontology::NodeKind,
        >("Markup", "pr4xis_domains::social::software::markup", "Coombs et al. (1987); Goldfarb (1990)", Being::SocialObject),
        manual::<
            crate::social::software::markup::xml::ontology::XmlCategory,
            crate::social::software::markup::xml::ontology::XmlNodeKind,
        >("XML", "pr4xis_domains::social::software::markup::xml", "W3C XML 1.0 (2008)", Being::SocialObject),
        manual::<
            crate::social::software::markup::xml::rdf::ontology::RdfCategory,
            crate::social::software::markup::xml::rdf::ontology::RdfNodeKind,
        >("RDF", "pr4xis_domains::social::software::markup::xml::rdf", "W3C RDF 1.1 (2014)", Being::SocialObject),
        manual::<
            crate::social::software::markup::xml::owl::ontology::OwlCategory,
            crate::social::software::markup::xml::owl::ontology::OwlConcept,
        >("OWL", "pr4xis_domains::social::software::markup::xml::owl", "W3C OWL 2 (2012); Baader et al. (2003)", Being::SocialObject),
        // =================================================================
        // New this session — Lemon, OMV, NL ontologies
        // =================================================================
        crate::cognitive::linguistics::lemon::ontology::LemonOntology::vocabulary(),
        crate::formal::meta::omv::ontology::OmvOntology::vocabulary(),
        crate::cognitive::linguistics::pragmatics::discourse::ontology::DiscourseOntology::vocabulary(),
        crate::cognitive::linguistics::pragmatics::grounding::ontology::GroundingOntology::vocabulary(),
        crate::cognitive::linguistics::pragmatics::fragment::ontology::FragmentOntology::vocabulary(),
        // =================================================================
        // Sub-ontologies (migrated to define_ontology!)
        // =================================================================
        crate::formal::information::schema::alignment::AlignmentOntology::vocabulary(),
        crate::formal::information::schema::instance::InstanceOntology::vocabulary(),
        crate::formal::information::schema::trace_schema::TraceSchemaOntology::vocabulary(),
        crate::formal::information::storage::consistency::ConsistencyOntology::vocabulary(),
        crate::formal::information::storage::durability::DurabilityOntology::vocabulary(),
        crate::formal::information::storage::volatility::VolatilityOntology::vocabulary(),
        crate::formal::information::measurement::benchmark::BenchmarkOntology::vocabulary(),
        crate::formal::information::dialogue::grounding::DialogueGroundingOntology::vocabulary(),
        // =================================================================
        // Session ontologies — consciousness, pipeline, planning, text, algebra
        // =================================================================
        crate::cognitive::cognition::consciousness::ontology::C1Ontology::vocabulary(),
        crate::cognitive::cognition::consciousness::ontology::C2Ontology::vocabulary(),
        crate::cognitive::linguistics::pipeline::ontology::PipelineOntology::vocabulary(),
        crate::cognitive::linguistics::pragmatics::planning::ontology::PlanningOntology::vocabulary(),
        crate::cognitive::linguistics::text::ontology::TextOntology::vocabulary(),
        crate::formal::meta::algebra::ontology::AlgebraOntology::vocabulary(),
        // =================================================================
        // Converted to define_ontology!
        // =================================================================
        crate::applied::localization::slam::ontology::SlamOntologyMeta::vocabulary(),
        crate::applied::perception::lidar_camera::ontology::LidarCameraOntologyMeta::vocabulary(),
        crate::applied::perception::radar_camera::ontology::RadarCameraOntologyMeta::vocabulary(),
        crate::applied::tracking::multi_target::ontology::MultiTargetOntologyMeta::vocabulary(),
        crate::social::military::situation::ontology::SituationOntologyMeta::vocabulary(),
        // =================================================================
        // Remaining manual — complex compose/morphisms logic
        // =================================================================
        // Reference Frame: uses external FrameTransform type with custom new()
        manual::<
            crate::applied::sensor_fusion::frame::ontology::FrameCategory,
            crate::applied::sensor_fusion::frame::reference::ReferenceFrame,
        >("Reference Frame", "pr4xis_domains::applied::sensor_fusion::frame", "Sola et al. (2018)", Being::AbstractObject),
        // Compliance: algorithmic escalation ladder + de-escalation/abort patterns
        manual::<
            crate::social::compliance::ontology::ComplianceCategory,
            crate::social::compliance::escalation::EscalationLevel,
        >("Compliance", "pr4xis_domains::social::compliance", "ISO 37301 (2021)", Being::SocialObject),
    ]
}

fn manual<C: pr4xis::category::Category, E: pr4xis::category::entity::Entity>(
    name: &'static str,
    module_path: &'static str,
    source: &'static str,
    being: Being,
) -> Vocabulary {
    Vocabulary::from_ontology::<C, E>(name, module_path, source, Some(being))
}

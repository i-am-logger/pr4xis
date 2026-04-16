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
        manual::<
            crate::cognitive::linguistics::lexicon::ontology::LexicalCategory,
            crate::cognitive::linguistics::lexicon::pos::PosTag,
        >("Lexicon", "pr4xis_domains::cognitive::linguistics::lexicon", "Lambek (1958); Chiarcos & Sukhareva OLiA (2015)", Being::SocialObject),
        manual::<
            crate::cognitive::linguistics::morphology::tense::TenseCategory,
            crate::cognitive::linguistics::morphology::tense::TenseAspect,
        >("Tense & Aspect", "pr4xis_domains::cognitive::linguistics::morphology::tense", "Reichenbach (1947); Comrie (1976)", Being::AbstractObject),
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
        // Applied — HMI (manual)
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
        // Applied — Hardware (mixed: define_ontology! + manual)
        // =================================================================
        crate::applied::hardware::traffic::ontology::TrafficOntology::vocabulary(),
        manual::<
            crate::applied::hardware::elevator::ontology::ElevatorCategory,
            crate::applied::hardware::elevator::ontology::Floor,
        >("Elevator", "pr4xis_domains::applied::hardware::elevator", "Mandel (1989); Barney & Dos Santos (1985)", Being::SocialObject),
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
        // Social — Games (manual)
        // =================================================================
        manual::<
            crate::social::games::chess::ontology::ChessCategory,
            crate::social::games::chess::square::Square,
        >("Chess", "pr4xis_domains::social::games::chess", "FIDE Laws of Chess; Shannon (1950)", Being::SocialObject),
        manual::<
            crate::social::games::rubik::ontology::RubikCategory,
            crate::social::games::rubik::Face,
        >("Rubik's Cube", "pr4xis_domains::social::games::rubik", "Joyner (2008); Singmaster (1981)", Being::SocialObject),
        // =================================================================
        // Social — Judicial (manual)
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
        // Social — Software (manual)
        // =================================================================
        manual::<
            crate::social::software::protocols::http::ontology::HttpMethodCategory,
            crate::social::software::protocols::http::Method,
        >("HTTP", "pr4xis_domains::social::software::protocols::http", "RFC 9110 (2022); Fielding (2000)", Being::SocialObject),
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
        // Sub-ontologies (define_category! — not yet migrated to define_ontology!)
        // =================================================================
        manual::<
            crate::formal::information::schema::alignment::AlignmentCategory,
            crate::formal::information::schema::alignment::AlignmentConcept,
        >("Schema Alignment", "pr4xis_domains::formal::information::schema::alignment", "Spivak (2012); Euzenat & Shvaiko (2013)", Being::AbstractObject),
        manual::<
            crate::formal::information::schema::instance::InstanceCategory,
            crate::formal::information::schema::instance::InstanceConcept,
        >("Schema Instance", "pr4xis_domains::formal::information::schema::instance", "Spivak (2012)", Being::AbstractObject),
        manual::<
            crate::formal::information::schema::trace_schema::TraceSchemaCategory,
            crate::formal::information::schema::trace_schema::TraceSchemaElement,
        >("Trace Schema", "pr4xis_domains::formal::information::schema::trace_schema", "W3C PROV-O (2013)", Being::AbstractObject),
        manual::<
            crate::formal::information::storage::consistency::ConsistencyCategory,
            crate::formal::information::storage::consistency::ConsistencyModel,
        >("Consistency", "pr4xis_domains::formal::information::storage::consistency", "Viotti & Vukolic (2016); Herlihy & Wing (1990)", Being::AbstractObject),
        manual::<
            crate::formal::information::storage::durability::DurabilityCategory,
            crate::formal::information::storage::durability::DurabilityLevel,
        >("Durability", "pr4xis_domains::formal::information::storage::durability", "Haerder & Reuter (1983); Pelley et al. (2014)", Being::AbstractObject),
        manual::<
            crate::formal::information::storage::volatility::VolatilityCategory,
            crate::formal::information::storage::volatility::StorageMedia,
        >("Volatility", "pr4xis_domains::formal::information::storage::volatility", "Pelley et al. (2014)", Being::AbstractObject),
        manual::<
            crate::formal::information::measurement::benchmark::BenchmarkCategory,
            crate::formal::information::measurement::benchmark::BenchmarkConcept,
        >("Benchmark", "pr4xis_domains::formal::information::measurement::benchmark", "JCGM 200:2012 (VIM)", Being::AbstractObject),
        manual::<
            crate::formal::information::dialogue::grounding::GroundingCategory,
            crate::formal::information::dialogue::grounding::GroundingState,
        >("Dialogue Grounding", "pr4xis_domains::formal::information::dialogue::grounding", "Clark (1996); Traum (1994)", Being::Process),
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
        // Pre-existing unregistered — manual Category impls
        // =================================================================
        manual::<
            crate::applied::localization::slam::ontology::SlamCategory,
            crate::applied::localization::slam::ontology::SlamComponent,
        >("SLAM", "pr4xis_domains::applied::localization::slam", "Durrant-Whyte & Bailey (2006)", Being::Process),
        manual::<
            crate::applied::perception::lidar_camera::ontology::LidarCameraCategory,
            crate::applied::perception::lidar_camera::ontology::FusionStage,
        >("Lidar-Camera Fusion", "pr4xis_domains::applied::perception::lidar_camera", "Qi et al. (2018)", Being::Process),
        manual::<
            crate::applied::perception::radar_camera::ontology::RadarCameraCategory,
            crate::applied::perception::radar_camera::ontology::RadarCameraStage,
        >("Radar-Camera Fusion", "pr4xis_domains::applied::perception::radar_camera", "Nobis et al. (2019)", Being::Process),
        manual::<
            crate::applied::sensor_fusion::frame::ontology::FrameCategory,
            crate::applied::sensor_fusion::frame::reference::ReferenceFrame,
        >("Reference Frame", "pr4xis_domains::applied::sensor_fusion::frame", "Sola et al. (2018)", Being::AbstractObject),
        manual::<
            crate::applied::tracking::multi_target::ontology::TrackLifecycleCategory,
            crate::applied::tracking::multi_target::ontology::TrackState,
        >("Multi-Target Tracking", "pr4xis_domains::applied::tracking::multi_target", "Bar-Shalom et al. (2001)", Being::Process),
        manual::<
            crate::social::compliance::ontology::ComplianceCategory,
            crate::social::compliance::escalation::EscalationLevel,
        >("Compliance", "pr4xis_domains::social::compliance", "ISO 37301 (2021)", Being::SocialObject),
        manual::<
            crate::social::military::situation::ontology::SituationCategory,
            crate::social::military::situation::ontology::SituationElement,
        >("Situation Awareness", "pr4xis_domains::social::military::situation", "Endsley (1995); JDL (1999)", Being::Process),
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

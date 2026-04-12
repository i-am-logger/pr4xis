// DOLCE classifications for every domain ontology in praxis.
//
// Each domain declares what type of Being it models, per DOLCE upper ontology.
// Reference: Masolo et al., WonderWeb Deliverable D18 (2003)
//
// The Classified trait lives in pr4xis::ontology::upper::classify.
// These impls apply it to domain categories.

use pr4xis::ontology::upper::being::Being;
use pr4xis::ontology::upper::classify::Classified;

// =============================================================================
// Science domains
// =============================================================================

/// Physics laws are timeless mathematical truths.
impl Classified for crate::natural::physics::ontology::PhysicsCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "physical laws are timeless mathematical relations"
    }
}

/// Mathematics is pure abstraction.
impl Classified for crate::formal::math::ontology::NumberHierarchy {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "numbers and mathematical structures are timeless abstracts"
    }
}

/// Colors (as perceived) are qualities inhering in physical objects.
impl Classified for crate::natural::colors::ontology::ColorCategory {
    fn being() -> Being {
        Being::Quality
    }
    fn classification_reason() -> &'static str {
        "color is a measurable perceptual quality"
    }
}

/// Systems thinking is an abstract framework for understanding wholes.
impl Classified for crate::formal::systems::ontology::SystemsCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "systems theory is an abstract framework (cybernetics)"
    }
}

/// Information theory is abstract (Shannon 1948).
impl Classified for crate::formal::information::ontology::InfoCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "information is an abstract quantity (bits, entropy)"
    }
}

/// Event-driven architecture is an abstract pattern.
impl Classified for crate::formal::information::events::ontology::EventCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "event-driven architecture is an abstract pattern"
    }
}

/// Concurrency is an abstract model of parallel computation.
impl Classified for crate::formal::information::concurrency::ontology::ConcurrencyCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "concurrency is an abstract model (CSP, CCS, Actor)"
    }
}

/// Dialogue is a process (extended over time).
impl Classified for crate::formal::information::dialogue::ontology::DialogueCategory {
    fn being() -> Being {
        Being::Process
    }
    fn classification_reason() -> &'static str {
        "dialogue is an extended temporal process between participants"
    }
}

/// Distinction (Spencer-Brown) is the most abstract concept.
impl Classified for crate::cognitive::cognition::distinction::DistinctionCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "distinction is the most fundamental abstract operation (Laws of Form)"
    }
}

/// Epistemics is a mental/cognitive framework.
impl Classified for crate::cognitive::cognition::epistemics::EpistemicCategory {
    fn being() -> Being {
        Being::MentalObject
    }
    fn classification_reason() -> &'static str {
        "epistemic states are cognitive/mental (what we know about knowing)"
    }
}

/// Metacognition is thinking about thinking — a mental process.
impl Classified for crate::cognitive::cognition::metacognition::MetaCognitionCategory {
    fn being() -> Being {
        Being::MentalObject
    }
    fn classification_reason() -> &'static str {
        "metacognition is a mental process (second-order cybernetics)"
    }
}

/// Provenance tracks origin of knowledge — an abstract framework.
impl Classified for crate::formal::information::provenance::ontology::ProvenanceCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "provenance is an abstract framework for tracking knowledge origins (W3C PROV-O)"
    }
}

/// English lexical categories are a social convention.
impl Classified for crate::cognitive::linguistics::lexicon::ontology::LexicalCategory {
    fn being() -> Being {
        Being::SocialObject
    }
    fn classification_reason() -> &'static str {
        "language categories are evolved social conventions"
    }
}

// =============================================================================
// Technology domains
// =============================================================================

/// Chess rules are an agreed-upon social object.
impl Classified for crate::social::games::chess::ontology::ChessCategory {
    fn being() -> Being {
        Being::SocialObject
    }
    fn classification_reason() -> &'static str {
        "chess rules are agreed-upon social conventions"
    }
}

/// Rubik's cube rules are an engineered social object.
impl Classified for crate::social::games::rubik::ontology::RubikCategory {
    fn being() -> Being {
        Being::SocialObject
    }
    fn classification_reason() -> &'static str {
        "puzzle rules are engineered social constructs"
    }
}

/// HTTP specification is an agreed-upon standard (W3C/IETF).
impl Classified for crate::social::software::protocols::http::ontology::HttpMethodCategory {
    fn being() -> Being {
        Being::SocialObject
    }
    fn classification_reason() -> &'static str {
        "HTTP is a W3C/IETF standard — an agreed-upon protocol"
    }
}

/// XML specification is a W3C standard.
impl Classified for crate::social::software::markup::xml::ontology::XmlCategory {
    fn being() -> Being {
        Being::SocialObject
    }
    fn classification_reason() -> &'static str {
        "XML is a W3C standard — an agreed-upon markup language"
    }
}

/// Markup in general is a social convention for structuring documents.
impl Classified for crate::social::software::markup::ontology::MarkupCategory {
    fn being() -> Being {
        Being::SocialObject
    }
    fn classification_reason() -> &'static str {
        "markup languages are social conventions for document structure"
    }
}

/// RDF is a W3C standard data model.
impl Classified for crate::social::software::markup::xml::rdf::ontology::RdfCategory {
    fn being() -> Being {
        Being::SocialObject
    }
    fn classification_reason() -> &'static str {
        "RDF is a W3C standard — an agreed-upon data model"
    }
}

/// OWL is a W3C standard ontology language.
impl Classified for crate::social::software::markup::xml::owl::ontology::OwlCategory {
    fn being() -> Being {
        Being::SocialObject
    }
    fn classification_reason() -> &'static str {
        "OWL is a W3C standard — an agreed-upon ontology language"
    }
}

/// Traffic rules are an engineered social system.
impl Classified for crate::applied::hardware::traffic::ontology::TrafficCategory {
    fn being() -> Being {
        Being::SocialObject
    }
    fn classification_reason() -> &'static str {
        "traffic rules are engineered social conventions"
    }
}

/// Elevator rules are an engineered system.
impl Classified for crate::applied::hardware::elevator::ontology::ElevatorCategory {
    fn being() -> Being {
        Being::SocialObject
    }
    fn classification_reason() -> &'static str {
        "elevator operation rules are engineered specifications"
    }
}

// =============================================================================
// Governance domains
// =============================================================================

/// Judicial proceedings are social processes (extended over time).
impl Classified for crate::social::judicial::ontology::CaseLifecycleCategory {
    fn being() -> Being {
        Being::Process
    }
    fn classification_reason() -> &'static str {
        "a court case is a social process unfolding over time"
    }
}

// =============================================================================
// Newly classified domains
// =============================================================================

impl Classified for crate::formal::information::communication::ontology::CommunicationCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "communication is an abstract framework (Shannon 1948; Jakobson 1960)"
    }
}

impl Classified for crate::formal::systems::control::ControlCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "control systems are abstract models of regulation (Wiener 1948; Conant-Ashby 1970)"
    }
}

impl Classified for crate::cognitive::linguistics::morphology::tense::TenseCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "tense/aspect are abstract temporal relations (Reichenbach 1947)"
    }
}

impl Classified for crate::cognitive::linguistics::orthography::distance::SpellingErrorCategory {
    fn being() -> Being {
        Being::Quality
    }
    fn classification_reason() -> &'static str {
        "spelling errors are measurable deviations in written language quality"
    }
}

impl Classified for crate::cognitive::linguistics::orthography::channel::ChannelCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "the noisy channel is an abstract model (Shannon 1948; Kernighan 1990)"
    }
}

impl Classified for crate::cognitive::linguistics::pragmatics::reference::ReferenceCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "discourse reference is an abstract framework (Kamp DRT 1981; Centering 1995)"
    }
}

impl Classified for crate::formal::information::knowledge::ontology::KnowledgeBaseCategory {
    fn being() -> Being {
        Being::AbstractObject
    }
    fn classification_reason() -> &'static str {
        "knowledge base is an abstract self-describing framework (VoID 2011; Herre & Loebe 2005)"
    }
}

impl Classified for crate::cognitive::cognition::self_model::SelfModelCategory {
    fn being() -> Being {
        Being::MentalObject
    }
    fn classification_reason() -> &'static str {
        "self-model is the system's eigenform (von Foerster 1981; IEEE AuR 2021; MAPE-K 2003)"
    }
}

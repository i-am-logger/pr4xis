// Manual ontology registrations — for ontologies with hand-written
// Category/Entity impls that don't go through define_ontology! / ontology!
// macros and so don't auto-register.
//
// Each entry emits a `#[linkme::distributed_slice]` attribute that adds
// the ontology's Vocabulary to the global registry. This file exists only
// because these ontologies cannot self-register (they predate the macro
// system). As they're migrated to the macro, entries here disappear.

use pr4xis::register_manual;

register_manual!(
    ident: LEXICON,
    category: crate::cognitive::linguistics::lexicon::ontology::LexicalCategory,
    entity: crate::cognitive::linguistics::lexicon::pos::PosTag,
    name: "Lexicon",
    module: "pr4xis_domains::cognitive::linguistics::lexicon",
    source: "Lambek (1958); Chiarcos & Sukhareva OLiA (2015)",
    being: SocialObject,
);

register_manual!(
    ident: TENSE,
    category: crate::cognitive::linguistics::morphology::tense::TenseCategory,
    entity: crate::cognitive::linguistics::morphology::tense::TenseAspect,
    name: "Tense & Aspect",
    module: "pr4xis_domains::cognitive::linguistics::morphology::tense",
    source: "Reichenbach (1947); Comrie (1976)",
    being: AbstractObject,
);

register_manual!(
    ident: SPELLING_ERRORS,
    category: crate::cognitive::linguistics::orthography::distance::SpellingErrorCategory,
    entity: crate::cognitive::linguistics::orthography::distance::SpellingErrorConcept,
    name: "Spelling Errors",
    module: "pr4xis_domains::cognitive::linguistics::orthography::distance",
    source: "Damerau (1964); Brill & Moore (2000)",
    being: Quality,
);

register_manual!(
    ident: THEMING,
    category: crate::applied::hmi::theming::ontology::ThemingCategory,
    entity: crate::applied::hmi::theming::base16::ColorSlot,
    name: "Theming",
    module: "pr4xis_domains::applied::hmi::theming",
    source: "Base16 styling spec; WCAG 2.1",
    being: Quality,
);

register_manual!(
    ident: JUDICIAL,
    category: crate::social::judicial::ontology::CaseLifecycleCategory,
    entity: crate::social::judicial::PhaseTag,
    name: "Judicial",
    module: "pr4xis_domains::social::judicial",
    source: "Hart (1961); Sartor (2005)",
    being: Process,
);

register_manual!(
    ident: MARKUP,
    category: crate::social::software::markup::ontology::MarkupCategory,
    entity: crate::social::software::markup::ontology::NodeKind,
    name: "Markup",
    module: "pr4xis_domains::social::software::markup",
    source: "Coombs et al. (1987); Goldfarb (1990)",
    being: SocialObject,
);

register_manual!(
    ident: XML,
    category: crate::social::software::markup::xml::ontology::XmlCategory,
    entity: crate::social::software::markup::xml::ontology::XmlNodeKind,
    name: "XML",
    module: "pr4xis_domains::social::software::markup::xml",
    source: "W3C XML 1.0 (2008)",
    being: SocialObject,
);

register_manual!(
    ident: RDF,
    category: crate::social::software::markup::xml::rdf::ontology::RdfCategory,
    entity: crate::social::software::markup::xml::rdf::ontology::RdfNodeKind,
    name: "RDF",
    module: "pr4xis_domains::social::software::markup::xml::rdf",
    source: "W3C RDF 1.1 (2014)",
    being: SocialObject,
);

register_manual!(
    ident: OWL,
    category: crate::social::software::markup::xml::owl::ontology::OwlCategory,
    entity: crate::social::software::markup::xml::owl::ontology::OwlConcept,
    name: "OWL",
    module: "pr4xis_domains::social::software::markup::xml::owl",
    source: "W3C OWL 2 (2012); Baader et al. (2003)",
    being: SocialObject,
);

register_manual!(
    ident: REFERENCE_FRAME,
    category: crate::applied::sensor_fusion::frame::ontology::FrameCategory,
    entity: crate::applied::sensor_fusion::frame::reference::ReferenceFrame,
    name: "Reference Frame",
    module: "pr4xis_domains::applied::sensor_fusion::frame",
    source: "Sola et al. (2018)",
    being: AbstractObject,
);

register_manual!(
    ident: COMPLIANCE,
    category: crate::social::compliance::ontology::ComplianceCategory,
    entity: crate::social::compliance::escalation::EscalationLevel,
    name: "Compliance",
    module: "pr4xis_domains::social::compliance",
    source: "ISO 37301 (2021)",
    being: SocialObject,
);

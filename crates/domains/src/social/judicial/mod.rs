pub mod argument;
pub mod authority;
pub mod authority_strength;
pub mod citation;
pub mod decision;
pub mod element;
pub mod engine;
pub mod entity;
pub mod evidence_requirement;
pub mod fact;
pub mod finding;
pub mod lifecycle;
pub mod modality;
pub mod ontology;
pub mod proof_standard;
pub mod rule;
pub mod source;
pub mod source_text;
pub mod statute_structure;

// `legal_actor`, `temporal_constraint`, and `valence` are NOT exported. Each
// declared a single ontology that synthesized concepts across multiple
// primary sources (LegalActor's four-family Party/Adjudicator/Witness/
// Counsel hierarchy; TemporalConstraint's six-granularity unification of
// ISO 8601 + TimeML + FRCP Rule 6; Valence's Supportive/Defensive/
// Procedural trichotomy with no single primary attestation). Synthesis
// across sources is approximation — the praxis way is one ontology per
// primary source, then unions composed via the SourceTaxonomy Adjoins
// graph. The directories remain on disk for ease of revival if a single
// primary source is later identified for the synthesized hierarchy; until
// then they are intentionally excluded from the module tree.
//
// Replacement: per-source faithful ontologies live in the module list above
// (e.g. `frcp_rule_17`, `iso8601_calendar`, …). The container-type fields
// that previously referenced these synthesized ontologies use
// `formal::meta::identifier_format::Identifier` CURIE references, which
// resolve into the union of loaded per-source concepts via the
// `SourceTaxonomy` `Adjoins` graph.

pub use engine::{LegalAction, LegalEngine, new_case};
pub use entity::Concept;
pub use lifecycle::{Case, CaseAction, CasePhase, PhaseTag};

#[cfg(test)]
pub(crate) use argument::*;
#[cfg(test)]
pub(crate) use authority::*;
#[cfg(test)]
pub(crate) use decision::*;
#[cfg(test)]
pub(crate) use element::*;
#[cfg(test)]
pub(crate) use engine::*;
#[cfg(test)]
pub(crate) use entity::*;
#[cfg(test)]
pub(crate) use fact::*;
#[cfg(test)]
pub(crate) use finding::*;
#[cfg(test)]
pub(crate) use lifecycle::*;
#[cfg(test)]
pub(crate) use ontology::*;
#[cfg(test)]
pub(crate) use rule::*;
#[cfg(test)]
pub(crate) use source::*;

#[cfg(test)]
mod tests;

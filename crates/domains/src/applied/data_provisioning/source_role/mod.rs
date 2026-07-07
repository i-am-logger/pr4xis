//! Source role — a `prov:Role` classification of registered data sources.
//!
//! Two pieces:
//! - [`ontology`] — the `SourceRole` ontology: the three roles
//!   (`ChatKnowledge` / `DecoderInput` / `NotYetLoadable`), the
//!   `IsChatLoadable` quality, and the partition axioms.
//! - [`functor`] — the `SourceTaxonomy → SourceRole` functor
//!   ([`functor::SourceKindToRole`]) that assigns every source kind its role,
//!   plus the [`functor::source_role`] / [`functor::is_chat_loadable`]
//!   ontology queries the source catalog filters on.
//!
//! See [`ontology`] for the PROV-O literature grounding.

pub mod functor;
pub mod ontology;

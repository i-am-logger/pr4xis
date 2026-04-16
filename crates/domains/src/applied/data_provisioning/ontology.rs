//! The data_provisioning ontology — 7 entities modeling managed external
//! data sources, their cache, and their lifecycle states.
//!
//! The ontology composes against `formal/meta/artifact_identity/` for
//! identity claims, `formal/information/storage/` for cache semantics,
//! `formal/information/provenance/` for fetch events, and `formal/meta/
//! staging/` for the freeze functor framing.

use crate::formal::meta::artifact_identity::ontology::{CompositeIdentity, IdentityConcept};
use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

/// The domain concepts of external data provisioning.
///
/// A `DataSource` is the abstract concept of "an external artifact pr4xis
/// needs". `DataCache` is the local store where materialized sources live.
/// `ProvisioningEvent` is a timestamped fetch or verification (a
/// `prov:Activity`). The three lifecycle states — `VerifiedDataset`,
/// `StaleDataset`, `MissingDataset` — partition every `DataSource` based
/// on the current state of its local materialization. `DecoderFunctor` is
/// the typed transformation from raw bytes to a domain ontology for a
/// specific `ContentType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ProvisioningConcept {
    /// The abstract concept of a managed external data artifact.
    DataSource,
    /// The local directory holding materialized sources.
    DataCache,
    /// A fetch or verification event (prov:Activity).
    ProvisioningEvent,
    /// A `DataSource` whose local copy exists and verifies against every
    /// declared identity claim.
    VerifiedDataset,
    /// A `DataSource` whose local copy exists but fails verification (hash
    /// mismatch, version mismatch, broken archive, etc.).
    StaleDataset,
    /// A `DataSource` with no local copy on disk.
    MissingDataset,
    /// A typed transformation from raw bytes to a content-type-specific
    /// domain ontology instance. One decoder per `ContentType` variant.
    DecoderFunctor,
}

// ---------------------------------------------------------------------------
// Content type — polymorphism over what's inside the bytes
// ---------------------------------------------------------------------------

/// What kind of content a `DataSource` holds. The fetch pipeline is uniform
/// across content types; the decoder chain is specific per variant.
///
/// Initial PR only wires `XmlLmf` (for WordNet); the other variants are
/// declared so the type system forces future PRs to add their decoders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    /// WordNet LMF XML. Decoder: `xml_reader::read_xml → lmf::reader::read_wordnet`.
    XmlLmf,
    /// Academic PDF. Decoder: not yet implemented.
    Pdf,
    /// Plain text, UTF-8. Decoder: direct.
    Plaintext,
    /// JSON document. Decoder: serde_json parse.
    Json,
    /// Video file (mp4, webm). Decoder: not yet implemented.
    Video,
    /// Audio file (wav, flac, ogg). Decoder: not yet implemented.
    Audio,
    /// Raw bytes with no further decoding.
    Binary,
}

// ---------------------------------------------------------------------------
// RegistryEntry — the concrete managed datasets
// ---------------------------------------------------------------------------

/// One row in the data-provisioning registry. The registry is the ontology's
/// instance layer; each entry is a typed value declaring a `DataSource`'s
/// metadata, identity claims, and content type.
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    /// The name callers use to refer to this source (e.g. "wordnet").
    pub name: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// The remote URL where the content can be fetched.
    pub remote_location: &'static str,
    /// The local path (relative to the workspace root) where the content
    /// will be materialized after fetch.
    pub local_path: &'static str,
    /// What kind of content this is. Determines which decoder chain applies.
    pub content_type: ContentType,
    /// The identity claims the source declares. All must verify
    /// (`CompositeRequiresAll`). See `formal/meta/artifact_identity/`.
    pub identity: CompositeIdentity,
    /// Whether the remote content is gzipped.
    pub gzipped: bool,
}

// ---------------------------------------------------------------------------
// Ontology definition
// ---------------------------------------------------------------------------

define_ontology! {
    /// Dense category over provisioning concepts, with a taxonomy over
    /// dataset states and opposition between the state variants.
    pub DataProvisioningOntology for DataProvisioningCategory {
        entity: ProvisioningConcept,
        relation: ProvisioningRelation,
        being: Process,

        taxonomy: ProvisioningTaxonomy [
            // Dataset-state taxonomy: every lifecycle state is-a DataSource
            // (more precisely, "is a state of" — modeled as an is-a edge
            // since there's no "state-of" relation in the base taxonomy).
            (VerifiedDataset, DataSource),
            (StaleDataset, DataSource),
            (MissingDataset, DataSource),
        ],

        opposition: ProvisioningOpposition [
            // The lifecycle states oppose each other: a dataset is in
            // exactly one of these states at any given time.
            (VerifiedDataset, StaleDataset),
            (VerifiedDataset, MissingDataset),
            (StaleDataset, MissingDataset),
        ],
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Whether a dataset state means "the artifact is locally available and
/// usable right now". Only `VerifiedDataset` returns true.
#[derive(Debug, Clone)]
pub struct IsUsableLocally;

impl Quality for IsUsableLocally {
    type Individual = ProvisioningConcept;
    type Value = bool;

    fn get(&self, concept: &ProvisioningConcept) -> Option<bool> {
        use ProvisioningConcept::*;
        match concept {
            VerifiedDataset => Some(true),
            StaleDataset | MissingDataset => Some(false),
            _ => None,
        }
    }
}

/// Whether a dataset state is a terminal "needs-fetching" input to the
/// `pr4xis update` CLI. Both `StaleDataset` and `MissingDataset` do.
#[derive(Debug, Clone)]
pub struct TriggersUpdate;

impl Quality for TriggersUpdate {
    type Individual = ProvisioningConcept;
    type Value = bool;

    fn get(&self, concept: &ProvisioningConcept) -> Option<bool> {
        use ProvisioningConcept::*;
        match concept {
            VerifiedDataset => Some(false),
            StaleDataset | MissingDataset => Some(true),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Every registered `DataSource` has at least one identity claim. Content
/// addressing is non-negotiable; a source without a declared identity cannot
/// be trusted.
///
/// The const `DATA_SOURCES` entries carry an empty `CompositeIdentity`
/// placeholder because `Vec<IdentityClaim>` is not a const type. The real
/// identity lives in `registry::resolve_identity(name)`. This axiom queries
/// the resolver, not the placeholder.
pub struct EveryDataSourceHasIdentity;

impl Axiom for EveryDataSourceHasIdentity {
    fn description(&self) -> &str {
        "every RegistryEntry resolves to a non-empty CompositeIdentity"
    }

    fn holds(&self) -> bool {
        crate::applied::data_provisioning::registry::DATA_SOURCES
            .iter()
            .all(|entry| {
                crate::applied::data_provisioning::registry::resolve_identity(entry.name)
                    .is_some_and(|id| !id.0.is_empty())
            })
    }
}

/// No two `RegistryEntry` instances share a name. The name is the primary
/// key the CLI uses to look up a source.
pub struct RegistryUniquenessByName;

impl Axiom for RegistryUniquenessByName {
    fn description(&self) -> &str {
        "every RegistryEntry has a unique name"
    }

    fn holds(&self) -> bool {
        let mut names = HashSet::new();
        for entry in crate::applied::data_provisioning::registry::DATA_SOURCES {
            if !names.insert(entry.name) {
                return false;
            }
        }
        true
    }
}

/// Every `ContentType` variant in use by some `RegistryEntry` has a defined
/// `DecoderFunctor`. If a new content type is added to a registry entry
/// without a corresponding decoder, this axiom fails at test time.
pub struct DecoderTotalityPerContentType;

impl Axiom for DecoderTotalityPerContentType {
    fn description(&self) -> &str {
        "every ContentType in use has a defined decoder"
    }

    fn holds(&self) -> bool {
        for entry in crate::applied::data_provisioning::registry::DATA_SOURCES {
            if !crate::applied::data_provisioning::decoders::has_decoder_for(entry.content_type) {
                return false;
            }
        }
        true
    }
}

/// Every resolved identity claim uses a LEAF `IdentityConcept` — not a
/// family or the root. A claim with `concept: IdentityConcept::ContentHash`
/// (a family) would be ill-formed because families are abstract.
pub struct IdentityClaimsUseLeaves;

impl Axiom for IdentityClaimsUseLeaves {
    fn description(&self) -> &str {
        "every IdentityClaim uses a leaf IdentityConcept, not a family or root"
    }

    fn holds(&self) -> bool {
        use crate::formal::meta::artifact_identity::ontology::is_leaf;
        for entry in crate::applied::data_provisioning::registry::DATA_SOURCES {
            if let Some(identity) =
                crate::applied::data_provisioning::registry::resolve_identity(entry.name)
            {
                for claim in &identity.0 {
                    if !is_leaf(&claim.concept) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Ontology trait impl
// ---------------------------------------------------------------------------

impl Ontology for DataProvisioningOntology {
    type Cat = DataProvisioningCategory;
    type Qual = IsUsableLocally;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(EveryDataSourceHasIdentity),
            Box::new(RegistryUniquenessByName),
            Box::new(DecoderTotalityPerContentType),
            Box::new(IdentityClaimsUseLeaves),
        ]
    }
}

// Also remove the registry's const placeholder field now that axioms
// read from `resolve_identity` — the `identity:` field in RegistryEntry
// is kept for future const-compatible identity types but is currently
// unused. Silence the unused-field warning at the struct level via the
// #[allow] on the struct itself in registry.rs if clippy complains.

// Silence unused-import warning; IdentityConcept is re-exported for callers.
#[allow(dead_code)]
fn _identity_concept_witness(_: IdentityConcept) {}

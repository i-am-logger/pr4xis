//! Data provisioning — managed external data sources, cache, and lifecycle states.
//!
//! Composes `formal/meta/artifact_identity/` (identity claims),
//! `formal/information/storage/` (cache semantics), `formal/information/provenance/`
//! (fetch events), and `formal/meta/staging/` (the freeze functor framing).
//!
//! # Literature
//!
//! - **Wilkinson et al. (2016)** "The FAIR Guiding Principles for scientific
//!   data management and stewardship", *Scientific Data* 3 — F1 persistent
//!   identifier, A1 accessible, R1 reusable. The data-provisioning lifecycle
//!   here is the FAIR machinery.
//! - **Dolstra (2006)** *The Purely Functional Software Deployment Model*
//!   (PhD thesis, Utrecht University) — fixed-output derivations and content
//!   addressing as the basis for verifiable data provisioning.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use hashbrown::HashSet;

use crate::formal::meta::artifact_identity::ontology::{CompositeIdentity, IdentityConcept};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "DataProvisioning",
    source: "Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3; Dolstra (2006) The Purely Functional Software Deployment Model",

    concepts: [
        // === Core concepts (Wilkinson 2016 / Dolstra 2006) ===
        DataSource,
        DataCache,
        ProvisioningEvent,
        DecoderFunctor,

        // === Dataset lifecycle states (Dolstra 2006 fixed-output verdicts) ===
        VerifiedDataset,
        StaleDataset,
        MissingDataset,
    ],

    labels: {
        DataSource: ("en", "Data source",
            "Wilkinson (2016) F1: a managed external data artifact identified by a persistent identifier."),
        DataCache: ("en", "Data cache",
            "Local store where materialized DataSources live."),
        ProvisioningEvent: ("en", "Provisioning event",
            "A timestamped fetch or verification event — a `prov:Activity` per W3C PROV-O."),
        DecoderFunctor: ("en", "Decoder functor",
            "A typed transformation from raw bytes to a content-type-specific domain ontology instance. One decoder per ContentType variant."),
        VerifiedDataset: ("en", "Verified dataset",
            "Dolstra (2006): a DataSource whose local copy verifies against every declared identity claim."),
        StaleDataset: ("en", "Stale dataset",
            "A DataSource whose local copy exists but fails verification (hash / version / archive mismatch)."),
        MissingDataset: ("en", "Missing dataset",
            "A DataSource with no local copy on disk."),
    },

    is_a: [
        // The lifecycle states partition DataSource — each is-a DataSource.
        (VerifiedDataset, DataSource),
        (StaleDataset, DataSource),
        (MissingDataset, DataSource),
    ],

    opposes: [
        // The three lifecycle states are pairwise mutually exclusive.
        (VerifiedDataset, StaleDataset),
        (StaleDataset, VerifiedDataset),
        (VerifiedDataset, MissingDataset),
        (MissingDataset, VerifiedDataset),
        (StaleDataset, MissingDataset),
        (MissingDataset, StaleDataset),
    ],
}

// ---------------------------------------------------------------------------
// ContentType — polymorphism over what's inside the bytes
// ---------------------------------------------------------------------------

/// What kind of content a `DataSource` holds. The fetch pipeline is uniform
/// across content types; the decoder chain is specific per variant.
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
    pub name: &'static str,
    pub description: &'static str,
    pub remote_location: &'static str,
    pub local_path: &'static str,
    pub content_type: ContentType,
    pub identity: CompositeIdentity,
    pub gzipped: bool,
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: whether a dataset state means "the artifact is locally available
/// and usable right now". Only `VerifiedDataset` returns true.
#[derive(Debug, Clone)]
pub struct IsUsableLocally;

impl Quality for IsUsableLocally {
    type Individual = DataProvisioningConcept;
    type Value = bool;

    fn get(&self, concept: &DataProvisioningConcept) -> Option<bool> {
        use DataProvisioningConcept as C;
        match concept {
            C::VerifiedDataset => Some(true),
            C::StaleDataset | C::MissingDataset => Some(false),
            _ => None,
        }
    }
}

/// Quality: whether a dataset state is a terminal "needs-fetching" input to
/// the `pr4xis update` CLI. Both `StaleDataset` and `MissingDataset` trigger.
#[derive(Debug, Clone)]
pub struct TriggersUpdate;

impl Quality for TriggersUpdate {
    type Individual = DataProvisioningConcept;
    type Value = bool;

    fn get(&self, concept: &DataProvisioningConcept) -> Option<bool> {
        use DataProvisioningConcept as C;
        match concept {
            C::VerifiedDataset => Some(false),
            C::StaleDataset | C::MissingDataset => Some(true),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for DataProvisioningOntology {
    type Cat = DataProvisioningCategory;
    type Qual = IsUsableLocally;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(EveryDataSourceHasIdentity));
        axioms.push(Box::new(RegistryUniquenessByName));
        axioms.push(Box::new(DecoderTotalityPerContentType));
        axioms.push(Box::new(IdentityClaimsUseLeaves));
        axioms
    }
}

/// Axiom: every registered `DataSource` resolves to a non-empty
/// `CompositeIdentity` (FAIR F1 — persistent identifier).
///
/// Wilkinson (2016) F1: "(Meta)data are assigned a globally unique and
/// persistent identifier." A registry entry without a verifiable identity
/// cannot satisfy F1 and is therefore not a well-formed FAIR data source.
pub struct EveryDataSourceHasIdentity;

impl Axiom for EveryDataSourceHasIdentity {
    fn verify(&self) -> Verdict {
        let ok = crate::applied::data_provisioning::registry::DATA_SOURCES
            .iter()
            .all(|entry| {
                crate::applied::data_provisioning::registry::resolve_identity(entry.name)
                    .is_some_and(|id| !id.0.is_empty())
            });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EveryDataSourceHasIdentity",
        "every RegistryEntry resolves to a non-empty CompositeIdentity",
        "Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3 — F1"
    );
}

pr4xis::register_axiom!(
    EveryDataSourceHasIdentity,
    "Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3 — F1"
);

/// Axiom: no two `RegistryEntry` instances share a name. The name is the
/// primary key the CLI uses to look up a source.
///
/// Dolstra (2006) §5.1 — every derivation is uniquely identified by its
/// store path; for the user-facing registry layer, the human-readable name
/// must likewise be unique.
pub struct RegistryUniquenessByName;

impl Axiom for RegistryUniquenessByName {
    fn verify(&self) -> Verdict {
        let mut names = HashSet::new();
        for entry in crate::applied::data_provisioning::registry::DATA_SOURCES {
            if !names.insert(entry.name) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "RegistryUniquenessByName",
        "every RegistryEntry has a unique name",
        "Dolstra (2006) The Purely Functional Software Deployment Model §5.1"
    );
}

pr4xis::register_axiom!(
    RegistryUniquenessByName,
    "Dolstra (2006) The Purely Functional Software Deployment Model §5.1"
);

/// Axiom: every `ContentType` variant in use by some `RegistryEntry` has a
/// defined `DecoderFunctor`. If a new content type is added to a registry
/// entry without a corresponding decoder, this axiom fails at test time.
///
/// Structural / type-theoretic — totality of the decoder dispatch over the
/// `ContentType` variants actually referenced in the registry.
pub struct DecoderTotalityPerContentType;

impl Axiom for DecoderTotalityPerContentType {
    fn verify(&self) -> Verdict {
        for entry in crate::applied::data_provisioning::registry::DATA_SOURCES {
            if !crate::applied::data_provisioning::decoders::has_decoder_for(entry.content_type) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DecoderTotalityPerContentType",
        "every ContentType in use has a defined decoder",
        "Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3 — R1 reusable"
    );
}

pr4xis::register_axiom!(
    DecoderTotalityPerContentType,
    "Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3 — R1 reusable"
);

/// Axiom: every resolved identity claim uses a LEAF `IdentityConcept` — not
/// a family or the root. A claim with an abstract family concept would be
/// ill-formed because families do not specify a verification scheme.
///
/// Mirrors the artifact_identity ontology's taxonomy invariant: claims are
/// expressed at the leaves, never at internal subsumption nodes.
pub struct IdentityClaimsUseLeaves;

impl Axiom for IdentityClaimsUseLeaves {
    fn verify(&self) -> Verdict {
        use crate::formal::meta::artifact_identity::ontology::is_leaf;
        for entry in crate::applied::data_provisioning::registry::DATA_SOURCES {
            if let Some(identity) =
                crate::applied::data_provisioning::registry::resolve_identity(entry.name)
            {
                for claim in &identity.0 {
                    if !is_leaf(&claim.concept) {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "IdentityClaimsUseLeaves",
        "every IdentityClaim uses a leaf IdentityConcept, not a family or root",
        "Dolstra (2006) The Purely Functional Software Deployment Model §5.1"
    );
}

pr4xis::register_axiom!(
    IdentityClaimsUseLeaves,
    "Dolstra (2006) The Purely Functional Software Deployment Model §5.1"
);

// Silence unused-import warning; IdentityConcept is re-exported for callers.
#[allow(dead_code)]
fn _identity_concept_witness(_: IdentityConcept) {}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<DataProvisioningCategory>();
    }

    #[test]
    fn ontology_validates() {
        DataProvisioningOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn seven_concepts() {
        // DataSource, DataCache, ProvisioningEvent, DecoderFunctor,
        // VerifiedDataset, StaleDataset, MissingDataset.
        assert_eq!(DataProvisioningConcept::variants().len(), 7);
    }

    #[test]
    fn lifecycle_states_subsume_data_source() {
        let sub: Vec<_> = DataProvisioningCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == DataProvisioningRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for state in [
            DataProvisioningConcept::VerifiedDataset,
            DataProvisioningConcept::StaleDataset,
            DataProvisioningConcept::MissingDataset,
        ] {
            assert!(
                sub.contains(&(state, DataProvisioningConcept::DataSource)),
                "{:?} should subsume DataSource",
                state
            );
        }
    }

    #[test]
    fn lifecycle_states_pairwise_oppose() {
        let opp: Vec<_> = DataProvisioningCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == DataProvisioningRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        let states = [
            DataProvisioningConcept::VerifiedDataset,
            DataProvisioningConcept::StaleDataset,
            DataProvisioningConcept::MissingDataset,
        ];
        for a in states {
            for b in states {
                if a != b {
                    assert!(
                        opp.contains(&(a, b)),
                        "lifecycle states {:?} and {:?} should oppose",
                        a,
                        b
                    );
                }
            }
        }
    }

    #[test]
    fn verified_is_usable_locally() {
        assert_eq!(
            IsUsableLocally.get(&DataProvisioningConcept::VerifiedDataset),
            Some(true)
        );
    }

    #[test]
    fn stale_and_missing_trigger_update() {
        assert_eq!(
            TriggersUpdate.get(&DataProvisioningConcept::StaleDataset),
            Some(true)
        );
        assert_eq!(
            TriggersUpdate.get(&DataProvisioningConcept::MissingDataset),
            Some(true)
        );
    }

    #[test]
    fn every_data_source_has_identity_axiom() {
        assert!(EveryDataSourceHasIdentity.verify().is_ok());
    }

    #[test]
    fn registry_uniqueness_axiom() {
        assert!(RegistryUniquenessByName.verify().is_ok());
    }

    #[test]
    fn decoder_totality_axiom() {
        assert!(DecoderTotalityPerContentType.verify().is_ok());
    }

    #[test]
    fn identity_claims_use_leaves_axiom() {
        assert!(IdentityClaimsUseLeaves.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = DataProvisioningConcept> {
        proptest::sample::select(DataProvisioningConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in DataProvisioningCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in DataProvisioningOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_is_usable_total_on_lifecycle(c in arb_concept()) {
            // IsUsableLocally is total on the three lifecycle states and
            // None on the four non-state concepts.
            let v = IsUsableLocally.get(&c);
            let is_state = matches!(
                c,
                DataProvisioningConcept::VerifiedDataset
                | DataProvisioningConcept::StaleDataset
                | DataProvisioningConcept::MissingDataset
            );
            prop_assert_eq!(v.is_some(), is_state);
        }

        #[test]
        fn prop_triggers_update_total_on_lifecycle(c in arb_concept()) {
            let v = TriggersUpdate.get(&c);
            let is_state = matches!(
                c,
                DataProvisioningConcept::VerifiedDataset
                | DataProvisioningConcept::StaleDataset
                | DataProvisioningConcept::MissingDataset
            );
            prop_assert_eq!(v.is_some(), is_state);
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = DataProvisioningCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == DataProvisioningRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }

        #[test]
        fn prop_subsumption_targets_data_source(_seed in any::<u32>()) {
            // Every is-a edge in this ontology has DataSource as target.
            for m in DataProvisioningCategory::morphisms() {
                if m.kind() == DataProvisioningRelationKind::Subsumption {
                    prop_assert_eq!(m.target(), DataProvisioningConcept::DataSource);
                }
            }
        }
    }
}

//! Identity extractors — one per leaf of the taxonomy.
//!
//! Each file in this module defines the verification logic for one
//! `IdentityConcept` leaf. The initial PR implements two real extractors
//! (`raw_hash` and `xml_element_attribute`) and stubs the remaining 18;
//! stubs return `VerificationResult::Unverifiable { reason: "not yet
//! implemented" }` so the axiom `VerificationFailClosed` still holds.
//!
//! Adding a real implementation for any stubbed scheme is a focused PR that
//! touches exactly one file here.

use super::ontology::IdentityConcept;

pub mod raw_hash;
pub mod xml_element_attribute;

// Stubs — grouped by family for reviewability. Each stub is a thin module
// that returns Unverifiable; real implementations land as follow-up PRs.

// CryptographicSignature family (issue #70 tracks Sigstore)
pub mod ed25519_raw;
pub mod minisign;
pub mod openpgp;
pub mod sigstore_attestation;
pub mod ssh_signature;
pub mod x509_signature;

// ContentHash family (RawHash is real above)
pub mod bittorrent_info_hash;
pub mod git_object_sha;
pub mod ipfs_cid;
pub mod nix_store_path;

// PersistentIdentifier family
pub mod ark;
pub mod doi;
pub mod handle;
pub mod purl;

// SelfDescribingMetadata family (XmlElementAttribute is real above)
pub mod dct_identifier;
pub mod owl_version_info;
pub mod owl_version_iri;
pub mod xml_schema_version;

/// Does a leaf `IdentityConcept` have a defined extractor module in this
/// crate? Enumerates every leaf explicitly so the Rust compiler fails the
/// build if a new `IdentityConcept` variant is added without a matching
/// arm here. Family and root concepts return `false` because only leaves
/// are ever used in `IdentityClaim` instances.
///
/// This is the real backing for the `EverySchemeHasAnExtractor` axiom —
/// the axiom walks every leaf and asserts this returns `true`.
pub fn extractor_exists_for(concept: &IdentityConcept) -> bool {
    use IdentityConcept::*;
    match concept {
        // Root + families have no direct extractor; they are abstract.
        Identity | CryptographicSignature | ContentHash | PersistentIdentifier
        | SelfDescribingMetadata => false,

        // Every leaf below has an extractor module (real or stub).
        // CryptographicSignature family
        OpenPgp | SigstoreAttestation | SshSignature | Minisign | X509Signature | Ed25519Raw
        // ContentHash family
        | RawHash | GitObjectSha | IpfsCid | NixStorePath | BittorrentInfoHash
        // PersistentIdentifier family
        | Doi | Handle | Ark | Purl
        // SelfDescribingMetadata family
        | OwlVersionIri | OwlVersionInfo | DctIdentifier | XmlElementAttribute
        | XmlSchemaVersion => true,
    }
}

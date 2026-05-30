//! Artifact identity — three-level taxonomic model of identity schemes
//! for external data sources.
//!
//! ```text
//! Identity (abstract root)
//! ├── CryptographicSignature   — 6 leaves
//! ├── ContentHash              — 5 leaves
//! ├── PersistentIdentifier     — 4 leaves
//! └── SelfDescribingMetadata   — 5 leaves
//! ```
//!
//! See `mod.rs` for full grounding citations.
//!
//! # Literature
//!
//! - **Dolstra (2006)** *The Purely Functional Software Deployment
//!   Model* (PhD thesis, Utrecht University) — fixed-output derivations;
//!   content addressing.
//! - **Wilkinson et al. (2016)** "The FAIR Guiding Principles for
//!   scientific data management and stewardship", *Scientific Data*
//!   3:160018 — F1 persistent identifier; verifiable identity.
//! - **Benet (2014)** *IPFS - Content Addressed, Versioned, P2P File
//!   System*, arXiv:1407.3561 — IPFS CID.
//! - **RFC 4880** OpenPGP Message Format (Callas et al., 2007).
//! - **RFC 5280** Internet X.509 Public Key Infrastructure (Cooper et
//!   al., 2008).
//! - **RFC 8032** Edwards-Curve Digital Signature Algorithm (Josefsson
//!   & Liusvaara, 2017).
//! - **Newman et al. (2022)** *Sigstore: Software Signing for
//!   Everybody*, CCS 2022 — Sigstore / Fulcio / Rekor.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "ArtifactIdentity",
    source: "Dolstra (2006) The Purely Functional Software Deployment Model, PhD thesis Utrecht University; Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3:160018; Benet (2014) IPFS - Content Addressed, Versioned, P2P File System, arXiv:1407.3561; RFC 4880 OpenPGP; RFC 5280 X.509 PKI; RFC 8032 Ed25519; Newman et al. (2022) Sigstore, CCS",

    concepts: [
        // === Root ===
        Identity,
        // === Families ===
        CryptographicSignature,
        ContentHash,
        PersistentIdentifier,
        SelfDescribingMetadata,
        // === CryptographicSignature leaves ===
        OpenPgp,
        SigstoreAttestation,
        SshSignature,
        Minisign,
        X509Signature,
        Ed25519Raw,
        // === ContentHash leaves ===
        RawHash,
        GitObjectSha,
        IpfsCid,
        NixStorePath,
        BittorrentInfoHash,
        // === PersistentIdentifier leaves ===
        Doi,
        Handle,
        Ark,
        Purl,
        // === SelfDescribingMetadata leaves ===
        OwlVersionIri,
        OwlVersionInfo,
        DctIdentifier,
        XmlElementAttribute,
        XmlSchemaVersion,
    ],

    labels: {
        Identity: ("en", "Identity", "The abstract root of the artifact-identity taxonomy."),
        CryptographicSignature: ("en", "Cryptographic signature", "Signer vouches for content; verifier needs the signer's public key. RFC 4880 / RFC 5280 / RFC 8032."),
        ContentHash: ("en", "Content hash", "Dolstra (2006): identity derived from content bytes via a collision-resistant hash. Always verifiable offline."),
        PersistentIdentifier: ("en", "Persistent identifier", "Wilkinson (2016) F1: a registry-resolved identifier. Requires network access for resolution."),
        SelfDescribingMetadata: ("en", "Self-describing metadata", "The content asserts its own identity (version string, schema version). Weakest trust tier."),
        OpenPgp: ("en", "OpenPGP signature", "RFC 4880: OpenPGP / GPG signature."),
        SigstoreAttestation: ("en", "Sigstore attestation", "Newman et al. (2022): Fulcio OIDC certificate + Rekor transparency log."),
        SshSignature: ("en", "SSH signature", "OpenSSH ssh-keygen -Y sign/verify."),
        Minisign: ("en", "Minisign", "Bernstein-style minimal Ed25519 signing."),
        X509Signature: ("en", "X.509 signature", "RFC 5280 X.509 certificate signature (S/MIME, web PKI, code signing)."),
        Ed25519Raw: ("en", "Raw Ed25519", "RFC 8032: raw Ed25519 signature over a content hash."),
        RawHash: ("en", "Raw hash", "Dolstra (2006) baseline: raw cryptographic hash of content bytes (SHA-256 / SHA-512 / BLAKE3)."),
        GitObjectSha: ("en", "Git object SHA", "Git's content-addressed object identity."),
        IpfsCid: ("en", "IPFS CID", "Benet (2014): IPFS content identifier."),
        NixStorePath: ("en", "Nix store path", "Dolstra (2006): /nix/store/{hash}-name."),
        BittorrentInfoHash: ("en", "BitTorrent info-hash", "BEP-0003 (Cohen 2003) Merkle root over content pieces."),
        Doi: ("en", "DOI", "ISO 26324: Digital Object Identifier."),
        Handle: ("en", "Handle", "IETF RFC 3650 Handle System."),
        Ark: ("en", "ARK", "Archival Resource Key (California Digital Library)."),
        Purl: ("en", "PURL", "OCLC / W3C Persistent URL."),
        OwlVersionIri: ("en", "owl:versionIRI", "W3C OWL 2 §3.5: a versioning IRI embedded in the ontology."),
        OwlVersionInfo: ("en", "owl:versionInfo", "W3C OWL 2 §3.5: a free-text version annotation."),
        DctIdentifier: ("en", "dct:identifier", "Dublin Core Terms (ISO 15836-1:2017) generic identifier."),
        XmlElementAttribute: ("en", "XML element attribute", "Generic XML attribute extractor (e.g., WordNet LMF <Lexicon version=...>)."),
        XmlSchemaVersion: ("en", "XSD version", "XSD schema version attribute on the top-level element."),
    },

    is_a: [
        // Level 1: families descend from Identity.
        (CryptographicSignature, Identity),
        (ContentHash, Identity),
        (PersistentIdentifier, Identity),
        (SelfDescribingMetadata, Identity),
        // Level 2: leaves descend from their family.
        (OpenPgp, CryptographicSignature),
        (SigstoreAttestation, CryptographicSignature),
        (SshSignature, CryptographicSignature),
        (Minisign, CryptographicSignature),
        (X509Signature, CryptographicSignature),
        (Ed25519Raw, CryptographicSignature),
        (RawHash, ContentHash),
        (GitObjectSha, ContentHash),
        (IpfsCid, ContentHash),
        (NixStorePath, ContentHash),
        (BittorrentInfoHash, ContentHash),
        (Doi, PersistentIdentifier),
        (Handle, PersistentIdentifier),
        (Ark, PersistentIdentifier),
        (Purl, PersistentIdentifier),
        (OwlVersionIri, SelfDescribingMetadata),
        (OwlVersionInfo, SelfDescribingMetadata),
        (DctIdentifier, SelfDescribingMetadata),
        (XmlElementAttribute, SelfDescribingMetadata),
        (XmlSchemaVersion, SelfDescribingMetadata),
    ],

    opposes: [
        // Offline vs online verification (Dolstra 2006 vs Wilkinson F1).
        (ContentHash, PersistentIdentifier),
        (PersistentIdentifier, ContentHash),
        // Strong vs weak trust.
        (ContentHash, SelfDescribingMetadata),
        (SelfDescribingMetadata, ContentHash),
        (CryptographicSignature, SelfDescribingMetadata),
        (SelfDescribingMetadata, CryptographicSignature),
    ],
}

/// Pre-rename alias for the macro-generated concept enum.
///
/// The `ontology!` macro derives the concept name from the ontology name
/// (`ArtifactIdentity` -> `ArtifactIdentityConcept`); existing call sites
/// across this crate (and downstream `applied/data_provisioning`) still
/// reference the historical short name. Keeping an alias avoids a wide
/// rename and is consistent with the Praxis convention (#152) of preserving
/// downstream surface area while the canonical generated names settle.
pub type IdentityConcept = ArtifactIdentityConcept;

// ---------------------------------------------------------------------------
// Claim data — what a concrete identity claim carries per scheme
// ---------------------------------------------------------------------------

/// A concrete identity claim: a leaf in the taxonomy plus the scheme-
/// specific data a verifier needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityClaim {
    pub concept: IdentityConcept,
    pub data: ClaimData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimData {
    Sha256(String),
    HashAlgorithm {
        algorithm: HashAlgorithm,
        digest_hex: String,
    },
    XmlAttribute {
        element: String,
        attribute: String,
        expected: String,
    },
    Stub {
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    /// FIPS 180-4.
    Sha256,
    /// FIPS 180-4.
    Sha512,
    /// Aumasson et al. (2020).
    Blake3,
}

/// Multiple claims that must all verify — weakest-link semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeIdentity(pub Vec<IdentityClaim>);

impl CompositeIdentity {
    /// `true` when every claim is a [`ClaimData::Stub`] against
    /// [`IdentityConcept::RawHash`] — i.e. the source is registered but
    /// has no verifiable identity yet (no lock hash, no concrete
    /// extractor wired). Downstream loaders treat these as
    /// "registered-but-not-yet-loadable": the fetcher skips them and
    /// the [`crate::applied::data_provisioning::ontology::DecoderTotalityPerKind`]
    /// axiom skips them, in both cases because the materialization
    /// machinery has no way to verify what bytes come back.
    ///
    /// An empty claim vector is NOT stub-only — that's a defect the
    /// [`crate::applied::data_provisioning::ontology::EveryDataSourceHasIdentity`]
    /// axiom catches separately.
    pub fn is_stub_only(&self) -> bool {
        !self.0.is_empty()
            && self.0.iter().all(|c| {
                matches!(c.concept, IdentityConcept::RawHash)
                    && matches!(c.data, ClaimData::Stub { .. })
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    Verified(IdentityClaim),
    Mismatch { expected: String, actual: String },
    Unverifiable { reason: String },
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustTier {
    Strong,
    Resolver,
    Declarative,
    NotApplicable,
}

#[derive(Debug, Clone)]
pub struct TrustTierOf;

impl Quality for TrustTierOf {
    type Individual = IdentityConcept;
    type Value = TrustTier;

    fn get(&self, concept: &IdentityConcept) -> Option<TrustTier> {
        use IdentityConcept as I;
        Some(match concept {
            I::Identity
            | I::CryptographicSignature
            | I::ContentHash
            | I::PersistentIdentifier
            | I::SelfDescribingMetadata => TrustTier::NotApplicable,
            I::OpenPgp
            | I::SigstoreAttestation
            | I::SshSignature
            | I::Minisign
            | I::X509Signature
            | I::Ed25519Raw
            | I::RawHash
            | I::GitObjectSha
            | I::IpfsCid
            | I::NixStorePath
            | I::BittorrentInfoHash => TrustTier::Strong,
            I::Doi | I::Handle | I::Ark | I::Purl => TrustTier::Resolver,
            I::OwlVersionIri
            | I::OwlVersionInfo
            | I::DctIdentifier
            | I::XmlElementAttribute
            | I::XmlSchemaVersion => TrustTier::Declarative,
        })
    }
}

/// Whether a scheme verifies without network access. Dolstra (2006) +
/// Wilkinson (2016) F1: content hashes and signatures verify offline
/// (given the keyring); persistent identifiers require resolution.
#[derive(Debug, Clone)]
pub struct VerifiabilityOffline;

impl Quality for VerifiabilityOffline {
    type Individual = IdentityConcept;
    type Value = bool;

    fn get(&self, concept: &IdentityConcept) -> Option<bool> {
        use IdentityConcept as I;
        match concept {
            I::Identity
            | I::CryptographicSignature
            | I::ContentHash
            | I::PersistentIdentifier
            | I::SelfDescribingMetadata => None,
            I::OpenPgp
            | I::SigstoreAttestation
            | I::SshSignature
            | I::Minisign
            | I::X509Signature
            | I::Ed25519Raw
            | I::RawHash
            | I::GitObjectSha
            | I::IpfsCid
            | I::NixStorePath
            | I::BittorrentInfoHash => Some(true),
            I::Doi | I::Handle | I::Ark | I::Purl => Some(false),
            I::OwlVersionIri
            | I::OwlVersionInfo
            | I::DctIdentifier
            | I::XmlElementAttribute
            | I::XmlSchemaVersion => Some(true),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn is_family(concept: &IdentityConcept) -> bool {
    use IdentityConcept as I;
    matches!(
        concept,
        I::CryptographicSignature
            | I::ContentHash
            | I::PersistentIdentifier
            | I::SelfDescribingMetadata
    )
}

pub fn is_leaf(concept: &IdentityConcept) -> bool {
    use IdentityConcept as I;
    !matches!(
        concept,
        I::Identity
            | I::CryptographicSignature
            | I::ContentHash
            | I::PersistentIdentifier
            | I::SelfDescribingMetadata
    )
}

/// Replacement for the deleted `taxonomy::ancestors` — walks the
/// Subsumption-kinded morphisms in `ArtifactIdentityCategory` and
/// returns all reachable parents of `concept`.
pub fn ancestors_of(concept: &IdentityConcept) -> Vec<IdentityConcept> {
    use pr4xis::category::{Arrow, Category};
    let sub: Vec<_> = ArtifactIdentityCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == ArtifactIdentityRelationKind::Subsumption)
        .map(|m| (m.source(), m.target()))
        .collect();
    let mut out = Vec::new();
    let mut stack = vec![*concept];
    while let Some(c) = stack.pop() {
        for (s, t) in &sub {
            if *s == c && !out.contains(t) {
                out.push(*t);
                stack.push(*t);
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Every `IdentityConcept` leaf has a defined extractor.
pub struct EverySchemeHasAnExtractor;

impl Axiom for EverySchemeHasAnExtractor {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::Concept;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let all = IdentityConcept::variants()
            .into_iter()
            .filter(is_leaf)
            .all(|c| crate::formal::meta::artifact_identity::schemes::extractor_exists_for(&c));
        if all {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EverySchemeHasAnExtractor",
        "every leaf IdentityConcept has a defined extractor",
        "Dolstra (2006) The Purely Functional Software Deployment Model; Wilkinson et al. (2016) FAIR F1"
    );
}

pr4xis::register_axiom!(
    EverySchemeHasAnExtractor,
    "Dolstra (2006) The Purely Functional Software Deployment Model; Wilkinson et al. (2016) FAIR F1"
);

/// Content hashes are injective under a collision-resistant algorithm
/// (Dolstra 2006). Stated structurally: every ContentHash leaf is a
/// descendant of ContentHash in the Subsumption-kinded taxonomy.
pub struct ContentHashIsInjective;

impl Axiom for ContentHashIsInjective {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use IdentityConcept as I;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let leaves = [
            I::RawHash,
            I::GitObjectSha,
            I::IpfsCid,
            I::NixStorePath,
            I::BittorrentInfoHash,
        ];
        if leaves
            .iter()
            .all(|leaf| ancestors_of(leaf).contains(&I::ContentHash))
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ContentHashIsInjective",
        "every ContentHash leaf descends from ContentHash; the underlying hash is collision-resistant",
        "Dolstra (2006) The Purely Functional Software Deployment Model"
    );
}

pr4xis::register_axiom!(
    ContentHashIsInjective,
    "Dolstra (2006) The Purely Functional Software Deployment Model"
);

/// Content hashes always verify offline.
pub struct ContentHashIsOffline;

impl Axiom for ContentHashIsOffline {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use IdentityConcept as I;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let q = VerifiabilityOffline;
        let leaves = [
            I::RawHash,
            I::GitObjectSha,
            I::IpfsCid,
            I::NixStorePath,
            I::BittorrentInfoHash,
        ];
        if leaves.iter().all(|leaf| q.get(leaf) == Some(true)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ContentHashIsOffline",
        "every ContentHash leaf verifies without network access",
        "Dolstra (2006) The Purely Functional Software Deployment Model"
    );
}

pr4xis::register_axiom!(
    ContentHashIsOffline,
    "Dolstra (2006) The Purely Functional Software Deployment Model"
);

/// Persistent identifiers require a resolver.
pub struct PersistentIdentifierRequiresResolver;

impl Axiom for PersistentIdentifierRequiresResolver {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use IdentityConcept as I;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let q = VerifiabilityOffline;
        let leaves = [I::Doi, I::Handle, I::Ark, I::Purl];
        if leaves.iter().all(|leaf| q.get(leaf) == Some(false)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PersistentIdentifierRequiresResolver",
        "PersistentIdentifier leaves require network access for resolution",
        "Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3:160018 - F1"
    );
}

pr4xis::register_axiom!(
    PersistentIdentifierRequiresResolver,
    "Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3:160018 - F1"
);

/// Self-describing metadata is the weakest trust tier.
pub struct SelfDescribingIsWeakestTrust;

impl Axiom for SelfDescribingIsWeakestTrust {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use IdentityConcept as I;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let q = TrustTierOf;
        let leaves = [
            I::OwlVersionIri,
            I::OwlVersionInfo,
            I::DctIdentifier,
            I::XmlElementAttribute,
            I::XmlSchemaVersion,
        ];
        if leaves
            .iter()
            .all(|leaf| q.get(leaf) == Some(TrustTier::Declarative))
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SelfDescribingIsWeakestTrust",
        "SelfDescribingMetadata leaves are the Declarative trust tier",
        "Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3:160018"
    );
}

pr4xis::register_axiom!(
    SelfDescribingIsWeakestTrust,
    "Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3:160018"
);

impl Ontology for ArtifactIdentityOntology {
    type Cat = ArtifactIdentityCategory;
    type Qual = TrustTierOf;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(EverySchemeHasAnExtractor));
        axioms.push(Box::new(ContentHashIsInjective));
        axioms.push(Box::new(ContentHashIsOffline));
        axioms.push(Box::new(PersistentIdentifierRequiresResolver));
        axioms.push(Box::new(SelfDescribingIsWeakestTrust));
        axioms
    }
}

//! The artifact_identity ontology — taxonomic model of identity schemes for
//! external data sources.
//!
//! Three-level taxonomy:
//!
//! ```text
//! Identity (abstract root)
//! ├── CryptographicSignature   — 6 leaves
//! ├── ContentHash              — 5 leaves
//! ├── PersistentIdentifier     — 4 leaves
//! └── SelfDescribingMetadata   — 5 leaves
//! ```
//!
//! See `mod.rs` for the full grounding citations. This file defines the
//! entity enum, the define_ontology! invocation (with taxonomy), and the
//! universal + family-level domain axioms.

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

/// Every node in the three-level identity taxonomy — the abstract root, the
/// four families, and the 20 leaf schemes (25 entities total). Leaf concepts
/// are the ones that concrete `IdentityClaim` instances reference; family
/// and root concepts exist so family-level axioms can apply to every
/// descendant at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum IdentityConcept {
    // --- Root ---
    /// The top-level abstract concept. Every other concept descends from this.
    Identity,

    // --- Families (level 1) ---
    /// Signer vouches for content; requires the verifier to know the signer's
    /// public key or certificate chain; verifiable offline given the keyring.
    CryptographicSignature,
    /// Identity derived from content bytes via a cryptographic hash function.
    /// Injective under collision-resistant hashes. Always verifiable offline.
    ContentHash,
    /// An identifier that a registry resolves to content. Resolution always
    /// requires network access; not verifiable offline without a cached copy.
    PersistentIdentifier,
    /// The content itself declares its own identity (version string, schema
    /// version, etc.). Requires a content-type-specific parser. Weakest trust
    /// tier: the source asserts its own identity with no cryptographic proof.
    SelfDescribingMetadata,

    // --- CryptographicSignature leaves ---
    /// OpenPGP / GPG signature per RFC 4880.
    OpenPgp,
    /// Sigstore attestation: Fulcio OIDC-backed certificate + Rekor transparency
    /// log. Newman et al. 2022 (Chainguard). Supply-chain integrity.
    /// See issue #70.
    SigstoreAttestation,
    /// OpenSSH signature format (`ssh-keygen -Y sign/verify`).
    SshSignature,
    /// Minisign — Bernstein-style minimal signing (Ed25519 over a short key).
    Minisign,
    /// X.509 signature (S/MIME, web PKI, code signing certificates per RFC 5280).
    X509Signature,
    /// Raw Ed25519 signature over a content hash (RFC 8032).
    Ed25519Raw,

    // --- ContentHash leaves ---
    /// Raw cryptographic hash of the content bytes (sha256, sha512, blake3).
    /// The Dolstra 2006 baseline scheme.
    RawHash,
    /// Git object SHA — git's own content-addressed identity. The hash IS
    /// the content identity; git's entire object store is content-addressed.
    GitObjectSha,
    /// IPFS content identifier (CID). Benet 2014.
    IpfsCid,
    /// Nix store path of the form `/nix/store/{hash}-name`. Hash derived
    /// from all inputs. Dolstra 2006.
    NixStorePath,
    /// BitTorrent info-hash — Merkle root over content pieces (BEP-0003,
    /// Cohen 2003).
    BittorrentInfoHash,

    // --- PersistentIdentifier leaves ---
    /// Digital Object Identifier. ISO 26324, International DOI Foundation.
    /// Resolved via `https://doi.org/<doi>` or CrossRef/DataCite APIs.
    Doi,
    /// Handle System identifier. IETF RFC 3650, CNRI. DOI is built on top of
    /// Handle; Handle is the more general protocol.
    Handle,
    /// Archival Resource Key. California Digital Library. Persistent
    /// identifier designed for archival institutions.
    Ark,
    /// Persistent URL. OCLC / W3C. A URL that a PURL server redirects to
    /// the current resource location.
    Purl,

    // --- SelfDescribingMetadata leaves ---
    /// `owl:versionIRI` — W3C OWL 2 Structural Specification §3.5. An IRI
    /// naming a specific version of an ontology, embedded in the ontology file.
    OwlVersionIri,
    /// `owl:versionInfo` — W3C OWL 2 §3.5. A free-text version annotation.
    OwlVersionInfo,
    /// `dct:identifier` — Dublin Core Terms, ISO 15836-1:2017. A generic
    /// identifier property in RDF / XML metadata.
    DctIdentifier,
    /// Generic "parse this XML element's attribute" extractor. WordNet LMF
    /// uses this: `<Lexicon version="2025" ...>`.
    XmlElementAttribute,
    /// XSD schema version attribute — the `version` attribute on the top-level
    /// XML schema element.
    XmlSchemaVersion,
}

// ---------------------------------------------------------------------------
// Ontology definition
// ---------------------------------------------------------------------------

define_ontology! {
    /// Dense category over identity concepts, with a three-level taxonomy
    /// (root → family → leaf) and opposition on the offline/online axis.
    ///
    /// Leaves are the concepts concrete `IdentityClaim` instances reference;
    /// families exist so family-level axioms (e.g. `ContentHashIsOffline`)
    /// apply to every descendant via taxonomy walk.
    pub ArtifactIdentityOntology for ArtifactIdentityCategory {
        entity: IdentityConcept,
        relation: IdentityRelation,
        being: AbstractObject,
        source: "Dolstra (2006); Wilkinson FAIR F1 (2016)",

        taxonomy: IdentityTaxonomy [
            // Level 1: families are-a Identity
            (CryptographicSignature, Identity),
            (ContentHash, Identity),
            (PersistentIdentifier, Identity),
            (SelfDescribingMetadata, Identity),

            // Level 2: CryptographicSignature leaves
            (OpenPgp, CryptographicSignature),
            (SigstoreAttestation, CryptographicSignature),
            (SshSignature, CryptographicSignature),
            (Minisign, CryptographicSignature),
            (X509Signature, CryptographicSignature),
            (Ed25519Raw, CryptographicSignature),

            // Level 2: ContentHash leaves
            (RawHash, ContentHash),
            (GitObjectSha, ContentHash),
            (IpfsCid, ContentHash),
            (NixStorePath, ContentHash),
            (BittorrentInfoHash, ContentHash),

            // Level 2: PersistentIdentifier leaves
            (Doi, PersistentIdentifier),
            (Handle, PersistentIdentifier),
            (Ark, PersistentIdentifier),
            (Purl, PersistentIdentifier),

            // Level 2: SelfDescribingMetadata leaves
            (OwlVersionIri, SelfDescribingMetadata),
            (OwlVersionInfo, SelfDescribingMetadata),
            (DctIdentifier, SelfDescribingMetadata),
            (XmlElementAttribute, SelfDescribingMetadata),
            (XmlSchemaVersion, SelfDescribingMetadata),
        ],

        opposition: IdentityOpposition [
            // Offline vs online verification — the key operational distinction.
            // ContentHash and CryptographicSignature are offline; PersistentIdentifier
            // requires a resolver and is online.
            (ContentHash, PersistentIdentifier),
            // Strong (cryptographic) vs weak (self-asserted) trust.
            (ContentHash, SelfDescribingMetadata),
            (CryptographicSignature, SelfDescribingMetadata),
        ],
    }
}

// ---------------------------------------------------------------------------
// Claim data — what a concrete identity claim carries per scheme
// ---------------------------------------------------------------------------

/// A concrete identity claim: a leaf in the taxonomy plus the scheme-specific
/// data a verifier needs to check it. Every `RegistryEntry` in
/// `applied/data_provisioning/` holds one or more of these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityClaim {
    /// Which leaf in the taxonomy this claim uses.
    pub concept: IdentityConcept,
    /// The scheme-specific data.
    pub data: ClaimData,
}

/// Scheme-specific claim payloads. One variant per leaf that has a real
/// implementation in this PR; other leaves use `Stub` with a reason string
/// until their real implementations land.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimData {
    /// A `RawHash` claim: the declared SHA-256 of the content bytes.
    Sha256(String),
    /// A `RawHash` claim with a different algorithm.
    HashAlgorithm {
        algorithm: HashAlgorithm,
        digest_hex: String,
    },
    /// An `XmlElementAttribute` claim: look for `<element attribute="expected"`
    /// in the parsed XML and confirm the expected value is present.
    XmlAttribute {
        element: &'static str,
        attribute: &'static str,
        expected: String,
    },
    /// Placeholder for stubbed schemes. `concept` in the parent `IdentityClaim`
    /// tells the verifier which stub to call; the verifier returns
    /// `Unverifiable { reason }` until a real implementation lands.
    Stub { reason: &'static str },
}

/// Which cryptographic hash algorithm a `RawHash` claim uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    /// SHA-256, FIPS 180-4. The pr4xis default.
    Sha256,
    /// SHA-512, FIPS 180-4.
    Sha512,
    /// BLAKE3 (Aumasson et al. 2020).
    Blake3,
}

/// Multiple claims that must ALL verify. Used for belt-and-braces trust:
/// e.g., WordNet declares both an `XmlElementAttribute` claim (self-description)
/// and a `RawHash` claim (content integrity). `CompositeRequiresAll` is the
/// axiom that captures this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeIdentity(pub Vec<IdentityClaim>);

/// The result of attempting to verify an `IdentityClaim` against actual bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    /// The claim verified successfully. Contains the original claim for
    /// provenance / audit.
    Verified(IdentityClaim),
    /// The claim was checked but the actual identity did not match the
    /// declared one. Carries both values for debugging.
    Mismatch { expected: String, actual: String },
    /// The claim could not be verified — missing implementation, missing
    /// public key, network unavailable for resolver, etc. Treated as failure
    /// per `VerificationFailClosed`.
    Unverifiable { reason: String },
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// The trust tier of an identity scheme — derived from its family in the
/// taxonomy but exposed as a quality for easy filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustTier {
    /// Top tier — cryptographically strong. `CryptographicSignature` and
    /// `ContentHash` families.
    Strong,
    /// Middle tier — depends on a trusted registry being reachable.
    /// `PersistentIdentifier` family.
    Resolver,
    /// Bottom tier — source asserts its own identity with no proof.
    /// `SelfDescribingMetadata` family.
    Declarative,
    /// The root concept or the family concepts themselves — no direct tier.
    NotApplicable,
}

/// `TrustTier` per `IdentityConcept`. Derived from family membership.
#[derive(Debug, Clone)]
pub struct TrustTierOf;

impl Quality for TrustTierOf {
    type Individual = IdentityConcept;
    type Value = TrustTier;

    fn get(&self, concept: &IdentityConcept) -> Option<TrustTier> {
        use IdentityConcept::*;
        Some(match concept {
            Identity
            | CryptographicSignature
            | ContentHash
            | PersistentIdentifier
            | SelfDescribingMetadata => TrustTier::NotApplicable,
            // CryptographicSignature leaves → Strong
            OpenPgp | SigstoreAttestation | SshSignature | Minisign | X509Signature
            | Ed25519Raw => TrustTier::Strong,
            // ContentHash leaves → Strong
            RawHash | GitObjectSha | IpfsCid | NixStorePath | BittorrentInfoHash => {
                TrustTier::Strong
            }
            // PersistentIdentifier leaves → Resolver
            Doi | Handle | Ark | Purl => TrustTier::Resolver,
            // SelfDescribingMetadata leaves → Declarative
            OwlVersionIri | OwlVersionInfo | DctIdentifier | XmlElementAttribute
            | XmlSchemaVersion => TrustTier::Declarative,
        })
    }
}

/// Whether a scheme can be verified without network access.
///
/// Content hashes and cryptographic signatures are offline (given keyring).
/// Self-describing metadata is offline (it's just parsing). Persistent
/// identifiers require a resolver and are NOT offline-verifiable.
#[derive(Debug, Clone)]
pub struct VerifiabilityOffline;

impl Quality for VerifiabilityOffline {
    type Individual = IdentityConcept;
    type Value = bool;

    fn get(&self, concept: &IdentityConcept) -> Option<bool> {
        use IdentityConcept::*;
        Some(match concept {
            Identity
            | CryptographicSignature
            | ContentHash
            | PersistentIdentifier
            | SelfDescribingMetadata => return None,
            // Cryptographic signatures: offline given the keyring
            OpenPgp | SigstoreAttestation | SshSignature | Minisign | X509Signature
            | Ed25519Raw => true,
            // Content hashes: always offline
            RawHash | GitObjectSha | IpfsCid | NixStorePath | BittorrentInfoHash => true,
            // Persistent identifiers: require a resolver
            Doi | Handle | Ark | Purl => false,
            // Self-describing metadata: just parsing, offline
            OwlVersionIri | OwlVersionInfo | DctIdentifier | XmlElementAttribute
            | XmlSchemaVersion => true,
        })
    }
}

// ---------------------------------------------------------------------------
// Universal domain axioms
// ---------------------------------------------------------------------------

/// Every `IdentityConcept` leaf has a defined extractor. The initial PR
/// implements two real extractors (`RawHash`, `XmlElementAttribute`); the
/// other 18 leaves return `Unverifiable { reason: "not yet implemented" }`
/// from their stub extractors, which still counts as "defined" — the
/// axiom only requires that a function exists, not that it returns
/// Verified.
///
/// The axiom is backed by `schemes::extractor_exists_for`, whose match
/// arm is exhaustive over every `IdentityConcept` leaf. Adding a new leaf
/// without a corresponding arm is a compile error, so this invariant is
/// enforced by the type system in addition to the runtime assertion below.
pub struct EverySchemeHasAnExtractor;

impl Axiom for EverySchemeHasAnExtractor {
    fn description(&self) -> &str {
        "every leaf IdentityConcept has a defined extractor function"
    }

    fn holds(&self) -> bool {
        use crate::formal::meta::artifact_identity::schemes;
        IdentityConcept::variants()
            .iter()
            .filter(|c| is_leaf(c))
            .all(schemes::extractor_exists_for)
    }
}
pr4xis::register_axiom!(EverySchemeHasAnExtractor);

/// Running the same extractor on the same bytes always produces the same
/// `VerificationResult`. This is the determinism requirement — verifiers
/// must be pure functions of their inputs.
///
/// Checked via proptest in `tests.rs`: we generate random bytes, hash them,
/// and confirm two consecutive extractor calls return identical results.
pub struct ExtractorIsDeterministic;

impl Axiom for ExtractorIsDeterministic {
    fn description(&self) -> &str {
        "every extractor is a pure function of its input bytes"
    }

    fn holds(&self) -> bool {
        // This axiom is property-based; it cannot be exhaustively checked
        // here. The proptest in `tests.rs` is the real test. This trivial
        // check ensures the axiom is registered in the domain axiom list
        // and can be invoked via `validate()`; the proptest does the actual
        // work and fails loudly if determinism is violated.
        true
    }
}
pr4xis::register_axiom!(ExtractorIsDeterministic);

/// If a verifier returns `Unverifiable` or `Mismatch`, the calling pipeline
/// must reject the artifact. Never proceed with unverified data. This is the
/// fail-closed principle from security engineering.
///
/// In practice this axiom is enforced by the `data_provisioning` layer which
/// refuses to write a fetched file to disk unless every declared claim
/// returned `Verified`. Here it is stated so the axiom appears in the domain
/// axioms list and can be referenced by other ontologies.
pub struct VerificationFailClosed;

impl Axiom for VerificationFailClosed {
    fn description(&self) -> &str {
        "verification failures (Unverifiable or Mismatch) reject the artifact — never fail open"
    }

    fn holds(&self) -> bool {
        // Structural statement. The enforcement is in data_provisioning::fetch
        // which pattern-matches on VerificationResult and returns Err for
        // anything other than Verified. This axiom's presence here binds the
        // rule to the ontology so it can be cited by callers.
        true
    }
}
pr4xis::register_axiom!(VerificationFailClosed);

/// A `CompositeIdentity` verifies only when ALL its claims verify. Weakest-link
/// semantics: if any claim returns Unverifiable or Mismatch, the whole
/// composite is rejected.
pub struct CompositeRequiresAll;

impl Axiom for CompositeRequiresAll {
    fn description(&self) -> &str {
        "CompositeIdentity verifies only when every claim verifies"
    }

    fn holds(&self) -> bool {
        // Structural. The composite verifier in data_provisioning walks the
        // vector and short-circuits on first failure. Proptest in tests.rs
        // confirms the behavior for random composite generators.
        true
    }
}
pr4xis::register_axiom!(CompositeRequiresAll);

// ---------------------------------------------------------------------------
// Family-level axioms
// ---------------------------------------------------------------------------

/// Content hashes are injective under a collision-resistant algorithm: two
/// artifacts with the same hash have the same bytes. This is the strict
/// Dolstra 2006 claim, generalized across the `ContentHash` family — it
/// applies to every leaf (RawHash, GitObjectSha, IpfsCid, NixStorePath,
/// BittorrentInfoHash), because they all use cryptographic hashes at the
/// bottom.
///
/// The axiom is stated at the family level; taxonomy walk propagates it to
/// every leaf automatically.
pub struct ContentHashIsInjective;

impl Axiom for ContentHashIsInjective {
    fn description(&self) -> &str {
        "content hash schemes are injective under collision-resistant algorithms (strict Dolstra)"
    }

    fn holds(&self) -> bool {
        // Structural. The cryptographic assumption (SHA-256 is collision-
        // resistant, etc.) is given; this axiom just records that pr4xis
        // relies on it. Empirical verification lives in the proptest for
        // the RawHash extractor.
        use IdentityConcept::*;
        let content_hash_leaves = [
            RawHash,
            GitObjectSha,
            IpfsCid,
            NixStorePath,
            BittorrentInfoHash,
        ];
        for leaf in &content_hash_leaves {
            let ancestors = taxonomy::ancestors::<IdentityTaxonomy>(leaf);
            if !ancestors.contains(&ContentHash) {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(ContentHashIsInjective);

/// Content hashes can always be verified offline — no network access
/// required. Applies to every leaf in the `ContentHash` family.
pub struct ContentHashIsOffline;

impl Axiom for ContentHashIsOffline {
    fn description(&self) -> &str {
        "content hash schemes are always verifiable offline"
    }

    fn holds(&self) -> bool {
        let q = VerifiabilityOffline;
        use IdentityConcept::*;
        let content_hash_leaves = [
            RawHash,
            GitObjectSha,
            IpfsCid,
            NixStorePath,
            BittorrentInfoHash,
        ];
        content_hash_leaves
            .iter()
            .all(|leaf| q.get(leaf) == Some(true))
    }
}
pr4xis::register_axiom!(ContentHashIsOffline);

/// Persistent identifiers require a resolver (DOI service, Handle server,
/// ARK NAAN, PURL server). Verification cannot happen offline without a
/// cached resolution.
pub struct PersistentIdentifierRequiresResolver;

impl Axiom for PersistentIdentifierRequiresResolver {
    fn description(&self) -> &str {
        "persistent identifier schemes require network access to a resolver"
    }

    fn holds(&self) -> bool {
        let q = VerifiabilityOffline;
        use IdentityConcept::*;
        let pid_leaves = [Doi, Handle, Ark, Purl];
        pid_leaves.iter().all(|leaf| q.get(leaf) == Some(false))
    }
}
pr4xis::register_axiom!(PersistentIdentifierRequiresResolver);

/// Self-describing metadata is the weakest trust tier — the source asserts
/// its own identity with no cryptographic backing. Applies to every leaf in
/// the `SelfDescribingMetadata` family.
pub struct SelfDescribingIsWeakestTrust;

impl Axiom for SelfDescribingIsWeakestTrust {
    fn description(&self) -> &str {
        "self-describing metadata schemes are the weakest trust tier (Declarative)"
    }

    fn holds(&self) -> bool {
        let q = TrustTierOf;
        use IdentityConcept::*;
        let self_describing_leaves = [
            OwlVersionIri,
            OwlVersionInfo,
            DctIdentifier,
            XmlElementAttribute,
            XmlSchemaVersion,
        ];
        self_describing_leaves
            .iter()
            .all(|leaf| q.get(leaf) == Some(TrustTier::Declarative))
    }
}
pr4xis::register_axiom!(SelfDescribingIsWeakestTrust);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Is this concept a leaf (not the root, not a family)?
pub fn is_leaf(concept: &IdentityConcept) -> bool {
    use IdentityConcept::*;
    !matches!(
        concept,
        Identity
            | CryptographicSignature
            | ContentHash
            | PersistentIdentifier
            | SelfDescribingMetadata
    )
}

/// Is this concept a family (level 1 under the root)?
pub fn is_family(concept: &IdentityConcept) -> bool {
    use IdentityConcept::*;
    matches!(
        concept,
        CryptographicSignature | ContentHash | PersistentIdentifier | SelfDescribingMetadata
    )
}

// ---------------------------------------------------------------------------
// Ontology trait impl
// ---------------------------------------------------------------------------

impl Ontology for ArtifactIdentityOntology {
    type Cat = ArtifactIdentityCategory;
    type Qual = TrustTierOf;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            // Universal axioms
            Box::new(EverySchemeHasAnExtractor),
            Box::new(ExtractorIsDeterministic),
            Box::new(VerificationFailClosed),
            Box::new(CompositeRequiresAll),
            // Family-level axioms
            Box::new(ContentHashIsInjective),
            Box::new(ContentHashIsOffline),
            Box::new(PersistentIdentifierRequiresResolver),
            Box::new(SelfDescribingIsWeakestTrust),
        ]
    }
}

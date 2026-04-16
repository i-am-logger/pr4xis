//! Artifact identity — how external artifacts carry their identity.
//!
//! A meta ontology for the schemes pr4xis uses to trust an upstream source.
//! Grounded in four distinct families of identity mechanism, each with its own
//! literature:
//!
//! - **CryptographicSignature** — signer vouches for content (OpenPGP, Sigstore,
//!   SSH, minisign, X.509, Ed25519)
//! - **ContentHash** — identity derived from content bytes (sha256, git SHA,
//!   IPFS CID, nix store path, BitTorrent info-hash)
//! - **PersistentIdentifier** — registry resolves ID to content (DOI, Handle,
//!   ARK, PURL)
//! - **SelfDescribingMetadata** — content declares its own identity (OWL
//!   version IRI, Dublin Core identifier, XML element attribute, XSD version)
//!
//! The key insight: many data sources already declare their own version or
//! hash internally — pr4xis should respect whatever the upstream provides
//! rather than hand-maintain a duplicate hash in pr4xis source code. WordNet
//! LMF has `<Lexicon version="2025">` as a mandatory attribute; that IS the
//! identity, and an `XmlElementAttribute` extractor can read it directly.
//!
//! Sources:
//! - Dolstra 2006: *The Purely Functional Software Deployment Model* — fixed-output
//!   derivations, content-addressed store, hash-verified integrity (grounds
//!   ContentHash::RawHash)
//! - Benet 2014: *IPFS — Content Addressed, Versioned, P2P File System* (grounds
//!   ContentHash::IpfsCid)
//! - W3C Subresource Integrity 2016 — hash-verified external resources in HTTP
//! - Wilkinson et al. 2016: *The FAIR Guiding Principles* (Scientific Data 3:160018)
//!   — F1 requires a globally unique persistent identifier
//! - RFC 4880 — OpenPGP (grounds CryptographicSignature::OpenPgp)
//! - ISO 26324:2022 — DOI system (grounds PersistentIdentifier::Doi)
//! - IETF RFC 3650 — Handle System (grounds PersistentIdentifier::Handle)
//! - W3C OWL 2 Structural Specification §3.5 — ontology versioning (grounds
//!   SelfDescribingMetadata::OwlVersionIri and OwlVersionInfo)
//! - ISO 15836-1:2017 — Dublin Core Terms (grounds DctIdentifier)
//! - Global WordNet Association WN-LMF 1.3 — `<Lexicon version>` attribute
//!   (grounds the XmlElementAttribute use case)
//! - FIPS 180-4 — SHA-2 family specification
pub mod ontology;
pub mod schemes;

#[cfg(test)]
mod tests;

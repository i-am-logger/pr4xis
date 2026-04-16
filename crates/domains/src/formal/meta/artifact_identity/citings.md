# Citings — Artifact Identity

Every published source this ontology stands on, organized by the family it grounds. This is the per-ontology bibliography; the workspace-wide bibliography of sources cited by all ontologies lives at [`docs/papers/references.md`](../../../../../../docs/papers/references.md).

## ContentHash family

### Dolstra 2006 — *The Purely Functional Software Deployment Model*

- **Citation**: Dolstra, E. (2006). *The Purely Functional Software Deployment Model*. PhD thesis, Utrecht University.
- **Local PDF**: [`papers/dolstra-2006-purely-functional-deployment.pdf`](papers/dolstra-2006-purely-functional-deployment.pdf)
- **Upstream**: [edolstra.github.io/pubs/phd-thesis.pdf](https://edolstra.github.io/pubs/phd-thesis.pdf)
- **Grounds**: `ContentHash::RawHash` and `ContentHash::NixStorePath`. Specifically: §4 on content-addressed stores (`/nix/store/{hash}-name`), §5-6 on fixed-output derivations (a build spec that declares an output hash; materialization fails if the actual hash doesn't match), §7-8 on the formal integrity and reproducibility proofs. The axiom `ContentHashIsInjective` is Dolstra's strict claim generalized.
- **Cited at**: `ontology.rs` module doc, `ContentHashIsInjective` axiom description

### Benet 2014 — *IPFS: Content Addressed, Versioned, P2P File System*

- **Citation**: Benet, J. (2014). *IPFS — Content Addressed, Versioned, P2P File System*. arXiv:1407.3561.
- **Local PDF**: [`papers/benet-2014-ipfs.pdf`](papers/benet-2014-ipfs.pdf)
- **Upstream**: [arxiv.org/abs/1407.3561](https://arxiv.org/abs/1407.3561)
- **Grounds**: `ContentHash::IpfsCid` — the IPFS content identifier scheme. Extends Dolstra's single-machine content-addressing to a distributed P2P system.
- **Cited at**: `IdentityConcept::IpfsCid` doc comment

### W3C Subresource Integrity (2016)

- **Citation**: W3C Recommendation 23 June 2016. *Subresource Integrity*.
- **Local copy**: [`papers/w3c-sri-2016.html`](papers/w3c-sri-2016.html) *(the W3C publishes this only as HTML; vendored as-is so local browsing doesn't depend on w3.org)*
- **Upstream**: [w3.org/TR/SRI](https://www.w3.org/TR/SRI/)
- **Grounds**: `ContentHash::RawHash` as a web standard. The `<script integrity="sha384-...">` mechanism is direct precedent for hash-verified external resources over HTTP, which is the pattern `applied/data_provisioning/fetch.rs` implements.
- **Cited at**: `ontology.rs` module doc, `RawHash` scheme file

### FIPS 180-4 — Secure Hash Standard

- **Citation**: NIST (2015). *Federal Information Processing Standards Publication 180-4: Secure Hash Standard (SHS)*.
- **URL**: [nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf)
- **Grounds**: The SHA-256 and SHA-512 algorithms used by `RawHash` via the `sha2` crate.
- **Cited at**: `HashAlgorithm` enum doc comment

### Cohen 2003 — BitTorrent BEP-0003

- **Citation**: Cohen, B. (2003). *The BitTorrent Protocol Specification* (BEP 3).
- **URL**: [bittorrent.org/beps/bep_0003.html](https://www.bittorrent.org/beps/bep_0003.html)
- **Grounds**: `ContentHash::BittorrentInfoHash` — the Merkle-root info-hash scheme for content identification in BitTorrent. A historical parallel to modern content-addressed systems.
- **Cited at**: `BittorrentInfoHash` doc comment (stubbed)

## CryptographicSignature family

### RFC 4880 — OpenPGP Message Format

- **Citation**: Callas, J., Donnerhacke, L., Finney, H., Shaw, D., & Thayer, R. (2007). *OpenPGP Message Format*. IETF RFC 4880.
- **URL**: [rfc-editor.org/rfc/rfc4880](https://www.rfc-editor.org/rfc/rfc4880)
- **Grounds**: `CryptographicSignature::OpenPgp`. The canonical standard for the PGP / GnuPG signature format. pr4xis's future real extractor will delegate to `gpg --verify`.
- **Cited at**: `OpenPgp` doc comment (stubbed)

### Newman et al. 2022 — Sigstore whitepaper

- **Citation**: Newman, Z. et al. (2022). *Sigstore: Software Signing for Everybody*. Chainguard / CNCF.
- **Grounds**: `CryptographicSignature::SigstoreAttestation`. Cosign + Fulcio + Rekor: keyless OIDC-backed signing with a public transparency log. The real implementation is tracked in [issue #70](https://github.com/i-am-logger/pr4xis/issues/70).
- **Cited at**: `SigstoreAttestation` doc comment

### Torres-Arias et al. 2019 — *in-toto*

- **Citation**: Torres-Arias, S., Afzali, H., Kuppusamy, T. K., Curtmola, R., & Cappos, J. (2019). *in-toto: Providing farm-to-table guarantees for bits and bytes*. 28th USENIX Security Symposium.
- **URL**: [usenix.org/conference/usenixsecurity19/presentation/torres-arias](https://www.usenix.org/conference/usenixsecurity19/presentation/torres-arias)
- **Grounds**: The attestation-metadata foundation that Sigstore builds on. Relevant to `SigstoreAttestation`.
- **Cited at**: [issue #70](https://github.com/i-am-logger/pr4xis/issues/70)

### Samuel et al. 2010 — TUF

- **Citation**: Samuel, J., Mathewson, N., Cappos, J., & Dingledine, R. (2010). *Survivable Key Compromise in Software Update Systems*. ACM CCS 2010.
- **Grounds**: The Update Framework (TUF). Grounds the multi-signature composite-identity pattern. Relevant to `CryptographicSignature` family axioms about key compromise resistance.
- **Cited at**: [issue #70](https://github.com/i-am-logger/pr4xis/issues/70)

### RFC 8032 — Ed25519

- **Citation**: Josefsson, S., & Liusvaara, I. (2017). *Edwards-Curve Digital Signature Algorithm (EdDSA)*. IETF RFC 8032.
- **URL**: [rfc-editor.org/rfc/rfc8032](https://www.rfc-editor.org/rfc/rfc8032)
- **Grounds**: `CryptographicSignature::Ed25519Raw`. The algorithm behind `Minisign` and many modern signing systems.
- **Cited at**: `Ed25519Raw`, `Minisign` doc comments

### RFC 5280 — X.509 PKI

- **Citation**: Cooper, D. et al. (2008). *Internet X.509 Public Key Infrastructure Certificate and Certificate Revocation List (CRL) Profile*. IETF RFC 5280.
- **URL**: [rfc-editor.org/rfc/rfc5280](https://www.rfc-editor.org/rfc/rfc5280)
- **Grounds**: `CryptographicSignature::X509Signature`.
- **Cited at**: `X509Signature` doc comment

## PersistentIdentifier family

### ISO 26324:2022 — Digital Object Identifier System

- **Citation**: International Organization for Standardization (2022). *ISO 26324:2022 — Information and documentation — Digital object identifier system*.
- **Grounds**: `PersistentIdentifier::Doi`. The international standard for DOI.
- **Cited at**: `Doi` doc comment, `PersistentIdentifierRequiresResolver` axiom

### IETF RFC 3650 — Handle System

- **Citation**: Sun, S., Lannom, L., & Boesch, B. (2003). *Handle System Overview*. IETF RFC 3650.
- **URL**: [rfc-editor.org/rfc/rfc3650](https://www.rfc-editor.org/rfc/rfc3650)
- **Grounds**: `PersistentIdentifier::Handle`. The more general protocol that DOI is built on top of.
- **Cited at**: `Handle` doc comment

### California Digital Library — ARK specification

- **Citation**: Kunze, J. & Rodgers, R. (2013+). *The ARK Identifier Scheme*. California Digital Library and ARK Alliance.
- **URL**: [arks.org](https://arks.org/about/ark-spec/)
- **Grounds**: `PersistentIdentifier::Ark`. The archival-institution-focused persistent identifier.
- **Cited at**: `Ark` doc comment

### OCLC — PURL specification

- **Citation**: OCLC / W3C (2017+). *Persistent URL specification*.
- **URL**: [purl.archive.org](https://purl.archive.org/)
- **Grounds**: `PersistentIdentifier::Purl`.
- **Cited at**: `Purl` doc comment

## SelfDescribingMetadata family

### W3C OWL 2 Structural Specification (2012)

- **Citation**: W3C OWL Working Group (2012). *OWL 2 Web Ontology Language Structural Specification and Functional-Style Syntax (Second Edition)*. W3C Recommendation 11 December 2012.
- **URL**: [w3.org/TR/owl2-syntax](https://www.w3.org/TR/owl2-syntax/)
- **Grounds**: `SelfDescribingMetadata::OwlVersionIri` and `OwlVersionInfo` — §3.5 defines `owl:versionIRI` (an IRI naming a specific version of an ontology) and `owl:versionInfo` (free-text version annotation). Both are embedded in the ontology file itself.
- **Cited at**: `OwlVersionIri`, `OwlVersionInfo` doc comments

### ISO 15836-1:2017 — Dublin Core Terms

- **Citation**: International Organization for Standardization (2017). *ISO 15836-1:2017 — Information and documentation — The Dublin Core metadata element set — Part 1: Core elements*.
- **Grounds**: `SelfDescribingMetadata::DctIdentifier` — `dct:identifier`. Dublin Core's generic identifier property, used in RDF/XML metadata.
- **Cited at**: `DctIdentifier` doc comment

### Global WordNet Association — WN-LMF 1.3 schema

- **Citation**: Global WordNet Association (2020+). *WordNet Lexical Markup Framework (WN-LMF) Schema, version 1.3*.
- **Local copy**: [`papers/wn-lmf-1.3-schema.html`](papers/wn-lmf-1.3-schema.html) *(published by GWA only as HTML; vendored as-is)*
- **Upstream**: [globalwordnet.github.io/schemas](https://globalwordnet.github.io/schemas/)
- **Grounds**: The `<Lexicon version="...">` attribute that `applied/data_provisioning/`'s WordNet `RegistryEntry` uses as its `XmlElementAttribute` identity claim. Specific, concrete, directly referenced by the code.
- **Cited at**: `XmlElementAttribute` doc comment, `ontology.rs` module doc

### W3C XML 1.0 Fifth Edition (2008)

- **Citation**: W3C XML Core Working Group (2008). *Extensible Markup Language (XML) 1.0 (Fifth Edition)*. W3C Recommendation 26 November 2008.
- **URL**: [w3.org/TR/xml](https://www.w3.org/TR/xml/)
- **Grounds**: Reused via the existing `crates/domains/src/social/software/markup/xml/` ontology. `XmlElementAttribute::verify()` delegates to `xml_reader::read_xml` which implements this spec.
- **Cited at**: `XmlElementAttribute` scheme file

## Broader grounding

### Wilkinson et al. 2016 — FAIR Guiding Principles

- **Citation**: Wilkinson, M. D., Dumontier, M., Aalbersberg, I. J. et al. (2016). *The FAIR Guiding Principles for scientific data management and stewardship*. Scientific Data 3, 160018.
- **Local PDF**: [`papers/wilkinson-2016-fair.pdf`](papers/wilkinson-2016-fair.pdf)
- **DOI**: [10.1038/sdata.2016.18](https://doi.org/10.1038/sdata.2016.18)
- **Grounds**: F1 — "(meta)data are assigned a globally unique and persistent identifier" — is the principle every `IdentityScheme` must satisfy. Different schemes satisfy F1 in different ways (hash, DOI, version string, etc.); the ontology treats them all as F1-compliant at the top level.
- **Cited at**: `ontology.rs` module doc

### Haerder & Reuter 1983 — ACID

- **Citation**: Haerder, T. & Reuter, A. (1983). *Principles of transaction-oriented database recovery*. ACM Computing Surveys 15(4), 287–317.
- **Grounds**: The durability/consistency framing for stored materializations. Already cited by `formal/information/storage/`; `applied/data_provisioning/` inherits the framing via composition.
- **Cited at**: indirectly via the storage ontology

## Pending verification

- [x] Dolstra 2006 — vendored at `papers/dolstra-2006-purely-functional-deployment.pdf`
- [x] Benet 2014 IPFS — vendored at `papers/benet-2014-ipfs.pdf`
- [x] Wilkinson 2016 FAIR — vendored at `papers/wilkinson-2016-fair.pdf`
- [x] W3C SRI 2016 — vendored at `papers/w3c-sri-2016.html` (HTML-only upstream)
- [x] WN-LMF 1.3 schema — vendored at `papers/wn-lmf-1.3-schema.html` (HTML-only upstream)
- [ ] Vendor the Sigstore whitepaper (tracked in #70)
- [ ] Cross-check every citation here against `docs/papers/references.md`
- [ ] Add code-level citations (`// Source: ...`) in the stub extractor files when real implementations land

---

- **Document date:** 2026-04-15
- **How this file is maintained:** per-scheme citations live in the corresponding `schemes/*.rs` file doc comments; this file consolidates them and adds the cross-cutting references. When a new scheme moves from stub to real implementation, add its citation here under the relevant family.

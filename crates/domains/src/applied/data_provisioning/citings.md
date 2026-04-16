# Citings — Data Provisioning

This ontology is a **composition** — it imports types and functors from several existing pr4xis ontologies and only adds the applied glue layer. Most of its grounding comes transitively from the ontologies it composes with. Citations below are organized by which role each source plays in the composition.

The workspace-wide bibliography lives at [`docs/papers/references.md`](../../../../../../docs/papers/references.md).

## Composition inputs — ontologies this depends on

### `formal/meta/artifact_identity/`

The identity ontology, authored in the same PR. Grounds every identity claim via its taxonomy (CryptographicSignature, ContentHash, PersistentIdentifier, SelfDescribingMetadata). See [`../../formal/meta/artifact_identity/citings.md`](../../formal/meta/artifact_identity/citings.md) for the full grounding: Dolstra 2006, Benet 2014, W3C SRI, RFC 4880, ISO 26324, W3C OWL 2, ISO 15836, WN-LMF 1.3, Wilkinson FAIR, Haerder & Reuter.

### `formal/information/storage/`

The storage ontology. Grounds the `DataCache` concept as an instance of `Store`, and fetch-verify-write as an instance of `Materialize → Realize`. See `formal/information/storage/citings.md` for RDF4J, Jena TDB, Spivak 2012 functorial data migration, OMG MDA, Gupta & Mumick 1995 materialized views, Haerder & Reuter 1983 ACID.

### `formal/information/provenance/`

The provenance ontology (W3C PROV-O). Grounds `ProvisioningEvent` as an instance of `prov:Activity` with `prov:used = DataSource.url` and `prov:generated = VerifiedDataset`. See `formal/information/provenance/citings.md` for the W3C PROV-O recommendation.

### `formal/meta/staging/`

The staging / partial-evaluation ontology. A fetch is an instance of the `freeze: Dynamic → Static` functor from Futamura 1971 — the URL is the dynamic input, the local file is the static output, staging level = 1. See `formal/meta/staging/citings.md` for Futamura 1971 and Jones/Gomard/Sestoft 1993.

### `social/software/markup/xml/`

The XML ontology. The `XmlLmf` decoder delegates to `xml_reader::read_xml`, which implements the W3C XML 1.0 Fifth Edition (2008). Grounds the decoder chain's first stage.

### `social/software/markup/xml/lmf/`

The WordNet LMF ontology. The `XmlLmf` decoder delegates to `lmf::reader::read_wordnet`, which parses the Global WordNet Association WN-LMF 1.3 schema. Grounds the decoder chain's second stage.

### `cognitive/linguistics/english/`

The English ontology. `English::from_wordnet` is the final stage of the XmlLmf decoder chain for WordNet. The data-provisioning layer feeds raw bytes to this existing functor without changing anything in it.

## Direct citations for the data_provisioning layer itself

### Wilkinson et al. 2016 — FAIR Guiding Principles

- **Citation**: Wilkinson, M. D., Dumontier, M., Aalbersberg, I. J. et al. (2016). *The FAIR Guiding Principles for scientific data management and stewardship*. Scientific Data 3, 160018.
- **DOI**: [10.1038/sdata.2016.18](https://doi.org/10.1038/sdata.2016.18)
- **Grounds**: The meta-principle that every managed dataset must satisfy F1 (persistent identifier) + A1 (accessible via standard protocol) + R1.2 (provenance recorded). `EveryDataSourceHasIdentity` encodes F1; `remote_location` + fetch protocol encodes A1; the `formal/information/provenance/` integration encodes R1.2.
- **Cited at**: `EveryDataSourceHasIdentity` axiom description

### Dolstra 2006 — Fixed-output derivations (via artifact_identity)

- **Citation**: Dolstra, E. (2006). *The Purely Functional Software Deployment Model*. PhD thesis, Utrecht University.
- **Grounds**: The model of "declare the expected output hash; fail if actual doesn't match" that `RawHash` verification implements. The data-provisioning layer is a specific instance of Dolstra's fixed-output derivation pattern, applied to data rather than compiled software.
- **Cited at**: transitively via `formal/meta/artifact_identity/citings.md`

### Global WordNet Association — WN-LMF 1.3

- **Citation**: Global WordNet Association (2020+). *WordNet Lexical Markup Framework (WN-LMF) Schema, version 1.3*.
- **URL**: [globalwordnet.github.io/schemas](https://globalwordnet.github.io/schemas/)
- **Grounds**: The specific schema the WordNet `RegistryEntry` refers to. The `XmlElementAttribute` identity claim for WordNet reads the `version` attribute on the `<Lexicon>` element defined by this schema.
- **Cited at**: `registry.rs` WordNet entry, `ontology.rs` module doc

## Pending verification

- [ ] Cross-check every transitive citation with `docs/papers/references.md`
- [x] Registry pins the real sha256 `6f49adeec1…` of the decompressed upstream asset at `github.com/globalwordnet/english-wordnet/releases/tag/2025-edition`
- [ ] Add a code-level comment in `registry.rs` pointing to this citings file
- [ ] When new content types land (Pdf, Video, etc.), add their grounding sources to the "Composition inputs" section above

---

- **Document date:** 2026-04-15
- **How this file is maintained:** this ontology is a composition; new citations mostly come from extending its composition inputs. Direct citations for the data_provisioning layer itself are kept minimal because the real theoretical content lives upstream in `artifact_identity`, `storage`, `provenance`, and `staging`.

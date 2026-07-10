//! Data provisioning — managed external data sources, cache, and lifecycle states.
//!
//! Composes `formal/meta/artifact_identity/` (identity claims),
//! `formal/meta/source_taxonomy/` (the typed kinds of corpus this layer can
//! ingest), `formal/information/storage/` (cache semantics), and
//! `formal/information/provenance/` (fetch events).
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
use crate::formal::meta::source_taxonomy::ontology::{SourceTaxonomyConcept, concept_name};
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
            "A typed transformation from raw bytes to a SourceTaxonomy-typed ontology instance. One decoder per `SourceTaxonomyConcept` leaf via `canonical_encoding`."),
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
// ContentType — encoding format (internal; derived from SourceTaxonomyConcept)
// ---------------------------------------------------------------------------

/// The on-the-wire encoding praxis must decode for a given source. Distinct
/// from the source's *kind* in [`SourceTaxonomyConcept`]: kind is semantic
/// (Language, Statute, Regulation, …); ContentType is the byte-level format
/// (XML, plaintext, JSON, …). The mapping is canonical and derived by
/// [`canonical_encoding`] — `praxis.toml` does not declare encoding.
///
/// Most kinds have a single canonical encoding: legal corpora ship as
/// plaintext from .gov sources; lexicons ship as LMF XML per ISO 24613.
/// A future kind that genuinely needs alternative encodings (e.g. a
/// statute also available as PDF) would call for an optional `format`
/// override on the registry entry; we defer that until a real source
/// demands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    /// WordNet LMF XML (ISO 24613). Decoder: `xml_reader::read_xml → lmf::reader::read_wordnet`.
    XmlLmf,
    /// PDF, ISO 32000-2. Decoder: not yet implemented.
    Pdf,
    /// United States Legislative Markup XML, per the LRC's
    /// published USLM-1.0.15.xsd schema. Decoder:
    /// `xml::uslm::lens::read_uslm_title` (runtime) /
    /// `pr4xis::codegen::uslm::parse_uslm_xml` (build-time).
    /// Cited by 1 U.S.C. § 204 as the U.S. Code's authoritative
    /// publication. URL: uscode.house.gov/uslm/.
    UslmXml,
    /// W3C XML Schema Definition (XSD) 1.1, per W3C Recommendation
    /// 5 April 2012 (Gao, Sperberg-McQueen & Thompson 2012, Part 1:
    /// Structures; Peterson et al. 2012, Part 2: Datatypes). Decoder:
    /// `pr4xis::codegen::xsd::parse_xsd` — emits a typed AST of
    /// `xs:element`, `xs:complexType`, `xs:simpleType`, content models,
    /// attributes, substitution groups. The XSD authoritatively grounds
    /// content-type ontologies that load from it (e.g., USLM).
    XmlXsd,
    /// XHTML 1.0 text — the W3C publication format for text-form
    /// conceptual specifications (e.g., the XML Information Set
    /// recommendation per Cowan & Tobin 2004). The W3C-published
    /// XHTML edition is XHTML 1.0 Transitional with regular section-
    /// heading markup (`<h2>`, `<h3>` with class attributes); the
    /// build-time loader extracts the conceptual taxonomy by walking
    /// that markup.
    Xhtml,
    /// Plain text, UTF-8. Decoder: direct.
    Plaintext,
    /// Adobe Glyph List (`name;HEX[ HEX]*` lines) per Adobe's
    /// 2002–2019 published table; cited by ISO 32000-2:2020
    /// §9.6.5.4 + Adobe Tech Note #5014 as the canonical
    /// resolver for PDF `/Differences` glyph names. Decoder:
    /// `decoders::adobe_glyph_list`.
    AdobeGlyphList,
    /// JSON document, RFC 8259. Decoder: serde_json parse.
    Json,
    /// Video file (mp4, webm). Decoder: not yet implemented.
    Video,
    /// Audio file (wav, flac, ogg). Decoder: not yet implemented.
    Audio,
    /// gzip-compressed tar archive — the canonical bundle format for
    /// conformance test suites that ship as a single downloadable file
    /// (W3C QA Framework: Test Methodology Guidelines, Curran et al.
    /// 2008). The W3C XML Schema Test Suite (xsts-2007-06-20.tar.gz)
    /// is the bundled instance. Decoder: tar/gzip extraction +
    /// per-case `xml::parser::parse_document`.
    TarGzArchive,
    /// XML 1.0 Document Type Definition (DTD), per W3C XML 1.0 Fifth
    /// Edition §2.8 + §4 (Bray, Paoli, Sperberg-McQueen, Maler &
    /// Yergeau 2008). The pre-XSD machine-readable grammar form for
    /// XML applications. Global WordNet ships its WN-LMF schema as a
    /// DTD; this content type recognises it as a schema source.
    /// Decoder: magic-prefix identification (`<!ELEMENT` /
    /// `<!ATTLIST` markup declarations per §3.2 + §3.3).
    XmlDtd,
    /// PKZIP archive (.zip), per APPNOTE.TXT 6.3.10 (PKWARE Inc.,
    /// 2022) + ISO/IEC 21320-1:2015 (the OOXML / EPUB / OPC
    /// subset). The canonical container format for OOXML schemas
    /// (ECMA-376 5th edition ships its 21 XSDs as an
    /// `OfficeOpenXML-XMLSchema-Strict.zip` bundle) and for OOXML
    /// documents themselves (XLSX / DOCX / PPTX = ZIP-of-XML).
    /// Decoder: magic-prefix identification (PKZIP local-file-header
    /// signature `0x04034b50` = bytes `50 4B 03 04`).
    ZipArchive,
    /// W3C OWL 2 / RDF-XML vocabulary, per the OWL 2 Web Ontology
    /// Language Structural Specification (Motik, Patel-Schneider &
    /// Parsia eds., W3C Recommendation 11 December 2012) serialised as
    /// RDF/XML (Gandon & Schreiber eds., RDF 1.1 XML Syntax, W3C
    /// Recommendation 25 February 2014). The bundled SPAR CiTO
    /// vocabulary ships in this form. Decoder:
    /// `xml owl::reader::read_owl`.
    Owl,
    /// WN-LMF-shaped *closed-class lexicon* — a small, bounded LMF lexicon
    /// (the English function-word inventory; the U.S. legal closed-class
    /// lexicon) authored in the same `<LexicalEntry>`/`<Lemma>`/`<Synset>`
    /// schema as WordNet, but distinguished from the open-class `XmlLmf`
    /// (WordNet) because its runtime loader materializes the source bytes
    /// through the generalized feature-light RAW-bytes `.prx` path
    /// (`raw_source_prx`) — so the lexicon loads in the default `std`-only
    /// build with NO `prx`/gzip feature, where the graph `.prx.gz` envelope is
    /// unavailable. The reader over those raw bytes is still
    /// `lmf::reader::read_wordnet`; the byte-exact graph-faithful round-trip is
    /// still witnessed by the registered `WordNetLmfLens` audit. Decoder:
    /// `decoders::xml_lmf`.
    XmlLmfLexicon,
    /// A Base16/Base24 color-scheme COLLECTION — a directory of named
    /// color-scheme YAML files (each binding the framework's reserved
    /// slot keys `base00`..`base0F` / `base00`..`base17` to sRGB hex),
    /// archived into ONE deterministic content-addressed blob (the
    /// Tinted Theming `tinted-schemes` dataset). Unlike every other
    /// raw-source content type — each of which is a single file — this
    /// is a MANY-file collection, so its `.prx` carries a directory
    /// archive (`decoders::theme_collection`'s deterministic
    /// `path → bytes` manifest), not one file's bytes. Decoder:
    /// `decoders::theme_collection`. Cite: Tinted Theming, Base16
    /// Styling Guidelines + Base24 spec, github.com/tinted-theming/home.
    ThemeCollection,
    /// LMF-shaped mathematical-operator vocabulary — an authored closed-class
    /// controlled vocabulary serialised in the WN-LMF `<LexicalEntry>` shape
    /// (`math-operators.xml`: one operator glyph per `<Lemma>`, its OpenMath STS
    /// signature on the `<Sense>`). Unlike `XmlLmf` (WordNet / function-words /
    /// legal-lexicon), its runtime value is an `OperatorVocabulary`, NOT a
    /// WordNet graph — so it loads through the generalized RAW-bytes `.prx` path
    /// (`raw_source_prx`), not the graph-faithful `WordNetPrxEnvelope`. The reader
    /// is still `lmf::reader::read_wordnet` over the materialized raw bytes.
    /// Cite: OpenMath Standard 2.0 (Kohlhase & Rabe 2019) §2.1.4 + §4.3;
    /// ISO 80000-2:2019 (operator glyphs).
    MathOperatorLmf,
    /// Raw bytes with no further decoding.
    Binary,
}

/// Canonical encoding for a [`SourceTaxonomyConcept`] leaf.
///
/// Lexicons ship per ISO 24613 (LMF XML). Legal text corpora ship as
/// UTF-8 plaintext (govinfo.gov, ecfr.gov, court opinion repositories all
/// expose plaintext as their primary format). LegalLexicons ship as
/// LMF XML — same Global WordNet LMF 1.3 schema as the Language family
/// — so they reuse the existing `decoders::xml_lmf` decoder.
///
/// The decision to use LMF for LegalLexicons (rather than JSON or a
/// bespoke schema) follows the praxis "one substrate" principle: the
/// loader/reader infrastructure for closed-class lexica already exists
/// for WordNet and function-word taxonomies, and the same LMF schema
/// natively supports the `<LexicalEntry>`/`<Lemma>`/`<Synset>` shape
/// that closed-class bounded enumerations need.
pub fn canonical_encoding(kind: SourceTaxonomyConcept) -> ContentType {
    use SourceTaxonomyConcept as C;
    match kind {
        // The open-class WordNet (`Language`) ships as WN-LMF XML and loads
        // through the graph-faithful `WordNetPrxEnvelope` (`.prx.gz`) under the
        // `prx` feature — its corpus is far too large to materialize in the
        // default build.
        C::Language => ContentType::XmlLmf,
        // The closed-class function-word stratum (`ClosedClassLexicon`) is the
        // complementary half of the lexicon (Quirk et al. 1985 §2.34), authored
        // in the SAME WN-LMF `<LexicalEntry>`/`<Lemma>`/`<Synset>` schema — but
        // it is small and bounded, and its runtime loader runs in the default
        // `std`-only build (no `prx`/gzip), so it materializes its source bytes
        // through the generalized feature-light RAW-bytes `.prx` path
        // (`XmlLmfLexicon`), not the graph `.prx.gz` envelope. The reader is
        // unchanged (`read_wordnet`); the byte-exact graph round-trip is still
        // witnessed by the registered `WordNetLmfLens` audit.
        C::ClosedClassLexicon => ContentType::XmlLmfLexicon,
        // AGID (Atkinson 2016) ships as a plain-text inflection database
        // (`<word> <pos>: <forms>` lines); CatVar (Habash & Dorr 2003) ships as
        // plain-text derivational clusters (`word_POS%weight#…`). Neither is XML.
        C::InflectionLexicon | C::DerivationalLexicon => ContentType::Plaintext,
        // ControlledVocabulary leaves ship as plain-text tab-separated tables
        // (the praxis-TSV convention): the EWMH window-state atom vocabulary
        // (`bit_name<TAB>spec_atom<TAB>source`) and the OLiA→CCG lexical-category
        // projection (`olia_class<TAB>ccg_category[<TAB>valency]`). Decoder:
        // `decoders::plaintext_tsv` (the generic TSV → typed-record-stream codec).
        C::WindowStateVocabulary | C::LexicalCategoryProjection => ContentType::Plaintext,
        // The mathematical-operator vocabulary is a ControlledVocabulary leaf
        // authored in LMF-shaped XML (one operator glyph per `<LexicalEntry>`,
        // its OpenMath STS signature on the `<Sense>`). It is consumed as raw
        // bytes (parsed to an `OperatorVocabulary`, NOT a WordNet graph), so it
        // rides the generalized RAW-bytes `.prx` path under its own content type.
        C::MathOperatorVocabulary => ContentType::MathOperatorLmf,
        // The Base16/Base24 color-scheme vocabulary is a COLLECTION leaf — a
        // whole directory of named-scheme YAML files (the Tinted Theming
        // `tinted-schemes` dataset). Its canonical on-disk form + `.prx` is a
        // single deterministic directory archive (one blob, many files), so it
        // rides the generalized RAW-bytes `.prx` path under its own content
        // type, decoded by `decoders::theme_collection` back into the
        // `path → bytes` set the theming validator scans.
        C::ColorSchemeVocabulary => ContentType::ThemeCollection,
        // US federal statutes published by GPO ship as PDF on
        // govinfo.gov (ISO 32000-2:2020 PDF 2.0; Bluebook §18
        // preferred authenticated digital edition). The Statute /
        // UsFederalStatute leaves of the taxonomy retain this
        // canonical encoding for completeness; the registered
        // loadable form for US Code sections is the UsCodeTitle
        // USLM XML below, not per-section PDFs.
        C::Statute | C::UsFederalStatute => ContentType::Pdf,
        // Whole U.S. Code titles ship as USLM XML from
        // uscode.house.gov per 1 U.S.C. § 204. Each title is
        // an LRC publication unit containing many UsFederalStatute
        // sections; the title-level XML is sliced at build time.
        C::UsCodeTitle => ContentType::UslmXml,
        // Other legal corpora (regulations, constitutional
        // articles, procedural rules, case law) also commonly
        // ship as PDF on the authoritative source; same rule.
        C::Regulation | C::ConstitutionalArticle | C::ProceduralRule | C::CaseLaw => {
            ContentType::Pdf
        }
        // LegalLexicons ship as LMF XML — same `<LexicalEntry>` shape as the
        // function-word lexicon, reusing the same `decoders::xml_lmf` reader.
        // The bundled `us_legal_lexicon@2026` instance (a small bounded
        // closed-class lexicon consumed as a `BTreeSet` of lemma forms) is the
        // canonical example. Like `ClosedClassLexicon`, its loader runs in the
        // default `std`-only build, so it materializes its source bytes through
        // the generalized feature-light RAW-bytes `.prx` path (`XmlLmfLexicon`),
        // not the graph `.prx.gz` envelope; the byte-exact graph round-trip is
        // still witnessed by the registered `WordNetLmfLens` audit.
        C::LegalLexicon => ContentType::XmlLmfLexicon,
        // SchemaVocabularies ship as LMF XML — same shape as
        // LegalLexicons. The taxonomy slot remains for future
        // bundles in this family; the M4.η.4 deletion of
        // `schema_vocabulary@2026` left no registered instance,
        // because every schema-vocabulary recognition path now
        // consults the loaded XSD source directly (HTML5 XSD, W3C
        // xml.xsd, USLM-1.0.18 self-annotations) rather than a
        // separate LMF bundle.
        C::SchemaVocabulary => ContentType::XmlLmf,
        // Adobe Glyph List has its own typed decoder; the
        // `decoders::adobe_glyph_list` parser handles the
        // `name;HEX[ HEX]*` line format per Adobe's published
        // 2002–2019 table.
        C::TypographicGlyphSet => ContentType::AdobeGlyphList,
        // XSD documents are XML themselves but with the structural
        // semantics of W3C XSD 1.1 (Gao, Sperberg-McQueen & Thompson
        // 2012 Part 1; Peterson et al. 2012 Part 2). Decoder:
        // `pr4xis::codegen::xsd::parse_xsd`.
        C::XmlSchemaDefinition => ContentType::XmlXsd,
        // XML 1.0 DTDs ship as plain-text markup-declarations per
        // W3C XML 1.0 §2.8 + §4 (Bray et al. 2008). Used as the
        // canonical schema form for vocabularies that predate XSD
        // or chose DTD over XSD (e.g. Global WordNet's WN-LMF).
        C::XmlDocumentTypeDefinition => ContentType::XmlDtd,
        // OOXML schema bundles ship as PKZIP archives per ECMA-376
        // 5th edition (December 2016) — the `OfficeOpenXML-XMLSchema-
        // Strict.zip` bundle inside the ECMA standard's outer ZIP.
        // Decoder: zip-archive magic-prefix identification +
        // per-XSD consumer extraction.
        C::OoxmlSchemaArchive => ContentType::ZipArchive,
        // ConceptualSpec sources ship as XHTML (the W3C-published
        // recommendation format for text-form specifications such as
        // the XML Information Set rec, Cowan & Tobin 2004). Decoder:
        // build-time text-scan of the section-heading structure
        // (`<h3>` markup with regular class attribution).
        C::ConceptualSpec => ContentType::Xhtml,
        // XmlSchemaTestSuite ships as a gzip-compressed tar archive of
        // ~14k schema files plus testSet metadata XML, per the W3C QA
        // Framework: Test Methodology Guidelines (Curran et al. 2008).
        // Decoder: tar/gzip extract + per-case parse_document +
        // project_from_xml_document.
        C::XmlSchemaTestSuite => ContentType::TarGzArchive,
        // XmlConformanceTestSuite (XMLConf) ships as a gzip-compressed
        // tar archive of ~3k XML test files plus per-contributor
        // manifest XMLs (xmlconf.xml + sun/ / ibm/ / xmltest/ /
        // japanese/ / oasis/ / eduni/ sub-manifests). Decoder:
        // tar/gzip extract + per-case parse_document.
        C::XmlConformanceTestSuite => ContentType::TarGzArchive,
        // OntologyVocabulary sources ship as W3C OWL 2 / RDF-XML
        // (Motik, Patel-Schneider & Parsia 2012; Gandon & Schreiber
        // 2014). The bundled SPAR CiTO vocabulary is the canonical
        // instance. Decoder: `social::software::markup::xml::owl::
        // reader::read_owl`.
        C::OntologyVocabulary => ContentType::Owl,
        // Non-leaf concepts have no decoder — they're abstract.
        C::Source
        | C::Lexicon
        | C::DomainLexicon
        | C::LegalCorpus
        | C::TypographyResource
        | C::SchemaSpec
        | C::TestSuite
        | C::ControlledVocabulary => ContentType::Binary,
    }
}

// ---------------------------------------------------------------------------
// RegistryEntry — the concrete managed datasets
// ---------------------------------------------------------------------------

/// One row in the data-provisioning registry. The registry is the ontology's
/// instance layer; each entry is a typed value declaring a `DataSource`'s
/// metadata, semantic kind, and identity claims.
///
/// Entries are loaded from the workspace-root `praxis.toml` at runtime via
/// `OnceLock` in [`super::registry`]. The semantic kind is a typed
/// [`SourceTaxonomyConcept`] (not a string); the parser maps the TOML
/// `type = "Statute"` string to its variant at load time and fails closed
/// on unknown names.
///
/// The identity claims are synthesized at load time from the manifest's
/// declared `version` (used as the expected value for the XML-LMF version
/// attribute on Lexicon kinds) and from `praxis.lock`'s pinned digest for
/// `name@version`. Drift between manifest and lock is caught by the
/// `LockManifestAgreement` axiom.
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    /// Primary key: matches `[sources.<name>]` in praxis.toml.
    pub name: String,
    /// Free-form publication identifier (calendar year, amendment cycle,
    /// edition, etc.). Not semver.
    pub version: String,
    /// Semantic kind — a leaf concept in [`SourceTaxonomyConcept`].
    pub kind: SourceTaxonomyConcept,
    /// Fetch URL.
    pub url: String,
    /// Optional human description (carried through from `praxis.toml`).
    pub description: Option<String>,
    /// Optional explicit workspace-relative disk path (from the
    /// `local_path` field of `[sources.<name>]`), for sources whose
    /// on-disk filename is the publisher's canonical name rather than
    /// the `{name}-{version}` formula. This is registry DATA — the
    /// loaded-not-encoded replacement for the legacy `local_path_override`
    /// Rust table (audit 2026-06-12 D-8). When present it wins Layer 0
    /// in [`RegistryEntry::local_path`].
    pub local_path: Option<String>,
    /// Identity claims, synthesized from manifest + lock at load time.
    pub identity: CompositeIdentity,
}

impl RegistryEntry {
    /// `true` if the URL serves gzip-compressed bytes (i.e. ends with `.gz`).
    /// Implied from the URL; not a separate manifest field. A pure URL-shape
    /// predicate — does NOT imply the fetcher will decompress the body. The
    /// "decompress on fetch" decision is [`Self::transport_gzip`].
    pub fn gzipped(&self) -> bool {
        self.url.ends_with(".gz")
    }

    /// `true` when the gzip wrapper on the wire is purely a transport
    /// concern and the on-disk canonical form is the decompressed bytes.
    /// Computed as: URL ends with `.gz` AND the local path does NOT end
    /// with `.gz`. The fetcher decompresses iff this returns true.
    ///
    /// - `english-wordnet-2025.xml.gz` → `english-wordnet-2025.xml` ⇒
    ///   transport gzip, fetcher decompresses.
    /// - `xmlconf_xml_test_suite-2008-08-27.tar.gz` →
    ///   `xmlconf_xml_test_suite-2008-08-27.tar.gz` ⇒ wrapper preserved
    ///   on disk (the consumer is a tar.gz reader), fetcher writes raw.
    ///
    /// This distinction is the praxis-way fix for the previous bug where
    /// the fetcher gunzipped every `.gz` URL — yielding tar bytes for
    /// `.tar.gz` sources, which then mismatched the lock (the lock had
    /// the raw response SHA, not the gunzipped SHA).
    pub fn transport_gzip(&self) -> bool {
        self.url.ends_with(".gz") && !self.local_path().ends_with(".gz")
    }

    /// `true` if the URL serves a PKZIP archive (i.e. ends with `.zip`).
    /// Implied from the URL; not a separate manifest field. The fetch
    /// path extracts the single inner document (e.g. the `usc<NN>.xml`
    /// inside a USC release-point title archive) before verification.
    pub fn zipped(&self) -> bool {
        self.url.ends_with(".zip")
    }

    /// The encoding praxis decodes this source as, derived from `kind`.
    pub fn content_type(&self) -> ContentType {
        canonical_encoding(self.kind)
    }

    /// Workspace-relative local path where this entry materializes.
    ///
    /// Resolution proceeds in three layers:
    ///
    /// 1. **Per-source override** (`local_path_override`, private) — for sources
    ///    whose disk filename is the publisher's canonical name rather
    ///    than the `{name}-{version}` convention (e.g. W3C's
    ///    `xhtml-1.0-strict.xsd`, LC's `uslm-1.0.18.xsd`).
    /// 2. **Schema-spec formula** — `<family>/<name>-<version>.<ext>`
    ///    (no intermediate `<name>/` subdir; ext is the published
    ///    extension `.xsd` / `.dtd` / `.zip` / `.xhtml`). Applies to
    ///    [`SourceTaxonomyConcept::XmlSchemaDefinition`],
    ///    [`XmlDocumentTypeDefinition`](SourceTaxonomyConcept::XmlDocumentTypeDefinition),
    ///    [`OoxmlSchemaArchive`](SourceTaxonomyConcept::OoxmlSchemaArchive),
    ///    [`ConceptualSpec`](SourceTaxonomyConcept::ConceptualSpec).
    /// 3. **Default formula** — `<family>/<name>/<name>-<version>.<ext>`
    ///    (intermediate `<name>/` subdir). Applies to legal corpora
    ///    (USC titles, procedural rules, case-law packages) where the
    ///    per-source subdir keeps multi-file granules grouped.
    ///
    /// The `RegistryLocalPathsExist` axiom verifies the result
    /// always points to a file that actually exists in `crates/domains/data/`.
    pub fn local_path(&self) -> String {
        // Layer 0 — explicit `local_path` from praxis.toml (registry DATA): the
        // loaded-not-encoded path for every source whose on-disk name is the
        // publisher's canonical filename or whose corpus subdir is keyed on the
        // source NAME (which a kind-derived formula cannot recover). This
        // replaced both the Rust `local_path_override` table and the per-NAME
        // arms of `family_dir_for` (audit 2026-06-12 D-8 + D-9).
        if let Some(rel) = &self.local_path {
            return format!("crates/domains/data/{rel}");
        }

        let family = family_dir(self.kind);
        let ext = path_extension(self.content_type());

        // Layer 2 — schema-spec / test-suite formula (no intermediate
        // subdir, published ext). These all live as a single archive
        // or document under their family dir, with the per-name subdir
        // only emerging when the archive is extracted at test time.
        use SourceTaxonomyConcept as C;
        let is_schema_spec = matches!(
            self.kind,
            C::SchemaSpec
                | C::XmlSchemaDefinition
                | C::XmlDocumentTypeDefinition
                | C::OoxmlSchemaArchive
                | C::ConceptualSpec
                | C::OntologyVocabulary
                | C::TestSuite
                | C::XmlSchemaTestSuite
                | C::XmlConformanceTestSuite
        );
        if is_schema_spec {
            return format!(
                "crates/domains/data/{family}/{name}-{version}.{ext}",
                family = family,
                name = self.name,
                version = self.version,
                ext = ext
            );
        }

        // Layer 3 — default formula with intermediate {name}/ subdir.
        format!(
            "crates/domains/data/{family}/{name}/{name}-{version}.{ext}",
            family = family,
            name = self.name,
            version = self.version,
            ext = ext
        )
    }
}

/// Filename extension for a given decoded content type.
///
/// XmlXsd / XmlLmf / UslmXml all *encode as* XML 1.0 but use distinct
/// published file extensions: `.xsd` for W3C XML Schema Definition
/// documents, `.xml` for LMF lexica and USLM statute markup. Keeping
/// them separated lets `local_path()` round-trip to disk reality
/// without per-source overrides for the common case.
fn path_extension(ct: ContentType) -> &'static str {
    match ct {
        ContentType::XmlLmf
        | ContentType::XmlLmfLexicon
        | ContentType::UslmXml
        | ContentType::MathOperatorLmf => "xml",
        ContentType::XmlXsd => "xsd",
        ContentType::Xhtml => "xhtml",
        ContentType::Plaintext | ContentType::AdobeGlyphList => "txt",
        ContentType::Json => "json",
        ContentType::Pdf => "pdf",
        ContentType::Video | ContentType::Audio | ContentType::Binary => "bin",
        ContentType::TarGzArchive => "tar.gz",
        ContentType::XmlDtd => "dtd",
        ContentType::ZipArchive => "zip",
        ContentType::Owl => "owl",
        // The fetched raw color-scheme COLLECTION is a single deterministic
        // directory archive on disk (`<name>-<version>.themes`) — one blob,
        // many YAML schemes — so a single FILE extension keeps `local_path()`
        // pointing at a real file (the `RegistryLocalPathsExist` invariant) and
        // the generalized raw-source emit reads it as one source.
        ContentType::ThemeCollection => "themes",
    }
}

/// The on-disk family directory for a given source kind — a TOTAL function of
/// the typed [`SourceTaxonomyConcept`], the legitimate kind→family floor.
///
/// The per-SOURCE-NAME dispatch this function used to carry (USLM XSD vs XHTML
/// XSD vs MODS … all `XmlSchemaDefinition` but in different corpus subdirs) was
/// registry INSTANCE data baked into Rust; it now lives as `local_path` in
/// `praxis.toml`, consumed at Layer 0 of [`RegistryEntry::local_path`] (audit
/// 2026-06-12 D-9). Mirrors the praxis-domains code-path convention so the data
/// layout matches the ontology layout.
pub fn family_dir(kind: SourceTaxonomyConcept) -> &'static str {
    use crate::formal::meta::source_taxonomy::ontology::is_legal_corpus;
    use SourceTaxonomyConcept as C;
    if is_legal_corpus(kind) {
        match kind {
            C::Statute => "legal/statutes",
            C::UsFederalStatute => "legal/statutes/us_federal",
            C::UsCodeTitle => "legal/uscode",
            C::Regulation => "legal/regulations",
            C::ConstitutionalArticle => "legal/constitution",
            C::ProceduralRule => "legal/procedure",
            C::CaseLaw => "legal/case_law",
            _ => "legal",
        }
    } else if matches!(kind, C::TypographyResource | C::TypographicGlyphSet) {
        match kind {
            C::TypographicGlyphSet => "adobe",
            _ => "typography",
        }
    } else if matches!(kind, C::OntologyVocabulary) {
        // OWL vocabularies (the SPAR family — CiTO, DoCO, C4O, BiRO, PROV-O —
        // plus OLiA) live flat under `ontologies/<name>-<version>.owl`, read by
        // `social::software::markup::xml::owl::reader::read_owl`.
        "ontologies"
    } else if matches!(
        kind,
        C::ControlledVocabulary
            | C::WindowStateVocabulary
            | C::LexicalCategoryProjection
            | C::MathOperatorVocabulary
            | C::ColorSchemeVocabulary
    ) {
        // ControlledVocabulary leaves are DERIVED/authored or FETCHED sources
        // that live beside the code that consumes them (the EWMH window-state
        // table under `hmi/`, the OLiA→CCG projection under `grammar/`, the
        // math-operator vocabulary under `operators/`, the Base16/Base24
        // color-scheme collection under `themes/`). Each carries an explicit
        // `local_path` in praxis.toml (Layer 0 of `local_path()`), so this floor
        // is the family root, not the resolved path.
        match kind {
            C::WindowStateVocabulary => "hmi",
            C::LexicalCategoryProjection => "grammar",
            C::MathOperatorVocabulary => "operators",
            C::ColorSchemeVocabulary => "themes",
            _ => "vocabularies",
        }
    } else if matches!(
        kind,
        C::SchemaSpec
            | C::XmlSchemaDefinition
            | C::XmlDocumentTypeDefinition
            | C::ConceptualSpec
            | C::OoxmlSchemaArchive
            | C::TestSuite
            | C::XmlSchemaTestSuite
            | C::XmlConformanceTestSuite
    ) {
        // Every schema / spec / conformance-suite shares the markup-schemas
        // corpus root; the per-source subdir + filename is `local_path` data in
        // praxis.toml, no longer keyed on the source name here.
        "markup-schemas"
    } else {
        match kind {
            C::Language => "lexicons/languages",
            C::LegalLexicon => "lexicons/legal",
            C::DomainLexicon => "lexicons/domains",
            _ => "lexicons",
        }
    }
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
        axioms.push(Box::new(RegistryUniquenessByNameVersion));
        axioms.push(Box::new(DecoderTotalityPerKind));
        axioms.push(Box::new(IdentityClaimsUseLeaves));
        axioms.push(Box::new(KindIsTaxonomyLeaf));
        axioms.push(Box::new(LockManifestAgreement));
        axioms.push(Box::new(RegistryLocalPathsExist));
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
        let ok = crate::applied::data_provisioning::registry::data_sources()
            .iter()
            .all(|entry| !entry.identity.0.is_empty());
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

/// Axiom: `(name, version)` is the primary key — no two entries share both.
///
/// Dolstra (2006) §5.1 — every derivation is uniquely identified by its
/// store path; for the user-facing registry layer, the human-readable
/// `(name, version)` pair plays the same role.
pub struct RegistryUniquenessByNameVersion;

impl Axiom for RegistryUniquenessByNameVersion {
    fn verify(&self) -> Verdict {
        let mut seen: HashSet<(&str, &str)> = HashSet::new();
        for entry in crate::applied::data_provisioning::registry::data_sources() {
            if !seen.insert((entry.name.as_str(), entry.version.as_str())) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "RegistryUniquenessByNameVersion",
        "every RegistryEntry has a unique (name, version) pair",
        "Dolstra (2006) The Purely Functional Software Deployment Model §5.1"
    );
}

pr4xis::register_axiom!(
    RegistryUniquenessByNameVersion,
    "Dolstra (2006) The Purely Functional Software Deployment Model §5.1"
);

/// Axiom: every registered source is *realizable* through a runtime
/// decoder for its `canonical_encoding`. An entry without a decoder
/// would be unreachable; the axiom catches that at startup.
pub struct DecoderTotalityPerKind;

impl Axiom for DecoderTotalityPerKind {
    fn verify(&self) -> Verdict {
        for entry in crate::applied::data_provisioning::registry::data_sources() {
            // Sources whose RawHash identity claim is a Stub are
            // registered in praxis.toml but not yet loadable. They
            // are not expected to be materializable through the
            // runtime decoder until the loader fills in their hash;
            // the decoder-totality check re-activates automatically
            // when that happens.
            if entry.identity.is_stub_only() {
                continue;
            }
            let ct = canonical_encoding(entry.kind);
            let runtime_decoder = crate::applied::data_provisioning::decoders::has_decoder_for(ct);
            if !runtime_decoder {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DecoderTotalityPerKind",
        "every registered source has a runtime decoder for its canonical_encoding",
        "Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3 — R1 reusable"
    );
}

pr4xis::register_axiom!(
    DecoderTotalityPerKind,
    "Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3 — R1 reusable"
);

/// Axiom: every resolved identity claim uses a LEAF `IdentityConcept` — not
/// a family or the root. A claim with an abstract family concept would be
/// ill-formed because families do not specify a verification scheme.
pub struct IdentityClaimsUseLeaves;

impl Axiom for IdentityClaimsUseLeaves {
    fn verify(&self) -> Verdict {
        use crate::formal::meta::artifact_identity::ontology::is_leaf;
        for entry in crate::applied::data_provisioning::registry::data_sources() {
            for claim in &entry.identity.0 {
                if !is_leaf(&claim.concept) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
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

/// Axiom: every registered `kind` is a *leaf* in the SourceTaxonomy
/// (not Source, Lexicon, DomainLexicon, or LegalCorpus). Abstract
/// (family) kinds have no canonical encoding and would break decoder
/// dispatch.
///
/// Mirrors the `IdentityClaimsUseLeaves` invariant at the taxonomy level.
pub struct KindIsTaxonomyLeaf;

impl Axiom for KindIsTaxonomyLeaf {
    fn verify(&self) -> Verdict {
        use crate::formal::meta::source_taxonomy::ontology::is_leaf;
        for entry in crate::applied::data_provisioning::registry::data_sources() {
            if !is_leaf(entry.kind) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "KindIsTaxonomyLeaf",
        "every RegistryEntry's kind is a leaf in SourceTaxonomy",
        "Hart (1961) The Concept of Law — only concrete kinds can be instantiated"
    );
}

pr4xis::register_axiom!(KindIsTaxonomyLeaf, "Hart (1961) The Concept of Law");

/// Axiom: `praxis.lock` and `praxis.toml` agree.
///
/// Every manifest entry must have a matching lock hash; every lock hash
/// must match a manifest entry; every lock hash must match what the
/// manifest's identity claim records. Three failure modes:
///   - manifest entry with no lock hash → "regenerate praxis.lock"
///   - lock hash with no manifest entry → straggler from a removed source
///   - manifest+lock disagree on the hash → source drift
///
/// Dolstra (2006) §5.1 — content-addressing: a derivation's identity is
/// the hash of its inputs. The lock pins inputs; the manifest declares
/// expected inputs; agreement is the invariant.
pub struct LockManifestAgreement;

impl Axiom for LockManifestAgreement {
    fn verify(&self) -> Verdict {
        use crate::formal::meta::artifact_identity::ontology::ClaimData;
        let entries = crate::applied::data_provisioning::registry::data_sources();
        let lock = crate::applied::data_provisioning::registry::lock_hashes();

        // Every manifest entry's RawHash claim must equal the lock hash
        // for its (name, version). Identity is synthesized from the lock
        // at load time, so this checks the round-trip integrity.
        //
        // Exception: an entry whose RawHash claim is a Stub means it's
        // registered in praxis.toml but not yet loadable (no lock hash;
        // awaiting PDF loader / NLP extraction infrastructure). Stub-
        // only entries are skipped — they cannot drift because they
        // have nothing pinned. Drift detection re-activates when the
        // loader fills in the hash.
        for entry in entries {
            let key = format!("{}@{}", entry.name, entry.version);
            let lock_digest = lock.get(&key);
            let raw_hash_claim = entry
                .identity
                .0
                .iter()
                .find(|c| matches!(c.concept, IdentityConcept::RawHash));
            // The manifest claim is synthesized FROM the lock digest at load
            // time (`build_entry` -> `LockDigest::claim_data`), so agreement
            // is the TYPED claim equality: same algorithm, same digest.
            let manifest_claim = raw_hash_claim.and_then(|c| match &c.data {
                d @ ClaimData::HashAlgorithm { .. } => Some(d),
                _ => None,
            });
            let is_stub = raw_hash_claim
                .map(|c| matches!(c.data, ClaimData::Stub { .. }))
                .unwrap_or(false);
            match (is_stub, manifest_claim, lock_digest) {
                // Stub claim (loadable-pending) and no lock entry: OK.
                (true, _, None) => {}
                // Stub claim but lock has an entry: drift (manifest
                // forgot to pin even though the lock has data).
                (true, _, Some(_)) => {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
                // Typed claim on both sides, agree: OK.
                (false, Some(m), Some(l)) if *m == l.claim_data() => {}
                // Any other state is drift.
                _ => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
            }
        }

        // Lock entries with no matching manifest key are stragglers.
        let manifest_keys: HashSet<String> = entries
            .iter()
            .map(|e| format!("{}@{}", e.name, e.version))
            .collect();
        for lock_key in lock.keys() {
            if !manifest_keys.contains(lock_key) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }

        // praxis-way rule 10 (manifest agrees with lock): the same
        // straggler invariant over the `[compact_archive_signatures]`
        // space. The compact space is a SUBSET of `[hashes]` — only the
        // OWL/USC/Language compilable sources publish a portable compact
        // `.prx` content address — so the faithful assertion mirrors the
        // reverse direction above: no compact pin may exist for a source
        // `<name>@<version>` the manifest does not declare. (The forward
        // direction is intentionally not asserted: a registered source
        // having no compact pin is legitimate, just as a Stub has no
        // `[hashes]` entry.) Without this check the compact pins are only
        // parser-enforced and have no runnable Verdict.
        let compact =
            crate::applied::data_provisioning::registry::lock_compact_archive_signatures();
        for compact_key in compact.keys() {
            if !manifest_keys.contains(compact_key) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }

        // The same straggler invariant over the `[store_bundle_signatures]`
        // space (the English store bundle's per-toolchain build-output pin):
        // no bundle pin may exist for a source the manifest does not declare.
        let bundles = crate::applied::data_provisioning::registry::lock_store_bundle_signatures();
        for bundle_key in bundles.keys() {
            if !manifest_keys.contains(bundle_key) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }

        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "LockManifestAgreement",
        "every praxis.lock entry agrees with its praxis.toml counterpart on hash; no stragglers either side",
        "Dolstra (2006) The Purely Functional Software Deployment Model §5.1"
    );
}

pr4xis::register_axiom!(
    LockManifestAgreement,
    "Dolstra (2006) The Purely Functional Software Deployment Model §5.1"
);

/// Axiom: every registered source whose identity is materialized (a real
/// `RawHash` claim, not a `Stub`) resolves to a `local_path()` that points
/// to a file actually present in `crates/domains/data/`.
///
/// FAIR F2 — "data are described with rich metadata" — requires that the
/// declared local path be a true address into the data store, not a
/// formula-generated string. Without this axiom, `pr4xis update --check`
/// reports paths the runtime cannot read, and the manifest+lock invariants
/// say nothing about whether the bytes are reachable.
///
/// Stub-identity entries (declared in `praxis.toml` but not yet fetched)
/// are skipped — they have no bytes to verify and no lock entry either;
/// the [`LockManifestAgreement`] axiom owns their drift detection.
///
/// Citation: Wilkinson et al. (2016) The FAIR Guiding Principles for
/// Scientific Data Management and Stewardship, *Scientific Data* 3:160018,
/// §F2 (richly-described metadata) and §A1 (data retrievable by identifier).
pub struct RegistryLocalPathsExist;

impl Axiom for RegistryLocalPathsExist {
    fn verify(&self) -> Verdict {
        use crate::formal::meta::artifact_identity::ontology::ClaimData;
        let entries = crate::applied::data_provisioning::registry::data_sources();
        let workspace_root = workspace_root_for_test();
        for entry in entries {
            let raw_hash = entry
                .identity
                .0
                .iter()
                .find(|c| matches!(c.concept, IdentityConcept::RawHash));
            let is_stub = raw_hash
                .map(|c| matches!(c.data, ClaimData::Stub { .. }))
                .unwrap_or(true);
            // The empty-content digest (the digest of zero bytes UNDER THE
            // PIN'S OWN ALGORITHM — the one verify leg, `hash_hex`) is the
            // conventional placeholder in praxis.lock for sources
            // registered with deferred fetch — semantically a Stub even
            // though the lock has a digest slot filled.
            let is_placeholder_hash = raw_hash.is_some_and(|c| match &c.data {
                ClaimData::HashAlgorithm {
                    algorithm,
                    digest_hex,
                } => pr4xis_runtime::address::hash_hex(*algorithm, b"") == *digest_hex,
                _ => false,
            });
            if is_stub || is_placeholder_hash {
                continue;
            }
            let rel = entry.local_path();
            let abs = workspace_root.join(&rel);
            if !abs.is_file() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "RegistryLocalPathsExist",
        "every materialized source's local_path() points to a file on disk",
        "Wilkinson et al. (2016) FAIR Guiding Principles §F2 + §A1"
    );
}

pr4xis::register_axiom!(
    RegistryLocalPathsExist,
    "Wilkinson et al. (2016) FAIR Guiding Principles §F2 + §A1"
);

/// Locate the workspace root from the praxis-domains crate manifest
/// directory. The data tree lives at `<workspace-root>/crates/domains/data/`
/// — and `RegistryEntry::local_path()` returns paths workspace-relative,
/// so resolving them needs the workspace root.
///
/// Following the same convention as `pr4xis-cli`'s
/// [`workspace_root`](https://docs.rs/...) function, falls back to the
/// current directory when invoked outside Cargo (e.g. release-time
/// inclusion via `include_bytes!`).
fn workspace_root_for_test() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR points at crates/domains; workspace root is two
    // parents up. This is the same resolution pr4xis-cli uses.
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

// Silence unused-import warnings; `IdentityConcept` and `concept_name` are
// part of the public re-export surface this module advertises.
#[allow(dead_code)]
fn _identity_concept_witness(_: IdentityConcept) {}
#[allow(dead_code)]
fn _concept_name_witness(_: SourceTaxonomyConcept) -> &'static str {
    concept_name(SourceTaxonomyConcept::Source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<DataProvisioningCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        DataProvisioningOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn seven_concepts() {
        assert_eq!(DataProvisioningConcept::variants().len(), 7);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn canonical_encoding_covers_every_leaf() {
        use crate::formal::meta::source_taxonomy::ontology::is_leaf;
        for c in SourceTaxonomyConcept::variants() {
            if !is_leaf(c) {
                continue;
            }
            // Every leaf must map to a decoder-capable encoding (not
            // Binary, which is the "no decoder" sentinel).
            let ct = canonical_encoding(c);
            assert_ne!(ct, ContentType::Binary, "leaf {:?} has no decoder", c);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn family_dir_partitions_legal_and_lexicon() {
        use SourceTaxonomyConcept as C;
        assert!(family_dir(C::Statute).starts_with("legal/"));
        assert!(family_dir(C::Regulation).starts_with("legal/"));
        assert!(family_dir(C::Language).starts_with("lexicons/"));
        assert!(family_dir(C::LegalLexicon).starts_with("lexicons/"));
    }
}

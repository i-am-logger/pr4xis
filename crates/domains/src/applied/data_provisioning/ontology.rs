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
    /// `xml::uslm::reader::read_uslm_title` (runtime) /
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
        C::Language => ContentType::XmlLmf,
        // US federal statutes are published by GPO as PDF on
        // govinfo.gov (ISO 32000-2:2020 PDF 2.0; Bluebook §18
        // preferred authenticated digital edition). M4.γ's
        // PDF loader consumes the bytes and emits a typed
        // `PdfBuildExtraction` const at build time.
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
        // LegalLexicons ship as LMF XML — same shape as Language
        // (WordNet) and function-word lexica, reusing the same
        // `decoders::xml_lmf` decoder. The bundled
        // `us_legal_lexicon@2026` instance is the canonical example.
        C::LegalLexicon => ContentType::XmlLmf,
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
        // ConceptualSpec sources ship as XHTML (the W3C-published
        // recommendation format for text-form specifications such as
        // the XML Information Set rec, Cowan & Tobin 2004). Decoder:
        // build-time text-scan of the section-heading structure
        // (`<h3>` markup with regular class attribution).
        C::ConceptualSpec => ContentType::Xhtml,
        // Non-leaf concepts have no decoder — they're abstract.
        C::Source
        | C::Lexicon
        | C::DomainLexicon
        | C::LegalCorpus
        | C::TypographyResource
        | C::SchemaSpec => ContentType::Binary,
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
/// attribute on Lexicon kinds) and from `praxis.lock`'s pinned sha256 for
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
    /// Identity claims, synthesized from manifest + lock at load time.
    pub identity: CompositeIdentity,
}

impl RegistryEntry {
    /// `true` if the URL serves gzip-compressed bytes (i.e. ends with `.gz`).
    /// Implied from the URL; not a separate manifest field.
    pub fn gzipped(&self) -> bool {
        self.url.ends_with(".gz")
    }

    /// The encoding praxis decodes this source as, derived from `kind`.
    pub fn content_type(&self) -> ContentType {
        canonical_encoding(self.kind)
    }

    /// Workspace-relative local path where this entry materializes. Derived
    /// by convention from the kind's family and the source's `(name, version)`.
    ///
    /// Layout:
    ///   `crates/domains/data/<family>/<name>/<name>-<version>.<ext>`
    ///
    /// where `<family>` is `lexicons` (for Lexicon-family kinds) or `legal`
    /// (for LegalCorpus-family kinds), and `<ext>` is `.xml` for XmlLmf,
    /// `.txt` for Plaintext, `.json` for Json, `.pdf` for Pdf, `.bin`
    /// otherwise.
    pub fn local_path(&self) -> String {
        let family = family_dir_for(self.kind, &self.name);
        let ext = match self.content_type() {
            ContentType::XmlLmf | ContentType::UslmXml | ContentType::XmlXsd => "xml",
            ContentType::Xhtml => "xhtml",
            ContentType::Plaintext | ContentType::AdobeGlyphList => "txt",
            ContentType::Json => "json",
            ContentType::Pdf => "pdf",
            ContentType::Video => "bin",
            ContentType::Audio => "bin",
            ContentType::Binary => "bin",
        };
        // Adobe AGL has a fixed canonical filename in the public
        // typography registry; downstream code references it
        // directly via include_str! so the path is stable.
        if matches!(self.kind, SourceTaxonomyConcept::TypographicGlyphSet) {
            return format!(
                "crates/domains/data/{family}/{name}/glyphlist.{ext}",
                family = family,
                name = "adobe",
                ext = ext
            );
        }
        format!(
            "crates/domains/data/{family}/{name}/{name}-{version}.{ext}",
            family = family,
            name = self.name,
            version = self.version,
            ext = ext
        )
    }
}

/// The on-disk family directory for a given kind. Mirrors the praxis-domains
/// code-path convention so the data layout matches the ontology layout.
///
/// Kind-only; use [`family_dir_for`] when the source name is known
/// (some kinds — e.g. `XmlSchemaDefinition` — host instances that
/// belong to different corpora, and the corpus is keyed on the
/// source `name`).
pub fn family_dir(kind: SourceTaxonomyConcept) -> &'static str {
    family_dir_for(kind, "")
}

/// The on-disk family directory for a `(kind, name)` pair. Most
/// kinds have a single canonical family; the `name` parameter only
/// matters for `XmlSchemaDefinition` (USLM XSD lives under the U.S.
/// Code corpus, XHTML XSD lives under the markup-schemas corpus).
pub fn family_dir_for(kind: SourceTaxonomyConcept, name: &str) -> &'static str {
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
    } else if matches!(
        kind,
        C::SchemaSpec | C::XmlSchemaDefinition | C::ConceptualSpec
    ) {
        // XSDs live under the corpus they schema. The USLM XSD is
        // shipped at `data/legal/uscode/schema/` (the U.S. Code
        // corpus); the XHTML XSD is shipped at
        // `data/markup-schemas/xhtml/` (the M4.η.1 HTML5 ontology
        // grounding source); the W3C `xml.xsd` and Information Set
        // recommendation live under `data/markup-schemas/xml/` (the
        // M4.η.2 XML 1.0 ontology grounding sources). Future schema
        // kinds for non-legal corpora can extend this branch — the
        // per-name dispatch is the seam where new families plug in.
        match name {
            "xhtml_1_0_xsd" => "markup-schemas/xhtml",
            "xml_1_0_namespace_xsd" | "xml_infoset" => "markup-schemas/xml",
            _ => "legal/uscode/schema",
        }
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

/// Axiom: every registered source is *realizable* — either through a
/// runtime decoder (Heap-store path: bytes fetched from URL, decoded
/// per `canonical_encoding`) or through lock-time structural data
/// (Static-store path: the parsed ontology lives directly in
/// `praxis.lock`'s `[structural.*]` block, ready for build-time
/// codegen consumption).
///
/// An entry with neither path would be unreachable: no decoder, no
/// pre-baked structure. The axiom catches that asymmetry at startup.
pub struct DecoderTotalityPerKind;

impl Axiom for DecoderTotalityPerKind {
    fn verify(&self) -> Verdict {
        use crate::formal::meta::artifact_identity::ontology::ClaimData;
        for entry in crate::applied::data_provisioning::registry::data_sources() {
            // Sources whose RawHash identity claim is a Stub are
            // registered in praxis.toml but not yet loadable (awaiting
            // PDF/NLP infrastructure). They are not expected to be
            // materializable through the runtime decoder or the lock-
            // structural path until the loader exists; the
            // decoder-totality check re-activates automatically when
            // the loader fills in their hash + structural data.
            let is_stub_only = !entry.identity.0.is_empty()
                && entry.identity.0.iter().all(|c| {
                    matches!(c.concept, IdentityConcept::RawHash)
                        && matches!(c.data, ClaimData::Stub { .. })
                });
            if is_stub_only {
                continue;
            }
            let ct = canonical_encoding(entry.kind);
            let runtime_decoder = crate::applied::data_provisioning::decoders::has_decoder_for(ct);
            let lock_structural = crate::applied::data_provisioning::registry::structural_for(
                &entry.name,
                &entry.version,
            )
            .is_some();
            if !runtime_decoder && !lock_structural {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DecoderTotalityPerKind",
        "every registered source has a runtime decoder for its canonical_encoding OR a lock structural block",
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
            let lock_sha = lock.get(&key);
            let raw_hash_claim = entry
                .identity
                .0
                .iter()
                .find(|c| matches!(c.concept, IdentityConcept::RawHash));
            let manifest_sha = raw_hash_claim.and_then(|c| match &c.data {
                ClaimData::Sha256(hex) => Some(hex.as_str()),
                _ => None,
            });
            let is_stub = raw_hash_claim
                .map(|c| matches!(c.data, ClaimData::Stub { .. }))
                .unwrap_or(false);
            match (is_stub, manifest_sha, lock_sha) {
                // Stub claim (loadable-pending) and no lock entry: OK.
                (true, _, None) => {}
                // Stub claim but lock has an entry: drift (manifest
                // forgot to pin even though the lock has data).
                (true, _, Some(_)) => {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
                // Real hash on both sides, agree: OK.
                (false, Some(m), Some(l)) if m.eq_ignore_ascii_case(l.as_str()) => {}
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
    use pr4xis::category::Concept;
    use pr4xis::category::laws::assert_category_laws;

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
        assert_eq!(DataProvisioningConcept::variants().len(), 7);
    }

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

    #[test]
    fn family_dir_partitions_legal_and_lexicon() {
        use SourceTaxonomyConcept as C;
        assert!(family_dir(C::Statute).starts_with("legal/"));
        assert!(family_dir(C::Regulation).starts_with("legal/"));
        assert!(family_dir(C::Language).starts_with("lexicons/"));
        assert!(family_dir(C::LegalLexicon).starts_with("lexicons/"));
    }
}

//! Source taxonomy — concepts, hierarchy, adjunction graph, axioms.
//!
//! See `mod.rs` for the literature inventory and design rationale.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "SourceTaxonomy",
    source: "Hart (1961) The Concept of Law, Oxford; MacLane (1971) Categories for the Working Mathematician §IV.1; Pustejovsky (1995) The Generative Lexicon, MIT Press; Vossen (1998) EuroWordNet, Springer; Sartor (2005) Legal Reasoning, Springer; Solan (1993) The Language of Judges, Univ. Chicago; Marbury v. Madison, 5 U.S. 137 (1803); Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3:160018; Dolstra (2006) The Purely Functional Software Deployment Model, PhD thesis Utrecht University; Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema Definition Language (XSD) 1.1 Part 1: Structures, W3C Recommendation",

    concepts: [
        // === Root ===
        Source,

        // === Lexicon family ===
        Lexicon,
        Language,            // Lexicon for a natural language (English WordNet, etc.)
        DomainLexicon,       // Lexicon restricted to a specialty
        LegalLexicon,        // DomainLexicon for legal terms of art (Black's, statutory defs)
        SchemaVocabulary,    // DomainLexicon for schema-vocabulary names (XML/HTML element / attribute / type / group names from published schema specs)
        ClosedClassLexicon,  // Lexicon for the closed-class (function-word) stratum — the disjoint complement of open-class Language (Quirk et al. 1985 §2.34)
        InflectionLexicon,   // Lexicon of inflected word-forms ↦ base lemma (AGID — Atkinson 2016): the morphological-exception source
        DerivationalLexicon, // Lexicon of derivational word-clusters (CatVar — Habash & Dorr 2003): the cross-POS word-formation source

        // === LegalCorpus family (Hart 1961 primary + secondary rules) ===
        LegalCorpus,
        Statute,                // primary rule (legislative enactment) — generic, jurisdiction-agnostic
        UsFederalStatute,       // leaf: a Title-of-the-U.S.-Code section (e.g., 18 U.S.C. § 1514A)
        UsCodeTitle,            // leaf: a whole Title of the U.S. Code as USLM XML (e.g., Title 18). Container for many UsFederalStatutes.
        Regulation,             // secondary rule (agency-promulgated, implements statute)
        ConstitutionalArticle,  // supreme primary rule (Marbury v. Madison 1803)
        ProceduralRule,         // primary rule of procedure (FRCP, OALJ rules)
        CaseLaw,                // secondary rule (judicial precedent)

        // === TypographyResource family ===
        TypographyResource,      // root of typographic mapping tables
        TypographicGlyphSet,     // leaf: glyph-name → Unicode codepoint map (Adobe AGL, etc.)

        // === SchemaSpec family (W3C XSD 1.1 Part 1 §1.1) ===
        SchemaSpec,                 // root of structural schema specifications
        XmlSchemaDefinition,        // leaf: a W3C XSD 1.1 schema document (e.g., USLM XSD)
        XmlDocumentTypeDefinition,  // leaf: a W3C XML 1.0 §4 DTD (e.g., WN-LMF 1.3 DTD)
        OoxmlSchemaArchive,         // leaf: a ZIP bundle of XSDs published as a single ECMA-376 / ISO/IEC 29500 unit
        ConceptualSpec,             // leaf: a published text-form conceptual specification (e.g., W3C XML Information Set rec)
        OntologyVocabulary,         // leaf: a published W3C OWL 2 / RDF-XML vocabulary (e.g., the SPAR CiTO Citation Typing Ontology)

        // === TestSuite family (W3C QA Framework: Test Methodology Guidelines, 2008) ===
        TestSuite,                  // root of published conformance test corpora
        XmlSchemaTestSuite,         // leaf: the W3C XML Schema Test Suite (xsts) — a corpus of validity-labeled XSD documents
        XmlConformanceTestSuite,    // leaf: the W3C XML 1.0 Conformance Test Suite (XMLConf) — a corpus of {valid, invalid, not-wf, error}-labelled XML documents

        // === ControlledVocabulary family (ISO 25964-1 §4 / W3C SKOS) ===
        ControlledVocabulary,       // root of published controlled vocabularies / category-mapping tables (plain-text TSV)
        WindowStateVocabulary,      // leaf: the EWMH `_NET_WM_STATE` window-state atom set (freedesktop.org wm-spec)
        LexicalCategoryProjection,  // leaf: a grammatical-class → lexical-category map (OLiA Reference-Model class ↦ CCGbank category)
        MathOperatorVocabulary,     // leaf: the closed-class mathematical-operator vocabulary (OpenMath arith1/relation1 + ISO 80000-2 glyphs), authored as LMF-shaped XML
        ColorSchemeVocabulary,      // leaf: the Base16/Base24 named color-scheme collection (Tinted Theming framework spec) — a controlled set of {base00..base0F[+base10..base17]} hex palettes, one per named scheme
    ],

    labels: {
        Source: ("en", "Source",
            "Wilkinson et al. (2016) FAIR F1: the abstract root of every external corpus praxis can ingest."),
        Lexicon: ("en", "Lexicon",
            "Pustejovsky (1995) The Generative Lexicon: a structured lexical resource pairing entries with senses and qualia."),
        Language: ("en", "Language",
            "Vossen (1998) EuroWordNet: a Lexicon for a natural language (English, Spanish, etc.) — the bridge for general-vocabulary anchoring."),
        DomainLexicon: ("en", "Domain lexicon",
            "Pustejovsky (1995): a Lexicon scoped to a specialty domain, with domain-specific qualia (e.g., legal, medical, scientific)."),
        LegalLexicon: ("en", "Legal lexicon",
            "Solan (1993) The Language of Judges: a DomainLexicon for legal terms of art — statutory definitions, Black's Law Dictionary, judicial glossaries."),
        SchemaVocabulary: ("en", "Schema vocabulary",
            "Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema 1.1 Part 1 §3 (Schema Component Names): a DomainLexicon enumerating the closed-class element / attribute / type / group / model names declared by published schema specifications. The taxonomy slot is kept for future bundles in this family; the XSD → English projection's is_schema_vocabulary classifier now reads directly from each loaded XmlSchemaDefinition (HTML5 XSD, W3C xml.xsd, USLM-1.0.18 XSD self-annotations) rather than from a separate LMF bundle (M4.η.4)."),
        ClosedClassLexicon: ("en", "Closed-class lexicon",
            "Quirk, Greenbaum, Leech & Svartvik (1985) A Comprehensive Grammar of the English Language §2.34 (open vs closed word classes): a Lexicon enumerating the bounded CLOSED grammatical word classes — determiners, pronouns, prepositions, conjunctions, auxiliaries, copulas, particles, interjections, and the wh-/relative pronouns. The disjoint complement of the open-class natural-language WordNet (the `Language` leaf): function words, not content words. Category vocabulary per the OLiA Reference Model (Chiarcos & Sukhareva 2015, see [sources.olia]). The bundled crates/domains/data/function-words/english.xml is a hand-curated, citation-anchored inventory — the membership is NOT yet machine-loaded from the cited grammars (a tracked praxis-debt)."),
        InflectionLexicon: ("en", "Inflection lexicon",
            "Atkinson (2016) Automatically Generated Inflection Database (AGID), http://wordlist.aspell.net: a Lexicon mapping every inflected word-form to its base lemma and inflection slot (noun plural; verb past / past-participle / -ing / -s; adjective comparative / superlative). The authoritative source of English morphological EXCEPTIONS — the irregular forms a productive rule cannot generate — read by the dual-route lemmatizer (Pinker 1991). A Lexicon sibling of the open-class WordNet (Language) and the closed-class function words (ClosedClassLexicon), carrying the inflectional rather than the lexical-semantic axis."),
        DerivationalLexicon: ("en", "Derivational lexicon",
            "Habash & Dorr (2003) CatVar — A Database of Categorial Variations for English (HLT-NAACL): a Lexicon clustering words related by DERIVATIONAL morphology across part-of-speech (e.g. {nation_N, national_AJ, nationalize_V, nationalization_N}). The corpus-attested source of English word-formation, used to GROUND the productive derivational affix rules (Bauer 1983; the affixes themselves are productive generative rules, not an enumerable list). A Lexicon sibling of the inflectional AGID (InflectionLexicon), carrying the derivational (cross-POS word-formation) rather than the inflectional axis."),
        LegalCorpus: ("en", "Legal corpus",
            "Hart (1961) The Concept of Law: the root of legal text resources — primary rules (statutes, constitutional articles, procedural rules) and secondary rules (regulations, case law) about them."),
        Statute: ("en", "Statute",
            "Hart (1961) primary rule: a legislative enactment binding within its jurisdiction. Jurisdiction-agnostic parent; instances declare a specific leaf (UsFederalStatute, …)."),
        UsFederalStatute: ("en", "U.S. federal statute",
            "A Title-of-the-U.S.-Code section enacted by Congress, structured per House Legislative Counsel's Manual on Drafting Style (2017) and cited per Bluebook §3.3 (e.g., 18 U.S.C. § 1514A, 49 U.S.C. § 42121)."),
        UsCodeTitle: ("en", "U.S. Code title",
            "A whole Title of the United States Code (e.g., Title 18 — Crimes and Criminal Procedure) as published by the LRC in USLM XML per 1 U.S.C. § 204. Container for one or more UsFederalStatute sections; the LRC's per-title XML zip is its authoritative publication unit (uscode.house.gov/download/)."),
        Regulation: ("en", "Regulation",
            "Hart (1961) secondary rule: an agency-promulgated rule implementing a statute (e.g., 29 CFR Part 1980)."),
        ConstitutionalArticle: ("en", "Constitutional article",
            "Marbury v. Madison (1803): the supreme primary rule, authorizing legislation and judicial review."),
        ProceduralRule: ("en", "Procedural rule",
            "Sartor (2005): a primary rule governing court procedure (FRCP, FRE, OALJ rules)."),
        CaseLaw: ("en", "Case law",
            "Sartor (2005): a secondary, interpretive rule emerging from judicial decisions; binds via stare decisis."),
        TypographyResource: ("en", "Typography resource",
            "ISO 32000-2:2020 §9.6.5: published typographic mapping table such as a glyph list or encoding vector — the substrate on which digital text-format decoders translate bytes/glyphs to Unicode."),
        TypographicGlyphSet: ("en", "Typographic glyph set",
            "ISO 32000-2:2020 §9.6.5.4 + Adobe Tech Note #5014: a published name→codepoint table (Adobe Glyph List, AGLFN) cited by PDF /Differences arrays to resolve glyph names to Unicode codepoints."),
        SchemaSpec: ("en", "Schema specification",
            "Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema 1.1 Part 1 §1.1: a published structural specification for documents of some content-type. The substrate ontology that downstream content-type ontologies (USLM, LMF, OOXML) load from."),
        OoxmlSchemaArchive: ("en", "OOXML schema archive",
            "ECMA International / ISO/IEC JTC 1/SC 34, ECMA-376 / ISO/IEC 29500 — Office Open XML, 5th edition (December 2016). A ZIP bundle of the 21 XSDs that define the Office Open XML vocabularies — WordprocessingML (`wml.xsd`), SpreadsheetML (`sml.xsd`), PresentationML (`pml.xsd`), DrawingML (`dml-*.xsd`), and the shared common types. The published canonical unit for the schemas; downstream OOXML readers (DOCX / XLSX / PPTX) validate against the XSDs inside."),
        XmlDocumentTypeDefinition: ("en", "XML Document Type Definition (DTD)",
            "Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008) Extensible Markup Language (XML) 1.0 Fifth Edition, §2.8 + §4: a Document Type Declaration's markup-declarations (ELEMENT / ATTLIST / ENTITY / NOTATION). The pre-XSD machine-readable grammar form for XML applications; still the canonical schema form for some published vocabularies (e.g., the Global WordNet WN-LMF DTD that WordNet 2025 ships against)."),
        XmlSchemaDefinition: ("en", "XML Schema Definition (XSD)",
            "Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema 1.1 Part 1, W3C Recommendation 5 April 2012: an XSD document declaring element/complexType/simpleType/attribute/group declarations for an XML vocabulary. Cited by content-type ontologies (USLM 1.0.18) that ground their type system in the schema rather than hand-coding."),
        ConceptualSpec: ("en", "Conceptual specification",
            "Cowan & Tobin (2004) W3C XML Information Set (Second Edition), W3C Recommendation 4 February 2004: a published text-form specification that defines a conceptual model (information items, abstract structures, taxonomies) rather than a concrete machine-readable schema. The conceptual taxonomy is loaded by parsing the section-heading structure of the W3C-published XHTML edition of the recommendation. Sibling to XmlSchemaDefinition under SchemaSpec."),
        OntologyVocabulary: ("en", "Ontology vocabulary (OWL)",
            "W3C OWL 2 Web Ontology Language Structural Specification (Motik, Patel-Schneider & Parsia eds., W3C Recommendation 11 December 2012): a published OWL 2 / RDF-XML vocabulary declaring named classes, object properties, and their rdfs:subClassOf / rdfs:subPropertyOf hierarchies. The canonical SPAR (Semantic Publishing and Referencing) example is CiTO, the Citation Typing Ontology (Peroni & Shotton 2012, J. Web Semantics 17:33-43). Sibling to XmlSchemaDefinition and ConceptualSpec under SchemaSpec: like an XSD it grounds a content-type's concept inventory, but in OWL/RDF rather than XSD form, read by social::software::markup::xml::owl::reader::read_owl."),
        TestSuite: ("en", "Conformance test suite",
            "W3C QA Framework: Test Methodology Guidelines (Curran et al. eds., W3C Note 22 February 2008): a published conformance test corpus paired with a normative specification — each test case carries an expected outcome and the union of cases across categories certifies a reader's conformance to the spec."),
        XmlSchemaTestSuite: ("en", "W3C XML Schema Test Suite (xsts)",
            "W3C XML Schema Working Group, XML Schema Test Suite, archive xsts-2007-06-20 at <https://www.w3.org/XML/2004/xml-schema-test-suite/xmlschema2006-11-06/xsts-2007-06-20.tar.gz>: ~14,328 schemaTest cases (~11,598 valid + ~2,730 invalid) drawn from Boeing, Microsoft, NIST and Sun contributions, each pairing a <schemaDocument> with an <expected validity=...> classification per the W3C XSD 1.1 Parts 1 + 2 (Gao et al. 2012; Peterson et al. 2012)."),
        XmlConformanceTestSuite: ("en", "W3C XML Conformance Test Suite (XMLConf)",
            "W3C XML Test Suite Working Group, XML Conformance Test Suite (XMLConf), archive xmlts20080827.tar.gz at <https://www.w3.org/XML/Test/xmlts20080827.tar.gz>: ~3,000 test cases drawn from Sun, James Clark, IBM, NIST/OASIS, Fuji Xerox and University of Edinburgh contributions; each TEST entry pairs a URI with a TYPE ∈ {valid, invalid, not-wf, error} per W3C XML 1.0 Fifth Edition (Bray et al. 2008) §2.1 well-formedness + §2.8 validity."),
        ControlledVocabulary: ("en", "Controlled vocabulary",
            "ISO 25964-1:2011 §4 (Thesauri and interoperability with other vocabularies — Part 1: Thesauri for information retrieval) + W3C SKOS Reference (Miles & Bechhofer eds., W3C Recommendation 18 August 2009): a published, bounded, citation-anchored set of controlled terms — or a category-mapping table between two such sets — distributed as a plain-text tab-separated table rather than as a lexicon (no senses/qualia), a schema (no document grammar), or a corpus (no running text). The sibling of Lexicon under Source for the controlled-term and category-projection resources praxis loads-not-encodes."),
        WindowStateVocabulary: ("en", "Window-state vocabulary",
            "freedesktop.org Extended Window Manager Hints (EWMH) v1.5 §5 `_NET_WM_STATE` (<https://specifications.freedesktop.org/wm-spec/1.5/ar01s05.html>): the controlled set of window-state atoms (_NET_WM_STATE_FULLSCREEN, _MODAL, _ABOVE, _BELOW, …) a client adds/removes/toggles, plus two cited compositor extensions (bspwm pseudo_tiled; Wayland xdg-shell/wlroots floating). A ControlledVocabulary leaf: one `bit_name<TAB>spec_atom<TAB>source` row per atom, the authority the StateBit alphabet is machine-checked complete-and-sound against (window_state::VocabularyComplete)."),
        LexicalCategoryProjection: ("en", "Lexical-category projection",
            "A controlled mapping from a grammatical-annotation class to its lexical category: each row pairs an OLiA Reference-Model class (Chiarcos & Sukhareva 2015, Semantic Web 6(4):379-386; see [sources.olia]) with its standard CCGbank category notation (Hockenmaier & Steedman 2007, Computational Linguistics 33(3):355-396 — the Combinatory Categorial Grammar lexical-category inventory). A ControlledVocabulary leaf carried as `olia_class<TAB>ccg_category[<TAB>valency_class]` rows: the universal lexical-category functor, loaded as data and interpreted (projection-as-data), never a Rust match."),
        MathOperatorVocabulary: ("en", "Mathematical-operator vocabulary",
            "The closed-class inventory of mathematical operators — each glyph bound to one OpenMath symbol with its STS signature (role, arity, result sort). A ControlledVocabulary leaf authored as LMF-shaped XML (one operator per `<LexicalEntry>`, same reader as WordNet/function-words) but consumed as raw bytes into an `OperatorVocabulary`, NOT a WordNet graph. Cite: OpenMath Content Dictionaries `arith1` + `relation1` and OpenMath Standard 2.0 (Kohlhase & Rabe, eds., 2019) §2.1.4 (Role), §4.3 (STS signatures); ISO 80000-2:2019 (operator glyphs). DERIVED/authored source-of-truth (not URL-fetchable): git-tracked, excluded from the published crate, shipped as the committed content-addressed `.prx`."),
        ColorSchemeVocabulary: ("en", "Color-scheme vocabulary",
            "The Base16 / Base24 named color-scheme collection — a controlled set of named palettes, each binding the framework's reserved slot keys (`base00`..`base0F` for Base16; `base00`..`base17` for Base24) to sRGB hex values. The vocabulary is the Tinted Theming framework specification's scheme corpus (the community `tinted-schemes` dataset, github.com/tinted-theming/schemes), a published, bounded set of named color schemes distributed as one YAML file per scheme. A ControlledVocabulary leaf (ISO 25964-1 controlled terms — each scheme name is a controlled term; each slot key a reserved name): the substrate the theming validator scans to certify the praxis luminance-monotonicity and WCAG-AA contrast axioms over real-world palettes. A COLLECTION source — its `.prx` archives the whole directory of YAML schemes, not a single file. Cite: Tinted Theming, Base16 Styling Guidelines + Base24 specification, <https://github.com/tinted-theming/home>; W3C WCAG 2.1 §1.4.3 (contrast). FETCHED source: the i-am-logger/tinted-schemes fork is the fetch endpoint; the raw theme tree is gitignored after fetch, regenerated via `pr4xis update`, and only the committed content-addressed `.prx` ships."),
    },

    is_a: [
        // Lexicon family
        (Lexicon, Source),
        (Language, Lexicon),
        (DomainLexicon, Lexicon),
        (LegalLexicon, DomainLexicon),
        (SchemaVocabulary, DomainLexicon),
        // Sibling of Language, NOT a child of DomainLexicon: function
        // words are the domain-independent closed-class stratum (Quirk
        // et al. 1985 §2.34), an orthogonal axis to the specialty-domain
        // scoping that defines DomainLexicon (Pustejovsky 1995).
        (ClosedClassLexicon, Lexicon),
        // AGID (Atkinson 2016) — a Lexicon on the inflectional axis (forms
        // ↦ lemma), sibling of the lexical-semantic WordNet under Lexicon.
        (InflectionLexicon, Lexicon),
        // CatVar (Habash & Dorr 2003) — a Lexicon on the derivational axis
        // (cross-POS word-formation clusters), sibling of AGID under Lexicon.
        (DerivationalLexicon, Lexicon),

        // LegalCorpus family
        (LegalCorpus, Source),
        (Statute, LegalCorpus),
        (UsFederalStatute, Statute),
        (UsCodeTitle, LegalCorpus),
        (Regulation, LegalCorpus),
        (ConstitutionalArticle, LegalCorpus),
        (ProceduralRule, LegalCorpus),
        (CaseLaw, LegalCorpus),

        // TypographyResource family
        (TypographyResource, Source),
        (TypographicGlyphSet, TypographyResource),

        // SchemaSpec family
        (SchemaSpec, Source),
        (XmlSchemaDefinition, SchemaSpec),
        (XmlDocumentTypeDefinition, SchemaSpec),
        (OoxmlSchemaArchive, SchemaSpec),
        (ConceptualSpec, SchemaSpec),
        (OntologyVocabulary, SchemaSpec),

        // TestSuite family
        (TestSuite, Source),
        (XmlSchemaTestSuite, TestSuite),
        (XmlConformanceTestSuite, TestSuite),

        // ControlledVocabulary family
        (ControlledVocabulary, Source),
        (WindowStateVocabulary, ControlledVocabulary),
        (LexicalCategoryProjection, ControlledVocabulary),
        (MathOperatorVocabulary, ControlledVocabulary),
        (ColorSchemeVocabulary, ControlledVocabulary),
    ],

    // Adjunction graph: pairs of concepts whose instances are connected by
    // adjoint functor pairs. The codegen reads these at build time and
    // emits a `<AName>To<BName>` adjunction for every pair of loaded
    // instances `(a, b)` where `a` inhabits the source concept and `b`
    // inhabits the target concept. MacLane (1971) §IV.1 grounds the
    // adjoint-pair semantics; each edge below names a domain-specific
    // adjunction with its own literature pointer.
    edges: [
        // 1 U.S.C. § 204: a U.S. Code title is the LRC's
        // publication-unit grouping of primary rules (Hart 1961
        // §V). A Title contains many Statutes; a Statute is
        // contained in exactly one Title. The unit/counit
        // surfaces "Title sections not yet ingested as statutes"
        // and "statutes whose Title isn't registered" as
        // defensible gaps.
        (UsCodeTitle, Statute, Adjoins),

        // Hart (1961) §V: statute authorizes regulation; regulation
        // implements statute. The unit/counit pair surfaces "statute
        // provisions without implementing regs" and "regs without
        // statutory basis" as defensible gaps.
        (Statute, Regulation, Adjoins),

        // Solan (1993): statute terms of art anchored in the legal
        // lexicon.
        (Statute, LegalLexicon, Adjoins),

        // Sartor (2005): statutes reference procedure (e.g., a
        // whistleblower statute's exhaustion requirement points into
        // the procedural code).
        (Statute, ProceduralRule, Adjoins),

        // Regulations reuse the same terms of art the statutes define.
        (Regulation, LegalLexicon, Adjoins),

        // Sartor (2005): judicial precedent interprets primary rules and
        // their implementing regs.
        (CaseLaw, Statute, Adjoins),
        (CaseLaw, Regulation, Adjoins),
        (CaseLaw, LegalLexicon, Adjoins),

        // Marbury v. Madison (1803): constitution authorizes statutes
        // and judicial review of them.
        (ConstitutionalArticle, Statute, Adjoins),
        (ConstitutionalArticle, CaseLaw, Adjoins),

        // Procedural rules carry their own terminology.
        (ProceduralRule, LegalLexicon, Adjoins),

        // Solan (1993): legal English bridges to common English —
        // every legal-corpus chain reaches Language transitively
        // through this edge.
        (LegalLexicon, Language, Adjoins),

        // Schema vocabulary bridges to common English the same way:
        // every schema-vocabulary name is either a token whose base
        // lemma resolves through WordNet (per Huddleston & Pullum
        // 2002 Ch. 19 productive prefixation / compounding) or an
        // abbreviation defined in its source schema specification.
        // The unit/counit pair surfaces (a) schema names whose
        // English base lemma isn't in WordNet, and (b) WordNet
        // lemmas no schema reuses.
        (SchemaVocabulary, Language, Adjoins),

        // Quirk et al. (1985) §2.34: the closed-class function-word
        // stratum is the disjoint complement of the open-class WordNet
        // (`Language`). The unit/counit pair surfaces (a) function words
        // a WordNet sense list omits, and (b) open-class lemmas the
        // closed inventory doesn't carry — the two halves of the lexicon.
        (ClosedClassLexicon, Language, Adjoins),

        // W3C OWL 2 §5 + RDF Schema §2.1: an OWL vocabulary's classes
        // and object properties each carry an rdfs:label whose tokens
        // anchor in common English the same way schema-vocabulary names
        // do (CiTO's "cites as evidence", "agrees with", "disputes",
        // etc. are productive English phrases per Huddleston & Pullum
        // 2002 Ch. 19). The unit/counit pair surfaces (a) OWL labels
        // whose English base lemma isn't in WordNet, and (b) WordNet
        // lemmas the vocabulary doesn't reuse.
        (OntologyVocabulary, Language, Adjoins),

        // Gao, Sperberg-McQueen & Thompson (2012) W3C XSD 1.1 Part 1
        // §3: an XSD schema definition declares a closed set of
        // element / attribute / type / group / model names — i.e.
        // the schema vocabulary. The unit/counit pair surfaces
        // (a) XSD-declared names not registered in the schema
        // vocabulary (gaps), and (b) registered vocabulary names
        // no loaded XSD exercises.
        (XmlSchemaDefinition, SchemaVocabulary, Adjoins),

        // W3C XSD 1.1 Part 1 §1.1: a schema specification grounds
        // the content-type ontology of every document it validates.
        // The USLM XSD (XmlSchemaDefinition instance) grounds every
        // UsCodeTitle XML document — the schema declares which
        // element/attribute combinations are well-formed; titles
        // are instances of that grammar. The unit/counit pair
        // surfaces (a) titles whose XML doesn't validate against
        // the schema, and (b) schema constructs that no published
        // title exercises (potential ontology dead code).
        (XmlSchemaDefinition, UsCodeTitle, Adjoins),
    ],
}

// ---------------------------------------------------------------------------
// String <-> Concept conversion (parser boundary)
// ---------------------------------------------------------------------------
//
// TOML carries `type = "Statute"` as a string; the parser maps that string
// directly to a `SourceTaxonomyConcept` variant, so every downstream call
// site is typed. Unknown names fail closed — no silent default, no
// pass-through. The mapping mirrors variant identifiers exactly so the
// invariant `format!("{:?}", c)` ↔ `parse(s)` round-trips.

/// Parse a praxis-taxonomy concept name into its typed variant. `None` if
/// `s` does not match any declared concept.
pub fn parse_concept(s: &str) -> Option<SourceTaxonomyConcept> {
    use SourceTaxonomyConcept as C;
    Some(match s {
        "Source" => C::Source,
        "Lexicon" => C::Lexicon,
        "Language" => C::Language,
        "DomainLexicon" => C::DomainLexicon,
        "LegalLexicon" => C::LegalLexicon,
        "SchemaVocabulary" => C::SchemaVocabulary,
        "ClosedClassLexicon" => C::ClosedClassLexicon,
        "InflectionLexicon" => C::InflectionLexicon,
        "DerivationalLexicon" => C::DerivationalLexicon,
        "LegalCorpus" => C::LegalCorpus,
        "Statute" => C::Statute,
        "UsFederalStatute" => C::UsFederalStatute,
        "UsCodeTitle" => C::UsCodeTitle,
        "Regulation" => C::Regulation,
        "ConstitutionalArticle" => C::ConstitutionalArticle,
        "ProceduralRule" => C::ProceduralRule,
        "CaseLaw" => C::CaseLaw,
        "TypographyResource" => C::TypographyResource,
        "TypographicGlyphSet" => C::TypographicGlyphSet,
        "SchemaSpec" => C::SchemaSpec,
        "XmlSchemaDefinition" => C::XmlSchemaDefinition,
        "XmlDocumentTypeDefinition" => C::XmlDocumentTypeDefinition,
        "OoxmlSchemaArchive" => C::OoxmlSchemaArchive,
        "ConceptualSpec" => C::ConceptualSpec,
        "OntologyVocabulary" => C::OntologyVocabulary,
        "TestSuite" => C::TestSuite,
        "XmlSchemaTestSuite" => C::XmlSchemaTestSuite,
        "XmlConformanceTestSuite" => C::XmlConformanceTestSuite,
        "ControlledVocabulary" => C::ControlledVocabulary,
        "WindowStateVocabulary" => C::WindowStateVocabulary,
        "LexicalCategoryProjection" => C::LexicalCategoryProjection,
        "MathOperatorVocabulary" => C::MathOperatorVocabulary,
        "ColorSchemeVocabulary" => C::ColorSchemeVocabulary,
        _ => return None,
    })
}

/// Canonical string for a concept. Used in error messages and as the
/// inverse of [`parse_concept`].
pub fn concept_name(c: SourceTaxonomyConcept) -> &'static str {
    use SourceTaxonomyConcept as C;
    match c {
        C::Source => "Source",
        C::Lexicon => "Lexicon",
        C::Language => "Language",
        C::DomainLexicon => "DomainLexicon",
        C::LegalLexicon => "LegalLexicon",
        C::SchemaVocabulary => "SchemaVocabulary",
        C::ClosedClassLexicon => "ClosedClassLexicon",
        C::InflectionLexicon => "InflectionLexicon",
        C::DerivationalLexicon => "DerivationalLexicon",
        C::LegalCorpus => "LegalCorpus",
        C::Statute => "Statute",
        C::UsFederalStatute => "UsFederalStatute",
        C::UsCodeTitle => "UsCodeTitle",
        C::Regulation => "Regulation",
        C::ConstitutionalArticle => "ConstitutionalArticle",
        C::ProceduralRule => "ProceduralRule",
        C::CaseLaw => "CaseLaw",
        C::TypographyResource => "TypographyResource",
        C::TypographicGlyphSet => "TypographicGlyphSet",
        C::SchemaSpec => "SchemaSpec",
        C::XmlSchemaDefinition => "XmlSchemaDefinition",
        C::XmlDocumentTypeDefinition => "XmlDocumentTypeDefinition",
        C::OoxmlSchemaArchive => "OoxmlSchemaArchive",
        C::ConceptualSpec => "ConceptualSpec",
        C::OntologyVocabulary => "OntologyVocabulary",
        C::TestSuite => "TestSuite",
        C::XmlSchemaTestSuite => "XmlSchemaTestSuite",
        C::XmlConformanceTestSuite => "XmlConformanceTestSuite",
        C::ControlledVocabulary => "ControlledVocabulary",
        C::WindowStateVocabulary => "WindowStateVocabulary",
        C::LexicalCategoryProjection => "LexicalCategoryProjection",
        C::MathOperatorVocabulary => "MathOperatorVocabulary",
        C::ColorSchemeVocabulary => "ColorSchemeVocabulary",
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Walk `is_a` to find every ancestor of `concept` in the taxonomy. Returns
/// the ancestors in any order; does not include `concept` itself.
pub fn ancestors_of(concept: SourceTaxonomyConcept) -> Vec<SourceTaxonomyConcept> {
    let sub: Vec<_> = SourceTaxonomyCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == SourceTaxonomyRelationKind::Subsumption)
        .map(|m| (m.source(), m.target()))
        .collect();
    let mut out = Vec::new();
    let mut stack = vec![concept];
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

/// True iff `concept` is in the LegalCorpus subtree (i.e., LegalCorpus is
/// in its ancestor set).
pub fn is_legal_corpus(concept: SourceTaxonomyConcept) -> bool {
    concept == SourceTaxonomyConcept::LegalCorpus
        || ancestors_of(concept).contains(&SourceTaxonomyConcept::LegalCorpus)
}

/// True iff `concept` is in the Lexicon subtree.
pub fn is_lexicon(concept: SourceTaxonomyConcept) -> bool {
    concept == SourceTaxonomyConcept::Lexicon
        || ancestors_of(concept).contains(&SourceTaxonomyConcept::Lexicon)
}

/// True iff `concept` is a *leaf* (no proper descendant in the taxonomy).
/// The leaves are the kinds a `[[source]]` entry can declare as its
/// `type` field.
///
/// DERIVED from the loaded `is_a` graph (audit 2026-06-12 D-18): a leaf is a
/// concept no other concept is_a — i.e. one that is never the `target` of a
/// `Subsumption` morphism. This replaces a hand-maintained `matches!` list that
/// had to be edited in lockstep with the `is_a:` block and was only guarded by a
/// brittle leaf-count literal; now the predicate cannot drift from the graph it
/// describes.
pub fn is_leaf(concept: SourceTaxonomyConcept) -> bool {
    SourceTaxonomyCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == SourceTaxonomyRelationKind::Subsumption)
        .all(|m| m.target() != concept)
}

/// Adjunction edges from this concept (the right-hand sides of `Adjoins`
/// edges with `concept` as the source). The codegen consults this to
/// emit per-instance adjunction functors automatically.
pub fn adjoint_targets(concept: SourceTaxonomyConcept) -> Vec<SourceTaxonomyConcept> {
    SourceTaxonomyCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == SourceTaxonomyRelationKind::Adjoins && m.source() == concept)
        .map(|m| m.target())
        .collect()
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Hart (1961) primary-vs-secondary distinction.
///
/// Primary rules directly govern conduct (statutes, constitutional
/// articles, procedural rules). Secondary rules are *about* primary
/// rules — how they are recognized, changed, adjudicated (regulations
/// implement statutes; case law interprets them; legal lexicons gloss
/// their terms).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HartRuleKind {
    Primary,
    Secondary,
    NotApplicable,
}

#[derive(Debug, Clone)]
pub struct HartRule;

impl Quality for HartRule {
    type Individual = SourceTaxonomyConcept;
    type Value = HartRuleKind;

    fn get(&self, concept: &SourceTaxonomyConcept) -> Option<HartRuleKind> {
        use SourceTaxonomyConcept as C;
        Some(match concept {
            C::Statute | C::UsFederalStatute | C::ConstitutionalArticle | C::ProceduralRule => {
                HartRuleKind::Primary
            }
            C::Regulation | C::CaseLaw | C::LegalLexicon => HartRuleKind::Secondary,
            _ => HartRuleKind::NotApplicable,
        })
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for SourceTaxonomyOntology {
    type Cat = SourceTaxonomyCategory;
    type Qual = HartRule;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(SourceTaxonomyWellFormed));
        axioms.push(Box::new(EveryAdjointEdgeTyped));
        axioms.push(Box::new(LegalAdjunctionsTerminateInLanguage));
        axioms.push(Box::new(PrimarySecondaryDistinction));
        axioms
    }
}

/// Axiom: every non-root concept reaches `Source` via `is_a`.
///
/// Hart (1961) — taxonomic completeness: each kind of legal artifact
/// must inherit from a recognized category. For the broader Source
/// taxonomy, every leaf must trace back to the root or the registry
/// loses well-typed dispatch.
pub struct SourceTaxonomyWellFormed;

impl Axiom for SourceTaxonomyWellFormed {
    fn verify(&self) -> Verdict {
        for c in SourceTaxonomyConcept::variants() {
            if c == SourceTaxonomyConcept::Source {
                continue;
            }
            if !ancestors_of(c).contains(&SourceTaxonomyConcept::Source) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SourceTaxonomyWellFormed",
        "every non-root concept reaches Source via is_a",
        "Hart (1961) The Concept of Law — taxonomic completeness of legal kinds"
    );
}

pr4xis::register_axiom!(SourceTaxonomyWellFormed, "Hart (1961) The Concept of Law");

/// Axiom: every `Adjoins`-kinded edge connects two concepts in the
/// taxonomy. Trivially true at compile time because the macro
/// enforces that edges reference declared concepts, but stated
/// here so the axiom set documents the invariant explicitly.
///
/// MacLane (1971) §IV.1: an adjoint pair is by definition a pair of
/// functors between two categories. Untyped adjunctions are not well-formed.
pub struct EveryAdjointEdgeTyped;

impl Axiom for EveryAdjointEdgeTyped {
    fn verify(&self) -> Verdict {
        // The macro already enforces concept-typed edges; this axiom
        // asserts the at-least-one-Adjoins-edge invariant so an empty
        // adjunction graph would fail (no legal corpora wire up).
        let count = SourceTaxonomyCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == SourceTaxonomyRelationKind::Adjoins)
            .count();
        if count == 0 {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EveryAdjointEdgeTyped",
        "the adjunction graph is non-empty and every edge connects declared concepts",
        "MacLane (1971) Categories for the Working Mathematician §IV.1"
    );
}

pr4xis::register_axiom!(
    EveryAdjointEdgeTyped,
    "MacLane (1971) Categories for the Working Mathematician §IV.1"
);

/// Axiom: every leaf concept in the LegalCorpus subtree reaches
/// `Language` by traversing `Adjoins` edges transitively. This is the
/// "legal text is always anchorable in natural language" invariant:
/// given a statute, you can always chain adjunctions to reach English
/// senses through some path (Statute → LegalLexicon → Language is the
/// canonical one).
///
/// Solan (1993) *The Language of Judges* — legal English is a domain
/// variant of common English; every legal-of-art term either is itself
/// a common word with a specialized sense or is glossed by terms that
/// are.
pub struct LegalAdjunctionsTerminateInLanguage;

impl Axiom for LegalAdjunctionsTerminateInLanguage {
    fn verify(&self) -> Verdict {
        let adjoins: Vec<_> = SourceTaxonomyCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == SourceTaxonomyRelationKind::Adjoins)
            .map(|m| (m.source(), m.target()))
            .collect();
        let is_a_edges: Vec<_> = SourceTaxonomyCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == SourceTaxonomyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for c in SourceTaxonomyConcept::variants() {
            if !is_legal_corpus(c) || !is_leaf(c) {
                continue;
            }
            // BFS from c over Adjoins ∪ is_a edges; must reach
            // Language. is_a inclusion means a jurisdiction-specific
            // leaf (UsFederalStatute) inherits its parent's
            // (Statute's) adjunction reachability without duplicating
            // every Adjoins edge per leaf.
            let mut seen = vec![c];
            let mut stack = vec![c];
            let mut reached = false;
            while let Some(curr) = stack.pop() {
                if curr == SourceTaxonomyConcept::Language {
                    reached = true;
                    break;
                }
                for (s, t) in &adjoins {
                    if *s == curr && !seen.contains(t) {
                        seen.push(*t);
                        stack.push(*t);
                    }
                }
                for (s, t) in &is_a_edges {
                    if *s == curr && !seen.contains(t) {
                        seen.push(*t);
                        stack.push(*t);
                    }
                }
            }
            if !reached {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "LegalAdjunctionsTerminateInLanguage",
        "every LegalCorpus leaf reaches Language via the adjunction graph",
        "Solan (1993) The Language of Judges, Univ. Chicago Press"
    );
}

pr4xis::register_axiom!(
    LegalAdjunctionsTerminateInLanguage,
    "Solan (1993) The Language of Judges, Univ. Chicago Press"
);

/// Axiom: Hart's primary-vs-secondary distinction partitions the
/// LegalCorpus leaves correctly. Statute, ConstitutionalArticle, and
/// ProceduralRule must be Primary; Regulation, CaseLaw, and
/// LegalLexicon must be Secondary.
///
/// Hart (1961) §V — the union of primary and secondary rules
/// constitutes a legal system; their distinction is what makes the
/// system intelligible.
pub struct PrimarySecondaryDistinction;

impl Axiom for PrimarySecondaryDistinction {
    fn verify(&self) -> Verdict {
        use SourceTaxonomyConcept as C;
        let q = HartRule;
        let primary = [
            C::Statute,
            C::UsFederalStatute,
            C::ConstitutionalArticle,
            C::ProceduralRule,
        ];
        let secondary = [C::Regulation, C::CaseLaw, C::LegalLexicon];
        for c in primary {
            if q.get(&c) != Some(HartRuleKind::Primary) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        for c in secondary {
            if q.get(&c) != Some(HartRuleKind::Secondary) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "PrimarySecondaryDistinction",
        "Statute (+ UsFederalStatute) / ConstitutionalArticle / ProceduralRule are Primary; Regulation / CaseLaw / LegalLexicon are Secondary",
        "Hart (1961) The Concept of Law §V"
    );
}

pr4xis::register_axiom!(
    PrimarySecondaryDistinction,
    "Hart (1961) The Concept of Law §V"
);

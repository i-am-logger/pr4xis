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
        CaregivingLexicon,   // DomainLexicon for family-caregiving terms of art (OAA/Medicare/Medicaid/NIA) — Track 1 chat lexicon
        HcbsComplianceLexicon, // DomainLexicon for HCBS workforce/compliance terms of art (EVV, Access Rule, program integrity) — Track 2 chat lexicon
        SchemaVocabulary,    // DomainLexicon for schema-vocabulary names (XML/HTML element / attribute / type / group names from published schema specs)
        ClosedClassLexicon,  // Lexicon for the closed-class (function-word) stratum — the disjoint complement of open-class Language (Quirk et al. 1985 §2.34)
        InflectionLexicon,   // Lexicon of inflected word-forms ↦ base lemma (AGID — Atkinson 2016): the morphological-exception source
        DerivationalLexicon, // Lexicon of derivational word-clusters (CatVar — Habash & Dorr 2003): the cross-POS word-formation source
        VerbClassLexicon,    // Lexicon of English verbs classified into nested syntactic-semantic classes, each a MANY-file collection (Levin 1993 alternations; Kipper, Korhonen, Ryant & Palmer 2008 VerbNet), WordNet-sense-key-keyed membership
        PredicateArgumentLexicon, // Lexicon of predicate argument-structure framesets, each a MANY-file collection (Palmer, Gildea & Kingsbury 2005 PropBank), lemma-keyed (no native WordNet link) with cross-POS roleset aliasing

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
        RealizationFrameTable,      // leaf: a natural-language-generation realization-frame table (frame_id ↦ surface template), Reiter & Dale (2000) realization stage
        MathOperatorVocabulary,     // leaf: the closed-class mathematical-operator vocabulary (OpenMath arith1/relation1 + ISO 80000-2 glyphs), authored as LMF-shaped XML
        ColorSchemeVocabulary,      // leaf: the Base16/Base24 named color-scheme collection (Tinted Theming framework spec) — a controlled set of {base00..base0F[+base10..base17]} hex palettes, one per named scheme
        SupertagCostTable,          // leaf: the CCG supertag/rule cost table — published CCGbank category & type-changing-rule frequencies as negative-log costs (Hockenmaier & Steedman 2007; Hockenmaier 2003), the derivation-ranking floor of the Viterbi chart
        QuoteGlyphVocabulary,       // leaf: the Unicode quotation-mark glyph inventory (General_Category Pi/Pf, plus the direction-ambiguous ASCII quote marks) — Unicode Standard Annex #44 (UCD), the glyph set a quoted-mention span collapse recognizes
        DashPunctuationVocabulary,  // leaf: the Unicode dash-punctuation glyph inventory (General_Category Pd, 25 members) — Unicode Standard Annex #44 (UCD), the glyph set the tokenizer's flush_word queries to trim a trailing/leading Unicode dash (e.g. an em dash) off an accumulated word buffer
        EnglishGraphemeClasses,     // leaf: the English orthographic grapheme classes — the 26 basic-Latin letters partitioned into vowel letters (a e i o u), consonant letters, and the one dual letter (y) — Carney 1994 Ch.1, cross-grounded by Quirk et al. 1985 §3.21; NOT a UCD derivation (Unicode says nothing about vowel-hood, which is language-specific)
        CaseFoldingTable,           // leaf: the Unicode simple case-folding table (CaseFolding.txt, status C+S) — the codepoint-for-codepoint fold a fold-on-miss lexicon lookup uses to recognize a differently-cased surface
        AssociativeConceptTable,    // leaf: the ConceptNet commonsense-association graph (Speer, Chin & Havasi 2017), WordNet-lemma-crosswalk-filtered — a relation/start-lemma/end-lemma/weight mapping table between two WordNet-lemma sets, independent of both WordNet's own gloss/hypernym construction and VerbNet's syntactic-alternation classification
        FrameSemanticTable,         // leaf: the FrameNet lexical-unit + frame-relation graph (Baker, Fillmore & Lowe 1998), extracted to lemma/pos/frame and frame-relation TSV rows — independent of WordNet's own gloss/hypernym construction, VerbNet's syntactic-alternation classification, and ConceptNet's crowd-sourced association graph; no native WordNet link exists in the source data, so resolution is lemma+POS-keyed, live, at query time
        SumoWordNetMappingTable,    // leaf: the SUMO WordNet↔SUMO mapping table (Niles & Pease 2001, 2003), extracted + RESOLVED offline to concept-value/term/relation-code TSV rows — a flat WordNet-concept↔upper-ontology-term correspondence, NOT a structured lexicon; SUMO's PWN-3.0 synset offsets don't match OEWN 2025, so (like VerbNet) synset→ConceptId is resolved offline via the version-stable WordNet sense key and baked into the committed table
        GazetteerTable,             // leaf: the GeoNames countryInfo gazetteer (download.geonames.org/export/dump/countryInfo.txt) — ISO/Country/Capital/Continent/neighbours rows, the substrate `natural::geography`'s Country/Capital/Region/`borders` facts load from, replacing a prior hand-curated fixture
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
        CaregivingLexicon: ("en", "Caregiving lexicon",
            "Pustejovsky (1995) The Generative Lexicon (domain-scoped qualia); Solan (1993) The Language of Judges (statutory terms of art): a DomainLexicon for family-caregiving terms of art — Older Americans Act / Medicare / Medicaid statutory definitions (42 USC 300ii, 1395x, 1396n, 1396p, 3002, 3030s, 15002), Medicaid HCBS waiver service definitions (42 CFR 440/441), and NIA dementia-care clinical vocabulary. Unlike LegalLexicon (a closed-class statutory RECOGNIZER vocabulary), this leaf is a chat-composable DEFINITIONAL lexicon: every synset carries one cited gloss recited to family caregivers. Bundled as crates/domains/data/care/caregiving_lexicon.xml (WN-LMF)."),
        HcbsComplianceLexicon: ("en", "HCBS compliance lexicon",
            "Pustejovsky (1995) The Generative Lexicon (domain-scoped qualia); Solan (1993) The Language of Judges (statutory terms of art): a DomainLexicon for HCBS workforce/compliance terms of art — the 21st Century Cures Act EVV definitions (42 USC 1396b(l)), Medicaid HCBS waiver / person-centered planning / settings rules (42 CFR 440/441), the 2024 Ensuring Access to Medicaid Services final rule (89 FR 40542), billing (42 CFR 447/438), and program integrity (42 CFR 455; 31 USC 3729). A chat-composable DEFINITIONAL lexicon, sibling of CaregivingLexicon. Bundled as crates/domains/data/care/hcbs_compliance_lexicon.xml (WN-LMF)."),
        SchemaVocabulary: ("en", "Schema vocabulary",
            "Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema 1.1 Part 1 §3 (Schema Component Names): a DomainLexicon enumerating the closed-class element / attribute / type / group / model names declared by published schema specifications. The taxonomy slot is kept for future bundles in this family; the XSD → English projection's is_schema_vocabulary classifier now reads directly from each loaded XmlSchemaDefinition (HTML5 XSD, W3C xml.xsd, USLM-1.0.18 XSD self-annotations) rather than from a separate LMF bundle (M4.η.4)."),
        ClosedClassLexicon: ("en", "Closed-class lexicon",
            "Quirk, Greenbaum, Leech & Svartvik (1985) A Comprehensive Grammar of the English Language §2.34 (open vs closed word classes): a Lexicon enumerating the bounded CLOSED grammatical word classes — determiners, pronouns, prepositions, conjunctions, auxiliaries, copulas, particles, interjections, and the wh-/relative pronouns. The disjoint complement of the open-class natural-language WordNet (the `Language` leaf): function words, not content words. Category vocabulary per the OLiA Reference Model (Chiarcos & Sukhareva 2015, see [sources.olia]). The bundled crates/domains/data/function-words/english.xml is a hand-curated, citation-anchored inventory — the membership is NOT yet machine-loaded from the cited grammars (a tracked praxis-debt)."),
        InflectionLexicon: ("en", "Inflection lexicon",
            "Atkinson (2016) Automatically Generated Inflection Database (AGID), http://wordlist.aspell.net: a Lexicon mapping every inflected word-form to its base lemma and inflection slot (noun plural; verb past / past-participle / -ing / -s; adjective comparative / superlative). The authoritative source of English morphological EXCEPTIONS — the irregular forms a productive rule cannot generate — read by the dual-route lemmatizer (Pinker 1991). A Lexicon sibling of the open-class WordNet (Language) and the closed-class function words (ClosedClassLexicon), carrying the inflectional rather than the lexical-semantic axis."),
        DerivationalLexicon: ("en", "Derivational lexicon",
            "Habash & Dorr (2003) CatVar — A Database of Categorial Variations for English (HLT-NAACL): a Lexicon clustering words related by DERIVATIONAL morphology across part-of-speech (e.g. {nation_N, national_AJ, nationalize_V, nationalization_N}). The corpus-attested source of English word-formation, used to GROUND the productive derivational affix rules (Bauer 1983; the affixes themselves are productive generative rules, not an enumerable list). A Lexicon sibling of the inflectional AGID (InflectionLexicon), carrying the derivational (cross-POS word-formation) rather than the inflectional axis."),
        VerbClassLexicon: ("en", "Verb-class lexicon",
            "Kipper, Korhonen, Ryant & Palmer (2008) A Large-scale Classification of English Verbs, Language Resources and Evaluation 42(1):21-40 (VerbNet); Levin (1993) English Verb Classes and Alternations, University of Chicago Press: a Lexicon classifying English verbs into a NESTED hierarchy of syntactic-semantic classes (Levin's diagnostic syntactic alternations), each class carrying thematic-role frames, example sentences, and a semantic-predicate decomposition, with per-class `<MEMBER>` entries keyed by WordNet sense-key. Independent of WordNet's own gloss/hypernym construction — the classification derives from Levin's syntactic-alternation methodology, not lexicographic definition — so it grounds a genuinely separate signal for verb-sense relatedness. A many-file COLLECTION (one XML per class), archived via the SAME deterministic directory-archive codec as the color-scheme collection (ColorSchemeVocabulary)."),
        PredicateArgumentLexicon: ("en", "Predicate-argument lexicon",
            "Palmer, M., Gildea, D. & Kingsbury, P. (2005) The Proposition Bank: An Annotated Corpus of Semantic Roles, Computational Linguistics 31(1):71-106 (PropBank); Bonial, C., Bonn, J., Conger, K., Hwang, J. & Palmer, M. (2014) PropBank: Semantics of New Predicate Types, LREC 2014: a Lexicon of predicate ARGUMENT-STRUCTURE frames — each `<roleset>` (e.g. `trade.01`) is a lexicographic SENSE numbering a verb/noun/adjective predicate's argument structure, Pustejovsky's (1995) own \"argument structure\" qualia role (the same justification `Lexicon`'s root doc cites), which is why this is a `Lexicon` leaf and NOT a `ControlledVocabulary` leaf (that family's own root doc explicitly excludes anything with \"senses/qualia\"). A many-file COLLECTION (one XML per lemma, `frameset → predicate → roleset → aliases`), archived via the SAME deterministic directory-archive codec as `VerbClassLexicon`'s collection — but, unlike VerbNet, lemma-KEYED rather than WordNet-sense-key-keyed: the source's `<alias pos=\"...\">` entries carry bare lemma+POS strings with NO native WordNet sense-key or synset reference (confirmed 2026-07-13: a grep across sample lemma files found zero machine-readable WordNet links, and the DTD's `rolelink`/`lexlink` cross-resource vocabulary observed in the data is `{AMR, FrameNet, PropBank, VerbNet}`, never `WordNet`), so resolution is live, at query time, against the loaded WordNet — FrameNet's precedent, not VerbNet's precomputed sense-key crosswalk. The cross-POS SIGNAL this lexicon grounds is same-roleset aliasing across DIFFERENT parts of speech (e.g. `trade.01` carries both a `v` alias `trade` and an `n` alias `trading` — a verb sense and a noun sense sharing one argument-structure frame), never a nested `<SUBCLASSES>` hierarchy the way VerbNet's classes nest."),
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
        RealizationFrameTable: ("en", "Realization-frame table",
            "A controlled table of natural-language-generation realization frames: each row pairs a frame id with a surface template carrying typed slots the realizer fills. Reiter & Dale (2000) *Building Natural Language Generation Systems* (CUP), the realization stage — a communicative goal is realized by filling a STORED linguistic template, not by concatenating a literal in code. A ControlledVocabulary leaf carried as `frame_id<TAB>template` rows (the abstain-frame surfaces for the honest-abstention doctrine, Reiter 1978 closed-world): loaded as data and interpreted, never a Rust `format!`. DERIVED/authored source-of-truth (not URL-fetchable): git-tracked, excluded from the published crate, shipped as the committed content-addressed `.prx`."),
        MathOperatorVocabulary: ("en", "Mathematical-operator vocabulary",
            "The closed-class inventory of mathematical operators — each glyph bound to one OpenMath symbol with its STS signature (role, arity, result sort). A ControlledVocabulary leaf authored as LMF-shaped XML (one operator per `<LexicalEntry>`, same reader as WordNet/function-words) but consumed as raw bytes into an `OperatorVocabulary`, NOT a WordNet graph. Cite: OpenMath Content Dictionaries `arith1` + `relation1` and OpenMath Standard 2.0 (Kohlhase & Rabe, eds., 2019) §2.1.4 (Role), §4.3 (STS signatures); ISO 80000-2:2019 (operator glyphs). DERIVED/authored source-of-truth (not URL-fetchable): git-tracked, excluded from the published crate, shipped as the committed content-addressed `.prx`."),
        ColorSchemeVocabulary: ("en", "Color-scheme vocabulary",
            "The Base16 / Base24 named color-scheme collection — a controlled set of named palettes, each binding the framework's reserved slot keys (`base00`..`base0F` for Base16; `base00`..`base17` for Base24) to sRGB hex values. The vocabulary is the Tinted Theming framework specification's scheme corpus (the community `tinted-schemes` dataset, github.com/tinted-theming/schemes), a published, bounded set of named color schemes distributed as one YAML file per scheme. A ControlledVocabulary leaf (ISO 25964-1 controlled terms — each scheme name is a controlled term; each slot key a reserved name): the substrate the theming validator scans to certify the praxis luminance-monotonicity and WCAG-AA contrast axioms over real-world palettes. A COLLECTION source — its `.prx` archives the whole directory of YAML schemes, not a single file. Cite: Tinted Theming, Base16 Styling Guidelines + Base24 specification, <https://github.com/tinted-theming/home>; W3C WCAG 2.1 §1.4.3 (contrast). FETCHED source: the i-am-logger/tinted-schemes fork is the fetch endpoint; the raw theme tree is gitignored after fetch, regenerated via `pr4xis update`, and only the committed content-addressed `.prx` ships."),
        SupertagCostTable: ("en", "Supertag cost table",
            "The CCG supertag/rule cost table — PUBLISHED CCGbank frequencies as negative-log costs (nats), the derivation-ranking floor of the weighted chart. Lexical-category counts from Hockenmaier (2003) *Data and Models for Statistical Parsing with CCG*, PhD thesis, Univ. of Edinburgh, Table 5.5 p. 176 (section 00 gold, 45,422 tokens); type-changing rule counts from Hockenmaier & Steedman (2005) *CCGbank User's Manual* MS-CIS-05-09, Table 4.3 p. 86 (sections 02-21, 929,552 tokens); normalizers (§8 p. 387) and the hapax default (439 hapax categories, §8.2 p. 389) from Hockenmaier & Steedman (2007) Computational Linguistics 33(3). Read-off published counts — not invented weights, not trained parameters (the Clark & Curran 2007 log-linear model is out of scope). A ControlledVocabulary leaf carried as kind-discriminated `lex`/`unary`/`default` TSV rows, loaded as data and interpreted by the generic min-cost fold (`lambek::supertag_costs`), never a Rust `match` of numbers. DERIVED source-of-truth (not URL-fetchable): git-tracked, excluded from the published crate, shipped as the committed content-addressed `.prx`."),
        QuoteGlyphVocabulary: ("en", "Quote-glyph vocabulary",
            "The Unicode quotation-mark glyph inventory — every code point in General_Category Pi (Initial_Punctuation, 12 members) and Pf (Final_Punctuation, 10 members), plus the two direction-ambiguous ASCII quote marks (U+0022 QUOTATION MARK, U+0027 APOSTROPHE — General_Category Po, carried separately per Unicode's own Quotation_Mark property listing since Po does not encode directionality). Unicode Standard Annex #44 (Unicode Character Database), General_Category property, as tabulated in the UCD's DerivedGeneralCategory.txt (verified against Unicode 18.0.0, 2026); each Pi glyph's paired Pf counterpart (where the UCD pairs one — 10 of the 12 Pi members pair with a Pf sibling; U+201B/U+201F have none) is carried as loaded data. A ControlledVocabulary leaf carried as `codepoint<TAB>role[<TAB>pairs_with]` rows: the closed glyph set a quoted-mention span recognizer (Slice B's do-support object-question frame) queries to recognize `open word... close` as a mentioned expression licensing an NP reading (Quine 1940:26 via SEP 'Quotation' §3.1), never an inline `is_ascii_punctuation`/hardcoded-char check. DERIVED/authored source-of-truth (not URL-fetchable): git-tracked, excluded from the published crate, shipped as the committed content-addressed `.prx`."),
        DashPunctuationVocabulary: ("en", "Dash-punctuation vocabulary",
            "The Unicode dash-punctuation glyph inventory — every code point in General_Category Pd (Dash_Punctuation, 25 members: U+002D HYPHEN-MINUS through U+10EAD YEZIDI HYPHENATION MARK). Unicode Standard Annex #44 (Unicode Character Database), General_Category property, verified against Unicode 18.0.0 (2026) via compart.com/en/unicode/category/Pd, cross-referenced against the UCD's own DerivedGeneralCategory.txt tabulation. Unlike QuoteGlyphVocabulary's paired Pi/Pf glyphs, Pd carries no pairing/nesting structure, so this leaf is a flat `codepoint<TAB>name` table: the closed glyph set the tokenizer's `flush_word` (lambek::tokenize) queries to trim a trailing/leading Unicode dash (e.g. the EM DASH U+2014 ending 'the term X means—' in real U.S. Code drafting) off an accumulated word buffer — the fix for a confirmed bug where the raw `char::is_ascii_punctuation()` check silently left `means—` untrimmed, missing lexicon lookup for the bare word `means`. DERIVED/authored source-of-truth (not URL-fetchable): git-tracked, excluded from the published crate, shipped as the committed content-addressed `.prx`."),
        EnglishGraphemeClasses: ("en", "English grapheme classes",
            "The English orthographic grapheme classes — the 26 basic-Latin letters partitioned into vowel letters (a e i o u), consonant letters, and the single dual-function letter <y>. Carney, Edward (1994), A Survey of English Spelling (London: Routledge), Ch.1 — the standard descriptive survey of English orthography, which sets out the letter inventory used by English and treats <y> as dual (consonant word/syllable-initially, 'yes'; vowel elsewhere, 'baby', 'boy'). Cross-grounded by Quirk, Greenbaum, Leech & Svartvik (1985), A Comprehensive Grammar of the English Language (London: Longman) §3.21, whose orthographic plural rule is stated over exactly this partition and turns on <y> patterning as a vowel in 'boy' -> 'boys'. Deliberately NOT a Unicode/UCD derivation: the UCD classifies these code points only as General_Category=Ll/Lu and encodes nothing about vowel-hood, which is a language-specific orthographic fact, so this leaf is authored from the descriptive literature rather than mechanically filtered from a primary data file. A ControlledVocabulary leaf carried as `letter<TAB>class<TAB>note` rows: the alphabet the English morphological GENERATION rules (morphology::english::generation — Quirk et al. 1985 §3 C-V-C doubling, §3.21 plural; Spencer 1991 §5.2 silent-e elision) range over, replacing a hardcoded `matches!(c, 'a'|'e'|'i'|'o'|'u')` that was duplicated across generation.rs and irregular.rs with <y>'s dual status scattered beside it as ad-hoc `|| c == 'y'` tests. <y> is carried as class Dual rather than forced into one bucket because the consuming rules need different readings, exposed as two named queries (is_syllabic / is_consonantal) instead of a lossy boolean. AUTHORED source-of-truth (not URL-fetchable): git-tracked, excluded from the published crate, shipped as the committed content-addressed `.prx`."),
        CaseFoldingTable: ("en", "Case-folding table",
            "The Unicode simple case-folding table — every codepoint whose CaseFolding.txt status is C (common, simple and full mappings agree) or S (simple-only, where it differs from full folding), each mapped to its single-codepoint fold target (status F, full folding that grows the string, and status T, Turkic-locale dotted/dotless I, are excluded — the former is not fixed-width, the latter is not used by default per the source file's own usage note). Unicode Standard Annex #44 (Unicode Character Database), CaseFolding.txt, verified against Unicode 18.0.0 (2026-02-03) by direct inspection of the primary data file. A ControlledVocabulary leaf carried as `codepoint<TAB>folded<TAB>name` rows: the fold-on-miss lexicon lookup's substrate (Slice D, `.notes/chat-fix-c-build-state.md`) — a query whose exact-case WordIndex lookup misses is re-tried through this table's fold, recovering a differently-cased loaded surface ('Section Eight', 'Turkish bath', 'O.K.') the tokenizer's lowercasing would otherwise discard, never a bare `str::to_lowercase` call. DERIVED/authored source-of-truth (not URL-fetchable): git-tracked, excluded from the published crate, shipped as the committed content-addressed `.prx`."),
        AssociativeConceptTable: ("en", "Associative concept table",
            "Speer, R., Chin, J. & Havasi, C. (2017) ConceptNet 5.5: An Open Multilingual Graph of General Knowledge, Proceedings of AAAI 2017: a crowd- and dataset-sourced commonsense semantic network of `(relation, start, end)` assertions (RelatedTo, IsA, PartOf, UsedFor, CapableOf, …, 34 relation types over the English portion of ConceptNet 5.7.0). No official English-only or WordNet-linked subset is distributed, so the loaded table is a MECHANICAL filter of the full 34,074,917-row assertions CSV (s3.amazonaws.com/conceptnet/downloads/2019/edges/conceptnet-assertions-5.7.0.csv.gz) down to the 932,948 rows whose start AND end concepts both resolve to a lemma in the loaded WordNet (english-wordnet-2025.xml) — a pure format-conversion (raw relation/start/end/weight fields carried through unchanged), not an interpretive adaptation. A ControlledVocabulary leaf carried as `relation<TAB>start_lemma<TAB>end_lemma<TAB>weight` rows. Data licensed CC BY-SA 4.0 (conceptnet5/LICENSE.txt); the bundled `conceptnet-LICENSE.txt` carries the license text and the project's suggested attribution. Independent of both WordNet's own gloss/hypernym construction and VerbNet's syntactic-alternation classification (Levin 1993) — a genuinely separate, third signal for concept relatedness, mapped generically onto the existing `Association` relation kind (SKOS `related`) rather than a fine-grained per-relation-type mapping."),
        FrameSemanticTable: ("en", "Frame-semantic table",
            "Baker, C. F., Fillmore, C. J. & Lowe, J. B. (1998) The Berkeley FrameNet Project, Proceedings of COLING-ACL 1998; Ruppenhofer, J., Ellsworth, M., Petruck, M. R. L., Johnson, C. R. & Scheffczyk, J. (2016) FrameNet II: Extended Theory and Practice, ICSI: FrameNet 1.7's lexical units (lemma + POS + evoked semantic frame, e.g. `cause.v` evokes `Causation`) and frame-to-frame relations (9 types: Inheritance, Using, Subframe, Perspective_on, Causative_of, Inchoative_of, Precedes, Metaphor, See_also). No official small subset is distributed either — the `lu/` directory alone is ~13,574 files / ~84 MB compressed inside the official release (NLTK-mirrored `framenet_v17.zip`, SHA256 `22f6aad6fb799ba4dbed0440714e1118442ad7d7345351de37428581284f471c`) — so the loaded table is a MECHANICAL extraction of each lexical unit's root `<lexUnit POS=\"...\" name=\"...\" frame=\"...\">` start-tag attributes plus `frRelation.xml`'s relation rows, discarding each LU file's embedded annotated-sentence corpus this project has no use for. A ControlledVocabulary leaf carried as type-tagged `LU<TAB>lemma<TAB>pos<TAB>frame` / `REL<TAB>relation<TAB>sub_frame<TAB>super_frame` rows. Data licensed CC BY 3.0 Unported; the bundled `framenet-LICENSE.txt` carries the license text and attribution. Independent of WordNet's own gloss/hypernym construction, VerbNet's syntactic-alternation classification, and ConceptNet's crowd-sourced association graph — a third, genuinely separate signal for concept relatedness, mapped generically onto the existing `Association` relation kind (SKOS `related`) rather than a fine-grained per-relation-type mapping."),
        SumoWordNetMappingTable: ("en", "SUMO WordNet-mapping table",
            "Niles, I. & Pease, A. (2001) Towards a Standard Upper Ontology, Proceedings of the 2nd International Conference on Formal Ontology in Information Systems (FOIS 2001), pp. 2-9; Niles, I. & Pease, A. (2003) Linking Lexicons and Ontologies: Mapping WordNet to the Suggested Upper Merged Ontology, Proceedings of the IEEE International Conference on Information and Knowledge Engineering (IKE 2003), pp. 412-416: the WordNet↔SUMO correspondence table — for each Princeton WordNet 3.0 synset, the SUMO upper-ontology term(s) it maps to, each tagged with the relation the source's own legend defines by suffix: `=` equivalence, `+` subsumption (is-a), `@` instance, and the three complements `:`/`[`/`]` (the synset is explicitly NOT that class). The source `WordNetMappings30-{noun,verb,adj,adv}.txt` files are annotated COPIES of the WordNet 3.0 `data.*` files (each data line is a full WNDB synset record with `&%<term><suffix>` tokens appended), ~22 MB across the four files, so the loaded table is a MECHANICAL extraction of each annotation, RESOLVED to this project's `ConceptId`. SUMO's PWN-3.0 synset offsets do NOT match Open English WordNet 2025 (only ~385 of 82,115 noun offsets collide), so — exactly like VerbNet — the regen parses the OEWN XML once, offline, and resolves each synset via the version-stable WordNet SENSE KEY (reconstructed from the WNDB record's member words + `lex_filenum`), baking the `ConceptId` value (stable across load paths) into the committed table. Verified 2026-07-14 against ontologyportal/sumo `master` (commit 152b9abc440477073b5e4e573983b730a619da7b): 116,007 annotated synsets resolve to 91,088 (90,478 via sense key, 610 via offset; 24,919 dropped as unmapped — largely satellite adjectives whose sense keys need a head word), yielding 91,218 `(concept, term, relation)` rows after sort+dedup, over 90,490 distinct concepts. A ControlledVocabulary leaf — a flat concept↔term correspondence table, NOT a structured lexicon — carried as `concept_value<TAB>term<TAB>relation_code` rows (`EQ`/`SUB`/`INST`/`CEQ`/`CSUB`/`CINST`). Only the direct crosswalk is loaded (SUMO's Merge.kif upper-ontology hierarchy is NOT), so matching is flat same-term intersection with the `Complement*` relations excluded (two synsets both being NOT-the-same-class is not a similarity witness). A fourth, genuinely separate signal for concept relatedness, independent of WordNet's own hypernym graph, VerbNet's syntactic-alternation classes, ConceptNet's crowd-sourced associations, and FrameNet's frame semantics. Data licensed GNU General Public License (GPL) — a THIRD-PARTY license, distinct from and not superseded by this repository's own CC BY-NC-SA 4.0 license (see LICENSE); this file is bundled and redistributed under its own GPL terms."),
        GazetteerTable: ("en", "Gazetteer table",
            "GeoNames.org countryInfo.txt — <https://download.geonames.org/export/dump/countryInfo.txt>: one row per ISO 3166-1 country/territory, tab-separated `ISO<TAB>ISO3<TAB>ISO-Numeric<TAB>fips<TAB>Country<TAB>Capital<TAB>Area(in sq km)<TAB>Population<TAB>Continent<TAB>tld<TAB>CurrencyCode<TAB>CurrencyName<TAB>Phone<TAB>PostalCodeFormat<TAB>PostalCodeRegex<TAB>Languages<TAB>geonameid<TAB>neighbours<TAB>EquivalentFipsCode`, preceded by ~50 `#`-prefixed header comment lines (praxis-TSV convention). The `Country`/`Capital` columns give the `capital_of` fact (`natural::geography`'s `EveryCountryHasExactlyOneCapital`, some territories carry an empty `Capital`); `Continent` (a 2-letter code: EU/AS/AF/NA/SA/OC/AN) gives continent-contains-country (`RegionContainmentIsRcc8ProperPart`); `neighbours` (comma-separated ISO codes, empty for islands) gives country-borders-country (`BordersIsRcc8ExternalConnection`). Data licensed Creative Commons Attribution (CC BY) 4.0 — GeoNames.org's own export terms (<https://www.geonames.org/export/>) require attribution: \"You should give credit to GeoNames when using data or web services with a link or another reference to GeoNames.\" A ControlledVocabulary leaf carried through the generic praxis-TSV decoder (`decoders::plaintext_tsv`), the same codec as `AssociativeConceptTable`/`FrameSemanticTable`/`SumoWordNetMappingTable`. BUNDLED source-of-truth: a single ~31 KB HTTP-fetchable file, git-tracked, EXCLUDED from the published crate by the `[package].include` allowlist, present on a fresh checkout so `pr4xis update` verifies it against the pin instead of re-fetching (re-sync deliberately via `pr4xis update geonames_countryinfo --lock`). Shipped as the committed content-addressed `.prx`."),
    },

    is_a: [
        // Lexicon family
        (Lexicon, Source),
        (Language, Lexicon),
        (DomainLexicon, Lexicon),
        (LegalLexicon, DomainLexicon),
        // The two Caregiver AI Challenge chat lexicons — DomainLexicon
        // siblings of LegalLexicon (Pustejovsky 1995: specialty-domain
        // scoping), deliberately SEPARATE leaves so chat residency can
        // select definitional lexicons BY KIND without ever loading the
        // closed-class statutory recognizer vocabulary (LegalLexicon).
        (CaregivingLexicon, DomainLexicon),
        (HcbsComplianceLexicon, DomainLexicon),
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
        // VerbNet (Kipper et al. 2008) — a Lexicon on the syntactic-semantic
        // classification axis (verbs grouped by Levin 1993 alternations into
        // a nested class hierarchy), sibling of AGID/CatVar under Lexicon.
        (VerbClassLexicon, Lexicon),
        // PropBank (Palmer, Gildea & Kingsbury 2005) — a Lexicon on the
        // predicate argument-structure axis (rolesets numbering a
        // predicate's sense-specific argument frame), sibling of
        // VerbClassLexicon under Lexicon: lemma-keyed with live cross-POS
        // roleset aliasing rather than a nested WordNet-sense-key-keyed
        // class hierarchy.
        (PredicateArgumentLexicon, Lexicon),

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
        (RealizationFrameTable, ControlledVocabulary),
        (MathOperatorVocabulary, ControlledVocabulary),
        (ColorSchemeVocabulary, ControlledVocabulary),
        (SupertagCostTable, ControlledVocabulary),
        (QuoteGlyphVocabulary, ControlledVocabulary),
        (DashPunctuationVocabulary, ControlledVocabulary),
        (EnglishGraphemeClasses, ControlledVocabulary),
        (CaseFoldingTable, ControlledVocabulary),
        (AssociativeConceptTable, ControlledVocabulary),
        (FrameSemanticTable, ControlledVocabulary),
        (SumoWordNetMappingTable, ControlledVocabulary),
        (GazetteerTable, ControlledVocabulary),
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

        // Solan (1993), same grounding as LegalLexicon: the caregiving /
        // HCBS-compliance definitional lexicons gloss their statutory and
        // regulatory terms of art in common English — every gloss anchors
        // through WordNet senses.
        (CaregivingLexicon, Language, Adjoins),
        (HcbsComplianceLexicon, Language, Adjoins),

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
        "CaregivingLexicon" => C::CaregivingLexicon,
        "HcbsComplianceLexicon" => C::HcbsComplianceLexicon,
        "SchemaVocabulary" => C::SchemaVocabulary,
        "ClosedClassLexicon" => C::ClosedClassLexicon,
        "InflectionLexicon" => C::InflectionLexicon,
        "DerivationalLexicon" => C::DerivationalLexicon,
        "VerbClassLexicon" => C::VerbClassLexicon,
        "PredicateArgumentLexicon" => C::PredicateArgumentLexicon,
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
        "RealizationFrameTable" => C::RealizationFrameTable,
        "MathOperatorVocabulary" => C::MathOperatorVocabulary,
        "ColorSchemeVocabulary" => C::ColorSchemeVocabulary,
        "SupertagCostTable" => C::SupertagCostTable,
        "QuoteGlyphVocabulary" => C::QuoteGlyphVocabulary,
        "DashPunctuationVocabulary" => C::DashPunctuationVocabulary,
        "EnglishGraphemeClasses" => C::EnglishGraphemeClasses,
        "CaseFoldingTable" => C::CaseFoldingTable,
        "AssociativeConceptTable" => C::AssociativeConceptTable,
        "FrameSemanticTable" => C::FrameSemanticTable,
        "SumoWordNetMappingTable" => C::SumoWordNetMappingTable,
        "GazetteerTable" => C::GazetteerTable,
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
        C::CaregivingLexicon => "CaregivingLexicon",
        C::HcbsComplianceLexicon => "HcbsComplianceLexicon",
        C::SchemaVocabulary => "SchemaVocabulary",
        C::ClosedClassLexicon => "ClosedClassLexicon",
        C::InflectionLexicon => "InflectionLexicon",
        C::DerivationalLexicon => "DerivationalLexicon",
        C::VerbClassLexicon => "VerbClassLexicon",
        C::PredicateArgumentLexicon => "PredicateArgumentLexicon",
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
        C::RealizationFrameTable => "RealizationFrameTable",
        C::MathOperatorVocabulary => "MathOperatorVocabulary",
        C::ColorSchemeVocabulary => "ColorSchemeVocabulary",
        C::SupertagCostTable => "SupertagCostTable",
        C::QuoteGlyphVocabulary => "QuoteGlyphVocabulary",
        C::DashPunctuationVocabulary => "DashPunctuationVocabulary",
        C::EnglishGraphemeClasses => "EnglishGraphemeClasses",
        C::CaseFoldingTable => "CaseFoldingTable",
        C::AssociativeConceptTable => "AssociativeConceptTable",
        C::FrameSemanticTable => "FrameSemanticTable",
        C::SumoWordNetMappingTable => "SumoWordNetMappingTable",
        C::GazetteerTable => "GazetteerTable",
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
            // The two caregiving definitional lexicons carry statutory /
            // regulatory glosses ABOUT primary rules — secondary, exactly
            // as LegalLexicon (Hart 1961 §V: rules about rules).
            C::Regulation
            | C::CaseLaw
            | C::LegalLexicon
            | C::CaregivingLexicon
            | C::HcbsComplianceLexicon => HartRuleKind::Secondary,
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
        let secondary = [
            C::Regulation,
            C::CaseLaw,
            C::LegalLexicon,
            C::CaregivingLexicon,
            C::HcbsComplianceLexicon,
        ];
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
        "Statute (+ UsFederalStatute) / ConstitutionalArticle / ProceduralRule are Primary; Regulation / CaseLaw / LegalLexicon / CaregivingLexicon / HcbsComplianceLexicon are Secondary",
        "Hart (1961) The Concept of Law §V"
    );
}

pr4xis::register_axiom!(
    PrimarySecondaryDistinction,
    "Hart (1961) The Concept of Law §V"
);

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;

// WordNet Lexical Markup Framework (LMF) ontology.
//
// LMF is an XML application (schema) for encoding lexical databases.
// It extends the XML ontology with lexical meaning — what Synset, LexicalEntry,
// Sense, and SenseRelation MEAN, not just that they're XML elements.
//
// Reference: Global Wordnet Association, WN-LMF 1.3
// https://globalwordnet.github.io/schemas/
//
// # rkyv-serializability under `prx`
//
// The graph-faithful `.prx` envelope
// ([`WordNetPrxEnvelope`](super::prx::WordNetPrxEnvelope)) carries this typed
// `WordNet` model — the ontology — directly, so it must be rkyv-serializable
// under the `prx` feature (the same feature that links rkyv and compiles the
// envelope). Every plain-data type here therefore carries a CFG-GATED rkyv
// derive (`#[cfg_attr(feature = "prx", derive(rkyv::Archive, …))]`): present in
// the `prx`/`fetch` build where the archive consumes it, ABSENT from the
// default + wasm32 builds where rkyv is not linked (rkyv is an OPTIONAL dep,
// `prx = ["dep:rkyv", …]`). An unconditional derive would fail those builds; a
// gated one keeps them clean while making the model the archive's ontology
// payload — the same `cfg_attr(feature = "prx")` discipline
// [`RoundTripFidelity`](crate::formal::meta::well_behaved_lens::RoundTripFidelity)
// already uses. No Owned mirror is needed: these are flat data structs the
// derive handles directly.

/// A synset — a set of words sharing the same meaning (a concept).
/// This is the fundamental unit of WordNet: not a word, but a MEANING.
///
/// Content model `<!ELEMENT Synset (Definition*, ILIDefinition?,
/// SynsetRelation*, Example*)>` (DTD line 98); attributes per
/// `<!ATTLIST Synset>` (DTD lines 99-122).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Synset {
    pub id: String,
    pub ili: Option<String>,
    pub pos: LmfPos,
    pub members: Vec<String>,
    pub definitions: Vec<String>,
    /// The single optional `<ILIDefinition>` — the Interlingual Index
    /// gloss proposed for a not-yet-mapped synset (`ILIDefinition?` in
    /// the content model, DTD line 98; element decl line 145). Present
    /// ×3184 in Open English WordNet 2025; previously dropped.
    pub ili_definition: Option<String>,
    pub examples: Vec<String>,
    pub relations: Vec<SynsetRelation>,
    /// `<!ATTLIST Synset lexfile CDATA #IMPLIED>` (DTD line 122) — the
    /// legacy WordNet lexicographer-file name (e.g. `noun.animal`).
    pub lexfile: Option<String>,
    /// `<!ATTLIST Synset dc:source CDATA #IMPLIED>` (DTD line 113).
    pub dc_source: Option<String>,
    /// `<!ATTLIST Synset confidenceScore CDATA #IMPLIED>` (DTD line 119).
    pub confidence_score: Option<String>,
}

/// A lexical entry — a word with its senses (connections to synsets).
///
/// Content model `<!ELEMENT LexicalEntry (Lemma, Form*, Sense*,
/// SyntacticBehaviour*)>` (DTD line 33).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct LexicalEntry {
    pub id: String,
    pub lemma: Lemma,
    pub senses: Vec<Sense>,
    pub forms: Vec<Form>,
    /// `<SyntacticBehaviour>` children declared on this entry — the
    /// per-entry placement of subcategorization frames (the WN-LMF 1.3
    /// content model allows them on `LexicalEntry` as well as on
    /// `Lexicon`; DTD lines 33 + 228). Present ×39 in Open English
    /// WordNet 2025 (lexicon-level there, but the reader/writer carry
    /// them faithfully wherever the DTD permits them).
    pub syntactic_behaviours: Vec<SyntacticBehaviour>,
}

/// A lemma — the canonical form of a word.
///
/// Content model `<!ELEMENT Lemma (Pronunciation*, Tag*)>` (DTD line
/// 53); attributes `writtenForm`, `script`, `partOfSpeech` (DTD lines
/// 54-57).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Lemma {
    pub written_form: String,
    pub pos: LmfPos,
    /// `<!ATTLIST Lemma script CDATA #IMPLIED>` (DTD line 56) — the
    /// ISO 15924 script code when the written form is non-Latin.
    pub script: Option<String>,
    /// `<Pronunciation>` children (`Pronunciation*` in the content
    /// model, DTD line 53). Present ×43534 in Open English WordNet
    /// 2025 — the single largest class of element the reader dropped.
    pub pronunciations: Vec<Pronunciation>,
}

/// A pronunciation of a lemma or form.
///
/// `<!ELEMENT Pronunciation (#PCDATA)>` with `<!ATTLIST Pronunciation
/// xml:space (default|preserve) "default" variety CDATA #IMPLIED
/// notation CDATA #IMPLIED phonemic (true|false) "true" audio CDATA
/// #IMPLIED>` (DTD lines 63-69). The text is the transcription itself
/// (e.g. an IPA string).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Pronunciation {
    /// The `#PCDATA` transcription text (e.g. `/dɒɡ/`).
    pub text: String,
    /// `variety CDATA #IMPLIED` — the regional variety the
    /// transcription is for (e.g. `GB`, `US`).
    pub variety: Option<String>,
    /// `notation CDATA #IMPLIED` — the transcription notation (e.g.
    /// `ipa`).
    pub notation: Option<String>,
    /// `phonemic (true|false) "true"` — `true` iff the transcription is
    /// phonemic (broad) rather than phonetic (narrow). `None` when the
    /// attribute is absent (the reader does not synthesize the DTD
    /// default, so an absent attribute round-trips as absent).
    pub phonemic: Option<String>,
    /// `audio CDATA #IMPLIED` — a reference to an audio rendering.
    pub audio: Option<String>,
}

/// A sense — the connection between a word and a meaning (synset).
///
/// Content model `<!ELEMENT Sense (SenseRelation*, Example*, Count*)>`
/// (DTD line 74); attributes per `<!ATTLIST Sense>` (DTD lines 75-97).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Sense {
    pub id: String,
    pub synset: String,
    pub relations: Vec<SenseRelation>,
    /// Subcategorization frame IDs (verb frames for transitivity).
    /// From LMF `subcat` attribute. E.g., ["vtai", "vtaa"] for transitive.
    pub subcat: Vec<String>,
    /// `<!ATTLIST Sense adjposition (a|ip|p) #IMPLIED>` (DTD line 96) —
    /// adjective syntactic position (attributive / immediately
    /// postnominal / predicative) for adjective senses.
    pub adjposition: Option<String>,
    /// `<!ATTLIST Sense dc:source CDATA #IMPLIED>` (DTD line 88).
    pub dc_source: Option<String>,
    /// `<Count>` children (`Count*` in the content model, DTD line 74;
    /// element decl line 233) — corpus frequency counts for this sense.
    pub counts: Vec<Count>,
}

/// A corpus frequency count for a sense.
///
/// `<!ELEMENT Count (#PCDATA)>` (DTD line 233) — the count value is the
/// `#PCDATA` text; the dc:* / status / note attrs (DTD lines 234-252)
/// are carried as a flat string→string map preserving their declared
/// names.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Count {
    /// The `#PCDATA` count value (a non-negative integer as text).
    pub value: String,
}

/// A subcategorization-frame declaration.
///
/// `<!ELEMENT SyntacticBehaviour EMPTY>` with `<!ATTLIST
/// SyntacticBehaviour id ID #IMPLIED subcategorizationFrame CDATA
/// #REQUIRED senses IDREFS #IMPLIED>` (DTD lines 228-232). Present ×39
/// in Open English WordNet 2025; previously dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct SyntacticBehaviour {
    /// `id ID #IMPLIED` — the frame's optional identifier.
    pub id: Option<String>,
    /// `subcategorizationFrame CDATA #REQUIRED` — the frame template
    /// string (e.g. `Somebody ----s something`).
    pub subcategorization_frame: String,
    /// `senses IDREFS #IMPLIED` — the sense ids this frame applies to
    /// (whitespace-separated IDREFS).
    pub senses: Vec<String>,
}

/// A morphological form — an inflected variant of a word.
///
/// Content model `<!ELEMENT Form (Pronunciation*, Tag*)>` (DTD line
/// 58); attributes `id`, `writtenForm`, `script` (DTD lines 59-62).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Form {
    pub written_form: String,
    /// `<!ATTLIST Form id ID #IMPLIED>` (DTD line 60).
    pub id: Option<String>,
    /// `<!ATTLIST Form script CDATA #IMPLIED>` (DTD line 62).
    pub script: Option<String>,
    /// `<Pronunciation>` children (`Pronunciation*`, DTD line 58).
    pub pronunciations: Vec<Pronunciation>,
}

/// Synset-level relation (between concepts).
/// These map directly to our reasoning ontology:
/// - hypernym → TaxonomyDef (child is-a parent)
/// - meronym → MereologyDef (whole has-a part)
/// - antonym → OppositionDef
/// - causes → CausalDef
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct SynsetRelation {
    pub rel_type: SynsetRelationType,
    pub target: String,
}

/// Sense-level relation (between word senses).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct SenseRelation {
    pub rel_type: SenseRelationType,
    pub target: String,
}

/// Types of synset-level relations in WordNet.
///
/// Covers the Global WordNet Association LMF schema (Vossen et al.)
/// — synset relations in WordNet 2025 fall into these categories.
/// See <https://globalwordnet.github.io/schemas/>.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum SynsetRelationType {
    Hypernym,
    InstanceHypernym,
    Hyponym,
    InstanceHyponym,
    HoloMember,
    HoloPart,
    HoloSubstance,
    MeroMember,
    MeroPart,
    MeroSubstance,
    Causes,
    IsCausedBy,
    Entails,
    IsEntailedBy,
    Similar,
    Also,
    Attribute,
    DomainTopic,
    HasDomainTopic,
    DomainRegion,
    HasDomainRegion,
    Exemplifies,
    IsExemplifiedBy,
    Participle,
    /// Any WN-LMF 1.3 `relType` the typed model does not give a named
    /// variant — the long tail of the ~70-value DTD enumeration
    /// (`<!ATTLIST SynsetRelation relType (…) #REQUIRED>`, DTD lines
    /// 188-189: `co_agent_instrument`, `holo_location`, `feminine`,
    /// `antonym`, …). Carries the ACTUAL source string so
    /// [`as_str`](Self::as_str) reproduces it byte-for-byte, making
    /// `parse`/`as_str` a true inverse over the whole enumeration
    /// (previously this was `Other(u8)`, which discarded the string and
    /// re-emitted the placeholder `"other"` — a value loss).
    Other(String),
}

impl SynsetRelationType {
    pub fn parse(s: &str) -> Self {
        match s {
            "hypernym" => Self::Hypernym,
            "instance_hypernym" => Self::InstanceHypernym,
            "hyponym" => Self::Hyponym,
            "instance_hyponym" => Self::InstanceHyponym,
            "holo_member" => Self::HoloMember,
            "holo_part" => Self::HoloPart,
            "holo_substance" => Self::HoloSubstance,
            "mero_member" => Self::MeroMember,
            "mero_part" => Self::MeroPart,
            "mero_substance" => Self::MeroSubstance,
            "causes" => Self::Causes,
            "is_caused_by" => Self::IsCausedBy,
            "entails" => Self::Entails,
            "is_entailed_by" => Self::IsEntailedBy,
            "similar" => Self::Similar,
            "also" => Self::Also,
            "attribute" => Self::Attribute,
            "domain_topic" => Self::DomainTopic,
            "has_domain_topic" => Self::HasDomainTopic,
            "domain_region" => Self::DomainRegion,
            "has_domain_region" => Self::HasDomainRegion,
            "exemplifies" => Self::Exemplifies,
            "is_exemplified_by" => Self::IsExemplifiedBy,
            "participle" => Self::Participle,
            other => Self::Other(other.to_string()),
        }
    }

    /// The WN-LMF `relType` string for this relation — the exact
    /// inverse of [`parse`](Self::parse) for EVERY variant, named or
    /// not (`parse(x.as_str()) == x`).
    ///
    /// The `Other(s)` arm now reproduces the original source string `s`
    /// captured at `parse` time, so a long-tail `relType` the typed
    /// model has no named variant for (e.g. `co_agent_instrument`,
    /// `feminine`) survives the round-trip byte-for-byte. This makes
    /// `as_str` a total inverse over the whole DTD enumeration.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Hypernym => "hypernym",
            Self::InstanceHypernym => "instance_hypernym",
            Self::Hyponym => "hyponym",
            Self::InstanceHyponym => "instance_hyponym",
            Self::HoloMember => "holo_member",
            Self::HoloPart => "holo_part",
            Self::HoloSubstance => "holo_substance",
            Self::MeroMember => "mero_member",
            Self::MeroPart => "mero_part",
            Self::MeroSubstance => "mero_substance",
            Self::Causes => "causes",
            Self::IsCausedBy => "is_caused_by",
            Self::Entails => "entails",
            Self::IsEntailedBy => "is_entailed_by",
            Self::Similar => "similar",
            Self::Also => "also",
            Self::Attribute => "attribute",
            Self::DomainTopic => "domain_topic",
            Self::HasDomainTopic => "has_domain_topic",
            Self::DomainRegion => "domain_region",
            Self::HasDomainRegion => "has_domain_region",
            Self::Exemplifies => "exemplifies",
            Self::IsExemplifiedBy => "is_exemplified_by",
            Self::Participle => "participle",
            // The original source string, captured at `parse` time —
            // reproduced exactly so the long-tail relType round-trips.
            Self::Other(s) => s.as_str(),
        }
    }

    /// Is this a taxonomy (is-a) relation?
    pub fn is_taxonomy(&self) -> bool {
        matches!(self, Self::Hypernym | Self::InstanceHypernym)
    }

    /// Is this a mereology (has-a) relation?
    pub fn is_mereology(&self) -> bool {
        matches!(
            self,
            Self::HoloMember
                | Self::HoloPart
                | Self::HoloSubstance
                | Self::MeroMember
                | Self::MeroPart
                | Self::MeroSubstance
        )
    }

    /// Is this a causal relation?
    pub fn is_causal(&self) -> bool {
        matches!(self, Self::Causes | Self::Entails)
    }
}

/// Types of sense-level relations.
///
/// Per Global WordNet Association LMF schema; documented in
/// Fellbaum (1998) Ch. 1 + Ch. 5, Fellbaum-Osherson-Clark (2009)
/// for `derivation` morphosemantic links.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum SenseRelationType {
    Antonym,
    Similar,
    Pertainym,
    Derivation,
    Also,
    Exemplifies,
    IsExemplifiedBy,
    Participle,
    /// Any WN-LMF 1.3 `SenseRelation/relType` the typed model does not
    /// name (`<!ATTLIST SenseRelation relType (…) #REQUIRED>`, DTD
    /// lines 209-210: `domain_topic`, `simple_aspect_ip`, `feminine`,
    /// …). Carries the ACTUAL source string so
    /// [`as_str`](Self::as_str) reproduces it byte-for-byte (previously
    /// `Other(u8)`, which discarded it).
    Other(String),
}

impl SenseRelationType {
    pub fn parse(s: &str) -> Self {
        match s {
            "antonym" => Self::Antonym,
            "similar" => Self::Similar,
            "pertainym" => Self::Pertainym,
            "derivation" => Self::Derivation,
            "also" => Self::Also,
            "exemplifies" => Self::Exemplifies,
            "is_exemplified_by" => Self::IsExemplifiedBy,
            "participle" => Self::Participle,
            other => Self::Other(other.to_string()),
        }
    }

    /// The WN-LMF `relType` string for this sense relation — the exact
    /// inverse of [`parse`](Self::parse) for EVERY variant, named or
    /// not (`parse(x.as_str()) == x`).
    ///
    /// As with [`SynsetRelationType::as_str`], the `Other(s)` arm now
    /// reproduces the original source string captured at `parse` time,
    /// so the long-tail `relType` round-trips byte-for-byte.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Antonym => "antonym",
            Self::Similar => "similar",
            Self::Pertainym => "pertainym",
            Self::Derivation => "derivation",
            Self::Also => "also",
            Self::Exemplifies => "exemplifies",
            Self::IsExemplifiedBy => "is_exemplified_by",
            Self::Participle => "participle",
            Self::Other(s) => s.as_str(),
        }
    }

    /// Is this an opposition (antonym) relation?
    pub fn is_opposition(&self) -> bool {
        matches!(self, Self::Antonym)
    }
}

/// LMF part-of-speech tags.
///
/// Extended beyond WordNet's 4 open-class tags (n, v, a, r) to include
/// closed-class function words, per Universal Dependencies and OLiA.
///
/// References:
/// - WordNet-LMF: n, v, a, s, r
/// - Universal Dependencies: DET, PRON, ADP, CCONJ, SCONJ, PART, AUX, INTJ
/// - OLiA: Determiner, Pronoun, Copula, Auxiliary, Preposition, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum LmfPos {
    // Open class (WordNet)
    Noun,
    Verb,
    Adjective,
    /// Satellite adjective — the WN-LMF `s` `partOfSpeech` value
    /// (`<!ATTLIST Lemma partOfSpeech (n|v|a|r|s|t|c|p|x|u)>`, DTD
    /// line 57). A satellite adjective is an adjective in a cluster
    /// whose head is its `similar` adjective (Fellbaum 1998 §1.5);
    /// it is semantically an adjective but the source tag is the
    /// DISTINCT byte `s`, not `a`. Carrying it as its own variant
    /// keeps [`parse`](Self::parse) / [`to_tag`](Self::to_tag) an
    /// exact bijection on the DTD enumeration, so the source byte
    /// round-trips losslessly (it previously collapsed into
    /// `Adjective` and re-emitted as `a` — a byte loss).
    SatelliteAdjective,
    Adverb,
    // Closed class (function words)
    Determiner,
    Pronoun,
    Preposition,
    Conjunction,
    Particle,
    Copula,
    Auxiliary,
    Interjection,
    Numeral,
    Other,
}

impl LmfPos {
    /// Parse from LMF/Universal Dependencies POS tag string.
    ///
    /// On the WN-LMF 1.3 `partOfSpeech` enumeration
    /// (`n|v|a|r|s|t|c|p|x|u`, DTD line 57) this is the exact inverse
    /// of [`to_tag`](Self::to_tag) for every named WordNet tag — in
    /// particular `s` (satellite adjective) maps to its own
    /// [`SatelliteAdjective`](Self::SatelliteAdjective) variant rather
    /// than collapsing into `Adjective`, so the source byte survives a
    /// round-trip. The remaining WN-LMF-specific tags `t`/`c`/`p`/`x`/`u`
    /// (terminology / closed-class compound / preposition / other /
    /// unknown — narrow WN-LMF semantics the typed model does not yet
    /// distinguish) project to [`Other`](Self::Other).
    pub fn parse(s: &str) -> Self {
        match s {
            // WordNet open class
            "n" => Self::Noun,
            "v" => Self::Verb,
            "a" => Self::Adjective,
            "s" => Self::SatelliteAdjective,
            "r" => Self::Adverb,
            // Closed class (Universal Dependencies / OLiA tags)
            "det" | "d" => Self::Determiner,
            "pron" => Self::Pronoun,
            "adp" | "prep" => Self::Preposition,
            "cconj" | "sconj" | "conj" => Self::Conjunction,
            "part" => Self::Particle,
            "cop" => Self::Copula,
            "aux" => Self::Auxiliary,
            "intj" => Self::Interjection,
            "num" => Self::Numeral,
            _ => Self::Other,
        }
    }

    /// The WN-LMF `partOfSpeech` tag for this POS — the exact inverse
    /// of [`parse`](Self::parse) on every WordNet tag, including the
    /// satellite-adjective `s` (which previously re-emitted as `a`, a
    /// byte loss). `Other` re-emits the DTD's `x` ("other") sentinel.
    pub fn to_tag(&self) -> &'static str {
        match self {
            Self::Noun => "n",
            Self::Verb => "v",
            Self::Adjective => "a",
            Self::SatelliteAdjective => "s",
            Self::Adverb => "r",
            Self::Determiner => "det",
            Self::Pronoun => "pron",
            Self::Preposition => "adp",
            Self::Conjunction => "conj",
            Self::Particle => "part",
            Self::Copula => "cop",
            Self::Auxiliary => "aux",
            Self::Interjection => "intj",
            Self::Numeral => "num",
            Self::Other => "x",
        }
    }

    /// Is this an open-class (content word) POS?
    pub fn is_open_class(&self) -> bool {
        matches!(
            self,
            Self::Noun | Self::Verb | Self::Adjective | Self::SatelliteAdjective | Self::Adverb
        )
    }

    /// Is this a closed-class (function word) POS?
    pub fn is_closed_class(&self) -> bool {
        !self.is_open_class() && *self != Self::Other
    }
}

/// Verb transitivity determined from WordNet subcategorization frames.
/// The frame ID encodes the argument structure of the verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerbTransitivity {
    Intransitive,
    Transitive,
    Ditransitive,
}

impl VerbTransitivity {
    /// Determine transitivity from a WordNet subcategorization-frame TEXT —
    /// the LOADED `subcategorizationFrame` of the `<SyntacticBehaviour>` the
    /// sense references (audit 2026-06-12 D-15).
    ///
    /// Transitivity is the count of bare object NPs the verb takes: parse the
    /// Princeton sentence-frame template (ISO 24613 LMF `SyntacticBehaviour`;
    /// Open English WordNet 2025) and count the LEADING run of `somebody` /
    /// `something` placeholders AFTER the `----` verb marker, stopping at the
    /// first oblique (a preposition / complementizer / category placeholder like
    /// PP / INFINITIVE / CLAUSE / Adjective). 0 objects = Intransitive (incl. the
    /// impersonal `It is ----ing` / `It ----s that CLAUSE` frames the old
    /// id-prefix test silently dropped to `None`); 1 = Transitive; 2 =
    /// Ditransitive. Reading the frame grammar IS reading the loaded data — the
    /// frame text is the authoritative signal, not the lossy 2-char id prefix.
    pub fn from_frame(frame: &str) -> Option<Self> {
        let toks: Vec<String> = frame.split_whitespace().map(str::to_lowercase).collect();
        let verb = toks.iter().position(|t| t.contains("----"))?;
        let objects = toks[verb + 1..]
            .iter()
            .take_while(|t| t.as_str() == "somebody" || t.as_str() == "something")
            .count();
        Some(match objects {
            0 => Self::Intransitive,
            1 => Self::Transitive,
            _ => Self::Ditransitive,
        })
    }

    /// Determine transitivity from a frame ID by its documented `v[ti][ai]…`
    /// prefix scheme — the FALLBACK used only when an id has no
    /// `<SyntacticBehaviour>` definition to read the text from (never fires on
    /// OEWN 2025, where every used id is defined). Prefer [`from_frame`], which
    /// reads the loaded frame text; the prefix is a lossy derivative of it (it
    /// drops the non-`vt`/`vi` impersonal frames).
    pub fn from_frame_id(frame_id: &str) -> Option<Self> {
        match frame_id {
            "ditransitive" => Some(Self::Ditransitive),
            id if id.starts_with("vt") => Some(Self::Transitive),
            id if id.starts_with("vi") => Some(Self::Intransitive),
            _ => None,
        }
    }

    /// Determine the best transitivity from a set of frame IDs.
    /// If a verb has both transitive and intransitive frames, it's both.
    /// Returns the "richest" (ditransitive > transitive > intransitive).
    pub fn from_frame_ids(frame_ids: &[String]) -> Option<Self> {
        let mut best = None;
        for id in frame_ids {
            if let Some(t) = Self::from_frame_id(id) {
                best = Some(match (best, t) {
                    (None, t) => t,
                    (Some(Self::Ditransitive), _) | (_, Self::Ditransitive) => Self::Ditransitive,
                    (Some(Self::Transitive), _) | (_, Self::Transitive) => Self::Transitive,
                    (Some(Self::Intransitive), Self::Intransitive) => Self::Intransitive,
                });
            }
        }
        best
    }
}

/// `<Lexicon>` metadata — the attributes declared on `<!ATTLIST
/// Lexicon>` (DTD lines 6-32). All-`Option` so an absent `#IMPLIED`
/// attribute round-trips as absent; the four `#REQUIRED` attrs
/// (`id`/`label`/`language`/`email`/`license`/`version`) are still
/// modelled as `Option` to keep the reader total over malformed input
/// (a missing required attr reads as `None` rather than failing the
/// whole parse). Only the corpus-confirmed attrs are surfaced as named
/// fields; the full Dublin Core set lands in `dc`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct LexiconMetadata {
    /// `id ID #REQUIRED` (DTD line 7).
    pub id: Option<String>,
    /// `label CDATA #REQUIRED` (DTD line 8).
    pub label: Option<String>,
    /// `language CDATA #REQUIRED` (DTD line 9) — a BCP-47 language tag.
    pub language: Option<String>,
    /// `email CDATA #REQUIRED` (DTD line 10).
    pub email: Option<String>,
    /// `license CDATA #REQUIRED` (DTD line 11).
    pub license: Option<String>,
    /// `version CDATA #REQUIRED` (DTD line 12).
    pub version: Option<String>,
    /// `url CDATA #IMPLIED` (DTD line 13).
    pub url: Option<String>,
    /// `citation CDATA #IMPLIED` (DTD line 14).
    pub citation: Option<String>,
    /// `logo CDATA #IMPLIED` (DTD line 15).
    pub logo: Option<String>,
    /// `status CDATA #IMPLIED` (DTD line 30).
    pub status: Option<String>,
    /// `confidenceScore CDATA "1.0"` (DTD line 32).
    pub confidence_score: Option<String>,
    /// The Dublin Core `dc:*` attributes (DTD lines 16-29), keyed by
    /// their declared prefixed name (e.g. `"dc:source"`), in the order
    /// the DTD declares them. A flat name→value map keeps the writer a
    /// faithful inverse without enumerating fifteen near-identical
    /// fields; the prefixed name is the DTD's, so it round-trips
    /// verbatim.
    pub dc: Vec<(String, String)>,
}

/// A complete WordNet lexicon loaded from LMF.
///
/// `PartialEq + Eq` so the graph-faithful `.prx` envelope
/// ([`WordNetPrxEnvelope`](super::prx::WordNetPrxEnvelope)) — which carries this
/// ontology directly — can derive the structural equality the rkyv round-trip
/// tests assert (`assert_eq!(envelope, decoded)`); every field type is itself
/// `Eq` (String/Vec/Option over the `Eq` LMF structs).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct WordNet {
    /// `<Lexicon>` metadata attributes (DTD lines 6-32). `None` only if
    /// no `<Lexicon>` element carried any attributes; otherwise the
    /// captured attribute set (so the metadata survives a round-trip).
    pub lexicon: LexiconMetadata,
    pub synsets: Vec<Synset>,
    pub entries: Vec<LexicalEntry>,
    /// Lexicon-level `<SyntacticBehaviour>` children (`SyntacticBehaviour*`
    /// in `<!ELEMENT Lexicon …>`, DTD line 5). In Open English WordNet
    /// 2025 the 39 `<SyntacticBehaviour>` elements live here, at lexicon
    /// scope, referencing senses by `senses` IDREFS.
    pub syntactic_behaviours: Vec<SyntacticBehaviour>,
}

impl WordNet {
    pub fn synset_count(&self) -> usize {
        self.synsets.len()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn find_synset(&self, id: &str) -> Option<&Synset> {
        self.synsets.iter().find(|s| s.id == id)
    }

    pub fn lookup_word(&self, word: &str) -> Vec<&Synset> {
        let synset_ids: Vec<&str> = self
            .entries
            .iter()
            .filter(|e| e.lemma.written_form == word)
            .flat_map(|e| e.senses.iter().map(|s| s.synset.as_str()))
            .collect();
        synset_ids
            .iter()
            .filter_map(|id| self.find_synset(id))
            .collect()
    }

    /// All taxonomy (is-a) relations: (child synset ID, parent synset ID).
    pub fn taxonomy_relations(&self) -> Vec<(&str, &str)> {
        self.synsets
            .iter()
            .flat_map(|s| {
                s.relations
                    .iter()
                    .filter(|r| r.rel_type.is_taxonomy())
                    .map(move |r| (s.id.as_str(), r.target.as_str()))
            })
            .collect()
    }

    /// All mereology (has-a) relations.
    pub fn mereology_relations(&self) -> Vec<(&str, &str)> {
        self.synsets
            .iter()
            .flat_map(|s| {
                s.relations
                    .iter()
                    .filter(|r| r.rel_type.is_mereology())
                    .map(move |r| (s.id.as_str(), r.target.as_str()))
            })
            .collect()
    }

    /// All opposition (antonym) relations from sense-level.
    pub fn opposition_relations(&self) -> Vec<(&str, &str)> {
        self.entries
            .iter()
            .flat_map(|e| {
                e.senses.iter().flat_map(|s| {
                    s.relations
                        .iter()
                        .filter(|r| r.rel_type.is_opposition())
                        .map(move |r| (s.id.as_str(), r.target.as_str()))
                })
            })
            .collect()
    }

    /// All causal relations.
    pub fn causal_relations(&self) -> Vec<(&str, &str)> {
        self.synsets
            .iter()
            .flat_map(|s| {
                s.relations
                    .iter()
                    .filter(|r| r.rel_type.is_causal())
                    .map(move |r| (s.id.as_str(), r.target.as_str()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;

    #[test]
    fn lmf_pos_entity_variants() {
        let variants = LmfPos::variants();
        // Structural floor — the typed enum carries at least the
        // 4 WordNet open-class tags + the satellite-adjective +
        // a non-zero Other sentinel, so 6 is the lower bound.
        // Exact count is an artifact of which Universal-Dependencies
        // / OLiA tags the praxis ontology has classified; asserting
        // the exact number here is a bounded-discovery claim per
        // `feedback_no_bounded_discovery_counts`. Use
        // [`axiom_lmf_pos_parse_covers_wn_lmf_dtd_enumeration`] for
        // the load-bearing coverage check.
        assert!(
            variants.len() >= 6,
            "LmfPos must carry at least the 4 WordNet open-class tags + satellite + Other; \
             got only {} variants",
            variants.len()
        );
        // Every variant must round-trip through `to_tag()` and
        // `parse()` — exhaustive bijection check, not a count.
        for pos in variants {
            assert_eq!(LmfPos::parse(pos.to_tag()), pos);
        }
    }

    /// Every WN-LMF 1.3 DTD-declared `partOfSpeech` enumeration
    /// value (per `<!ATTLIST Lemma partOfSpeech (n|v|a|r|s|t|c|p|x|u)
    /// #REQUIRED>` at DTD line 57 + the same enumeration on
    /// `Synset` at line 102) parses to a deterministic
    /// [`LmfPos`] variant — totality across the DTD enumeration.
    ///
    /// The four WordNet open-class tags (`n`/`v`/`a`/`r`) plus the
    /// `s` satellite-adjective tag project to named variants
    /// (Noun/Verb/Adjective/Adverb); the WN-LMF-specific tags
    /// `t`/`c`/`p`/`x`/`u` (terminology / closed-class compounds /
    /// preposition / other / unknown — narrow WN-LMF semantics)
    /// project to [`LmfPos::Other`]. Both projections are
    /// deterministic and grounded in the loaded DTD enumeration
    /// rather than scattered through unrelated source files
    /// (`feedback_bottom_up_loaded_not_encoded`).
    #[test]
    fn axiom_lmf_pos_parse_covers_wn_lmf_dtd_enumeration() {
        let dtd_values = super::super::dtd::wn_lmf_attlist_enum_values("Lemma", "partOfSpeech")
            .expect("WN-LMF 1.3 declares <!ATTLIST Lemma partOfSpeech …>");
        assert_eq!(
            dtd_values.len(),
            10,
            "WN-LMF 1.3 partOfSpeech declares 10 values; got {} — DTD or extractor drifted",
            dtd_values.len()
        );
        // Totality: parse() returns a deterministic LmfPos variant
        // for every DTD-declared input; the call never panics and
        // the result is reproducible across calls (LmfPos is Copy).
        for value in &dtd_values {
            let first = LmfPos::parse(value);
            let second = LmfPos::parse(value);
            assert_eq!(
                first, second,
                "LmfPos::parse({value:?}) returned different results on consecutive calls — \
                 non-determinism"
            );
        }
        // The four WordNet open-class tags + the satellite-adjective
        // `s` MUST project to DISTINCT named variants — the satellite
        // tag is its own variant so the source byte `s` survives a
        // round-trip (it no longer collapses into `Adjective`/`a`).
        // This is the load-bearing chunk of the dispatch.
        assert_eq!(LmfPos::parse("n"), LmfPos::Noun);
        assert_eq!(LmfPos::parse("v"), LmfPos::Verb);
        assert_eq!(LmfPos::parse("a"), LmfPos::Adjective);
        assert_eq!(LmfPos::parse("s"), LmfPos::SatelliteAdjective);
        assert_eq!(LmfPos::parse("r"), LmfPos::Adverb);
        // The satellite tag is an EXACT bijection: `s → SatelliteAdjective
        // → "s"`, the byte-loss this slice closes.
        assert_eq!(LmfPos::SatelliteAdjective.to_tag(), "s");
        assert_eq!(
            LmfPos::parse(LmfPos::SatelliteAdjective.to_tag()),
            LmfPos::SatelliteAdjective
        );
    }

    /// Every WN-LMF 1.3 DTD-declared `SynsetRelation/relType`
    /// enumeration value must parse to a known `SynsetRelationType`
    /// variant OR to the explicit `Other(0)` fallback (the spec
    /// declares ~70 values; the typed enum captures the canonical
    /// taxonomy/mereology/causal subset and routes the long tail
    /// to `Other(0)`). The axiom asserts parse() is total on the
    /// DTD enumeration — every declared value gets a deterministic
    /// answer, none panics.
    #[test]
    fn axiom_synset_relation_type_parse_covers_wn_lmf_dtd_enumeration() {
        let dtd_values = super::super::dtd::wn_lmf_attlist_enum_values("SynsetRelation", "relType")
            .expect("WN-LMF 1.3 declares <!ATTLIST SynsetRelation relType …>");
        assert!(
            dtd_values.len() >= 30,
            "WN-LMF 1.3 SynsetRelation/relType declares many values; got only {} — \
             extractor or DTD parse may be regressed",
            dtd_values.len()
        );
        // Totality AND exact round-trip over the WHOLE enumeration: every
        // DTD-declared relType parses deterministically, and `as_str` is
        // now its exact inverse for EVERY value — the long tail routes to
        // `Other(s)`, which reproduces the source string `s` byte-for-byte
        // (the value loss this slice closes). So `parse(as_str(parse(v)))
        // == parse(v)` AND `as_str(parse(v)) == v` for all declared `v`.
        for value in &dtd_values {
            let parsed = SynsetRelationType::parse(value);
            assert_eq!(
                parsed.as_str(),
                value.as_str(),
                "as_str must reproduce the source relType {value:?} exactly"
            );
            assert_eq!(
                SynsetRelationType::parse(parsed.as_str()),
                parsed,
                "parse∘as_str must be identity for {value:?}"
            );
        }
    }

    /// The typed model carries a field for every WN-LMF 1.3 ELEMENT the
    /// reader previously dropped: `Pronunciation` (DTD line 63),
    /// `ILIDefinition` (line 145), `SyntacticBehaviour` (line 228) and
    /// `Count` (line 233) are all declared element types in the loaded
    /// DTD, and each now has a typed home. Grounded against the loaded
    /// DTD's element-decl set (`feedback_bottom_up_loaded_not_encoded`),
    /// not a hardcoded name list.
    #[test]
    fn axiom_typed_model_covers_dropped_wn_lmf_elements() {
        use super::super::dtd::is_wn_lmf_element;
        // Each element the reader used to drop IS a declared WN-LMF 1.3
        // element type — so dropping it WAS losing schema-declared content.
        for elem in [
            "Pronunciation",
            "ILIDefinition",
            "SyntacticBehaviour",
            "Count",
        ] {
            assert!(
                is_wn_lmf_element(elem),
                "WN-LMF 1.3 must declare <!ELEMENT {elem} …> — DTD or extractor drifted"
            );
        }
        // …and each now has a typed home (constructed here = it compiles
        // with the field, the structural witness that the drop is closed).
        let _pron = Pronunciation {
            text: "/dɒɡ/".into(),
            variety: Some("GB".into()),
            notation: Some("ipa".into()),
            phonemic: Some("true".into()),
            audio: None,
        };
        let _count = Count { value: "42".into() };
        let _sb = SyntacticBehaviour {
            id: None,
            subcategorization_frame: "Somebody ----s something".into(),
            senses: vec!["s1".into()],
        };
        // ILIDefinition is carried as Synset::ili_definition (Option<String>).
    }

    /// The typed model carries a field for the corpus-confirmed WN-LMF 1.3
    /// ATTRIBUTES the reader previously dropped — each is a declared
    /// `<!ATTLIST>` attribute in the loaded DTD. We assert the DTD declares
    /// each attribute (its enumeration extractor returns the declared
    /// enumeration where the attr is enumerated, e.g. `Sense/adjposition`),
    /// and that the typed struct has the field (it constructs).
    #[test]
    fn axiom_typed_model_covers_dropped_wn_lmf_attributes() {
        use super::super::dtd::wn_lmf_attlist_enum_values;
        // `Sense/adjposition` is an ENUMERATED attr `(a|ip|p)` (DTD line 96):
        // the extractor returns exactly those three values.
        let adjpos = wn_lmf_attlist_enum_values("Sense", "adjposition")
            .expect("WN-LMF 1.3 declares <!ATTLIST Sense adjposition (a|ip|p)>");
        assert_eq!(adjpos, vec!["a", "ip", "p"]);
        // `Pronunciation/phonemic` is an enumerated `(true|false)` attr
        // (DTD line 68).
        let phonemic = wn_lmf_attlist_enum_values("Pronunciation", "phonemic")
            .expect("WN-LMF 1.3 declares <!ATTLIST Pronunciation phonemic (true|false)>");
        assert_eq!(phonemic, vec!["true", "false"]);
        // The typed structs carry the corpus-confirmed CDATA attrs as fields
        // (constructs ⇒ the field exists ⇒ the value has a home to survive in).
        let _form = Form {
            written_form: "dogs".into(),
            id: Some("f1".into()),
            script: None,
            pronunciations: Vec::new(),
        };
        let _syn = Synset {
            id: "s1".into(),
            ili: Some("i1".into()),
            pos: LmfPos::Noun,
            members: Vec::new(),
            definitions: Vec::new(),
            ili_definition: Some("gloss".into()),
            examples: Vec::new(),
            relations: Vec::new(),
            lexfile: Some("noun.animal".into()),
            dc_source: Some("pwn".into()),
            confidence_score: None,
        };
        let _meta = LexiconMetadata {
            id: Some("oewn".into()),
            label: Some("Open English WordNet".into()),
            language: Some("en".into()),
            email: Some("x@y".into()),
            license: Some("CC".into()),
            version: Some("2025".into()),
            url: Some("https://…".into()),
            citation: None,
            logo: None,
            status: Some("valid".into()),
            confidence_score: Some("1.0".into()),
            dc: vec![("dc:source".into(), "pwn".into())],
        };
    }

    /// Symmetric coverage axiom for `SenseRelation/relType` — every
    /// DTD-declared value must parse deterministically (named
    /// variant or `Other(s)`) and round-trip exactly.
    #[test]
    fn axiom_sense_relation_type_parse_covers_wn_lmf_dtd_enumeration() {
        let dtd_values = super::super::dtd::wn_lmf_attlist_enum_values("SenseRelation", "relType")
            .expect("WN-LMF 1.3 declares <!ATTLIST SenseRelation relType …>");
        assert!(
            !dtd_values.is_empty(),
            "DTD's SenseRelation/relType enumeration must be non-empty"
        );
        // Exact round-trip over the whole enumeration (see the SynsetRelation
        // axiom): `as_str` reproduces every declared relType verbatim, the
        // long tail via `Other(s)`.
        for value in &dtd_values {
            let parsed = SenseRelationType::parse(value);
            assert_eq!(
                parsed.as_str(),
                value.as_str(),
                "as_str must reproduce the source relType {value:?} exactly"
            );
            assert_eq!(
                SenseRelationType::parse(parsed.as_str()),
                parsed,
                "parse∘as_str must be identity for {value:?}"
            );
        }
    }

    #[test]
    fn pos_parse_roundtrip() {
        for pos in LmfPos::variants() {
            let tag = pos.to_tag();
            let parsed = LmfPos::parse(tag);
            assert_eq!(
                parsed, pos,
                "roundtrip failed for {:?} -> {} -> {:?}",
                pos, tag, parsed
            );
        }
    }

    #[test]
    fn open_closed_partition() {
        for pos in LmfPos::variants() {
            if pos != LmfPos::Other {
                assert!(
                    pos.is_open_class() ^ pos.is_closed_class(),
                    "{:?} must be exactly one of open/closed",
                    pos
                );
            }
        }
    }

    #[test]
    fn synset_relation_taxonomy() {
        assert!(SynsetRelationType::Hypernym.is_taxonomy());
        assert!(SynsetRelationType::InstanceHypernym.is_taxonomy());
        assert!(!SynsetRelationType::Causes.is_taxonomy());
    }

    #[test]
    fn synset_relation_mereology() {
        assert!(SynsetRelationType::HoloPart.is_mereology());
        assert!(SynsetRelationType::MeroPart.is_mereology());
        assert!(!SynsetRelationType::Hypernym.is_mereology());
    }

    #[test]
    fn verb_transitivity_from_frame_text() {
        use VerbTransitivity as VT;
        // Object count over the LOADED Princeton frame text (D-15).
        assert_eq!(VT::from_frame("Somebody ----s"), Some(VT::Intransitive));
        assert_eq!(
            VT::from_frame("Somebody ----s something"),
            Some(VT::Transitive)
        );
        assert_eq!(
            VT::from_frame("Something ----s somebody"),
            Some(VT::Transitive)
        );
        assert_eq!(
            VT::from_frame("Somebody ----s somebody something"),
            Some(VT::Ditransitive)
        );
        // Obliques are not objects (preposition / category placeholder stop the run).
        assert_eq!(
            VT::from_frame("Somebody ----s to somebody"),
            Some(VT::Intransitive)
        );
        assert_eq!(
            VT::from_frame("Somebody ----s at something"),
            Some(VT::Intransitive)
        );
        assert_eq!(
            VT::from_frame("Somebody ----s Adjective"),
            Some(VT::Intransitive)
        );
        assert_eq!(
            VT::from_frame("Somebody ----s something to somebody"),
            Some(VT::Transitive)
        );
        // The impersonal frames the id-prefix dropped to None are now Intransitive.
        assert_eq!(VT::from_frame("It is ----ing"), Some(VT::Intransitive));
        assert_eq!(
            VT::from_frame("It ----s that CLAUSE"),
            Some(VT::Intransitive)
        );
        // A text with no verb marker is unclassifiable.
        assert_eq!(VT::from_frame("not a frame"), None);
    }

    #[test]
    fn from_frame_agrees_with_id_prefix_where_the_prefix_applies() {
        use VerbTransitivity as VT;
        // Where the documented id prefix IS total (vt*/vi*/ditransitive), the
        // loaded-frame parse must agree with it — the migration is a superset,
        // not a behaviour change.
        for (id, frame) in [
            ("via", "Somebody ----s"),
            ("vii", "Something ----s"),
            ("via-to", "Somebody ----s to somebody"),
            ("via-adj", "Somebody ----s Adjective"),
            ("vtai", "Somebody ----s something"),
            ("vtaa", "Somebody ----s somebody"),
            ("vtia", "Something ----s somebody"),
            ("vtai-to", "Somebody ----s something to somebody"),
            ("vtaa-with", "Somebody ----s somebody with something"),
            ("ditransitive", "Somebody ----s somebody something"),
        ] {
            assert_eq!(
                VT::from_frame(frame),
                VT::from_frame_id(id),
                "frame `{frame}` (id {id}) disagreed"
            );
        }
    }

    // ── Property-based round-trip laws for LMF enum parsers ───────
    //
    // Every relType string documented in the Global WordNet
    // Association schema must round-trip to a non-Other variant.
    // Unknown strings must collapse to Other.

    use proptest::prelude::*;

    const KNOWN_SYNSET_REL_TYPES: &[&str] = &[
        "hypernym",
        "instance_hypernym",
        "hyponym",
        "instance_hyponym",
        "holo_member",
        "holo_part",
        "holo_substance",
        "mero_member",
        "mero_part",
        "mero_substance",
        "causes",
        "is_caused_by",
        "entails",
        "is_entailed_by",
        "similar",
        "also",
        "attribute",
        "domain_topic",
        "has_domain_topic",
        "domain_region",
        "has_domain_region",
        "exemplifies",
        "is_exemplified_by",
        "participle",
    ];

    const KNOWN_SENSE_REL_TYPES: &[&str] = &[
        "antonym",
        "similar",
        "pertainym",
        "derivation",
        "also",
        "exemplifies",
        "is_exemplified_by",
        "participle",
    ];

    #[test]
    fn property_every_known_synset_reltype_parses_non_other() {
        for s in KNOWN_SYNSET_REL_TYPES {
            let parsed = SynsetRelationType::parse(s);
            assert!(
                !matches!(parsed, SynsetRelationType::Other(_)),
                "documented relType `{s}` parsed to Other"
            );
        }
    }

    #[test]
    fn property_every_known_sense_reltype_parses_non_other() {
        for s in KNOWN_SENSE_REL_TYPES {
            let parsed = SenseRelationType::parse(s);
            assert!(
                !matches!(parsed, SenseRelationType::Other(_)),
                "documented sense relType `{s}` parsed to Other"
            );
        }
    }

    /// `as_str` is the exact inverse of `parse` for every NON-`Other`
    /// `SynsetRelationType` variant: `parse(x.as_str()) == x`. This is
    /// the law the WN-LMF structural writer relies on to emit the same
    /// `relType` string the reader consumed. The `Other(_)` arm is
    /// excluded — `parse` discarded the original string (an L1
    /// limitation documented on `as_str`).
    #[test]
    fn synset_relation_type_as_str_inverts_parse() {
        for s in KNOWN_SYNSET_REL_TYPES {
            let variant = SynsetRelationType::parse(s);
            assert!(
                !matches!(variant, SynsetRelationType::Other(_)),
                "fixture `{s}` should be a named variant"
            );
            assert_eq!(
                SynsetRelationType::parse(variant.as_str()),
                variant,
                "parse(as_str()) must be identity for {variant:?}"
            );
        }
    }

    /// Symmetric inverse law for every NON-`Other` `SenseRelationType`
    /// variant: `parse(x.as_str()) == x`.
    #[test]
    fn sense_relation_type_as_str_inverts_parse() {
        for s in KNOWN_SENSE_REL_TYPES {
            let variant = SenseRelationType::parse(s);
            assert!(
                !matches!(variant, SenseRelationType::Other(_)),
                "fixture `{s}` should be a named variant"
            );
            assert_eq!(
                SenseRelationType::parse(variant.as_str()),
                variant,
                "parse(as_str()) must be identity for {variant:?}"
            );
        }
    }

    proptest! {
        #[test]
        fn property_unknown_synset_reltype_collapses_to_other(
            s in "[a-z_]{3,20}",
        ) {
            // Skip the known set to avoid false negatives.
            if KNOWN_SYNSET_REL_TYPES.contains(&s.as_str()) {
                return Ok(());
            }
            prop_assert!(matches!(
                SynsetRelationType::parse(&s),
                SynsetRelationType::Other(_)
            ));
        }

        #[test]
        fn property_unknown_sense_reltype_collapses_to_other(
            s in "[a-z_]{3,20}",
        ) {
            if KNOWN_SENSE_REL_TYPES.contains(&s.as_str()) {
                return Ok(());
            }
            prop_assert!(matches!(
                SenseRelationType::parse(&s),
                SenseRelationType::Other(_)
            ));
        }
    }

    #[test]
    fn property_taxonomy_predicate_covers_hypernym_family() {
        // is_taxonomy() should return true exactly for the hypernym
        // family and false for everything else.
        for s in KNOWN_SYNSET_REL_TYPES {
            let parsed = SynsetRelationType::parse(s);
            let expected = matches!(s, &"hypernym" | &"instance_hypernym");
            assert_eq!(
                parsed.is_taxonomy(),
                expected,
                "is_taxonomy on `{s}` returned wrong value"
            );
        }
    }

    #[test]
    fn property_mereology_predicate_covers_meronym_family() {
        for s in KNOWN_SYNSET_REL_TYPES {
            let parsed = SynsetRelationType::parse(s);
            let expected = matches!(
                s,
                &"holo_member"
                    | &"holo_part"
                    | &"holo_substance"
                    | &"mero_member"
                    | &"mero_part"
                    | &"mero_substance"
            );
            assert_eq!(
                parsed.is_mereology(),
                expected,
                "is_mereology on `{s}` returned wrong value"
            );
        }
    }
}

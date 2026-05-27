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

/// A synset — a set of words sharing the same meaning (a concept).
/// This is the fundamental unit of WordNet: not a word, but a MEANING.
#[derive(Debug, Clone, PartialEq)]
pub struct Synset {
    pub id: String,
    pub ili: Option<String>,
    pub pos: LmfPos,
    pub members: Vec<String>,
    pub definitions: Vec<String>,
    pub examples: Vec<String>,
    pub relations: Vec<SynsetRelation>,
}

/// A lexical entry — a word with its senses (connections to synsets).
#[derive(Debug, Clone, PartialEq)]
pub struct LexicalEntry {
    pub id: String,
    pub lemma: Lemma,
    pub senses: Vec<Sense>,
    pub forms: Vec<Form>,
}

/// A lemma — the canonical form of a word.
#[derive(Debug, Clone, PartialEq)]
pub struct Lemma {
    pub written_form: String,
    pub pos: LmfPos,
}

/// A sense — the connection between a word and a meaning (synset).
#[derive(Debug, Clone, PartialEq)]
pub struct Sense {
    pub id: String,
    pub synset: String,
    pub relations: Vec<SenseRelation>,
    /// Subcategorization frame IDs (verb frames for transitivity).
    /// From LMF `subcat` attribute. E.g., ["vtai", "vtaa"] for transitive.
    pub subcat: Vec<String>,
}

/// A morphological form — an inflected variant of a word.
#[derive(Debug, Clone, PartialEq)]
pub struct Form {
    pub written_form: String,
}

/// Synset-level relation (between concepts).
/// These map directly to our reasoning ontology:
/// - hypernym → TaxonomyDef (child is-a parent)
/// - meronym → MereologyDef (whole has-a part)
/// - antonym → OppositionDef
/// - causes → CausalDef
#[derive(Debug, Clone, PartialEq)]
pub struct SynsetRelation {
    pub rel_type: SynsetRelationType,
    pub target: String,
}

/// Sense-level relation (between word senses).
#[derive(Debug, Clone, PartialEq)]
pub struct SenseRelation {
    pub rel_type: SenseRelationType,
    pub target: String,
}

/// Types of synset-level relations in WordNet.
///
/// Covers the Global WordNet Association LMF schema (Vossen et al.)
/// — synset relations in WordNet 2025 fall into these categories.
/// See <https://globalwordnet.github.io/schemas/>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    Other(u8),
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
            _ => Self::Other(0),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SenseRelationType {
    Antonym,
    Similar,
    Pertainym,
    Derivation,
    Also,
    Exemplifies,
    IsExemplifiedBy,
    Participle,
    Other(u8),
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
            _ => Self::Other(0),
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
pub enum LmfPos {
    // Open class (WordNet)
    Noun,
    Verb,
    Adjective,
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
    pub fn parse(s: &str) -> Self {
        match s {
            // WordNet open class
            "n" => Self::Noun,
            "v" => Self::Verb,
            "a" | "s" => Self::Adjective,
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

    pub fn to_tag(&self) -> &'static str {
        match self {
            Self::Noun => "n",
            Self::Verb => "v",
            Self::Adjective => "a",
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
            Self::Noun | Self::Verb | Self::Adjective | Self::Adverb
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
    /// Determine transitivity from a WordNet subcategorization frame ID.
    /// Frame IDs follow the pattern: `v[ti][ai][ai][-suffix]`
    /// - "via" / "vii" = intransitive (Somebody/Something ----s)
    /// - "vtaa" / "vtai" / "vtia" / "vtii" = transitive
    /// - "ditransitive" = ditransitive
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

/// A complete WordNet lexicon loaded from LMF.
#[derive(Debug, Clone)]
pub struct WordNet {
    pub synsets: Vec<Synset>,
    pub entries: Vec<LexicalEntry>,
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
    use pr4xis::category::Concept;

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
        // `s` MUST project to named variants. This is the load-
        // bearing chunk of the dispatch.
        assert_eq!(LmfPos::parse("n"), LmfPos::Noun);
        assert_eq!(LmfPos::parse("v"), LmfPos::Verb);
        assert_eq!(LmfPos::parse("a"), LmfPos::Adjective);
        assert_eq!(LmfPos::parse("s"), LmfPos::Adjective);
        assert_eq!(LmfPos::parse("r"), LmfPos::Adverb);
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
        for value in &dtd_values {
            let _ = SynsetRelationType::parse(value);
        }
    }

    /// Symmetric coverage axiom for `SenseRelation/relType` — every
    /// DTD-declared value must parse deterministically (named
    /// variant or `Other(0)`).
    #[test]
    fn axiom_sense_relation_type_parse_covers_wn_lmf_dtd_enumeration() {
        let dtd_values = super::super::dtd::wn_lmf_attlist_enum_values("SenseRelation", "relType")
            .expect("WN-LMF 1.3 declares <!ATTLIST SenseRelation relType …>");
        assert!(
            !dtd_values.is_empty(),
            "DTD's SenseRelation/relType enumeration must be non-empty"
        );
        for value in &dtd_values {
            let _ = SenseRelationType::parse(value);
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
    fn verb_transitivity_from_frame() {
        assert_eq!(
            VerbTransitivity::from_frame_id("vtai"),
            Some(VerbTransitivity::Transitive)
        );
        assert_eq!(
            VerbTransitivity::from_frame_id("via"),
            Some(VerbTransitivity::Intransitive)
        );
        assert_eq!(
            VerbTransitivity::from_frame_id("ditransitive"),
            Some(VerbTransitivity::Ditransitive)
        );
        assert_eq!(VerbTransitivity::from_frame_id("unknown"), None);
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

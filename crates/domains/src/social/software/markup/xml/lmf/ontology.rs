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
    Entails,
    Similar,
    Also,
    Attribute,
    DomainTopic,
    DomainRegion,
    Exemplifies,
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
            "entails" => Self::Entails,
            "similar" => Self::Similar,
            "also" => Self::Also,
            "attribute" => Self::Attribute,
            "domain_topic" => Self::DomainTopic,
            "domain_region" => Self::DomainRegion,
            "exemplifies" => Self::Exemplifies,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SenseRelationType {
    Antonym,
    Similar,
    Pertainym,
    Derivation,
    Exemplifies,
    Other(u8),
}

impl SenseRelationType {
    pub fn parse(s: &str) -> Self {
        match s {
            "antonym" => Self::Antonym,
            "similar" => Self::Similar,
            "pertainym" => Self::Pertainym,
            "derivation" => Self::Derivation,
            "exemplifies" => Self::Exemplifies,
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
    /// Frame IDs follow the pattern: v[ti][ai][ai][-suffix]
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
        assert_eq!(variants.len(), 14);
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
}

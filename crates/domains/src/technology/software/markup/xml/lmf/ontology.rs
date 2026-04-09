use praxis::category::Entity;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LmfPos {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Other,
}

impl Entity for LmfPos {
    fn variants() -> Vec<Self> {
        vec![
            Self::Noun,
            Self::Verb,
            Self::Adjective,
            Self::Adverb,
            Self::Other,
        ]
    }
}

impl LmfPos {
    pub fn parse(s: &str) -> Self {
        match s {
            "n" => Self::Noun,
            "v" => Self::Verb,
            "a" | "s" => Self::Adjective,
            "r" => Self::Adverb,
            _ => Self::Other,
        }
    }

    pub fn to_tag(&self) -> &'static str {
        match self {
            Self::Noun => "n",
            Self::Verb => "v",
            Self::Adjective => "a",
            Self::Adverb => "r",
            Self::Other => "x",
        }
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

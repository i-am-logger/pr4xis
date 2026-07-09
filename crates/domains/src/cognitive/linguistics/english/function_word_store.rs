//! The English function-word lexicon as a compact, zero-copy archive.
//!
//! `English` maps every closed-class function word (from
//! [`build_english_function_words`](crate::cognitive::linguistics::language::build_english_function_words))
//! to the rich [`LexicalEntry`] readings it can carry, plus a parallel list of the
//! function-word texts (spelling-correction candidates). Held naively as a
//! `HashMap<String, Vec<LexicalEntry>>` + a `Vec<String>` these are two owned
//! collections on `English`; this module reclaims BOTH into one archive.
//!
//! # The representation
//!
//! [`LexicalEntry`] is a 13-variant data-carrying enum whose leaves are `String` /
//! `Option<String>` / `char`-free `Copy` unit enums — all archivable. An authored
//! `FunctionWordRecords` mirror (kept OFF the live domain types per the
//! [`concept_store`](super::concept_store) / OWL `prx` precedent) carries a
//! sorted-by-key `Vec<String>` of the function-word texts alongside a parallel
//! `Vec<Vec<LexicalEntryRecord>>` of their readings. It is `rkyv`-serialized into a
//! single `AlignedVec<16>`, `bytecheck`-validated ONCE at build. The separate
//! `function_word_list` field is GONE — the sorted key set IS the word list
//! (`words`).
//!
//! # Reads
//!
//! * `first` / `all`
//!   — binary-search the archived keys (zero-copy `&str` compares), then
//!   materialize only the readings the caller consumes, each through
//!   [`From<&ArchivedLexicalEntryRecord>`] — `first` materializes exactly ONE
//!   entry, `all` materializes each element lazily; never a whole-vector
//!   deserialize. The call sites (`lexical_lookup` `.first()`, `lexical_lookup_all`
//!   iterate) own the resulting `LexicalEntry`s, so this is output-identical to a
//!   clone while reading through the buffer — no borrow of the archive escapes.
//! * `words` — the sorted key set, as
//!   zero-copy `&str` (the `known_words` / `word_count` reader).
//!
//! # Endianness invariant
//!
//! `rkyv`'s archived layout is little-endian by construction; the archived variant
//! is compiled only under `cfg(target_endian = "little")` (a big-endian target
//! falls back to the owned maps).
//!
//! Reference: Koloski, D. *rkyv: zero-copy deserialization framework for Rust* (v0.8)
//! — see <https://github.com/rkyv/rkyv>.

use alloc::vec::Vec;

use hashbrown::HashMap;

use crate::cognitive::linguistics::lexicon::pos::LexicalEntry;

/// The zero-copy, `prx`-gated function-word store (little-endian targets).
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::FunctionWordStore;

/// The function-word-store leaf-lens mirror root — exposed so the shared
/// lens-law axioms (`formal::meta::lens::rkyv_lens_laws`) can name it as the
/// `Mirror` of the `RkyvLens<HashMap<String, Vec<LexicalEntry>>, FunctionWordRecords>`
/// instance.
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::FunctionWordRecords;

/// The owned fallback store (no `prx`, or a big-endian target).
#[cfg(not(all(feature = "prx", target_endian = "little")))]
pub use owned::FunctionWordStore;

// ── shared surface ──────────────────────────────────────────────────────────

impl FunctionWordStore {
    /// Is the lexicon empty (no function words)?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl core::fmt::Debug for FunctionWordStore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FunctionWordStore")
            .field("len", &self.len())
            .finish()
    }
}

// ── owned fallback ───────────────────────────────────────────────────────────

/// The owned fallback: the `HashMap` the lexicon used to be. The mandatory
/// non-`prx` (and big-endian) path.
///
/// Compiled ALSO under `test` on the archived path so the cast unit tests can build
/// both representations from the same map and assert identical reads.
#[cfg(any(not(all(feature = "prx", target_endian = "little")), test))]
mod owned {
    use super::*;

    /// Function word → its readings, held as an owned map.
    pub struct FunctionWordStore {
        map: HashMap<alloc::string::String, Vec<LexicalEntry>>,
    }

    impl FunctionWordStore {
        /// Retain the owned build as-is.
        pub fn build(map: HashMap<alloc::string::String, Vec<LexicalEntry>>) -> Self {
            Self { map }
        }

        /// The FIRST reading for `word`, cloned (`None` if absent) — the
        /// `lexical_lookup` reader.
        pub fn first(&self, word: &str) -> Option<LexicalEntry> {
            self.map.get(word).and_then(|v| v.first().cloned())
        }

        /// ALL readings for `word`, cloned (empty if absent) — the
        /// `lexical_lookup_all` reader.
        pub fn all(&self, word: &str) -> Vec<LexicalEntry> {
            self.map.get(word).cloned().unwrap_or_default()
        }

        /// Every function-word text — the `known_words` reader (owned order is
        /// unspecified; the archived variant yields them sorted).
        pub fn words(&self) -> impl Iterator<Item = &str> {
            self.map.keys().map(|s| s.as_str())
        }

        /// Number of distinct function words.
        pub fn len(&self) -> usize {
            self.map.len()
        }
    }
}

// ── archived variant ─────────────────────────────────────────────────────────

/// The zero-copy archived store: the [`FunctionWordRecords`] mirror (sorted keys +
/// parallel reading vectors) `rkyv`-serialized into one [`AlignedVec<16>`].
#[cfg(all(feature = "prx", target_endian = "little"))]
mod archived {
    use alloc::string::String;

    use rkyv::util::AlignedVec;

    use pr4xis_runtime::lens::rkyv_lens::{RkyvLens, RkyvMirror, RkyvMirrorOwned, RkyvOwned};

    use super::*;
    use crate::cognitive::linguistics::lexicon::pos::{
        Adjective, Adverb, Auxiliary, Conjunction, Copula, Countability, Determiner,
        DeterminerKind, Interjection, InterjectionKind, Noun, NounKind, Number, Numeral, Particle,
        Person, Preposition, Pronoun, PronounKind, Tense, Transitivity, Verb,
    };

    const _: () = assert!(cfg!(target_endian = "little"));

    /// `FunctionWordStore` is `Sync`: its only field is an [`AlignedVec`] (rkyv
    /// declares it `Send + Sync`) plus a `Copy` scalar — no interior mutability.
    const _: fn() = || {
        fn assert_sync<T: Sync>() {}
        assert_sync::<FunctionWordStore>();
    };

    // ── mirror records: one `*Record` per POS struct ─────────────────────────

    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct NounRecord {
        pub text: String,
        pub number: Number,
        pub person: Person,
        pub countability: Countability,
        pub kind: NounKind,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct VerbRecord {
        pub text: String,
        pub lemma: String,
        pub number: Number,
        pub person: Person,
        pub tense: Tense,
        pub transitivity: Transitivity,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct DeterminerRecord {
        pub text: String,
        pub kind: DeterminerKind,
        pub number: Option<Number>,
        pub olia_class: Option<String>,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct AdjectiveRecord {
        pub text: String,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct AdverbRecord {
        pub text: String,
        pub olia_class: Option<String>,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct PrepositionRecord {
        pub text: String,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct ConjunctionRecord {
        pub text: String,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct PronounRecord {
        pub text: String,
        pub number: Number,
        pub person: Person,
        pub kind: PronounKind,
        pub olia_class: Option<String>,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct CopulaRecord {
        pub text: String,
        pub number: Number,
        pub person: Person,
        pub tense: Tense,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct AuxiliaryRecord {
        pub text: String,
        pub number: Option<Number>,
        pub tense: Option<Tense>,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct InterjectionRecord {
        pub text: String,
        pub kind: InterjectionKind,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct ParticleRecord {
        pub text: String,
    }
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct NumeralRecord {
        pub text: String,
    }

    /// Mirror of [`LexicalEntry`] — the 13-variant reading enum.
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub enum LexicalEntryRecord {
        Noun(NounRecord),
        Verb(VerbRecord),
        Determiner(DeterminerRecord),
        Adjective(AdjectiveRecord),
        Adverb(AdverbRecord),
        Preposition(PrepositionRecord),
        Conjunction(ConjunctionRecord),
        Pronoun(PronounRecord),
        Copula(CopulaRecord),
        Auxiliary(AuxiliaryRecord),
        Interjection(InterjectionRecord),
        Particle(ParticleRecord),
        Numeral(NumeralRecord),
    }

    /// The archive's root: sorted function-word texts + their parallel readings.
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct FunctionWordRecords {
        /// Function-word texts, sorted by raw UTF-8 bytes (the order `lookup`'s
        /// binary search assumes). `keys[i]` names `entries[i]`.
        pub keys: Vec<String>,
        /// Each key's readings, parallel to `keys`.
        pub entries: Vec<Vec<LexicalEntryRecord>>,
    }

    // ── owned → record (build side) ───────────────────────────────────────────

    impl From<&LexicalEntry> for LexicalEntryRecord {
        fn from(e: &LexicalEntry) -> Self {
            match e {
                LexicalEntry::Noun(n) => LexicalEntryRecord::Noun(NounRecord {
                    text: n.text.clone(),
                    number: n.number,
                    person: n.person,
                    countability: n.countability,
                    kind: n.kind,
                }),
                LexicalEntry::Verb(v) => LexicalEntryRecord::Verb(VerbRecord {
                    text: v.text.clone(),
                    lemma: v.lemma.clone(),
                    number: v.number,
                    person: v.person,
                    tense: v.tense,
                    transitivity: v.transitivity,
                }),
                LexicalEntry::Determiner(d) => LexicalEntryRecord::Determiner(DeterminerRecord {
                    text: d.text.clone(),
                    kind: d.kind,
                    number: d.number,
                    olia_class: d.olia_class.clone(),
                }),
                LexicalEntry::Adjective(a) => LexicalEntryRecord::Adjective(AdjectiveRecord {
                    text: a.text.clone(),
                }),
                LexicalEntry::Adverb(a) => LexicalEntryRecord::Adverb(AdverbRecord {
                    text: a.text.clone(),
                    olia_class: a.olia_class.clone(),
                }),
                LexicalEntry::Preposition(p) => {
                    LexicalEntryRecord::Preposition(PrepositionRecord {
                        text: p.text.clone(),
                    })
                }
                LexicalEntry::Conjunction(c) => {
                    LexicalEntryRecord::Conjunction(ConjunctionRecord {
                        text: c.text.clone(),
                    })
                }
                LexicalEntry::Pronoun(p) => LexicalEntryRecord::Pronoun(PronounRecord {
                    text: p.text.clone(),
                    number: p.number,
                    person: p.person,
                    kind: p.kind,
                    olia_class: p.olia_class.clone(),
                }),
                LexicalEntry::Copula(c) => LexicalEntryRecord::Copula(CopulaRecord {
                    text: c.text.clone(),
                    number: c.number,
                    person: c.person,
                    tense: c.tense,
                }),
                LexicalEntry::Auxiliary(a) => LexicalEntryRecord::Auxiliary(AuxiliaryRecord {
                    text: a.text.clone(),
                    number: a.number,
                    tense: a.tense,
                }),
                LexicalEntry::Interjection(i) => {
                    LexicalEntryRecord::Interjection(InterjectionRecord {
                        text: i.text.clone(),
                        kind: i.kind,
                    })
                }
                LexicalEntry::Particle(p) => LexicalEntryRecord::Particle(ParticleRecord {
                    text: p.text.clone(),
                }),
                LexicalEntry::Numeral(n) => LexicalEntryRecord::Numeral(NumeralRecord {
                    text: n.text.clone(),
                }),
            }
        }
    }

    // ── owned entry → record (build side, MOVE) ───────────────────────────────

    /// By-value twin of the borrowing [`From<&LexicalEntry>`] above: MOVE each
    /// reading's `text` / `lemma` / `olia_class` heap payload into the record
    /// rather than cloning it, for the owned PUT leg that consumes the lexicon at
    /// build. Byte-identical in result to its borrowing sibling.
    impl From<LexicalEntry> for LexicalEntryRecord {
        fn from(e: LexicalEntry) -> Self {
            match e {
                LexicalEntry::Noun(n) => LexicalEntryRecord::Noun(NounRecord {
                    text: n.text,
                    number: n.number,
                    person: n.person,
                    countability: n.countability,
                    kind: n.kind,
                }),
                LexicalEntry::Verb(v) => LexicalEntryRecord::Verb(VerbRecord {
                    text: v.text,
                    lemma: v.lemma,
                    number: v.number,
                    person: v.person,
                    tense: v.tense,
                    transitivity: v.transitivity,
                }),
                LexicalEntry::Determiner(d) => LexicalEntryRecord::Determiner(DeterminerRecord {
                    text: d.text,
                    kind: d.kind,
                    number: d.number,
                    olia_class: d.olia_class,
                }),
                LexicalEntry::Adjective(a) => {
                    LexicalEntryRecord::Adjective(AdjectiveRecord { text: a.text })
                }
                LexicalEntry::Adverb(a) => LexicalEntryRecord::Adverb(AdverbRecord {
                    text: a.text,
                    olia_class: a.olia_class,
                }),
                LexicalEntry::Preposition(p) => {
                    LexicalEntryRecord::Preposition(PrepositionRecord { text: p.text })
                }
                LexicalEntry::Conjunction(c) => {
                    LexicalEntryRecord::Conjunction(ConjunctionRecord { text: c.text })
                }
                LexicalEntry::Pronoun(p) => LexicalEntryRecord::Pronoun(PronounRecord {
                    text: p.text,
                    number: p.number,
                    person: p.person,
                    kind: p.kind,
                    olia_class: p.olia_class,
                }),
                LexicalEntry::Copula(c) => LexicalEntryRecord::Copula(CopulaRecord {
                    text: c.text,
                    number: c.number,
                    person: c.person,
                    tense: c.tense,
                }),
                LexicalEntry::Auxiliary(a) => LexicalEntryRecord::Auxiliary(AuxiliaryRecord {
                    text: a.text,
                    number: a.number,
                    tense: a.tense,
                }),
                LexicalEntry::Interjection(i) => {
                    LexicalEntryRecord::Interjection(InterjectionRecord {
                        text: i.text,
                        kind: i.kind,
                    })
                }
                LexicalEntry::Particle(p) => {
                    LexicalEntryRecord::Particle(ParticleRecord { text: p.text })
                }
                LexicalEntry::Numeral(n) => {
                    LexicalEntryRecord::Numeral(NumeralRecord { text: n.text })
                }
            }
        }
    }

    // ── record → owned (read side, after deserialize) ─────────────────────────

    impl From<LexicalEntryRecord> for LexicalEntry {
        fn from(r: LexicalEntryRecord) -> Self {
            match r {
                LexicalEntryRecord::Noun(n) => LexicalEntry::Noun(Noun {
                    text: n.text,
                    number: n.number,
                    person: n.person,
                    countability: n.countability,
                    kind: n.kind,
                }),
                LexicalEntryRecord::Verb(v) => LexicalEntry::Verb(Verb {
                    text: v.text,
                    lemma: v.lemma,
                    number: v.number,
                    person: v.person,
                    tense: v.tense,
                    transitivity: v.transitivity,
                }),
                LexicalEntryRecord::Determiner(d) => LexicalEntry::Determiner(Determiner {
                    text: d.text,
                    kind: d.kind,
                    number: d.number,
                    olia_class: d.olia_class,
                }),
                LexicalEntryRecord::Adjective(a) => {
                    LexicalEntry::Adjective(Adjective { text: a.text })
                }
                LexicalEntryRecord::Adverb(a) => LexicalEntry::Adverb(Adverb {
                    text: a.text,
                    olia_class: a.olia_class,
                }),
                LexicalEntryRecord::Preposition(p) => {
                    LexicalEntry::Preposition(Preposition { text: p.text })
                }
                LexicalEntryRecord::Conjunction(c) => {
                    LexicalEntry::Conjunction(Conjunction { text: c.text })
                }
                LexicalEntryRecord::Pronoun(p) => LexicalEntry::Pronoun(Pronoun {
                    text: p.text,
                    number: p.number,
                    person: p.person,
                    kind: p.kind,
                    olia_class: p.olia_class,
                }),
                LexicalEntryRecord::Copula(c) => LexicalEntry::Copula(Copula {
                    text: c.text,
                    number: c.number,
                    person: c.person,
                    tense: c.tense,
                }),
                LexicalEntryRecord::Auxiliary(a) => LexicalEntry::Auxiliary(Auxiliary {
                    text: a.text,
                    number: a.number,
                    tense: a.tense,
                }),
                LexicalEntryRecord::Interjection(i) => LexicalEntry::Interjection(Interjection {
                    text: i.text,
                    kind: i.kind,
                }),
                LexicalEntryRecord::Particle(p) => {
                    LexicalEntry::Particle(Particle { text: p.text })
                }
                LexicalEntryRecord::Numeral(n) => LexicalEntry::Numeral(Numeral { text: n.text }),
            }
        }
    }

    // ── materialize-one (hot read): &ArchivedLexicalEntryRecord → owned ───────

    /// Materialize ONE owned [`LexicalEntry`] from a single archived reading read
    /// straight out of the buffer — the zero-copy hot-path conversion
    /// [`first`](FunctionWordStore::first) / [`all`](FunctionWordStore::all) use.
    /// It deserializes exactly the one matched reading record (the
    /// [`concept_store`](super::super::concept_store) `ConceptView` precedent:
    /// read through the archived record, materialize only what the by-value trait
    /// needs) — it does NOT `rkyv::deserialize` the whole readings vector.
    impl From<&ArchivedLexicalEntryRecord> for LexicalEntry {
        fn from(archived: &ArchivedLexicalEntryRecord) -> Self {
            let record = rkyv::deserialize::<LexicalEntryRecord, rkyv::rancor::Error>(archived)
                .expect("the once-validated function-word archive must deserialize one record");
            LexicalEntry::from(record)
        }
    }

    // ── leaf lens: HashMap<String, Vec<LexicalEntry>> ⇄ FunctionWordRecords ───

    /// PUT leg: sort the owned map by raw UTF-8 key bytes (the order
    /// `slot`'s binary search assumes) and project into
    /// the parallel sorted-key / reading-vector mirror.
    impl RkyvMirror<HashMap<String, Vec<LexicalEntry>>> for FunctionWordRecords {
        fn from_owned(map: &HashMap<String, Vec<LexicalEntry>>) -> Self {
            let mut pairs: Vec<(&String, &Vec<LexicalEntry>)> = map.iter().collect();
            pairs.sort_unstable_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            let mut keys: Vec<String> = Vec::with_capacity(pairs.len());
            let mut entries: Vec<Vec<LexicalEntryRecord>> = Vec::with_capacity(pairs.len());
            for (k, v) in pairs {
                keys.push(k.clone());
                entries.push(v.iter().map(LexicalEntryRecord::from).collect());
            }
            FunctionWordRecords { keys, entries }
        }
    }

    /// Owned PUT leg: CONSUME the owned map, sort its pairs by raw UTF-8 key bytes
    /// (the same order [`from_owned`](RkyvMirror::from_owned) produces — keys are
    /// unique, so the sort is total and the order is identical), then MOVE each
    /// key and each reading (per-element via the by-value
    /// `From<LexicalEntry>` leaf) into the mirror rather
    /// than cloning them. Byte-identical to the borrowing leg.
    impl RkyvMirrorOwned<HashMap<String, Vec<LexicalEntry>>> for FunctionWordRecords {
        fn from_owned_value(map: HashMap<String, Vec<LexicalEntry>>) -> Self {
            let mut pairs: Vec<(String, Vec<LexicalEntry>)> = map.into_iter().collect();
            pairs.sort_unstable_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            let mut keys: Vec<String> = Vec::with_capacity(pairs.len());
            let mut entries: Vec<Vec<LexicalEntryRecord>> = Vec::with_capacity(pairs.len());
            for (k, v) in pairs {
                keys.push(k);
                entries.push(v.into_iter().map(LexicalEntryRecord::from).collect());
            }
            FunctionWordRecords { keys, entries }
        }
    }

    /// GET leg: rebuild the owned map from the parallel mirror. Total — the
    /// record → entry decode cannot fail (key order is immaterial to the map).
    impl RkyvOwned<FunctionWordRecords> for HashMap<String, Vec<LexicalEntry>> {
        type Error = core::convert::Infallible;
        fn from_mirror(mirror: FunctionWordRecords) -> Result<Self, core::convert::Infallible> {
            let mut map = HashMap::with_capacity(mirror.keys.len());
            for (k, recs) in mirror.keys.into_iter().zip(mirror.entries) {
                map.insert(k, recs.into_iter().map(LexicalEntry::from).collect());
            }
            Ok(map)
        }
    }

    /// The concrete lens for the function-word store instance.
    type FunctionWordLens = RkyvLens<HashMap<String, Vec<LexicalEntry>>, FunctionWordRecords>;

    type ArchivedRoot = rkyv::Archived<FunctionWordRecords>;

    /// The function-word lexicon, held as one `rkyv`-archived, immutable buffer.
    pub struct FunctionWordStore {
        buf: AlignedVec<16>,
        /// Number of distinct function words (`= keys.len()`), cached to answer
        /// [`len`](Self::len) without re-reading the archive root.
        len: usize,
    }

    impl FunctionWordStore {
        /// Transcode the owned `HashMap` build into the archived buffer ONCE,
        /// consuming and freeing the map. Keys are sorted by raw UTF-8 bytes so
        /// [`first`](Self::first) / [`all`](Self::all) can binary-search them.
        pub fn build(map: HashMap<alloc::string::String, Vec<LexicalEntry>>) -> Self {
            let len = map.len();

            // PUT the owned map through the shared `RkyvLens` (sort keys, project
            // the parallel mirror, `rkyv`-serialize to a 16-aligned buffer), then
            // validate ONCE here at materialize. CONSUME `map` through the OWNED
            // PUT leg, MOVING each key + reading into the mirror rather than
            // cloning them (byte-identical to `put_aligned(&map)` by
            // `RkyvLensOwnedPutAgrees`); only `buf` survives.
            let buf = FunctionWordLens::put_aligned_owned(map);
            FunctionWordLens::access(buf.as_slice())
                .expect("freshly-serialized function-word records must bytecheck-validate");

            Self { buf, len }
        }

        /// The archived root, borrowed zero-copy from the buffer.
        #[inline]
        fn root(&self) -> &ArchivedRoot {
            // SAFETY: `buf` was produced by `RkyvLens::put_aligned` (16-aligned,
            // sound), `bytecheck`-validated ONCE in `build`, and never mutated.
            unsafe { FunctionWordLens::access_unchecked(self.buf.as_slice()) }
        }

        /// The archive slot for `word`, by binary search over the sorted keys.
        #[inline]
        fn slot(&self, word: &str) -> Option<usize> {
            let keys = &self.root().keys;
            let target = word.as_bytes();
            let (mut lo, mut hi) = (0usize, self.len);
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                match keys[mid].as_bytes().cmp(target) {
                    core::cmp::Ordering::Less => lo = mid + 1,
                    core::cmp::Ordering::Greater => hi = mid,
                    core::cmp::Ordering::Equal => return Some(mid),
                }
            }
            None
        }

        /// The FIRST reading for `word` (`None` if absent) — the `lexical_lookup`
        /// reader, on the hot tokenizer path. Reads the matched readings vector
        /// ZERO-COPY through the archived buffer and materializes ONLY the first
        /// reading (the `ConceptView` precedent) — it does NOT `rkyv::deserialize`
        /// the whole readings vector.
        pub fn first(&self, word: &str) -> Option<LexicalEntry> {
            let i = self.slot(word)?;
            self.root().entries[i].first().map(LexicalEntry::from)
        }

        /// ALL readings for `word` (empty if absent) — the `lexical_lookup_all`
        /// reader. Materializes each reading lazily from the archived slice (one
        /// owned entry per element via the archived-record conversion), never a
        /// single whole-vector `rkyv::deserialize`.
        pub fn all(&self, word: &str) -> Vec<LexicalEntry> {
            let Some(i) = self.slot(word) else {
                return Vec::new();
            };
            self.root().entries[i]
                .iter()
                .map(LexicalEntry::from)
                .collect()
        }

        /// Every function-word text, as `&str`, in sorted order — read ZERO-COPY out
        /// of the archived keys (the `known_words` / `word_count` reader).
        pub fn words(&self) -> impl Iterator<Item = &str> {
            self.root().keys.iter().map(|k| k.as_str())
        }

        /// Number of distinct function words.
        pub fn len(&self) -> usize {
            self.len
        }
    }
}

// ── record cast unit tests (archived path) ───────────────────────────────────
//
// Direct coverage of the sorted-key dict + the LexicalEntry mirror round-trip: a
// small KNOWN function-word map (a determiner, a multi-reading surface, a pronoun
// carrying olia_class + a None field), asserting the archived reads reconstruct the
// exact entries, a miss → None/empty, AND the archived store reads identically to
// the owned fallback built from the SAME map.
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod cast_tests {
    use super::FunctionWordStore; // the archived store
    use super::owned::FunctionWordStore as OwnedStore; // the owned fallback
    use super::{HashMap, LexicalEntry};
    use crate::cognitive::linguistics::lexicon::pos::{
        Determiner, DeterminerKind, Number, Person, Pronoun, PronounKind,
    };
    use alloc::string::String;
    use alloc::vec::Vec;

    /// Keys sort by raw bytes to `the < what < who`, NOT insertion order.
    ///   "who"  → [Pronoun{olia_class: None}]          (single reading, None field)
    ///   "the"  → [Determiner{Definite}]               (single)
    ///   "what" → [Pronoun{Interrogative, olia}, Determiner]  (multi — order preserved)
    fn fixture() -> HashMap<String, Vec<LexicalEntry>> {
        let mut map: HashMap<String, Vec<LexicalEntry>> = HashMap::new();
        map.insert(
            String::from("who"),
            alloc::vec![LexicalEntry::Pronoun(Pronoun {
                text: String::from("who"),
                number: Number::Singular,
                person: Person::Third,
                kind: PronounKind::Interrogative,
                olia_class: None,
            })],
        );
        map.insert(
            String::from("the"),
            alloc::vec![LexicalEntry::Determiner(Determiner {
                text: String::from("the"),
                kind: DeterminerKind::Definite,
                number: None,
                olia_class: None,
            })],
        );
        map.insert(
            String::from("what"),
            alloc::vec![
                LexicalEntry::Pronoun(Pronoun {
                    text: String::from("what"),
                    number: Number::Singular,
                    person: Person::Third,
                    kind: PronounKind::Interrogative,
                    olia_class: Some(String::from("InterrogativePronoun")),
                }),
                LexicalEntry::Determiner(Determiner {
                    text: String::from("what"),
                    kind: DeterminerKind::Indefinite,
                    number: None,
                    olia_class: Some(String::from("InterrogativeDeterminer")),
                }),
            ],
        );
        map
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn archived_reads_match_the_known_map() {
        let store = FunctionWordStore::build(fixture());
        assert_eq!(store.len(), 3);

        // Single reading — first == all[0].
        assert_eq!(store.first("the"), Some(fixture()["the"][0].clone()));
        assert_eq!(store.all("the"), fixture()["the"]);

        // Multi-reading surface — order preserved, both readings recovered.
        assert_eq!(store.all("what"), fixture()["what"]);
        assert_eq!(store.first("what"), Some(fixture()["what"][0].clone()));

        // A reading carrying a None olia_class round-trips.
        assert_eq!(store.all("who"), fixture()["who"]);

        // Miss → None / empty.
        assert_eq!(store.first("zzz"), None);
        assert!(store.all("zzz").is_empty());

        // Sorted key set.
        let words: Vec<&str> = store.words().collect();
        assert_eq!(words, alloc::vec!["the", "what", "who"]);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn archived_is_identical_to_the_owned_fallback() {
        let archived = FunctionWordStore::build(fixture());
        let owned = OwnedStore::build(fixture());
        assert_eq!(archived.len(), owned.len());
        for word in ["the", "what", "who", "absent"] {
            assert_eq!(archived.first(word), owned.first(word), "first {word}");
            assert_eq!(archived.all(word), owned.all(word), "all {word}");
        }
        // Same key SET (owned order is unspecified — compare as sorted sets).
        let mut a: Vec<&str> = archived.words().collect();
        let mut o: Vec<&str> = owned.words().collect();
        a.sort_unstable();
        o.sort_unstable();
        assert_eq!(a, o);
    }
}

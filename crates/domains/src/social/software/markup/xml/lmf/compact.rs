//! Compact interned encoding of a WN-LMF [`WordNet`] — the size-reduced `.prx`
//! ontology core.
//!
//! The graph-faithful envelope stores [`WordNet`] **un-interned**: every synset
//! id, lemma, relation target, POS tag and gloss is repeated inline across
//! ~107k synsets and ~136k entries (78 MB raw / 17.6 MB gz for Open English
//! WordNet 2025). [`InternedWordNet`] stores each UNIQUE string ONCE in a
//! [`pool`](InternedWordNet::pool) and replaces every string field with a
//! [`Sym`] (`u32`) handle into it. The transform is LOSSLESS —
//! `deintern(intern(wn)) == wn` byte-for-byte at the struct level — so it is the
//! compact, COMPLETE ontology the `.prx` carries (whether the `.prx` is loaded
//! dynamically or embedded in the wasm; the encoding is the same bytes either
//! way). It materializes the SAME full [`English`](crate::cognitive::linguistics::english::English)
//! by de-interning to [`WordNet`] and running `English::from_wordnet` — no new
//! materialization path, no relation type dropped.
//!
//! This is the FIRST size lever (interning); the succinct layer (front-coded
//! dictionary + LOUDS/WebGraph-BV topology + FSST text, → ~3 MB) sits beneath
//! it as a later, research-backed encoding of the same `(pool, topology)` split.

use alloc::{string::String, string::ToString, vec::Vec};

use hashbrown::HashMap;

use super::ontology::{
    Count, Form, Lemma, LexicalEntry, LexiconMetadata, LmfPos, Pronunciation, Sense, SenseRelation,
    SenseRelationType, Synset, SynsetRelation, SynsetRelationType, SyntacticBehaviour, WordNet,
};

/// A handle into [`InternedWordNet::pool`] — the interned form of one string.
pub type Sym = u32;

/// The compact, interned, COMPLETE WordNet ontology — the `.prx` core.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct InternedWordNet {
    /// Every unique string, in first-seen order. The dictionary.
    pub pool: Vec<String>,
    pub lexicon: ILexiconMetadata,
    pub synsets: Vec<ISynset>,
    pub entries: Vec<ILexicalEntry>,
    pub syntactic_behaviours: Vec<ISyntacticBehaviour>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ILexiconMetadata {
    pub id: Option<Sym>,
    pub label: Option<Sym>,
    pub language: Option<Sym>,
    pub email: Option<Sym>,
    pub license: Option<Sym>,
    pub version: Option<Sym>,
    pub url: Option<Sym>,
    pub citation: Option<Sym>,
    pub logo: Option<Sym>,
    pub status: Option<Sym>,
    pub confidence_score: Option<Sym>,
    pub dc: Vec<(Sym, Sym)>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ISynset {
    pub id: Sym,
    pub ili: Option<Sym>,
    pub pos: LmfPos,
    pub members: Vec<Sym>,
    pub definitions: Vec<Sym>,
    pub ili_definition: Option<Sym>,
    pub examples: Vec<Sym>,
    pub relations: Vec<ISynsetRelation>,
    pub lexfile: Option<Sym>,
    pub dc_source: Option<Sym>,
    pub confidence_score: Option<Sym>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ISynsetRelation {
    pub rel_type: SynsetRelationType,
    pub target: Sym,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ILexicalEntry {
    pub id: Sym,
    pub lemma: ILemma,
    pub senses: Vec<ISense>,
    pub forms: Vec<IForm>,
    pub syntactic_behaviours: Vec<ISyntacticBehaviour>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ILemma {
    pub written_form: Sym,
    pub pos: LmfPos,
    pub script: Option<Sym>,
    pub pronunciations: Vec<IPronunciation>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct IPronunciation {
    pub text: Sym,
    pub variety: Option<Sym>,
    pub notation: Option<Sym>,
    pub phonemic: Option<Sym>,
    pub audio: Option<Sym>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ISense {
    pub id: Sym,
    pub synset: Sym,
    pub relations: Vec<ISenseRelation>,
    pub subcat: Vec<Sym>,
    pub adjposition: Option<Sym>,
    pub dc_source: Option<Sym>,
    pub counts: Vec<ICount>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ISenseRelation {
    pub rel_type: SenseRelationType,
    pub target: Sym,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct IForm {
    pub written_form: Sym,
    pub id: Option<Sym>,
    pub script: Option<Sym>,
    pub pronunciations: Vec<IPronunciation>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ISyntacticBehaviour {
    pub id: Option<Sym>,
    pub subcategorization_frame: Sym,
    pub senses: Vec<Sym>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ICount {
    pub value: Sym,
}

// ── intern: WordNet -> InternedWordNet ────────────────────────────────────

#[derive(Default)]
struct Interner {
    pool: Vec<String>,
    map: HashMap<String, Sym>,
}

impl Interner {
    fn s(&mut self, s: &str) -> Sym {
        if let Some(&id) = self.map.get(s) {
            return id;
        }
        let id = self.pool.len() as Sym;
        self.pool.push(s.to_string());
        self.map.insert(s.to_string(), id);
        id
    }
    fn opt(&mut self, o: &Option<String>) -> Option<Sym> {
        o.as_deref().map(|s| self.s(s))
    }
    fn many(&mut self, xs: &[String]) -> Vec<Sym> {
        xs.iter().map(|x| self.s(x)).collect()
    }
}

/// Intern a [`WordNet`] into its compact [`InternedWordNet`] form. Deterministic
/// (first-seen pool order over a fixed walk), so equal inputs give equal bytes.
pub fn intern(wn: &WordNet) -> InternedWordNet {
    let mut it = Interner::default();
    let lexicon = intern_lexicon(&mut it, &wn.lexicon);
    let synsets: Vec<ISynset> = wn
        .synsets
        .iter()
        .map(|s| intern_synset(&mut it, s))
        .collect();
    let entries: Vec<ILexicalEntry> = wn
        .entries
        .iter()
        .map(|e| intern_entry(&mut it, e))
        .collect();
    let syntactic_behaviours: Vec<ISyntacticBehaviour> = wn
        .syntactic_behaviours
        .iter()
        .map(|sb| intern_synbehav(&mut it, sb))
        .collect();
    InternedWordNet {
        pool: it.pool,
        lexicon,
        synsets,
        entries,
        syntactic_behaviours,
    }
}

fn intern_lexicon(it: &mut Interner, lx: &LexiconMetadata) -> ILexiconMetadata {
    ILexiconMetadata {
        id: it.opt(&lx.id),
        label: it.opt(&lx.label),
        language: it.opt(&lx.language),
        email: it.opt(&lx.email),
        license: it.opt(&lx.license),
        version: it.opt(&lx.version),
        url: it.opt(&lx.url),
        citation: it.opt(&lx.citation),
        logo: it.opt(&lx.logo),
        status: it.opt(&lx.status),
        confidence_score: it.opt(&lx.confidence_score),
        dc: lx.dc.iter().map(|(k, v)| (it.s(k), it.s(v))).collect(),
    }
}

fn intern_synset(it: &mut Interner, s: &Synset) -> ISynset {
    ISynset {
        id: it.s(&s.id),
        ili: it.opt(&s.ili),
        pos: s.pos,
        members: it.many(&s.members),
        definitions: it.many(&s.definitions),
        ili_definition: it.opt(&s.ili_definition),
        examples: it.many(&s.examples),
        relations: s
            .relations
            .iter()
            .map(|r| ISynsetRelation {
                rel_type: r.rel_type.clone(),
                target: it.s(&r.target),
            })
            .collect(),
        lexfile: it.opt(&s.lexfile),
        dc_source: it.opt(&s.dc_source),
        confidence_score: it.opt(&s.confidence_score),
    }
}

fn intern_entry(it: &mut Interner, e: &LexicalEntry) -> ILexicalEntry {
    ILexicalEntry {
        id: it.s(&e.id),
        lemma: intern_lemma(it, &e.lemma),
        senses: e.senses.iter().map(|s| intern_sense(it, s)).collect(),
        forms: e.forms.iter().map(|f| intern_form(it, f)).collect(),
        syntactic_behaviours: e
            .syntactic_behaviours
            .iter()
            .map(|sb| intern_synbehav(it, sb))
            .collect(),
    }
}

fn intern_lemma(it: &mut Interner, l: &Lemma) -> ILemma {
    ILemma {
        written_form: it.s(&l.written_form),
        pos: l.pos,
        script: it.opt(&l.script),
        pronunciations: l
            .pronunciations
            .iter()
            .map(|p| intern_pron(it, p))
            .collect(),
    }
}

fn intern_pron(it: &mut Interner, p: &Pronunciation) -> IPronunciation {
    IPronunciation {
        text: it.s(&p.text),
        variety: it.opt(&p.variety),
        notation: it.opt(&p.notation),
        phonemic: it.opt(&p.phonemic),
        audio: it.opt(&p.audio),
    }
}

fn intern_sense(it: &mut Interner, s: &Sense) -> ISense {
    ISense {
        id: it.s(&s.id),
        synset: it.s(&s.synset),
        relations: s
            .relations
            .iter()
            .map(|r| ISenseRelation {
                rel_type: r.rel_type.clone(),
                target: it.s(&r.target),
            })
            .collect(),
        subcat: it.many(&s.subcat),
        adjposition: it.opt(&s.adjposition),
        dc_source: it.opt(&s.dc_source),
        counts: s
            .counts
            .iter()
            .map(|c| ICount {
                value: it.s(&c.value),
            })
            .collect(),
    }
}

fn intern_form(it: &mut Interner, f: &Form) -> IForm {
    IForm {
        written_form: it.s(&f.written_form),
        id: it.opt(&f.id),
        script: it.opt(&f.script),
        pronunciations: f
            .pronunciations
            .iter()
            .map(|p| intern_pron(it, p))
            .collect(),
    }
}

fn intern_synbehav(it: &mut Interner, sb: &SyntacticBehaviour) -> ISyntacticBehaviour {
    ISyntacticBehaviour {
        id: it.opt(&sb.id),
        subcategorization_frame: it.s(&sb.subcategorization_frame),
        senses: it.many(&sb.senses),
    }
}

// ── deintern: InternedWordNet -> WordNet ──────────────────────────────────

/// De-intern back to a [`WordNet`]. The inverse of [`intern`]:
/// `deintern(&intern(wn)) == *wn`. Panics if a handle is out of range (only an
/// internally-inconsistent `InternedWordNet` could trigger it).
pub fn deintern(iwn: &InternedWordNet) -> WordNet {
    let p = iwn.pool.as_slice();
    WordNet {
        lexicon: deintern_lexicon(p, &iwn.lexicon),
        synsets: iwn.synsets.iter().map(|s| deintern_synset(p, s)).collect(),
        entries: iwn.entries.iter().map(|e| deintern_entry(p, e)).collect(),
        syntactic_behaviours: iwn
            .syntactic_behaviours
            .iter()
            .map(|sb| deintern_synbehav(p, sb))
            .collect(),
    }
}

fn g(p: &[String], sym: Sym) -> String {
    p[sym as usize].clone()
}
fn go(p: &[String], o: &Option<Sym>) -> Option<String> {
    o.map(|s| p[s as usize].clone())
}
fn gm(p: &[String], xs: &[Sym]) -> Vec<String> {
    xs.iter().map(|&s| p[s as usize].clone()).collect()
}

fn deintern_lexicon(p: &[String], lx: &ILexiconMetadata) -> LexiconMetadata {
    LexiconMetadata {
        id: go(p, &lx.id),
        label: go(p, &lx.label),
        language: go(p, &lx.language),
        email: go(p, &lx.email),
        license: go(p, &lx.license),
        version: go(p, &lx.version),
        url: go(p, &lx.url),
        citation: go(p, &lx.citation),
        logo: go(p, &lx.logo),
        status: go(p, &lx.status),
        confidence_score: go(p, &lx.confidence_score),
        dc: lx.dc.iter().map(|&(k, v)| (g(p, k), g(p, v))).collect(),
    }
}

fn deintern_synset(p: &[String], s: &ISynset) -> Synset {
    Synset {
        id: g(p, s.id),
        ili: go(p, &s.ili),
        pos: s.pos,
        members: gm(p, &s.members),
        definitions: gm(p, &s.definitions),
        ili_definition: go(p, &s.ili_definition),
        examples: gm(p, &s.examples),
        relations: s
            .relations
            .iter()
            .map(|r| SynsetRelation {
                rel_type: r.rel_type.clone(),
                target: g(p, r.target),
            })
            .collect(),
        lexfile: go(p, &s.lexfile),
        dc_source: go(p, &s.dc_source),
        confidence_score: go(p, &s.confidence_score),
    }
}

fn deintern_entry(p: &[String], e: &ILexicalEntry) -> LexicalEntry {
    LexicalEntry {
        id: g(p, e.id),
        lemma: deintern_lemma(p, &e.lemma),
        senses: e.senses.iter().map(|s| deintern_sense(p, s)).collect(),
        forms: e.forms.iter().map(|f| deintern_form(p, f)).collect(),
        syntactic_behaviours: e
            .syntactic_behaviours
            .iter()
            .map(|sb| deintern_synbehav(p, sb))
            .collect(),
    }
}

fn deintern_lemma(p: &[String], l: &ILemma) -> Lemma {
    Lemma {
        written_form: g(p, l.written_form),
        pos: l.pos,
        script: go(p, &l.script),
        pronunciations: l
            .pronunciations
            .iter()
            .map(|x| deintern_pron(p, x))
            .collect(),
    }
}

fn deintern_pron(p: &[String], x: &IPronunciation) -> Pronunciation {
    Pronunciation {
        text: g(p, x.text),
        variety: go(p, &x.variety),
        notation: go(p, &x.notation),
        phonemic: go(p, &x.phonemic),
        audio: go(p, &x.audio),
    }
}

fn deintern_sense(p: &[String], s: &ISense) -> Sense {
    Sense {
        id: g(p, s.id),
        synset: g(p, s.synset),
        relations: s
            .relations
            .iter()
            .map(|r| SenseRelation {
                rel_type: r.rel_type.clone(),
                target: g(p, r.target),
            })
            .collect(),
        subcat: gm(p, &s.subcat),
        adjposition: go(p, &s.adjposition),
        dc_source: go(p, &s.dc_source),
        counts: s
            .counts
            .iter()
            .map(|c| Count {
                value: g(p, c.value),
            })
            .collect(),
    }
}

fn deintern_form(p: &[String], f: &IForm) -> Form {
    Form {
        written_form: g(p, f.written_form),
        id: go(p, &f.id),
        script: go(p, &f.script),
        pronunciations: f
            .pronunciations
            .iter()
            .map(|x| deintern_pron(p, x))
            .collect(),
    }
}

fn deintern_synbehav(p: &[String], sb: &ISyntacticBehaviour) -> SyntacticBehaviour {
    SyntacticBehaviour {
        id: go(p, &sb.id),
        subcategorization_frame: g(p, sb.subcategorization_frame),
        senses: gm(p, &sb.senses),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use super::*;
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;

    fn gz_len(bytes: &[u8]) -> usize {
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        e.write_all(bytes).expect("gz write");
        e.finish().expect("gz finish").len()
    }

    /// The correctness GATE: interning is LOSSLESS. `deintern(intern(wn)) == wn`
    /// over the real corpora, AND the interned `.prx` core is materially smaller
    /// than the un-interned `WordNet` (the size win). Reads the tiny lexicon
    /// (instant) + the 89 MB english (one heavy parse); graceful skip if absent.
    #[test]
    fn intern_roundtrip_is_lossless_and_smaller() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let sources = [
            ("us_legal_lexicon", "data/legal-text/us_legal_lexicon.xml"),
            ("english_wordnet", "data/wordnet/english-wordnet-2025.xml"),
        ];
        let mut measured = 0usize;
        for (name, rel) in sources {
            let Ok(bytes) = std::fs::read(manifest.join(rel)) else {
                continue;
            };
            let text = core::str::from_utf8(&bytes).expect("UTF-8");
            let wn = read_wordnet(text).expect("parse WN-LMF");

            let iwn = intern(&wn);
            let back = deintern(&iwn);
            assert_eq!(
                back, wn,
                "{name}: intern -> deintern is NOT lossless (the .prx core would corrupt the ontology)"
            );

            let wn_raw = rkyv::to_bytes::<rkyv::rancor::Error>(&wn).expect("rkyv wn");
            let iwn_raw = rkyv::to_bytes::<rkyv::rancor::Error>(&iwn).expect("rkyv iwn");
            let wn_gz = gz_len(&wn_raw);
            let iwn_gz = gz_len(&iwn_raw);

            eprintln!(
                "COMPACT {name}: un-interned wn raw={:.2}MB/gz={:.2}MB  ->  interned .prx core \
                 raw={:.2}MB/gz={:.2}MB  ({:.2}x raw, {:.2}x gz smaller) || pool_strings={} \
                 synsets={} entries={}",
                wn_raw.len() as f64 / 1e6,
                wn_gz as f64 / 1e6,
                iwn_raw.len() as f64 / 1e6,
                iwn_gz as f64 / 1e6,
                wn_raw.len() as f64 / iwn_raw.len().max(1) as f64,
                wn_gz as f64 / iwn_gz.max(1) as f64,
                iwn.pool.len(),
                iwn.synsets.len(),
                iwn.entries.len(),
            );
            // The interned ENCODING is always smaller (less duplication) — that
            // is the `.prx` size win the user asked for, independent of the gz
            // wrapper.
            assert!(
                iwn_raw.len() < wn_raw.len(),
                "{name}: interned .prx core raw ({}) is not smaller than un-interned wn raw ({})",
                iwn_raw.len(),
                wn_raw.len()
            );
            // The gz win only materializes at CORPUS scale: gzip's 32 KB window
            // already dedups the small repeats in a tiny lexicon (so interning's
            // u32 handles can lose there), but it CANNOT dedup identical strings
            // megabytes apart — which interning does. Assert the gz win only for
            // large corpora; the tiny lexicon is reported, not gated.
            if wn_raw.len() > 1_000_000 {
                assert!(
                    iwn_gz < wn_gz,
                    "{name}: interned .prx core ({iwn_gz} gz) is not smaller than un-interned wn ({wn_gz} gz) at corpus scale"
                );
            }
            measured += 1;
        }
        assert!(
            measured >= 1,
            "no WN-LMF source on disk to exercise interning"
        );
    }
}

//! Compact, integer-addressed encoding of a WN-LMF [`WordNet`] — the `.prx`
//! ontology core, sized for the runtime (loaded dynamically or embedded in the
//! wasm; same bytes either way).
//!
//! # Why this shape
//!
//! The graph-faithful envelope stored [`WordNet`] **un-interned and string-
//! addressed**: every synset id, sense id, lemma, relation target and gloss
//! repeated inline (78 MB raw / 17.6 MB gz for Open English WordNet). Two
//! independent levers shrink that to the integer-addressed floor (~9–10 MB gz),
//! with NO change to the gz wrapper and NO succinct coding yet:
//!
//! 1. **Columnar layout** — parallel arrays per node class instead of a tree of
//!    nested structs, so rkyv stores one length-prefix / padding run per COLUMN
//!    instead of per element across ~107k synsets × ~11 fields.
//! 2. **Integer node-addressing** — a synset's array index IS its `ConceptId`
//!    (`from_wordnet` already assigns `ConceptId` by synset order, ontology.rs),
//!    a sense's global index IS its `SenseId`, an entry's index IS its position.
//!    Every cross-reference (relation targets, `sense.synset`, synset `members`
//!    → entry, syntactic-behaviour senses) becomes a `u32` index, so the
//!    `oewn-…` id-strings are NOT stored at all. The dictionary holds only
//!    LEXICAL text (lemmas, glosses, examples, frames, pronunciations, ILI
//!    codes, tags), deduplicated.
//!
//! [`encode`] drops the same dangling edges `English::from_wordnet` drops (an
//! edge whose endpoint id is unknown), so it is **reasoning-equivalent**: the
//! materialized [`English`](crate::cognitive::linguistics::english::English) is
//! identical (same `ConceptId`s, same relations, same word index) — only the
//! original `oewn-…` id-strings become index-derived synthetic ids (`s{i}` /
//! `k{g}` / `e{e}`), which the runtime never surfaces (it addresses by integer
//! `ConceptId`). It is NOT a byte-exact source round-trip — that decompile
//! property is deliberately not in the shipped `.prx`.
//!
//! The succinct layer (front-coded / FSST dictionary, LOUDS / WebGraph-BV
//! topology → ~3–5 MB) is the NEXT phase, encoding the same `(dict, columns)`
//! split; it is not applied here.

use alloc::{format, string::String, string::ToString, vec::Vec};

use hashbrown::HashMap;

use super::ontology::{
    Count, Form, Lemma, LexicalEntry, LexiconMetadata, LmfPos, Pronunciation, Sense, SenseRelation,
    SenseRelationType, Synset, SynsetRelation, SynsetRelationType, SyntacticBehaviour, WordNet,
};

/// An index into [`CompactWordNet::dict`] (a lexical string).
pub type Dict = u32;

/// The compact, integer-addressed, COMPLETE WordNet ontology — the `.prx` core.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct CompactWordNet {
    /// Every unique LEXICAL string, deduplicated (NO synset/sense/entry ids —
    /// those are array indices).
    pub dict: Vec<String>,
    pub lexicon: ILexiconMetadata,

    // ── synsets · array index == ConceptId ──
    pub syn_pos: Vec<LmfPos>,
    pub syn_ili: Vec<Option<Dict>>,
    pub syn_definitions: Vec<Vec<Dict>>,
    pub syn_ili_definition: Vec<Option<Dict>>,
    pub syn_examples: Vec<Vec<Dict>>,
    pub syn_relations: Vec<Vec<ISynRel>>,
    /// Synset → member LexicalEntry indices.
    pub syn_members: Vec<Vec<u32>>,
    pub syn_lexfile: Vec<Option<Dict>>,
    pub syn_dc_source: Vec<Option<Dict>>,
    pub syn_confidence: Vec<Option<Dict>>,

    // ── senses · global array index == SenseId (entry order) ──
    pub sense_synset: Vec<u32>,
    pub sense_relations: Vec<Vec<ISenRel>>,
    pub sense_subcat: Vec<Vec<Dict>>,
    pub sense_adjposition: Vec<Option<Dict>>,
    pub sense_dc_source: Vec<Option<Dict>>,
    pub sense_counts: Vec<Vec<Dict>>,

    // ── entries · array index == position; senses live in [start, start+count) ──
    pub entry_lemma_form: Vec<Dict>,
    pub entry_lemma_pos: Vec<LmfPos>,
    pub entry_lemma_script: Vec<Option<Dict>>,
    pub entry_lemma_prons: Vec<Vec<IPron>>,
    pub entry_sense_start: Vec<u32>,
    pub entry_sense_count: Vec<u32>,
    pub entry_forms: Vec<Vec<IForm>>,
    pub entry_synbehav: Vec<Vec<ISynBehav>>,

    pub lex_synbehav: Vec<ISynBehav>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ISynRel {
    pub rel_type: SynsetRelationType,
    /// Target synset index (ConceptId).
    pub target: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ISenRel {
    pub rel_type: SenseRelationType,
    /// Target sense global index (SenseId).
    pub target: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct IPron {
    pub text: Dict,
    pub variety: Option<Dict>,
    pub notation: Option<Dict>,
    pub phonemic: Option<Dict>,
    pub audio: Option<Dict>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct IForm {
    pub written_form: Dict,
    pub id: Option<Dict>,
    pub script: Option<Dict>,
    pub pronunciations: Vec<IPron>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ISynBehav {
    pub id: Option<Dict>,
    pub subcategorization_frame: Dict,
    /// Member sense global indices (SenseIds).
    pub senses: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ILexiconMetadata {
    pub id: Option<Dict>,
    pub label: Option<Dict>,
    pub language: Option<Dict>,
    pub email: Option<Dict>,
    pub license: Option<Dict>,
    pub version: Option<Dict>,
    pub url: Option<Dict>,
    pub citation: Option<Dict>,
    pub logo: Option<Dict>,
    pub status: Option<Dict>,
    pub confidence_score: Option<Dict>,
    pub dc: Vec<(Dict, Dict)>,
}

// ── encode: WordNet -> CompactWordNet ──────────────────────────────────────

#[derive(Default)]
struct DictBuilder {
    pool: Vec<String>,
    map: HashMap<String, Dict>,
}
impl DictBuilder {
    fn s(&mut self, s: &str) -> Dict {
        if let Some(&id) = self.map.get(s) {
            return id;
        }
        let id = self.pool.len() as Dict;
        self.pool.push(s.to_string());
        self.map.insert(s.to_string(), id);
        id
    }
    fn opt(&mut self, o: &Option<String>) -> Option<Dict> {
        o.as_deref().map(|s| self.s(s))
    }
    fn many(&mut self, xs: &[String]) -> Vec<Dict> {
        xs.iter().map(|x| self.s(x)).collect()
    }
}

/// Encode a parsed [`WordNet`] into the compact integer-addressed core.
/// Deterministic; equal inputs give equal bytes. Dangling edges (endpoint id
/// not present) are dropped — the SAME drop `English::from_wordnet` performs —
/// so the materialized ontology is unchanged.
pub fn encode(wn: &WordNet) -> CompactWordNet {
    let mut d = DictBuilder::default();

    // Index maps for the three id-spaces.
    let syn_idx: HashMap<&str, u32> = wn
        .synsets
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i as u32))
        .collect();
    let entry_idx: HashMap<&str, u32> = wn
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.id.as_str(), i as u32))
        .collect();
    // Global sense index in entry order.
    let mut sense_idx: HashMap<&str, u32> = HashMap::new();
    {
        let mut g = 0u32;
        for e in &wn.entries {
            for s in &e.senses {
                sense_idx.insert(s.id.as_str(), g);
                g += 1;
            }
        }
    }

    let lexicon = encode_lexicon(&mut d, &wn.lexicon);

    // Synsets.
    let n = wn.synsets.len();
    let mut syn_pos = Vec::with_capacity(n);
    let mut syn_ili = Vec::with_capacity(n);
    let mut syn_definitions = Vec::with_capacity(n);
    let mut syn_ili_definition = Vec::with_capacity(n);
    let mut syn_examples = Vec::with_capacity(n);
    let mut syn_relations = Vec::with_capacity(n);
    let mut syn_members = Vec::with_capacity(n);
    let mut syn_lexfile = Vec::with_capacity(n);
    let mut syn_dc_source = Vec::with_capacity(n);
    let mut syn_confidence = Vec::with_capacity(n);
    for s in &wn.synsets {
        syn_pos.push(s.pos);
        syn_ili.push(d.opt(&s.ili));
        syn_definitions.push(d.many(&s.definitions));
        syn_ili_definition.push(d.opt(&s.ili_definition));
        syn_examples.push(d.many(&s.examples));
        syn_relations.push(
            s.relations
                .iter()
                .filter_map(|r| {
                    syn_idx.get(r.target.as_str()).map(|&t| ISynRel {
                        rel_type: r.rel_type.clone(),
                        target: t,
                    })
                })
                .collect(),
        );
        syn_members.push(
            s.members
                .iter()
                .filter_map(|m| entry_idx.get(m.as_str()).copied())
                .collect(),
        );
        syn_lexfile.push(d.opt(&s.lexfile));
        syn_dc_source.push(d.opt(&s.dc_source));
        syn_confidence.push(d.opt(&s.confidence_score));
    }

    // Senses (global, entry order) + entry columns.
    let mut sense_synset = Vec::new();
    let mut sense_relations = Vec::new();
    let mut sense_subcat = Vec::new();
    let mut sense_adjposition = Vec::new();
    let mut sense_dc_source = Vec::new();
    let mut sense_counts = Vec::new();

    let m = wn.entries.len();
    let mut entry_lemma_form = Vec::with_capacity(m);
    let mut entry_lemma_pos = Vec::with_capacity(m);
    let mut entry_lemma_script = Vec::with_capacity(m);
    let mut entry_lemma_prons = Vec::with_capacity(m);
    let mut entry_sense_start = Vec::with_capacity(m);
    let mut entry_sense_count = Vec::with_capacity(m);
    let mut entry_forms = Vec::with_capacity(m);
    let mut entry_synbehav = Vec::with_capacity(m);

    for e in &wn.entries {
        entry_lemma_form.push(d.s(&e.lemma.written_form));
        entry_lemma_pos.push(e.lemma.pos);
        entry_lemma_script.push(d.opt(&e.lemma.script));
        entry_lemma_prons.push(encode_prons(&mut d, &e.lemma.pronunciations));

        entry_sense_start.push(sense_synset.len() as u32);
        entry_sense_count.push(e.senses.len() as u32);
        for s in &e.senses {
            sense_synset.push(*syn_idx.get(s.synset.as_str()).unwrap_or(&u32::MAX));
            sense_relations.push(
                s.relations
                    .iter()
                    .filter_map(|r| {
                        sense_idx.get(r.target.as_str()).map(|&t| ISenRel {
                            rel_type: r.rel_type.clone(),
                            target: t,
                        })
                    })
                    .collect(),
            );
            sense_subcat.push(d.many(&s.subcat));
            sense_adjposition.push(d.opt(&s.adjposition));
            sense_dc_source.push(d.opt(&s.dc_source));
            sense_counts.push(s.counts.iter().map(|c| d.s(&c.value)).collect());
        }

        entry_forms.push(e.forms.iter().map(|f| encode_form(&mut d, f)).collect());
        entry_synbehav.push(
            e.syntactic_behaviours
                .iter()
                .map(|sb| encode_synbehav(&mut d, &sense_idx, sb))
                .collect(),
        );
    }

    let lex_synbehav = wn
        .syntactic_behaviours
        .iter()
        .map(|sb| encode_synbehav(&mut d, &sense_idx, sb))
        .collect();

    CompactWordNet {
        dict: d.pool,
        lexicon,
        syn_pos,
        syn_ili,
        syn_definitions,
        syn_ili_definition,
        syn_examples,
        syn_relations,
        syn_members,
        syn_lexfile,
        syn_dc_source,
        syn_confidence,
        sense_synset,
        sense_relations,
        sense_subcat,
        sense_adjposition,
        sense_dc_source,
        sense_counts,
        entry_lemma_form,
        entry_lemma_pos,
        entry_lemma_script,
        entry_lemma_prons,
        entry_sense_start,
        entry_sense_count,
        entry_forms,
        entry_synbehav,
        lex_synbehav,
    }
}

fn encode_lexicon(d: &mut DictBuilder, lx: &LexiconMetadata) -> ILexiconMetadata {
    ILexiconMetadata {
        id: d.opt(&lx.id),
        label: d.opt(&lx.label),
        language: d.opt(&lx.language),
        email: d.opt(&lx.email),
        license: d.opt(&lx.license),
        version: d.opt(&lx.version),
        url: d.opt(&lx.url),
        citation: d.opt(&lx.citation),
        logo: d.opt(&lx.logo),
        status: d.opt(&lx.status),
        confidence_score: d.opt(&lx.confidence_score),
        dc: lx.dc.iter().map(|(k, v)| (d.s(k), d.s(v))).collect(),
    }
}

fn encode_prons(d: &mut DictBuilder, ps: &[Pronunciation]) -> Vec<IPron> {
    ps.iter()
        .map(|p| IPron {
            text: d.s(&p.text),
            variety: d.opt(&p.variety),
            notation: d.opt(&p.notation),
            phonemic: d.opt(&p.phonemic),
            audio: d.opt(&p.audio),
        })
        .collect()
}

fn encode_form(d: &mut DictBuilder, f: &Form) -> IForm {
    IForm {
        written_form: d.s(&f.written_form),
        id: d.opt(&f.id),
        script: d.opt(&f.script),
        pronunciations: encode_prons(d, &f.pronunciations),
    }
}

fn encode_synbehav(
    d: &mut DictBuilder,
    sense_idx: &HashMap<&str, u32>,
    sb: &SyntacticBehaviour,
) -> ISynBehav {
    ISynBehav {
        id: d.opt(&sb.id),
        subcategorization_frame: d.s(&sb.subcategorization_frame),
        senses: sb
            .senses
            .iter()
            .filter_map(|s| sense_idx.get(s.as_str()).copied())
            .collect(),
    }
}

// ── decode: CompactWordNet -> WordNet (index-derived synthetic ids) ─────────

fn syn_id(i: usize) -> String {
    format!("s{i}")
}
fn sense_id(g: usize) -> String {
    format!("k{g}")
}
fn entry_id(e: usize) -> String {
    format!("e{e}")
}

/// Decode back to a reasoning-equivalent [`WordNet`]: identical graph, glosses
/// and word index, with index-derived synthetic ids in place of the original
/// `oewn-…` strings. `English::from_wordnet` on the result is identical to
/// `from_wordnet` on the original (same `ConceptId`s, same relations) — the only
/// difference is `Concept::original_id`, which the runtime addresses by integer.
pub fn decode(c: &CompactWordNet) -> WordNet {
    let dict = c.dict.as_slice();
    let g = |i: Dict| dict[i as usize].clone();
    let go = |o: &Option<Dict>| o.map(|i| dict[i as usize].clone());
    let gm = |xs: &[Dict]| {
        xs.iter()
            .map(|&i| dict[i as usize].clone())
            .collect::<Vec<_>>()
    };

    let lexicon = LexiconMetadata {
        id: go(&c.lexicon.id),
        label: go(&c.lexicon.label),
        language: go(&c.lexicon.language),
        email: go(&c.lexicon.email),
        license: go(&c.lexicon.license),
        version: go(&c.lexicon.version),
        url: go(&c.lexicon.url),
        citation: go(&c.lexicon.citation),
        logo: go(&c.lexicon.logo),
        status: go(&c.lexicon.status),
        confidence_score: go(&c.lexicon.confidence_score),
        dc: c.lexicon.dc.iter().map(|&(k, v)| (g(k), g(v))).collect(),
    };

    let synsets: Vec<Synset> = (0..c.syn_pos.len())
        .map(|i| Synset {
            id: syn_id(i),
            ili: go(&c.syn_ili[i]),
            pos: c.syn_pos[i],
            members: c.syn_members[i]
                .iter()
                .map(|&e| entry_id(e as usize))
                .collect(),
            definitions: gm(&c.syn_definitions[i]),
            ili_definition: go(&c.syn_ili_definition[i]),
            examples: gm(&c.syn_examples[i]),
            relations: c.syn_relations[i]
                .iter()
                .map(|r| SynsetRelation {
                    rel_type: r.rel_type.clone(),
                    target: syn_id(r.target as usize),
                })
                .collect(),
            lexfile: go(&c.syn_lexfile[i]),
            dc_source: go(&c.syn_dc_source[i]),
            confidence_score: go(&c.syn_confidence[i]),
        })
        .collect();

    let decode_sense = |gx: usize| Sense {
        id: sense_id(gx),
        synset: syn_id(c.sense_synset[gx] as usize),
        relations: c.sense_relations[gx]
            .iter()
            .map(|r| SenseRelation {
                rel_type: r.rel_type.clone(),
                target: sense_id(r.target as usize),
            })
            .collect(),
        subcat: gm(&c.sense_subcat[gx]),
        adjposition: go(&c.sense_adjposition[gx]),
        dc_source: go(&c.sense_dc_source[gx]),
        counts: c.sense_counts[gx]
            .iter()
            .map(|&v| Count { value: g(v) })
            .collect(),
    };

    let entries: Vec<LexicalEntry> = (0..c.entry_lemma_form.len())
        .map(|e| {
            let start = c.entry_sense_start[e] as usize;
            let count = c.entry_sense_count[e] as usize;
            LexicalEntry {
                id: entry_id(e),
                lemma: Lemma {
                    written_form: g(c.entry_lemma_form[e]),
                    pos: c.entry_lemma_pos[e],
                    script: go(&c.entry_lemma_script[e]),
                    pronunciations: decode_prons(dict, &c.entry_lemma_prons[e]),
                },
                senses: (start..start + count).map(decode_sense).collect(),
                forms: c.entry_forms[e]
                    .iter()
                    .map(|f| decode_form(dict, f))
                    .collect(),
                syntactic_behaviours: c.entry_synbehav[e]
                    .iter()
                    .map(|sb| decode_synbehav(dict, sb))
                    .collect(),
            }
        })
        .collect();

    WordNet {
        lexicon,
        synsets,
        entries,
        syntactic_behaviours: c
            .lex_synbehav
            .iter()
            .map(|sb| decode_synbehav(dict, sb))
            .collect(),
    }
}

fn decode_prons(dict: &[String], ps: &[IPron]) -> Vec<Pronunciation> {
    let go = |o: &Option<Dict>| o.map(|i| dict[i as usize].clone());
    ps.iter()
        .map(|p| Pronunciation {
            text: dict[p.text as usize].clone(),
            variety: go(&p.variety),
            notation: go(&p.notation),
            phonemic: go(&p.phonemic),
            audio: go(&p.audio),
        })
        .collect()
}

fn decode_form(dict: &[String], f: &IForm) -> Form {
    let go = |o: &Option<Dict>| o.map(|i| dict[i as usize].clone());
    Form {
        written_form: dict[f.written_form as usize].clone(),
        id: go(&f.id),
        script: go(&f.script),
        pronunciations: decode_prons(dict, &f.pronunciations),
    }
}

fn decode_synbehav(dict: &[String], sb: &ISynBehav) -> SyntacticBehaviour {
    SyntacticBehaviour {
        id: sb.id.map(|i| dict[i as usize].clone()),
        subcategorization_frame: dict[sb.subcategorization_frame as usize].clone(),
        senses: sb.senses.iter().map(|&s| sense_id(s as usize)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use super::*;
    use crate::cognitive::linguistics::english::English;
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;

    fn gz_len(bytes: &[u8]) -> usize {
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        e.write_all(bytes).expect("gz write");
        e.finish().expect("gz finish").len()
    }

    /// The 9.5 MB milestone: the compact integer-addressed `.prx` core is
    /// REASONING-EQUIVALENT to the source `WordNet` (materializes the same
    /// `English`), and materially smaller than the un-interned graph — with NO
    /// succinct coding and gz unchanged. Reads the tiny lexicon (instant) + the
    /// 89 MB english (one heavy parse); graceful skip if absent.
    #[test]
    fn compact_is_reasoning_equivalent_and_small() {
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

            let compact = encode(&wn);
            let wn2 = decode(&compact);

            // Reasoning-equivalence: from_wordnet over the original and over the
            // decoded (synthetic-id) WordNet build the SAME English — same
            // concept count and same word→concept index (the integer addressing
            // preserves every ConceptId, since both assign by synset order).
            let e_orig = English::from_wordnet(&wn);
            let e_compact = English::from_wordnet(&wn2);
            assert_eq!(
                e_orig.concept_count(),
                e_compact.concept_count(),
                "{name}: concept_count differs — the compact core dropped concepts"
            );
            assert_eq!(
                e_orig.word_index, e_compact.word_index,
                "{name}: word→concept index differs — lexical addressing not preserved"
            );

            let wn_raw = rkyv::to_bytes::<rkyv::rancor::Error>(&wn).expect("rkyv wn");
            let c_raw = rkyv::to_bytes::<rkyv::rancor::Error>(&compact).expect("rkyv compact");
            let wn_gz = gz_len(&wn_raw);
            let prx_gz = gz_len(&c_raw); // THE shipped .prx (rkyv core, gz-wrapped)
            // What you fetch to obtain the source itself: the distributed .xml.gz.
            let source_download = gz_len(&bytes);

            eprintln!(
                "COMPACT9 {name}: .prx = {:.2}MB gz  ({:.2}MB uncompressed/mmap)   vs   downloaded \
                 source (.xml.gz) = {:.2}MB   ->   .prx is {:.2}x the download || dict={} \
                 synsets={} senses={} entries={}  (un-interned wn ref: {:.2}MB gz)",
                prx_gz as f64 / 1e6,
                c_raw.len() as f64 / 1e6,
                source_download as f64 / 1e6,
                prx_gz as f64 / source_download.max(1) as f64,
                compact.dict.len(),
                compact.syn_pos.len(),
                compact.sense_synset.len(),
                compact.entry_lemma_form.len(),
                wn_gz as f64 / 1e6,
            );
            assert!(
                c_raw.len() < wn_raw.len(),
                "{name}: compact .prx raw ({}) not smaller than un-interned wn ({})",
                c_raw.len(),
                wn_raw.len()
            );
            measured += 1;
        }
        assert!(
            measured >= 1,
            "no WN-LMF source on disk to exercise the compact core"
        );
    }

    fn write_varint(out: &mut Vec<u8>, mut n: u64) {
        loop {
            let b = (n & 0x7f) as u8;
            n >>= 7;
            if n == 0 {
                out.push(b);
                break;
            }
            out.push(b | 0x80);
        }
    }

    /// Front-code a SORTED string list (HDT's queryable dictionary technique):
    /// for each entry store `varint(shared_prefix_len) varint(suffix_len) suffix`,
    /// so a shared prefix with the previous entry is never repeated. Stays
    /// binary-searchable. Returns the encoded bytes.
    fn front_code(sorted: &[String]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut prev: &[u8] = b"";
        for s in sorted {
            let b = s.as_bytes();
            let shared = prev.iter().zip(b).take_while(|(x, y)| x == y).count();
            write_varint(&mut out, shared as u64);
            write_varint(&mut out, (b.len() - shared) as u64);
            out.extend_from_slice(&b[shared..]);
            prev = b;
        }
        out
    }

    /// SUCCINCT-PHASE MEASURE-FIRST: split the compact `.prx` encoding into its
    /// dictionary (strings → front-coding/FSST target) and its structure (the
    /// graph → LOUDS/WebGraph-BV target), so the bigger lever is built first; and
    /// measure the front-coded-dictionary win (dependency-free, queryable). No
    /// crate yet. Graceful skip if the corpus is absent.
    #[test]
    fn succinct_floor_breakdown() {
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
            let wn = read_wordnet(core::str::from_utf8(&bytes).expect("UTF-8")).expect("parse");
            let compact = encode(&wn);

            let total = rkyv::to_bytes::<rkyv::rancor::Error>(&compact).expect("rkyv total");
            let dict_b = rkyv::to_bytes::<rkyv::rancor::Error>(&compact.dict).expect("rkyv dict");
            let struct_raw = total.len().saturating_sub(dict_b.len());

            let mut sorted = compact.dict.clone();
            sorted.sort();
            let fc = front_code(&sorted);

            eprintln!(
                "SUCCINCT {name}: .prx total raw={:.2}MB/gz={:.2}MB || DICT raw={:.2}MB/gz={:.2}MB  \
                 STRUCTURE raw≈{:.2}MB || dict front-coded raw={:.2}MB/gz={:.2}MB ({:.2}x vs dict \
                 rkyv raw, {:.2}x vs dict gz) || dict_strings={}",
                total.len() as f64 / 1e6,
                gz_len(&total) as f64 / 1e6,
                dict_b.len() as f64 / 1e6,
                gz_len(&dict_b) as f64 / 1e6,
                struct_raw as f64 / 1e6,
                fc.len() as f64 / 1e6,
                gz_len(&fc) as f64 / 1e6,
                dict_b.len() as f64 / fc.len().max(1) as f64,
                gz_len(&dict_b) as f64 / gz_len(&fc).max(1) as f64,
                compact.dict.len(),
            );
            measured += 1;
        }
        assert!(
            measured >= 1,
            "no WN-LMF source on disk for the succinct breakdown"
        );
    }
}

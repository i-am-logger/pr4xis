//! Compact, integer-addressed encoding of a WN-LMF [`WordNet`] — the `.prx`
//! ontology core (loaded dynamically or embedded in the wasm; same bytes either
//! way). The succinct serialization is [`super::compact_succinct`].
//!
//! Two properties make it compact:
//!
//! 1. **Columnar layout** — parallel arrays per node class rather than nested
//!    structs, so the encoding carries one length-prefix per COLUMN, not per
//!    element across ~107k synsets × ~11 fields.
//! 2. **Integer node-addressing** — a synset's array index IS its `ConceptId`
//!    (`from_wordnet` assigns `ConceptId` by synset order), a sense's global
//!    index IS its `SenseId`, an entry's index IS its position. Every
//!    cross-reference (relation targets, `sense.synset`, synset `members` →
//!    entry, syntactic-behaviour senses) is a `u32` index, so the `oewn-…`
//!    id-strings are not stored; the dictionary holds only lexical text (lemmas,
//!    glosses, examples, frames, pronunciations, ILI codes, tags), deduplicated.
//!
//! [`encode`] drops dangling edges (endpoint id absent), the same ones
//! `English::from_wordnet` drops, so the encoding is REASONING-EQUIVALENT: the
//! materialized [`English`](crate::cognitive::linguistics::english::English) is
//! identical (same `ConceptId`s, relations, word index); only a `Concept`'s
//! original id becomes an index-derived synthetic id (`s{i}` / `k{g}` / `e{e}`),
//! which the runtime never surfaces. It is not a byte-exact source round-trip.

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

    let built = CompactWordNet {
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
    };
    // Canonicalize the dictionary to SORTED order (remapping every dict
    // reference), so the codec's front-coding finds shared prefixes between
    // adjacent entries. Node-index references are untouched; reasoning-
    // equivalence is preserved (each reference still resolves to the same
    // string).
    canonicalize_dict(built)
}

/// Sort [`CompactWordNet::dict`] lexicographically and remap every dictionary
/// reference to the new index. Node-index references (relation targets,
/// `sense_synset`, members, synbehav senses, sense ranges) are NOT touched.
fn canonicalize_dict(mut c: CompactWordNet) -> CompactWordNet {
    let n = c.dict.len();
    let mut order: Vec<u32> = (0..n as u32).collect();
    order.sort_by(|&a, &b| c.dict[a as usize].cmp(&c.dict[b as usize]));
    let mut remap = alloc::vec![0u32; n];
    for (new, &old) in order.iter().enumerate() {
        remap[old as usize] = new as u32;
    }
    let new_dict: Vec<String> = order
        .iter()
        .map(|&old| c.dict[old as usize].clone())
        .collect();

    let rd = |d: &mut Dict| *d = remap[*d as usize];
    let rdo = |o: &mut Option<Dict>| {
        if let Some(d) = o {
            *d = remap[*d as usize];
        }
    };
    let remap_pron = |p: &mut IPron| {
        rd(&mut p.text);
        rdo(&mut p.variety);
        rdo(&mut p.notation);
        rdo(&mut p.phonemic);
        rdo(&mut p.audio);
    };

    c.syn_ili.iter_mut().for_each(&rdo);
    c.syn_definitions
        .iter_mut()
        .for_each(|v| v.iter_mut().for_each(&rd));
    c.syn_ili_definition.iter_mut().for_each(&rdo);
    c.syn_examples
        .iter_mut()
        .for_each(|v| v.iter_mut().for_each(&rd));
    c.syn_lexfile.iter_mut().for_each(&rdo);
    c.syn_dc_source.iter_mut().for_each(&rdo);
    c.syn_confidence.iter_mut().for_each(&rdo);

    c.sense_subcat
        .iter_mut()
        .for_each(|v| v.iter_mut().for_each(&rd));
    c.sense_adjposition.iter_mut().for_each(&rdo);
    c.sense_dc_source.iter_mut().for_each(&rdo);
    c.sense_counts
        .iter_mut()
        .for_each(|v| v.iter_mut().for_each(&rd));

    c.entry_lemma_form.iter_mut().for_each(&rd);
    c.entry_lemma_script.iter_mut().for_each(&rdo);
    c.entry_lemma_prons
        .iter_mut()
        .for_each(|ps| ps.iter_mut().for_each(&remap_pron));
    c.entry_forms.iter_mut().for_each(|fs| {
        fs.iter_mut().for_each(|f| {
            rd(&mut f.written_form);
            rdo(&mut f.id);
            rdo(&mut f.script);
            f.pronunciations.iter_mut().for_each(&remap_pron);
        })
    });

    let remap_sb = |sb: &mut ISynBehav| {
        rdo(&mut sb.id);
        rd(&mut sb.subcategorization_frame);
    };
    c.entry_synbehav
        .iter_mut()
        .for_each(|v| v.iter_mut().for_each(&remap_sb));
    c.lex_synbehav.iter_mut().for_each(&remap_sb);

    rdo(&mut c.lexicon.id);
    rdo(&mut c.lexicon.label);
    rdo(&mut c.lexicon.language);
    rdo(&mut c.lexicon.email);
    rdo(&mut c.lexicon.license);
    rdo(&mut c.lexicon.version);
    rdo(&mut c.lexicon.url);
    rdo(&mut c.lexicon.citation);
    rdo(&mut c.lexicon.logo);
    rdo(&mut c.lexicon.status);
    rdo(&mut c.lexicon.confidence_score);
    c.lexicon.dc.iter_mut().for_each(|(k, v)| {
        rd(k);
        rd(v);
    });

    // Sort each node's relations by target so the succinct codec can delta-code
    // the adjacency (per-node ascending gaps). Reasoning-equivalent: a synset's
    // / sense's relations are a SET — order carries no meaning.
    c.syn_relations
        .iter_mut()
        .for_each(|v| v.sort_by_key(|r| r.target));
    c.sense_relations
        .iter_mut()
        .for_each(|v| v.sort_by_key(|r| r.target));

    c.dict = new_dict;
    c
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

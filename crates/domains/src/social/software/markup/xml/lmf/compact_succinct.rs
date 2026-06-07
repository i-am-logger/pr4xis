//! Succinct codec for [`CompactWordNet`]:
//! [`to_succinct`] / [`from_succinct`] serialize the compact ontology to the
//! `.prx` bytes (embedded as `include_bytes!` of the `.gz`, or downloaded, then
//! gunzipped and decoded at load).
//!
//! Each columnar value uses `bits(max)` bits, LSB-first, in CSR (offset +
//! flat-value) layout. CSR offsets are stored as their per-node-length gaps;
//! relation adjacency is sorted per node and delta-coded. The lexical dictionary
//! is front-coded (sorted; shared prefixes elided). Relation types map through a
//! small string dictionary and reconstruct via `parse(as_str(_))`. The nested
//! tails (pronunciations, forms, syntactic behaviours, lexicon metadata) are one
//! rkyv blob.
//!
//! The bit-packing is pure `u64`-accumulator arithmetic with no dependency, so
//! the `.prx` decodes on any target including wasm32.
//! `from_succinct(&to_succinct(c)) == c`, composing with
//! [`super::compact::decode`] → `from_wordnet`.

use alloc::{string::String, vec::Vec};

use hashbrown::HashMap;

use super::compact::{CompactWordNet, IForm, ILexiconMetadata, IPron, ISynBehav, decode, encode};
use super::ontology::{LmfPos, SenseRelationType, SynsetRelationType, WordNet};
use crate::cognitive::linguistics::english::English;
use crate::social::software::markup::xml::succinct::{
    get_blob, get_csr, get_cv, get_delta, get_dict, get_dict_fc, get_ef, get_opt, get_varint,
    put_blob, put_csr, put_cv, put_delta, put_dict, put_dict_fc, put_ef, put_opt, put_varint,
};

// ── LmfPos <-> u8 (a closed enum; an exact bijection) ──────────────────────

fn pos_to_u8(p: LmfPos) -> u8 {
    match p {
        LmfPos::Noun => 0,
        LmfPos::Verb => 1,
        LmfPos::Adjective => 2,
        LmfPos::SatelliteAdjective => 3,
        LmfPos::Adverb => 4,
        LmfPos::Determiner => 5,
        LmfPos::Pronoun => 6,
        LmfPos::Preposition => 7,
        LmfPos::Conjunction => 8,
        LmfPos::Particle => 9,
        LmfPos::Copula => 10,
        LmfPos::Auxiliary => 11,
        LmfPos::Interjection => 12,
        LmfPos::Numeral => 13,
        LmfPos::Other => 14,
    }
}

fn pos_from_u8(b: u8) -> LmfPos {
    match b {
        0 => LmfPos::Noun,
        1 => LmfPos::Verb,
        2 => LmfPos::Adjective,
        3 => LmfPos::SatelliteAdjective,
        4 => LmfPos::Adverb,
        5 => LmfPos::Determiner,
        6 => LmfPos::Pronoun,
        7 => LmfPos::Preposition,
        8 => LmfPos::Conjunction,
        9 => LmfPos::Particle,
        10 => LmfPos::Copula,
        11 => LmfPos::Auxiliary,
        12 => LmfPos::Interjection,
        13 => LmfPos::Numeral,
        _ => LmfPos::Other,
    }
}

fn put_pos(out: &mut Vec<u8>, ps: &[LmfPos]) {
    put_cv(
        out,
        &ps.iter()
            .map(|&p| pos_to_u8(p) as usize)
            .collect::<Vec<_>>(),
    );
}
fn get_pos(buf: &[u8], pos: &mut usize) -> Vec<LmfPos> {
    get_cv(buf, pos)
        .into_iter()
        .map(|v| pos_from_u8(v as u8))
        .collect()
}

// ── the small nested tails, carried as one rkyv blob ──────────────────────

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct ComplexTail {
    entry_lemma_prons: Vec<Vec<IPron>>,
    entry_forms: Vec<Vec<IForm>>,
    entry_synbehav: Vec<Vec<ISynBehav>>,
    lex_synbehav: Vec<ISynBehav>,
    lexicon: ILexiconMetadata,
}

// ── the codec ─────────────────────────────────────────────────────────────

/// Encode a [`CompactWordNet`] into the succinct `.prx` bytes.
pub fn to_succinct(c: &CompactWordNet) -> Vec<u8> {
    let mut out = Vec::new();

    put_varint(&mut out, c.syn_pos.len() as u64);
    put_varint(&mut out, c.sense_synset.len() as u64);
    put_varint(&mut out, c.entry_lemma_form.len() as u64);

    put_dict_fc(&mut out, &c.dict);

    // Relation-type string dictionaries (distinct as_str values), so each
    // relation stores a tiny index; parse(as_str) reconstructs the enum.
    let mut syn_rt: Vec<String> = Vec::new();
    let mut syn_rt_idx: HashMap<&str, usize> = HashMap::new();
    let mut sense_rt: Vec<String> = Vec::new();
    let mut sense_rt_idx: HashMap<&str, usize> = HashMap::new();
    for v in &c.syn_relations {
        for r in v {
            let s = r.rel_type.as_str();
            if !syn_rt_idx.contains_key(s) {
                syn_rt_idx.insert(s, syn_rt.len());
                syn_rt.push(s.into());
            }
        }
    }
    for v in &c.sense_relations {
        for r in v {
            let s = r.rel_type.as_str();
            if !sense_rt_idx.contains_key(s) {
                sense_rt_idx.insert(s, sense_rt.len());
                sense_rt.push(s.into());
            }
        }
    }
    put_dict(&mut out, &syn_rt);
    put_dict(&mut out, &sense_rt);

    // Synset columns.
    put_pos(&mut out, &c.syn_pos);
    put_opt(&mut out, &c.syn_ili);
    put_csr(&mut out, &c.syn_definitions);
    put_opt(&mut out, &c.syn_ili_definition);
    put_csr(&mut out, &c.syn_examples);
    // syn_relations: offsets + reltype-idx + target.
    {
        let mut offsets = Vec::with_capacity(c.syn_relations.len() + 1);
        let mut rts = Vec::new();
        let mut tgts = Vec::new();
        let mut acc = 0usize;
        offsets.push(0);
        for v in &c.syn_relations {
            acc += v.len();
            offsets.push(acc);
            for r in v {
                rts.push(syn_rt_idx[r.rel_type.as_str()]);
                tgts.push(r.target as usize);
            }
        }
        put_ef(&mut out, &offsets);
        put_cv(&mut out, &rts);
        put_delta(&mut out, &tgts, &offsets);
    }
    put_csr(&mut out, &c.syn_members);
    put_opt(&mut out, &c.syn_lexfile);
    put_opt(&mut out, &c.syn_dc_source);
    put_opt(&mut out, &c.syn_confidence);

    // Sense columns.
    let n_syn = c.syn_pos.len();
    // sense_synset has u32::MAX for a dangling link; remap to n_syn so the
    // column width stays ~log2(n_syn) rather than 32.
    put_cv(
        &mut out,
        &c.sense_synset
            .iter()
            .map(|&s| if s == u32::MAX { n_syn } else { s as usize })
            .collect::<Vec<_>>(),
    );
    {
        let mut offsets = Vec::with_capacity(c.sense_relations.len() + 1);
        let mut rts = Vec::new();
        let mut tgts = Vec::new();
        let mut acc = 0usize;
        offsets.push(0);
        for v in &c.sense_relations {
            acc += v.len();
            offsets.push(acc);
            for r in v {
                rts.push(sense_rt_idx[r.rel_type.as_str()]);
                tgts.push(r.target as usize);
            }
        }
        put_ef(&mut out, &offsets);
        put_cv(&mut out, &rts);
        put_delta(&mut out, &tgts, &offsets);
    }
    put_csr(&mut out, &c.sense_subcat);
    put_opt(&mut out, &c.sense_adjposition);
    put_opt(&mut out, &c.sense_dc_source);
    put_csr(&mut out, &c.sense_counts);

    // Entry columns.
    put_cv(
        &mut out,
        &c.entry_lemma_form
            .iter()
            .map(|&x| x as usize)
            .collect::<Vec<_>>(),
    );
    put_pos(&mut out, &c.entry_lemma_pos);
    put_opt(&mut out, &c.entry_lemma_script);
    put_cv(
        &mut out,
        &c.entry_sense_start
            .iter()
            .map(|&x| x as usize)
            .collect::<Vec<_>>(),
    );
    put_cv(
        &mut out,
        &c.entry_sense_count
            .iter()
            .map(|&x| x as usize)
            .collect::<Vec<_>>(),
    );

    // Small nested tails as one rkyv blob.
    let tail = ComplexTail {
        entry_lemma_prons: c.entry_lemma_prons.clone(),
        entry_forms: c.entry_forms.clone(),
        entry_synbehav: c.entry_synbehav.clone(),
        lex_synbehav: c.lex_synbehav.clone(),
        lexicon: c.lexicon.clone(),
    };
    let tail_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&tail).expect("rkyv tail");
    put_blob(&mut out, &tail_bytes);

    out
}

/// Decode the succinct `.prx` bytes back into an exact [`CompactWordNet`].
pub fn from_succinct(buf: &[u8]) -> CompactWordNet {
    use super::compact::{ISenRel, ISynRel};
    let mut pos = 0usize;

    let _n_syn = get_varint(buf, &mut pos) as usize;
    let _n_sense = get_varint(buf, &mut pos) as usize;
    let _n_entry = get_varint(buf, &mut pos) as usize;

    let dict = get_dict_fc(buf, &mut pos);
    let syn_rt = get_dict(buf, &mut pos);
    let sense_rt = get_dict(buf, &mut pos);

    let syn_pos = get_pos(buf, &mut pos);
    let syn_ili = get_opt(buf, &mut pos);
    let syn_definitions = get_csr(buf, &mut pos);
    let syn_ili_definition = get_opt(buf, &mut pos);
    let syn_examples = get_csr(buf, &mut pos);
    let syn_relations = {
        let offsets = get_ef(buf, &mut pos);
        let rts = get_cv(buf, &mut pos);
        let tgts = get_delta(buf, &mut pos, &offsets);
        let n = offsets.len().saturating_sub(1);
        (0..n)
            .map(|i| {
                (offsets[i]..offsets[i + 1])
                    .map(|j| ISynRel {
                        rel_type: SynsetRelationType::parse(&syn_rt[rts[j]]),
                        target: tgts[j] as u32,
                    })
                    .collect()
            })
            .collect()
    };
    let syn_members = get_csr(buf, &mut pos);
    let syn_lexfile = get_opt(buf, &mut pos);
    let syn_dc_source = get_opt(buf, &mut pos);
    let syn_confidence = get_opt(buf, &mut pos);

    let n_syn = syn_pos.len();
    let sense_synset = get_cv(buf, &mut pos)
        .into_iter()
        .map(|s| if s == n_syn { u32::MAX } else { s as u32 })
        .collect();
    let sense_relations = {
        let offsets = get_ef(buf, &mut pos);
        let rts = get_cv(buf, &mut pos);
        let tgts = get_delta(buf, &mut pos, &offsets);
        let n = offsets.len().saturating_sub(1);
        (0..n)
            .map(|i| {
                (offsets[i]..offsets[i + 1])
                    .map(|j| ISenRel {
                        rel_type: SenseRelationType::parse(&sense_rt[rts[j]]),
                        target: tgts[j] as u32,
                    })
                    .collect()
            })
            .collect()
    };
    let sense_subcat = get_csr(buf, &mut pos);
    let sense_adjposition = get_opt(buf, &mut pos);
    let sense_dc_source = get_opt(buf, &mut pos);
    let sense_counts = get_csr(buf, &mut pos);

    let entry_lemma_form = get_cv(buf, &mut pos)
        .into_iter()
        .map(|x| x as u32)
        .collect();
    let entry_lemma_pos = get_pos(buf, &mut pos);
    let entry_lemma_script = get_opt(buf, &mut pos);
    let entry_sense_start = get_cv(buf, &mut pos)
        .into_iter()
        .map(|x| x as u32)
        .collect();
    let entry_sense_count = get_cv(buf, &mut pos)
        .into_iter()
        .map(|x| x as u32)
        .collect();

    // rkyv zero-copy access needs an aligned buffer; the blob is a slice at an
    // arbitrary offset, so re-align it into an AlignedVec before decoding.
    let tail_bytes = get_blob(buf, &mut pos);
    let mut aligned = rkyv::util::AlignedVec::<16>::new();
    aligned.extend_from_slice(tail_bytes);
    let tail: ComplexTail =
        rkyv::from_bytes::<ComplexTail, rkyv::rancor::Error>(aligned.as_slice())
            .expect("rkyv tail decode");

    CompactWordNet {
        dict,
        lexicon: tail.lexicon,
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
        entry_lemma_prons: tail.entry_lemma_prons,
        entry_sense_start,
        entry_sense_count,
        entry_forms: tail.entry_forms,
        entry_synbehav: tail.entry_synbehav,
        lex_synbehav: tail.lex_synbehav,
    }
}

// ── `.prx.gz` emit / load — the shipped artifact and its runtime decode ─────

use crate::social::software::markup::xml::succinct::{gunzip, gzip};

/// Serialize a parsed [`WordNet`] to the `.prx.gz` bytes — the artifact that is
/// embedded (`include_bytes!`) or downloaded.
pub fn emit_prx_gz(wn: &WordNet) -> Vec<u8> {
    gzip(&to_succinct(&encode(wn)))
}

/// Load `.prx.gz` bytes into a materialized [`English`]: gunzip → succinct
/// decode → [`decode`] → `English::from_wordnet`.
pub fn load_prx_gz(prx_gz: &[u8]) -> English {
    English::from_wordnet(&decode(&from_succinct(&gunzip(prx_gz))))
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

    /// The codec is LOSSLESS over the compact core
    /// (`from_succinct(to_succinct(c)) == c`) and the `.prx` is smaller than the
    /// raw source. Reads the tiny lexicon (instant) + 89 MB english (one parse);
    /// graceful skip if absent.
    #[test]
    fn succinct_codec_roundtrip_and_smaller() {
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

            let succ = to_succinct(&compact);
            let back = from_succinct(&succ);
            assert_eq!(back, compact, "{name}: succinct codec is not lossless");

            let succ_gz = gz_len(&succ);
            let source_raw = bytes.len(); // the .xml on disk
            let source_dl = gz_len(&bytes); // the .xml.gz you download
            eprintln!(
                "SUCCINCT {name}: .prx = {:.2}MB ({:.2}MB gz)   vs   SOURCE = {:.2}MB xml \
                 ({:.2}MB .xml.gz download)   ->   .prx is {:.2}x the raw source, {:.2}x the \
                 download",
                succ.len() as f64 / 1e6,
                succ_gz as f64 / 1e6,
                source_raw as f64 / 1e6,
                source_dl as f64 / 1e6,
                succ.len() as f64 / source_raw.max(1) as f64,
                succ_gz as f64 / source_dl.max(1) as f64,
            );
            // The succinct .prx must beat the raw source on disk (and, at corpus
            // scale, the download too).
            assert!(
                succ.len() < source_raw,
                "{name}: .prx ({}) not smaller than the raw source ({})",
                succ.len(),
                source_raw
            );
            measured += 1;
        }
        assert!(
            measured >= 1,
            "no WN-LMF source on disk for the succinct codec"
        );
    }

    /// End-to-end: `emit_prx_gz` → `load_prx_gz` materializes an `English` equal
    /// to `from_wordnet` over the source (same concept count and word→concept
    /// index) — the full embed/download → gunzip → decode → reason pipeline.
    #[test]
    fn prx_gz_round_trips_to_english() {
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

            let prx_gz = emit_prx_gz(&wn);
            let t = std::time::Instant::now();
            let loaded = load_prx_gz(&prx_gz);
            let load_ms = t.elapsed().as_secs_f64() * 1e3;
            let reference = English::from_wordnet(&wn);

            assert_eq!(
                loaded.concept_count(),
                reference.concept_count(),
                "{name}: loaded English concept_count differs from from_wordnet"
            );
            assert_eq!(
                loaded.word_index, reference.word_index,
                "{name}: loaded English word→concept index differs"
            );

            let source_download = gz_len(&bytes);
            eprintln!(
                "PRX-GZ {name}: .prx.gz = {:.2}MB  loads to {} concepts in {:.0}ms (native)  vs \
                 source download {:.2}MB",
                prx_gz.len() as f64 / 1e6,
                loaded.concept_count(),
                load_ms,
                source_download as f64 / 1e6,
            );
            // THE COMPACTNESS GATE: the shipped `.prx.gz` must be smaller than
            // fetching the source itself (`gzip(source)`). This is the guard the
            // .prx-bigger-than-source regression lacked; it fails closed if any
            // source ever re-bloats past its own download.
            assert!(
                prx_gz.len() < source_download,
                "{name}: .prx.gz ({} B) is NOT smaller than the source download gzip(source) \
                 ({} B) — compactness regressed",
                prx_gz.len(),
                source_download,
            );
            measured += 1;
        }
        assert!(
            measured >= 1,
            "no WN-LMF source on disk for the prx.gz round-trip"
        );
    }
}

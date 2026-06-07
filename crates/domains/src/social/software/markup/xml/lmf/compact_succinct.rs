//! Succinct codec for [`CompactWordNet`](super::compact::CompactWordNet) — the
//! size-reduced `.prx` bytes.
//!
//! Stage S1: the 31 MB columnar STRUCTURE is re-encoded as bit-packed columns
//! (`sucds::CompactVector`, each at `ceil(log2(max))` bits instead of rkyv's
//! flat 32) with CSR (offset + flat-value) layout for the `Vec<Vec<_>>` columns,
//! eliminating rkyv's per-inner-`Vec` length-prefix / relative-pointer overhead.
//! Relation types are mapped through a tiny per-codec string dictionary and
//! reconstructed by `parse(as_str(_))` (a proven inverse, incl. the `Other(_)`
//! tail). The lexical dictionary is a length-prefixed UTF-8 blob (front-coding /
//! FSST is Stage S2); the small nested tails (pronunciations, forms, syntactic
//! behaviours, lexicon metadata) ride along as one rkyv blob.
//!
//! [`from_succinct`] reconstructs the `CompactWordNet` exactly
//! (`from_succinct(&to_succinct(c)) == c`), so it composes with `compact::decode`
//! → `from_wordnet` unchanged. LOUDS for the hypernym tree + Elias-Fano adjacency
//! (toward the ~3 MB floor) layer on top of this same `(dict, columns)` split.

use alloc::{string::String, vec::Vec};

use hashbrown::HashMap;
use sucds::Serializable;
use sucds::int_vectors::CompactVector;
use sucds::mii_sequences::{EliasFano, EliasFanoBuilder};

use super::compact::{CompactWordNet, IForm, ILexiconMetadata, IPron, ISynBehav};
use super::ontology::{LmfPos, SenseRelationType, SynsetRelationType};

// ── primitive writers / readers ───────────────────────────────────────────

fn put_varint(out: &mut Vec<u8>, mut n: u64) {
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

fn get_varint(buf: &[u8], pos: &mut usize) -> u64 {
    let mut n = 0u64;
    let mut shift = 0;
    loop {
        let b = buf[*pos];
        *pos += 1;
        n |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    n
}

fn put_blob(out: &mut Vec<u8>, bytes: &[u8]) {
    put_varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

fn get_blob<'a>(buf: &'a [u8], pos: &mut usize) -> &'a [u8] {
    let len = get_varint(buf, pos) as usize;
    let b = &buf[*pos..*pos + len];
    *pos += len;
    b
}

/// A bit-packed column (`sucds::CompactVector`, width = `ceil(log2(max+1))`),
/// written as a length-prefixed blob. Empty → an empty blob.
fn put_cv(out: &mut Vec<u8>, vals: &[usize]) {
    let mut tmp = Vec::new();
    if !vals.is_empty() {
        // CompactVector needs width ≥ 1; an all-zero column would yield width 0.
        let cv = if vals.iter().all(|&v| v == 0) {
            let mut c = CompactVector::with_capacity(vals.len(), 1).expect("cv cap");
            c.extend(vals.iter().copied()).expect("cv extend");
            c
        } else {
            CompactVector::from_slice(vals).expect("cv from_slice")
        };
        cv.serialize_into(&mut tmp).expect("cv serialize");
    }
    put_blob(out, &tmp);
}

fn get_cv(buf: &[u8], pos: &mut usize) -> Vec<usize> {
    let b = get_blob(buf, pos);
    if b.is_empty() {
        return Vec::new();
    }
    let cv = CompactVector::deserialize_from(b).expect("cv deserialize");
    (0..cv.len())
        .map(|i| cv.get_int(i).expect("cv get"))
        .collect()
}

/// A MONOTONE non-decreasing sequence (CSR offsets) via Elias-Fano — ~`n·(2 +
/// log2(universe/n))` bits, far below `CompactVector`'s `n·log2(universe)` for
/// the offset arrays. Length-prefixed so the empty / all-zero cases round-trip.
fn put_ef(out: &mut Vec<u8>, vals: &[usize]) {
    put_varint(out, vals.len() as u64);
    let mut tmp = Vec::new();
    if !vals.is_empty() {
        let universe = vals[vals.len() - 1] + 1;
        let mut bld = EliasFanoBuilder::new(universe, vals.len()).expect("ef builder");
        for &v in vals {
            bld.push(v).expect("ef push");
        }
        bld.build().serialize_into(&mut tmp).expect("ef serialize");
    }
    put_blob(out, &tmp);
}

fn get_ef(buf: &[u8], pos: &mut usize) -> Vec<usize> {
    let n = get_varint(buf, pos) as usize;
    let b = get_blob(buf, pos);
    if n == 0 {
        return Vec::new();
    }
    let ef = EliasFano::deserialize_from(b).expect("ef deserialize");
    (0..n).map(|i| ef.select(i).expect("ef select")).collect()
}

/// Delta-code a flat value array within each CSR node range `[offsets[i],
/// offsets[i+1])`: store ascending gaps (the relations are sorted by target, so
/// the gaps are small and the bit-packed column is far below the absolute one).
fn put_delta(out: &mut Vec<u8>, values: &[usize], offsets: &[usize]) {
    let mut d = Vec::with_capacity(values.len());
    for w in offsets.windows(2) {
        let mut prev = 0usize;
        for &v in &values[w[0]..w[1]] {
            d.push(v - prev);
            prev = v;
        }
    }
    put_cv(out, &d);
}

fn get_delta(buf: &[u8], pos: &mut usize, offsets: &[usize]) -> Vec<usize> {
    let d = get_cv(buf, pos);
    let mut values = alloc::vec![0usize; d.len()];
    for w in offsets.windows(2) {
        let mut prev = 0usize;
        for (slot, &delta) in values[w[0]..w[1]].iter_mut().zip(&d[w[0]..w[1]]) {
            prev += delta;
            *slot = prev;
        }
    }
    values
}

// ── CSR (Vec<Vec<u32>>) and option columns ────────────────────────────────

fn put_csr(out: &mut Vec<u8>, vecs: &[Vec<u32>]) {
    let mut offsets = Vec::with_capacity(vecs.len() + 1);
    let mut values = Vec::new();
    let mut acc = 0usize;
    offsets.push(0);
    for v in vecs {
        acc += v.len();
        offsets.push(acc);
        values.extend(v.iter().map(|&x| x as usize));
    }
    put_ef(out, &offsets);
    put_cv(out, &values);
}

fn get_csr(buf: &[u8], pos: &mut usize) -> Vec<Vec<u32>> {
    let offsets = get_ef(buf, pos);
    let values = get_cv(buf, pos);
    let n = offsets.len().saturating_sub(1);
    (0..n)
        .map(|i| {
            values[offsets[i]..offsets[i + 1]]
                .iter()
                .map(|&x| x as u32)
                .collect()
        })
        .collect()
}

/// `Vec<Option<u32>>` → one bit-packed column with `None→0`, `Some(v)→v+1`.
fn put_opt(out: &mut Vec<u8>, opts: &[Option<u32>]) {
    let vals: Vec<usize> = opts
        .iter()
        .map(|o| o.map_or(0, |v| v as usize + 1))
        .collect();
    put_cv(out, &vals);
}

fn get_opt(buf: &[u8], pos: &mut usize) -> Vec<Option<u32>> {
    get_cv(buf, pos)
        .into_iter()
        .map(|v| if v == 0 { None } else { Some((v - 1) as u32) })
        .collect()
}

fn put_dict(out: &mut Vec<u8>, dict: &[String]) {
    put_varint(out, dict.len() as u64);
    for s in dict {
        put_blob(out, s.as_bytes());
    }
}

fn get_dict(buf: &[u8], pos: &mut usize) -> Vec<String> {
    let n = get_varint(buf, pos) as usize;
    (0..n)
        .map(|_| String::from_utf8(get_blob(buf, pos).to_vec()).expect("dict utf8"))
        .collect()
}

/// Front-coded dictionary (HDT's technique): for a SORTED list, store each
/// entry as `varint(shared_prefix_len) varint(suffix_len) suffix`, so a shared
/// prefix with the previous entry is never repeated. The `compact::encode`
/// pass sorts the dict so adjacent entries share prefixes. Decodes back to the
/// exact `Vec<String>` (a sequential pass at load — not random-access; that is
/// the FSST/query-in-place tradeoff, deliberately not taken here).
fn put_dict_fc(out: &mut Vec<u8>, dict: &[String]) {
    put_varint(out, dict.len() as u64);
    let mut fc = Vec::new();
    let mut prev: &[u8] = b"";
    for s in dict {
        let b = s.as_bytes();
        let shared = prev.iter().zip(b).take_while(|(x, y)| x == y).count();
        put_varint(&mut fc, shared as u64);
        put_varint(&mut fc, (b.len() - shared) as u64);
        fc.extend_from_slice(&b[shared..]);
        prev = b;
    }
    put_blob(out, &fc);
}

fn get_dict_fc(buf: &[u8], pos: &mut usize) -> Vec<String> {
    let n = get_varint(buf, pos) as usize;
    let fc = get_blob(buf, pos);
    let mut fp = 0usize;
    let mut out = Vec::with_capacity(n);
    let mut prev: Vec<u8> = Vec::new();
    for _ in 0..n {
        let shared = get_varint(fc, &mut fp) as usize;
        let suffix_len = get_varint(fc, &mut fp) as usize;
        let mut s = prev[..shared].to_vec();
        s.extend_from_slice(&fc[fp..fp + suffix_len]);
        fp += suffix_len;
        out.push(String::from_utf8(s.clone()).expect("dict utf8"));
        prev = s;
    }
    out
}

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

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use super::*;
    use crate::social::software::markup::xml::lmf::compact::encode;
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;

    fn gz_len(bytes: &[u8]) -> usize {
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        e.write_all(bytes).expect("gz write");
        e.finish().expect("gz finish").len()
    }

    /// Stage S1 gate: the succinct codec is LOSSLESS over the compact core
    /// (`from_succinct(to_succinct(c)) == c`) and shrinks the encoding. Reads the
    /// tiny lexicon (instant) + 89 MB english (one parse); graceful skip.
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
}

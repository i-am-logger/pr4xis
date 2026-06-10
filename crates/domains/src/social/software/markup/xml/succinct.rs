//! Source-agnostic succinct bit-packing primitives — the shared toolkit the
//! compact `.prx` codecs build on. Each columnar value uses `bits(max)` bits,
//! LSB-first, in CSR (offset + flat-value) layout; monotone offset sequences are
//! gap-coded and per-range value runs delta-coded; string dictionaries are
//! front-coded. The bit-packing is pure `u64`-accumulator arithmetic with no
//! dependency, so the `.prx` decodes on any target including wasm32.
//!
//! The WN-LMF codec ([`lmf::compact_succinct`](super::lmf::compact_succinct)) and
//! the codegen-interchange codec ([`owl::prx::OwnedCodegenData`](super::owl::prx::OwnedCodegenData))
//! both serialize through these — one bit-packing kernel, one set of round-trip
//! invariants, regardless of the source the data came from.

use alloc::{string::String, vec::Vec};

// ── varints + length-prefixed blobs ────────────────────────────────────────

/// LEB128 unsigned varint.
pub(crate) fn put_varint(out: &mut Vec<u8>, mut n: u64) {
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

pub(crate) fn get_varint(buf: &[u8], pos: &mut usize) -> u64 {
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

pub(crate) fn put_blob(out: &mut Vec<u8>, bytes: &[u8]) {
    put_varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

pub(crate) fn get_blob<'a>(buf: &'a [u8], pos: &mut usize) -> &'a [u8] {
    let len = get_varint(buf, pos) as usize;
    let b = &buf[*pos..*pos + len];
    *pos += len;
    b
}

// ── bit-packed columns ─────────────────────────────────────────────────────

/// A bit-packed column: each value uses `width = bits(max)` bits, LSB-first into
/// a byte stream. Format: `varint(len)`, then if non-empty `u8(width)` + packed
/// bits (`width == 0` for an all-zero column → no payload). Pure `u64`-
/// accumulator arithmetic — wasm32-safe.
pub(crate) fn put_cv(out: &mut Vec<u8>, vals: &[usize]) {
    put_varint(out, vals.len() as u64);
    if vals.is_empty() {
        return;
    }
    let max = vals.iter().copied().max().unwrap_or(0) as u64;
    let width = (u64::BITS - max.leading_zeros()) as usize; // 0 iff max == 0
    out.push(width as u8);
    if width == 0 {
        return;
    }
    let mut acc: u64 = 0;
    let mut bits = 0usize;
    for &v in vals {
        acc |= (v as u64) << bits;
        bits += width;
        while bits >= 8 {
            out.push((acc & 0xff) as u8);
            acc >>= 8;
            bits -= 8;
        }
    }
    if bits > 0 {
        out.push((acc & 0xff) as u8);
    }
}

pub(crate) fn get_cv(buf: &[u8], pos: &mut usize) -> Vec<usize> {
    let n = get_varint(buf, pos) as usize;
    if n == 0 {
        return Vec::new();
    }
    let width = buf[*pos] as usize;
    *pos += 1;
    if width == 0 {
        return alloc::vec![0usize; n];
    }
    let mask = if width >= 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    };
    let mut out = Vec::with_capacity(n);
    let mut acc: u64 = 0;
    let mut bits = 0usize;
    for _ in 0..n {
        while bits < width {
            acc |= (buf[*pos] as u64) << bits;
            *pos += 1;
            bits += 8;
        }
        out.push((acc & mask) as usize);
        acc >>= width;
        bits -= width;
    }
    out
}

/// A MONOTONE non-decreasing sequence (CSR offsets) stored as its consecutive
/// GAPS, bit-packed via [`put_cv`]. The gaps are the per-node lengths — small —
/// so the packed width is tiny, the same compression Elias-Fano gives on offsets
/// but wasm32-safe and dependency-free. Prefix-summed back on read.
pub(crate) fn put_ef(out: &mut Vec<u8>, vals: &[usize]) {
    let mut gaps = Vec::with_capacity(vals.len());
    let mut prev = 0usize;
    for &v in vals {
        gaps.push(v - prev);
        prev = v;
    }
    put_cv(out, &gaps);
}

pub(crate) fn get_ef(buf: &[u8], pos: &mut usize) -> Vec<usize> {
    let gaps = get_cv(buf, pos);
    let mut out = Vec::with_capacity(gaps.len());
    let mut acc = 0usize;
    for g in gaps {
        acc += g;
        out.push(acc);
    }
    out
}

/// Delta-code a flat value array within each CSR node range `[offsets[i],
/// offsets[i+1])`: store ascending gaps (the values are sorted per node, so the
/// gaps are small and the bit-packed column is far below the absolute one).
pub(crate) fn put_delta(out: &mut Vec<u8>, values: &[usize], offsets: &[usize]) {
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

pub(crate) fn get_delta(buf: &[u8], pos: &mut usize, offsets: &[usize]) -> Vec<usize> {
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

pub(crate) fn put_csr(out: &mut Vec<u8>, vecs: &[Vec<u32>]) {
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

pub(crate) fn get_csr(buf: &[u8], pos: &mut usize) -> Vec<Vec<u32>> {
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
pub(crate) fn put_opt(out: &mut Vec<u8>, opts: &[Option<u32>]) {
    let vals: Vec<usize> = opts
        .iter()
        .map(|o| o.map_or(0, |v| v as usize + 1))
        .collect();
    put_cv(out, &vals);
}

pub(crate) fn get_opt(buf: &[u8], pos: &mut usize) -> Vec<Option<u32>> {
    get_cv(buf, pos)
        .into_iter()
        .map(|v| if v == 0 { None } else { Some((v - 1) as u32) })
        .collect()
}

// ── string dictionaries ────────────────────────────────────────────────────

pub(crate) fn put_dict(out: &mut Vec<u8>, dict: &[String]) {
    put_varint(out, dict.len() as u64);
    for s in dict {
        put_blob(out, s.as_bytes());
    }
}

pub(crate) fn get_dict(buf: &[u8], pos: &mut usize) -> Vec<String> {
    let n = get_varint(buf, pos) as usize;
    (0..n)
        .map(|_| String::from_utf8(get_blob(buf, pos).to_vec()).expect("dict utf8"))
        .collect()
}

/// Front-coded dictionary: for a sorted dict, store each entry as
/// `varint(shared_prefix_len) varint(suffix_len) suffix`, so a prefix shared
/// with the previous entry is not repeated. Lossless for any input order (the
/// shared count is computed against the previous entry); sorting the dict before
/// calling maximizes the elided prefix. Decodes to the exact `Vec<String>` in
/// one sequential pass.
pub(crate) fn put_dict_fc(out: &mut Vec<u8>, dict: &[String]) {
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

pub(crate) fn get_dict_fc(buf: &[u8], pos: &mut usize) -> Vec<String> {
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

// ── gzip (RFC 1952) wrapper ────────────────────────────────────────────────

#[cfg(feature = "std")]
pub(crate) fn gzip(data: &[u8]) -> Vec<u8> {
    use std::io::Write as _;
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(data).expect("gz write");
    e.finish().expect("gz finish")
}

#[cfg(feature = "std")]
pub(crate) fn gunzip(data: &[u8]) -> Vec<u8> {
    use std::io::Read as _;
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(data)
        .read_to_end(&mut out)
        .expect("gunzip");
    out
}

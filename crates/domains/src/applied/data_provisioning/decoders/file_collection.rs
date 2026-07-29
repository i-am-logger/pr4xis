//! Generic directory-archive codec — the deterministic `path → bytes`
//! collection format shared by every MANY-file raw-source content type
//! (the Base16/Base24 color-scheme collection, the VerbNet class-XML
//! collection).
//!
//! Every other raw-source content type is a single file (an XSD, a DTD, a
//! TSV, a glyph list). A COLLECTION content type is a directory tree of many
//! named files. The generalized raw-source `.prx` envelope
//! ([`raw_source_prx`](super::super::raw_source_prx)) carries a single byte
//! blob — so this module is the *archive* layer that flattens a whole
//! directory into ONE deterministic blob (the source's canonical raw bytes),
//! and the *decode* layer that recovers the `path → bytes` set a consumer
//! scans. The blob then rides the SAME content-addressed `.prx` envelope
//! every other raw source uses; this codec is its payload, not a second
//! envelope.
//!
//! ## Why an archive, and why deterministic
//!
//! To load a whole directory through the generalized gated `.prx` path
//! (rather than `std::fs` of a fetched checkout), the directory must become
//! one content-addressable artifact. The archive is therefore DETERMINISTIC:
//! entries are emitted in sorted-path order, so the same file tree always
//! yields byte-identical archive bytes and hence the same content address
//! (Dolstra 2006). A non-deterministic archive would defeat the pin.
//!
//! ## Wire format (dependency-free, portable)
//!
//! `put_u64(entry_count)` then, per entry in sorted-path order,
//! `put_blob(relative_path) put_blob(content)`. The framing is the SAME
//! LEB128 length-prefixing the raw-source envelope uses (no rkyv, no gzip),
//! so the layout — and the content address taken over it — is stable across
//! toolchains and targets (wasm32 included). The decoder is fully
//! bounds-checked: a truncated archive is an `Err`, never a panic.
//!
//! Extracted from `theme_collection` (the first many-file collection this
//! codebase loaded) once a second, unrelated collection (VerbNet's class
//! hierarchy) needed the identical archive/decode shape — the LEB128 framing
//! and sorted-path determinism logic carry no theme-specific behavior at all;
//! only the directory-walk driver that PRODUCES a [`FileCollection`] from a
//! fetched checkout is source-specific, and stays in each collection's own
//! decoder module.
//!
//! ## Citations
//!
//! - **Dolstra, E. (2006)** *The Purely Functional Software Deployment Model*
//!   — content-addressing requires a deterministic (reproducible) artifact;
//!   the sorted-path archive realises it.

#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// One archived file: its collection-relative path (forward-slash separated,
/// e.g. `base16/apathy/default.yaml` or `verbnet3.3/stop-55.4.xml`) and its
/// raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionFile {
    /// The path relative to the collection root, `/`-separated.
    pub path: String,
    /// The file's raw bytes, byte-for-byte.
    pub content: Vec<u8>,
}

/// A decoded file collection — the in-order (sorted-path) set of archived
/// files. The structure a collection's consumer iterates, the directory-
/// archive analogue of `plaintext_tsv::TsvRecords`.
pub type FileCollection = Vec<CollectionFile>;

/// A failure decoding a [`FileCollection`] archive — fail-closed, naming the
/// cause; a truncated / malformed archive is an `Err`, never a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileCollectionError {
    /// A varint or blob span runs past the end of the buffer.
    Malformed(String),
    /// An archived path is not UTF-8 (paths are text by construction).
    PathNotUtf8(String),
    /// The entries are not in strictly-sorted-path order — the archive is not
    /// the canonical (deterministic) form, so its content address is not the
    /// one the pin was taken over. Rejected so a re-ordered archive can never
    /// silently pass the gate.
    NotCanonical { prev: String, next: String },
}

impl core::fmt::Display for FileCollectionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FileCollectionError::Malformed(m) => {
                write!(f, "file-collection archive malformed: {m}")
            }
            FileCollectionError::PathNotUtf8(m) => {
                write!(f, "file-collection archive path not UTF-8: {m}")
            }
            FileCollectionError::NotCanonical { prev, next } => write!(
                f,
                "file-collection archive is not in canonical sorted-path order \
                 (`{prev}` precedes `{next}`) — re-emit it deterministically"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FileCollectionError {}

// ---------------------------------------------------------------------------
// LEB128 framing (the SAME dependency-free codec the raw-source envelope uses)
// ---------------------------------------------------------------------------

/// Append a u64 as an LEB128 varint.
pub(crate) fn put_u64(out: &mut Vec<u8>, mut n: u64) {
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

/// Read one LEB128 varint with full bounds + overflow checking.
fn get_u64(buf: &[u8], pos: &mut usize) -> Result<u64, FileCollectionError> {
    let mut len: u64 = 0;
    let mut shift = 0u32;
    loop {
        let b = *buf.get(*pos).ok_or_else(|| {
            FileCollectionError::Malformed("varint runs past end of buffer".into())
        })?;
        *pos += 1;
        len |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(FileCollectionError::Malformed(
                "varint length overflow".into(),
            ));
        }
    }
    Ok(len)
}

/// Append `bytes` length-prefixed (LEB128 length + raw bytes).
pub(crate) fn put_blob(out: &mut Vec<u8>, bytes: &[u8]) {
    put_u64(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

/// Read one length-prefixed blob with full bounds checking.
fn get_blob<'a>(buf: &'a [u8], pos: &mut usize) -> Result<&'a [u8], FileCollectionError> {
    let len = get_u64(buf, pos)? as usize;
    let end = pos
        .checked_add(len)
        .filter(|&e| e <= buf.len())
        .ok_or_else(|| FileCollectionError::Malformed("blob runs past end of buffer".into()))?;
    let b = &buf[*pos..end];
    *pos = end;
    Ok(b)
}

// ---------------------------------------------------------------------------
// The deterministic directory-archive codec
// ---------------------------------------------------------------------------

/// Encode a file collection into the portable, DETERMINISTIC archive blob:
/// `put_u64(count)` then `put_blob(path) put_blob(content)` per entry, in
/// sorted-path order. Entries are sorted here (the caller need not pre-sort),
/// so the SAME directory always produces byte-identical bytes — the
/// reproducibility content-addressing requires (Dolstra 2006). Duplicate paths
/// are kept (the archive is a faithful image of the tree); the deterministic
/// order is the only normalization.
#[must_use]
pub fn encode_collection(files: &[CollectionFile]) -> Vec<u8> {
    let mut sorted: Vec<&CollectionFile> = files.iter().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    let mut out = Vec::new();
    put_u64(&mut out, sorted.len() as u64);
    for f in sorted {
        put_blob(&mut out, f.path.as_bytes());
        put_blob(&mut out, &f.content);
    }
    out
}

/// Decode a file-collection archive blob back into the in-order files — the
/// exact inverse of [`encode_collection`] for canonical (sorted) input.
/// Fail-closed: a truncated/malformed blob, a non-UTF-8 path, or entries that
/// are NOT in strictly-sorted-path order are each an `Err` (the last forbids
/// a re-ordered, non-canonical archive from passing as if it were the pinned
/// one).
pub fn decode(bytes: &[u8]) -> Result<FileCollection, FileCollectionError> {
    let mut pos = 0usize;
    let count = get_u64(bytes, &mut pos)? as usize;
    // Bound the pre-allocation by the bytes that remain: every entry costs at
    // least two length-prefix varints (≥ 2 bytes), so a `count` larger than the
    // buffer is already malformed and `get_blob` below refuses it cleanly. This
    // keeps the optimisation for honest archives while refusing — not aborting
    // on a multi-petabyte allocation — for an adversarial count prefix.
    let mut out: FileCollection = Vec::with_capacity(count.min(bytes.len()));
    let mut prev: Option<String> = None;
    for _ in 0..count {
        let path_bytes = get_blob(bytes, &mut pos)?;
        let path = core::str::from_utf8(path_bytes)
            .map_err(|e| FileCollectionError::PathNotUtf8(e.to_string()))?
            .to_string();
        if let Some(p) = &prev
            && p.as_str() > path.as_str()
        {
            return Err(FileCollectionError::NotCanonical {
                prev: p.clone(),
                next: path,
            });
        }
        let content = get_blob(bytes, &mut pos)?.to_vec();
        prev = Some(path.clone());
        out.push(CollectionFile { path, content });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn cf(path: &str, content: &[u8]) -> CollectionFile {
        CollectionFile {
            path: path.to_string(),
            content: content.to_vec(),
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn encode_decode_round_trips_exact() {
        let files = vec![
            cf(
                "base16/apathy/default.yaml",
                b"system: base16\nbase00: \"#000\"\n",
            ),
            cf("verbnet3.3/stop-55.4.xml", b"<VNCLASS ID=\"stop-55.4\"/>"),
            cf("base16/zzz/default.yaml", b""),
        ];
        let blob = encode_collection(&files);
        let back = decode(&blob).expect("decode");
        let mut expect = files.clone();
        expect.sort_by(|a, b| a.path.cmp(&b.path));
        assert_eq!(back, expect);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn encode_is_deterministic_regardless_of_input_order() {
        let a = vec![
            cf("b/2.xml", b"two"),
            cf("a/1.xml", b"one"),
            cf("a/0.xml", b"zero"),
        ];
        let mut b = a.clone();
        b.reverse();
        assert_eq!(encode_collection(&a), encode_collection(&b));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn empty_collection_round_trips() {
        let blob = encode_collection(&[]);
        assert_eq!(
            decode(&blob).expect("decode empty"),
            Vec::<CollectionFile>::new()
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn decode_rejects_truncated_archive_without_panic() {
        let files = vec![cf("verbnet3.3/x.xml", b"some xml bytes here")];
        let mut blob = encode_collection(&files);
        blob.truncate(blob.len() - 3);
        let err = decode(&blob).expect_err("truncated must be Err");
        assert!(
            matches!(err, FileCollectionError::Malformed(_)),
            "got {err:?}"
        );
    }

    #[pr4xis::praxis_value(Honest, Verifiable, Deterministic)]
    #[test]
    fn decode_rejects_non_canonical_order_fail_closed() {
        // Hand-build an archive with entries out of sorted order; the decoder
        // must reject it (a re-ordered archive is not the canonical/pinned one).
        let mut blob = Vec::new();
        put_u64(&mut blob, 2);
        put_blob(&mut blob, b"z.xml");
        put_blob(&mut blob, b"z");
        put_blob(&mut blob, b"a.xml");
        put_blob(&mut blob, b"a");
        let err = decode(&blob).expect_err("out-of-order must be Err");
        assert!(
            matches!(err, FileCollectionError::NotCanonical { .. }),
            "got {err:?}"
        );
    }

    proptest! {
        /// FORALL round-trip: any generated set of files (arbitrary `/`-pathed
        /// names, arbitrary byte contents) archives → decodes back to the SAME
        /// set in sorted-path order — the GetPut leg of the collection ⇄
        /// archive lens, exercised across sizes, paths, and contents.
        #[test]
        fn prop_collection_round_trips(
            files in proptest::collection::vec(
                ("[a-z0-9/_-]{1,24}", proptest::collection::vec(any::<u8>(), 0..64)),
                0..30,
            )
        ) {
            let files: Vec<CollectionFile> = files
                .into_iter()
                .map(|(path, content)| CollectionFile { path, content })
                .collect();
            let blob = encode_collection(&files);
            let back = decode(&blob)
                .map_err(|e| TestCaseError::fail(format!("decode: {e}")))?;
            let mut expect = files.clone();
            expect.sort_by(|a, b| a.path.cmp(&b.path));
            prop_assert_eq!(&back, &expect);
            // Determinism: re-encoding the round-tripped set is byte-identical.
            prop_assert_eq!(encode_collection(&back), blob);
        }
    }

    pr4xis::register_praxis_value!(prop_collection_round_trips, Deterministic);
}

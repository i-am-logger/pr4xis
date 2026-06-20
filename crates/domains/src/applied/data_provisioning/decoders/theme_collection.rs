//! Theme-collection decoder — the directory-archive codec for the
//! `ContentType::ThemeCollection` source (the Base16/Base24 named
//! color-scheme corpus, the Tinted Theming `tinted-schemes` dataset).
//!
//! Every other raw-source content type is a single file (an XSD, a DTD, a
//! TSV, a glyph list). This one is a MANY-file COLLECTION: a directory tree
//! of named-scheme YAML files (`base16/<scheme>/<variant>.yaml`,
//! `base24/<scheme>/<variant>.yaml`). The generalized raw-source `.prx`
//! envelope ([`raw_source_prx`]) carries a single byte blob — so this module
//! is the *archive* layer that flattens the whole directory of schemes into
//! ONE deterministic blob (the source's canonical raw bytes), and the
//! *decode* layer that recovers the `path → bytes` set the theming validator
//! scans. The blob then rides the SAME content-addressed `.prx` envelope every
//! other raw source uses; this codec is its payload, not a second envelope.
//!
//! ## Why an archive, and why deterministic
//!
//! The theming validator
//! ([`crate::applied::hmi::report::validator`]) walks the scheme files,
//! parses each YAML palette, and certifies it against the praxis luminance-
//! monotonicity + WCAG-AA-contrast axioms. To load that corpus through the
//! generalized gated `.prx` path (rather than `std::fs` of a git submodule),
//! the whole directory must become one content-addressable artifact. The
//! archive is therefore DETERMINISTIC: entries are emitted in sorted-path
//! order, so the same theme tree always yields byte-identical archive bytes
//! and hence the same content address (Dolstra 2006). A non-deterministic
//! archive would defeat the pin.
//!
//! ## Wire format (dependency-free, portable)
//!
//! `put_u64(entry_count)` then, per entry in sorted-path order,
//! `put_blob(relative_path) put_blob(content)`. The framing is the SAME
//! LEB128 length-prefixing the raw-source envelope uses (no rkyv, no gzip), so
//! the layout — and the content address taken over it — is stable across
//! toolchains and targets (wasm32 included). The decoder is fully
//! bounds-checked: a truncated archive is an `Err`, never a panic.
//!
//! ## Citations
//!
//! - **Tinted Theming** — *Base16 Styling Guidelines* + *Base24
//!   specification*, <https://github.com/tinted-theming/home>: the named
//!   color-scheme corpus this collection archives.
//! - **Dolstra, E. (2006)** *The Purely Functional Software Deployment Model*
//!   — content-addressing requires a deterministic (reproducible) artifact;
//!   the sorted-path archive realises it.
//!
//! [`raw_source_prx`]: crate::applied::data_provisioning::raw_source_prx

#[allow(unused_imports)]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// The [`ContentType`](crate::applied::data_provisioning::ontology::ContentType)
/// this module realizes — the single declaration of which content type this
/// file decodes, read by `super::has_decoder_for` (audit 2026-06-12 D-22).
pub const DECODES: crate::applied::data_provisioning::ontology::ContentType =
    crate::applied::data_provisioning::ontology::ContentType::ThemeCollection;

/// One archived theme file: its collection-relative path (forward-slash
/// separated, e.g. `base16/apathy/default.yaml`) and its raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeFile {
    /// The path relative to the collection root, `/`-separated.
    pub path: String,
    /// The file's raw bytes (the YAML scheme text, byte-for-byte).
    pub content: Vec<u8>,
}

/// The decoded theme collection — the in-order (sorted-path) set of archived
/// theme files. The structure the theming validator iterates, the directory-
/// archive analogue of `plaintext_tsv::TsvRecords`.
pub type ThemeCollection = Vec<ThemeFile>;

/// A failure decoding a `ThemeCollection` archive — fail-closed, naming the
/// cause; a truncated / malformed archive is an `Err`, never a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeCollectionError {
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

impl core::fmt::Display for ThemeCollectionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ThemeCollectionError::Malformed(m) => {
                write!(f, "theme-collection archive malformed: {m}")
            }
            ThemeCollectionError::PathNotUtf8(m) => {
                write!(f, "theme-collection archive path not UTF-8: {m}")
            }
            ThemeCollectionError::NotCanonical { prev, next } => write!(
                f,
                "theme-collection archive is not in canonical sorted-path order \
                 (`{prev}` precedes `{next}`) — re-emit it deterministically"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ThemeCollectionError {}

// ---------------------------------------------------------------------------
// LEB128 framing (the SAME dependency-free codec the raw-source envelope uses)
// ---------------------------------------------------------------------------

/// Append a u64 as an LEB128 varint.
fn put_u64(out: &mut Vec<u8>, mut n: u64) {
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
fn get_u64(buf: &[u8], pos: &mut usize) -> Result<u64, ThemeCollectionError> {
    let mut len: u64 = 0;
    let mut shift = 0u32;
    loop {
        let b = *buf.get(*pos).ok_or_else(|| {
            ThemeCollectionError::Malformed("varint runs past end of buffer".into())
        })?;
        *pos += 1;
        len |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(ThemeCollectionError::Malformed(
                "varint length overflow".into(),
            ));
        }
    }
    Ok(len)
}

/// Append `bytes` length-prefixed (LEB128 length + raw bytes).
fn put_blob(out: &mut Vec<u8>, bytes: &[u8]) {
    put_u64(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

/// Read one length-prefixed blob with full bounds checking.
fn get_blob<'a>(buf: &'a [u8], pos: &mut usize) -> Result<&'a [u8], ThemeCollectionError> {
    let len = get_u64(buf, pos)? as usize;
    let end = pos
        .checked_add(len)
        .filter(|&e| e <= buf.len())
        .ok_or_else(|| ThemeCollectionError::Malformed("blob runs past end of buffer".into()))?;
    let b = &buf[*pos..end];
    *pos = end;
    Ok(b)
}

// ---------------------------------------------------------------------------
// The deterministic directory-archive codec
// ---------------------------------------------------------------------------

/// Encode a theme collection into the portable, DETERMINISTIC archive blob:
/// `put_u64(count)` then `put_blob(path) put_blob(content)` per entry, in
/// sorted-path order. Entries are sorted here (the caller need not pre-sort),
/// so the SAME directory always produces byte-identical bytes — the
/// reproducibility content-addressing requires (Dolstra 2006). Duplicate paths
/// are kept (the archive is a faithful image of the tree); the deterministic
/// order is the only normalization.
#[must_use]
pub fn encode_collection(files: &[ThemeFile]) -> Vec<u8> {
    let mut sorted: Vec<&ThemeFile> = files.iter().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    let mut out = Vec::new();
    put_u64(&mut out, sorted.len() as u64);
    for f in sorted {
        put_blob(&mut out, f.path.as_bytes());
        put_blob(&mut out, &f.content);
    }
    out
}

/// Decode a theme-collection archive blob back into the in-order theme files —
/// the exact inverse of [`encode_collection`] for canonical (sorted) input.
/// Fail-closed: a truncated/malformed blob, a non-UTF-8 path, or entries that
/// are NOT in strictly-sorted-path order are each an `Err` (the last forbids a
/// re-ordered, non-canonical archive from passing as if it were the pinned
/// one). This is the registry-side decoder entry point the
/// [`DecoderTotalityPerKind`] axiom consults via [`super::has_decoder_for`].
///
/// [`DecoderTotalityPerKind`]: super::super::ontology::DecoderTotalityPerKind
pub fn decode(bytes: &[u8]) -> Result<ThemeCollection, ThemeCollectionError> {
    let mut pos = 0usize;
    let count = get_u64(bytes, &mut pos)? as usize;
    let mut out: ThemeCollection = Vec::with_capacity(count);
    let mut prev: Option<String> = None;
    for _ in 0..count {
        let path_bytes = get_blob(bytes, &mut pos)?;
        let path = core::str::from_utf8(path_bytes)
            .map_err(|e| ThemeCollectionError::PathNotUtf8(e.to_string()))?
            .to_string();
        if let Some(p) = &prev
            && p.as_str() > path.as_str()
        {
            return Err(ThemeCollectionError::NotCanonical {
                prev: p.clone(),
                next: path,
            });
        }
        let content = get_blob(bytes, &mut pos)?.to_vec();
        prev = Some(path.clone());
        out.push(ThemeFile { path, content });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// std-only: archive a directory tree into the deterministic blob
// ---------------------------------------------------------------------------

/// Walk a theme-collection root directory (e.g. the fetched `tinted-schemes`
/// checkout) and archive every `*.yaml` / `*.yml` scheme file under the
/// `base16/` and `base24/` subtrees into the deterministic archive blob.
///
/// Only the two scheme subtrees are archived (the corpus the validator scans);
/// repository scaffolding (`README.md`, `LICENSE`, `.github/`, `scripts/`) is
/// excluded — it is not part of the color-scheme vocabulary. Paths are stored
/// collection-root-relative and `/`-separated. The result is the source's
/// canonical raw bytes: write it to the `<name>-<version>.themes` on-disk file
/// and the generalized raw-source emit treats it as one source.
///
/// This is the `pr4xis update` / regenerate side; the runtime load side never
/// touches the filesystem (it decodes the committed `.prx`).
#[cfg(feature = "std")]
pub fn archive_directory(root: &std::path::Path) -> std::io::Result<Vec<u8>> {
    let mut files: Vec<ThemeFile> = Vec::new();
    for family in ["base16", "base24"] {
        let dir = root.join(family);
        if !dir.is_dir() {
            continue;
        }
        collect_yaml(&dir, family, &mut files)?;
    }
    Ok(encode_collection(&files))
}

/// Recursively collect `*.yaml` / `*.yml` files under `dir`, recording each
/// with the `/`-joined path `rel_prefix/.../<file>`.
#[cfg(feature = "std")]
fn collect_yaml(
    dir: &std::path::Path,
    rel_prefix: &str,
    out: &mut Vec<ThemeFile>,
) -> std::io::Result<()> {
    // Read + sort the directory entries so the walk order is deterministic
    // (the encoder re-sorts by full path too, but a deterministic walk keeps
    // the regenerate output stable regardless of filesystem iteration order).
    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let rel = format!("{rel_prefix}/{name}");
        if path.is_dir() {
            collect_yaml(&path, &rel, out)?;
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("yaml") | Some("yml")
        ) {
            let content = std::fs::read(&path)?;
            out.push(ThemeFile { path: rel, content });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn tf(path: &str, content: &[u8]) -> ThemeFile {
        ThemeFile {
            path: path.to_string(),
            content: content.to_vec(),
        }
    }

    #[test]
    fn encode_decode_round_trips_exact() {
        let files = vec![
            tf(
                "base16/apathy/default.yaml",
                b"system: base16\nbase00: \"#000\"\n",
            ),
            tf(
                "base24/cyberpunk/default.yaml",
                b"system: base24\nbase00: \"#111\"\n",
            ),
            tf("base16/zzz/default.yaml", b""),
        ];
        let blob = encode_collection(&files);
        let back = decode(&blob).expect("decode");
        // The decode yields the SAME files, in sorted-path order.
        let mut expect = files.clone();
        expect.sort_by(|a, b| a.path.cmp(&b.path));
        assert_eq!(back, expect);
    }

    #[test]
    fn encode_is_deterministic_regardless_of_input_order() {
        let a = vec![
            tf("b/2.yaml", b"two"),
            tf("a/1.yaml", b"one"),
            tf("a/0.yaml", b"zero"),
        ];
        let mut b = a.clone();
        b.reverse();
        // Same set, different input order → byte-identical archive (the
        // deterministic property content-addressing depends on).
        assert_eq!(encode_collection(&a), encode_collection(&b));
    }

    #[test]
    fn empty_collection_round_trips() {
        let blob = encode_collection(&[]);
        assert_eq!(
            decode(&blob).expect("decode empty"),
            Vec::<ThemeFile>::new()
        );
    }

    #[test]
    fn decode_rejects_truncated_archive_without_panic() {
        let files = vec![tf("base16/x/default.yaml", b"some yaml bytes here")];
        let mut blob = encode_collection(&files);
        blob.truncate(blob.len() - 3);
        let err = decode(&blob).expect_err("truncated must be Err");
        assert!(
            matches!(err, ThemeCollectionError::Malformed(_)),
            "got {err:?}"
        );
    }

    #[test]
    fn decode_rejects_non_canonical_order_fail_closed() {
        // Hand-build an archive with entries out of sorted order; the decoder
        // must reject it (a re-ordered archive is not the canonical/pinned one).
        let mut blob = Vec::new();
        put_u64(&mut blob, 2);
        put_blob(&mut blob, b"base16/z.yaml");
        put_blob(&mut blob, b"z");
        put_blob(&mut blob, b"base16/a.yaml");
        put_blob(&mut blob, b"a");
        let err = decode(&blob).expect_err("out-of-order must be Err");
        assert!(
            matches!(err, ThemeCollectionError::NotCanonical { .. }),
            "got {err:?}"
        );
    }

    proptest! {
        /// FORALL round-trip: any generated set of theme files (arbitrary
        /// `/`-pathed names, arbitrary byte contents) archives → decodes back to
        /// the SAME set in sorted-path order — the GetPut leg of the collection
        /// ⇄ archive lens, exercised across sizes, paths, and contents.
        #[test]
        fn prop_collection_round_trips(
            files in proptest::collection::vec(
                ("[a-z0-9/_-]{1,24}", proptest::collection::vec(any::<u8>(), 0..64)),
                0..30,
            )
        ) {
            let files: Vec<ThemeFile> = files
                .into_iter()
                .map(|(path, content)| ThemeFile { path, content })
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
}

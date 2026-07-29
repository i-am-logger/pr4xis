//! Theme-collection decoder — the directory-archive codec for the
//! `ContentType::ThemeCollection` source (the Base16/Base24 named
//! color-scheme corpus, the Tinted Theming `tinted-schemes` dataset).
//!
//! The archive/decode wire format is [`file_collection`]
//! — a fully generic, dependency-free `path → bytes` codec shared with every
//! other MANY-file raw-source collection (e.g. the VerbNet class-XML
//! collection, [`verbnet_class_collection`](super::verbnet_class_collection)).
//! This module supplies only the theme-specific piece: walking a fetched
//! `tinted-schemes` checkout and filtering it down to the `base16/`/`base24/`
//! YAML scheme files that constitute the color-scheme vocabulary.
//!
//! ## Citations
//!
//! - **Tinted Theming** — *Base16 Styling Guidelines* + *Base24
//!   specification*, <https://github.com/tinted-theming/home>: the named
//!   color-scheme corpus this collection archives.
//! - **Dolstra, E. (2006)** *The Purely Functional Software Deployment Model*
//!   — content-addressing requires a deterministic (reproducible) artifact;
//!   the sorted-path archive realises it (see
//!   [`file_collection`] for the codec itself).

use super::file_collection::{self, CollectionFile, FileCollection, FileCollectionError};

/// The [`ContentType`](crate::applied::data_provisioning::ontology::ContentType)
/// this module realizes — the single declaration of which content type this
/// file decodes, read by `super::has_decoder_for` (audit 2026-06-12 D-22).
pub const DECODES: crate::applied::data_provisioning::ontology::ContentType =
    crate::applied::data_provisioning::ontology::ContentType::ThemeCollection;

/// One archived theme file — a thin alias of the generic collection entry,
/// kept as a named type for call-site readability at theme-collection sites.
pub type ThemeFile = CollectionFile;

/// The decoded theme collection — the in-order (sorted-path) set of archived
/// theme files. The structure the theming validator iterates, the directory-
/// archive analogue of `plaintext_tsv::TsvRecords`.
pub type ThemeCollection = FileCollection;

/// A failure decoding a `ThemeCollection` archive — re-exported from the
/// generic codec's error type (the failure modes are format-level, not
/// theme-specific).
pub type ThemeCollectionError = FileCollectionError;

/// Encode a theme collection into the portable, deterministic archive blob.
/// Thin re-export of [`file_collection::encode_collection`].
#[must_use]
pub fn encode_collection(files: &[ThemeFile]) -> alloc::vec::Vec<u8> {
    file_collection::encode_collection(files)
}

/// Decode a theme-collection archive blob back into the in-order theme files.
/// Thin re-export of [`file_collection::decode`], kept under this module's
/// own error alias so existing theme-collection call sites are unaffected.
pub fn decode(bytes: &[u8]) -> Result<ThemeCollection, ThemeCollectionError> {
    file_collection::decode(bytes)
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
pub fn archive_directory(root: &std::path::Path) -> std::io::Result<alloc::vec::Vec<u8>> {
    let mut files: alloc::vec::Vec<ThemeFile> = alloc::vec::Vec::new();
    for family in ["base16", "base24"] {
        let dir = root.join(family);
        if !dir.is_dir() {
            continue;
        }
        collect_by_extension(&dir, family, &["yaml", "yml"], &mut files)?;
    }
    Ok(encode_collection(&files))
}

/// Recursively collect files under `dir` whose extension is one of `exts`,
/// recording each with the `/`-joined path `rel_prefix/.../<file>`. Shared
/// walk shape with `verbnet_class_collection`'s own collector (extension set
/// and root differ; the recursion/sorting logic is identical, small enough
/// that duplicating it beats a premature `walk_by_extension` abstraction over
/// two call sites with slightly different depth-cutoff needs).
#[cfg(feature = "std")]
fn collect_by_extension(
    dir: &std::path::Path,
    rel_prefix: &str,
    exts: &[&str],
    out: &mut alloc::vec::Vec<ThemeFile>,
) -> std::io::Result<()> {
    let mut entries: alloc::vec::Vec<std::path::PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let rel = alloc::format!("{rel_prefix}/{name}");
        if path.is_dir() {
            collect_by_extension(&path, &rel, exts, out)?;
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| exts.contains(&e))
        {
            let content = std::fs::read(&path)?;
            out.push(ThemeFile { path: rel, content });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tf(path: &str, content: &[u8]) -> ThemeFile {
        ThemeFile {
            path: path.to_string(),
            content: content.to_vec(),
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn theme_collection_round_trips_through_the_shared_codec() {
        let files = vec![
            tf(
                "base16/apathy/default.yaml",
                b"system: base16\nbase00: \"#000\"\n",
            ),
            tf(
                "base24/cyberpunk/default.yaml",
                b"system: base24\nbase00: \"#111\"\n",
            ),
        ];
        let blob = encode_collection(&files);
        let back = decode(&blob).expect("decode");
        let mut expect = files.clone();
        expect.sort_by(|a, b| a.path.cmp(&b.path));
        assert_eq!(back, expect);
    }
}

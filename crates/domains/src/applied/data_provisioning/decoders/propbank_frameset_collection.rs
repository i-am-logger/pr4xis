//! PropBank frameset-collection decoder — the directory-archive codec for the
//! `ContentType::PropBankFramesetCollection` source (Palmer, Gildea &
//! Kingsbury 2005; Bonial, Bonn, Conger, Hwang & Palmer 2014).
//!
//! The archive/decode wire format is [`file_collection`]
//! — the SAME generic, dependency-free `path → bytes` codec
//! [`verbnet_class_collection`](super::verbnet_class_collection) and
//! [`theme_collection`](super::theme_collection) use. This module supplies
//! only the PropBank-specific piece: walking a fetched
//! `propbank/propbank-frames` checkout (pinned to tag `v3.4.0`, commit
//! `4087fa9ab5c40907c34ff91a56acc2cab1670145`) and archiving every per-lemma
//! `frames/<lemma>.xml` frameset file. Parsing a file's content into typed
//! frameset/predicate/roleset/alias structure is the loaded PropBank
//! ontology's job (it consumes the decoded `path → bytes` set), not this
//! decoder's — mirroring how `verbnet_class_collection` doesn't parse
//! `<VNCLASS>` XML either.
//!
//! ## Why a whole-directory collection, not FrameNet's extracted TSV
//!
//! `frames/` at the pinned commit is 7,568 directory entries (7,565 `.xml`
//! frame files + `.gitignore` + `README.txt` + `frameset.dtd`), 29,794,394
//! bytes total (verified 2026-07-13 via `gh api
//! repos/propbank/propbank-frames/git/trees/<sha>:frames`) — VerbNet's shape
//! (many small structured per-lemma XML files needing full nested parse:
//! `frameset → predicate → roleset → aliases`), not FrameNet's (one release
//! dominated by a huge annotated-sentence corpus stripped to 3 flat
//! attributes per file). Flattening PropBank to a TSV would require
//! re-encoding the multi-level repeated substructure (N aliases per roleset,
//! M rolesets per predicate) — exactly the shape this generic archive codec
//! exists for.
//!
//! ## Citations
//!
//! - **Palmer, M., Gildea, D. & Kingsbury, P. (2005)** "The Proposition
//!   Bank: An Annotated Corpus of Semantic Roles", *Computational
//!   Linguistics* 31(1):71-106 — PropBank itself.
//! - **Bonial, C., Bonn, J., Conger, K., Hwang, J. & Palmer, M. (2014)**
//!   "PropBank: Semantics of New Predicate Types", *LREC 2014* — the
//!   frame-file format consumers are asked to cite alongside the original
//!   paper (propbank.github.io).
//! - **Dolstra, E. (2006)** *The Purely Functional Software Deployment
//!   Model* — content-addressing requires a deterministic (reproducible)
//!   artifact; the sorted-path archive realises it (see [`file_collection`]
//!   for the codec itself).

use super::file_collection::{self, CollectionFile, FileCollection, FileCollectionError};

/// The [`ContentType`](crate::applied::data_provisioning::ontology::ContentType)
/// this module realizes — the single declaration of which content type this
/// file decodes, read by `super::has_decoder_for` (audit 2026-06-12 D-22).
pub const DECODES: crate::applied::data_provisioning::ontology::ContentType =
    crate::applied::data_provisioning::ontology::ContentType::PropBankFramesetCollection;

/// One archived PropBank frameset file — a thin alias of the generic
/// collection entry, kept as a named type for call-site readability at
/// PropBank sites.
pub type PropBankFramesetFile = CollectionFile;

/// The decoded PropBank frameset collection — the in-order (sorted-path) set
/// of archived per-lemma frame files. The structure the PropBank ontology's
/// loader iterates and parses into typed framesets.
pub type PropBankFramesetCollection = FileCollection;

/// A failure decoding a `PropBankFramesetCollection` archive — re-exported
/// from the generic codec's error type (the failure modes are format-level,
/// not PropBank-specific).
pub type PropBankFramesetCollectionError = FileCollectionError;

/// Encode a PropBank frameset collection into the portable, deterministic
/// archive blob. Thin re-export of [`file_collection::encode_collection`].
#[must_use]
pub fn encode_collection(files: &[PropBankFramesetFile]) -> alloc::vec::Vec<u8> {
    file_collection::encode_collection(files)
}

/// Decode a PropBank-frameset-collection archive blob back into the in-order
/// frame files. Thin re-export of [`file_collection::decode`], kept under
/// this module's own error alias.
pub fn decode(bytes: &[u8]) -> Result<PropBankFramesetCollection, PropBankFramesetCollectionError> {
    file_collection::decode(bytes)
}

/// Is `name` a real PropBank frame file, as opposed to the three non-frame
/// entries the real `frames/` directory also carries (`.gitignore`,
/// `README.txt`, `frameset.dtd` — confirmed 2026-07-13 via `gh api`: 7,568
/// tree entries, only 7,565 end `.xml`)? A pure extension check, fail-closed
/// (excludes anything that isn't `.xml`) rather than a hand-maintained
/// exclude-list that would drift if the upstream repo ever adds another
/// non-frame file.
#[must_use]
pub fn is_frame_xml_filename(name: &str) -> bool {
    name.ends_with(".xml")
}

// ---------------------------------------------------------------------------
// std-only: archive a directory tree into the deterministic blob
// ---------------------------------------------------------------------------

/// Walk a PropBank checkout's `frames/` directory (e.g. a `v3.4.0`-tagged
/// `propbank/propbank-frames` checkout) and archive every real frame `.xml`
/// file into the deterministic archive blob.
///
/// Only frame files are archived — `.gitignore`, `README.txt`, and
/// `frameset.dtd` are excluded via [`is_frame_xml_filename`] (they are
/// directory scaffolding / the schema the frame files validate against, not
/// frame data themselves). Paths are stored root-relative and `/`-separated.
/// The result is the source's canonical raw bytes: write it to the
/// `<name>-<version>.propbank` on-disk file and the generalized raw-source
/// emit treats it as one source.
///
/// This is the `pr4xis update` / regenerate side; the runtime load side never
/// touches the filesystem (it decodes the committed `.prx`).
#[cfg(feature = "std")]
pub fn archive_directory(frames_dir: &std::path::Path) -> std::io::Result<alloc::vec::Vec<u8>> {
    let mut files: alloc::vec::Vec<PropBankFramesetFile> = alloc::vec::Vec::new();
    let mut entries: alloc::vec::Vec<std::path::PathBuf> = std::fs::read_dir(frames_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if is_frame_xml_filename(&name) {
            let content = std::fs::read(&path)?;
            files.push(PropBankFramesetFile {
                path: name,
                content,
            });
        }
    }
    Ok(encode_collection(&files))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pf(path: &str, content: &[u8]) -> PropBankFramesetFile {
        PropBankFramesetFile {
            path: path.to_string(),
            content: content.to_vec(),
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn propbank_frameset_collection_round_trips_through_the_shared_codec() {
        let files = vec![
            pf(
                "trade.xml",
                br#"<frameset><predicate lemma="trade"><roleset id="trade.01"/></predicate></frameset>"#,
            ),
            pf(
                "abandon.xml",
                br#"<frameset><predicate lemma="abandon"><roleset id="abandon.01"/></predicate></frameset>"#,
            ),
        ];
        let blob = encode_collection(&files);
        let back = decode(&blob).expect("decode");
        let mut expect = files.clone();
        expect.sort_by(|a, b| a.path.cmp(&b.path));
        assert_eq!(back, expect);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_frame_xml_filename_excludes_the_three_real_non_frame_entries() {
        // The real `frames/` directory (verified 2026-07-13 via `gh api`
        // against propbank/propbank-frames commit
        // 4087fa9ab5c40907c34ff91a56acc2cab1670145) carries exactly these
        // three non-frame entries alongside 7,565 `.xml` frame files.
        assert!(!is_frame_xml_filename(".gitignore"));
        assert!(!is_frame_xml_filename("README.txt"));
        assert!(!is_frame_xml_filename("frameset.dtd"));
        assert!(is_frame_xml_filename("trade.xml"));
        assert!(is_frame_xml_filename("out_trade.xml"));
    }
}

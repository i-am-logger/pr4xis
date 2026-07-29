//! VerbNet class-collection decoder — the directory-archive codec for the
//! `ContentType::VerbNetClassCollection` source (the VerbNet 3.3 class-XML
//! corpus, Kipper, Korhonen, Ryant & Palmer 2008).
//!
//! The archive/decode wire format is [`file_collection`]
//! — the SAME generic, dependency-free `path → bytes` codec the color-scheme
//! collection ([`theme_collection`](super::theme_collection)) uses. This
//! module supplies only the VerbNet-specific piece: walking a fetched
//! `cu-clear/verbnet` checkout (tag `vn-3.3`) and archiving every per-class
//! `<VNCLASS>` XML file. Parsing a file's content into typed class/member/
//! subclass structure is the loaded VerbNet ontology's job (it consumes the
//! decoded `path → bytes` set), not this decoder's — mirroring how
//! `theme_collection` doesn't parse YAML palettes either.
//!
//! ## Citations
//!
//! - **Kipper, K., Korhonen, A., Ryant, N. & Palmer, M. (2008)** "A
//!   Large-scale Classification of English Verbs", *Language Resources and
//!   Evaluation* 42(1):21-40 — VerbNet itself.
//! - **Levin, B. (1993)** *English Verb Classes and Alternations*, University
//!   of Chicago Press — the syntactic-alternation diagnostics VerbNet's class
//!   hierarchy is built from.
//! - **Dolstra, E. (2006)** *The Purely Functional Software Deployment Model*
//!   — content-addressing requires a deterministic (reproducible) artifact;
//!   the sorted-path archive realises it (see
//!   [`file_collection`] for the codec itself).

use super::file_collection::{self, CollectionFile, FileCollection, FileCollectionError};

/// The [`ContentType`](crate::applied::data_provisioning::ontology::ContentType)
/// this module realizes — the single declaration of which content type this
/// file decodes, read by `super::has_decoder_for` (audit 2026-06-12 D-22).
pub const DECODES: crate::applied::data_provisioning::ontology::ContentType =
    crate::applied::data_provisioning::ontology::ContentType::VerbNetClassCollection;

/// One archived VerbNet class file — a thin alias of the generic collection
/// entry, kept as a named type for call-site readability at VerbNet sites.
pub type VerbNetClassFile = CollectionFile;

/// The decoded VerbNet class collection — the in-order (sorted-path) set of
/// archived per-class XML files. The structure the VerbNet ontology's loader
/// iterates and parses into typed classes.
pub type VerbNetClassCollection = FileCollection;

/// A failure decoding a `VerbNetClassCollection` archive — re-exported from
/// the generic codec's error type (the failure modes are format-level, not
/// VerbNet-specific).
pub type VerbNetClassCollectionError = FileCollectionError;

/// Encode a VerbNet class collection into the portable, deterministic archive
/// blob. Thin re-export of [`file_collection::encode_collection`].
#[must_use]
pub fn encode_collection(files: &[VerbNetClassFile]) -> alloc::vec::Vec<u8> {
    file_collection::encode_collection(files)
}

/// Decode a VerbNet-class-collection archive blob back into the in-order
/// class files. Thin re-export of [`file_collection::decode`], kept under
/// this module's own error alias.
pub fn decode(bytes: &[u8]) -> Result<VerbNetClassCollection, VerbNetClassCollectionError> {
    file_collection::decode(bytes)
}

// ---------------------------------------------------------------------------
// std-only: archive a directory tree into the deterministic blob
// ---------------------------------------------------------------------------

/// Walk a VerbNet checkout's class directory (e.g. the `verbnet3.3/` tree of
/// a `vn-3.3`-tagged `cu-clear/verbnet` checkout) and archive every
/// `<ClassId>.xml` file into the deterministic archive blob.
///
/// Only the flat class-file directory is archived — repository scaffolding
/// (`README.md`, `api/`, `vn-gl/`, `verbnet-test/`) is excluded, since those
/// are alternate encodings / test fixtures of the same class data, not the
/// class hierarchy itself. Paths are stored root-relative and `/`-separated.
/// The result is the source's canonical raw bytes: write it to the
/// `<name>-<version>.verbnet` on-disk file and the generalized raw-source
/// emit treats it as one source.
///
/// This is the `pr4xis update` / regenerate side; the runtime load side never
/// touches the filesystem (it decodes the committed `.prx`).
#[cfg(feature = "std")]
pub fn archive_directory(class_dir: &std::path::Path) -> std::io::Result<alloc::vec::Vec<u8>> {
    let mut files: alloc::vec::Vec<VerbNetClassFile> = alloc::vec::Vec::new();
    let mut entries: alloc::vec::Vec<std::path::PathBuf> = std::fs::read_dir(class_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if path.extension().and_then(|e| e.to_str()) == Some("xml") {
            let content = std::fs::read(&path)?;
            files.push(VerbNetClassFile {
                path: name,
                content,
            });
        }
    }
    Ok(encode_collection(&files))
}

/// The bundled path of the WordNet sense-key -> OEWN synset-id crosswalk
/// entry within the archived collection (see
/// `regenerate::regenerate_verbnet_archive`'s doc for why this is
/// precomputed rather than resolved at load time). A leading `_` sorts
/// before every real VerbNet class id (`stop-55.4`, `cut-21.1`, ...) and no
/// class id ever starts with an underscore, so this can never collide with a
/// real class file's path.
pub const WORDNET_CROSSWALK_PATH: &str = "_wordnet_crosswalk.tsv";

/// REGENERATE PATH (`--ignored`, WRITES): archive the checked-out `vn-3.3`
/// tag's `verbnet3.3/` class directory (`data/verbnet-checkout/verbnet3.3/`,
/// TRANSIENT/gitignored, populated by hand or `pr4xis update`) into the
/// deterministic `.verbnet` blob, PLUS a precomputed WordNet sense-key ->
/// OEWN synset-id crosswalk (see doc below) bundled alongside the class
/// files as one more collection entry, and write the result to the
/// git-tracked `data/verbnet/verbnet-3.3.verbnet` bundled source-of-truth.
/// Run by hand after re-syncing the checkout:
/// `cargo test -p pr4xis-domains -- --ignored regenerate_verbnet_archive`,
/// then `pr4xis compile --compact --lock` to re-pin the `.prx`. Mirrors
/// `regenerate_tinted_schemes_archive`.
///
/// ## Why the crosswalk is precomputed here, not resolved at load time
///
/// VerbNet's `wn="cut%2:30:00"` attribute is a Princeton WordNet SENSE key;
/// [`super::super::verbnet_sense_key::oewn_sense_id_for_sense_key`]
/// mechanically converts it to the OEWN `Sense` id it corresponds to
/// (`"oewn-cut__2.30.00.."`) — but `English`'s runtime `ConceptId`s are keyed
/// by SYNSET, and the raw `Sense.id` string is a build-time-only local
/// `English::from_wordnet` intentionally discards (see that function's own
/// disposal comment) — no accessor from a Sense-id string to a `ConceptId`
/// survives past construction, and `English`'s committed compact `.prx`
/// never re-parses raw XML at normal runtime load (by design, to avoid the
/// ~89 MB parse cost). Re-parsing WordNet's raw XML *again*, independently,
/// just to complete this crosswalk at ordinary VerbNet-load time would both
/// duplicate that cost and reintroduce the exact XML-parse-at-startup
/// English's compact-archive path was built to avoid. So the sense-id ->
/// synset-id resolution happens ONCE, here, in this `--ignored` (offline,
/// hand-run) regeneration step — which CAN afford the one-time WordNet XML
/// parse — and the small RESULT (only the ~5,800 sense-keys VerbNet members
/// actually reference, not the full ~185k-sense WordNet) ships as ordinary
/// committed data, exactly like `english_irregulars.tsv` is a DERIVED source
/// computed once from AGID rather than re-derived at runtime.
#[cfg(feature = "std")]
#[cfg(test)]
mod regenerate {
    /// Build the `sense_key\t<ConceptId numeric value>` crosswalk TSV bytes
    /// for every WordNet sense-key referenced by any member across
    /// `classes`, by loading the raw registered WordNet source, building a
    /// fresh `English` from it, and resolving each crosswalked OEWN sense-id
    /// through its synset id (a one-time fold over
    /// `WordNet.entries[].senses[]`, mirroring `English::from_wordnet`'s own
    /// Phase 2 loop) to `English::concept_by_synset`'s `ConceptId`.
    ///
    /// The crosswalk targets `ConceptId`'s numeric value, NOT the OEWN
    /// synset-id STRING (`"oewn-00293269-v"`) — verified empirically that
    /// `ConceptView::original_id()` only returns that real string on this
    /// freshly-built (raw-XML) `English`; the committed compact/store-bundle
    /// archive `english_loaded()` normally serves at runtime instead returns
    /// a SYNTHETIC placeholder (`"s{concept_id}"`, literally the stringified
    /// `ConceptId`, per `lmf::compact::decode`'s own doc: "index-derived
    /// synthetic ids in place of the original `oewn-…` strings ... the only
    /// difference is `Concept::original_id`"). `ConceptId` itself, by that
    /// same doc's explicit guarantee, IS identical between the raw and
    /// compact/store-bundle load paths ("same `ConceptId`s, same
    /// relations") — so it is the crosswalk's actually-stable target.
    /// Sense-keys with no resolvable concept (a handful of VerbNet members
    /// predate full WordNet coverage) are silently omitted — an absent
    /// crosswalk entry is exactly the honest "no data" case
    /// `VerbNetStore::has_coverage` is designed to represent.
    fn build_wordnet_crosswalk_tsv(
        classes: &[crate::cognitive::linguistics::verbnet::ontology::VerbNetClass],
    ) -> String {
        use crate::applied::data_provisioning::registry::data_sources;
        use crate::cognitive::linguistics::english::English;
        use crate::cognitive::linguistics::verbnet::store::oewn_sense_id_for_sense_key;
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
        use crate::social::software::markup::xml::lmf::reader::read_wordnet;
        use alloc::collections::BTreeMap;

        // Selected by kind, mirroring `english_load_owned_inner`'s own lookup
        // (English is the sole Language implementor) — not a name literal, so
        // a registry rename can never silently desync this regen step.
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let entry = data_sources()
            .iter()
            .find(|e| e.kind == SourceTaxonomyConcept::Language)
            .expect("no Language-kind source registered");
        let xml_path = workspace_root.join(entry.local_path());
        let xml = std::fs::read_to_string(&xml_path).expect("read WordNet XML");
        let wn = read_wordnet(&xml).expect("parse WordNet XML");
        let english = English::from_wordnet(&wn);

        let mut sense_id_to_synset: BTreeMap<String, String> = BTreeMap::new();
        for lex_entry in &wn.entries {
            for sense in &lex_entry.senses {
                sense_id_to_synset.insert(sense.id.clone(), sense.synset.clone());
            }
        }

        let mut sense_keys: alloc::collections::BTreeSet<String> =
            alloc::collections::BTreeSet::new();
        for class in classes {
            for c in class.self_and_descendants() {
                for member in &c.members {
                    for key in &member.wn_sense_keys {
                        sense_keys.insert(key.clone());
                    }
                }
            }
        }

        let mut rows: Vec<String> = Vec::new();
        for key in &sense_keys {
            let Some(sense_id) = oewn_sense_id_for_sense_key(key) else {
                continue;
            };
            let Some(synset_id) = sense_id_to_synset.get(&sense_id) else {
                continue;
            };
            let Some(concept) = english.concept_by_synset(synset_id) else {
                continue;
            };
            rows.push(format!("{key}\t{}", concept.id().value()));
        }
        rows.sort();
        rows.join("\n")
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    #[ignore]
    fn regenerate_verbnet_archive() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/verbnet-checkout/verbnet3.3");
        let blob = super::archive_directory(&root).expect("archive verbnet3.3 checkout");

        // Augment with the precomputed WordNet crosswalk (see module doc):
        // decode the archived collection back to typed classes, resolve
        // every referenced sense-key to its synset, re-encode with the
        // crosswalk as one more collection entry.
        let collection = super::decode(&blob).expect("decode freshly-built archive");
        let vn = crate::cognitive::linguistics::verbnet::reader::read_verbnet(&collection);
        let crosswalk_tsv = build_wordnet_crosswalk_tsv(&vn.classes);
        let mut files = collection;
        files.push(
            crate::applied::data_provisioning::decoders::file_collection::CollectionFile {
                path: super::WORDNET_CROSSWALK_PATH.to_string(),
                content: crosswalk_tsv.into_bytes(),
            },
        );
        let blob = super::encode_collection(&files);

        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/verbnet/verbnet-3.3.verbnet");
        std::fs::write(&out, &blob).expect("write verbnet-3.3.verbnet");
        eprintln!(
            "wrote {} ({} bytes) address {}",
            out.display(),
            blob.len(),
            pr4xis_runtime::address::ContentAddress::of(&blob).to_hex()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vf(path: &str, content: &[u8]) -> VerbNetClassFile {
        VerbNetClassFile {
            path: path.to_string(),
            content: content.to_vec(),
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn verbnet_class_collection_round_trips_through_the_shared_codec() {
        let files = vec![
            vf(
                "stop-55.4.xml",
                br#"<VNCLASS ID="stop-55.4"><MEMBERS><MEMBER name="cut" wn="cut%2:30:00"/></MEMBERS></VNCLASS>"#,
            ),
            vf(
                "cut-21.1.xml",
                br#"<VNCLASS ID="cut-21.1"><MEMBERS><MEMBER name="cut" wn="cut%2:35:00"/></MEMBERS></VNCLASS>"#,
            ),
        ];
        let blob = encode_collection(&files);
        let back = decode(&blob).expect("decode");
        let mut expect = files.clone();
        expect.sort_by(|a, b| a.path.cmp(&b.path));
        assert_eq!(back, expect);
    }
}

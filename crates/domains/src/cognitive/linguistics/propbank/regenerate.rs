//! Offline regeneration of the committed PropBank bundle: a deterministic
//! directory archive of every `frames/*.xml` frameset file, VerbNet's
//! whole-directory-collection codec — not FrameNet's/SUMO's
//! field-extraction-to-TSV shape.
//!
//! ## Why a whole-directory archive, not an extraction
//!
//! Unlike FrameNet's regen (which strips each LU file down to 3 flat
//! attributes, discarding a huge embedded annotated-sentence corpus) or
//! SUMO's regen (which extracts `&%<term><suffix>` annotations AND resolves
//! them to a `ConceptId` offline), PropBank's `frames/*.xml` files ARE
//! already the exact shape this project needs: small, structured,
//! nested-XML per-lemma documents (`frameset → predicate → roleset →
//! aliases`) with no oversized payload to strip out. Flattening that nested
//! structure to a TSV would require re-encoding N aliases × M rolesets per
//! predicate — exactly the shape [`file_collection`](crate::applied::data_provisioning::decoders::file_collection)
//! (VerbNet's own archive codec) exists for. So this module does NO field
//! extraction: it walks the checkout, archives every real frame file
//! verbatim via
//! [`propbank_frameset_collection::archive_directory`](crate::applied::data_provisioning::decoders::propbank_frameset_collection::archive_directory),
//! and the FIELD parsing (predicate/roleset/alias structure, the POS-code
//! → `LmfPos` mapping) happens LIVE, at load time, in [`super::reader`] —
//! mirroring VerbNet's own division of labor exactly.
//!
//! ## Prerequisite (external, not run by this module)
//!
//! ```text
//! mkdir -p crates/domains/data/propbank-checkout
//! git clone https://github.com/propbank/propbank-frames.git \
//!   crates/domains/data/propbank-checkout/propbank-frames
//! git -C crates/domains/data/propbank-checkout/propbank-frames checkout \
//!   4087fa9ab5c40907c34ff91a56acc2cab1670145
//! ```
//!
//! `data/propbank-checkout/` is gitignored (transient staging, mirroring
//! `data/verbnet-checkout/`, `data/framenet-download/`, and
//! `data/sumo-download/`) — only this module's OUTPUT, the deterministic
//! `.propbank` archive, is committed. The source has a real tagged release
//! (`v3.4.0`); the fetch is pinned by BOTH the tag and the commit SHA it
//! resolves to (recorded in the `[sources.propbank]` registry description),
//! following VerbNet's tag-pinned template rather than SUMO's commit-only
//! style.

/// The tagged release this regen is pinned to — `v3.4.0`. Independently
/// verified 2026-07-13 via `gh api repos/propbank/propbank-frames/tags`
/// against [`PROPBANK_COMMIT_SHA`] (see that const's own doc for the exact
/// call). This whole module is already `#[cfg(feature = "std")] #[cfg(test)]`-
/// gated at its `mod regenerate;` declaration in `mod.rs`, so nothing here
/// needs its own redundant `#[cfg(test)]`.
const PROPBANK_TAG: &str = "v3.4.0";

/// The commit SHA tag `v3.4.0` resolves to — verified 2026-07-13 via
/// `gh api repos/propbank/propbank-frames/tags` (`.[] | select(.name==
/// "v3.4.0") | .commit.sha`), returning exactly this value.
const PROPBANK_COMMIT_SHA: &str = "4087fa9ab5c40907c34ff91a56acc2cab1670145";

#[pr4xis::praxis_value(Deterministic)]
#[test]
#[ignore]
fn regenerate_propbank_archive() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/propbank-checkout/propbank-frames/frames");
    let blob =
        crate::applied::data_provisioning::decoders::propbank_frameset_collection::archive_directory(
            &root,
        )
        .unwrap_or_else(|e| panic!("archive propbank-frames/frames checkout ({PROPBANK_TAG} / {PROPBANK_COMMIT_SHA}): {e}"));

    // Sanity check against the real, independently-verified frame count
    // (7,565 `.xml` files — confirmed 2026-07-13 via `gh api
    // repos/propbank/propbank-frames/git/trees/<sha>:frames`) before
    // committing, so a partial/wrong checkout fails loudly rather than
    // silently shipping a truncated archive.
    let collection =
        crate::applied::data_provisioning::decoders::propbank_frameset_collection::decode(&blob)
            .expect("decode freshly-built archive");
    eprintln!("archived {} frame files", collection.len());
    assert_eq!(
        collection.len(),
        7565,
        "expected exactly 7,565 archived .xml frame files (the real, gh-api-verified count \
         for {PROPBANK_TAG} / {PROPBANK_COMMIT_SHA}) — got {}; checkout may be partial or stale",
        collection.len()
    );

    // Exercise the reader/store over the freshly-archived data as a
    // post-regen smoke test — mirrors SUMO's regen printing resolution
    // stats, verifying the pipeline end to end before anything is committed.
    let pb = crate::cognitive::linguistics::propbank::reader::read_propbank(&collection);
    let mut roleset_count = 0usize;
    let mut alias_count = 0usize;
    let mut mapped_pos_alias_count = 0usize;
    for frameset in &pb.framesets {
        for predicate in &frameset.predicates {
            for roleset in &predicate.rolesets {
                roleset_count += 1;
                alias_count += roleset.aliases.len();
                mapped_pos_alias_count +=
                    roleset.aliases.iter().filter(|a| a.pos.is_some()).count();
            }
        }
    }
    eprintln!(
        "parsed {} framesets, {roleset_count} rolesets, {alias_count} aliases \
         ({mapped_pos_alias_count} with a mapped LmfPos)",
        pb.framesets.len()
    );

    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/propbank/propbank-3.4.0.propbank");
    std::fs::create_dir_all(out.parent().expect("has parent")).expect("mkdir data/propbank");
    std::fs::write(&out, &blob).unwrap_or_else(|e| panic!("write {}: {e}", out.display()));
    eprintln!(
        "wrote {} ({} bytes) address {}",
        out.display(),
        blob.len(),
        pr4xis_runtime::address::ContentAddress::of(&blob).to_hex()
    );
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn pinned_commit_sha_is_a_well_formed_forty_char_hex_string() {
        // Guards against a typo'd pin — the same protective discipline as
        // this codebase's other commit-SHA pins (e.g. SUMO's
        // `ontologyportal/sumo` master pin), just made a real assertion
        // instead of prose-only.
        assert_eq!(PROPBANK_COMMIT_SHA.len(), 40, "{PROPBANK_COMMIT_SHA}");
        assert!(
            PROPBANK_COMMIT_SHA.chars().all(|c| c.is_ascii_hexdigit()),
            "{PROPBANK_COMMIT_SHA}"
        );
        assert!(
            PROPBANK_COMMIT_SHA.chars().all(|c| !c.is_ascii_uppercase()),
            "commit SHAs are conventionally lowercase: {PROPBANK_COMMIT_SHA}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn pinned_tag_matches_the_expected_release_string() {
        assert_eq!(PROPBANK_TAG, "v3.4.0");
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn archiving_a_small_synthetic_collection_round_trips_through_the_regen_codec() {
        // A lightweight smoke test of the codec this module's ignored regen
        // test drives, without touching the filesystem or the real
        // 7,565-file checkout — the encode/decode round trip the regen
        // relies on, exercised deterministically.
        use crate::applied::data_provisioning::decoders::propbank_frameset_collection::{
            PropBankFramesetFile, decode, encode_collection,
        };
        let files = alloc::vec![
            PropBankFramesetFile {
                path: "abandon.xml".to_string(),
                content: b"<frameset/>".to_vec(),
            },
            PropBankFramesetFile {
                path: "trade.xml".to_string(),
                content: b"<frameset><predicate lemma=\"trade\"/></frameset>".to_vec(),
            },
        ];
        let blob = encode_collection(&files);
        let back = decode(&blob).expect("decode");
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].path, "abandon.xml");
        assert_eq!(back[1].path, "trade.xml");
    }
}

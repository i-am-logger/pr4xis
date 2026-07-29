//! Product-metric regression gates for the compact U.S. Code `.prx`.
//!
//! These assert — over EVERY on-disk USC title (source-agnostic via the
//! data-source registry, not a hardcoded list) — the two properties the `.prx`
//! work exists to deliver, which the retro found went unmeasured for a week:
//!
//! 1. **Compactness** — the shipped `compact .prx.gz` is smaller than fetching
//!    the source itself (`gzip(source)`). Fails closed if any title ever
//!    re-bloats past its own download.
//! 2. **Load-speed (the #271 win)** — materializing the corpus from the compact
//!    `.prx` is dramatically faster than parsing + materializing the raw USLM
//!    XML. Asserted on the aggregate with a generous 2× margin against the
//!    ~30× reality, so it gates the regression without flaking on CI jitter.
//!
//! Section-count parity rides along here: the compact-loaded section count must
//! equal the XML-parsed count for every title (covers the 113 MB Title 42). The
//! compact codec itself only round-trips its compact `(data, aux)` view —
//! `usc_envelope_bytes_round_trip_and_deterministic` (uslm/corpus/prx.rs) asserts
//! `decode(encode(envelope)) == envelope`, lossless over that projection but NOT
//! over the source bytes. Full byte-exact source losslessness is proven
//! separately, by the per-source reconstruct gates — the in-memory cousins in
//! uslm/corpus/prx.rs (`usc_raw_leaf_reconstructs_source_byte_exact`,
//! `usc_title1_graph_faithful_reconstructs_source_byte_exact`) and the
//! full-path corpus round-trip in this heavy-corpus lane
//! (`usc_round_trip.rs::usc_title1_graph_faithful_prx_round_trip_over_real_corpus`)
//! — this gate adds only the cheap corpus-scale parity check.
//!
//! Run under `cargo test` in the heavy-corpus lane; each title is read once in
//! this binary. Replaces the old `#[ignore]` `deep_dive_all_usc_titles` report.

use std::time::Instant;

use pr4xis_domains::applied::data_provisioning::registry::data_sources;
use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis_domains::formal::meta::well_behaved_lens::DecompileKind;
use pr4xis_domains::social::software::markup::xml::owl::prx::gzip;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::prx::test_internals::{
    compact_usc_from_succinct, compact_usc_to_succinct, title_to_owned,
};
use pr4xis_domains::social::software::markup::xml::uslm::corpus::prx::{
    emit_compact_usc_prx_gz, load_compact_usc_prx_gz,
};
use pr4xis_domains::social::software::markup::xml::uslm::{UsCode, read_uslm_title};
use praxis_corpus_tests::{require_provisioned, workspace_root};

/// gzip size of `bytes` — the apples-to-apples "source download" baseline.
fn gz_len(bytes: &[u8]) -> usize {
    use flate2::{Compression, write::GzEncoder};
    use std::io::Write;
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(bytes).expect("gz write");
    e.finish().expect("gz finish").len()
}

#[test]
fn compact_usc_prx_beats_source_download_and_loads_faster_than_xml_over_every_title() {
    let root = workspace_root();
    let mut measured = 0usize;
    let (mut tot_xml_ms, mut tot_prx_ms) = (0f64, 0f64);

    for entry in data_sources() {
        if DecompileKind::from_content_type(entry.content_type()) != Some(DecompileKind::UsCode) {
            continue;
        }
        let path = root.join(entry.local_path());
        let Ok(source) = std::fs::read(&path) else {
            continue; // title not provisioned on this checkout — skip gracefully
        };
        let text = core::str::from_utf8(&source).expect("USLM source is UTF-8");

        // ── XML path (the loaded() fallback): parse + materialize, timed ──
        let t = Instant::now();
        let title = read_uslm_title(text).expect("parse title");
        let xml_corpus = UsCode::from_uslm_titles_owned(vec![title]);
        let xml_ms = t.elapsed().as_secs_f64() * 1e3;

        // ── COMPACTNESS GATE: compact .prx.gz < gzip(source) ──
        let compact = emit_compact_usc_prx_gz(&source).expect("emit compact .prx.gz");
        let source_dl = gz_len(&source);
        assert!(
            compact.len() < source_dl,
            "{}: compact .prx.gz ({} B) is NOT smaller than the source download gzip(source) \
             ({} B) — compactness regressed",
            entry.name,
            compact.len(),
            source_dl,
        );

        // ── compact path (the loaded() fast path): gunzip + materialize, timed ──
        let t = Instant::now();
        let prx_corpus = load_compact_usc_prx_gz(&compact).expect("load compact .prx.gz");
        let prx_ms = t.elapsed().as_secs_f64() * 1e3;

        // ── LOSSLESSNESS: compact-loaded corpus equals the XML-parsed corpus ──
        assert_eq!(
            prx_corpus.section_count(),
            xml_corpus.section_count(),
            "{}: compact-loaded section count differs from the XML-parsed corpus",
            entry.name,
        );

        tot_xml_ms += xml_ms;
        tot_prx_ms += prx_ms;
        measured += 1;
        eprintln!(
            "{:<16} .prx.gz {:>6.2}MB vs gzip(src) {:>6.2}MB ({:>5.2}x)   load {:>6.0}ms vs xml {:>7.0}ms",
            entry.name,
            compact.len() as f64 / 1e6,
            source_dl as f64 / 1e6,
            compact.len() as f64 / source_dl.max(1) as f64,
            prx_ms,
            xml_ms,
        );
    }

    require_provisioned(measured, "usc");

    // ── LOAD-SPEED GATE: the compact fast path must be much faster than XML.
    //    Generous 2× margin vs the ~30× reality keeps it robust to CI jitter. ──
    assert!(
        tot_xml_ms > 2.0 * tot_prx_ms,
        "compact load total ({:.0}ms) is NOT >=2x faster than XML parse total ({:.0}ms) across \
         {measured} titles — the #271 fast-load win regressed",
        tot_prx_ms,
        tot_xml_ms,
    );
    eprintln!(
        "LOAD-SPEED across {measured} titles: XML {:.0}ms total vs compact {:.0}ms total = {:.0}x faster",
        tot_xml_ms,
        tot_prx_ms,
        tot_xml_ms / tot_prx_ms.max(1.0),
    );
}

/// COMPACTNESS GATE — for every registered, on-disk USC title small enough for
/// the per-test budget, the succinct `(data, aux)` codec round-trips losslessly
/// over the REAL corpus (`compact_usc_from_succinct(compact_usc_to_succinct(d,a))
/// == (d,a)`), the compact runtime `.prx.gz` is smaller than fetching its source
/// (`gzip(source)`), and it materializes to a corpus with the same section count.
/// Registry-driven (no hardcoded title set); a title not on disk is skipped
/// gracefully (USC titles are externally provisioned, not git-committed).
///
/// The succinct `(data, aux)` byte-exactness over REAL corpus bytes is the UNIQUE
/// assertion here: the sibling
/// `compact_usc_codec_roundtrips_smaller_and_reasoning_equivalent`
/// (uslm/corpus/prx.rs, fast lane) covers the codec only over SYNTHETIC fixture
/// data, and the corpus-scale gate above checks compactness + section_count, not
/// the succinct `(data, aux)` byte-exactness. Lifted into this heavy-corpus lane
/// because it parses real USC titles (≤ 16 MB cap); under nextest that parse was
/// paid per process-isolated test.
#[test]
fn compact_usc_prx_gz_smaller_than_source() {
    // Same proven-CI-safe cap the archive-anchor test uses — covers the small
    // + mid titles (1/28/18/29/50) while staying well under the lane budget.
    const CAP: u64 = 16 * 1024 * 1024;

    let root = workspace_root();
    let mut measured = 0usize;
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
            continue;
        }
        let path = root.join(entry.local_path());
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if meta.len() > CAP {
            continue;
        }
        let source = std::fs::read(&path).expect("read USC title");
        let text = core::str::from_utf8(&source).expect("UTF-8");
        let title = read_uslm_title(text).expect("parse title");
        let (data, aux) = title_to_owned(&title);

        let succ = compact_usc_to_succinct(&data, &aux);
        let (data_back, aux_back) = compact_usc_from_succinct(&succ);
        assert_eq!(data_back, data, "{}: data not lossless", entry.name);
        assert_eq!(aux_back, aux, "{}: aux not lossless", entry.name);

        let prx_gz = gzip(&succ).expect("gzip");
        let source_dl = gzip(&source).expect("gzip source").len();
        eprintln!(
            "USC-COMPACT {}@{}: .prx = {:.2}MB ({:.2}MB gz)  vs  SOURCE = {:.2}MB xml \
             ({:.2}MB .xml.gz)  ->  .prx.gz is {:.2}x the download",
            entry.name,
            entry.version,
            succ.len() as f64 / 1e6,
            prx_gz.len() as f64 / 1e6,
            source.len() as f64 / 1e6,
            source_dl as f64 / 1e6,
            prx_gz.len() as f64 / source_dl.max(1) as f64,
        );
        assert!(
            prx_gz.len() < source_dl,
            "{}: compact .prx.gz ({} B) NOT smaller than gzip(source) ({} B)",
            entry.name,
            prx_gz.len(),
            source_dl,
        );

        let usc = load_compact_usc_prx_gz(&prx_gz).expect("load compact");
        assert_eq!(
            usc.section_count().value,
            data.entity_count as f64,
            "{}: compact-loaded section count differs",
            entry.name
        );
        measured += 1;
    }
    // A title MUST be on disk within the cap — absence is a real failure (CI
    // provisions via `pr4xis update`), not a skip. The succinct codec's
    // correctness over synthetic fixtures is also covered by the inline-fixture
    // sibling `compact_usc_codec_roundtrips_smaller_and_reasoning_equivalent`
    // (fast lane), but this gate must still run over the REAL corpus.
    require_provisioned(measured, "usc");
}

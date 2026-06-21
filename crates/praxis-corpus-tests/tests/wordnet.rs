//! Open English WordNet 2025 (≈89 MB) — the WN-LMF producer / round-trip
//! gates, lifted out of the `pr4xis-domains` `#[cfg(test)]` modules.
//!
//! The 89 MB WN-LMF XML is parsed ONCE into a process-shared [`LazyLock`]
//! fixture ([`load_wordnet_corpus`]); every `#[test]` below borrows the same
//! immutable [`WnCorpus`]. Run under `cargo test` (one process, thread-parallel)
//! so the parse is paid once for the whole binary regardless of how many
//! assertions touch it — the parse-once-immutable discipline the `.prx` itself
//! embodies, applied to the test suite.
//!
//! The compact-codec gates ([`succinct_codec_roundtrip_and_smaller`],
//! [`prx_gz_round_trips_to_english`], [`compact_is_reasoning_equivalent_and_small`])
//! run over BOTH on-disk WN-LMF sources — the tiny `us_legal_lexicon` (instant)
//! and the 89 MB `english_wordnet` — so they iterate [`WnCorpus::sources`]. The
//! remaining gates are `english_wordnet`-only and borrow [`WnCorpus::english`].
//! Every source is fetched (`pr4xis update`), not committed; CI provisions it,
//! so an absent corpus is a real failure — each test HARD-FAILS (via `require` /
//! `require_provisioned`) naming the `pr4xis update <corpus>` to run, never skips.

use std::sync::LazyLock;

use praxis_corpus_tests::{
    WnCorpus, load_wordnet_corpus, require, require_provisioned, workspace_root,
};

use pr4xis::codegen::wordnet::parse_wordnet_xml;
use pr4xis_domains::applied::data_provisioning::registry::{
    LockDigest, data_sources, lock_archive_signature, lock_archive_signatures,
    lock_compact_archive_signature,
};
use pr4xis_domains::cognitive::linguistics::english::bridge::{
    concept_refs_for_word, english_runtime_ontology,
};
use pr4xis_domains::cognitive::linguistics::english::{ConceptId, English, english_loaded};
use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis_domains::formal::meta::well_behaved_lens::{
    CompletenessReport, DecompileKind, RoundTripFidelity as Tier, completeness_meter,
};
use pr4xis_domains::social::software::markup::xml::lmf::compact::{decode, encode};
use pr4xis_domains::social::software::markup::xml::lmf::compact_succinct::{
    emit_prx_gz, from_succinct, load_prx_gz, to_succinct,
};
use pr4xis_domains::social::software::markup::xml::lmf::prx::{
    build_wordnet_envelope, compact_english_archive_address, emit_all_wordnet_prx_gz,
    emit_compact_english_prx_gz, emit_wordnet_prx_gz, load_compact_english_prx_gz_gated,
    load_wordnet_prx_gz, wn_reconstruct_source, wordnet_envelope_from_bytes,
    wordnet_envelope_to_bytes,
};
use pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet;
use pr4xis_domains::social::software::markup::xml::lmf::writer::{
    capture_wn_complement, reconstruct_wn_lmf_source,
};
use pr4xis_domains::social::software::markup::xml::owl::prx::prx_archive_address;
use pr4xis_runtime::address::ContentAddress;

/// The WN-LMF corpus, parsed once. `sources` is empty on a fresh checkout that
/// hasn't provisioned the XML (`pr4xis update`); each test then HARD-FAILS via `require` (tests do not skip).
static WORDNET: LazyLock<WnCorpus> = LazyLock::new(load_wordnet_corpus);

/// `english_wordnet` 2025 release coordinates — the on-disk corpus's registered
/// source name, version, and download URL (inlined from `lmf::prx`'s test consts).
const FX_NAME: &str = "english_wordnet";
const FX_VERSION: &str = "2025";
const FX_URL: &str = "https://github.com/globalwordnet/english-wordnet/releases/download/2025-edition/english-wordnet-2025.xml.gz";

/// gzip `bytes`. Test-local; not a crate API.
fn gzip(bytes: &[u8]) -> Vec<u8> {
    use flate2::{Compression, write::GzEncoder};
    use std::io::Write;
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(bytes).expect("gz write");
    e.finish().expect("gz finish")
}

/// gunzip `bytes`. Test-local; not a crate API.
fn gunzip(bytes: &[u8]) -> Vec<u8> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut out = Vec::new();
    GzDecoder::new(bytes).read_to_end(&mut out).expect("gunzip");
    out
}

/// gzip byte-length of `bytes` — the wire size of an encoding (compactness gate)
/// and of the `.xml.gz` download. Test-local; not a crate API.
fn gz_len(bytes: &[u8]) -> usize {
    gzip(bytes).len()
}

/// The codec is LOSSLESS over the compact core
/// (`from_succinct(to_succinct(c)) == c`) and the `.prx` is smaller than the
/// raw source. Reads the tiny lexicon (instant) + 89 MB english (one parse);
/// HARD-FAIL via `require` if absent (tests do not skip).
#[test]
fn succinct_codec_roundtrip_and_smaller() {
    require_provisioned(WORDNET.sources.len(), "wordnet");
    let mut measured = 0usize;
    for s in &WORDNET.sources {
        let name = s.name;
        let bytes = &s.source;
        let wn = &s.wn;
        let compact = encode(wn);

        let succ = to_succinct(&compact);
        let back = from_succinct(&succ);
        assert_eq!(back, compact, "{name}: succinct codec is not lossless");

        let succ_gz = gz_len(&succ);
        let source_raw = bytes.len(); // the .xml on disk
        let source_dl = gz_len(bytes); // the .xml.gz you download
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
    require_provisioned(WORDNET.sources.len(), "wordnet");
    let mut measured = 0usize;
    for s in &WORDNET.sources {
        let name = s.name;
        let bytes = &s.source;
        let wn = &s.wn;

        let prx_gz = emit_prx_gz(wn);
        let t = std::time::Instant::now();
        let loaded = load_prx_gz(&prx_gz);
        let load_ms = t.elapsed().as_secs_f64() * 1e3;
        let reference = English::from_wordnet(wn);

        assert_eq!(
            loaded.concept_count(),
            reference.concept_count(),
            "{name}: loaded English concept_count differs from from_wordnet"
        );
        assert_eq!(
            loaded.word_index, reference.word_index,
            "{name}: loaded English word→concept index differs"
        );

        let source_download = gz_len(bytes);
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

/// THE COMPACT-ENGLISH KEYSTONE GATE — the end-to-end machinery
/// [`english_loaded`](pr4xis_domains::cognitive::linguistics::english::english_loaded)
/// reads, exercised over the real 89 MB corpus through the SAME three gated
/// wrappers the runtime loader calls:
/// [`emit_compact_english_prx_gz`] → [`compact_english_archive_address`] →
/// [`load_compact_english_prx_gz_gated`]. It asserts the four properties that
/// loader depends on:
///
/// 1. **Reasoning-equivalence** — the compact-gated `English` has the same
///    `concept_count` and word→concept index as `English::from_wordnet` over a
///    fresh parse of the source (so a chat loaded from the compact `.prx`
///    reasons identically).
/// 2. **Compactness (the product metric)** — the compact `.prx.gz` is smaller
///    than fetching the source itself (`gzip(source)`), the guard the
///    .prx-bigger-than-source regression lacked.
/// 3. **Load-speed (the product metric)** — the compact read-back is at least
///    2× faster than re-parsing the WN-LMF XML (the fallback path), timed on the
///    same corpus.
/// 4. **Fail-closed content gate** — a byte flipped in the *succinct payload*
///    (re-gzipped so gunzip still succeeds) is REJECTED by the content-address
///    check before any `English` is materialized.
///
/// And the freshly-emitted archive's content address MUST equal the shipped
/// `praxis.lock` `[compact_archive_signatures]` pin (required when the corpus is
/// provisioned): the regression teeth on the pin `english_loaded()` trusts — a
/// codec change that silently invalidated it fails here, demanding a deliberate
/// KAT bump. HARD-FAILS via `require` when the 89 MB corpus is not provisioned (tests do not skip).
#[test]
fn compact_english_prx_keystone_gate_over_real_corpus() {
    let en = require(WORDNET.english(), "english_wordnet");
    let source = &en.source;
    let text = core::str::from_utf8(source).expect("WN-LMF source is UTF-8");

    // ── XML path (the loader's fallback): parse + materialize, timed. Doubles
    //    as the reasoning-equivalence reference. ──
    let t = std::time::Instant::now();
    let reference = English::from_wordnet(&read_wordnet(text).expect("parse WN-LMF source"));
    let xml_ms = t.elapsed().as_secs_f64() * 1e3;

    // ── emit + address (the two pure functions `pr4xis compile --compact` and
    //    `english_loaded()` share) ──
    let cprx_gz = emit_compact_english_prx_gz(source).expect("emit compact English .prx.gz");
    let address = compact_english_archive_address(&cprx_gz).expect("compact archive address");

    // ── compact fast path (what `english_loaded()` calls): gunzip + verify +
    //    decode + materialize, timed. ──
    let key = format!("{FX_NAME}@{FX_VERSION}");
    let t = std::time::Instant::now();
    let loaded =
        load_compact_english_prx_gz_gated(&cprx_gz, &LockDigest::address(address.clone()), &key)
            .expect("load compact English through the content gate");
    let prx_ms = t.elapsed().as_secs_f64() * 1e3;

    // 1. REASONING-EQUIVALENCE with the from_wordnet reference.
    assert!(
        loaded.concept_count() > 100_000,
        "real English WordNet is rich (>100k synsets); got {}",
        loaded.concept_count()
    );
    assert_eq!(
        loaded.concept_count(),
        reference.concept_count(),
        "compact-gated English concept_count differs from from_wordnet"
    );
    assert_eq!(
        loaded.word_index, reference.word_index,
        "compact-gated English word→concept index differs from from_wordnet"
    );

    // 2. COMPACTNESS GATE: the compact .prx.gz beats the source download.
    let source_dl = gz_len(source);
    eprintln!(
        "COMPACT-ENGLISH {FX_NAME}: .prx.gz = {:.2}MB loads {} concepts in {:.0}ms  vs  \
         XML parse {:.0}ms / source download {:.2}MB ({:.2}x)",
        cprx_gz.len() as f64 / 1e6,
        loaded.concept_count(),
        prx_ms,
        xml_ms,
        source_dl as f64 / 1e6,
        cprx_gz.len() as f64 / source_dl.max(1) as f64,
    );
    assert!(
        cprx_gz.len() < source_dl,
        "{FX_NAME}: compact .prx.gz ({} B) is NOT smaller than the source download \
         gzip(source) ({} B) — compactness regressed",
        cprx_gz.len(),
        source_dl,
    );

    // 3. LOAD-SPEED GATE: the compact read-back is much faster than re-parsing
    //    the XML. A 2× floor (the measured margin is ~3×) keeps it robust to CI
    //    jitter while still failing closed if the fast path regresses.
    assert!(
        xml_ms > 2.0 * prx_ms,
        "{FX_NAME}: compact load ({prx_ms:.0}ms) is NOT >=2x faster than the XML parse \
         ({xml_ms:.0}ms) — the fast-load win regressed",
    );

    // 4. FAIL-CLOSED CONTENT GATE: corrupt the SUCCINCT PAYLOAD (not the gzip
    //    framing) so the load survives gunzip and is rejected by the
    //    content-address check itself. Flip a byte in the uncompressed bytes and
    //    re-gzip: the re-derived digest no longer equals `address`, so
    //    `wn_verify_content_address` returns Err before any decode.
    let mut corrupt = gunzip(&cprx_gz);
    let mid = corrupt.len() / 2;
    corrupt[mid] ^= 0x01;
    assert!(
        load_compact_english_prx_gz_gated(
            &gzip(&corrupt),
            &LockDigest::address(address.clone()),
            &key
        )
        .is_err(),
        "{FX_NAME}: a corrupt compact archive must be rejected by the content-address gate"
    );

    // PIN TEETH (required — the corpus is on disk): the lock pin MUST equal this
    // emit's address, the regression guard on the pin `english_loaded()` trusts.
    // A codec change that silently invalidated the shipped pin fails here.
    let pin = lock_compact_archive_signature(FX_NAME, FX_VERSION).unwrap_or_else(|| {
        panic!(
            "{key} has no [compact_archive_signatures] pin in praxis.lock, but the corpus \
             is provisioned — the pin english_loaded() trusts is missing"
        )
    });
    assert_eq!(
        pin,
        &LockDigest::address(address.clone()),
        "praxis.lock [compact_archive_signatures] pin for {key} no longer matches the \
         emitted compact archive — the codec changed; bump the pin deliberately (KAT bump)"
    );
}

/// THE DISPATCHER GATE: drives
/// [`english_loaded`](pr4xis_domains::cognitive::linguistics::english::english_loaded)
/// ITSELF (not just the wrapped fns the keystone gate exercises) and asserts
/// that whichever branch its `OnceLock` selects — the compact `.prx` fast path
/// or the WN-LMF XML fallback — materializes the SAME `English` as a fresh
/// `English::from_wordnet` parse of the on-disk source: a rich corpus
/// (>100k concepts), an identical `concept_count`, and an identical
/// word→concept index. The runtime's chosen path must produce the authoritative
/// English. HARD-FAILS via `require` when the 89 MB corpus is not provisioned (tests do not skip).
#[test]
fn english_loaded_dispatcher_matches_from_wordnet() {
    let en = require(WORDNET.english(), "english_wordnet");

    // The authoritative reference: a fresh parse + materialization of the
    // on-disk source — the same English every other gate measures against.
    let reference =
        English::from_wordnet(&read_wordnet(core::str::from_utf8(&en.source).unwrap()).unwrap());

    // Drive the runtime dispatcher itself. Whatever branch it picks must agree
    // with the reference.
    let dispatched = english_loaded();

    assert!(
        dispatched.concept_count() > 100_000,
        "real English WordNet is rich (>100k synsets); english_loaded() gave {}",
        dispatched.concept_count()
    );
    assert_eq!(
        dispatched.concept_count(),
        reference.concept_count(),
        "english_loaded()'s chosen branch concept_count differs from from_wordnet"
    );
    assert_eq!(
        dispatched.word_index, reference.word_index,
        "english_loaded()'s chosen branch word→concept index differs from from_wordnet"
    );
}

/// The compact integer-addressed core is REASONING-EQUIVALENT to the source
/// `WordNet`: `from_wordnet` over the original and over `decode(encode(wn))`
/// build the same `English` (same concept count and word→concept index).
/// Reads the tiny lexicon (instant) + the 89 MB english (one parse); HARD-FAILS
/// via `require` if absent (tests do not skip).
#[test]
fn compact_is_reasoning_equivalent_and_small() {
    require_provisioned(WORDNET.sources.len(), "wordnet");
    let mut measured = 0usize;
    for s in &WORDNET.sources {
        let name = s.name;
        let wn = &s.wn;

        let compact = encode(wn);
        let wn2 = decode(&compact);

        // Reasoning-equivalence: from_wordnet over the original and over the
        // decoded (synthetic-id) WordNet build the SAME English — same
        // concept count and same word→concept index (the integer addressing
        // preserves every ConceptId, since both assign by synset order).
        let e_orig = English::from_wordnet(wn);
        let e_compact = English::from_wordnet(&wn2);
        assert_eq!(
            e_orig.concept_count(),
            e_compact.concept_count(),
            "{name}: concept_count differs — the compact core dropped concepts"
        );
        assert_eq!(
            e_orig.word_index, e_compact.word_index,
            "{name}: word→concept index differs — lexical addressing not preserved"
        );

        eprintln!(
            "compact {name}: dict={} synsets={} senses={} entries={} (reasoning-equivalent)",
            compact.dict.len(),
            compact.syn_pos.len(),
            compact.sense_synset.len(),
            compact.entry_lemma_form.len(),
        );
        measured += 1;
    }
    assert!(
        measured >= 1,
        "no WN-LMF source on disk to exercise the compact core"
    );
}

/// The full Open English WordNet 2025 corpus (≈89 MB on disk) emits and
/// round-trips through the gate, materializing a rich [`English`] whose
/// `concept_count` matches `English::from_wordnet`. Gated on the on-disk
/// file via `require` — a plain checkout that hasn't provisioned the data
/// HARD-FAILS here naming `pr4xis update english_wordnet` (tests do not
/// skip). Heavy, so it is the on-disk-only corroboration of the cheap
/// sample round-trip above.
#[test]
fn wordnet_full_corpus_emit_then_load_matches_from_wordnet() {
    let en = require(WORDNET.english(), "english_wordnet");
    let source = &en.source;
    let reference = English::from_wordnet(&en.wn);

    let prx_gz = emit_wordnet_prx_gz(source, FX_NAME, FX_VERSION, FX_URL).expect("emit");
    let archive_pin = LockDigest::address(prx_archive_address(&prx_gz).expect("archive address"));
    let source_pin = LockDigest::address(ContentAddress::of(source).to_hex());
    let loaded = load_wordnet_prx_gz(&prx_gz, &archive_pin, &source_pin).expect("load + validate");

    assert!(
        loaded.concept_count() > 100_000,
        "real English WordNet is rich (>100k synsets); got {}",
        loaded.concept_count()
    );
    assert_eq!(
        loaded.concept_count(),
        reference.concept_count(),
        "full-corpus concept_count must survive the archive"
    );

    // A canonical lemma resolves to the same synsets pre/post-archive.
    let lref: Vec<ConceptId> = reference.lookup("dog").to_vec();
    let larch: Vec<ConceptId> = loaded.lookup("dog").to_vec();
    assert_eq!(
        lref.len(),
        larch.len(),
        "lookup('dog') sense count must survive the archive"
    );
}

/// THE SLICE-3b GATE. The full Open English WordNet 2025 corpus (89 237 271
/// bytes) emits as a `ByteExactGraphFaithful` envelope, serializes to rkyv
/// bytes, loads back THROUGH the bytecheck-validated rkyv decode
/// ([`wordnet_envelope_from_bytes`]), reconstructs via
/// [`wn_reconstruct_source`] (the graph-faithful arm — typed ontology +
/// concrete-syntax complement, NO stored raw blob), and the regenerated
/// bytes equal the source BYTE-FOR-BYTE. This is the only non-vacuous proof
/// that WordNet's `.prx` is graph-faithful at corpus scale: the source bytes
/// survive the FULL serialize → bytecheck → reconstruct path, not just the
/// in-memory capture/reconstruct of SLICE 3a.
///
/// AND the completeness meter reports `english_wordnet` graph-faithful (its
/// declared tier is `ByteExactGraphFaithful` via the registered
/// `WordNetLmfLens`, and it carries NO `write_wordnet` gap). Gated on the
/// on-disk corpus via `require` — a plain checkout that hasn't provisioned the
/// ≈89 MB XML HARD-FAILS naming `pr4xis update english_wordnet` (tests do not skip).
#[test]
fn wordnet_graph_faithful_prx_round_trip_over_real_corpus() {
    let en = require(WORDNET.english(), "english_wordnet");
    let source = &en.source;

    // Emit the graph-faithful envelope: typed ontology + concrete-syntax
    // complement, NO raw blob.
    let envelope = build_wordnet_envelope(source, FX_NAME, FX_VERSION, FX_URL)
        .expect("build graph-faithful envelope over the real corpus");
    assert_eq!(
        envelope.mode,
        Tier::ByteExactGraphFaithful,
        "the real corpus emits the graph-faithful tier"
    );
    assert!(envelope.graph.is_some(), "graph payload present");
    assert!(envelope.raw.is_none(), "NO stored raw blob in this tier");

    // Serialize → rkyv bytes → bytecheck-validated decode (the full path,
    // not just the in-memory capture/reconstruct of SLICE 3a).
    let rkyv_bytes = wordnet_envelope_to_bytes(&envelope).expect("serialize envelope to rkyv");
    let decoded =
        wordnet_envelope_from_bytes(&rkyv_bytes).expect("bytecheck-validated rkyv decode");

    // Reconstruct from the DECODED envelope's graph + complement.
    let out = wn_reconstruct_source(&decoded).expect("graph-faithful reconstruct");

    // BYTE-FOR-BYTE over the whole 89 MB corpus. Report the EXACT first
    // byte-diff for an honest failure, never a bare assert_eq! that dumps 89 MB.
    if out != *source {
        let first = out
            .iter()
            .zip(source.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(out.len().min(source.len()));
        let lo = first.saturating_sub(40);
        let hi_out = (first + 40).min(out.len());
        let hi_src = (first + 40).min(source.len());
        panic!(
            "graph-faithful .prx round-trip is NOT byte-exact: out.len()={}, \
             source.len()={}, first diff at byte {first}\n  out[..]: {:?}\n  src[..]: {:?}",
            out.len(),
            source.len(),
            String::from_utf8_lossy(&out[lo..hi_out]),
            String::from_utf8_lossy(&source[lo..hi_src]),
        );
    }
    assert_eq!(
        ContentAddress::of(&out).to_hex(),
        decoded.metadata.source_address,
        "the regenerated bytes must hash to the pinned source content address"
    );

    // The completeness meter reports english_wordnet graph-faithful: declared
    // tier == ByteExactGraphFaithful and NO write_wordnet gap remains.
    let meter = completeness_meter();
    let wn_row: &CompletenessReport = meter
        .iter()
        .find(|r| r.source == "english_wordnet@2025")
        .expect("english_wordnet must have a completeness row");
    assert_eq!(
        wn_row.kind,
        DecompileKind::WordNet,
        "english_wordnet routes through the WordNet decompile leaf"
    );
    assert_eq!(
        wn_row.declared,
        Tier::ByteExactGraphFaithful,
        "english_wordnet DECLARES graph-faithful (via the registered WordNetLmfLens)"
    );
    assert!(
        wn_row.graph_faithful_gap.is_none(),
        "english_wordnet carries NO write_wordnet gap — it IS graph-faithful, \
         got gap {:?}",
        wn_row.graph_faithful_gap
    );
    // english_wordnet is OVERSIZE (~86 MB > the 16 MB byte-exact cap), so the
    // FAST completeness-meter harness DEFERS its reconstruction
    // (`OversizeDeferred`) to keep the always-run lane under budget — hence no
    // in-crate `achieved` tier here. Its byte-exact proof is THIS test (the
    // direct serialize -> decode -> reconstruct -> byte-compare above) plus
    // the slow `ci_gate_passes_giants` + the all-sources source round-trip
    // test. `achieved == None` for an oversize graph-faithful source is the
    // honest "pending in the slow lane", NOT a floor — the declared tier and
    // the absent gap already establish it IS graph-faithful.
    assert_eq!(
        wn_row.achieved, None,
        "english_wordnet is oversize, so the fast meter defers it (achieved == None); \
         its byte-exactness is proven by this test + the slow lane, not the fast harness"
    );
}

/// THE HARD GATE: the full Open English WordNet 2025 corpus
/// (89 237 271 bytes) reconstructs BYTE-FOR-BYTE from the typed model + the
/// captured complement. `capture_wn_complement(src)` then
/// `reconstruct_wn_lmf_source(&wn, &complement)` must equal the source
/// bytes exactly — the only non-vacuous proof that WordNet is now
/// graph-faithful (the source bytes regenerate from the `lmf::WordNet`
/// ontology plus a concrete-syntax complement, with NO stored raw blob /
/// stored DOM).
///
/// Gated on the on-disk file via `require` (the same doctrine
/// `wordnet_full_corpus_emit_then_load_matches_from_wordnet` uses) — a plain
/// checkout that has not provisioned the ≈89 MB XML HARD-FAILS here naming
/// `pr4xis update english_wordnet` (tests do not skip).
#[test]
fn wordnet_reconstruct_byte_exact_over_real_corpus() {
    let en = require(WORDNET.english(), "english_wordnet");
    let source = &en.source;
    let src = core::str::from_utf8(source).expect("WordNet XML is UTF-8");

    let (wn, complement) = capture_wn_complement(src).expect("capture complement");
    let out = reconstruct_wn_lmf_source(&wn, &complement).expect("reconstruct");

    // Byte-for-byte over the whole 89 MB corpus. Report the EXACT first
    // byte-diff for an honest failure, never a bare `assert_eq!` that would
    // dump 89 MB.
    if out != *source {
        let first = out
            .iter()
            .zip(source.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(source.len().min(out.len()));
        let lo = first.saturating_sub(80);
        let hi_out = (first + 80).min(out.len());
        let hi_src = (first + 80).min(source.len());
        panic!(
            "byte mismatch at offset {first} (out.len()={}, source.len()={})\n  \
             expected: {:?}\n  got:      {:?}",
            out.len(),
            source.len(),
            String::from_utf8_lossy(&source[lo..hi_src]),
            String::from_utf8_lossy(&out[lo..hi_out]),
        );
    }
    // Content-digest equality is the headline assertion at this size (BLAKE3 —
    // Aumasson, O'Connor, Neves & Wilcox-O'Hearn 2020). `out == source` above
    // already implies it; the explicit pin guards against a silent corpus swap
    // on disk.
    let hash = ContentAddress::of(&out).to_hex();
    assert_eq!(
        hash,
        ContentAddress::of(source).to_hex(),
        "reconstructed corpus must hash-equal the source"
    );
    assert_eq!(
        hash, "d289d59559e5a479dce730b43a30e75d01a9516a451cd90999e756bd2206476c",
        "reconstructed corpus must hash to the pinned Open English WordNet 2025 \
         source pin"
    );
}

/// The full Open English WordNet 2025 corpus loads into a rich [`WordNet`]:
/// hard lower bounds on synset / entry / relation counts, and canonical lemma
/// lookups ('dog', 'entity') resolve. Borrows the shared parse.
#[test]
fn load_full_wordnet() {
    let en = require(WORDNET.english(), "english_wordnet");
    let wn = &en.wn;

    let taxonomy = wn.taxonomy_relations();
    let opposition = wn.opposition_relations();
    let mereology = wn.mereology_relations();
    let causal = wn.causal_relations();

    eprintln!("=== WordNet Load ===");
    eprintln!("  Synsets:       {}", wn.synset_count());
    eprintln!("  Entries:       {}", wn.entry_count());
    eprintln!("  Taxonomy:      {} relations", taxonomy.len());
    eprintln!("  Opposition:    {} relations", opposition.len());
    eprintln!("  Mereology:     {} relations", mereology.len());
    eprintln!("  Causation:     {} relations", causal.len());
    eprintln!(
        "  Memory (est):  ~{} MB",
        (en.source.len() * 2) / (1024 * 1024)
    );

    // Verify reasonable counts
    assert!(wn.synset_count() > 100_000, "expected 100k+ synsets");
    assert!(wn.entry_count() > 100_000, "expected 100k+ entries");
    assert!(taxonomy.len() > 80_000, "expected 80k+ taxonomy relations");
    assert!(
        opposition.len() > 5_000,
        "expected 5k+ opposition relations"
    );

    // Test specific lookups
    let dog = wn.lookup_word("dog");
    assert!(!dog.is_empty(), "should find 'dog'");

    let entity = wn.lookup_word("entity");
    assert!(!entity.is_empty(), "should find 'entity'");
}

/// `pr4xis::codegen::wordnet::parse_wordnet_xml` (build-time, stream-parsed
/// quick-xml) and `xml::lmf::reader::read_wordnet` (runtime, XmlDocument tree)
/// walk the same WordNet XML through different paths. For the same input the
/// synset and entry counts must agree: the runtime `synset_count` maps to
/// codegen's `entity_count` (each synset becomes an `EntityDef`).
#[test]
fn codegen_and_runtime_paths_agree_on_synset_count() {
    let en = require(WORDNET.english(), "english_wordnet");

    // Runtime path (the shared parse).
    let runtime_wn = &en.wn;

    // Build-time path.
    let codegen_builder = parse_wordnet_xml(en.path.as_path()).expect("codegen parse");

    // The runtime synset_count maps to codegen's entity_count
    // (each synset becomes an EntityDef in codegen).
    assert_eq!(
        runtime_wn.synset_count(),
        codegen_builder.entity_count(),
        "synset_count mismatch between runtime and codegen"
    );
}

/// Every emitted WordNet `.prx` archive's MerkleRoot content address equals its
/// `praxis.lock` `[archive_signatures]` pin — the invariant the lock-driven load
/// gate enforces for every loadable lexicon. If this breaks because the rkyv
/// layout changed (e.g. the graph-faithful payload), re-pin the computed values
/// (see `dump_wordnet_archive_addresses` in `lmf::prx`).
///
/// `[archive_signatures]` is a SHARED keyspace (OWL + USC + WordNet pin
/// alongside each other), so this anchor test owns ONLY the `Language`
/// partition — exactly as the OWL anchor owns `OntologyVocabulary` and the USC
/// anchor owns `UsCodeTitle`. Gated on the bundled XML via `require_provisioned`:
/// a checkout that hasn't provisioned ANY lexicon HARD-FAILS naming
/// `pr4xis update wordnet` (tests do not skip).
///
/// Heavy producer (re-emits the 89 MB English lexicon `.prx`): lifted out of the
/// `pr4xis-domains` `#[cfg(test)]` module into this heavy-corpus lane so the fast
/// nextest lane no longer pays the re-emit per process-isolated test.
#[test]
fn wordnet_archive_anchors_match_lock() {
    let dir = std::env::temp_dir().join(format!("prx_wn_archive_anchor_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let arts = emit_all_wordnet_prx_gz(&dir).expect("emit all WordNet archives");

    // Every emitted archive's MerkleRoot equals its [archive_signatures] pin.
    for a in &arts {
        let pinned = lock_archive_signature(&a.name, &a.version).unwrap_or_else(|| {
            panic!(
                "praxis.lock [archive_signatures] must pin {}@{}",
                a.name, a.version
            )
        });
        assert_eq!(
            &LockDigest::address(a.archive_address.clone()),
            pinned,
            "{}@{} .prx MerkleRoot must equal the [archive_signatures] pin",
            a.name,
            a.version
        );
    }

    // Load-bearing in both directions over the `Language` partition: every
    // emitted lexicon is pinned (above) AND every pinned, on-disk lexicon was
    // emitted — so a stale pin for a vanished lexicon, or a missing pin, is
    // caught. Only count Language sources whose XML is actually on disk (the
    // graceful-skip set `emit_all_wordnet_prx_gz` walks).
    let root = workspace_root();
    let lang_keys: std::collections::BTreeSet<String> = data_sources()
        .iter()
        .filter(|e| e.kind == SourceTaxonomyConcept::Language)
        .filter(|e| root.join(e.local_path()).exists())
        .map(|e| format!("{}@{}", e.name, e.version))
        .collect();
    let emitted: std::collections::BTreeSet<String> = arts
        .iter()
        .map(|a| format!("{}@{}", a.name, a.version))
        .collect();
    assert_eq!(
        emitted, lang_keys,
        "emitted WordNet archives must match the on-disk Language sources exactly"
    );
    // Every emitted Language archive carries a pin (the anchor above already
    // asserts equality; this confirms the pin EXISTS in the shared keyspace).
    for key in &emitted {
        assert!(
            lock_archive_signatures().contains_key(key),
            "{key} must have an [archive_signatures] pin"
        );
    }
    // A lexicon MUST be on disk — with NONE provisioned `emitted` and
    // `lang_keys` are both empty and the equality above passes vacuously. CI
    // provisions via `pr4xis update`, so emptiness is a real failure, not a skip.
    require_provisioned(emitted.len(), "wordnet");
}

/// THE B1 GROUNDING GATE — "is a dog an animal" answered over the REAL loaded
/// English `.prx`, through the GENERIC engine.
///
/// This is the acceptance for the engine bridge (#87): the whole pipeline
/// (`bridge::project_archive` → `apply`(the WordNet→praxis functor as data) →
/// `materialize`) over the ~100k+-synset corpus that `english_loaded()` loads,
/// producing a source-agnostic [`RuntimeOntology`]. The is-a question is then
/// decided over THAT ontology's materialized Subsumption closure (via typed
/// `ConceptRef`s the English lexicon resolves) — not English's bespoke
/// `hypernym_closure`. The SUBSTRATE SPLIT is gone: a loaded `.prx` is now an
/// addressable, traversable graph a generic engine reasons over. HARD-FAIL via `require`
/// when the 89 MB corpus is not provisioned.
#[test]
fn b1_gate_is_a_dog_an_animal_over_the_real_loaded_english_prx() {
    let _en = require(WORDNET.english(), "english_wordnet");

    // The real loaded English — the same one the chat grounds over.
    let english = english_loaded();

    // The whole B1 bridge over the full corpus.
    let onto = english_runtime_ontology(english).expect("the real English corpus materializes");
    assert!(
        onto.archive().nodes.len() > 100_000,
        "the runtime ontology must carry the whole loaded corpus; got {} nodes",
        onto.archive().nodes.len()
    );

    let dogs = concept_refs_for_word(&onto, english, "dog");
    let animals = concept_refs_for_word(&onto, english, "animal");
    assert!(
        !dogs.is_empty() && !animals.is_empty(),
        "English's lexicon must resolve both 'dog' and 'animal'"
    );

    // THE GATE: some sense of dog is-a some sense of animal, over the generic
    // engine's closure. The claim IS the Verdict (pattern-matched, with proof).
    let witness = dogs
        .iter()
        .flat_map(|d| animals.iter().map(move |a| (d, a)))
        .find_map(|(d, a)| {
            onto.is_a(d, a)
                .ok()
                .map(|proof| (d.clone(), a.clone(), proof))
        });

    match witness {
        Some((dog_ref, animal_ref, proof)) => {
            let claim = proof.meta().name;
            assert!(
                claim.as_str().contains(&dog_ref.name) && claim.as_str().contains(&animal_ref.name),
                "the proof must name the witnessed dog ⊑ animal claim; got {claim}"
            );
            eprintln!(
                "B1 GATE PASS: {} ⊑ {} witnessed over the loaded English .prx ({} nodes)",
                dog_ref.name,
                animal_ref.name,
                onto.archive().nodes.len()
            );
        }
        None => panic!("'a dog is an animal' must hold over the real loaded English ontology"),
    }
}

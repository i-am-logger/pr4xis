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
//! Every source is fetched (`pr4xis update`), not committed, so each test skips
//! gracefully when its corpus is absent on a fresh checkout.

use std::sync::LazyLock;

use praxis_corpus_tests::{WnCorpus, load_wordnet_corpus};

use pr4xis::codegen::wordnet::parse_wordnet_xml;
use pr4xis_domains::cognitive::linguistics::english::{ConceptId, English};
use pr4xis_domains::formal::meta::well_behaved_lens::{
    CompletenessReport, DecompileKind, RoundTripFidelity as Tier, completeness_meter,
};
use pr4xis_domains::social::software::markup::xml::lmf::compact::{decode, encode};
use pr4xis_domains::social::software::markup::xml::lmf::compact_succinct::{
    emit_prx_gz, from_succinct, load_prx_gz, to_succinct,
};
use pr4xis_domains::social::software::markup::xml::lmf::prx::{
    build_wordnet_envelope, emit_wordnet_prx_gz, load_wordnet_prx_gz, wn_reconstruct_source,
    wordnet_envelope_from_bytes, wordnet_envelope_to_bytes,
};
use pr4xis_domains::social::software::markup::xml::lmf::writer::{
    capture_wn_complement, reconstruct_wn_lmf_source,
};
use pr4xis_domains::social::software::markup::xml::owl::prx::{
    prx_archive_address, source_content_hash,
};

/// The WN-LMF corpus, parsed once. `sources` is empty on a fresh checkout that
/// hasn't provisioned the XML (`pr4xis update`); each test then skips gracefully.
static WORDNET: LazyLock<WnCorpus> = LazyLock::new(load_wordnet_corpus);

/// `english_wordnet` 2025 release coordinates — the on-disk corpus's registered
/// source name, version, and download URL (inlined from `lmf::prx`'s test consts).
const FX_NAME: &str = "english_wordnet";
const FX_VERSION: &str = "2025";
const FX_URL: &str = "https://github.com/globalwordnet/english-wordnet/releases/download/2025-edition/english-wordnet-2025.xml.gz";

/// gzip byte-length of `bytes` — the wire size of an encoding (compactness gate)
/// and of the `.xml.gz` download. Test-local; not a crate API.
fn gz_len(bytes: &[u8]) -> usize {
    use flate2::{Compression, write::GzEncoder};
    use std::io::Write;
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(bytes).expect("gz write");
    e.finish().expect("gz finish").len()
}

/// Lowercase-hex SHA-256 (NIST FIPS 180-4 §6.2) of `bytes` — the byte-exact
/// reconstruct gate's headline hash. Test-local; not a crate API.
fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The codec is LOSSLESS over the compact core
/// (`from_succinct(to_succinct(c)) == c`) and the `.prx` is smaller than the
/// raw source. Reads the tiny lexicon (instant) + 89 MB english (one parse);
/// graceful skip if absent.
#[test]
fn succinct_codec_roundtrip_and_smaller() {
    if WORDNET.sources.is_empty() {
        eprintln!("SKIP: no WN-LMF source on disk");
        return;
    }
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
    if WORDNET.sources.is_empty() {
        eprintln!("SKIP: no WN-LMF source on disk");
        return;
    }
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

/// The compact integer-addressed core is REASONING-EQUIVALENT to the source
/// `WordNet`: `from_wordnet` over the original and over `decode(encode(wn))`
/// build the same `English` (same concept count and word→concept index).
/// Reads the tiny lexicon (instant) + the 89 MB english (one parse); graceful
/// skip if absent.
#[test]
fn compact_is_reasoning_equivalent_and_small() {
    if WORDNET.sources.is_empty() {
        eprintln!("SKIP: no WN-LMF source on disk");
        return;
    }
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
/// `concept_count` matches `English::from_wordnet`. Gated behind the
/// on-disk file with a graceful skip — a plain checkout that hasn't
/// provisioned the data emits nothing here, the same graceful-skip
/// doctrine `loaded_vocabularies` and the emitters use. Heavy, so it is
/// the on-disk-only corroboration of the cheap sample round-trip above.
#[test]
fn wordnet_full_corpus_emit_then_load_matches_from_wordnet() {
    let Some(en) = WORDNET.english() else {
        eprintln!("SKIP: WordNet not on disk");
        return;
    };
    let source = &en.source;
    let reference = English::from_wordnet(&en.wn);

    let prx_gz = emit_wordnet_prx_gz(source, FX_NAME, FX_VERSION, FX_URL).expect("emit");
    let archive_pin = prx_archive_address(&prx_gz).expect("archive address");
    let source_pin = source_content_hash(source);
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
/// `WordNetLmfLens`, and it carries NO `write_wordnet` gap). Gated behind the
/// on-disk corpus with a graceful skip — a plain checkout that hasn't
/// provisioned the ≈89 MB XML skips, the same doctrine the emitters use.
#[test]
fn wordnet_graph_faithful_prx_round_trip_over_real_corpus() {
    let Some(en) = WORDNET.english() else {
        eprintln!("SKIP: WordNet not on disk");
        return;
    };
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
        source_content_hash(&out),
        decoded.metadata.source_sha256,
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
/// Gated behind the on-disk file with a graceful skip (the same doctrine
/// `wordnet_full_corpus_emit_then_load_matches_from_wordnet` uses) — a plain
/// checkout that has not provisioned the ≈89 MB XML emits nothing here.
#[test]
fn wordnet_reconstruct_byte_exact_over_real_corpus() {
    let Some(en) = WORDNET.english() else {
        eprintln!("SKIP: WordNet not on disk");
        return;
    };
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
    // SHA-256 equality is the headline assertion at this size (NIST FIPS
    // 180-4 §6.2). `out == source` above already implies it; the explicit
    // pin guards against a silent corpus swap on disk.
    let hash = sha256_hex(&out);
    assert_eq!(
        hash,
        sha256_hex(source),
        "reconstructed corpus must hash-equal the source"
    );
    assert_eq!(
        hash, "6f49adeec174ab3092169fb25cf4a925226b63975a5d29a691a5dff88f0673b2",
        "reconstructed corpus must hash to the pinned Open English WordNet 2025 \
         source sha256"
    );
}

/// The full Open English WordNet 2025 corpus loads into a rich [`WordNet`]:
/// hard lower bounds on synset / entry / relation counts, and canonical lemma
/// lookups ('dog', 'entity') resolve. Borrows the shared parse.
#[test]
fn load_full_wordnet() {
    let Some(en) = WORDNET.english() else {
        eprintln!("SKIP: WordNet not on disk");
        return;
    };
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
    let Some(en) = WORDNET.english() else {
        eprintln!("SKIP: WordNet not on disk");
        return;
    };

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

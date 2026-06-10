//! `.prx` compactness-breakdown MEASUREMENT — the go/no-go bench for the
//! generative-serialization redesign (retro 2026-06-06).
//!
//! For each provisioned USC title, split the graph-faithful `.prx` into its two
//! layers and gzip each:
//!
//! - SEMANTIC graph = `UsCodeTitle` (the meaning — the master ~compact path).
//! - COMPLEMENT = `UslmSyntaxComplement` (the per-element concrete-syntax residue
//!   — the byte-exact tax that bloated the artifact).
//!
//! It then reports the per-element residue POPULATION (whitespace / attribute-
//! order / child-order entries) — the exception-rate proxy. The hypothesis the
//! bidi-transform literature endorses: the semantic graph is compact
//! (< gzip(source)) and the complement is dominated by REGULAR per-element
//! residue (whitespace = f(depth), attr order = schema order) that a generative
//! serialization ontology regenerates — so moving it out reclaims the size.
//!
//! Was a `#[cfg(test)]` measurement in `uslm::corpus::prx`; it carried no gate
//! assertion (only non-vacuity), so it is a criterion bench, not a test. USC
//! titles are externally provisioned (`pr4xis update`); a title not on disk is
//! skipped. Run with `cargo bench -p pr4xis-domains --features prx`.

use std::io::Write as _;

use criterion::{Criterion, criterion_group, criterion_main};

use pr4xis_domains::social::software::markup::xml::uslm::corpus::prx::{
    build_usc_envelope, usc_envelope_to_bytes,
};

const T1_URL: &str =
    "https://uscode.house.gov/download/releasepoints/us/pl/119/90/xml_usc01@119-90.zip";

/// Workspace root — grandparent of this crate's manifest dir (`crates/domains`).
fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// gzip byte-length of `bytes`.
fn gz_len(bytes: &[u8]) -> usize {
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(bytes).expect("gz write");
    e.finish().expect("gz finish").len()
}

/// Measure + report the two-layer compactness breakdown for each on-disk title.
fn report_compactness_breakdown() {
    // Small → moderate titles only (skip the >20 MB giants to stay resource-light).
    let titles = ["usc_title_1", "usc_title_28", "usc_title_5"];
    let mut measured = 0usize;
    for name in titles {
        let path = workspace_root().join(format!(
            "crates/domains/data/legal/uscode/{name}/{name}-pl-119-90.xml"
        ));
        let Ok(source) = std::fs::read(&path) else {
            continue;
        };
        let envelope = build_usc_envelope(&source, name, "pl-119-90", T1_URL, None)
            .unwrap_or_else(|e| panic!("build {name}: {e}"));
        let g = envelope
            .graph
            .as_ref()
            .unwrap_or_else(|| panic!("{name} must be graph-faithful for this measurement"));

        let source_gz = gz_len(&source);
        let total_gz = gz_len(&usc_envelope_to_bytes(&envelope).expect("envelope bytes"));
        let semantic_gz =
            gz_len(&rkyv::to_bytes::<rkyv::rancor::Error>(&g.title).expect("rkyv title"));
        let complement_gz =
            gz_len(&rkyv::to_bytes::<rkyv::rancor::Error>(&g.complement).expect("rkyv complement"));
        let ws = g.complement.regenerated.content_whitespace.len();
        let ao = g.complement.regenerated.attribute_overrides.len();
        let co = g.complement.regenerated.child_order.len();

        eprintln!(
            "COMPACTNESS {name}: source.xml={:.2}MB | gzip(source)={:.2}MB \
             total.prx.gz={:.2}MB || semantic(graph)={:.2}MB  complement(residue)={:.2}MB \
             || residue entries: whitespace={ws} attr_overrides={ao} child_order={co} sections={} \
             || semantic<gzip(source)? {}",
            source.len() as f64 / 1e6,
            source_gz as f64 / 1e6,
            total_gz as f64 / 1e6,
            semantic_gz as f64 / 1e6,
            complement_gz as f64 / 1e6,
            envelope.aux.len(),
            semantic_gz < source_gz,
        );
        measured += 1;
    }
    if measured == 0 {
        eprintln!(
            "COMPACTNESS: no USC title provisioned on disk (pr4xis update) — nothing to measure"
        );
    }
}

fn bench_prx_compactness(c: &mut Criterion) {
    // Emit the breakdown report once up front (the measurement this bench exists
    // for), then register a timed measurement of the build+serialize pipeline over
    // the smallest provisioned title so criterion has a benchmark to run.
    report_compactness_breakdown();

    let path = workspace_root()
        .join("crates/domains/data/legal/uscode/usc_title_1/usc_title_1-pl-119-90.xml");
    let Ok(source) = std::fs::read(&path) else {
        // Not provisioned — register no benchmark (criterion runs the empty group
        // cleanly; the report above already noted the skip).
        return;
    };
    c.bench_function("usc_title_1_build_and_serialize_prx", |b| {
        b.iter(|| {
            let envelope = build_usc_envelope(&source, "usc_title_1", "pl-119-90", T1_URL, None)
                .expect("build envelope");
            std::hint::black_box(usc_envelope_to_bytes(&envelope).expect("envelope bytes"));
        });
    });
}

criterion_group!(benches, bench_prx_compactness);
criterion_main!(benches);

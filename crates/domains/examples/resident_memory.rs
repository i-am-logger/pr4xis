//! Resident-memory measurement gate for the runtime reasoning substrate.
//!
//! A COMMITTED, reusable measurement (the praxis-way "measured gate": the number
//! is re-derivable by running this bin, not asserted in a flaky `#[test]`). It
//! reads `VmRSS` (resident set) and `VmHWM` (peak resident) from
//! `/proc/self/status` at each stage of building the reasoning substrate and
//! prints them, so the footprint of a representation change (e.g. an eager vs.
//! lazy transitive closure) is a number anyone can reproduce.
//!
//! Run: `cargo run --release -p pr4xis-domains --example resident_memory --features prx`
//!
//! Stages (each prints RSS, the delta since the prior stage, and peak):
//!
//! 1. baseline — process start, nothing loaded.
//! 2. synthetic taxonomy archive — an English-scale (~88.5k concept) shallow
//!    hypernym tree built as a raw `Archive`.
//! 3. materialize — the `RuntimeOntology` + its reachability engine. THIS is the
//!    stage the eager→lazy closure change moves: the eager form folds the whole
//!    `O(V·depth)` transitive closure here; the lazy form keeps only the
//!    generators and answers on demand. The stage-3 delta IS the closure cost.
//! 4. a handful of reachability queries — what a running chat actually asks.
//! 5. embedded English (WordNet) — the production reasoner substrate; skipped
//!    with a notice if the corpus is not on disk (a fresh checkout has no
//!    WordNet — run `pr4xis update`).
//! 6. english_runtime_ontology — English AS a `RuntimeOntology` (the closure path
//!    for the 107,519-concept corpus).
//! 7. ComposedReasoner — English composed as one `LexicalReasoner`.
//!
//! `/proc/self/status` is Linux-only; on a non-Linux host the reader reports the
//! stage as unavailable rather than failing.

use std::panic;
use std::rc::Rc;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::english::bridge::english_runtime_ontology;
use pr4xis_domains::cognitive::linguistics::english::english_load_owned;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::definition::{Definition, EdgeTarget};
use pr4xis_runtime::ontology::{materialize, subsumption_kind};

/// Branching factor of the synthetic hypernym tree — a shallow, wide taxonomy,
/// the shape of WordNet's hypernym forest (few parents, many siblings).
const SYNTH_BRANCHING: usize = 3;
/// Depth of the synthetic hypernym tree. `SYNTH_BRANCHING = 3`, `SYNTH_DEPTH =
/// 10` yields `(3^11 - 1) / 2 = 88,573` concepts — the same order of magnitude
/// as the review's measured English corpus (107,519 concepts), with a
/// comparable (single-digit-to-low-teens) hypernym depth, so the closure cost
/// this probe measures is representative of the real one.
const SYNTH_DEPTH: usize = 10;

/// A resident-memory reading, in kibibytes, from `/proc/self/status`.
#[derive(Clone, Copy)]
struct MemReading {
    /// `VmRSS` — current resident set size.
    rss_kib: Option<u64>,
    /// `VmHWM` — peak ("high water mark") resident set size.
    hwm_kib: Option<u64>,
}

/// Read `VmRSS` + `VmHWM` from `/proc/self/status`. Returns `None` fields on a
/// host without procfs (non-Linux) rather than failing the whole run.
fn read_memory() -> MemReading {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let field = |key: &str| -> Option<u64> {
        status
            .lines()
            .find(|line| line.starts_with(key))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|kib| kib.parse::<u64>().ok())
    };
    MemReading {
        rss_kib: field("VmRSS:"),
        hwm_kib: field("VmHWM:"),
    }
}

/// Format a kib reading as MiB (or `n/a` when the host has no procfs).
fn mib(kib: Option<u64>) -> String {
    match kib {
        Some(k) => format!("{:.1} MiB", k as f64 / 1024.0),
        None => "n/a".to_string(),
    }
}

/// Print one stage's RSS + HWM and the RSS delta since the previous reading.
fn report(stage: &str, now: MemReading, prev: MemReading) {
    let delta = match (now.rss_kib, prev.rss_kib) {
        (Some(n), Some(p)) => format!("{:+.1} MiB", (n as f64 - p as f64) / 1024.0),
        _ => "n/a".to_string(),
    };
    println!(
        "{:<34} RSS {:>12}   (Δ {:>12})   peak {:>12}",
        stage,
        mib(now.rss_kib),
        delta,
        mib(now.hwm_kib),
    );
}

/// Build a shallow, wide hypernym tree as a raw `Archive`: concept `c{i}` (for
/// `i > 0`) has one `Subsumption` (is-a) edge to its parent `c{(i-1)/B}`, the
/// child→parent orientation of a WordNet hypernym. `reachable_from(c{i})` is then
/// `c{i}`'s ancestor chain — small per vertex, but the whole transitive closure
/// is `O(V · depth)` pairs, which is what the eager fold materializes and the
/// lazy engine does not.
fn synthetic_taxonomy(branching: usize, depth: usize) -> Archive {
    // Node count of a complete `branching`-ary tree of the given depth.
    let mut count = 1usize;
    let mut level = 1usize;
    for _ in 0..depth {
        level *= branching;
        count += level;
    }
    let mut nodes: Vec<Definition> = Vec::with_capacity(count);
    for i in 0..count {
        let edges = if i == 0 {
            Vec::new()
        } else {
            let parent = (i - 1) / branching;
            vec![(
                "Subsumption".to_string(),
                EdgeTarget::Local(format!("c{parent}")),
            )]
        };
        nodes.push(Definition {
            kind: "Concept".to_string(),
            name: format!("c{i}"),
            edges,
            axioms: Vec::new(),
            lexical: None,
        });
    }
    Archive {
        nodes,
        connections: Vec::new(),
    }
}

/// Run `f`, suppressing the default panic print, and return `None` if it panics
/// (e.g. `english_load_owned` when no WordNet corpus is on disk). Lets the bin
/// report a missing corpus as a notice instead of aborting.
fn try_or_none<T>(f: impl FnOnce() -> T + panic::UnwindSafe) -> Option<T> {
    let prev = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let out = panic::catch_unwind(f).ok();
    panic::set_hook(prev);
    out
}

fn main() {
    println!("pr4xis runtime — resident-memory gate (VmRSS / VmHWM from /proc/self/status)\n");

    let baseline = read_memory();
    report("1. baseline", baseline, baseline);

    // 2. Build the synthetic English-scale archive (the raw loaded form).
    let archive = synthetic_taxonomy(SYNTH_BRANCHING, SYNTH_DEPTH);
    let node_count = archive.nodes.len();
    let after_archive = read_memory();
    report(
        &format!("2. archive ({node_count} concepts)"),
        after_archive,
        baseline,
    );

    // 3. Materialize — build the RuntimeOntology + its reachability engine. The
    //    eager closure folds the whole transitive image HERE; the lazy engine
    //    keeps only the generators. The Δ at this stage is the closure footprint.
    let onto = materialize(archive, OntologyName::new_static("SyntheticTaxonomy"))
        .expect("the synthetic taxonomy is referentially closed and materializes");
    let after_materialize = read_memory();
    report(
        "3. materialize (closure engine)",
        after_materialize,
        after_archive,
    );

    // 4. A handful of reachability queries — the deepest leaf's ancestors, an
    //    is-a decision, and a lattice meet — what a running chat actually asks.
    //    Under the lazy engine these are the ONLY vertices whose image is
    //    computed + memoized.
    let deepest = onto.concept(format!("c{}", node_count - 1));
    let root = onto.concept("c0");
    let ancestors = onto.reachable_from(&deepest, subsumption_kind());
    let is_a = onto.is_a(&deepest, &root).is_ok();
    let meet = onto
        .closure()
        .subsumption_meet(&deepest, &onto.concept(format!("c{}", node_count - 2)));
    let after_queries = read_memory();
    report("4. after 3 queries", after_queries, after_materialize);
    println!(
        "     (deepest leaf has {} ancestors; is-a root = {is_a}; meet = {:?})",
        ancestors.len(),
        meet.map(|m| m.name),
    );

    println!();

    // 5. Embedded English (WordNet) — the production reasoner substrate. Skipped
    //    with a notice if the corpus is not on disk.
    let before_english = read_memory();
    let Some(english) = try_or_none(english_load_owned) else {
        println!(
            "5. embedded English            SKIPPED — WordNet corpus not on disk \
             (run `pr4xis update`; cite the review's prior 336 MB measurement)."
        );
        return;
    };
    // The `ComposedReasoner` now BORROWS a `&'static English` (single-substrate-
    // instance ownership). This measurement harness is a one-shot process, so the
    // one English we just loaded is promoted to `'static` in place — the reasoner
    // references it rather than owning a second ~73 MiB copy (which is the very
    // saving this profile demonstrates).
    let english: &'static English = Box::leak(Box::new(english));
    let after_english = read_memory();
    report(
        &format!("5. embedded English ({} concepts)", english.concept_count()),
        after_english,
        before_english,
    );

    // 6. English AS a RuntimeOntology — the closure path for the full corpus.
    let english_onto =
        english_runtime_ontology(english).expect("English materializes into a RuntimeOntology");
    let after_english_onto = read_memory();
    report(
        "6. english_runtime_ontology",
        after_english_onto,
        after_english,
    );

    // 7. ComposedReasoner — English composed as one LexicalReasoner.
    let _reasoner = ComposedReasoner::new(english, vec![Rc::new(english_onto)]);
    let after_reasoner = read_memory();
    report("7. ComposedReasoner", after_reasoner, after_english_onto);

    println!();
    report("TOTAL (since baseline)", after_reasoner, baseline);
}

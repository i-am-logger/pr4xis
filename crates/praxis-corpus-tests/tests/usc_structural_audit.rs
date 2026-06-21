//! Phase-1 USLM structural audit over EVERY registered U.S. Code title —
//! the informational raw-vs-typed element census, lifted out of the
//! `pr4xis-domains` `#[cfg(test)]` modules.
//!
//! Reads each registered title's raw USLM XML (113 MB for Title 42) and
//! audits which element kinds the typed view drops. That is giant-parse
//! work, so it lives in the heavy-corpus lane (`cargo test`, one process,
//! no strict per-test cap) rather than in-crate under nextest, where the
//! 4-core CI runner blows the 30 s ci-profile cap — the same parse-once
//! doctrine every other giant-corpus test here follows.
//!
//! A title absent on disk is left out of THIS run's census, but the test
//! HARD-FAILS if NONE are provisioned (`pr4xis update`) — it cannot audit
//! nothing. The test passes regardless of gap size — the audit is
//! informational; the per-title report (run with `--nocapture`) is what a
//! reviewer reads to decide which dropped element kinds warrant lifting
//! into the typed view. The title set derives from
//! `data_sources()` filtered to `SourceTaxonomyConcept::UsCodeTitle`, so
//! additions to `praxis.toml` flow through automatically.

use pr4xis_domains::applied::data_provisioning::registry::data_sources;
use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis_domains::social::software::markup::xml::uslm::lens::structural_audit::{
    GapRow, audit_structural_content, render_audit,
};
use praxis_corpus_tests::{require_provisioned, workspace_root};

#[test]
fn phase1_structural_audit_across_registered_usc_titles() {
    let workspace_root = workspace_root();

    let mut audited = 0usize;
    let mut total_dropped = 0i64;
    let mut report_lines: Vec<String> = Vec::new();
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
            continue;
        }
        let path = workspace_root.join(entry.local_path());
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!(
                "{}@{}: {} not on disk — not audited this run",
                entry.name,
                entry.version,
                path.display()
            );
            continue;
        };
        let label = format!("{}@{}", entry.name, entry.version);
        let audit = audit_structural_content(&bytes)
            .unwrap_or_else(|e| panic!("audit failed for {label}: {e}"));
        total_dropped += audit.total_dropped();
        audited += 1;

        // Compact per-title summary line.
        report_lines.push(format!(
            "  {label}: raw_total={}  typed_total={}  total_dropped={}  raw_distinct={}  typed_distinct={}",
            audit.raw.total(),
            audit.typed.total(),
            audit.total_dropped(),
            audit.raw.distinct(),
            audit.typed.distinct(),
        ));

        // Per-element dropped-only diff (raw > typed).
        let mut drops: Vec<&GapRow> = audit.dropped_elements().collect();
        drops.sort_by_key(|g| std::cmp::Reverse(g.gap));
        for d in drops.iter().take(20) {
            let ns = d.namespace.as_deref().unwrap_or("(none)");
            report_lines.push(format!(
                "      drop {{{ns}}}{}  raw={}  typed={}  gap={:+}",
                d.local, d.raw, d.typed, d.gap,
            ));
        }
        // Emit the full per-title report to stderr — useful when run
        // with --nocapture.
        eprintln!("{}", render_audit(&label, &audit));
    }
    require_provisioned(audited, "usc");
    eprintln!("=== Phase-1 USLM structural audit ===");
    eprintln!("titles audited: {audited}");
    eprintln!("aggregate total_dropped: {total_dropped}");
    for line in report_lines {
        eprintln!("{line}");
    }
}

//! Corpus-wide audit of the praxis XML 1.0 parser against the full
//! **W3C XML Conformance Test Suite** (XMLConf) — the deferred
//! follow-up to the in-repo conformance canon ([`super::conformance`]).
//!
//! Per `feedback_corpus_wide_audit_on_load`, every new
//! `[sources.X]` entry ships with an at-test-time audit that walks
//! every record through the understanding pipeline and surfaces any
//! unresolved item. The XMLConf entry
//! (`xmlconf_xml_test_suite@2008-08-27` in `praxis.toml`) is no
//! exception — this module is its audit.
//!
//! ## Pre-condition
//!
//! The audit reads the **extracted** XMLConf tree at
//! `crates/domains/data/markup-schemas/xmlconf/xmlconf/`, not the
//! .tar.gz directly: tar / gzip are intentionally not pulled into
//! pr4xis-domains' dependency surface. Extract once via
//! `pr4xis update` or by hand
//! (`tar -xzf .../xmlconf_xml_test_suite-2008-08-27.tar.gz`).
//! When the extracted tree is absent the audit soft-passes (mirroring
//! [`crate::formal::meta::xsd::xsts_audit::XstsAuditOutcome::ExtractedTreeAbsent`]).
//!
//! ## Categories
//!
//! Each XMLConf `<TEST>` entry declares one of four `TYPE` values
//! per the testcases.dtd shipped with the suite:
//!
//! - `valid` — well-formed AND DTD-valid XML.
//! - `invalid` — well-formed but NOT DTD-valid.
//! - `not-wf` — not well-formed (XML 1.0 §2.1 violation).
//! - `error` — optional-error case (parser may accept or reject).
//!
//! The praxis XML 1.0 parser is a well-formedness parser (no DTD
//! validation), so the audit's expectations are:
//!
//! - `valid` / `invalid` cases: well-formed → must parse-ok.
//! - `not-wf` cases: must be rejected.
//! - `error` cases: either outcome is acceptable per the spec.
//!
//! ## Citation
//!
//! - **W3C XML Test Suite Working Group**, *XML Conformance Test
//!   Suite*, <https://www.w3.org/XML/Test/>.
//! - **Bray, T., Paoli, J., Sperberg-McQueen, C. M., Maler, E. &
//!   Yergeau, F.** (2008) *Extensible Markup Language (XML) 1.0
//!   (Fifth Edition)*, W3C Recommendation 26 November 2008 — §2.1
//!   well-formed XML, §2.8 prolog and document type declaration.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use std::sync::OnceLock;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::grammar::parse_document;
use crate::applied::data_provisioning::registry::by_name_version;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// One TEST entry from XMLConf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlConfCase {
    /// Absolute path to the test's XML document.
    pub doc_path: std::path::PathBuf,
    /// The TYPE attribute's value.
    pub case_type: XmlConfType,
}

/// XMLConf `TYPE` enumeration (testcases.dtd).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlConfType {
    /// Well-formed AND DTD-valid (XML 1.0 §2.8).
    Valid,
    /// Well-formed but not DTD-valid.
    Invalid,
    /// Not well-formed (XML 1.0 §2.1 violation).
    NotWf,
    /// Optional-error — a parser MAY accept or reject; spec is
    /// neutral on the outcome.
    Error,
}

/// The full loaded W3C XML Conformance Test Suite, exposed as
/// queryable praxis data — every TEST entry materialised in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlConfCorpus {
    /// Every loaded TEST case, in document order across the six
    /// contributor sub-manifests.
    pub cases: Vec<XmlConfCase>,
}

impl XmlConfCorpus {
    /// All cases.
    #[must_use]
    pub fn cases(&self) -> &[XmlConfCase] {
        &self.cases
    }

    /// Cases of a given type.
    pub fn cases_of_type(&self, t: XmlConfType) -> impl Iterator<Item = &XmlConfCase> {
        self.cases.iter().filter(move |c| c.case_type == t)
    }

    /// Total case count, as a dimensionless [`Quantity`] (`unit::UNITLESS`)
    /// -- a count, not a physical quantity.
    #[must_use]
    pub fn len(&self) -> Quantity {
        Quantity::from_unit(self.cases.len() as f64, &unit::UNITLESS)
    }

    /// True iff zero cases loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }
}

/// Cached loaded XMLConf corpus, or `None` when the extracted tree
/// isn't on disk yet. First call walks every sub-manifest, parses
/// the `<TEST>` entries, and caches a `&'static XmlConfCorpus`.
pub fn loaded_xmlconf() -> Option<&'static XmlConfCorpus> {
    static CACHE: OnceLock<XmlConfCorpus> = OnceLock::new();
    if let Some(c) = CACHE.get() {
        return Some(c);
    }
    let extracted = resolve_extracted_tree_path().ok().flatten()?;
    let cases = collect_cases(&extracted);
    if cases.is_empty() {
        return None;
    }
    Some(CACHE.get_or_init(|| XmlConfCorpus { cases }))
}

/// Resolve the praxis-registry archive path and derive the extracted
/// tree's directory (`<archive-dir>/xmlconf/`) next to it.
fn resolve_extracted_tree_path() -> Result<Option<std::path::PathBuf>, ()> {
    let entry = by_name_version("xmlconf_xml_test_suite", "2008-08-27").ok_or(())?;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path_str = entry.local_path();
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .and_then(std::path::Path::parent);
    let abs_archive = workspace_root
        .map(|root| root.join(&path_str))
        .unwrap_or_else(|| std::path::PathBuf::from(&path_str));
    let extracted = abs_archive.parent().map(|d| d.join("xmlconf")).ok_or(())?;
    Ok(if extracted.is_dir() {
        Some(extracted)
    } else {
        None
    })
}

/// Walk every `.xml` sub-manifest under `base` by **parsing the
/// manifest XML through the praxis XML 1.0 parser** and walking the
/// typed [`XmlDocument`] tree (no substring or attribute
/// hand-scanning). Each `<TEST>` element contributes one
/// [`XmlConfCase`] iff it has `URI` and `TYPE` attributes per the
/// XMLConf testcases.dtd shipped with the suite.
fn collect_cases(base: &std::path::Path) -> Vec<XmlConfCase> {
    let mut cases = Vec::new();
    for meta in list_meta_files(base) {
        let Ok(bytes) = std::fs::read(&meta) else {
            continue;
        };
        let Ok(doc) = parse_document(&bytes) else {
            continue;
        };
        let meta_dir = meta.parent().unwrap_or(std::path::Path::new("."));
        walk_test_elements(&doc.root, meta_dir, &mut cases);
    }
    cases
}

/// Recursive walker over the typed [`XmlDocument`] tree. Every
/// `<TEST>` element contributes one [`XmlConfCase`] iff it has a
/// recognised `TYPE` and a `URI` (per the XMLConf testcases.dtd
/// shipped with the suite).
fn walk_test_elements(
    element: &crate::social::software::markup::xml::ontology::XmlElement,
    meta_dir: &std::path::Path,
    out: &mut Vec<XmlConfCase>,
) {
    if element.name.local == "TEST"
        && let Some(case) = case_from_test(element, meta_dir)
    {
        out.push(case);
    }
    for child in &element.children {
        if let crate::social::software::markup::xml::ontology::XmlNode::Element(el) = child {
            walk_test_elements(el, meta_dir, out);
        }
    }
}

/// Project one `<TEST>` element to an [`XmlConfCase`], or `None`
/// when the test is out of scope for the praxis parser:
///
/// 1. Missing/unrecognised `URI` or `TYPE` attribute.
/// 2. `EDITION` attribute lists XML 1.0 editions but excludes the
///    Fifth Edition the praxis parser implements. The §B Character
///    Classes (Letter / BaseChar / Ideographic / CombiningChar /
///    Digit / Extender — productions \[85\]–\[89\]) were removed in 5e
///    in favour of Unicode-range NameStartChar / NameChar; the
///    4e-era `not-wf` tests of characters outside the Letter class
///    (`EDITION="1 2 3 4"`) are well-formed in 5e and must be
///    excluded.
/// 3. `not-wf` case with `ENTITIES != "none"` — the malformedness
///    is in an external entity the parser would need to fetch to
///    surface it, and praxis is intentionally a non-validating
///    single-document parser. The XMLConf testcases.dtd itself
///    documents this:
///
///    > The type of (external) ENTITIES required affect the
///    > results permitted for certain types of nonvalidating
///    > parsers. In some cases, errors (even well-formedness
///    > errors) can't be seen [without loading external entities].
///
///    `valid` and `invalid` cases with external entities stay in
///    the audit because a well-formed document is still well-
///    formed without resolving external references — the praxis
///    parser may legitimately accept them.
fn case_from_test(
    test: &crate::social::software::markup::xml::ontology::XmlElement,
    meta_dir: &std::path::Path,
) -> Option<XmlConfCase> {
    let uri = attr_value(test, "URI")?;
    let case_type = match attr_value(test, "TYPE")?.as_str() {
        "valid" => XmlConfType::Valid,
        "invalid" => XmlConfType::Invalid,
        "not-wf" => XmlConfType::NotWf,
        "error" => XmlConfType::Error,
        _ => return None,
    };
    if let Some(edition) = attr_value(test, "EDITION") {
        let applies_to_5e = edition.split_whitespace().any(|e| e == "5");
        if !applies_to_5e {
            return None;
        }
    }
    // XMLConf testcases.dtd ATTLIST: `VERSION CDATA #IMPLIED`.
    // When present, the test asserts behaviour of an XML 1.1
    // processor — a separate W3C Recommendation (Bray et al.
    // 2008 *Extensible Markup Language (XML) 1.1*). The praxis
    // parser implements XML 1.0 only. eduni/errata-2e/E50 is the
    // canonical case: `VERSION="1.1"`, declares `version="1.1"`,
    // and exercises ISO-8859-1 transcoding through an XML 1.1
    // processor's encoding-detection path.
    if let Some(version) = attr_value(test, "VERSION")
        && version != "1.0"
    {
        return None;
    }
    if case_type == XmlConfType::NotWf
        && let Some(entities) = attr_value(test, "ENTITIES")
        && entities != "none"
    {
        return None;
    }
    let doc_path = meta_dir.join(&uri);
    let path_str = doc_path.to_string_lossy();
    // The IBM `xml-1.1/` subtree and the W3C edu-ni `xml-1.1/`
    // subtree are XML 1.1 cases — a separate W3C Recommendation
    // (Bray et al. 2006/2008 *XML 1.1 (Second Edition)*). Our
    // parser implements XML 1.0 Fifth Edition; per §2.8 spec note
    // a 1.0 processor "will accept 1.x documents provided they do
    // not use any non-1.0 features" — the entire point of these
    // tests is to exercise 1.1-only features (NEL/LSEP line
    // endings, expanded NameStartChar, C1 controls as Char) that
    // 1.0 forbids. Including them in the audit would score the
    // 1.0 parser against the wrong spec.
    if path_str.contains("/xml-1.1/") {
        return None;
    }
    // The W3C edu-ni `namespaces/` subtree tests *Namespaces in
    // XML* (Bray, Hollander, Layman, Tobin & Thompson 2009 eds.,
    // W3C Recommendation) — a layered spec on top of XML 1.0/1.1
    // adding xmlns / prefix-binding and undeclared-prefix WFCs.
    // The praxis XML 1.0 parser is the *base* level (Name as
    // §2.3 [4], not §3 NCName) and intentionally treats `:` as
    // a name character without binding semantics. Namespace-
    // specific WFCs are scope of a separate ontology layer.
    if path_str.contains("/namespaces/") {
        return None;
    }
    Some(XmlConfCase {
        doc_path,
        case_type,
    })
}

/// Look up an unprefixed attribute by local name (XMLConf TEST
/// attributes are all in the no-namespace per the testcases.dtd).
fn attr_value(
    element: &crate::social::software::markup::xml::ontology::XmlElement,
    local: &str,
) -> Option<String> {
    element
        .attributes
        .iter()
        .find(|a| a.name.prefix.is_none() && a.name.local == local)
        .map(|a| a.value.clone())
}

/// Recursively list every `.xml` file under `base`. Every file is a
/// *candidate* manifest; the structural filter ("does the parsed
/// document contain any `<TEST>` element?") happens downstream in
/// [`walk_test_elements`]. Non-manifest `.xml` files (the actual
/// test documents) parse cleanly but emit zero cases via the walker;
/// `TYPE="not-wf"` test documents fail [`parse_document`] and are
/// silently skipped. This replaces the previous hand-coded
/// per-contributor filename whitelist with a structural recogniser
/// grounded in the XMLConf testcases.dtd `<!ELEMENT TESTCASES (TEST*)>`
/// shape (Bray et al. 2008 §3.2 — the testcases.dtd shipped with
/// the suite).
fn list_meta_files(base: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(ft) = entry.file_type()
                && ft.is_dir()
            {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) == Some("xml") {
                out.push(path);
            }
        }
    }
    out
}

/// What [`run_audit`] reports.
#[derive(Debug, Clone)]
pub enum XmlConfAuditOutcome {
    /// The corpus was walked.
    Walked(XmlConfAuditReport),
    /// The extracted tree is not at the expected on-disk path; soft-pass.
    ExtractedTreeAbsent { path: String },
    /// Source not registered — hard-fail.
    SourceNotRegistered,
}

/// Aggregate numbers from one full walk.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct XmlConfAuditReport {
    /// Total `TYPE="valid"` cases.
    pub valid_total: usize,
    /// Of `valid_total`, how many our parser accepted.
    pub valid_parse_ok: usize,
    /// Total `TYPE="invalid"` cases (well-formed but not DTD-valid).
    pub invalid_total: usize,
    /// Of `invalid_total`, how many our parser accepted (should be
    /// all — we don't do DTD validation).
    pub invalid_parse_ok: usize,
    /// Total `TYPE="not-wf"` cases.
    pub not_wf_total: usize,
    /// Of `not_wf_total`, how many our parser correctly rejected.
    pub not_wf_rejected: usize,
    /// Total `TYPE="error"` cases (optional-error: either outcome OK).
    pub error_total: usize,
    /// Of `error_total`, how many our parser accepted (informational).
    pub error_parse_ok: usize,
    /// Divergences bucketed by submanifest path (e.g.
    /// `ibm/not-wf/P85`). Counts are `[valid-rejected,
    /// invalid-rejected, not-wf-accepted]`. Empty buckets are
    /// omitted. Sorted by descending total divergence so the
    /// audit's failure message names the largest cluster first.
    pub divergence_buckets: alloc::collections::BTreeMap<String, [usize; 3]>,
    /// First divergent case in each bucket, with the parse error
    /// (for `valid`/`invalid` rejections) or empty (for `not-wf`
    /// accepts that contain no error). One sample per bucket
    /// keeps the failure message representative without flooding.
    pub divergence_samples: alloc::collections::BTreeMap<String, (String, String)>,
    /// Every divergent case per bucket, in walk order, up to
    /// [`Self::PER_BUCKET_LIMIT`] entries. The audit uses this to
    /// drive cluster-level triage: when one submanifest accounts
    /// for 20+ rejections we want every file name, not only the
    /// first. Each tuple is `(path, parse_error)`; the error is
    /// empty when the divergence is a `not-wf-accepted` (the
    /// parser produced an `Ok`).
    pub divergence_file_list: alloc::collections::BTreeMap<String, Vec<(String, String)>>,
}

impl XmlConfAuditReport {
    /// Maximum entries kept per bucket in
    /// [`Self::divergence_file_list`]. Caps the worst-case audit
    /// report size at this × bucket-count even on a wildly
    /// non-conformant build.
    pub const PER_BUCKET_LIMIT: usize = 50;

    /// 100% well-formedness conformance: every `valid` and `invalid`
    /// document (both well-formed by spec) parses, every `not-wf`
    /// document is rejected. `error` cases are informational.
    #[must_use]
    pub fn is_spec_conformant(&self) -> bool {
        self.valid_total > 0
            && self.not_wf_total > 0
            && self.valid_parse_ok == self.valid_total
            && self.invalid_parse_ok == self.invalid_total
            && self.not_wf_rejected == self.not_wf_total
    }

    /// Format the largest divergence buckets, descending. The audit
    /// failure-message uses this to surface where the gap is
    /// concentrated. Pass `top` to limit the row count. Each row
    /// is followed by the bucket's first divergent sample
    /// (path + parse-error) so the message is immediately
    /// actionable.
    #[must_use]
    pub fn divergence_summary(&self, top: usize) -> String {
        use core::fmt::Write;
        let mut rows: Vec<(&String, &[usize; 3])> = self.divergence_buckets.iter().collect();
        rows.sort_by(|a, b| {
            let ta: usize = a.1.iter().sum();
            let tb: usize = b.1.iter().sum();
            tb.cmp(&ta).then_with(|| a.0.cmp(b.0))
        });
        let mut out = String::new();
        out.push_str("submanifest, valid-rejected, invalid-rejected, not-wf-accepted\n");
        for (k, v) in rows.iter().take(top) {
            if v.iter().sum::<usize>() == 0 {
                continue;
            }
            let _ = writeln!(out, "  {} {} {} {}", k, v[0], v[1], v[2]);
            if let Some(samples) = self.divergence_file_list.get(*k) {
                for (path, err) in samples {
                    let leaf = path.rsplit('/').next().unwrap_or(path.as_str());
                    if err.is_empty() {
                        let _ = writeln!(out, "    - {leaf}");
                    } else {
                        let _ = writeln!(out, "    - {leaf}  ::  {err}");
                    }
                }
            } else if let Some((path, err)) = self.divergence_samples.get(*k) {
                let leaf = path.rsplit('/').next().unwrap_or(path.as_str());
                if err.is_empty() {
                    let _ = writeln!(out, "    sample: {leaf}");
                } else {
                    let _ = writeln!(out, "    sample: {leaf}  ::  {err}");
                }
            }
        }
        out
    }
}

/// Run the corpus-wide audit by consuming the cached [`loaded_xmlconf`]
/// corpus.
#[must_use]
pub fn run_audit() -> XmlConfAuditOutcome {
    if by_name_version("xmlconf_xml_test_suite", "2008-08-27").is_none() {
        return XmlConfAuditOutcome::SourceNotRegistered;
    }
    let Some(corpus) = loaded_xmlconf() else {
        return XmlConfAuditOutcome::ExtractedTreeAbsent {
            path: "crates/domains/data/markup-schemas/xmlconf/xmlconf".to_string(),
        };
    };
    let mut report = XmlConfAuditReport::default();
    for case in corpus.cases() {
        let Ok(bytes) = std::fs::read(&case.doc_path) else {
            continue;
        };
        let parsed = parse_document(&bytes);
        let (valid_rejected, invalid_rejected, not_wf_accepted) = match case.case_type {
            XmlConfType::Valid => {
                report.valid_total += 1;
                if parsed.is_ok() {
                    report.valid_parse_ok += 1;
                    (false, false, false)
                } else {
                    (true, false, false)
                }
            }
            XmlConfType::Invalid => {
                report.invalid_total += 1;
                if parsed.is_ok() {
                    report.invalid_parse_ok += 1;
                    (false, false, false)
                } else {
                    (false, true, false)
                }
            }
            XmlConfType::NotWf => {
                report.not_wf_total += 1;
                if parsed.is_err() {
                    report.not_wf_rejected += 1;
                    (false, false, false)
                } else {
                    (false, false, true)
                }
            }
            XmlConfType::Error => {
                report.error_total += 1;
                if parsed.is_ok() {
                    report.error_parse_ok += 1;
                }
                (false, false, false)
            }
        };
        if valid_rejected || invalid_rejected || not_wf_accepted {
            let bucket = submanifest_bucket(&case.doc_path);
            let counts = report
                .divergence_buckets
                .entry(bucket.clone())
                .or_insert([0, 0, 0]);
            if valid_rejected {
                counts[0] += 1;
            }
            if invalid_rejected {
                counts[1] += 1;
            }
            if not_wf_accepted {
                counts[2] += 1;
            }
            let path = case.doc_path.display().to_string();
            let err = match &parsed {
                Err(e) => format!("{e}"),
                Ok(_) => String::new(),
            };
            if !report.divergence_samples.contains_key(&bucket) {
                report
                    .divergence_samples
                    .insert(bucket.clone(), (path.clone(), err.clone()));
            }
            let list = report.divergence_file_list.entry(bucket).or_default();
            if list.len() < XmlConfAuditReport::PER_BUCKET_LIMIT {
                list.push((path, err));
            }
        }
    }
    XmlConfAuditOutcome::Walked(report)
}

/// Map a test-case path to a stable per-submanifest bucket key
/// like `ibm/not-wf/P85` — three path components below the
/// extracted `xmlconf/` root.
fn submanifest_bucket(p: &std::path::Path) -> String {
    let s = p.display().to_string();
    s.rsplit("/xmlconf/")
        .next()
        .and_then(|s| {
            let mut parts = s.splitn(4, '/');
            let a = parts.next()?;
            let b = parts.next()?;
            let c = parts.next().unwrap_or("");
            Some(format!("{a}/{b}/{c}"))
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Axiom: the praxis XML 1.0 parser is well-formedness-conformant on
/// every applicable case in the W3C XML Conformance Test Suite —
/// `valid` + `invalid` (well-formed by spec) all parse, `not-wf` all
/// reject. Soft-passes when the extracted tree is absent.
pub struct XmlConfCorpusAuditPasses;

impl Axiom for XmlConfCorpusAuditPasses {
    fn verify(&self) -> Verdict {
        match run_audit() {
            XmlConfAuditOutcome::Walked(report) if report.is_spec_conformant() => {
                Ok(Box::new(SimpleProof::new(self.meta())))
            }
            XmlConfAuditOutcome::Walked(_) => Err(Box::new(SimpleCounterexample::new(self.meta()))),
            XmlConfAuditOutcome::ExtractedTreeAbsent { .. } => {
                Ok(Box::new(SimpleProof::new(self.meta())))
            }
            XmlConfAuditOutcome::SourceNotRegistered => {
                Err(Box::new(SimpleCounterexample::new(self.meta())))
            }
        }
    }

    pr4xis::axiom_meta!(
        "XmlConfCorpusAuditPasses",
        "the praxis XML 1.0 parser is well-formedness-conformant on every applicable XMLConf case (valid + invalid both parse, not-wf rejects)",
        "W3C XML Test Suite Working Group, XML Conformance Test Suite; Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008) Extensible Markup Language (XML) 1.0 Fifth Edition, W3C Recommendation"
    );
}

pr4xis::register_axiom!(
    XmlConfCorpusAuditPasses,
    "W3C XML Test Suite Working Group, XML Conformance Test Suite; Bray et al. (2008) XML 1.0 Fifth Edition"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn case_from_test_walks_typed_xml() {
        // Verify the ontological walker recognises a synthesised
        // manifest. The fixture is parsed via parse_document (the
        // same XML 1.0 parser the audit uses); each TEST element's
        // URI + TYPE attributes are read through the typed
        // XmlAttribute API, not substring-scanned.
        let manifest = br#"<?xml version="1.0"?>
<TESTCASES>
  <TEST URI="valid/pe01.xml" ID="pe01" TYPE="valid"/>
  <TEST URI="not-wf/p01.xml" ID="p01" TYPE="not-wf"/>
  <TEST URI="ok/inv.xml" ID="inv" TYPE="invalid"/>
  <TEST URI="opt/err.xml" ID="err" TYPE="error"/>
</TESTCASES>"#;
        let doc = parse_document(manifest).expect("manifest must parse");
        let dir = std::path::Path::new("/tmp/synthetic");
        let mut out = Vec::new();
        walk_test_elements(&doc.root, dir, &mut out);
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].case_type, XmlConfType::Valid);
        assert!(out[0].doc_path.ends_with("valid/pe01.xml"));
        assert_eq!(out[1].case_type, XmlConfType::NotWf);
        assert_eq!(out[2].case_type, XmlConfType::Invalid);
        assert_eq!(out[3].case_type, XmlConfType::Error);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn loaded_xmlconf_is_either_cached_or_absent() {
        match loaded_xmlconf() {
            Some(c1) => {
                assert!(
                    c1.len().value >= 1_000.0,
                    "expected ≥1k cases, got {}",
                    c1.len().value
                );
                let c2 = loaded_xmlconf().expect("once-cached, always Some");
                assert!(core::ptr::eq(c1 as *const _, c2 as *const _));
            }
            None => {
                // Tree absent on this machine; soft-pass.
            }
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn audit_runs_and_reports() {
        match run_audit() {
            XmlConfAuditOutcome::Walked(report) => {
                // Order-of-magnitude sanity (the 2008-08-27 archive
                // declares ≈1.7k applicable cases across the four
                // types; the praxis parser handles them all).
                assert!(report.valid_total >= 500);
                assert!(report.not_wf_total >= 300);
                assert!(
                    report.is_spec_conformant(),
                    "non-conformance in audit:\n  totals: \
                     valid {}/{}, invalid {}/{}, not-wf {}/{}\n\
                     top divergence clusters:\n{}",
                    report.valid_parse_ok,
                    report.valid_total,
                    report.invalid_parse_ok,
                    report.invalid_total,
                    report.not_wf_rejected,
                    report.not_wf_total,
                    report.divergence_summary(15)
                );
            }
            XmlConfAuditOutcome::ExtractedTreeAbsent { .. } => {}
            XmlConfAuditOutcome::SourceNotRegistered => {
                panic!("xmlconf source must be registered in praxis.toml");
            }
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_holds() {
        assert!(XmlConfCorpusAuditPasses.verify().is_ok());
    }
}

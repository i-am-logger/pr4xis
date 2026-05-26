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

    /// Total case count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cases.len()
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
/// if the required attributes are missing or carry unrecognised
/// values.
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
    Some(XmlConfCase {
        doc_path: meta_dir.join(&uri),
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
}

impl XmlConfAuditReport {
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
        match case.case_type {
            XmlConfType::Valid => {
                report.valid_total += 1;
                if parsed.is_ok() {
                    report.valid_parse_ok += 1;
                }
            }
            XmlConfType::Invalid => {
                report.invalid_total += 1;
                if parsed.is_ok() {
                    report.invalid_parse_ok += 1;
                }
            }
            XmlConfType::NotWf => {
                report.not_wf_total += 1;
                if parsed.is_err() {
                    report.not_wf_rejected += 1;
                }
            }
            XmlConfType::Error => {
                report.error_total += 1;
                if parsed.is_ok() {
                    report.error_parse_ok += 1;
                }
            }
        }
    }
    XmlConfAuditOutcome::Walked(report)
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

    #[test]
    fn loaded_xmlconf_is_either_cached_or_absent() {
        match loaded_xmlconf() {
            Some(c1) => {
                assert!(c1.len() >= 1_000, "expected ≥1k cases, got {}", c1.len());
                let c2 = loaded_xmlconf().expect("once-cached, always Some");
                assert!(core::ptr::eq(c1 as *const _, c2 as *const _));
            }
            None => {
                // Tree absent on this machine; soft-pass.
            }
        }
    }

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
                    "non-conformance in audit: {report:?}"
                );
            }
            XmlConfAuditOutcome::ExtractedTreeAbsent { .. } => {}
            XmlConfAuditOutcome::SourceNotRegistered => {
                panic!("xmlconf source must be registered in praxis.toml");
            }
        }
    }

    #[test]
    fn axiom_holds() {
        assert!(XmlConfCorpusAuditPasses.verify().is_ok());
    }
}

//! Full Title 15 — Commerce and Trade (the Securities Act of 1933 and the
//! Securities Exchange Act of 1934, the statutory backbone of the SOX
//! whistleblower case: § 78j / Rule 10b-5, § 78m periodic reporting,
//! § 78u-6 Dodd-Frank whistleblower, § 78j-1 audit committees).
//!
//! The XML is parsed ONCE into a process-shared [`LazyLock`] fixture; every
//! `#[test]` below borrows the same immutable [`UsCodeTitle`]. Run under
//! `cargo test` (one process, thread-parallel), so the parse is paid once for
//! the whole file regardless of how many assertions touch it.

use std::sync::LazyLock;

use pr4xis::codegen::uslm::parse_uslm_str;
use pr4xis_domains::social::compliance::statutes::from_uslm::{
    derive_structural, from_uslm_section,
};
use pr4xis_domains::social::software::markup::xml::uslm::axioms::{
    axiom_child_identifier_extends_parent, axiom_every_container_has_identifier,
    axiom_every_section_has_num, axiom_hierarchy_strictly_nested, axiom_ref_hrefs_well_formed,
    axiom_section_identifiers_unique, section_identifier_to_statute_name,
};
use praxis_corpus_tests::{UslmCorpus, load_uslm_corpus};

/// Title 15, parsed once. `None` if the giant is not on disk (fresh checkout
/// before `pr4xis update`); each test then skips gracefully.
static TITLE_15: LazyLock<Option<UslmCorpus>> =
    LazyLock::new(|| load_uslm_corpus("legal/uscode/usc_title_15/usc_title_15-pl-119-90.xml"));

/// Borrow the shared corpus, or return early with a SKIP note when absent.
macro_rules! corpus_or_skip {
    () => {
        match &*TITLE_15 {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Title 15 USLM not on disk (run `pr4xis update`)");
                return;
            }
        }
    };
}

#[test]
fn full_title_15_parses_with_expected_section_count() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    assert_eq!(title.identifier, "/us/usc/t15");
    assert_eq!(title.number, 15);
    assert!(
        title.heading.to_ascii_uppercase().contains("COMMERCE"),
        "got heading: {:?}",
        title.heading
    );
    // LRC publishes ≥1,000 sections in Title 15 at this release point
    // (1,633 sections at pl-119-90 — Securities Act + Securities
    // Exchange Act alone account for ~250 sections in the §§ 77-78
    // run). Lower bound is conservative — the exact count drifts as
    // Congress adds/repeals sections.
    assert!(
        title.sections.len() >= 1_000,
        "expected ≥1,000 sections, got {}",
        title.sections.len()
    );
}

#[test]
fn full_title_15_every_section_has_unique_identifier() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let mut seen = std::collections::HashSet::new();
    for s in &title.sections {
        assert!(
            seen.insert(&s.identifier),
            "duplicate identifier: {}",
            s.identifier
        );
    }
}

#[test]
fn full_title_15_every_section_satisfies_every_axiom() {
    let UslmCorpus { title, .. } = corpus_or_skip!();

    axiom_every_section_has_num(title).expect("EverySectionHasNum must hold for full Title 15");
    axiom_every_container_has_identifier(title)
        .expect("EveryContainerHasIdentifier must hold for full Title 15");
    axiom_child_identifier_extends_parent(title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 15");
    axiom_hierarchy_strictly_nested(title)
        .expect("HierarchyStrictlyNested must hold for full Title 15");
    axiom_section_identifiers_unique(title)
        .expect("SectionIdentifiersUnique must hold for full Title 15");
    axiom_ref_hrefs_well_formed(title).expect("RefHrefsWellFormed must hold for full Title 15");
}

#[test]
fn full_title_15_every_section_lifts_to_statute() {
    let UslmCorpus { title, .. } = corpus_or_skip!();

    let mut failed = 0usize;
    let mut first_failure: Option<(String, String)> = None;
    for s in &title.sections {
        let name = section_identifier_to_statute_name(&s.identifier);
        if let Err(e) = from_uslm_section(&name, "pl-119-90", s) {
            failed += 1;
            if first_failure.is_none() {
                first_failure = Some((name, format!("{e}")));
            }
        }
    }
    if let Some((n, msg)) = first_failure {
        panic!(
            "{failed}/{} sections failed to lift to Statute; first: {n}: {msg}",
            title.sections.len()
        );
    }
    assert_eq!(failed, 0);
}

#[test]
fn full_title_15_known_sections_present() {
    // Sentinel sections directly relevant to the SOX whistleblower
    // case. Failure here means the LRC's release point drifted or
    // our slicing is wrong.
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let ids: std::collections::HashSet<&str> = title
        .sections
        .iter()
        .map(|s| s.identifier.as_str())
        .collect();

    // Securities Act of 1933 — short title (§ 77a) and definitions
    // (§ 77b). The 1933 Act ('33 Act) governs primary-market
    // securities offerings; SOX § 1514A protects reporting of
    // violations of "any rule or regulation of the Securities and
    // Exchange Commission" enacted under either act.
    assert!(
        ids.contains("/us/usc/t15/s77a"),
        "§ 77a (Securities Act short title) missing"
    );
    assert!(
        ids.contains("/us/usc/t15/s77b"),
        "§ 77b (Securities Act definitions) missing"
    );
    // Securities Act § 17(a) — fraudulent interstate transactions,
    // codified at 15 U.S.C. § 77q. One of the anti-fraud statutes
    // SOX § 1514A protects reporting of.
    assert!(
        ids.contains("/us/usc/t15/s77q"),
        "§ 77q (Securities Act § 17 anti-fraud) missing"
    );

    // Securities Exchange Act of 1934 — short title (§ 78a) and
    // definitions (§ 78c). The '34 Act governs secondary-market
    // trading and creates the SEC.
    assert!(
        ids.contains("/us/usc/t15/s78a"),
        "§ 78a (Exchange Act short title) missing"
    );
    assert!(
        ids.contains("/us/usc/t15/s78c"),
        "§ 78c (Exchange Act definitions) missing"
    );

    // § 78j — Manipulative and deceptive devices. § 78j(b) is the
    // statutory basis for SEC Rule 10b-5 (17 C.F.R. § 240.10b-5),
    // the workhorse private cause of action for securities fraud
    // and the underlying conduct SOX § 1514A protects reporting of.
    assert!(
        ids.contains("/us/usc/t15/s78j"),
        "§ 78j (Exchange Act § 10, Rule 10b-5 enabling) missing"
    );

    // § 78m — Periodicals and other reports. SOX § 302's CEO/CFO
    // certification requirement and SOX § 404's "internal control"
    // assessment are amendments to this section. The case's
    // protected activity centers on alleged misstatements in
    // § 78m-filed reports.
    assert!(
        ids.contains("/us/usc/t15/s78m"),
        "§ 78m (Exchange Act § 13 periodic reporting; SOX § 302/§ 404) missing"
    );

    // § 78ff — Penalties. Exchange Act § 32. Criminal penalties for
    // wilful violations, including knowingly making false § 78m
    // filings.
    assert!(
        ids.contains("/us/usc/t15/s78ff"),
        "§ 78ff (Exchange Act § 32 penalties) missing"
    );

    // § 78u-6 — Securities whistleblower incentives and protection.
    // Dodd-Frank § 922; the SEC monetary-award + anti-retaliation
    // scheme. Alternative whistleblower remedy that often co-exists
    // with a SOX § 806 (§ 1514A) claim — Digital Realty Trust, Inc.
    // v. Somers, 138 S. Ct. 767 (2018) drew the line between the
    // two regimes.
    //
    // NB: The LRC USLM encodes suffix-numbered section identifiers
    // with a U+2013 EN DASH (`–`), NOT the ASCII U+002D HYPHEN-MINUS
    // (`-`) used in citations. This is the GPO Style Manual 2016
    // §3.39 typographic convention for compound section numbers
    // (a hyphen vs an en-dash is intentional: en-dashes mark
    // numeric ranges and compound numbering). When a SOX brief
    // cites "15 U.S.C. § 78u-6" it uses ASCII; the USLM URN at the
    // LRC uses en-dash. Praxis stores identifiers verbatim from
    // the LRC source, so the en-dash form is authoritative here.
    assert!(
        ids.contains("/us/usc/t15/s78u\u{2013}6"),
        "§ 78u-6 (Dodd-Frank SEC whistleblower) missing"
    );

    // § 78j-1 — Audit committees. SOX § 301 added this entire
    // section to the Exchange Act: audit-committee independence,
    // pre-approval of audit/non-audit services, complaint-handling
    // procedures. Same LRC en-dash convention as § 78u-6.
    assert!(
        ids.contains("/us/usc/t15/s78j\u{2013}1"),
        "§ 78j-1 (SOX § 301 audit committees) missing"
    );
}

#[test]
fn full_title_15_codegen_and_runtime_agree_on_rule_10b5() {
    // The SOX case's load-bearing § in Title 15 is § 78j (Exchange
    // Act § 10, the Rule 10b-5 enabling statute). Verify codegen
    // and runtime extract the same term IDs for it — the same
    // contract Title 49's air21_42121 satisfies.
    let UslmCorpus { xml, title } = corpus_or_skip!();
    let section = title
        .section("/us/usc/t15/s78j")
        .expect("§ 78j must be present");
    let runtime_data = derive_structural("sea_78j", section);
    let codegen_doc = parse_uslm_str(xml, "/us/usc/t15/s78j", "sea_78j").expect("codegen parse");
    let runtime_ids: std::collections::HashSet<&str> =
        runtime_data.terms.iter().map(|t| t.id.as_str()).collect();
    let codegen_ids: std::collections::HashSet<&str> = codegen_doc
        .terms
        .iter()
        .filter(|t| t.id != "sea_78j")
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        runtime_ids, codegen_ids,
        "codegen and runtime diverge on Title 15 § 78j"
    );
}

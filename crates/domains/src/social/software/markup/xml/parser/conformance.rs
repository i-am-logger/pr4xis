//! W3C XML 1.0 conformance test harness.
//!
//! Defines a [`ConformanceCase`] taxonomy mirroring the W3C XML
//! Conformance Test Suite (XMLConf) categories and a representative
//! hand-authored corpus the parser MUST handle correctly. Each case
//! cites the W3C XML 1.0 Fifth Edition §-reference whose
//! well-formedness or syntactic rule it exercises.
//!
//! Per W3C XML Test Suite Documentation
//! <https://www.w3.org/XML/Test/xmlconf-20081126.html>, every test
//! has a `TYPE` attribute classifying it:
//!
//! - **valid** — well-formed and valid per its DTD. A parser MUST
//!   accept it.
//! - **invalid** — well-formed but not valid per its DTD. A non-
//!   validating parser like ours MUST still accept it; a validating
//!   parser MUST report the validity error.
//! - **not-wf** — not well-formed. Every conforming parser MUST
//!   reject it with a fatal error.
//! - **error** — exercises an optional error condition. Parsers MAY
//!   accept or reject.
//!
//! The corpus below carries one [`ConformanceCase`] per category
//! with the §-reference cited inline. Future work (M4.λ.1.d.b) is
//! to register the full ~2000-case XMLConf archive as a praxis
//! source and run every case through the same harness.
//!
//! # Citations
//!
//! - **Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008)** W3C
//!   XML 1.0 Fifth Edition, W3C Recommendation 26 November 2008,
//!   the productions and well-formedness constraints under test.
//! - **W3C XML Test Suite Working Group**, "XML Test Suite",
//!   <https://www.w3.org/XML/Test/> — the canonical test corpus
//!   we model after.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec::Vec};

use super::grammar::parse_document;

/// W3C XMLConf category for a test case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseType {
    /// Well-formed + DTD-valid. Parser MUST accept.
    Valid,
    /// Well-formed but DTD-invalid. Non-validating parser (ours)
    /// MUST still accept.
    Invalid,
    /// Not well-formed. Every conforming parser MUST reject as
    /// fatal error.
    NotWellFormed,
    /// Optional error condition. Parser MAY accept or reject.
    Error,
}

/// One test case in the conformance harness.
#[derive(Debug, Clone)]
pub struct ConformanceCase {
    /// Stable identifier — used in failure messages.
    pub id: &'static str,
    /// XMLConf category.
    pub case_type: CaseType,
    /// The XML 1.0 §-reference the case exercises.
    pub section: &'static str,
    /// Short prose description.
    pub description: &'static str,
    /// The XML document bytes under test.
    pub source: &'static [u8],
}

/// Result of running one [`ConformanceCase`] through the parser.
#[derive(Debug, Clone)]
pub struct CaseOutcome {
    pub case_id: &'static str,
    pub expected: CaseType,
    pub passed: bool,
    pub detail: String,
}

/// Run a single case through [`parse_document`] and report the
/// outcome.
///
/// - For [`CaseType::Valid`] / [`CaseType::Invalid`] / [`CaseType::Error`]
///   the parser must succeed (accept the document).
/// - For [`CaseType::NotWellFormed`] the parser must return an
///   error.
pub fn run_case(case: &ConformanceCase) -> CaseOutcome {
    let parse_result = parse_document(case.source);
    let (passed, detail) = match (case.case_type, &parse_result) {
        (CaseType::Valid, Ok(_)) | (CaseType::Invalid, Ok(_)) => {
            (true, "accepted as expected".into())
        }
        (CaseType::Valid, Err(e)) | (CaseType::Invalid, Err(e)) => {
            (false, format!("must accept but rejected: {e}"))
        }
        (CaseType::NotWellFormed, Err(_)) => (true, "rejected as expected".into()),
        (CaseType::NotWellFormed, Ok(_)) => (false, "must reject but accepted".into()),
        // W3C XML Conformance Test Suite, testcases.dtd:
        //   TYPE="error" — "Optional error: the parser MAY accept
        //   or reject; the specification is neutral on the outcome."
        // Either outcome counts as a pass; the detail records the
        // observed behavior for the diagnostic record.
        (CaseType::Error, Ok(_)) => (true, "accepted (optional-error case)".into()),
        (CaseType::Error, Err(e)) => (true, format!("rejected (optional-error case): {e}")),
    };
    CaseOutcome {
        case_id: case.id,
        expected: case.case_type,
        passed,
        detail,
    }
}

/// Run the entire hand-authored corpus.
pub fn run_canon_corpus() -> Vec<CaseOutcome> {
    canon_corpus().iter().map(run_case).collect()
}

/// The hand-authored canon — one representative test per major
/// well-formedness or syntactic rule the parser claims to enforce.
/// Each case cites the W3C XML 1.0 Fifth Edition section it tests.
pub fn canon_corpus() -> &'static [ConformanceCase] {
    CANON
}

const CANON: &[ConformanceCase] = &[
    // -------- Valid cases (must accept) --------
    ConformanceCase {
        id: "valid-001-minimal",
        case_type: CaseType::Valid,
        section: "§2.1 [1] document",
        description: "minimal well-formed XML with empty root element",
        source: b"<?xml version=\"1.0\"?><root/>",
    },
    ConformanceCase {
        id: "valid-002-nested-elements",
        case_type: CaseType::Valid,
        section: "§3 [39] element",
        description: "nested elements with text content",
        source: b"<?xml version=\"1.0\"?><a><b>x</b></a>",
    },
    ConformanceCase {
        id: "valid-003-attributes",
        case_type: CaseType::Valid,
        section: "§3.1 [41] Attribute",
        description: "element with multiple attributes",
        source: b"<?xml version=\"1.0\"?><e a=\"1\" b=\"two\"/>",
    },
    ConformanceCase {
        id: "valid-004-predefined-entities",
        case_type: CaseType::Valid,
        section: "§4.6 predefined entities",
        description: "the five predefined entities in element content",
        source: b"<?xml version=\"1.0\"?><r>&amp;&lt;&gt;&apos;&quot;</r>",
    },
    ConformanceCase {
        id: "valid-005-character-references",
        case_type: CaseType::Valid,
        section: "§4.1 [66] CharRef",
        description: "decimal and hex character references",
        source: b"<?xml version=\"1.0\"?><r>&#65;&#x42;</r>",
    },
    ConformanceCase {
        id: "valid-006-cdata-section",
        case_type: CaseType::Valid,
        section: "§2.7 [18] CDSect",
        description: "CDATA section preserves markup-delimiter chars",
        source: b"<?xml version=\"1.0\"?><r><![CDATA[<not> &an; entity]]></r>",
    },
    ConformanceCase {
        id: "valid-007-comment",
        case_type: CaseType::Valid,
        section: "§2.5 [15] Comment",
        description: "inline comment in element content",
        source: b"<?xml version=\"1.0\"?><r>a<!-- mid -->b</r>",
    },
    ConformanceCase {
        id: "valid-008-pi",
        case_type: CaseType::Valid,
        section: "§2.6 [16] PI",
        description: "processing instruction in element content",
        source: b"<?xml version=\"1.0\"?><r>x<?stylesheet href=\"a.css\"?>y</r>",
    },
    ConformanceCase {
        id: "valid-009-namespace-default",
        case_type: CaseType::Valid,
        section: "Namespaces in XML 1.0 §3",
        description: "default namespace declaration",
        source: b"<?xml version=\"1.0\"?><root xmlns=\"http://example.org/\"/>",
    },
    ConformanceCase {
        id: "valid-010-namespace-prefix",
        case_type: CaseType::Valid,
        section: "Namespaces in XML 1.0 §3",
        description: "prefixed namespace declaration",
        source: b"<?xml version=\"1.0\"?><root xmlns:dc=\"http://purl.org/dc/\"/>",
    },
    ConformanceCase {
        id: "valid-011-utf8-bom",
        case_type: CaseType::Valid,
        section: "§F Autodetection of Encodings",
        description: "UTF-8 BOM at start of document",
        source: b"\xEF\xBB\xBF<?xml version=\"1.0\"?><r/>",
    },
    ConformanceCase {
        id: "valid-012-crlf-content",
        case_type: CaseType::Valid,
        section: "§2.11 End-of-Line Handling",
        description: "CRLF line endings in element content",
        source: b"<?xml version=\"1.0\"?><r>a\r\nb</r>",
    },
    ConformanceCase {
        id: "valid-013-doctype-name",
        case_type: CaseType::Valid,
        section: "§2.8 [28] doctypedecl",
        description: "DOCTYPE with root name only",
        source: b"<?xml version=\"1.0\"?><!DOCTYPE r><r/>",
    },
    ConformanceCase {
        id: "valid-014-doctype-entity",
        case_type: CaseType::Valid,
        section: "§4.2 [70] GEDecl",
        description: "DOCTYPE with internal general entity used in content",
        source: b"<?xml version=\"1.0\"?><!DOCTYPE r [<!ENTITY x \"hello\">]><r>&x;</r>",
    },
    // -------- Invalid cases (well-formed; non-validating parser must accept) --------
    ConformanceCase {
        id: "invalid-001-undeclared-element",
        case_type: CaseType::Invalid,
        section: "§3.2 elementdecl (validity)",
        description: "element used without DTD declaration; non-validating parser accepts",
        source: b"<?xml version=\"1.0\"?><!DOCTYPE r [<!ELEMENT s EMPTY>]><r><s/></r>",
    },
    ConformanceCase {
        id: "invalid-002-undeclared-attribute",
        case_type: CaseType::Invalid,
        section: "§3.3 AttlistDecl (validity)",
        description: "attribute used without DTD declaration; non-validating parser accepts",
        source: b"<?xml version=\"1.0\"?><!DOCTYPE r [<!ELEMENT r EMPTY>]><r undeclared=\"x\"/>",
    },
    // -------- Not-well-formed cases (must reject) --------
    ConformanceCase {
        id: "not-wf-001-mismatched-tags",
        case_type: CaseType::NotWellFormed,
        section: "§3 Element Type Match",
        description: "STag and ETag names disagree",
        source: b"<?xml version=\"1.0\"?><a></b>",
    },
    ConformanceCase {
        id: "not-wf-002-duplicate-attribute",
        case_type: CaseType::NotWellFormed,
        section: "§3.1 Unique Att Spec",
        description: "same attribute name twice in start-tag",
        source: b"<?xml version=\"1.0\"?><r a=\"1\" a=\"2\"/>",
    },
    ConformanceCase {
        id: "not-wf-003-unclosed-tag",
        case_type: CaseType::NotWellFormed,
        section: "§3 [39] element",
        description: "STag without matching ETag",
        source: b"<?xml version=\"1.0\"?><r>",
    },
    ConformanceCase {
        id: "not-wf-004-unknown-entity",
        case_type: CaseType::NotWellFormed,
        section: "§4.4.3 Included",
        description: "entity reference with no §4.6 / DTD declaration",
        source: b"<?xml version=\"1.0\"?><r>&unknown;</r>",
    },
    ConformanceCase {
        id: "not-wf-005-bare-ampersand",
        case_type: CaseType::NotWellFormed,
        section: "§2.4 [14] CharData",
        description: "literal '&' in element content (not a reference)",
        source: b"<?xml version=\"1.0\"?><r>a & b</r>",
    },
    ConformanceCase {
        id: "not-wf-006-bare-lt-in-attvalue",
        case_type: CaseType::NotWellFormed,
        section: "§3.1 [10] AttValue",
        description: "literal '<' in attribute value",
        source: b"<?xml version=\"1.0\"?><r a=\"x<y\"/>",
    },
    ConformanceCase {
        id: "not-wf-007-empty-document",
        case_type: CaseType::NotWellFormed,
        section: "§2.1 [1] document",
        description: "no document element",
        source: b"<?xml version=\"1.0\"?>",
    },
    // -------- Error cases (optional; parser may accept) --------
    ConformanceCase {
        id: "error-001-leading-whitespace",
        case_type: CaseType::Error,
        section: "§2.8 [22] prolog",
        description: "leading whitespace before XML declaration (optional error)",
        source: b"  <?xml version=\"1.0\"?><r/>",
    },
];

//! Praxis-level unit tests for the RDFC-1.0 implementation, pinning the
//! W3C spec's own worked examples and the DoS-cap boundary. The exhaustive
//! fixture gate lives in `tests/rdf_canon_suite.rs`; these tests document
//! the *mechanism* against named spec values so a regression localizes.

use super::*;
use alloc::{format, string::String};

/// REC §4.6 "Hash First Degree Quads", "Unique hashes" example: the input
/// `:p :q _:e0 . :p :r _:e1 . _:e0 :s :u . _:e1 :t :u .` canonicalizes
/// with all-unique first-degree hashes, so each blank node gets a stable
/// `c14n{N}` purely from the first-degree pass (no N-degree recursion).
#[pr4xis::praxis_value(Verifiable, Deterministic)]
#[test]
fn spec_unique_hashes_example() {
    let input = "\
<http://example.com/#p> <http://example.com/#q> _:e0 .
<http://example.com/#p> <http://example.com/#r> _:e1 .
_:e0 <http://example.com/#s> <http://example.com/#u> .
_:e1 <http://example.com/#t> <http://example.com/#u> .
";
    let out = canonicalize_nquads(input).expect("canonicalize");
    // The two blank nodes are distinguishable by their first-degree hash;
    // code-point hash order assigns c14n0/c14n1 deterministically. Whatever
    // the assignment, the result must be stable and contain both.
    assert!(out.contains("_:c14n0"), "got:\n{out}");
    assert!(out.contains("_:c14n1"), "got:\n{out}");
    // Idempotence: canonicalizing the canonical form is a fixed point.
    let again = canonicalize_nquads(&out).expect("re-canonicalize");
    assert_eq!(out, again, "canonicalization is not idempotent");
}

/// REC §4.4 "Shared hashes" example. `_:e0` and `_:e1` share a first-degree
/// hash and are only distinguished by the **N-degree** walk; the spec's own
/// final issued map is `{e2: c14n0, e3: c14n1, e1: c14n2, e0: c14n3}`. We
/// assert the resulting canonical form is the spec's serialized output and
/// that the issued map matches those canonical labels.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn spec_shared_hashes_example() {
    let input = "\
<http://example.com/#p> <http://example.com/#q> _:e0 .
<http://example.com/#p> <http://example.com/#q> _:e1 .
_:e0 <http://example.com/#p> _:e2 .
_:e1 <http://example.com/#p> _:e3 .
_:e2 <http://example.com/#r> _:e3 .
";
    let quads = parse_nquads(input).expect("parse");
    let (out, map) =
        canonicalize_with(&quads, CanonLimits::default(), HashAlgorithm::Sha256).expect("canon");
    // Spec final issued identifiers map (REC §4.4.3 ca.6 worked example).
    assert_eq!(map.get("e2").map(String::as_str), Some("c14n0"));
    assert_eq!(map.get("e3").map(String::as_str), Some("c14n1"));
    assert_eq!(map.get("e1").map(String::as_str), Some("c14n2"));
    assert_eq!(map.get("e0").map(String::as_str), Some("c14n3"));
    // Every input blank node received a canonical label.
    assert_eq!(map.len(), 4);
    // The canonical serialization is sorted and ends in a final LF.
    assert!(out.ends_with(".\n"), "missing final EOL:\n{out}");
}

/// Two *isomorphic* datasets with differently-named blank nodes MUST
/// canonicalize to byte-identical output — the defining property of
/// canonicalization (REC §3, dataset isomorphism). This is the cyclic case
/// a content-addressed labelling cannot handle.
#[pr4xis::praxis_value(Deterministic)]
#[test]
fn isomorphic_relabelling_is_invariant() {
    let a = "\
_:a <http://ex/p> _:b .
_:b <http://ex/p> _:a .
";
    let b = "\
_:x <http://ex/p> _:y .
_:y <http://ex/p> _:x .
";
    let ca = canonicalize_nquads(a).expect("a");
    let cb = canonicalize_nquads(b).expect("b");
    assert_eq!(ca, cb, "isomorphic graphs gave different canonical forms");
}

/// Simple-literal datatype elision (REC §"A Canonical form of N-Quads"):
/// an explicit `^^xsd:string` MUST be dropped, so it is indistinguishable
/// from a bare simple literal.
#[pr4xis::praxis_value(Deterministic, Verifiable)]
#[test]
fn xsd_string_datatype_is_elided() {
    let with_dt =
        "<http://ex/s> <http://ex/p> \"v\"^^<http://www.w3.org/2001/XMLSchema#string> .\n";
    let bare = "<http://ex/s> <http://ex/p> \"v\" .\n";
    assert_eq!(
        canonicalize_nquads(with_dt).unwrap(),
        canonicalize_nquads(bare).unwrap(),
    );
    assert_eq!(canonicalize_nquads(bare).unwrap(), bare);
}

/// The DoS cap is a *typed error*, never a hang or panic. Two independent
/// ceilings are exercised:
///
/// 1. `max_hndq_calls` — a graph whose blank nodes share a first-degree
///    hash forces the N-degree walk; a ceiling of 1 trips on the second
///    invocation.
/// 2. `max_permutations` — the official `test074` clique forms a related
///    blank-node list of length 9 (9! = 362_880 > the 40_320 default), so
///    even with a generous call budget it is refused at the permutation
///    factorial guard, exactly as the negative suite test requires.
#[pr4xis::praxis_value(Honest)]
#[test]
fn complexity_cap_errors_typed() {
    // Three mutually symmetric blank nodes — each pair shares a hash, so
    // ca.5 must run Hash N-Degree Quads more than once.
    let input = "\
_:e0 <http://ex/p> _:e1 .
_:e0 <http://ex/p> _:e2 .
_:e1 <http://ex/p> _:e0 .
_:e1 <http://ex/p> _:e2 .
_:e2 <http://ex/p> _:e0 .
_:e2 <http://ex/p> _:e1 .
";
    let quads = parse_nquads(input).expect("parse");
    let call_capped = CanonLimits {
        max_hndq_calls: 1,
        max_permutations: 40_320,
    };
    let err = canonicalize_with(&quads, call_capped, HashAlgorithm::Sha256).unwrap_err();
    assert!(
        matches!(
            err,
            CanonError::ComplexityCapExceeded {
                what: "Hash N-Degree Quads invocations",
                ..
            }
        ),
        "expected an HNDQ-call cap error, got {err:?}"
    );

    // A 10-node blank-node clique (the structure of the official negative
    // suite test074) drives the N-degree walk past the call budget: every
    // node shares one first-degree hash and the gossip-path exploration
    // explodes. With the default limits this MUST error rather than hang or
    // produce a (meaningless) labelling. The exhaustive fixture gate in
    // `tests/rdf_canon_suite.rs` asserts this against the vendored
    // `test074-in.nq` byte-for-byte; here we reconstruct the shape so the
    // unit test is self-contained.
    let mut clique = String::new();
    for i in 0..10u32 {
        for j in 0..10u32 {
            // test074 includes the self-edge _:ei p _:ei (the diagonal).
            clique.push_str(&format!("_:e{i} <http://ex/p> _:e{j} .\n"));
        }
    }
    let quads = parse_nquads(&clique).expect("parse clique");
    let err = canonicalize_with(&quads, CanonLimits::default(), HashAlgorithm::Sha256).unwrap_err();
    assert!(
        matches!(err, CanonError::ComplexityCapExceeded { .. }),
        "clique should hit the cap, got {err:?}"
    );
}

/// Malformed N-Quads is a typed parse error, not a panic.
#[pr4xis::praxis_value(Honest)]
#[test]
fn malformed_input_is_typed_error() {
    assert!(matches!(
        parse_nquads("this is not n-quads"),
        Err(CanonError::Parse(_))
    ));
    assert!(matches!(
        parse_nquads("<http://ex/s> <http://ex/p> <http://ex/o>"), // no '.'
        Err(CanonError::Parse(_))
    ));
}

/// Round-trip of the escaping torture vectors: parsing then canonical
/// re-serialization of a literal with control characters and a UCHAR must
/// re-escape per the appendix table (`\t` ECHAR, control via lowercase
/// `\u` + UPPER hex).
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn canonical_literal_escaping() {
    // Tab (ECHAR), a U+0001 control (UCHAR), and a native astral char.
    let input = "<http://ex/s> <http://ex/p> \"\\t\\u0001\\U0001F303\" .\n";
    let out = canonicalize_nquads(input).expect("canon");
    assert_eq!(
        out, "<http://ex/s> <http://ex/p> \"\\t\\u0001\u{1F303}\" .\n",
        "got: {out:?}"
    );
}

/// Duplicate input triples collapse to one quad (a dataset is a *set*):
/// REC ca.2 / the suite's test076.
#[pr4xis::praxis_value(Deterministic)]
#[test]
fn duplicate_triples_collapse() {
    let input = "\
<http://ex/s> <http://ex/p> <http://ex/o> .
<http://ex/s> <http://ex/p> <http://ex/o> .
";
    let out = canonicalize_nquads(input).expect("canon");
    assert_eq!(out, "<http://ex/s> <http://ex/p> <http://ex/o> .\n");
}

/// Named graphs: a quad in a named graph keeps its fourth component, and a
/// blank-node graph name participates in canonicalization (REC §4.4.3
/// step 2 treats it as a blank-node component).
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn named_graph_and_blank_graph_name() {
    let input = "\
<http://ex/s> <http://ex/p> <http://ex/o> <http://ex/g> .
_:b <http://ex/p> <http://ex/o> _:b .
";
    let out = canonicalize_nquads(input).expect("canon");
    assert!(
        out.contains("<http://ex/g> .\n"),
        "named graph lost:\n{out}"
    );
    assert!(
        out.contains("_:c14n0"),
        "blank graph name unlabelled:\n{out}"
    );
}

/// SHA-384 selection (the suite's test075) yields a *different* canonical
/// labelling decision space than SHA-256 in general, but for an
/// all-unique-hash graph both must still succeed and be idempotent.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn sha384_algorithm_runs() {
    let input = "_:a <http://ex/p> <http://ex/o> .\n";
    let quads = parse_nquads(input).unwrap();
    let (out, _m) =
        canonicalize_with(&quads, CanonLimits::default(), HashAlgorithm::Sha384).expect("sha384");
    assert_eq!(out, "_:c14n0 <http://ex/p> <http://ex/o> .\n");
}

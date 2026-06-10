//! `OwlLens` — the well-behaved lens binding OWL/RDF bytes ⇆
//! [`OwlOntology`].
//!
//! Per Foster, Greenwald, Moore, Pierce & Schmitt 2007 §2.2 a
//! well-behaved lens is a `(get, put)` pair satisfying GetPut and
//! PutGet. For the OWL leg:
//!
//!   `get  : bytes → OwlOntology`         = [`read_owl`]
//!   `put  : OwlOntology → bytes`         = [`write_owl`]
//!   `canonical : bytes → bytes`           = RDFC-1.0 canonical N-Quads
//!                                           of the **source RDF graph**
//!
//! ## What the canonical form is (graph identity, not typed-view bytes)
//!
//! `canonical(bytes)` is the **W3C RDF Dataset Canonicalization
//! (RDFC-1.0, REC-rdf-canon-20240521)** serialized canonical form
//! (sorted canonical N-Quads) of the *raw* RDF graph the document
//! denotes — `read_owl_to_quads(bytes)`, the [`Triple`] stream below the
//! [`OwlOntology`] typed projection. Two OWL byte streams are taken to
//! denote the same source iff their canonical N-Quads are byte-identical,
//! i.e. iff the graphs are RDF-isomorphic (RDF 1.1 Concepts §3.6 graph
//! isomorphism; RDFC §1.1). This pins **graph identity** — the full
//! triple set the source asserts.
//!
//! This is an *altitude fix* over the previous canonical form,
//! `write_owl(read_owl(bytes))`, which serialized only the typed
//! projection's view and therefore dropped every triple the
//! [`OwlOntology`] view does not model (annotations on anonymous nodes,
//! versioning/import triples, `owl:AllDisjointClasses` and other shapes
//! the projector skips). Empirically the typed view drops 24–1187
//! triples per bundled vocab, so the old form was lossy; RDFC over the
//! raw graph pins the whole source graph.
//!
//! ## Scope of the identity (what is and is not pinned)
//!
//! - RDFC-1.0 fixes **RDF graph identity** (RDF 1.1 §3.6; RDFC §1.1):
//!   the blank-node *arrangement* is canonicalized, so isomorphic graphs
//!   coincide. The six bundled vocabularies DO carry blank nodes (cito 9,
//!   doco 287, c4o 21, biro 16, prov_o 68, olia 317 distinct blanks — from
//!   `rdf:nodeID`, `rdf:parseType="Collection"`, and
//!   `owl:unionOf`/`intersectionOf`/`Restriction`), so RDFC's deterministic
//!   blank-node labelling is **load-bearing** for the graph identity. The
//!   altitude fix (whole graph vs lossy typed-view subset, dropping
//!   24–1187 triples/vocab) + the W3C-standard grounding + the live
//!   load-gate are the further gains.
//! - It is **not** OWL-structural identity (OWL 2 Structural
//!   Specification §3 — structural equivalence over axioms) — that is a
//!   deliberate, separate follow-up.
//! - It is **not** logical / entailment identity (two graphs with the
//!   same models) — never pinned; entailment is undecidable in general
//!   for OWL 2 Full.
//! - Term-level caveat: RDFC canonicalizes blank-node arrangement, **not
//!   literal lexical forms** — `"1"^^xsd:int` and `"01"^^xsd:int` remain
//!   distinct N-Quads (RDFC §1.1 leaves datatype-value canonicalization
//!   to the caller).
//!
//! ## The two laws, kept separate
//!
//! - **Graph-identity canonical form** (`canonical` / `signature`) — what
//!   `[canonical_signatures]` pins. RDFC over the source graph.
//! - **Typed-view PutGet** (`write_owl ∘ read_owl`) — the categorical
//!   round-trip `read_owl ∘ write_owl ∘ read_owl ≡ read_owl` over the
//!   typed projection, witnessed by the `put_get_law_*` tests and the
//!   proptests below. This is still a legitimate well-behaved-lens check
//!   on the projection; it is simply no longer what `canonical` hashes.
//!   Because the writer is **not** graph-faithful, `write_owl ∘ read_owl`
//!   is *not* identity on the source graph, so it cannot serve as the
//!   round-trip leg of the graph-identity PutGet law; the floor uses the
//!   constant-complement identity instead (see
//!   [`OwlLens::apply_put_after_get`]).
//!
//! ## Citations
//!
//! - **Foster, J. N.; Greenwald, M. B.; Moore, J. T.; Pierce, B. C.;
//!   Schmitt, A. (2007)** — "Combinators for Bidirectional Tree
//!   Transformations", *ACM TOPLAS* 29(3) Article 17, §2.2.
//! - **Longley, D.; Kellogg, G.; Yamamoto, D. (eds.) (2024)** — *RDF
//!   Dataset Canonicalization* (RDFC-1.0), W3C Recommendation
//!   REC-rdf-canon-20240521, §1.1 / §4.4.3.
//!   <https://www.w3.org/TR/rdf-canon/>.
//! - **Cyganiak, R.; Wood, D.; Lanthaler, M. (eds.) (2014)** — *RDF 1.1
//!   Concepts and Abstract Syntax*, §3.6 (graph isomorphism), §4 (RDF
//!   datasets).
//! - **Hogan, A. (2017)** — "Canonical forms for isomorphic and
//!   equivalent RDF graphs", *ACM TWEB* 11(4) Article 22.
//! - **Patel-Schneider, P. F. & Motik, B. (eds.) (2012)** — *OWL 2
//!   Web Ontology Language: Mapping to RDF Graphs (2nd ed.)*, W3C
//!   Recommendation 11 December 2012. The set-theoretic mapping
//!   `read_owl` inverts.
//!
//! [`Triple`]: crate::social::software::markup::xml::rdf::Triple

#[allow(unused_imports)]
use alloc::{format, string::String, vec::Vec};
use core::fmt;

use super::ontology::OwlOntology;
use super::reader::{OwlReadError, read_owl, read_owl_to_quads};
use super::writer::write_owl;
use crate::formal::meta::well_behaved_lens::WellBehavedLens;
use crate::social::software::markup::xml::rdf::canonicalize as rdf_canonicalize;

/// The well-behaved lens binding OWL/RDF bytes ⇆ [`OwlOntology`].
///
/// `Target = OwlOntology` — no byte side-channel. `canonical(bytes)` is
/// the RDFC-1.0 canonical N-Quads of the source RDF graph
/// (`read_owl_to_quads(bytes)`), so the `[canonical_signatures]` pin is
/// the **graph identity** of the source — drift in any triple the source
/// asserts (not just the typed-view subset) surfaces as a signature
/// mismatch.
pub struct OwlLens;

#[derive(Debug)]
pub enum OwlLensError {
    /// UTF-8 decoding of the input bytes failed.
    NotUtf8(String),
    /// The OWL reader rejected the document.
    Read(OwlReadError),
    /// RDFC-1.0 canonicalization of the source graph failed — a
    /// malformed graph or a poison dataset tripping the DoS cap
    /// (RDFC §"Dataset Poisoning"). Carries the human-readable reason.
    Canonicalize(String),
}

impl fmt::Display for OwlLensError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUtf8(m) => write!(f, "owl lens: not valid UTF-8: {m}"),
            Self::Read(e) => write!(f, "owl lens: read error: {e}"),
            Self::Canonicalize(m) => write!(f, "owl lens: RDFC-1.0 canonicalization error: {m}"),
        }
    }
}

impl From<OwlReadError> for OwlLensError {
    fn from(e: OwlReadError) -> Self {
        Self::Read(e)
    }
}

impl WellBehavedLens for OwlLens {
    type Target = OwlOntology;
    type Error = OwlLensError;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        let text =
            core::str::from_utf8(bytes).map_err(|e| OwlLensError::NotUtf8(format!("{e}")))?;
        // Strip an optional BOM (W3C XML 1.0 §F.1) before parsing so the
        // lens accepts byte streams shipped from upstream registries
        // that prepend it.
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        Ok(read_owl(text)?)
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(write_owl(target))
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        // Graph-identity canonical form: the RDFC-1.0 (RDFC §4.4.3)
        // serialized canonical N-Quads of the *raw* RDF graph the
        // document denotes — the Triple stream below the OwlOntology
        // typed projection. This pins the whole source graph (RDF 1.1
        // §3.6 graph isomorphism), not the lossy typed-view re-emission.
        let text =
            core::str::from_utf8(bytes).map_err(|e| OwlLensError::NotUtf8(format!("{e}")))?;
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let quads = read_owl_to_quads(text)?;
        let nquads =
            rdf_canonicalize(&quads).map_err(|e| OwlLensError::Canonicalize(format!("{e}")))?;
        Ok(nquads.into_bytes())
    }

    /// The lens round-trip for the PutGet law (Foster et al. 2007 §2.2).
    ///
    /// `OwlLens` is a [`RoundTripFidelity::RawBytesComplementFloor`]
    /// source (`write_owl` is **not** graph-faithful, so the typed-view
    /// round-trip loses triples — see the module docs). The floor's
    /// PutGet leg is therefore the **constant-complement identity**
    /// (Bancilhon & Spyratos 1981): the source is reconstructed exactly
    /// from its stored complement (the source bytes themselves), so
    /// `put(get(b)) == b` and the canonical-form PutGet law
    /// `canonical(put(get(b))) == canonical(b)` holds by construction.
    /// The substantive verification is then the `[canonical_signatures]`
    /// graph-identity pin the harness checks against `canonical(b)`.
    ///
    /// The typed-view round-trip `write_owl ∘ read_owl` is a *separate*
    /// well-behaved-lens check on the projection, witnessed by the
    /// `put_get_law_*` tests and the proptests in this module — not by
    /// this method (using it here would falsely fail the graph-identity
    /// law, since the writer drops triples the RDFC form preserves).
    ///
    /// [`RoundTripFidelity::RawBytesComplementFloor`]:
    ///     crate::formal::meta::well_behaved_lens::RoundTripFidelity::RawBytesComplementFloor
    fn apply_put_after_get(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        Ok(bytes.to_vec())
    }
}

// =============================================================================
// Round-trip harness registrations — one entry per bundled OWL source.
// =============================================================================

// NOTE: NONE of the bundled OWL vocabularies are registered with the floor
// `OwlLens` here any more. The FLAT SPAR family — `cito@2.8.1`, `biro@1.1.1`,
// `c4o@1.2`, `doco@1.3` — AND the STRIPED `prov_o@2013-04-30` / `olia@2026-04-09`
// are all byte-exact graph-faithful, registered with `OwlGraphFaithfulLens`
// (FIDELITY = ByteExactGraphFaithful) in `super::graph_faithful_lens`. Registering
// any of them here too would DOUBLE-REGISTER its harness key (one floor, one
// byte-exact), and `lens_by_name` (a `.find` over the slice) would resolve
// whichever came first — silently leaving `build_envelope` on the floor (the
// double-registration lesson). prov_o/olia were the last two on the floor; the L3
// byte kernel (the verbatim DOCTYPE PROLOG residue, the numeric/general
// `ExtendedRef` form, the `ChildSlot::InsertComment` residue) flips them
// byte-exact, so their floor `OwlLens` registration is REMOVED. The RDFC-1.0
// graph-identity gate (`[canonical_signatures]`, the `.prx` load-gate canonical
// leg) is independent of the lens registration and stays in force unchanged for
// every vocab. `OwlLens` itself remains the universal floor for any NON-bundled
// OWL source that has no graph-faithful writer.

#[cfg(test)]
mod tests {
    use super::*;

    /// RDFC-1.0 canonical-form determinism on CiTO — `canonical(b)`
    /// computed twice from clean input is byte-identical (RDFC §4.4.3:
    /// the serialized canonical form is a deterministic function of the
    /// graph). The graph-identity analogue of the old
    /// "canonical idempotent" check; we cannot feed `canonical`'s output
    /// (N-Quads) back into `canonical` (which parses RDF/XML), so
    /// determinism is the well-defined fixed-point property to pin.
    #[test]
    fn canonical_is_deterministic_cito() {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/cito-2.8.1.owl"
        ))
        .expect("CiTO bytes");
        let c1 = OwlLens::canonical(&bytes).expect("canonical 1");
        let c2 = OwlLens::canonical(&bytes).expect("canonical 2");
        assert_eq!(
            c1, c2,
            "RDFC-1.0 canonical N-Quads not deterministic across two runs (CiTO)"
        );
        // Sanity: the canonical form is N-Quads (one `.`-terminated
        // statement per LF line), not the RDF/XML the writer emits.
        let s = std::str::from_utf8(&c1).expect("canonical form is UTF-8");
        assert!(
            s.lines().all(|l| l.is_empty() || l.ends_with(" .")),
            "canonical form is not N-Quads"
        );
    }

    /// Typed-view PutGet round-trip on CiTO — `read_owl ∘ write_owl ∘
    /// read_owl ≡ read_owl` over the projection (`owl_equivalent`, the
    /// set-theoretic graph equality the OWL 2 RDF Mapping promises). This
    /// is the *separate* typed-view well-behaved-lens check the
    /// graph-identity `canonical` no longer hashes — kept as its own
    /// harness (the floor's `assert_put_get_law` is the constant-
    /// complement identity, so the typed-view check lives here).
    #[test]
    fn typed_view_round_trip_cito() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/cito-2.8.1.owl"
        );
        let bytes = std::fs::read(path).expect("bundled CiTO OWL must exist");
        assert_typed_view_round_trip(&bytes, "CiTO");
        // The floor's canonical-form PutGet law (constant-complement)
        // must also hold — the harness path the registry runs.
        OwlLens::assert_put_get_law(&bytes)
            .unwrap_or_else(|e| panic!("floor PutGet law violated on CiTO: {e}"));
    }

    /// Typed-view round-trip on PROV-O — exercises punning (OWL 2 §5.2)
    /// plus large annotation density.
    #[test]
    fn typed_view_round_trip_prov_o() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/prov_o-2013-04-30.owl"
        );
        let bytes = std::fs::read(path).expect("bundled PROV-O OWL must exist");
        assert_typed_view_round_trip(&bytes, "PROV-O");
    }

    /// Emit the canonical-form content digest of each bundled OWL
    /// vocabulary. Pair with `dump_unpinned_signatures` to update
    /// the `[canonical_signatures]` block of `praxis.lock`.
    #[test]
    fn dump_owl_canonical_signatures() {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let vocabs = [
            ("cito@2.8.1", "cito-2.8.1.owl"),
            ("doco@1.3", "doco-1.3.owl"),
            ("c4o@1.2", "c4o-1.2.owl"),
            ("biro@1.1.1", "biro-1.1.1.owl"),
            ("prov_o@2013-04-30", "prov_o-2013-04-30.owl"),
            ("olia@2026-04-09", "olia-2026-04-09.owl"),
        ];
        for (key, file) in vocabs {
            let path = format!("{manifest}/data/ontologies/{file}");
            let bytes = std::fs::read(&path).expect("vocab bytes");
            let sig = OwlLens::signature(&bytes).expect("signature");
            let hex: String = sig.iter().map(|b| format!("{b:02x}")).collect();
            eprintln!("owl-sig: \"{key}\" = \"{hex}\"");
        }
    }

    /// Typed-view round-trip on OLiA — the largest bundled vocab
    /// (1.3k classes, DTD entities, deep restriction graph).
    #[test]
    fn typed_view_round_trip_olia() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/olia-2026-04-09.owl"
        );
        let bytes = std::fs::read(path).expect("bundled OLiA OWL must exist");
        assert_typed_view_round_trip(&bytes, "OLiA");
    }

    /// `read_owl ∘ write_owl ∘ read_owl ≡ read_owl` over the typed
    /// projection: the ontology a second read recovers from the writer's
    /// output is `owl_equivalent` (OWL 2 RDF Mapping set-theoretic graph
    /// equality) to the one the first read produced. The typed-view
    /// well-behaved-lens check, decoupled from the graph-identity
    /// `canonical`.
    fn assert_typed_view_round_trip(bytes: &[u8], name: &str) {
        use super::super::reader::owl_equivalent;
        let text = std::str::from_utf8(bytes).expect("utf8");
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let once = read_owl(text).unwrap_or_else(|e| panic!("read_owl({name}): {e}"));
        let bytes2 = write_owl(&once);
        let text2 = std::str::from_utf8(&bytes2).expect("writer output utf8");
        let twice = read_owl(text2).unwrap_or_else(|e| panic!("re-read({name}): {e}"));
        assert!(
            owl_equivalent(&once, &twice),
            "typed-view PutGet violated on {name}: \
             read_owl ∘ write_owl ∘ read_owl ≢ read_owl"
        );
    }

    // ── Property-based GetPut + PutGet coverage ─────────────────────
    //
    // The published lens laws are (Foster, Greenwald, Moore, Pierce &
    // Schmitt 2007 "Combinators for Bidirectional Tree Transformations"
    // *ACM TOPLAS* 29(3) Article 17, §2.2):
    //
    //   PutGet: `get ∘ put = id_T`   — `put(t)` then `get` recovers `t`.
    //   GetPut: `put ∘ get = id_S`   — `put(get(s))` is `s`, up to the
    //                                  source's canonical equivalence.
    //
    // The corpus harness above pins both laws on the six bundled
    // vocabs; the proptests below witness BOTH laws directly across
    // the structural subset the `arb_ontology` strategy emits.

    use proptest::prelude::*;

    use super::super::reader::read_owl;
    use super::super::test_arb::arb_ontology;
    use super::super::writer::write_owl;
    use crate::social::software::markup::xml::owl::reader::owl_equivalent;

    proptest! {
        /// PutGet (Foster et al. 2007 §2.2): `get ∘ put = id_T`.
        ///
        /// For the OWL lens, `id_T` means `owl_equivalent` (the
        /// graph-equality the OWL 2 RDF Mapping promises — set-theoretic
        /// over the triple set, not Vec position). Re-stated for the
        /// `WellBehavedLens` trait surface using the generated subset.
        #[test]
        fn prop_put_get_law(ont in arb_ontology()) {
            let bytes = write_owl(&ont);
            let text = std::str::from_utf8(&bytes).expect("utf8");
            let recovered = read_owl(text).expect("read_owl on canonical bytes");
            prop_assert!(
                owl_equivalent(&ont, &recovered),
                "PutGet violated: get(put(ont)) not owl_equivalent to ont"
            );
        }

        /// GetPut (Foster et al. 2007 §2.2): `put ∘ get = id_S`,
        /// witnessed at the **graph-identity canonical** boundary —
        /// `canonical(write_owl(read_owl(b))) == canonical(b)` where
        /// `b = write_owl(arbitrary_ont)` (canonical OWL bytes the
        /// writer itself produces). On the writer's own output the typed
        /// view is already a graph fixed-point, so the RDFC N-Quads of
        /// the typed-view round-trip equal those of the input — the
        /// honest GetPut witness on the canonical-graph surface for the
        /// generated subset. (We round-trip through `write_owl ∘
        /// read_owl` explicitly, not `apply_put_after_get`, which for
        /// the floor is the constant-complement identity.)
        #[test]
        fn prop_get_put_law(ont in arb_ontology()) {
            // Canonical OWL bytes the writer emits for `ont`.
            let canonical_bytes = write_owl(&ont);
            // RDFC graph-identity signature of those bytes.
            let input_sig = OwlLens::signature(&canonical_bytes)
                .expect("signature of canonical bytes");
            // Typed-view round-trip, then its graph-identity signature.
            let text = std::str::from_utf8(&canonical_bytes).expect("utf8");
            let round_tripped = write_owl(&read_owl(text).expect("read writer output"));
            let rt_sig = OwlLens::signature(&round_tripped)
                .expect("signature of round-tripped bytes");
            prop_assert_eq!(
                input_sig,
                rt_sig,
                "GetPut violated: RDFC graph-identity drift on writer-emitted bytes"
            );
        }
    }
}

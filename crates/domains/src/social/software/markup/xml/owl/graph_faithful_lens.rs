//! `OwlGraphFaithfulLens` — the byte-exact graph-faithful [`WellBehavedLens`]
//! for an OWL/RDF source, and the registration that flips `cito` off the
//! universal floor in the completeness meter.
//!
//! # The graph-faithful OWL vocabularies
//!
//! ALL SIX bundled OWL vocabularies are held to the strict byte-exact PutGet law
//! ([`RoundTripFidelity::ByteExactGraphFaithful`], Foster, Greenwald, Moore,
//! Pierce & Schmitt 2007 §3, Definition 3.2): each source regenerates from the typed
//! [`OwlOntology`] graph PLUS a content-addressed concrete-syntax complement
//! ([`OwlSyntaxComplement`] — the structured RDF/XML striping + the generic
//! DOCTYPE/namespace/white-space/attribute/entity/EOL residue), with NO stored
//! raw blob.
//!
//! - The **flat SPAR family — cito, biro, c4o, doco** — serialise every node a
//!   top-level `<rdf:Description>` (no parseType, no DOCTYPE, no comments, no
//!   numeric/general references). CiTO was the FIRST; biro/c4o/doco joined it.
//! - The **striped family — prov_o, olia** — nest inline resources
//!   (`parseType="Collection"`, `owl:Restriction`) AND carry the concrete syntax
//!   the flat form lacks: an internal-subset DOCTYPE (`<!DOCTYPE rdf:RDF [
//!   <!ENTITY …> ]>`), §4.1 numeric character references (`&#39;`), §4.1
//!   general-entity references (`&rdfs;seeAlso`), and interspersed §2.5 comments.
//!   The L3 byte kernel captures each as STRUCTURED concrete-syntax residue (the
//!   DOCTYPE verbatim PROLOG residue, the numeric/general `ExtendedRef` form, the
//!   `ChildSlot::InsertComment` residue), NOT a stored DOM, so the recursive
//!   node-block writer reconstructs both byte-for-byte.
//!
//! It is the OWL sibling of the WN-LMF [`WordNetLmfLens`] and the USLM
//! `UslmGraphFaithfulLens`:
//!
//! - **`get : &[u8] → OwlGraphFaithfulView`** — [`capture_owl_complement`]:
//!   parse the source, project the typed OWL graph AND the byte-affecting
//!   residue. Fails closed on malformed input, a non-flat RDF/XML
//!   serialization, or a structural-writer backbone divergence.
//! - **`put : &OwlGraphFaithfulView → Vec<u8>`** —
//!   [`reconstruct_owl_rdfxml_source`]: regenerate the element backbone from the
//!   structured striping, re-apply the complement, serialize byte-exact.
//! - **`canonical`** — the IDENTITY: a byte-exact lens's source IS its own
//!   canonical form (`put(get(b)) == b`), so there is no separate canonical
//!   normalization; the byte-exact harness path never calls it (it compares raw
//!   bytes via `assert_byte_exact_law` and signs the raw bytes,
//!   `[byte_exact_signatures]`). Provided only for trait totality.
//!
//! # What registering this lens does
//!
//! The completeness meter reads each source's DECLARED fidelity from its
//! registered lens's [`WellBehavedLens::FIDELITY`]. Registering
//! `OwlGraphFaithfulLens` for each bundled vocab — the flat `cito@2.8.1`,
//! `biro@1.1.1`, `c4o@1.2`, `doco@1.3` AND the striped `prov_o@2013-04-30`,
//! `olia@2026-04-09` (each REPLACING its floor `OwlLens` registration) — with
//! `FIDELITY = ByteExactGraphFaithful` declares it graph-faithful; the harness
//! MEASURES the achieved tier by running the byte-exact law against the
//! `[byte_exact_signatures]` pin (which, because `put(get(b)) == b`, equals the
//! raw-source `[hashes]` pin). No bundled OWL vocab remains on the floor.
//!
//! # The RDFC-1.0 graph-identity gate is untouched
//!
//! This is the **byte-exact** law (raw-bytes round-trip). The separate
//! RDFC-1.0 graph-identity gate ([`OwlLens::canonical`](super::lens::OwlLens) /
//! `[canonical_signatures]`, the load-gate canonical leg) is unchanged: the
//! `.prx` load path still re-derives the canonical N-Quads of CiTO's source
//! graph and checks them against `[canonical_signatures]`. The two laws bind
//! different identities and run independently.
//!
//! # Citations
//!
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** *ACM TOPLAS* 29(3)
//!   §3, Definition 3.2 — the strict byte-exact PutGet law.
//! - **Gandon & Schreiber (2014)** *RDF 1.1 XML Syntax* — the RDF/XML
//!   serialization the structured striping projects.
//!
//! [`OwlOntology`]: super::ontology::OwlOntology
//! [`WordNetLmfLens`]: crate::social::software::markup::xml::lmf::lens::WordNetLmfLens

#[allow(unused_imports)]
use alloc::{format, string::String, vec::Vec};

use super::ontology::OwlOntology;
use super::rdfxml_writer::{
    OwlReconstructError, OwlSyntaxComplement, capture_owl_complement, reconstruct_owl_rdfxml_source,
};
use crate::formal::meta::well_behaved_lens::{RoundTripFidelity, WellBehavedLens};

/// The OWL/RDF byte-exact graph-faithful lens: `bytes ↔ (OWL ontology +
/// structured RDF/XML complement)`. The first praxis OWL lens declaring
/// [`RoundTripFidelity::ByteExactGraphFaithful`].
#[derive(Debug)]
pub struct OwlGraphFaithfulLens;

/// The graph-faithful target: the typed [`OwlOntology`] graph paired with the
/// concrete-syntax [`OwlSyntaxComplement`] the byte-exact `put` re-applies.
/// `get` produces this pair; `put` consumes it to regenerate the source bytes.
///
/// No `PartialEq`/`Eq` derive — [`OwlOntology`] is not `Eq` (it carries the
/// proof/category machinery), and the [`WellBehavedLens`] trait places no
/// bounds on `Target`. The byte-exact law compares the regenerated BYTES, not
/// the view value.
#[derive(Debug, Clone)]
pub struct OwlGraphFaithfulView {
    /// The typed OWL ontology — the navigable reasoning graph.
    pub ontology: OwlOntology,
    /// The concrete-syntax residue the typed ontology does not carry (the
    /// RDF/XML striping + the generic DOCTYPE/namespace/white-space/attribute
    /// residue).
    pub complement: OwlSyntaxComplement,
}

/// Error from the OWL graph-faithful lens: a UTF-8 decode failure or an
/// [`OwlReconstructError`] from the capture/reconstruct pair.
#[derive(Debug)]
pub enum OwlGraphFaithfulLensError {
    /// The source bytes are not valid UTF-8 (OWL/RDF is XML text).
    NotUtf8(String),
    /// The graph-faithful capture or reconstruction failed.
    Reconstruct(OwlReconstructError),
}

impl core::fmt::Display for OwlGraphFaithfulLensError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotUtf8(e) => write!(f, "OWL/RDF source is not UTF-8: {e}"),
            Self::Reconstruct(e) => write!(f, "OWL/RDF graph-faithful round-trip: {e}"),
        }
    }
}

impl std::error::Error for OwlGraphFaithfulLensError {}

impl From<OwlReconstructError> for OwlGraphFaithfulLensError {
    fn from(e: OwlReconstructError) -> Self {
        Self::Reconstruct(e)
    }
}

impl WellBehavedLens for OwlGraphFaithfulLens {
    type Target = OwlGraphFaithfulView;
    type Error = OwlGraphFaithfulLensError;

    /// Every bundled OWL vocab's tier — held to the strict byte-exact PutGet law.
    const FIDELITY: RoundTripFidelity = RoundTripFidelity::ByteExactGraphFaithful;

    /// `get` — capture the typed graph AND the concrete-syntax complement from
    /// the source ([`capture_owl_complement`]).
    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        let text = core::str::from_utf8(bytes)
            .map_err(|e| OwlGraphFaithfulLensError::NotUtf8(format!("{e}")))?;
        // Strip an optional BOM (W3C XML 1.0 §F.1), as OwlLens::get does.
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let (ontology, complement) = capture_owl_complement(text)?;
        Ok(OwlGraphFaithfulView {
            ontology,
            complement,
        })
    }

    /// `put` — regenerate the source bytes from the structured complement
    /// ([`reconstruct_owl_rdfxml_source`]), NO stored raw blob.
    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(reconstruct_owl_rdfxml_source(&target.complement)?)
    }

    /// `canonical` — the IDENTITY for a byte-exact lens: the source is its own
    /// canonical form (`put(get(b)) == b`). The byte-exact harness path never
    /// calls this; it is here only for trait totality.
    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        Ok(bytes.to_vec())
    }
}

// =============================================================================
// Harness registration — flips EVERY bundled OWL vocab to graph-faithful.
//
// Binds `OwlGraphFaithfulLens` to each bundled vocab, REPLACING its floor
// `OwlLens` registration:
//   • the FLAT SPAR family — `cito@2.8.1` (the first), `biro@1.1.1`, `c4o@1.2`,
//     `doco@1.3` — which serialise every node (named and blank) as a top-level
//     `<rdf:Description>` with leaf property elements (no `parseType`, no DOCTYPE,
//     no comments, no numeric/general references), so the structural writer
//     regenerates each byte-for-byte;
//   • the STRIPED family — `prov_o@2013-04-30`, `olia@2026-04-09` — which nest
//     inline resources AND carry the concrete syntax the flat form lacks (an
//     internal-subset DOCTYPE, §4.1 numeric `&#39;` and general-entity `&rdfs;`
//     references, interspersed §2.5 comments). The L3 byte kernel captures each
//     as STRUCTURED concrete-syntax residue (the verbatim DOCTYPE PROLOG residue,
//     the numeric/general `ExtendedRef` form, the `ChildSlot::InsertComment`
//     residue), NOT a stored DOM, so the recursive node-block writer reconstructs
//     both byte-for-byte.
//
// The harness runs the byte-exact law (FIDELITY is ByteExactGraphFaithful) and
// verifies each raw-bytes signature against `[byte_exact_signatures]` in
// praxis.lock. No bundled OWL vocab remains on the floor.
//
// Native only — linkme's distributed slice is unsupported on wasm32, mirroring
// every other `register_lens!`.
// =============================================================================

crate::register_lens!(
    CITO_GRAPH_FAITHFUL_LENS,
    "cito",
    "2.8.1",
    OwlGraphFaithfulLens
);

crate::register_lens!(
    BIRO_GRAPH_FAITHFUL_LENS,
    "biro",
    "1.1.1",
    OwlGraphFaithfulLens
);

crate::register_lens!(C4O_GRAPH_FAITHFUL_LENS, "c4o", "1.2", OwlGraphFaithfulLens);

crate::register_lens!(
    DOCO_GRAPH_FAITHFUL_LENS,
    "doco",
    "1.3",
    OwlGraphFaithfulLens
);

// The STRIPED OWL vocabs — prov_o + olia. These were blocked, in the prior
// slice, BELOW the writer layer by parser-level concrete syntax: an
// internal-subset DOCTYPE (`<!DOCTYPE rdf:RDF [ <!ENTITY …> ]>`), §4.1 numeric
// character references (`&#39;`), §4.1 general-entity references
// (`&rdfs;seeAlso`), and interspersed §2.5 comments. The L3 byte kernel captures
// each as STRUCTURED concrete-syntax residue (the DOCTYPE verbatim PROLOG
// residue, the numeric/general `ExtendedRef` form, the `ChildSlot::InsertComment`
// residue), so the recursive node-block writer reconstructs each byte-for-byte —
// NO stored DOM, NO raw blob. Registering them graph-faithful REPLACES each one's
// floor `OwlLens` registration (removed in `super::lens`).

crate::register_lens!(
    PROV_O_GRAPH_FAITHFUL_LENS,
    "prov_o",
    "2013-04-30",
    OwlGraphFaithfulLens
);

crate::register_lens!(
    OLIA_GRAPH_FAITHFUL_LENS,
    "olia",
    "2026-04-09",
    OwlGraphFaithfulLens
);

#[cfg(test)]
mod tests {
    use super::*;

    /// The lens's byte-exact PutGet law holds on the REAL bundled CiTO:
    /// `put(get(b)) == b` byte-for-byte — the law the harness runs for `cito`.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn owl_graph_faithful_lens_is_byte_exact_on_cito() {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/cito-2.8.1.owl"
        ))
        .expect("bundled CiTO must exist");
        OwlGraphFaithfulLens::assert_byte_exact_law(&bytes)
            .expect("OWL graph-faithful lens must satisfy the byte-exact PutGet law on CiTO");
    }

    /// The lens declares the graph-faithful tier — the const the completeness
    /// meter reads to flip `cito` off the floor.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn owl_graph_faithful_lens_declares_graph_faithful() {
        assert_eq!(
            OwlGraphFaithfulLens::FIDELITY,
            RoundTripFidelity::ByteExactGraphFaithful,
            "CiTO is praxis's first graph-faithful OWL source"
        );
    }

    /// `canonical` is the identity (a byte-exact lens's source is its own
    /// canonical form) — provided only for trait totality.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn owl_graph_faithful_canonical_is_identity() {
        let sample = b"<x/>";
        let c = OwlGraphFaithfulLens::canonical(sample).expect("canonical");
        assert_eq!(c, sample, "byte-exact lens canonical is identity");
    }

    /// Each flat SPAR vocab's `OwlGraphFaithfulLens` registration is LIVE in this
    /// binary and resolves to the byte-exact tier — `cito@2.8.1`, `biro@1.1.1`,
    /// `c4o@1.2`, `doco@1.3`. This both proves the flip (the completeness meter +
    /// `build_envelope` read the SAME registry) AND keeps the `register_lens!`
    /// statics from being linker-GC'd out of the lib-test binary (a `linkme`
    /// distributed-slice entry needs a live reference to survive `--test`
    /// dead-code elimination, otherwise `build_envelope` would silently emit the
    /// FLOOR envelope for an unreferenced registration).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn flat_spar_owl_family_registered_graph_faithful() {
        use crate::formal::meta::well_behaved_lens::{LensRegistration, lens_by_name};
        // Touch the registration statics so the linker retains them in --test.
        let _live: &[&'static LensRegistration] = &[
            &CITO_GRAPH_FAITHFUL_LENS,
            &BIRO_GRAPH_FAITHFUL_LENS,
            &C4O_GRAPH_FAITHFUL_LENS,
            &DOCO_GRAPH_FAITHFUL_LENS,
        ];
        for key in ["cito@2.8.1", "biro@1.1.1", "c4o@1.2", "doco@1.3"] {
            let reg = lens_by_name(key)
                .unwrap_or_else(|| panic!("{key} must have a registered lens in this binary"));
            assert_eq!(
                reg.fidelity,
                RoundTripFidelity::ByteExactGraphFaithful,
                "{key} must be registered byte-exact graph-faithful"
            );
        }
    }

    /// The byte-exact PutGet law holds on the REAL bundled biro/c4o/doco —
    /// `put(get(b)) == b` byte-for-byte, the law the harness runs for each.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn owl_graph_faithful_lens_is_byte_exact_on_flat_spar_family() {
        for file in ["biro-1.1.1.owl", "c4o-1.2.owl", "doco-1.3.owl"] {
            let bytes = std::fs::read(format!(
                "{}/data/ontologies/{}",
                env!("CARGO_MANIFEST_DIR"),
                file
            ))
            .unwrap_or_else(|_| panic!("bundled {file} must exist"));
            OwlGraphFaithfulLens::assert_byte_exact_law(&bytes)
                .unwrap_or_else(|e| panic!("{file} must satisfy the byte-exact PutGet law: {e}"));
        }
    }
}

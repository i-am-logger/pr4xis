//! `WordNetLmfLens` — the [`WellBehavedLens`] binding the WN-LMF reader/writer
//! to the byte-exact graph-faithful round-trip, and the registration that flips
//! `english_wordnet` off the universal floor in the completeness meter.
//!
//! # The first graph-faithful lens
//!
//! Every other registered source today rides the universal FLOOR
//! ([`RoundTripFidelity::RawBytesComplementFloor`]) — byte-exact via a stored
//! constant complement. WN-LMF is praxis's FIRST source held to the strict
//! **byte-exact** PutGet law ([`RoundTripFidelity::ByteExactGraphFaithful`],
//! Foster, Greenwald, Moore, Pierce & Schmitt 2007 ACM TOPLAS 29(3) §3, Definition 3.2; the
//! equivalence-of-categories counit at byte identity, Mac Lane §IV.4): the
//! source regenerates from the typed [`WordNet`] ontology PLUS a content-
//! addressed concrete-syntax complement ([`WnSyntaxComplement`]), with NO stored
//! raw blob.
//!
//! - **`get : &[u8] → (WordNet, WnSyntaxComplement)`** —
//!   [`capture_wn_complement`]: parse the source through the WN-LMF grammar,
//!   yielding the typed ontology AND the byte-affecting residue (the §2.8
//!   `<!DOCTYPE>`, the root namespaces, the §2.4 inter-element white-space, the
//!   §3.1 intra-tag layout, the §4.6 entity-reference form, the source attribute
//!   sequences). Fails closed on malformed input or a structural-writer
//!   divergence.
//! - **`put : &(WordNet, WnSyntaxComplement) → Vec<u8>`** —
//!   [`reconstruct_wn_lmf_source`]: re-apply the complement to the structural
//!   writer's regenerated tree and serialize byte-exact.
//! - **`canonical`** — the IDENTITY: a byte-exact lens guarantees the source IS
//!   its own canonical form (`put(get(b)) == b`), so there is no separate
//!   canonical normalization. The byte-exact harness path never calls
//!   `canonical` anyway — it compares raw bytes via `assert_byte_exact_law` and
//!   signs the raw bytes (`[byte_exact_signatures]`); `canonical` is provided
//!   only to satisfy the trait totality.
//!
//! # What registering this lens does
//!
//! The completeness meter
//! (`crate::formal::meta::well_behaved_lens::completeness`) reads each
//! source's DECLARED fidelity from its registered lens's
//! [`WellBehavedLens::FIDELITY`]. Without a lens a source can only declare the
//! floor (no graph-faithful writer ⇒ no graph-faithful claim). Registering
//! `WordNetLmfLens` with `FIDELITY = ByteExactGraphFaithful` declares
//! `english_wordnet` graph-faithful and drops its `write_wordnet` gap; when the
//! corpus is provisioned on disk the harness MEASURES the achieved tier by
//! running the byte-exact law, and the anti-lie cross-check
//! (`crate::formal::meta::well_behaved_lens::completeness::declared_matches_achieved`)
//! confirms declared == achieved.
//!
//! # Citations
//!
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** — "Combinators for
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §3, Definition 3.2 (the strict
//!   byte-exact PutGet law).
//! - **Bray et al. (2008)** XML 1.0 Fifth Edition §2.8 / §3.1 / §4.6 — the
//!   concrete-syntax decisions the complement carries.
//! - **Mac Lane (1998)** *Categories for the Working Mathematician* §IV.4 — the
//!   equivalence-of-categories counit at byte identity.

#[allow(unused_imports)]
use alloc::{string::String, vec::Vec};

use super::ontology::WordNet;
use super::writer::{
    WnReconstructError, WnSyntaxComplement, capture_wn_complement, reconstruct_wn_lmf_source,
};
use crate::formal::meta::well_behaved_lens::{RoundTripFidelity, WellBehavedLens};

/// The WN-LMF byte-exact graph-faithful lens: `bytes ↔ (WordNet ontology +
/// concrete-syntax complement)`. The first praxis lens declaring
/// [`RoundTripFidelity::ByteExactGraphFaithful`].
#[derive(Debug)]
pub struct WordNetLmfLens;

/// The graph-faithful target: the typed [`WordNet`] ontology paired with the
/// concrete-syntax [`WnSyntaxComplement`] the byte-exact `put` re-applies.
/// `get` produces this pair; `put` consumes it to regenerate the source bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordNetLmfView {
    /// The typed WN-LMF lexicon ontology — the graph the source regenerates from.
    pub wn: WordNet,
    /// The concrete-syntax residue the typed ontology does not carry.
    pub complement: WnSyntaxComplement,
}

/// Error from the WN-LMF lens: a UTF-8 decode failure (the source is not valid
/// UTF-8 text) or a [`WnReconstructError`] from the capture/reconstruct pair.
#[derive(Debug)]
pub enum WordNetLmfLensError {
    /// The source bytes are not valid UTF-8 (WN-LMF is XML text).
    NotUtf8(String),
    /// The graph-faithful capture or reconstruction failed.
    Reconstruct(WnReconstructError),
}

impl core::fmt::Display for WordNetLmfLensError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotUtf8(e) => write!(f, "WN-LMF source is not UTF-8: {e}"),
            Self::Reconstruct(e) => write!(f, "WN-LMF graph-faithful round-trip: {e}"),
        }
    }
}

impl std::error::Error for WordNetLmfLensError {}

impl From<WnReconstructError> for WordNetLmfLensError {
    fn from(e: WnReconstructError) -> Self {
        Self::Reconstruct(e)
    }
}

impl WellBehavedLens for WordNetLmfLens {
    type Target = WordNetLmfView;
    type Error = WordNetLmfLensError;

    /// English's tier — held to the strict byte-exact PutGet law.
    const FIDELITY: RoundTripFidelity = RoundTripFidelity::ByteExactGraphFaithful;

    /// `get` — capture the typed ontology AND the concrete-syntax complement
    /// from the source ([`capture_wn_complement`]).
    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        let text = core::str::from_utf8(bytes)
            .map_err(|e| WordNetLmfLensError::NotUtf8(format!("{e}")))?;
        let (wn, complement) = capture_wn_complement(text)?;
        Ok(WordNetLmfView { wn, complement })
    }

    /// `put` — regenerate the source bytes from the graph + complement
    /// ([`reconstruct_wn_lmf_source`]), NO stored raw blob.
    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(reconstruct_wn_lmf_source(&target.wn, &target.complement)?)
    }

    /// `canonical` — the IDENTITY for a byte-exact lens: the source is its own
    /// canonical form (`put(get(b)) == b`). The byte-exact harness path never
    /// calls this (it compares raw bytes); it is here only for trait totality.
    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        Ok(bytes.to_vec())
    }
}

// =============================================================================
// Harness registration — flips `english_wordnet` to graph-faithful.
//
// Binds `WordNetLmfLens` to the registered `english_wordnet@2025` source. The
// harness runs the byte-exact law (because FIDELITY is ByteExactGraphFaithful)
// and verifies the raw-bytes signature against `[byte_exact_signatures]` in
// praxis.lock. The completeness meter reads the FIDELITY const to declare the
// source graph-faithful and drop its `write_wordnet` gap. Native only — linkme's
// distributed slice is unsupported on wasm32 (the harness is a native CI/audit
// tool), mirroring every other `register_lens!`.
// =============================================================================

crate::register_lens!(
    ENGLISH_WORDNET_LENS,
    "english_wordnet",
    "2025",
    WordNetLmfLens
);

// =============================================================================
// us_legal_lexicon@2026 — the SECOND WN-LMF graph-faithful source.
//
// The U.S. Federal Legal-Text Closed-Class Lexicon
// (crates/domains/data/legal-text/us_legal_lexicon.xml) is a small WN-LMF
// lexicon. It rides the IDENTICAL `WordNetLmfLens` — the capture/reconstruct
// pair is source-agnostic (generic WN-LMF concrete-syntax residue, now including
// the CHILD-ORDER permutation species), exactly as the multiple USLM titles
// share `UslmXmlLens` and the multiple XSD schemas share `XsdSchemaLens`.
// Registering it flips `us_legal_lexicon` off the universal floor: the
// completeness meter reads this lens's `FIDELITY = ByteExactGraphFaithful` to
// declare it graph-faithful, and `build_wordnet_envelope`'s registry gate then
// emits `graph = Some` / `raw = None`. Its source children are DTD-ordered (every
// `<LexicalEntry>` is `Lemma, Sense`, all entries precede all synsets), so the
// child-order residue is a no-op for it — but the generic species is what makes
// the claim sound for ANY WN-LMF child order.
crate::register_lens!(
    US_LEGAL_LEXICON_LENS,
    "us_legal_lexicon",
    "2026",
    WordNetLmfLens
);

// =============================================================================
// english_function_words@2026 — the THIRD WN-LMF graph-faithful source.
//
// The English closed-class / function-word lexicon
// (crates/domains/data/function-words/english.xml, the `ClosedClassLexicon`
// disjoint complement of the open-class english_wordnet — Quirk et al. 1985
// §2.34) is a small WN-LMF lexicon. It rides the IDENTICAL source-agnostic
// `WordNetLmfLens`, exactly as us_legal_lexicon does. Registering it flips
// english_function_words off the universal floor: `build_wordnet_envelope`'s
// registry gate emits `graph = Some` / `raw = None`, and the source earns a
// durable `[byte_exact_signatures]` identity pin — integrity parity with its
// nearest sibling us_legal_lexicon (audit 2026-06-12 FW-A). Its source children
// are DTD-ordered, so the child-order residue is a no-op — but the generic
// species keeps the claim sound for ANY WN-LMF child order.
crate::register_lens!(
    ENGLISH_FUNCTION_WORDS_LENS,
    "english_function_words",
    "2026",
    WordNetLmfLens
);

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal but full-shape WN-LMF lexicon mirroring the writer's fixture,
    /// in the REAL Open English WordNet shape (DOCTYPE, root `xmlns:dc`,
    /// two-space indentation) so the lens exercises every residue class.
    const SAMPLE: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE LexicalResource SYSTEM \"http://globalwordnet.github.io/schemas/WN-LMF-1.3.dtd\">\n\
<LexicalResource xmlns:dc=\"https://globalwordnet.github.io/schemas/dc/\">\n\
  <Lexicon id=\"test-en\" label=\"Test\" language=\"en\" email=\"x@y\" license=\"CC\" version=\"1.0\">\n\
    <LexicalEntry id=\"e-dog-n\">\n\
      <Lemma writtenForm=\"dog\" partOfSpeech=\"n\"/>\n\
      <Sense id=\"dog-n-01\" synset=\"s-dog\"/>\n\
    </LexicalEntry>\n\
    <Synset id=\"s-dog\" ili=\"i1\" partOfSpeech=\"n\">\n\
      <Definition>a domesticated canine</Definition>\n\
    </Synset>\n\
  </Lexicon>\n\
</LexicalResource>\n";

    /// The lens's byte-exact PutGet law holds on the sample: `put(get(b)) == b`
    /// byte-for-byte — the law the harness runs for `english_wordnet`.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn wordnet_lmf_lens_is_byte_exact() {
        WordNetLmfLens::assert_byte_exact_law(SAMPLE.as_bytes())
            .expect("WN-LMF lens must satisfy the byte-exact PutGet law on the sample");
    }

    /// The lens declares the graph-faithful tier — the const the completeness
    /// meter reads to flip `english_wordnet` off the floor.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn wordnet_lmf_lens_declares_graph_faithful() {
        assert_eq!(
            WordNetLmfLens::FIDELITY,
            RoundTripFidelity::ByteExactGraphFaithful,
            "WN-LMF is praxis's first graph-faithful source"
        );
    }

    /// `canonical` is the identity (a byte-exact lens's source is its own
    /// canonical form) — provided only for trait totality, never used by the
    /// byte-exact harness path.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn wordnet_lmf_canonical_is_identity() {
        let c = WordNetLmfLens::canonical(SAMPLE.as_bytes()).expect("canonical");
        assert_eq!(
            c,
            SAMPLE.as_bytes(),
            "byte-exact lens canonical is identity"
        );
    }
}

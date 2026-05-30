//! `OwlLens` — the well-behaved lens binding OWL/RDF bytes ⇆
//! [`OwlOntology`].
//!
//! Per Foster, Greenwald, Moore, Pierce & Schmitt 2007 §2.2 a
//! well-behaved lens is a `(get, put)` pair satisfying GetPut and
//! PutGet. For the OWL leg:
//!
//!   `get  : bytes → OwlOntology`         = [`read_owl`]
//!   `put  : OwlOntology → bytes`         = [`write_owl`]
//!   `canonical : bytes → bytes`           = `write_owl ∘ read_owl`
//!
//! The `Target` carries **no** byte side-channel (`complement: Vec<u8>`)
//! — the typed view IS the lens's target. PutGet (canonical-byte
//! equality of `canonical(put(get(b)))` and `canonical(b)`) holds iff
//! `read_owl ∘ write_owl ∘ read_owl ≡ read_owl` (categorical
//! idempotence) AND `write_owl` is deterministic. Phase 2 verified the
//! former across the six bundled vocabularies; the latter is enforced
//! by the writer's sort-and-canonicalize discipline.
//!
//! ## Citations
//!
//! - **Foster, J. N.; Greenwald, M. B.; Moore, J. T.; Pierce, B. C.;
//!   Schmitt, A. (2007)** — "Combinators for Bidirectional Tree
//!   Transformations", *ACM TOPLAS* 29(3) Article 17, §2.2.
//! - **Patel-Schneider, P. F. & Motik, B. (eds.) (2012)** — *OWL 2
//!   Web Ontology Language: Mapping to RDF Graphs (2nd ed.)*, W3C
//!   Recommendation 11 December 2012. The set-theoretic mapping
//!   `read_owl` inverts.

#[allow(unused_imports)]
use alloc::{format, string::String, vec::Vec};
use core::fmt;

use super::ontology::OwlOntology;
use super::reader::{OwlReadError, read_owl};
use super::writer::write_owl;
use crate::formal::meta::well_behaved_lens::WellBehavedLens;

/// The well-behaved lens binding OWL/RDF bytes ⇆ [`OwlOntology`].
///
/// `Target = OwlOntology` — no byte side-channel. `canonical(bytes)`
/// is the lens output of `read_owl(bytes)`, so a PutGet violation
/// surfaces as canonical-form drift between `write_owl(read_owl(b))`
/// and `write_owl(read_owl(write_owl(read_owl(b))))`, exactly the
/// signal Phase 2's categorical round-trip test verified.
pub struct OwlLens;

#[derive(Debug)]
pub enum OwlLensError {
    /// UTF-8 decoding of the input bytes failed.
    NotUtf8(String),
    /// The OWL reader rejected the document.
    Read(OwlReadError),
}

impl fmt::Display for OwlLensError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUtf8(m) => write!(f, "owl lens: not valid UTF-8: {m}"),
            Self::Read(e) => write!(f, "owl lens: read error: {e}"),
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
        // The canonical form is the lens-output bytes of `get(bytes)`.
        // PutGet then reduces to `read_owl ∘ write_owl ∘ read_owl ≡
        // read_owl` (categorical idempotence) AND `write_owl`
        // deterministic — both verified by Phase 2.
        let target = Self::get(bytes)?;
        Ok(write_owl(&target))
    }
}

// =============================================================================
// Round-trip harness registrations — one entry per bundled OWL source.
// =============================================================================

crate::register_lens!(CITO_LENS, "cito", "2.8.1", OwlLens);
crate::register_lens!(DOCO_LENS, "doco", "1.3", OwlLens);
crate::register_lens!(C4O_LENS, "c4o", "1.2", OwlLens);
crate::register_lens!(BIRO_LENS, "biro", "1.1.1", OwlLens);
crate::register_lens!(PROV_O_LENS, "prov_o", "2013-04-30", OwlLens);
crate::register_lens!(OLIA_LENS, "olia", "2026-04-09", OwlLens);

#[cfg(test)]
mod tests {
    use super::*;

    /// `canonical ∘ canonical = canonical` — the PutGet law in its
    /// most direct form. Diagnoses where the writer's output isn't
    /// fully canonical.
    #[test]
    fn canonical_is_idempotent_cito() {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/cito-2.8.1.owl"
        ))
        .expect("CiTO bytes");
        let c1 = OwlLens::canonical(&bytes).expect("canonical 1");
        let c2 = OwlLens::canonical(&c1).expect("canonical 2");
        if c1 != c2 {
            // Find first diff and emit a tight window around it.
            for (i, (a, b)) in c1.iter().zip(c2.iter()).enumerate() {
                if a != b {
                    let start = i.saturating_sub(60);
                    let end = (i + 120).min(c1.len()).min(c2.len());
                    let s1 = core::str::from_utf8(&c1[start..end])
                        .unwrap_or("[non-utf8]")
                        .to_string();
                    let s2 = core::str::from_utf8(&c2[start..end])
                        .unwrap_or("[non-utf8]")
                        .to_string();
                    panic!(
                        "canonical not idempotent (CiTO) — first diff at byte {i}\n  c1: {s1:?}\n  c2: {s2:?}"
                    );
                }
            }
            panic!(
                "canonical not idempotent (CiTO), lengths {} vs {}",
                c1.len(),
                c2.len()
            );
        }
    }

    /// PutGet law on the SPAR CiTO vocabulary — the canonical form of
    /// `write_owl(get(bytes))` matches the canonical form of `bytes`.
    #[test]
    fn put_get_law_cito() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/cito-2.8.1.owl"
        );
        let bytes = std::fs::read(path).expect("bundled CiTO OWL must exist");
        OwlLens::assert_put_get_law(&bytes)
            .unwrap_or_else(|e| panic!("PutGet law violated on CiTO: {e}"));
    }

    /// PutGet law on PROV-O — exercises punning (OWL 2 §5.2) plus
    /// large annotation density.
    #[test]
    fn put_get_law_prov_o() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/prov_o-2013-04-30.owl"
        );
        let bytes = std::fs::read(path).expect("bundled PROV-O OWL must exist");
        OwlLens::assert_put_get_law(&bytes)
            .unwrap_or_else(|e| panic!("PutGet law violated on PROV-O: {e}"));
    }

    /// Emit the canonical-form SHA-256 of each bundled OWL
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

    /// PutGet law on OLiA — exercises the largest bundled vocab
    /// (1.3k classes, DTD entities, deep restriction graph).
    #[test]
    fn put_get_law_olia() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/olia-2026-04-09.owl"
        );
        let bytes = std::fs::read(path).expect("bundled OLiA OWL must exist");
        OwlLens::assert_put_get_law(&bytes)
            .unwrap_or_else(|e| panic!("PutGet law violated on OLiA: {e}"));
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
        /// witnessed at the canonical-byte / SHA-256 boundary —
        /// `Sha256(canonical(put(get(b)))) == Sha256(canonical(b))`
        /// where `b = write_owl(arbitrary_ont)` (canonical OWL bytes
        /// the lens itself produces). This is the byte-level form of
        /// the PutGet law on the lens's canonical-byte surface and
        /// pins the lens's canonical-form idempotence over the
        /// generated subset.
        #[test]
        fn prop_get_put_law(ont in arb_ontology()) {
            // Canonical bytes the lens emits for `ont`.
            let canonical_bytes = write_owl(&ont);
            // SHA-256 of the canonical input.
            let input_sig = OwlLens::signature(&canonical_bytes)
                .expect("signature of canonical bytes");
            // get → put on those bytes, then SHA-256 of its canonical form.
            let round_tripped = OwlLens::apply_put_after_get(&canonical_bytes)
                .expect("apply_put_after_get on canonical bytes");
            let rt_sig = OwlLens::signature(&round_tripped)
                .expect("signature of round-tripped bytes");
            prop_assert_eq!(
                input_sig,
                rt_sig,
                "GetPut violated: canonical-form SHA-256 drift on lens-emitted bytes"
            );
        }
    }
}

//! Canonical-codec ontology — the deterministic DAG-CBOR encoding, the
//! content address taken over it, and the two behavioural guarantees the
//! `.prx` substrate rests on (round-trip fidelity, decode totality).
//!
//! This is the *first-class praxis ontology* the canonical codec realises:
//! rather than describe the codec in doc-comments and prove it with untagged
//! `assert!`s inside the runtime crate, praxis declares its concepts here and
//! proves its guarantees as runnable axioms (see [`super::axioms`], whose
//! `verify()` predicates exercise the real [`pr4xis_runtime::codec`]
//! functions). Content-addressing REQUIRES a single canonical encoding so that
//! identical data → identical bytes → the identical content address,
//! reproducibly across implementations and toolchains; DAG-CBOR is that form.
//!
//! # Literature
//!
//! - **Bormann & Hoffman (2020)** *Concise Binary Object Representation
//!   (CBOR)*, RFC 8949 §4.2 — deterministically encoded CBOR (sorted map
//!   keys, shortest-form integers, no indefinite-length items).
//! - **IPLD** *DAG-CBOR codec specification*
//!   (<https://ipld.io/specs/codecs/dag-cbor/>) — the strict CBOR profile
//!   the content address is computed over.
//! - **Merkle (1987)** *A Digital Signature Based on a Conventional
//!   Encryption Function*, CRYPTO '87 — the content address whose identity
//!   the encoding grounds.
//! - **Smith et al. (2005)** *Relations in biomedical ontologies* (OBO
//!   Relation Ontology), *Genome Biology* 6:R46 — the `depends on`
//!   (RO:0002502) relation the `ContentAddress → CanonicalEncoding` morphism
//!   is kinded by.
//! - **Bormann & Hoffman (2020)** RFC 8949 §5.3 (Validity of Items) and
//!   Appendix F (Well-Formedness Errors) — the well-formedness rules a decoder
//!   enforces to be fail-closed, grounding `DecodeTotality`.
//! - **Sassaman, Patterson, Bratus & Locasto (2011)** *Security Applications
//!   of Formal Language Theory* (Dartmouth TR2011-709) — LangSec: an input
//!   handler must be a full recognizer that rejects malformed input before
//!   acting, the defensive-parsing basis for the fail-closed decode.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "CanonicalCodec",
    source: "Bormann & Hoffman (2020) Concise Binary Object Representation (CBOR), RFC 8949 §3 (Specification of the CBOR Encoding), §4.2 (deterministically encoded CBOR), §5.3 (Validity of Items) & Appendix F (Well-Formedness Errors); IPLD DAG-CBOR codec specification (https://ipld.io/specs/codecs/dag-cbor/); Merkle (1987) A Digital Signature Based on a Conventional Encryption Function, CRYPTO '87; Sassaman, Patterson, Bratus & Locasto (2011) Security Applications of Formal Language Theory, Dartmouth TR2011-709 (LangSec); Smith et al. (2005) Relations in biomedical ontologies (OBO Relation Ontology), Genome Biology 6:R46",

    concepts: [
        CanonicalEncoding,
        ContentAddress,
        CodecRoundTrip,
        DecodeTotality,
    ],

    labels: {
        CanonicalEncoding: ("en", "Canonical encoding",
            "Bormann & Hoffman (2020) RFC 8949 §4.2; IPLD DAG-CBOR: the single deterministic byte encoding of a value — sorted map keys, shortest-form integers, no indefinite-length items — so that equal values yield equal bytes across implementations and toolchains."),
        ContentAddress: ("en", "Content address",
            "Merkle (1987): the cryptographic-hash identity of a value, computed OVER its canonical encoding; identical data yield the identical address. It depends on the canonical encoding — a drift in the encoding is a drift in the address."),
        CodecRoundTrip: ("en", "Codec round-trip",
            "IPLD DAG-CBOR: the total inverse law of a serialization codec (a section/retract isomorphism, decode ∘ encode = id — NOT a lens law: one type, no distinct source/view), so no information is lost across the encode/decode boundary the runtime loads ontologies through."),
        DecodeTotality: ("en", "Decode totality",
            "Bormann & Hoffman (2020) RFC 8949 §5.3 (Validity of Items) / Appendix F (Well-Formedness Errors); LangSec input-recognizer robustness (Sassaman, Patterson, Bratus & Locasto 2011): decode is a TOTAL, fail-closed recognizer on arbitrary bytes — an adversarial input (e.g. a length prefix declaring 2^64-1 items) is REFUSED with a typed error, never an unbounded allocation, OOM, or panic."),
    },

    // The one relation that matters here: the content address is DERIVED FROM
    // the canonical encoding, so it depends on it (OBO-RO `depends on`,
    // RO:0002502; Smith et al. 2005). This is a traversable, kinded morphism —
    // `ContentAddress -[Dependency]-> CanonicalEncoding` — NOT a law: it is the
    // edge a future `reachable_subgraph(MerkleRoot) -> CanonicalEncoding`
    // traversal walks. It is deliberately NOT expressed as an
    // `address_of(v) == ContentAddress::of(canonical_encode(v))` axiom, which
    // would be tautological: `address_of` is DEFINED as that composition
    // (codec.rs:62-63), so asserting the equality re-runs the body and proves
    // only `f(x) == f(x)`.
    edges: [
        (ContentAddress, CanonicalEncoding, Dependency),
    ],
}

/// Quality: a short symbolic description of each canonical-codec concept,
/// matching the citation column in the ontology header.
#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = CanonicalCodecConcept;
    type Value = &'static str;

    fn get(&self, c: &CanonicalCodecConcept) -> Option<&'static str> {
        use CanonicalCodecConcept as C;
        Some(match c {
            C::CanonicalEncoding => {
                "deterministic DAG-CBOR bytes; equal values → equal bytes (RFC 8949 §4.2)"
            }
            C::ContentAddress => "hash identity computed over the canonical encoding (Merkle 1987)",
            C::CodecRoundTrip => {
                "decode(encode(v)) == v — a total inverse pair; the codec loses nothing (IPLD DAG-CBOR)"
            }
            C::DecodeTotality => {
                "decode is total/fail-closed: adversarial input is refused, never OOM/panic"
            }
        })
    }
}

impl Ontology for CanonicalCodecOntology {
    type Cat = CanonicalCodecCategory;
    type Qual = ConceptDescription;

    fn axioms() -> alloc::vec::Vec<alloc::boxed::Box<dyn Axiom>> {
        // The canonical codec ([`pr4xis_runtime::codec`]) is UNCONDITIONALLY
        // linked (domains/Cargo.toml), so these predicates run against the real
        // machinery in every build — no `feature = "prx"` gate (contrast
        // `super::ontology_archive`, whose realisation lives behind `.prx`).
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        use super::axioms::{
            CanonicalEncodingDeterministic, CodecRoundTrip, DecodeRefusesAdversarialLength,
        };
        axioms.push(alloc::boxed::Box::new(CanonicalEncodingDeterministic));
        axioms.push(alloc::boxed::Box::new(CodecRoundTrip));
        axioms.push(alloc::boxed::Box::new(DecodeRefusesAdversarialLength));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::ontology::registry::{axiom_by_name, describe_knowledge_base};

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<CanonicalCodecCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        // Runs the category laws AND all three codec axioms' `verify()`
        // against the real `pr4xis_runtime::codec`.
        CanonicalCodecOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn four_concepts() {
        assert_eq!(CanonicalCodecConcept::variants().len(), 4);
    }

    /// The machinery is reasoned about through the SAME registry as any
    /// statute: the ontology is discoverable in `VOCABULARIES` and each of its
    /// three axioms re-binds by name through `axiom_by_name` (the load-time
    /// rebind gate). If the codec ontology were opaque runtime code, neither
    /// would resolve — this is what "self-describing" buys.
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn discoverable_via_self_model() {
        // The `ontology!` macro registers the vocabulary under the ontology
        // type's name (`<name>Ontology`) — the same convention every other
        // registered ontology follows (e.g. `SelfModelOntology`,
        // `KnowledgeOntology`).
        assert!(
            describe_knowledge_base()
                .iter()
                .any(|v| v.name() == "CanonicalCodecOntology"),
            "CanonicalCodec must be discoverable in the ontology registry"
        );
        for axiom in [
            "CanonicalEncodingDeterministic",
            "CodecRoundTrip",
            "DecodeRefusesAdversarialLength",
        ] {
            assert!(
                axiom_by_name(axiom).is_some(),
                "codec axiom {axiom} must re-bind through the registry (axiom_by_name)"
            );
        }
    }
}

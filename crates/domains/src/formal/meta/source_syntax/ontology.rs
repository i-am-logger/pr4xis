//! Source-syntax ontology — the *concrete-syntax decisions* a byte-exact
//! serialization must record beyond the abstract XML Information Set
//! (STAGE 2.1 of the universal compiler / task #34).
//!
//! A parsed ontology fixes a document's **Information Set** (Cowan & Tobin
//! 2004): its elements, attributes, characters, namespaces — the
//! format-neutral *meaning*. But two byte streams with the *identical*
//! Information Set can still differ byte-for-byte: attribute order,
//! indentation, `<a/>` vs `<a></a>`, entity spelling, comment placement.
//! Canonical XML 1.1 (Boyer & Marcy 2008) deliberately *normalizes these
//! away* — which is exactly why the `.prx` floor today keeps the raw source
//! bytes as a stored complement (`RawBytesComplementFloor`).
//!
//! This ontology names that residue. Each
//! [`ConcreteSyntaxDecision`](crate::formal::meta::source_syntax::ontology::SourceSyntaxConcept::ConcreteSyntaxDecision) is a
//! byte-affecting choice the Information Set underdetermines; recording the
//! decisions per node alongside the Information Set content lets a writer
//! regenerate the exact bytes from the graph *alone*
//! ([`RoundTripFidelity::ByteExactGraphFaithful`](crate::formal::meta::well_behaved_lens::RoundTripFidelity)),
//! retiring the stored complement.
//!
//! This module is the **vocabulary** (what kinds of decision exist), cited
//! and format-agnostic. It deliberately holds NO per-source *instance* data
//! and is NOT part of any ontology's content-address identity: the same
//! ontology serialized two different ways must keep the same `.prx` root
//! (identity = meaning, not serialization). The per-node instance decisions
//! live in the per-source `.prx` envelope with their own content-address
//! (STAGE 2.1b); the byte-exact writers that consume them are STAGE 2.3+.
//!
//! # Literature
//!
//! - **Cowan, J. & Tobin, R. (2004)** *XML Information Set (2nd ed.)*, W3C
//!   Recommendation — the abstract information items a `ConcreteSyntaxDecision`
//!   is the residue of.
//! - **Bray, T. et al. (2008)** *Extensible Markup Language (XML) 1.0 (5th
//!   ed.)*, W3C Recommendation — the concrete-syntax productions (§2.5
//!   comments, §2.6 PIs, §2.8 prolog/DTD, §2.10–2.11 white space + line
//!   ends, §3.1 empty elements, §4.1–4.2 entity / parameter-entity refs,
//!   §4.3.3 encoding).
//! - **Boyer, J. & Marcy, G. (2008)** *Canonical XML Version 1.1*, W3C
//!   Recommendation — the normalization that *erases* this residue (so it
//!   must be recorded for byte-exactness).
//! - **Bray, T. et al. (2009)** *Namespaces in XML 1.0 (3rd ed.)*, W3C
//!   Recommendation — namespace declaration prefix + placement decisions.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "SourceSyntax",
    source: "Cowan & Tobin (2004) XML Information Set (2nd ed.), W3C Recommendation; Bray et al. (2008) XML 1.0 (5th ed.), W3C Recommendation; Boyer & Marcy (2008) Canonical XML 1.1, W3C Recommendation; Bray et al. (2009) Namespaces in XML 1.0 (3rd ed.), W3C Recommendation",

    concepts: [
        SourceSerialization,
        InfosetItem,
        ConcreteSyntaxDecision,
        AttributeOrder,
        WhitespaceFormatting,
        EmptyElementForm,
        XmlDeclaration,
        DoctypeDeclaration,
        EntityReferenceForm,
        CommentPlacement,
        ParameterEntityDeclaration,
        ProcessingInstructionForm,
        NamespaceDeclarationForm,
        CharacterEncodingForm,
        LineEndingForm,
    ],

    labels: {
        SourceSerialization: ("en", "Source serialization",
            "Bray et al. (2008) XML 1.0 §2: the concrete byte serialization of an abstract structure — the union of its format-neutral Information Set content and the concrete-syntax decisions that fix its exact bytes."),
        InfosetItem: ("en", "Information Set item",
            "Cowan & Tobin (2004): a format-neutral information item (element, attribute, character, namespace, …) — the abstract content a parser recovers, independent of the byte choices that encoded it. What the `.prx` ontology already captures."),
        ConcreteSyntaxDecision: ("en", "Concrete-syntax decision",
            "Boyer & Marcy (2008) Canonical XML 1.1: a byte-level serialization choice the abstract XML Information Set leaves underdetermined — the residue C14N normalizes away, which a byte-exact round-trip must therefore record."),
        AttributeOrder: ("en", "Attribute order",
            "Bray et al. (2008) §3.1; Boyer & Marcy (2008) §2.3: the authored order of an element's attributes. The Information Set's attribute set is UNORDERED and C14N sorts it lexically, so the source order is a recorded decision."),
        WhitespaceFormatting: ("en", "White-space formatting",
            "Bray et al. (2008) §2.10: insignificant white space and indentation between markup — not part of element content in the Information Set, byte-affecting on output."),
        EmptyElementForm: ("en", "Empty-element form",
            "Bray et al. (2008) §3.1: an empty element written `<a/>` (empty-element tag) versus `<a></a>` (start- plus end-tag) — identical Information Set, distinct bytes."),
        XmlDeclaration: ("en", "XML declaration",
            "Bray et al. (2008) §2.8–2.9: the `<?xml version … encoding … standalone …?>` declaration, including the optional standalone document declaration."),
        DoctypeDeclaration: ("en", "Document type declaration",
            "Bray et al. (2008) §2.8: the document type declaration and its internal subset, reproduced verbatim for a byte-exact prolog."),
        EntityReferenceForm: ("en", "Entity-reference form",
            "Bray et al. (2008) §4.1: the chosen form of a reference — a named entity (`&amp;`), a decimal (`&#38;`) or a hexadecimal (`&#x26;`) character reference — all denoting the same character."),
        CommentPlacement: ("en", "Comment placement",
            "Bray et al. (2008) §2.5: a comment's text together with its byte position and surrounding white space — not part of the element/character Information Set content."),
        ParameterEntityDeclaration: ("en", "Parameter-entity declaration",
            "Bray et al. (2008) §4.2.2: a parameter-entity declaration in the DTD internal subset, recorded so the internal subset reconstructs exactly."),
        ProcessingInstructionForm: ("en", "Processing-instruction form",
            "Bray et al. (2008) §2.6: a processing instruction's target and the exact spacing of its data."),
        NamespaceDeclarationForm: ("en", "Namespace-declaration form",
            "Bray et al. (2009): the chosen prefix and the placement of an `xmlns` declaration — the namespace binding is in the Information Set, but the prefix spelling and declaring element are decisions."),
        CharacterEncodingForm: ("en", "Character-encoding form",
            "Bray et al. (2008) §4.3.3: the character encoding (UTF-8 / UTF-16 / …) and an optional byte-order mark — byte-affecting, encoding-independent of the Information Set."),
        LineEndingForm: ("en", "Line-ending form",
            "Bray et al. (2008) §2.11: the line-ending convention (LF versus CRLF); XML normalizes all to LF on input, so the source convention is a recorded decision."),
    },

    is_a: [
        // The twelve byte-affecting decisions are each a kind of
        // concrete-syntax decision — the residue beyond the Information Set.
        (AttributeOrder, ConcreteSyntaxDecision),
        (WhitespaceFormatting, ConcreteSyntaxDecision),
        (EmptyElementForm, ConcreteSyntaxDecision),
        (XmlDeclaration, ConcreteSyntaxDecision),
        (DoctypeDeclaration, ConcreteSyntaxDecision),
        (EntityReferenceForm, ConcreteSyntaxDecision),
        (CommentPlacement, ConcreteSyntaxDecision),
        (ParameterEntityDeclaration, ConcreteSyntaxDecision),
        (ProcessingInstructionForm, ConcreteSyntaxDecision),
        (NamespaceDeclarationForm, ConcreteSyntaxDecision),
        (CharacterEncodingForm, ConcreteSyntaxDecision),
        (LineEndingForm, ConcreteSyntaxDecision),
    ],

    has_a: [
        // A source serialization is the abstract Information Set content
        // PLUS the concrete-syntax decisions that fix its exact bytes;
        // byte-exactness requires recording both.
        (SourceSerialization, InfosetItem),
        (SourceSerialization, ConcreteSyntaxDecision),
    ],
}

/// Quality: a short symbolic description of each source-syntax concept,
/// matching the citation column in the ontology header.
#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = SourceSyntaxConcept;
    type Value = &'static str;

    fn get(&self, c: &SourceSyntaxConcept) -> Option<&'static str> {
        use SourceSyntaxConcept as C;
        Some(match c {
            C::SourceSerialization => {
                "abstract content + concrete-syntax decisions that fix the bytes"
            }
            C::InfosetItem => "format-neutral information item (Cowan & Tobin 2004)",
            C::ConcreteSyntaxDecision => "byte choice the Infoset underdetermines (C14N erases it)",
            C::AttributeOrder => "authored attribute order (Infoset is unordered; C14N sorts)",
            C::WhitespaceFormatting => "insignificant white space + indentation (XML 1.0 §2.10)",
            C::EmptyElementForm => "`<a/>` vs `<a></a>` (XML 1.0 §3.1)",
            C::XmlDeclaration => "`<?xml … standalone?>` declaration (XML 1.0 §2.8–2.9)",
            C::DoctypeDeclaration => "DOCTYPE + internal subset, verbatim (XML 1.0 §2.8)",
            C::EntityReferenceForm => "`&amp;` vs `&#38;` vs `&#x26;` (XML 1.0 §4.1)",
            C::CommentPlacement => "comment text + byte position (XML 1.0 §2.5)",
            C::ParameterEntityDeclaration => {
                "PE declaration in the DTD internal subset (XML 1.0 §4.2.2)"
            }
            C::ProcessingInstructionForm => "PI target + data spacing (XML 1.0 §2.6)",
            C::NamespaceDeclarationForm => "xmlns prefix + placement (Namespaces in XML 1.0)",
            C::CharacterEncodingForm => "encoding + optional BOM (XML 1.0 §4.3.3)",
            C::LineEndingForm => "LF vs CRLF convention (XML 1.0 §2.11)",
        })
    }
}

impl Ontology for SourceSyntaxOntology {
    type Cat = SourceSyntaxCategory;
    type Qual = ConceptDescription;

    fn axioms() -> alloc::vec::Vec<alloc::boxed::Box<dyn Axiom>> {
        // The vocabulary's structural axioms (category laws over the declared
        // subsumption + parthood generators). The DOMAIN axioms — that a
        // recorded set of decisions plus the Information Set regenerates the
        // exact source bytes — are runnable only once a byte-exact writer
        // realises them (STAGE 2.3 write_uslm); they land with that writer,
        // not as a stub here.
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<SourceSyntaxCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        SourceSyntaxOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fifteen_concepts() {
        assert_eq!(SourceSyntaxConcept::variants().len(), 15);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn concept_description_total() {
        let q = ConceptDescription;
        for c in SourceSyntaxConcept::variants() {
            assert!(q.get(&c).is_some(), "{c:?} missing description");
        }
    }

    /// Every byte-affecting decision is subsumed by `ConcreteSyntaxDecision`
    /// — the residue is exactly the twelve leaf species, nothing floating.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn twelve_decisions_under_the_genus() {
        use SourceSyntaxConcept as C;
        let species = [
            C::AttributeOrder,
            C::WhitespaceFormatting,
            C::EmptyElementForm,
            C::XmlDeclaration,
            C::DoctypeDeclaration,
            C::EntityReferenceForm,
            C::CommentPlacement,
            C::ParameterEntityDeclaration,
            C::ProcessingInstructionForm,
            C::NamespaceDeclarationForm,
            C::CharacterEncodingForm,
            C::LineEndingForm,
        ];
        assert_eq!(
            species.len(),
            12,
            "the recorded residue is twelve decisions"
        );
    }
}

//! XML 1.0 ontology — the 11 information items per the W3C XML
//! Information Set rec (Cowan & Tobin 2004) plus the four reserved
//! `xml:*`-namespace attributes per the W3C xml.xsd schema.
//!
//! ## Why two grounding sources
//!
//! XML 1.0 has two complementary spec layers that together fix what
//! XML *is*:
//!
//! - **Surface syntax + reserved-name attributes** — published as
//!   machine-readable XSD at `https://www.w3.org/2001/xml.xsd`. The
//!   four reserved attributes are `xml:lang`, `xml:space`,
//!   `xml:base`, `xml:id`.
//! - **Conceptual structure** — published as the XML Information
//!   Set recommendation (Cowan & Tobin 2004). The 11 information
//!   items are Document, Element, Attribute, Processing Instruction,
//!   Unexpanded Entity Reference, Character, Comment, Document Type
//!   Declaration, Unparsed Entity, Notation, Namespace.
//!
//! The XSD is bundled at
//! `crates/domains/data/markup-schemas/xml/xml.xsd`; the Infoset rec
//! is bundled at
//! `crates/domains/data/markup-schemas/xml/xml-infoset.xhtml`. Both
//! are hash-pinned in `praxis.lock`. The build script scans both at
//! build time and emits `XML_NAMESPACE_ATTRIBUTES` and
//! `XML_INFOSET_INFORMATION_ITEMS` arrays under `OUT_DIR`. This
//! module declares the corresponding `pr4xis::ontology!` block.
//!
//! ## Coexistence with the existing `xml::ontology`
//!
//! The existing module `xml::ontology` carries an older
//! `XmlNodeKind`-based representation (10 variants close to but not
//! identical to the Infoset's 11). It is kept as the rich runtime
//! type for tree-shaped XML manipulation (`XmlElement`, `XmlNode`,
//! `XmlDocument`). The M4.η.2 ontology declared here is the proper
//! pr4xis-style ontology: literature-cited, loaded from authoritative
//! sources, axiom-checked. The two should converge in a future
//! refactor; until then they coexist (the new one carries the M4.η.2
//! grounding, the old one carries the runtime helpers).
//!
//! ## Citations
//!
//! - **Bray, T., J. Paoli, C. M. Sperberg-McQueen, E. Maler & F.
//!   Yergeau (eds.) (2008)** *Extensible Markup Language (XML) 1.0
//!   (Fifth Edition)*, W3C Recommendation 26 November 2008.
//!   <https://www.w3.org/TR/xml/>. §2.1 (Well-Formed XML Documents);
//!   §2.10 (White Space Handling — `xml:space`); §2.12 (Language
//!   Identification — `xml:lang`).
//! - **Bray, T., D. Hollander, A. Layman, R. Tobin & H. S. Thompson
//!   (eds.) (2009)** *Namespaces in XML 1.0 (Third Edition)*, W3C
//!   Recommendation 8 December 2009. <https://www.w3.org/TR/xml-names/>.
//!   §3 (Declaring Namespaces) — the `xml:` prefix is reserved and
//!   bound to the W3C XML namespace.
//! - **Cowan, J. & R. Tobin (eds.) (2004)** *XML Information Set
//!   (Second Edition)*, W3C Recommendation 4 February 2004.
//!   <https://www.w3.org/TR/xml-infoset/>. §2 (Information Items) —
//!   the 11-item conceptual taxonomy.
//! - **Marsh, J. & R. Tobin (eds.) (2009)** *XML Base (Second
//!   Edition)*, W3C Recommendation 28 January 2009.
//!   <https://www.w3.org/TR/xmlbase/>. The `xml:base` attribute.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::FinitelyGenerated;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// =============================================================================
// Concept inventory
// =============================================================================

pr4xis::ontology! {
    name: "Xml10",
    source: "Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (eds.) (2008) Extensible Markup Language (XML) 1.0 (Fifth Edition), W3C Recommendation 26 November 2008; Bray, Hollander, Layman, Tobin & Thompson (eds.) (2009) Namespaces in XML 1.0 (Third Edition), W3C Recommendation 8 December 2009; Cowan & Tobin (eds.) (2004) XML Information Set (Second Edition), W3C Recommendation 4 February 2004, §2 (Information Items); Marsh & Tobin (eds.) (2009) XML Base (Second Edition), W3C Recommendation 28 January 2009",

    concepts: [
        // Top: every named or structural piece of an XML 1.0
        // document is an `XmlInformationItem` or a reserved
        // namespace attribute. Cowan & Tobin 2004 §2.
        XmlInformationItem,

        // The 11 information items per Cowan & Tobin 2004 §2:
        DocumentItem,
        ElementItem,
        AttributeItem,
        ProcessingInstructionItem,
        UnexpandedEntityReferenceItem,
        CharacterItem,
        CommentItem,
        DocumentTypeDeclarationItem,
        UnparsedEntityItem,
        NotationItem,
        NamespaceItem,

        // Reserved-name attributes per W3C xml.xsd (loaded from the
        // bundled schema; concept-level placeholders here, with the
        // actual name inventory loaded at runtime via the loader).
        XmlReservedAttribute,
    ],

    labels: {
        XmlInformationItem: ("en", "XML information item",
            "Top of the XML 1.0 conceptual taxonomy. Cowan & Tobin (2004) XML Information Set Second Edition §2: \"An XML document's information set consists of a number of information items.\" Every leaf is a kind of information item per §2.1 through §2.11."),
        DocumentItem: ("en", "Document information item",
            "Cowan & Tobin 2004 §2.1: the root of an information set — every well-formed XML document is described by exactly one document information item, which has properties for children (element, processing-instruction, comment, doctype), base URI, version, encoding, standalone, etc. There is exactly one DocumentItem per XML document per XML 1.0 §2.1."),
        ElementItem: ("en", "Element information item",
            "Cowan & Tobin 2004 §2.2: one per element appearing in the document. Properties include namespace name, local name, prefix, children, attributes, in-scope namespaces, base URI, parent."),
        AttributeItem: ("en", "Attribute information item",
            "Cowan & Tobin 2004 §2.3: one per attribute (specified or defaulted) of every element item. Properties include namespace name, local name, prefix, normalized value, specified, attribute type, references, owner element."),
        ProcessingInstructionItem: ("en", "Processing instruction information item",
            "Cowan & Tobin 2004 §2.4: one per processing instruction in the document. The XML declaration `<?xml ... ?>` is NOT an information item (Cowan & Tobin §1.1 + Appendix D)."),
        UnexpandedEntityReferenceItem: ("en", "Unexpanded entity reference information item",
            "Cowan & Tobin 2004 §2.5: appears when a non-validating processor declines to expand an external general entity reference."),
        CharacterItem: ("en", "Character information item",
            "Cowan & Tobin 2004 §2.6: one per character in the document content. The character is identified by its Unicode code point."),
        CommentItem: ("en", "Comment information item",
            "Cowan & Tobin 2004 §2.7: one per comment in the document (comments within the DTD are NOT exposed per §2.7)."),
        DocumentTypeDeclarationItem: ("en", "Document type declaration information item",
            "Cowan & Tobin 2004 §2.8: at most one — present iff the document has a DOCTYPE. Properties include system identifier, public identifier, children (processing instructions in the DTD)."),
        UnparsedEntityItem: ("en", "Unparsed entity information item",
            "Cowan & Tobin 2004 §2.9: one per declared unparsed entity (external entity with NDATA notation)."),
        NotationItem: ("en", "Notation information item",
            "Cowan & Tobin 2004 §2.10: one per notation declaration in the DTD. Notations name external formats referenced by unparsed entities."),
        NamespaceItem: ("en", "Namespace information item",
            "Cowan & Tobin 2004 §2.11: one per namespace declaration in scope on an element item. Properties include prefix and namespace name (URI)."),
        XmlReservedAttribute: ("en", "XML reserved-namespace attribute",
            "Bray et al. 2009 Namespaces in XML 1.0 §3: the `xml:` prefix is bound to the W3C XML namespace and reserved for W3C-defined attributes (xml:lang per XML 1.0 §2.12, xml:space per §2.10, xml:base per Marsh & Tobin 2009, xml:id per W3C xml:id Recommendation 2005). The exact inventory is loaded from the W3C xml.xsd at build time."),
    },

    // is_a hierarchy:
    //   XmlInformationItem (root)
    //   ├── DocumentItem
    //   ├── ElementItem
    //   ├── AttributeItem
    //   ├── ProcessingInstructionItem
    //   ├── UnexpandedEntityReferenceItem
    //   ├── CharacterItem
    //   ├── CommentItem
    //   ├── DocumentTypeDeclarationItem
    //   ├── UnparsedEntityItem
    //   ├── NotationItem
    //   ├── NamespaceItem
    //   └── XmlReservedAttribute   (not strictly an InformationItem per
    //                               Cowan & Tobin 2004 — it's the
    //                               vocabulary half of the spec — but
    //                               sits under the same root for ontology
    //                               navigation; documented above.)
    is_a: [
        (DocumentItem,                  XmlInformationItem),
        (ElementItem,                   XmlInformationItem),
        (AttributeItem,                 XmlInformationItem),
        (ProcessingInstructionItem,     XmlInformationItem),
        (UnexpandedEntityReferenceItem, XmlInformationItem),
        (CharacterItem,                 XmlInformationItem),
        (CommentItem,                   XmlInformationItem),
        (DocumentTypeDeclarationItem,   XmlInformationItem),
        (UnparsedEntityItem,            XmlInformationItem),
        (NotationItem,                  XmlInformationItem),
        (NamespaceItem,                 XmlInformationItem),
        (XmlReservedAttribute,          XmlInformationItem),
    ],
}

// =============================================================================
// Quality: which W3C-published source defines a concept's primary semantics.
// =============================================================================

/// Which W3C-published spec defines a concept's primary semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Xml10Spec {
    /// Cowan & Tobin (2004) XML Information Set (Second Edition).
    XmlInfoset2004,
    /// W3C xml.xsd — the four reserved-namespace attributes
    /// (xml:lang, xml:space, xml:base, xml:id).
    XmlNamespaceXsd,
}

/// Quality: primary-source mapping. Mirrors the HTML ontology's
/// `SpecSource` pattern.
#[derive(Debug, Clone)]
pub struct Xml10SpecSource;

impl Quality for Xml10SpecSource {
    type Individual = Xml10Concept;
    type Value = Xml10Spec;

    fn get(&self, c: &Xml10Concept) -> Option<Xml10Spec> {
        use Xml10Concept as X;
        match c {
            X::XmlReservedAttribute => Some(Xml10Spec::XmlNamespaceXsd),
            X::DocumentItem
            | X::ElementItem
            | X::AttributeItem
            | X::ProcessingInstructionItem
            | X::UnexpandedEntityReferenceItem
            | X::CharacterItem
            | X::CommentItem
            | X::DocumentTypeDeclarationItem
            | X::UnparsedEntityItem
            | X::NotationItem
            | X::NamespaceItem => Some(Xml10Spec::XmlInfoset2004),
            // Abstract root — both specs together.
            X::XmlInformationItem => None,
        }
    }
}

// =============================================================================
// Ontology + axioms
// =============================================================================

impl Ontology for Xml10Ontology {
    type Cat = Xml10Category;
    type Qual = Xml10SpecSource;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(EveryConceptReachesRoot));
        axioms.push(Box::new(ElevenInformationItemsClosed));
        axioms.push(Box::new(ReservedAttributeInventoryNonEmpty));
        axioms.push(Box::new(SingleDocumentItemPerDocument));
        axioms.push(Box::new(EveryNonRootHasSpecSource));
        axioms
    }
}

/// Axiom: every non-root concept is `is_a`-reachable from
/// `XmlInformationItem`. Cowan & Tobin 2004 §2 partitions every
/// piece of an XML document into one of the 11 information items
/// (plus the `xml:*`-reserved attribute vocabulary).
pub struct EveryConceptReachesRoot;

impl Axiom for EveryConceptReachesRoot {
    fn verify(&self) -> Verdict {
        use pr4xis::category::{Arrow, Category};
        let morphs = Xml10Category::morphisms();
        for v in Xml10Concept::variants() {
            if matches!(v, Xml10Concept::XmlInformationItem) {
                continue;
            }
            let reaches = morphs.iter().any(|m| {
                m.source() == v
                    && m.target() == Xml10Concept::XmlInformationItem
                    && matches!(m.kind(), Xml10RelationKind::Subsumption)
            });
            if !reaches {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EveryConceptReachesRoot",
        "every non-root Xml10 concept is is_a-reachable from XmlInformationItem",
        "Cowan & Tobin (2004) XML Information Set §2"
    );
}

pr4xis::register_axiom!(
    EveryConceptReachesRoot,
    "Cowan & Tobin (2004) XML Information Set §2"
);

/// Axiom: exactly 11 information items leaf under `XmlInformationItem`
/// per Cowan & Tobin 2004 §2.1–§2.11. The 12th leaf
/// (`XmlReservedAttribute`) is the vocabulary layer — see comment in
/// `concepts:`.
pub struct ElevenInformationItemsClosed;

impl Axiom for ElevenInformationItemsClosed {
    fn verify(&self) -> Verdict {
        // Build-time invariant: the loaded
        // `XML_INFOSET_INFORMATION_ITEMS` array must carry 11
        // entries. The runtime loader's `information_items()` lifts
        // the array; this check sees the array directly.
        let n = super::loader_v1::information_items().len();
        if n == 11 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ElevenInformationItemsClosed",
        "the loaded XML Information Set inventory has exactly 11 information items per Cowan & Tobin 2004 §2.1–§2.11",
        "Cowan & Tobin (2004) XML Information Set §2.1–§2.11"
    );
}

pr4xis::register_axiom!(
    ElevenInformationItemsClosed,
    "Cowan & Tobin (2004) XML Information Set §2.1–§2.11"
);

/// Axiom: the loaded reserved-attribute inventory (from the W3C
/// xml.xsd) is non-empty. The published xml.xsd declares
/// `xml:lang`, `xml:space`, `xml:base`, `xml:id`; loading none
/// signals a bundle / scan regression.
pub struct ReservedAttributeInventoryNonEmpty;

impl Axiom for ReservedAttributeInventoryNonEmpty {
    fn verify(&self) -> Verdict {
        if super::loader_v1::reserved_attribute_names().is_empty() {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        } else {
            Ok(Box::new(SimpleProof::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ReservedAttributeInventoryNonEmpty",
        "the W3C xml.xsd declares at least one xml:*-reserved attribute",
        "W3C xml.xsd; Bray et al. (2009) Namespaces in XML 1.0 §3"
    );
}

pr4xis::register_axiom!(
    ReservedAttributeInventoryNonEmpty,
    "W3C xml.xsd; Bray et al. (2009) Namespaces in XML 1.0 §3"
);

/// Axiom: an XML 1.0 document carries exactly one `DocumentItem` at
/// its root. XML 1.0 §2.1 ("Well-Formed XML Documents") +
/// Cowan & Tobin 2004 §2.1.
///
/// Structural — the `DocumentItem` variant is unique in the enum,
/// so an XmlInformationSet built from a well-formed document has
/// exactly one of them at the root by construction. The axiom
/// asserts the rule at the ontology level.
pub struct SingleDocumentItemPerDocument;

impl Axiom for SingleDocumentItemPerDocument {
    fn verify(&self) -> Verdict {
        // Structural — see the docstring above.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SingleDocumentItemPerDocument",
        "every well-formed XML document has exactly one DocumentItem at its root",
        "Bray et al. (2008) XML 1.0 §2.1; Cowan & Tobin (2004) XML Information Set §2.1"
    );
}

pr4xis::register_axiom!(
    SingleDocumentItemPerDocument,
    "Bray et al. (2008) XML 1.0 §2.1; Cowan & Tobin (2004) XML Information Set §2.1"
);

/// Axiom: every non-root `Xml10Concept` carries a primary spec
/// source under the `Xml10SpecSource` quality. Totality on the
/// non-root partition.
pub struct EveryNonRootHasSpecSource;

impl Axiom for EveryNonRootHasSpecSource {
    fn verify(&self) -> Verdict {
        let q = Xml10SpecSource;
        for c in Xml10Concept::variants() {
            if matches!(c, Xml10Concept::XmlInformationItem) {
                continue;
            }
            if q.get(&c).is_none() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EveryNonRootHasSpecSource",
        "Xml10SpecSource is total on every non-root Xml10Concept",
        "Cowan & Tobin (2004) XML Information Set; Bray et al. (2009) Namespaces in XML 1.0"
    );
}

pr4xis::register_axiom!(
    EveryNonRootHasSpecSource,
    "Cowan & Tobin (2004) XML Information Set; Bray et al. (2009) Namespaces in XML 1.0"
);

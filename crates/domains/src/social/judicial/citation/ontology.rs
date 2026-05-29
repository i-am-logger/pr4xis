//! Pinpoint citation ontology — concepts, is_a, axioms.
//!
//! See `mod.rs` for literature and the `PinpointCite` value type.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "PinpointCitation",
    source: "United States Legislative Markup (USLM) schema, Office of the Law Revision Counsel — the subdivision <level> element hierarchy (subsection/paragraph/subparagraph/clause/subclause/item), the machine-loaded praxis source for the structure; The Bluebook: A Uniform System of Citation, 21st ed. (2020) §3.2 and §3.3 — the parenthesized citation-rendering convention (LLM-checked, web, print); House Office of the Legislative Counsel Manual on Drafting Style §312(a) — Congressional drafting convention (LLM-checked, web); ALWD Guide to Legal Citation, 7th ed. (2021) ch. 14 — practitioner standard (LLM-checked, web)",

    concepts: [
        // Root
        PinpointCitation,

        // Levels, in nesting order (outermost-first within a Section)
        Title,
        Section,
        Subsection,
        Paragraph,
        Subparagraph,
        Clause,
    ],

    labels: {
        PinpointCitation: ("en", "Pinpoint citation",
            "Bluebook §3.2: a hierarchical citation to a specific subdivision of a statute, regulation, or rule."),
        Title: ("en", "Title",
            "U.S. Code Title (e.g., \"Title 18\"). Outermost level of the statutory citation hierarchy."),
        Section: ("en", "Section",
            "Bluebook §3.2: § N — the primary numbered division of a Title (e.g., 18 U.S.C. § 1514A)."),
        Subsection: ("en", "Subsection",
            "Bluebook §3.3: lower-case parenthesized division (e.g., (a), (b), (c))."),
        Paragraph: ("en", "Paragraph",
            "Bluebook §3.3: numeric parenthesized division within a Subsection (e.g., (1), (2))."),
        Subparagraph: ("en", "Subparagraph",
            "Bluebook §3.3: upper-case parenthesized division within a Paragraph (e.g., (A), (B))."),
        Clause: ("en", "Clause",
            "Bluebook §3.3: lower-case roman parenthesized division within a Subparagraph (e.g., (i), (ii), (iii))."),
    },

    is_a: [
        (Title, PinpointCitation),
        (Section, PinpointCitation),
        (Subsection, PinpointCitation),
        (Paragraph, PinpointCitation),
        (Subparagraph, PinpointCitation),
        (Clause, PinpointCitation),
    ],
}

// ---------------------------------------------------------------------------
// Quality: NestingDepth — the position in the citation hierarchy
// (Title=0, Section=1, Subsection=2, Paragraph=3, Subparagraph=4, Clause=5).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NestingDepth;

impl Quality for NestingDepth {
    type Individual = PinpointCitationConcept;
    type Value = u8;

    fn get(&self, c: &PinpointCitationConcept) -> Option<u8> {
        use PinpointCitationConcept as P;
        match c {
            P::Title => Some(0),
            P::Section => Some(1),
            P::Subsection => Some(2),
            P::Paragraph => Some(3),
            P::Subparagraph => Some(4),
            P::Clause => Some(5),
            P::PinpointCitation => None,
        }
    }
}

pub fn leaves() -> [PinpointCitationConcept; 6] {
    [
        PinpointCitationConcept::Title,
        PinpointCitationConcept::Section,
        PinpointCitationConcept::Subsection,
        PinpointCitationConcept::Paragraph,
        PinpointCitationConcept::Subparagraph,
        PinpointCitationConcept::Clause,
    ]
}

pub fn is_leaf(c: PinpointCitationConcept) -> bool {
    !matches!(c, PinpointCitationConcept::PinpointCitation)
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

impl Ontology for PinpointCitationOntology {
    type Cat = PinpointCitationCategory;
    type Qual = NestingDepth;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(NestingDepthIsStrictTotalOrder));
        axioms
    }
}

/// Axiom: the six nesting levels admit a strict total order via
/// `NestingDepth`. Title outermost (0), Clause innermost (5). The
/// Bluebook §3.3 nesting convention is the source of truth.
pub struct NestingDepthIsStrictTotalOrder;

impl Axiom for NestingDepthIsStrictTotalOrder {
    fn verify(&self) -> Verdict {
        let q = NestingDepth;
        let levels = [
            PinpointCitationConcept::Title,
            PinpointCitationConcept::Section,
            PinpointCitationConcept::Subsection,
            PinpointCitationConcept::Paragraph,
            PinpointCitationConcept::Subparagraph,
            PinpointCitationConcept::Clause,
        ];
        let mut prev: Option<u8> = None;
        for l in levels {
            let Some(v) = q.get(&l) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if let Some(p) = prev
                && v <= p
            {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            prev = Some(v);
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "NestingDepthIsStrictTotalOrder",
        "NestingDepth gives a strict order Title < Section < Subsection < Paragraph < Subparagraph < Clause",
        "USLM schema subdivision hierarchy (loaded praxis source); Bluebook §3.3 (21st ed., 2020) — citation-rendering convention"
    );
}

pr4xis::register_axiom!(
    NestingDepthIsStrictTotalOrder,
    "Bluebook §3.3 (21st ed., 2020)"
);

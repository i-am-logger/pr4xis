//! LegalSources ontology — the formal sources of law and their
//! subsumption hierarchy, grounded in LKIF-Core (machine-verified
//! against `norm.owl`) with the doctrinal genus/species framing from
//! Salmond and Hart.
//!
//! See `mod.rs` for the full literature inventory, the citation-tier
//! table, and the machine-verification record against LKIF-Core
//! `norm.owl`.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "LegalSources",
    source: "Hoekstra, Breuker, Di Bello & Boer (2007) LKIF-Core; Salmond on Jurisprudence (formal sources of law); Hart (1961) The Concept of Law",

    concepts: [
        // Genus — the formal source of law (Salmond; LKIF Legal_Source).
        LegalSource,

        // The document species genus (lkif:Legal_Document).
        LegalDocument,

        // Enacted-document species (all lkif:Legal_Document subclasses).
        Statute,
        Regulation,
        Constitution,
        Treaty,
        Code,

        // Unenacted sources sitting directly under LegalSource
        // (lkif places these under Legal_Source, NOT Legal_Document).
        Precedent,
        CustomaryLaw,
    ],

    labels: {
        // Genus. Tier: doctrinal (Salmond) + LKIF machine-verifiable
        // (lkif:Legal_Source ⊑ expression:Medium in norm.owl).
        LegalSource: ("en", "law",
            "A formal source of law — that from which a rule derives its legal force (Salmond on Jurisprudence; LKIF-Core Legal_Source, norm.owl: \"a source for legal statements, both norms and legal expressions\"). The English word 'law' most commonly denotes this genus. Tier: doctrinal (Salmond) + LKIF machine-verifiable."),

        // Tier: LKIF machine-verifiable (lkif:Legal_Document ⊑ Legal_Source
        // AND expression:Document in norm.owl).
        LegalDocument: ("en", "legal document",
            "lkif:Legal_Document (norm.owl): \"a document bearing norms or normative statements\". The genus of the enacted, written source-types. Tier: LKIF machine-verifiable."),

        // Tier: LKIF machine-verifiable + doctrinal (Salmond enacted law).
        Statute: ("en", "statute",
            "lkif:Statute (norm.owl): \"a statute bears one or more norms, all uttered by some legal person\"; Salmond's \"enacted law\". Tier: LKIF machine-verifiable + doctrinal."),

        // Tier: LKIF machine-verifiable + doctrinal (administrative rulemaking).
        Regulation: ("en", "regulation",
            "lkif:Regulation (norm.owl): \"a regulation bears one or more norms, all uttered by legislative bodies\"; administrative rulemaking (Chevron U.S.A. v. NRDC, 467 U.S. 837 (1984)). Tier: LKIF machine-verifiable + doctrinal."),

        // Tier: DOCTRINAL-ONLY (FLAGGED). Constitution is NOT an LKIF-Core
        // class — norm.owl has no Constitution. This is an honest addition
        // grounded in Hart's rule of recognition; it is placed under
        // LegalDocument by analogy to the other enacted written instruments,
        // NOT on LKIF authority.
        Constitution: ("en", "constitution",
            "HONEST ADDITION — not an LKIF-Core class. Grounded in Hart (1961) The Concept of Law, Ch. VI (the rule of recognition supplies the ultimate criteria of legal validity); U.S. Const. The is_a placement under LegalDocument is doctrinal analogy to the other enacted written instruments, not an LKIF edge. Tier: doctrinal-only, FLAGGED."),

        // Tier: LKIF machine-verifiable + doctrinal.
        Treaty: ("en", "treaty",
            "lkif:Treaty (norm.owl): \"a binding agreement under international law entered into by states and organizations\" (⊑ International_Agreement AND Legal_Document); U.S. Const. Art. II §2. Tier: LKIF machine-verifiable + doctrinal."),

        // Tier: LKIF machine-verifiable.
        Code: ("en", "code",
            "lkif:Code (norm.owl): \"a legal code bears norms uttered by legislative bodies only\" — a compilation of legislation (the United States Code is the grounding target). Tier: LKIF machine-verifiable."),

        // Tier: LKIF machine-verifiable + doctrinal.
        Precedent: ("en", "case law",
            "lkif:Precedent (norm.owl): \"a legal case establishing a principle courts may adopt in subsequent similar cases\" — sits directly under Legal_Source, NOT Legal_Document; Garner (2016) Black's Law Dictionary. Tier: LKIF machine-verifiable + doctrinal."),

        // Tier: LKIF machine-verifiable + doctrinal (Salmond material sources).
        CustomaryLaw: ("en", "customary law",
            "lkif:Customary_Law (norm.owl): \"established patterns of behaviour objectively verified within particular social settings\" (⊑ Legal_Source AND Custom); Salmond's material sources of law. Tier: LKIF machine-verifiable + doctrinal."),
    },

    is_a: [
        // Machine-verified against LKIF-Core norm.owl (WebFetch of
        // RinkeHoekstra/lkif-core master), except (Constitution,
        // LegalDocument) which is a flagged doctrinal analogy.
        (LegalDocument, LegalSource),      // lkif: Legal_Document ⊑ Legal_Source ✓
        (Statute, LegalDocument),          // lkif: Statute ⊑ Legal_Document ✓
        (Regulation, LegalDocument),       // lkif: Regulation ⊑ Legal_Document ✓
        (Constitution, LegalDocument),     // DOCTRINAL analogy (Hart 1961) — not LKIF
        (Treaty, LegalDocument),           // lkif: Treaty ⊑ Legal_Document ✓
        (Code, LegalDocument),             // lkif: Code ⊑ Legal_Document ✓
        (Precedent, LegalSource),          // lkif: Precedent ⊑ Legal_Source (direct) ✓
        (CustomaryLaw, LegalSource),       // lkif: Customary_Law ⊑ Legal_Source (direct) ✓
    ],
}

// ---------------------------------------------------------------------------
// Quality: IsEnactedOf — Salmond's enacted vs. unenacted distinction
// ---------------------------------------------------------------------------

/// Quality: whether a source is *enacted* (deliberately laid down by a
/// competent authority — statute, regulation, constitution, treaty,
/// code) or *unenacted* (arising otherwise — precedent by adjudication,
/// customary law by practice). Salmond on Jurisprudence draws exactly
/// this line between enacted law and case/customary law.
///
/// Returns `None` for the two abstract genera (`LegalSource`,
/// `LegalDocument`), which carry no enactment status of their own.
#[derive(Debug, Clone)]
pub struct IsEnactedOf;

impl Quality for IsEnactedOf {
    type Individual = LegalSourcesConcept;
    type Value = bool;

    fn get(&self, c: &LegalSourcesConcept) -> Option<bool> {
        use LegalSourcesConcept as L;
        match c {
            // Enacted written instruments (Salmond "enacted law").
            L::Statute | L::Regulation | L::Constitution | L::Treaty | L::Code => Some(true),
            // Unenacted sources (Salmond case law + customary law).
            L::Precedent | L::CustomaryLaw => Some(false),
            // Abstract genera carry no enactment status.
            L::LegalSource | L::LegalDocument => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The seven *concrete* source-type concepts — the leaves and
/// enacted-document species that name an actual kind of legal source
/// (excludes the two abstract genera `LegalSource`/`LegalDocument`).
pub fn concrete_source_types() -> [LegalSourcesConcept; 7] {
    [
        LegalSourcesConcept::Statute,
        LegalSourcesConcept::Regulation,
        LegalSourcesConcept::Constitution,
        LegalSourcesConcept::Treaty,
        LegalSourcesConcept::Code,
        LegalSourcesConcept::Precedent,
        LegalSourcesConcept::CustomaryLaw,
    ]
}

/// True iff a Subsumption path `from ⊑ … ⊑ to` exists in the category.
///
/// This is a closure-membership query, not a re-walked BFS: `ontology!`
/// materialises the per-kind transitive closure into `morphisms()`
/// (Floyd–Warshall at macro expansion), so a multi-step subsumption
/// path `from ⊑ mid ⊑ to` is already composed to a single
/// `(from, to, Subsumption)` edge in the closed set.
pub fn subsumes_transitively(from: LegalSourcesConcept, to: LegalSourcesConcept) -> bool {
    LegalSourcesCategory::morphisms().iter().any(|m| {
        m.source() == from && m.target() == to && m.kind() == LegalSourcesRelationKind::Subsumption
    })
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for LegalSourcesOntology {
    type Cat = LegalSourcesCategory;
    type Qual = IsEnactedOf;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(StatuteIsALaw));
        axioms.push(Box::new(SourceTypesUnderGenus));
        axioms.push(Box::new(PrecedentIsNotADocument));
        axioms
    }
}

/// Axiom: the transitive subsumption path Statute ⊑ Legal_Document ⊑
/// Legal_Source exists in the category — a statute *is* a law.
///
/// Machine-verified against LKIF-Core norm.owl (Statute ⊑ Legal_Document,
/// Legal_Document ⊑ Legal_Source); doctrinally, Salmond's enacted law is
/// law.
pub struct StatuteIsALaw;

impl Axiom for StatuteIsALaw {
    fn verify(&self) -> Verdict {
        if subsumes_transitively(
            LegalSourcesConcept::Statute,
            LegalSourcesConcept::LegalSource,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StatuteIsALaw",
        "the transitive subsumption path Statute -> LegalDocument -> LegalSource exists in the category",
        "LKIF-Core (Hoekstra et al. 2007) Statute subClassOf Legal_Document subClassOf Legal_Source; Salmond (enacted law is law)"
    );
}

pr4xis::register_axiom!(
    StatuteIsALaw,
    "LKIF-Core (Hoekstra et al. 2007); Salmond on Jurisprudence"
);

/// Axiom: every concrete source concept (Statute, Regulation,
/// Constitution, Treaty, Code, Precedent, CustomaryLaw) reaches the
/// genus `LegalSource` by subsumption — each *is* a formal source of law.
///
/// Machine-verified against LKIF-Core norm.owl for all but Constitution
/// (a flagged doctrinal addition under Hart's rule of recognition).
pub struct SourceTypesUnderGenus;

impl Axiom for SourceTypesUnderGenus {
    fn verify(&self) -> Verdict {
        for c in concrete_source_types() {
            if !subsumes_transitively(c, LegalSourcesConcept::LegalSource) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SourceTypesUnderGenus",
        "every concrete source concept reaches LegalSource by subsumption",
        "LKIF-Core (Hoekstra et al. 2007) formal sources under Legal_Source; Salmond (formal sources of law)"
    );
}

pr4xis::register_axiom!(
    SourceTypesUnderGenus,
    "LKIF-Core (Hoekstra et al. 2007); Salmond on Jurisprudence"
);

/// Axiom: Precedent has NO subsumption path to LegalDocument. LKIF-Core
/// places `Precedent ⊑ Legal_Source` *directly*, not under
/// `Legal_Document` — case law is a source of law, but the doctrine it
/// establishes is not itself the written document. Verified against
/// LKIF-Core norm.owl (Precedent rdfs:subClassOf Legal_Source only).
pub struct PrecedentIsNotADocument;

impl Axiom for PrecedentIsNotADocument {
    fn verify(&self) -> Verdict {
        let reaches_document = subsumes_transitively(
            LegalSourcesConcept::Precedent,
            LegalSourcesConcept::LegalDocument,
        );
        let reaches_source = subsumes_transitively(
            LegalSourcesConcept::Precedent,
            LegalSourcesConcept::LegalSource,
        );
        // Precedent must reach the genus but NOT the document species.
        if reaches_source && !reaches_document {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PrecedentIsNotADocument",
        "Precedent reaches LegalSource but has no subsumption path to LegalDocument (LKIF places it directly under Legal_Source)",
        "LKIF-Core (Hoekstra et al. 2007) norm.owl: Precedent subClassOf Legal_Source (direct)"
    );
}

pr4xis::register_axiom!(
    PrecedentIsNotADocument,
    "LKIF-Core (Hoekstra et al. 2007) norm.owl"
);

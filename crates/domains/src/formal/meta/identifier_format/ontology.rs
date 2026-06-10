//! Identifier format ontology — concepts, is_a, axioms.
//!
//! See `mod.rs` for the literature inventory and the `Identifier` value type.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::FinitelyGenerated;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "IdentifierFormat",
    source: "W3C CURIE Syntax 1.0 (Birbeck & McCarron 2010) Working Group Note; RFC 4122 (Leach, Mealling, Salz 2005) UUID URN Namespace; RFC 3986 (Berners-Lee, Fielding, Masinter 2005) URI Generic Syntax; ISO/IEC 8824-1:2021 ASN.1 Basic Notation; ITU-T X.660 (2011) OID Registration Authorities; LRC USLM User Guide §11.5 Identifiers (path per §13 Referencing Model); 1 U.S.C. § 204",

    concepts: [
        IdentifierFormat,
        Curie,
        Uuid,
        Uri,
        Oid,
        UslmUrn,
    ],

    labels: {
        IdentifierFormat: ("en", "Identifier format",
            "The syntactic form of an identifier string, independent of its identity-verification semantics."),
        Curie: ("en", "CURIE",
            "W3C CURIE Syntax 1.0: compact URI of the form `prefix:local` (e.g., sox_1514a:a)."),
        Uuid: ("en", "UUID",
            "RFC 4122: 128-bit universally unique identifier in canonical 36-character hyphenated hex form."),
        Uri: ("en", "URI",
            "RFC 3986: generic uniform resource identifier — scheme:hierarchical-part with optional authority/path/query/fragment."),
        Oid: ("en", "OID",
            "ISO 8824-1 / ITU-T X.660: dot-separated decimal arcs forming a hierarchical object identifier (e.g., 1.3.6.1.4.1)."),
        UslmUrn: ("en", "USLM URN",
            "LRC USLM User Guide §11.5 Identifiers (path per §13 Referencing Model): hierarchical relative-reference path identifying a U.S. Code component, e.g., `/us/usc/t18`, `/us/usc/t18/s1514A`, `/us/usc/t18/s1514A/a/1/A`. Begins with `/us/`; jurisdiction-then-corpus-then-component-path. Used as the `identifier` attribute on every USLM structural element."),
    },

    is_a: [
        (Curie, IdentifierFormat),
        (Uuid, IdentifierFormat),
        (Uri, IdentifierFormat),
        (Oid, IdentifierFormat),
        (UslmUrn, IdentifierFormat),
    ],
}

// ---------------------------------------------------------------------------
// Quality: HasResolver — does the format imply a centralised resolution
// mechanism, or is it self-contained?
// ---------------------------------------------------------------------------

/// Quality: whether the identifier format implies a centralised
/// resolution authority. URIs and OIDs imply resolvers (DNS for URIs,
/// the OID registration tree for OIDs); CURIEs and UUIDs don't.
///
/// Returns `None` for the abstract root.
#[derive(Debug, Clone)]
pub struct HasResolver;

impl Quality for HasResolver {
    type Individual = IdentifierFormatConcept;
    type Value = bool;

    fn get(&self, c: &IdentifierFormatConcept) -> Option<bool> {
        use IdentifierFormatConcept as I;
        match c {
            // URI scheme + authority resolution (RFC 3986 §3.2): DNS / scheme-specific.
            I::Uri => Some(true),
            // OID registration tree (ITU-T X.660): centralised arcs.
            I::Oid => Some(true),
            // USLM URN: LRC's uscode.house.gov resolves the path
            // grammar to a Title / Section / Subdivision instance.
            // The resolver is centralised — same model as URI/OID.
            I::UslmUrn => Some(true),
            // CURIE is namespace-prefix-mapped client-side (W3C CURIE §3): no resolver.
            I::Curie => Some(false),
            // UUID is statistically unique without registration (RFC 4122 §1).
            I::Uuid => Some(false),
            I::IdentifierFormat => None,
        }
    }
}

pub fn leaves() -> [IdentifierFormatConcept; 5] {
    [
        IdentifierFormatConcept::Curie,
        IdentifierFormatConcept::Uuid,
        IdentifierFormatConcept::Uri,
        IdentifierFormatConcept::Oid,
        IdentifierFormatConcept::UslmUrn,
    ]
}

pub fn is_leaf(c: IdentifierFormatConcept) -> bool {
    !matches!(c, IdentifierFormatConcept::IdentifierFormat)
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

impl Ontology for IdentifierFormatOntology {
    type Cat = IdentifierFormatCategory;
    type Qual = HasResolver;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(PartitionCompleteness));
        axioms.push(Box::new(EveryLeafHasResolverClassification));
        axioms
    }
}

/// Axiom: the IdentifierFormat partition has exactly five leaves
/// (CURIE / UUID / URI / OID / USLM URN) — the five widely-published
/// syntactic specifications covered by this ontology.
pub struct PartitionCompleteness;

impl Axiom for PartitionCompleteness {
    fn verify(&self) -> Verdict {
        let count = IdentifierFormatConcept::variants()
            .into_iter()
            .filter(|c| is_leaf(*c))
            .count();
        if count == 5 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PartitionCompleteness",
        "IdentifierFormat has five leaves: CURIE, UUID, URI, OID, USLM URN",
        "W3C CURIE 1.0; RFC 4122; RFC 3986; ISO 8824-1; LRC USLM User Guide §11.5 Identifiers"
    );
}

pr4xis::register_axiom!(
    PartitionCompleteness,
    "W3C CURIE 1.0; RFC 4122; RFC 3986; ISO 8824-1; LRC USLM User Guide §11.5 Identifiers"
);

/// Axiom: every leaf has a defined HasResolver classification (true or
/// false — never absent). The decentralised/centralised distinction is
/// total on the four formats.
pub struct EveryLeafHasResolverClassification;

impl Axiom for EveryLeafHasResolverClassification {
    fn verify(&self) -> Verdict {
        let q = HasResolver;
        for leaf in leaves() {
            if q.get(&leaf).is_none() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EveryLeafHasResolverClassification",
        "every IdentifierFormat leaf has a defined HasResolver value",
        "RFC 3986 §3.2; ITU-T X.660; W3C CURIE 1.0 §3; RFC 4122 §1"
    );
}

pr4xis::register_axiom!(
    EveryLeafHasResolverClassification,
    "RFC 3986 §3.2; ITU-T X.660; W3C CURIE 1.0 §3; RFC 4122 §1"
);

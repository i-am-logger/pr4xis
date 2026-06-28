//! Concrete RDF term and triple instances.
//!
//! The sibling [`super::ontology`] module models the *kinds* of node that can
//! occupy a triple (W3C RDF 1.1 Concepts §1.2). This module carries the
//! *runtime instances* — the concrete IRIs, blank-node labels, and
//! literal lexical forms a reader actually produces.
//!
//! Every concrete [`RdfTerm`] classifies back to one of the kinds in
//! [`super::ontology::RdfNodeKind`] via [`RdfTerm::kind`], so the
//! published-W3C axioms (e.g. [`super::ontology::LiteralsCannotBeSubjects`])
//! remain enforceable on a stream of triples.
//!
//! ## Citations
//!
//! - **W3C RDF 1.1 Concepts and Abstract Syntax** (Cyganiak, Wood &
//!   Lanthaler, eds.), W3C Recommendation 2014-02-25 — §3 IRIs, §3.3
//!   Literals, §3.4 Blank Nodes, §3.1 Triples.
//!   <https://www.w3.org/TR/rdf11-concepts/>.

#[allow(unused_imports)]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use super::ontology::RdfNodeKind;

/// One concrete RDF term — what occupies a subject, predicate, or
/// object position in a triple at runtime.
///
/// W3C RDF 1.1 Concepts §3 distinguishes three syntactic categories of
/// term: IRIs (§3.1), blank nodes (§3.4) and literals (§3.3). A literal
/// carries a lexical form together with at most one of (a) a non-empty
/// language tag (BCP 47), making it an `rdf:langString`, or (b) a
/// datatype IRI, making it a typed literal. A literal with neither tag
/// nor datatype is, per RDF 1.1 §3.3, a `xsd:string`-typed literal —
/// kept here as `lang: None, datatype: None` and classified by
/// [`Self::kind`] as a [`RdfNodeKind::PlainLiteral`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RdfTerm {
    /// An absolute IRI reference (RDF 1.1 §3.1; IRIs per RFC 3987).
    Iri(String),
    /// A blank-node identifier with local document scope (RDF 1.1 §3.4).
    /// The label is the *reader-assigned* identifier — auto-generated
    /// blanks use `_:b<counter>`; `rdf:nodeID="..."` blanks use
    /// `_:n_<value>` to avoid colliding with the auto-generated space.
    Blank(String),
    /// A literal (RDF 1.1 §3.3). The `lexical` field is the literal's
    /// lexical form; `lang` is a non-empty BCP 47 language tag when
    /// present (`rdf:langString`); `datatype` is the datatype IRI when
    /// present (typed literal). At most one of `lang` / `datatype` is
    /// `Some` — the two are mutually exclusive per RDF 1.1 §3.3.
    Literal {
        lexical: String,
        lang: Option<String>,
        datatype: Option<String>,
    },
}

impl RdfTerm {
    /// Classify this concrete term back to its [`RdfNodeKind`].
    ///
    /// Per W3C RDF 1.1 §3.3 a literal with neither a language tag nor a
    /// datatype IRI is implicitly typed `xsd:string`; the praxis RDF
    /// ontology models that as [`RdfNodeKind::PlainLiteral`]. A literal
    /// with a non-empty language tag is `rdf:langString` (still a plain
    /// literal in the praxis taxonomy). A literal with an explicit
    /// datatype IRI is [`RdfNodeKind::TypedLiteral`].
    pub fn kind(&self) -> RdfNodeKind {
        match self {
            Self::Iri(_) => RdfNodeKind::IriResource,
            Self::Blank(_) => RdfNodeKind::BlankNode,
            Self::Literal { datatype, .. } => {
                if datatype.is_some() {
                    RdfNodeKind::TypedLiteral
                } else {
                    RdfNodeKind::PlainLiteral
                }
            }
        }
    }

    /// True iff this term may legally appear in subject position.
    ///
    /// Direct application of the published axiom
    /// [`super::ontology::LiteralsCannotBeSubjects`] (W3C RDF 1.1 §3):
    /// subjects are IRIs or blank nodes; literals are excluded.
    pub fn can_be_subject(&self) -> bool {
        self.kind().can_be_subject()
    }
}

/// One RDF triple: `(subject, predicate, object)`, W3C RDF 1.1 §3.1.
///
/// The predicate is always an IRI (the W3C axiom
/// [`super::ontology::PredicatesMustBeProperties`]). Subjects and
/// objects carry their full [`RdfTerm`] shape so the literal-subject
/// invariant can be checked structurally on any [`Triple`] stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Triple {
    pub subject: RdfTerm,
    pub predicate: String,
    pub object: RdfTerm,
}

impl Triple {
    /// Structural check: subject is non-literal (W3C RDF 1.1 §3 —
    /// the [`super::ontology::LiteralsCannotBeSubjects`] axiom
    /// applied to a concrete triple instance).
    pub fn subject_is_admissible(&self) -> bool {
        self.subject.can_be_subject()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each concrete term variant maps to the corresponding kind in the
    /// meta-ontology (W3C RDF 1.1 §3 mapping).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn term_kinds_match_meta_ontology() {
        assert_eq!(RdfTerm::Iri("x".into()).kind(), RdfNodeKind::IriResource);
        assert_eq!(RdfTerm::Blank("b0".into()).kind(), RdfNodeKind::BlankNode);
        // Plain literal: no lang, no datatype → xsd:string (RDF 1.1 §3.3).
        let plain = RdfTerm::Literal {
            lexical: "hello".into(),
            lang: None,
            datatype: None,
        };
        assert_eq!(plain.kind(), RdfNodeKind::PlainLiteral);
        // Lang-tagged literal: rdf:langString, still a plain literal in
        // the praxis taxonomy.
        let lang = RdfTerm::Literal {
            lexical: "hello".into(),
            lang: Some("en".into()),
            datatype: None,
        };
        assert_eq!(lang.kind(), RdfNodeKind::PlainLiteral);
        // Typed literal: datatype IRI present.
        let typed = RdfTerm::Literal {
            lexical: "42".into(),
            lang: None,
            datatype: Some("http://www.w3.org/2001/XMLSchema#integer".into()),
        };
        assert_eq!(typed.kind(), RdfNodeKind::TypedLiteral);
    }

    /// Subject-admissibility tracks the
    /// [`super::ontology::LiteralsCannotBeSubjects`] axiom.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn subject_admissibility_excludes_literals() {
        assert!(RdfTerm::Iri("x".into()).can_be_subject());
        assert!(RdfTerm::Blank("b0".into()).can_be_subject());
        assert!(
            !RdfTerm::Literal {
                lexical: "x".into(),
                lang: None,
                datatype: None,
            }
            .can_be_subject()
        );
        assert!(
            !RdfTerm::Literal {
                lexical: "x".into(),
                lang: None,
                datatype: Some("urn:int".into()),
            }
            .can_be_subject()
        );
    }
}

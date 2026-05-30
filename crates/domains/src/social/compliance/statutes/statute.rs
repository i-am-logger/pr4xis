//! Runtime `Statute` type — the loaded form of a statute registered in
//! `praxis.toml` with a `[structural."<name>@<version>"]` block in
//! `praxis.lock`.
//!
//! Mirrors the [`English::from_wordnet`][english] pattern from
//! `cognitive::linguistics::english`: a typed runtime struct constructed
//! by reading the loaded lock data once and caching the result behind a
//! `OnceLock`. Each registered statute exposes a thin
//! `pub fn statute() -> &'static Statute` accessor (see
//! `sox_1514a/mod.rs` for the canonical example).
//!
//! [english]: crate::cognitive::linguistics::english::English::from_wordnet
//!
//! The Statute holds a `Vec<LegalTerm>` (each term is fully typed
//! against `social::judicial::ontology` — `Identifier` CURIE id,
//! `SourceTextRef` verbatim citations, typed `ProofStandard` / typed
//! `ObligationLanguage` / etc.) and a `Vec<LegalRelation>` (each relation
//! is `(Identifier, Identifier, RelationType)` typed). The
//! `from_structural` constructor reads
//! `super::super::super::applied::data_provisioning::registry::StructuralData`
//! and synthesizes the typed values.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::data_provisioning::registry::{StructuralData, StructuralRelation};
use crate::formal::meta::identifier_format::Identifier;
use crate::social::judicial::ontology::{
    Duration as JudicialDuration, LegalRelation, LegalTerm, RelationType,
};
use crate::social::judicial::source_text::SourceTextRef;

/// The runtime form of a loaded statute. Constructed via
/// [`Self::from_structural_with_context`] from USLM-derived structural
/// data; cached by each per-statute module behind a `OnceLock`.
#[derive(Debug, Clone)]
pub struct Statute {
    name: String,
    version: String,
    description: SourceTextRef,
    terms: Vec<LegalTerm>,
    relations: Vec<LegalRelation>,
}

/// Errors while constructing a `Statute` from praxis.lock structural data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatuteConstructError {
    /// A term's `id` field is not a valid CURIE.
    InvalidTermId {
        term_index: usize,
        id_string: String,
    },
    /// A relation's `from` or `to` field is not a valid CURIE.
    InvalidRelationEndpoint {
        relation_index: usize,
        which: &'static str,
        id_string: String,
    },
    /// A relation refers to a term id that doesn't exist in the
    /// statute's term set.
    DanglingRelation {
        relation_index: usize,
        missing_id: String,
    },
    /// A relation kind in the lock data is not a recognized
    /// `RelationType` variant.
    UnknownRelationKind {
        relation_index: usize,
        kind_string: String,
    },
}

impl core::fmt::Display for StatuteConstructError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidTermId {
                term_index,
                id_string,
            } => write!(f, "term #{term_index}: invalid CURIE id `{id_string}`"),
            Self::InvalidRelationEndpoint {
                relation_index,
                which,
                id_string,
            } => write!(
                f,
                "relation #{relation_index}: {which} endpoint is not a valid CURIE: `{id_string}`"
            ),
            Self::DanglingRelation {
                relation_index,
                missing_id,
            } => write!(
                f,
                "relation #{relation_index} references missing term id `{missing_id}`"
            ),
            Self::UnknownRelationKind {
                relation_index,
                kind_string,
            } => write!(
                f,
                "relation #{relation_index}: unknown relation kind `{kind_string}`"
            ),
        }
    }
}

impl Statute {
    /// Construct a `Statute` with an explicit provenance URI for
    /// every term's name/definition and the statute description.
    /// USLM-derived statutes pass the section URN (e.g.
    /// `/us/usc/t18/s1514A`).
    ///
    /// Validates that every term id is a well-formed CURIE, every
    /// relation endpoint is a valid CURIE and resolves to an
    /// existing term, and every relation kind maps to a known
    /// `RelationType` variant.
    pub fn from_structural_with_context(
        name: &str,
        version: &str,
        data: &StructuralData,
        context_uri: &str,
    ) -> Result<Self, StatuteConstructError> {
        let mut terms = Vec::with_capacity(data.terms.len());
        for (term_index, raw) in data.terms.iter().enumerate() {
            let id = Identifier::curie(raw.id.clone()).map_err(|_| {
                StatuteConstructError::InvalidTermId {
                    term_index,
                    id_string: raw.id.clone(),
                }
            })?;
            terms.push(LegalTerm {
                id,
                name: SourceTextRef::with_context(raw.name.clone(), context_uri),
                definition: SourceTextRef::with_context(raw.definition.clone(), context_uri),
                source_text: None,
                subsection: None,
                required_evidence: Vec::new(),
                obligations: Vec::new(),
                deadlines: Vec::new(),
                rights: Vec::new(),
                remedies: Vec::new(),
                burdens: Vec::new(),
                exceptions: Vec::new(),
            });
        }

        let term_ids: alloc::collections::BTreeSet<&str> =
            terms.iter().map(|t| t.id.value()).collect();

        let mut relations = Vec::with_capacity(data.relations.len());
        for (relation_index, raw) in data.relations.iter().enumerate() {
            let from = Identifier::curie(raw.from.clone()).map_err(|_| {
                StatuteConstructError::InvalidRelationEndpoint {
                    relation_index,
                    which: "from",
                    id_string: raw.from.clone(),
                }
            })?;
            let to = Identifier::curie(raw.to.clone()).map_err(|_| {
                StatuteConstructError::InvalidRelationEndpoint {
                    relation_index,
                    which: "to",
                    id_string: raw.to.clone(),
                }
            })?;
            if !term_ids.contains(from.value()) {
                return Err(StatuteConstructError::DanglingRelation {
                    relation_index,
                    missing_id: raw.from.clone(),
                });
            }
            if !term_ids.contains(to.value()) {
                return Err(StatuteConstructError::DanglingRelation {
                    relation_index,
                    missing_id: raw.to.clone(),
                });
            }
            let relation = parse_relation_kind(raw).ok_or_else(|| {
                StatuteConstructError::UnknownRelationKind {
                    relation_index,
                    kind_string: raw.relation.clone(),
                }
            })?;
            relations.push(LegalRelation { from, to, relation });
        }

        Ok(Self {
            name: name.to_string(),
            version: version.to_string(),
            description: SourceTextRef::with_context(data.description.clone(), context_uri),
            terms,
            relations,
        })
    }

    /// The statute's registered name (`"sox_1514a"`, etc.).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The statute's registered version (`"2002"`, etc.).
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The statute's description from praxis.lock.
    pub fn description(&self) -> &SourceTextRef {
        &self.description
    }

    /// All typed legal terms.
    pub fn terms(&self) -> &[LegalTerm] {
        &self.terms
    }

    /// All typed relations between terms.
    pub fn relations(&self) -> &[LegalRelation] {
        &self.relations
    }

    /// Look up a term by its typed `Identifier`.
    pub fn term_by_id(&self, id: &Identifier) -> Option<&LegalTerm> {
        self.terms.iter().find(|t| t.id == *id)
    }

    /// Look up a term by raw CURIE string (`"sox_1514a:a"`, etc.). A
    /// convenience for callers that haven't yet built a typed
    /// `Identifier`.
    pub fn term_by_curie(&self, curie: &str) -> Option<&LegalTerm> {
        self.terms.iter().find(|t| t.id.value() == curie)
    }

    /// Iterate all relations whose `from` is the given term.
    pub fn relations_from<'a>(
        &'a self,
        id: &'a Identifier,
    ) -> impl Iterator<Item = &'a LegalRelation> + 'a {
        self.relations.iter().filter(move |r| r.from == *id)
    }

    /// Iterate all relations whose `to` is the given term.
    pub fn relations_to<'a>(
        &'a self,
        id: &'a Identifier,
    ) -> impl Iterator<Item = &'a LegalRelation> + 'a {
        self.relations.iter().filter(move |r| r.to == *id)
    }
}

/// Map a praxis.lock relation-kind string to the typed
/// `RelationType` variant. Parameterless variants only: the lock
/// schema drops parametric qualifiers per the codegen mapping's
/// documented losses.
fn parse_relation_kind(raw: &StructuralRelation) -> Option<RelationType> {
    match raw.relation.as_str() {
        "Requires" => Some(RelationType::Requires),
        "SubtypeOf" => Some(RelationType::SubtypeOf),
        "Contradicts" => Some(RelationType::Contradicts),
        "Negates" => Some(RelationType::Negates),
        "AlternativeTo" => Some(RelationType::AlternativeTo),
        "AffirmativeDefenseTo" => Some(RelationType::AffirmativeDefenseTo),
        "SafeHarborFor" => Some(RelationType::SafeHarborFor),
        "ExhaustionRequiredFor" => Some(RelationType::ExhaustionRequiredFor),
        // Parametric variants: lock schema drops the qualifier, so we
        // emit the canonical-default form.
        "Precedes" => Some(RelationType::Precedes { max_days: None }),
        "Implies" => Some(RelationType::Implies {
            consequence: SourceTextRef::new(""),
        }),
        "Composes" => Identifier::curie(raw.to.clone())
            .ok()
            .map(|target| RelationType::Composes { into: target }),
        "Triggers" => Identifier::curie(raw.to.clone())
            .ok()
            .map(|target| RelationType::Triggers { obligation: target }),
        "Rebuts" => Some(RelationType::Rebuts {
            burden: SourceTextRef::new(""),
        }),
        _ => None,
    }
}

/// Silence unused-warning for the imported `JudicialDuration` (the
/// constructor only uses `RelationType` variants that reach it
/// transitively through `Precedes.max_days`).
#[allow(dead_code)]
const _DURATION_WITNESS: Option<JudicialDuration> = None;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applied::data_provisioning::registry::{StructuralRelation, StructuralTerm};

    fn term(id: &str, name: &str) -> StructuralTerm {
        StructuralTerm {
            id: id.to_string(),
            name: name.to_string(),
            definition: format!("definition of {name}"),
            lemmas: Vec::new(),
        }
    }

    fn rel(from: &str, to: &str, relation: &str) -> StructuralRelation {
        StructuralRelation {
            from: from.to_string(),
            to: to.to_string(),
            relation: relation.to_string(),
        }
    }

    fn minimal_data() -> StructuralData {
        StructuralData {
            description: "test statute".into(),
            terms: vec![term("test:a", "Alpha"), term("test:b", "Beta")],
            relations: vec![rel("test:a", "test:b", "Requires")],
        }
    }

    #[test]
    fn happy_path_constructs() {
        let s =
            Statute::from_structural_with_context("test", "1", &minimal_data(), "test://context")
                .unwrap();
        assert_eq!(s.name(), "test");
        assert_eq!(s.version(), "1");
        assert_eq!(s.terms().len(), 2);
        assert_eq!(s.relations().len(), 1);
    }

    #[test]
    fn term_with_invalid_curie_id_rejected() {
        let mut data = minimal_data();
        data.terms[0].id = "no-colon-here".into();
        let err = Statute::from_structural_with_context("test", "1", &data, "test://context")
            .unwrap_err();
        assert!(matches!(
            err,
            StatuteConstructError::InvalidTermId { term_index: 0, .. }
        ));
    }

    #[test]
    fn relation_with_invalid_from_curie_rejected() {
        let mut data = minimal_data();
        data.relations[0].from = "bare".into();
        let err = Statute::from_structural_with_context("test", "1", &data, "test://context")
            .unwrap_err();
        assert!(matches!(
            err,
            StatuteConstructError::InvalidRelationEndpoint {
                relation_index: 0,
                which: "from",
                ..
            }
        ));
    }

    #[test]
    fn relation_with_invalid_to_curie_rejected() {
        let mut data = minimal_data();
        data.relations[0].to = "bare".into();
        let err = Statute::from_structural_with_context("test", "1", &data, "test://context")
            .unwrap_err();
        assert!(matches!(
            err,
            StatuteConstructError::InvalidRelationEndpoint {
                relation_index: 0,
                which: "to",
                ..
            }
        ));
    }

    #[test]
    fn dangling_relation_rejected() {
        let mut data = minimal_data();
        data.relations[0].to = "test:nonexistent".into();
        let err = Statute::from_structural_with_context("test", "1", &data, "test://context")
            .unwrap_err();
        assert!(matches!(
            err,
            StatuteConstructError::DanglingRelation {
                relation_index: 0,
                ..
            }
        ));
    }

    #[test]
    fn unknown_relation_kind_rejected() {
        let mut data = minimal_data();
        data.relations[0].relation = "MadeUp".into();
        let err = Statute::from_structural_with_context("test", "1", &data, "test://context")
            .unwrap_err();
        assert!(matches!(
            err,
            StatuteConstructError::UnknownRelationKind {
                relation_index: 0,
                ..
            }
        ));
    }

    #[test]
    fn all_parameterless_relation_kinds_map() {
        // Every parameterless variant should round-trip through
        // parse_relation_kind.
        for kind in &[
            "Requires",
            "SubtypeOf",
            "Contradicts",
            "Negates",
            "AlternativeTo",
            "AffirmativeDefenseTo",
            "SafeHarborFor",
            "ExhaustionRequiredFor",
            "Precedes",
            "Implies",
            "Composes",
            "Triggers",
            "Rebuts",
        ] {
            let r = StructuralRelation {
                from: "test:a".into(),
                to: "test:b".into(),
                relation: (*kind).into(),
            };
            assert!(
                parse_relation_kind(&r).is_some(),
                "kind `{kind}` should map to a RelationType"
            );
        }
    }

    #[test]
    fn explicit_context_uri_propagates() {
        // The constructor must propagate the supplied `context_uri` to
        // every SourceTextRef it builds — the description and every
        // term's name/definition. USLM-derived consumers rely on this
        // to pin each statute back to its source URN.
        let s = Statute::from_structural_with_context(
            "test",
            "1",
            &minimal_data(),
            "/us/usc/t99/s9999",
        )
        .unwrap();
        assert_eq!(
            s.description().context_uri.as_deref(),
            Some("/us/usc/t99/s9999")
        );
        for t in s.terms() {
            assert_eq!(t.name.context_uri.as_deref(), Some("/us/usc/t99/s9999"));
        }
    }
}

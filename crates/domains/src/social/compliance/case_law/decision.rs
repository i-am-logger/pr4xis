//! Runtime `Decision` type — the loaded form of a case-law precedent
//! registered in `praxis.toml` with a `[structural."<name>@<year>"]`
//! block in `praxis.lock`.
//!
//! Mirrors `social::compliance::statutes::statute::Statute` in shape
//! (typed terms + typed relations + `from_structural` constructor +
//! query API), but diverges in case-specific metadata:
//!
//! - [`Decision::issuing_court`] — typed CURIE into a court ontology
//!   (`court:scotus`, `court:ca_5`, `court:dol_arb`, …).
//! - [`Decision::disposition`] — the procedural disposition
//!   ([`Disposition`] enum: Affirmed / Reversed / Remanded / etc.).
//! - [`Decision::authority_strength`] — the
//!   [`AuthorityStrengthConcept`] this case carries
//!   (SupremeCourtPrecedent / ControllingCircuitPrecedent /
//!   AdministrativeReviewBoardDecision / DistrictCourtPrecedent).
//!
//! These fields participate in the proof-framework composition
//! layer's authority-strength-tagged conflict-resolution rules.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::data_provisioning::registry::{StructuralData, StructuralRelation};
use crate::formal::meta::identifier_format::Identifier;
use crate::social::compliance::statutes::statute::Statute;
use crate::social::judicial::authority_strength::ontology::AuthorityStrengthConcept;
use crate::social::judicial::ontology::{LegalRelation, LegalTerm, RelationType};
use crate::social::judicial::source_text::SourceTextRef;

/// The procedural disposition of a case — what the court did to the
/// judgment below.
///
/// Source: Garner et al. (2016) *The Law of Judicial Precedent*
/// §6.1-6.4; Bluebook (21st ed.) Rule 10.7 (prior and subsequent history).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Disposition {
    /// Court below correct; judgment stands.
    Affirmed,
    /// Court below incorrect; judgment vacated and decided the other
    /// way.
    Reversed,
    /// Court below incorrect on one or more issues but the case
    /// returns to the lower court for further proceedings consistent
    /// with the opinion.
    Remanded,
    /// Partly affirmed and partly reversed (or partly remanded).
    AffirmedInPartReversedInPart,
    /// Court below's judgment vacated without disposition (often with
    /// remand for reconsideration).
    Vacated,
    /// First-instance ruling — no lower court to affirm or reverse.
    /// Used for trial-court opinions and original-jurisdiction
    /// disputes.
    Original,
}

impl Disposition {
    /// Parse a disposition string from praxis.lock's structural
    /// metadata. Recognises PascalCase variant names; returns `None`
    /// on unknown spellings.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "Affirmed" => Self::Affirmed,
            "Reversed" => Self::Reversed,
            "Remanded" => Self::Remanded,
            "AffirmedInPartReversedInPart" => Self::AffirmedInPartReversedInPart,
            "Vacated" => Self::Vacated,
            "Original" => Self::Original,
            _ => return None,
        })
    }
}

/// The runtime form of a loaded case-law decision.
#[derive(Debug, Clone)]
pub struct Decision {
    name: String,
    year: u16,
    description: SourceTextRef,
    issuing_court: Identifier,
    disposition: Disposition,
    authority_strength: AuthorityStrengthConcept,
    terms: Vec<LegalTerm>,
    relations: Vec<LegalRelation>,
}

/// Errors while constructing a `Decision` from praxis.lock structural data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionConstructError {
    InvalidTermId {
        term_index: usize,
        id_string: String,
    },
    InvalidRelationEndpoint {
        relation_index: usize,
        which: &'static str,
        id_string: String,
    },
    DanglingRelation {
        relation_index: usize,
        missing_id: String,
    },
    UnknownRelationKind {
        relation_index: usize,
        kind_string: String,
    },
    InvalidIssuingCourtCurie {
        court_string: String,
    },
}

impl core::fmt::Display for DecisionConstructError {
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
            Self::InvalidIssuingCourtCurie { court_string } => {
                write!(f, "issuing_court is not a valid CURIE: `{court_string}`")
            }
        }
    }
}

impl Decision {
    /// Construct a `Decision` from praxis.lock structural data.
    /// Validates every term CURIE, every relation endpoint, every
    /// relation kind, and the `issuing_court` CURIE.
    ///
    /// The caller supplies case metadata (`year`, `issuing_court`,
    /// `disposition`, `authority_strength`) because those live in
    /// the registry-level manifest (praxis.toml) rather than the
    /// `StructuralData` extracted from the lock — the structural
    /// block carries only term/relation content.
    pub fn from_structural(
        name: &str,
        year: u16,
        issuing_court: &str,
        disposition: Disposition,
        authority_strength: AuthorityStrengthConcept,
        data: &StructuralData,
    ) -> Result<Self, DecisionConstructError> {
        let issuing_court_id = Identifier::curie(issuing_court.to_string()).map_err(|_| {
            DecisionConstructError::InvalidIssuingCourtCurie {
                court_string: issuing_court.to_string(),
            }
        })?;
        let context_uri = format!("praxis-lock://{name}@{year}");

        let mut terms = Vec::with_capacity(data.terms.len());
        for (term_index, raw) in data.terms.iter().enumerate() {
            let id = Identifier::curie(raw.id.clone()).map_err(|_| {
                DecisionConstructError::InvalidTermId {
                    term_index,
                    id_string: raw.id.clone(),
                }
            })?;
            terms.push(LegalTerm {
                id,
                name: SourceTextRef::with_context(raw.name.clone(), &context_uri),
                definition: SourceTextRef::with_context(raw.definition.clone(), &context_uri),
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
                DecisionConstructError::InvalidRelationEndpoint {
                    relation_index,
                    which: "from",
                    id_string: raw.from.clone(),
                }
            })?;
            let to = Identifier::curie(raw.to.clone()).map_err(|_| {
                DecisionConstructError::InvalidRelationEndpoint {
                    relation_index,
                    which: "to",
                    id_string: raw.to.clone(),
                }
            })?;
            if !term_ids.contains(from.value()) {
                return Err(DecisionConstructError::DanglingRelation {
                    relation_index,
                    missing_id: raw.from.clone(),
                });
            }
            if !term_ids.contains(to.value()) {
                return Err(DecisionConstructError::DanglingRelation {
                    relation_index,
                    missing_id: raw.to.clone(),
                });
            }
            let relation = parse_relation_kind(raw).ok_or_else(|| {
                DecisionConstructError::UnknownRelationKind {
                    relation_index,
                    kind_string: raw.relation.clone(),
                }
            })?;
            relations.push(LegalRelation { from, to, relation });
        }

        Ok(Self {
            name: name.to_string(),
            year,
            description: SourceTextRef::with_context(data.description.clone(), &context_uri),
            issuing_court: issuing_court_id,
            disposition,
            authority_strength,
            terms,
            relations,
        })
    }

    /// The case's short-name slug (e.g. `"murray_v_ubs"`).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The decision year.
    pub fn year(&self) -> u16 {
        self.year
    }

    /// The case's description from praxis.lock.
    pub fn description(&self) -> &SourceTextRef {
        &self.description
    }

    /// The issuing court, as a CURIE (e.g. `court:scotus`,
    /// `court:dol_arb`).
    pub fn issuing_court(&self) -> &Identifier {
        &self.issuing_court
    }

    /// The procedural disposition.
    pub fn disposition(&self) -> Disposition {
        self.disposition
    }

    /// The `AuthorityStrengthConcept` this decision carries.
    /// Participates in the proof-framework composition layer's
    /// conflict resolution.
    pub fn authority_strength(&self) -> AuthorityStrengthConcept {
        self.authority_strength
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

    /// Look up a term by raw CURIE string.
    pub fn term_by_curie(&self, curie: &str) -> Option<&LegalTerm> {
        self.terms.iter().find(|t| t.id.value() == curie)
    }

    /// Iterate relations whose `from` is the given term.
    pub fn relations_from<'a>(
        &'a self,
        id: &'a Identifier,
    ) -> impl Iterator<Item = &'a LegalRelation> + 'a {
        self.relations.iter().filter(move |r| r.from == *id)
    }

    /// Iterate relations whose `to` is the given term.
    pub fn relations_to<'a>(
        &'a self,
        id: &'a Identifier,
    ) -> impl Iterator<Item = &'a LegalRelation> + 'a {
        self.relations.iter().filter(move |r| r.to == *id)
    }

    /// Convert this `Decision` into a `Statute`-shaped view — only
    /// the term/relation/description structure, dropping the
    /// case-specific metadata. Useful when downstream code wants to
    /// process statutes and cases uniformly through the proof-
    /// framework composition layer.
    ///
    /// Reuses `Statute::from_structural_with_context` to share
    /// validation. The provenance URI for the projected view is the
    /// case's `praxis-lock://name@year` shim form.
    pub fn as_statute_view(
        &self,
        data: &StructuralData,
    ) -> Result<Statute, crate::social::compliance::statutes::StatuteConstructError> {
        let version = alloc::format!("{}", self.year);
        let context_uri = alloc::format!("praxis-lock://{}@{}", self.name, version);
        Statute::from_structural_with_context(&self.name, &version, data, &context_uri)
    }
}

/// Map a praxis.lock relation-kind string to the typed
/// `RelationType` variant. Parameterless variants only; parametric
/// variants emit canonical-default forms per the codegen's
/// documented losses. Identical to `Statute::parse_relation_kind` so
/// that statutes and cases share relation semantics.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applied::data_provisioning::registry::{StructuralData, StructuralTerm};

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
            description: "test case".into(),
            terms: vec![term("c:h1", "Holding 1"), term("c:h2", "Holding 2")],
            relations: vec![rel("c:h1", "c:h2", "Requires")],
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn happy_path_constructs() {
        let d = Decision::from_structural(
            "test_case",
            2024,
            "court:scotus",
            Disposition::Affirmed,
            AuthorityStrengthConcept::SupremeCourtPrecedent,
            &minimal_data(),
        )
        .unwrap();
        assert_eq!(d.name(), "test_case");
        assert_eq!(d.year(), 2024);
        assert_eq!(d.disposition(), Disposition::Affirmed);
        assert_eq!(
            d.authority_strength(),
            AuthorityStrengthConcept::SupremeCourtPrecedent
        );
        assert_eq!(d.issuing_court().value(), "court:scotus");
        assert_eq!(d.terms().len(), 2);
        assert_eq!(d.relations().len(), 1);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn invalid_issuing_court_rejected() {
        let err = Decision::from_structural(
            "test_case",
            2024,
            "no_colon",
            Disposition::Affirmed,
            AuthorityStrengthConcept::SupremeCourtPrecedent,
            &minimal_data(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            DecisionConstructError::InvalidIssuingCourtCurie { .. }
        ));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn invalid_term_id_rejected() {
        let mut data = minimal_data();
        data.terms[0].id = "no-colon".into();
        let err = Decision::from_structural(
            "test_case",
            2024,
            "court:scotus",
            Disposition::Affirmed,
            AuthorityStrengthConcept::SupremeCourtPrecedent,
            &data,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            DecisionConstructError::InvalidTermId { term_index: 0, .. }
        ));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn dangling_relation_rejected() {
        let mut data = minimal_data();
        data.relations[0].to = "c:nonexistent".into();
        let err = Decision::from_structural(
            "test_case",
            2024,
            "court:scotus",
            Disposition::Affirmed,
            AuthorityStrengthConcept::SupremeCourtPrecedent,
            &data,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            DecisionConstructError::DanglingRelation {
                relation_index: 0,
                ..
            }
        ));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn unknown_relation_kind_rejected() {
        let mut data = minimal_data();
        data.relations[0].relation = "MadeUp".into();
        let err = Decision::from_structural(
            "test_case",
            2024,
            "court:scotus",
            Disposition::Affirmed,
            AuthorityStrengthConcept::SupremeCourtPrecedent,
            &data,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            DecisionConstructError::UnknownRelationKind { .. }
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn disposition_parse_known_variants() {
        assert_eq!(Disposition::parse("Affirmed"), Some(Disposition::Affirmed));
        assert_eq!(Disposition::parse("Reversed"), Some(Disposition::Reversed));
        assert_eq!(Disposition::parse("Remanded"), Some(Disposition::Remanded));
        assert_eq!(Disposition::parse("Vacated"), Some(Disposition::Vacated));
        assert_eq!(Disposition::parse("Original"), Some(Disposition::Original));
        assert_eq!(
            Disposition::parse("AffirmedInPartReversedInPart"),
            Some(Disposition::AffirmedInPartReversedInPart)
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn disposition_parse_unknown_returns_none() {
        assert_eq!(Disposition::parse("affirmed"), None);
        assert_eq!(Disposition::parse("Unknown"), None);
        assert_eq!(Disposition::parse(""), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lookup_helpers_work() {
        let d = Decision::from_structural(
            "test_case",
            2024,
            "court:scotus",
            Disposition::Affirmed,
            AuthorityStrengthConcept::SupremeCourtPrecedent,
            &minimal_data(),
        )
        .unwrap();
        assert_eq!(d.term_by_curie("c:h1").unwrap().name.text, "Holding 1");
        assert_eq!(d.term_by_curie("c:h2").unwrap().name.text, "Holding 2");
        assert!(d.term_by_curie("c:missing").is_none());

        let id = Identifier::curie("c:h1".to_string()).unwrap();
        assert!(d.term_by_id(&id).is_some());
        assert_eq!(d.relations_from(&id).count(), 1);
        assert_eq!(d.relations_to(&id).count(), 0);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lock_context_uri_format() {
        let d = Decision::from_structural(
            "test_case",
            2024,
            "court:scotus",
            Disposition::Affirmed,
            AuthorityStrengthConcept::SupremeCourtPrecedent,
            &minimal_data(),
        )
        .unwrap();
        assert_eq!(
            d.description().context_uri.as_deref(),
            Some("praxis-lock://test_case@2024")
        );
        for t in d.terms() {
            assert_eq!(
                t.name.context_uri.as_deref(),
                Some("praxis-lock://test_case@2024")
            );
        }
    }
}

//! Proof-framework composition type + per-framework instances.
//!
//! See `compositions/mod.rs` for the literature and the rationale
//! against new hand-coded concepts. Each framework instance lives in
//! its own sub-module and is exposed via a `framework()` accessor —
//! same pattern as `Statute::statute()` / `Decision::decision()`.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use hashbrown::HashMap;

use crate::formal::meta::identifier_format::Identifier;
use crate::social::compliance::statutes::Statute;
use crate::social::judicial::authority_strength::ontology::AuthorityStrengthConcept;
use crate::social::judicial::source_text::SourceTextRef;

pub mod sox_retaliation;

/// Kinds of cross-source reference. Subset of `RelationType` that
/// makes sense at the *cross-source* level: structural composition,
/// procedural requirement, and incorporation-by-reference.
///
/// Source: Dickerson (1975) §6.4 distinguishes three forms of
/// inter-statutory reference: incorporation by reference (`Requires`
/// in this taxonomy), structural composition (`Composes`), and
/// implication (`Implies`). Sartor (2005) §21.3 ties these to
/// formal authority-composition operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossReferenceKind {
    /// Source S's term T cannot be applied without simultaneously
    /// applying target T's term U. Mirrors `RelationType::Requires`.
    /// The canonical statutory-cross-reference pattern (Dickerson
    /// 1975 §6.4 — "incorporation by reference").
    Requires,
    /// Source S's term T is a *constituent part* of target T's term
    /// U. Used when one statute defines a term whose substance lives
    /// in another statute. Mirrors `RelationType::Composes`.
    Composes,
    /// Source S's term T, when satisfied, *triggers* target T's term
    /// U as a derived consequence. Mirrors `RelationType::Triggers`.
    Triggers,
    /// Source S's term T is the substantive realization of target
    /// T's procedural placeholder U. Mirrors `RelationType::Implies`.
    Implies,
}

impl CrossReferenceKind {
    /// Parse a cross-reference kind from string form (used by
    /// future praxis.lock-loaded compositions; current static
    /// compositions construct the variant directly).
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "Requires" => Self::Requires,
            "Composes" => Self::Composes,
            "Triggers" => Self::Triggers,
            "Implies" => Self::Implies,
            _ => return None,
        })
    }
}

/// A typed cross-source reference. Each endpoint names (source-name,
/// term-curie); endpoints must resolve to existing terms in the
/// bundled sources when the framework is constructed.
#[derive(Debug, Clone)]
pub struct CrossReference {
    pub from_source: String,
    pub from_term: Identifier,
    pub kind: CrossReferenceKind,
    pub to_source: String,
    pub to_term: Identifier,
    pub rationale: SourceTextRef,
}

/// The runtime form of a proof-framework composition.
#[derive(Debug, Clone)]
pub struct ProofFramework {
    name: String,
    description: SourceTextRef,
    /// Loaded statutes participating in this framework. Decisions
    /// (case-law) will join this list as the PDF loader unlocks
    /// per-case extraction; until then, sources is statutes-only.
    statutes: Vec<&'static Statute>,
    cross_references: Vec<CrossReference>,
    /// Authority strength per source name. Used by the conflict-
    /// resolution rule (when multiple sources disagree, higher tier
    /// wins — see `BindingForceOf` quality in
    /// `social::judicial::authority_strength`).
    authority_strengths: HashMap<String, AuthorityStrengthConcept>,
}

/// Errors constructing a `ProofFramework`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofFrameworkBuildError {
    /// A cross-reference's `from_source` doesn't match any bundled
    /// statute's name.
    UnknownFromSource {
        cross_ref_index: usize,
        source_name: String,
    },
    /// A cross-reference's `to_source` doesn't match any bundled
    /// statute's name.
    UnknownToSource {
        cross_ref_index: usize,
        source_name: String,
    },
    /// A cross-reference's `from_term` doesn't exist in its source
    /// statute.
    DanglingFromTerm {
        cross_ref_index: usize,
        source_name: String,
        term_curie: String,
    },
    /// A cross-reference's `to_term` doesn't exist in its target
    /// statute.
    DanglingToTerm {
        cross_ref_index: usize,
        source_name: String,
        term_curie: String,
    },
    /// `authority_strengths` is missing an entry for a bundled
    /// statute. Every source must carry an authority-strength tag
    /// for conflict resolution to be well-defined.
    MissingAuthorityStrength { source_name: String },
}

impl core::fmt::Display for ProofFrameworkBuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownFromSource {
                cross_ref_index,
                source_name,
            } => write!(
                f,
                "cross-reference #{cross_ref_index}: from_source `{source_name}` not among bundled statutes"
            ),
            Self::UnknownToSource {
                cross_ref_index,
                source_name,
            } => write!(
                f,
                "cross-reference #{cross_ref_index}: to_source `{source_name}` not among bundled statutes"
            ),
            Self::DanglingFromTerm {
                cross_ref_index,
                source_name,
                term_curie,
            } => write!(
                f,
                "cross-reference #{cross_ref_index}: from_term `{term_curie}` not found in source `{source_name}`"
            ),
            Self::DanglingToTerm {
                cross_ref_index,
                source_name,
                term_curie,
            } => write!(
                f,
                "cross-reference #{cross_ref_index}: to_term `{term_curie}` not found in source `{source_name}`"
            ),
            Self::MissingAuthorityStrength { source_name } => write!(
                f,
                "bundled statute `{source_name}` has no authority-strength tag"
            ),
        }
    }
}

impl ProofFramework {
    /// Construct a `ProofFramework`. Validates that:
    /// - Every cross-reference's `from_source` and `to_source` names a
    ///   bundled statute.
    /// - Every cross-reference's `from_term` and `to_term` resolves to
    ///   an existing term in its source statute.
    /// - Every bundled statute has an authority-strength tag.
    pub fn new(
        name: &str,
        description: SourceTextRef,
        statutes: Vec<&'static Statute>,
        cross_references: Vec<CrossReference>,
        authority_strengths: HashMap<String, AuthorityStrengthConcept>,
    ) -> Result<Self, ProofFrameworkBuildError> {
        // Authority-strength coverage.
        for s in &statutes {
            if !authority_strengths.contains_key(s.name()) {
                return Err(ProofFrameworkBuildError::MissingAuthorityStrength {
                    source_name: s.name().to_string(),
                });
            }
        }

        // Cross-reference endpoint validation.
        let source_names: hashbrown::HashSet<&str> = statutes.iter().map(|s| s.name()).collect();
        for (i, cr) in cross_references.iter().enumerate() {
            if !source_names.contains(cr.from_source.as_str()) {
                return Err(ProofFrameworkBuildError::UnknownFromSource {
                    cross_ref_index: i,
                    source_name: cr.from_source.clone(),
                });
            }
            if !source_names.contains(cr.to_source.as_str()) {
                return Err(ProofFrameworkBuildError::UnknownToSource {
                    cross_ref_index: i,
                    source_name: cr.to_source.clone(),
                });
            }
            let from_statute = statutes
                .iter()
                .find(|s| s.name() == cr.from_source)
                .expect("from_source verified above");
            if from_statute.term_by_id(&cr.from_term).is_none() {
                return Err(ProofFrameworkBuildError::DanglingFromTerm {
                    cross_ref_index: i,
                    source_name: cr.from_source.clone(),
                    term_curie: cr.from_term.value().to_string(),
                });
            }
            let to_statute = statutes
                .iter()
                .find(|s| s.name() == cr.to_source)
                .expect("to_source verified above");
            if to_statute.term_by_id(&cr.to_term).is_none() {
                return Err(ProofFrameworkBuildError::DanglingToTerm {
                    cross_ref_index: i,
                    source_name: cr.to_source.clone(),
                    term_curie: cr.to_term.value().to_string(),
                });
            }
        }

        Ok(Self {
            name: name.to_string(),
            description,
            statutes,
            cross_references,
            authority_strengths,
        })
    }

    /// The framework's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Description (rationale + literature reference).
    pub fn description(&self) -> &SourceTextRef {
        &self.description
    }

    /// All bundled statutes.
    pub fn statutes(&self) -> &[&'static Statute] {
        &self.statutes
    }

    /// All cross-references between bundled sources.
    pub fn cross_references(&self) -> &[CrossReference] {
        &self.cross_references
    }

    /// The authority-strength tag for a given source name.
    pub fn authority_strength(&self, source_name: &str) -> Option<AuthorityStrengthConcept> {
        self.authority_strengths.get(source_name).copied()
    }

    /// Look up a statute by name from the bundled set.
    pub fn statute_by_name(&self, name: &str) -> Option<&'static Statute> {
        self.statutes.iter().copied().find(|s| s.name() == name)
    }

    /// Cross-references originating in a given source.
    pub fn cross_references_from<'a>(
        &'a self,
        source_name: &'a str,
    ) -> impl Iterator<Item = &'a CrossReference> + 'a {
        self.cross_references
            .iter()
            .filter(move |cr| cr.from_source == source_name)
    }

    /// Cross-references terminating in a given source.
    pub fn cross_references_to<'a>(
        &'a self,
        source_name: &'a str,
    ) -> impl Iterator<Item = &'a CrossReference> + 'a {
        self.cross_references
            .iter()
            .filter(move |cr| cr.to_source == source_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::compliance::statutes::{air21_42121, sox_1514a};

    fn id(curie: &str) -> Identifier {
        Identifier::curie(curie.to_string()).expect("valid CURIE")
    }

    #[test]
    fn happy_path_constructs() {
        let statutes: Vec<&'static Statute> = vec![sox_1514a::statute(), air21_42121::statute()];
        let mut auth = HashMap::new();
        auth.insert(
            "sox_1514a".to_string(),
            AuthorityStrengthConcept::FederalStatute,
        );
        auth.insert(
            "air21_42121".to_string(),
            AuthorityStrengthConcept::FederalStatute,
        );
        let cross_refs = vec![CrossReference {
            from_source: "sox_1514a".to_string(),
            from_term: id("sox_1514a:b_2_C"),
            kind: CrossReferenceKind::Requires,
            to_source: "air21_42121".to_string(),
            to_term: id("air21_42121:b_2_B"),
            rationale: SourceTextRef::new(
                "SOX § 1514A(b)(2)(C) governs district-court actions by the AIR21 § 42121(b) burden-shifting framework",
            ),
        }];
        let fw = ProofFramework::new(
            "test",
            SourceTextRef::new("test framework"),
            statutes,
            cross_refs,
            auth,
        )
        .unwrap();
        assert_eq!(fw.name(), "test");
        assert_eq!(fw.statutes().len(), 2);
        assert_eq!(fw.cross_references().len(), 1);
    }

    #[test]
    fn unknown_from_source_rejected() {
        let statutes: Vec<&'static Statute> = vec![sox_1514a::statute()];
        let mut auth = HashMap::new();
        auth.insert(
            "sox_1514a".to_string(),
            AuthorityStrengthConcept::FederalStatute,
        );
        let cross_refs = vec![CrossReference {
            from_source: "nonexistent".to_string(),
            from_term: id("nonexistent:a"),
            kind: CrossReferenceKind::Requires,
            to_source: "sox_1514a".to_string(),
            to_term: id("sox_1514a:a"),
            rationale: SourceTextRef::new("test"),
        }];
        let err = ProofFramework::new(
            "test",
            SourceTextRef::new("test framework"),
            statutes,
            cross_refs,
            auth,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ProofFrameworkBuildError::UnknownFromSource { .. }
        ));
    }

    #[test]
    fn dangling_from_term_rejected() {
        let statutes: Vec<&'static Statute> = vec![sox_1514a::statute(), air21_42121::statute()];
        let mut auth = HashMap::new();
        auth.insert(
            "sox_1514a".to_string(),
            AuthorityStrengthConcept::FederalStatute,
        );
        auth.insert(
            "air21_42121".to_string(),
            AuthorityStrengthConcept::FederalStatute,
        );
        let cross_refs = vec![CrossReference {
            from_source: "sox_1514a".to_string(),
            from_term: id("sox_1514a:nonexistent"),
            kind: CrossReferenceKind::Requires,
            to_source: "air21_42121".to_string(),
            to_term: id("air21_42121:b_2_B"),
            rationale: SourceTextRef::new("test"),
        }];
        let err = ProofFramework::new(
            "test",
            SourceTextRef::new("test framework"),
            statutes,
            cross_refs,
            auth,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ProofFrameworkBuildError::DanglingFromTerm { .. }
        ));
    }

    #[test]
    fn missing_authority_strength_rejected() {
        let statutes: Vec<&'static Statute> = vec![sox_1514a::statute()];
        let auth = HashMap::new(); // empty
        let cross_refs = Vec::new();
        let err = ProofFramework::new(
            "test",
            SourceTextRef::new("test framework"),
            statutes,
            cross_refs,
            auth,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ProofFrameworkBuildError::MissingAuthorityStrength { .. }
        ));
    }

    #[test]
    fn cross_reference_kind_parse() {
        assert_eq!(
            CrossReferenceKind::parse("Requires"),
            Some(CrossReferenceKind::Requires)
        );
        assert_eq!(
            CrossReferenceKind::parse("Composes"),
            Some(CrossReferenceKind::Composes)
        );
        assert_eq!(
            CrossReferenceKind::parse("Triggers"),
            Some(CrossReferenceKind::Triggers)
        );
        assert_eq!(
            CrossReferenceKind::parse("Implies"),
            Some(CrossReferenceKind::Implies)
        );
        assert_eq!(CrossReferenceKind::parse("Unknown"), None);
    }
}

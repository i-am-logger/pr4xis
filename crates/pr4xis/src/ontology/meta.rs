use std::borrow::Cow;
use std::fmt;

use crate::ontology::upper::being::Being;

/// Lemon + PROV-O metadata shared by every structural entity in pr4xis —
/// ontologies, axioms, morphisms, functors, natural transformations, and
/// adjunctions (issue #153).
///
/// # Shape provenance
///
/// - `name`, `description` — ONTOLEX-Lemon (W3C 2016) canonical form +
///   Lemon Form label
/// - `citation` — PROV-O (W3C 2013) provenance reference
/// - `module_path` — implementation-specific (Rust module location)
///
/// # Mac Lane (1971) XII.3
///
/// Every "arrow" in pr4xis — whether a 1-cell morphism, a 1-cell functor
/// in Cat, a 2-cell natural transformation, or a structured 2-cell pair
/// adjunction — is a directed relationship with identity and provenance.
/// Gruber (1993) / OBO-RO (Smith et al. 2005): every relation is a
/// formally-named type. These two principles meet here: one metadata
/// shape for every named structural entity at every dimension.
///
/// Replaces the four parallel structs (OntologyMeta, FunctorMeta,
/// AdjunctionMeta, NaturalTransformationMeta) and the nearly-identical
/// AxiomMeta. Only [`Vocabulary`] stays separate — it carries
/// additional DOLCE `Being` classification and concept/morphism snapshots.
#[derive(Debug, Clone)]
pub struct RelationshipMeta {
    pub name: OntologyName,
    /// English-language label (Lemon Form). Defaults to name.
    pub description: Label,
    pub citation: Citation,
    pub module_path: ModulePath,
}

/// BCP 47 language tag — typed identifier for a language.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LanguageCode(Cow<'static, str>);

impl LanguageCode {
    pub const fn new_static(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }

    pub fn new(s: impl Into<Cow<'static, str>>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// English — the most common language in pr4xis.
    pub const ENGLISH: Self = Self::new_static("en");
}

impl From<&'static str> for LanguageCode {
    fn from(s: &'static str) -> Self {
        Self::new_static(s)
    }
}

impl From<String> for LanguageCode {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl AsRef<str> for LanguageCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A label — the written representation of a concept in a language.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(Cow<'static, str>);

impl Label {
    pub const fn new_static(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }

    pub fn new(s: impl Into<Cow<'static, str>>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl LanguageCode {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&'static str> for Label {
    fn from(s: &'static str) -> Self {
        Self::new_static(s)
    }
}

impl From<String> for Label {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<&str> for Label {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// A definition — the meaning of a concept in a language.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Definition(Cow<'static, str>);

impl Definition {
    pub const fn new_static(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }

    pub fn new(s: impl Into<Cow<'static, str>>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&'static str> for Definition {
    fn from(s: &'static str) -> Self {
        Self::new_static(s)
    }
}

impl From<String> for Definition {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Lexical metadata for a concept or morphism — Ontolex-Lemon (W3C 2016).
///
/// Each identifier can optionally carry its label, definition, and language.
/// This is the "lemon" attached to every name in the ontology.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lexical {
    pub label: Label,
    pub definition: Definition,
    pub language: LanguageCode,
}

impl Lexical {
    pub fn new(
        label: impl Into<Label>,
        definition: impl Into<Definition>,
        language: impl Into<LanguageCode>,
    ) -> Self {
        Self {
            label: label.into(),
            definition: definition.into(),
            language: language.into(),
        }
    }
}

/// Name of a concept — a typed identifier for an individual in an ontology.
///
/// Instance of Lemon LexicalEntry: carries the canonical form (identifier)
/// and optionally the lexical data (label, definition, language).
/// Typed so it can't be confused with a plain string or OntologyName.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConceptName {
    identifier: Cow<'static, str>,
    lexical: Option<Lexical>,
}

impl ConceptName {
    pub const fn new_static(s: &'static str) -> Self {
        Self {
            identifier: Cow::Borrowed(s),
            lexical: None,
        }
    }

    pub fn new(s: impl Into<Cow<'static, str>>) -> Self {
        Self {
            identifier: s.into(),
            lexical: None,
        }
    }

    pub fn with_lexical(mut self, lexical: Lexical) -> Self {
        self.lexical = Some(lexical);
        self
    }

    pub fn as_str(&self) -> &str {
        &self.identifier
    }

    pub fn lexical(&self) -> Option<&Lexical> {
        self.lexical.as_ref()
    }
}

impl From<&'static str> for ConceptName {
    fn from(s: &'static str) -> Self {
        Self::new_static(s)
    }
}

impl From<String> for ConceptName {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl AsRef<str> for ConceptName {
    fn as_ref(&self) -> &str {
        &self.identifier
    }
}

impl fmt::Display for ConceptName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.identifier.fmt(f)
    }
}

/// Kind of a morphism — the relation type between concepts, named per
/// the Relations umbrella ontology (Smith et al. 2005 OBO-RO; Gruber
/// 1993). Sugar clauses in `ontology!` (`is_a:`, `has_a:`, `causes:`,
/// `opposes:`) desugar to these canonical variants.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MorphismKind {
    Identity,
    Subsumption,
    Parthood,
    Causation,
    Opposition,
    Equivalence,
    Composed,
    Custom(Cow<'static, str>),
}

impl MorphismKind {
    pub fn as_str(&self) -> &str {
        match self {
            MorphismKind::Identity => "Identity",
            MorphismKind::Subsumption => "Subsumption",
            MorphismKind::Parthood => "Parthood",
            MorphismKind::Causation => "Causation",
            MorphismKind::Opposition => "Opposition",
            MorphismKind::Equivalence => "Equivalence",
            MorphismKind::Composed => "Composed",
            MorphismKind::Custom(s) => s,
        }
    }
}

impl fmt::Display for MorphismKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A morphism in an ontology — the full structure, not just an identifier.
///
/// Carries source concept, target concept, relation kind, and optional
/// lexical metadata (for "what does this relation mean" in a language).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Morphism {
    pub from: ConceptName,
    pub to: ConceptName,
    pub kind: MorphismKind,
    lexical: Option<Lexical>,
}

impl Morphism {
    pub fn new(
        from: impl Into<ConceptName>,
        to: impl Into<ConceptName>,
        kind: MorphismKind,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            kind,
            lexical: None,
        }
    }

    pub fn with_lexical(mut self, lexical: Lexical) -> Self {
        self.lexical = Some(lexical);
        self
    }

    pub fn lexical(&self) -> Option<&Lexical> {
        self.lexical.as_ref()
    }
}

impl fmt::Display for Morphism {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}--{}-->{}", self.from, self.kind, self.to)
    }
}

/// Name of an ontology — a typed identifier, not a raw string.
///
/// Compile-time names are `Cow::Borrowed(&'static str)` — zero allocation.
/// Runtime-composed names are `Cow::Owned(String)` — no leak, dropped with the Vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OntologyName(Cow<'static, str>);

impl OntologyName {
    pub const fn new_static(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }

    pub fn new(s: impl Into<Cow<'static, str>>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for OntologyName {
    fn from(s: &'static str) -> Self {
        Self::new_static(s)
    }
}

impl From<String> for OntologyName {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl AsRef<str> for OntologyName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OntologyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Rust module path where an ontology is defined.
///
/// This IS Rust-specific (not part of the abstract ontology).
/// The `domain()` method strips Rust-specific prefixes for display.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(Cow<'static, str>);

impl ModulePath {
    pub const fn new_static(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }

    pub fn new(s: impl Into<Cow<'static, str>>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Derive the domain path (e.g., "formal.math") from the module path.
    pub fn domain(&self) -> String {
        let s = self.0.as_ref();
        let s = s.strip_prefix("pr4xis_domains::").unwrap_or(s);
        let s = s.strip_suffix("::ontology").unwrap_or(s);
        s.replace("::", ".")
    }
}

impl From<&'static str> for ModulePath {
    fn from(s: &'static str) -> Self {
        Self::new_static(s)
    }
}

impl From<String> for ModulePath {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl AsRef<str> for ModulePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A bibliographic year — typed wrapper for a calendar year in a citation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Year(u32);

impl Year {
    pub const fn new(n: u32) -> Self {
        Self(n)
    }

    pub fn parse(s: &str) -> Option<Self> {
        s.parse::<u32>().ok().map(Self)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for Year {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Synkolation level (Heim's terminology) — depth in the Metroplex hierarchy.
///
/// Level 0 = base ontology. Each Composer composition (Heim's *Korporator*) increments by 1.
/// The runtime `Ontology` in compose.rs carries this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynkolationLevel(usize);

impl SynkolationLevel {
    pub const ZERO: Self = Self(0);

    pub const fn new(n: usize) -> Self {
        Self(n)
    }

    pub fn value(&self) -> usize {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
}

impl fmt::Display for SynkolationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Metroplex grade — an equivalence class of Ontologies at the same synkolation level.
///
/// Metroplex is indexed by Grade; each grade collects ontologies of the same
/// SynkolationLevel. Equivalent to SynkolationLevel but semantically distinct
/// (Grade is the index in the Metroplex, Level is the depth of composition).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Grade(usize);

impl Grade {
    pub const fn new(n: usize) -> Self {
        Self(n)
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

impl From<SynkolationLevel> for Grade {
    fn from(l: SynkolationLevel) -> Self {
        Self(l.0)
    }
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Source citation — structured reference to prior work.
///
/// Instance of Provenance::Citation (W3C PROV-O). Parses free-form input
/// like "Shannon (1948); Jakobson (1960)" into structured entries, each
/// with authors and year. The raw text is preserved for display.
///
/// Multiple entries are supported via `;` separator.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Citation {
    entries: Vec<CitationEntry>,
    raw: Cow<'static, str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CitationEntry {
    pub authors: Cow<'static, str>,
    pub year: Option<u32>,
}

impl Citation {
    pub const EMPTY: Self = Self {
        entries: Vec::new(),
        raw: Cow::Borrowed(""),
    };

    /// Parse a citation string from a static source.
    /// Format: "Author (Year)" or "Author; Author (Year); Author et al. (Year)"
    pub fn parse_static(s: &'static str) -> Self {
        let entries = parse_entries(s);
        Self {
            entries,
            raw: Cow::Borrowed(s),
        }
    }

    /// Parse a citation string from an owned source.
    pub fn parse(s: impl Into<String>) -> Self {
        let s = s.into();
        let entries = parse_entries(&s);
        Self {
            entries,
            raw: Cow::Owned(s),
        }
    }

    pub fn entries(&self) -> &[CitationEntry] {
        &self.entries
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
}

fn parse_entries(s: &str) -> Vec<CitationEntry> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split(';')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|part| {
            if let (Some(open), Some(close)) = (part.rfind('('), part.rfind(')'))
                && close > open
            {
                let year_str = &part[open + 1..close];
                let year = year_str.parse::<u32>().ok();
                let authors = part[..open].trim().to_string();
                return CitationEntry {
                    authors: Cow::Owned(authors),
                    year,
                };
            }
            CitationEntry {
                authors: Cow::Owned(part.to_string()),
                year: None,
            }
        })
        .collect()
}

impl From<&'static str> for Citation {
    fn from(s: &'static str) -> Self {
        Self::parse_static(s)
    }
}

impl From<String> for Citation {
    fn from(s: String) -> Self {
        Self::parse(s)
    }
}

impl AsRef<str> for Citation {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

impl fmt::Display for Citation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

/// A Vocabulary — instance of KnowledgeConcept::Vocabulary (VoID).
///
/// An ontology describing itself. Generated by `define_ontology!` or `ontology!`.
/// Fields are typed ontological concepts, not primitive strings:
///   - `ontology_name`: typed identifier
///   - `module_path`: Rust-specific location (with domain() accessor)
///   - `source`: structured Citation (authors, years, raw text)
///
/// Counts are snapshots captured at Vocabulary construction. Since
/// Vocabulary is a description (not a live view), a snapshot is correct
/// semantically — it describes what the ontology was when described.
///
/// Labels and descriptions live in the Lemon lexicon, not here.
/// Transport uses Schema Presentation, not serde.
///
/// Source: W3C VoID (2011); Spivak (2012); W3C PROV-O (2013)
/// Vocabulary carries metadata only — no counts.
///
/// Counts (concepts, morphisms) are calculated from the underlying
/// ontology via `concepts()` / `morphisms()` which return name lists.
#[derive(Debug, Clone)]
pub struct Vocabulary {
    pub ontology_name: OntologyName,
    pub module_path: ModulePath,
    pub source: Citation,
    pub being: Option<Being>,
    /// Private source of concepts/morphisms — resolved on demand.
    source_of_truth: Source,
}

#[derive(Debug, Clone)]
enum Source {
    /// Static ontology: concepts/morphisms pulled from type-driven functions each call.
    Static {
        concepts: fn() -> Vec<ConceptName>,
        morphisms: fn() -> Vec<Morphism>,
    },
    /// Runtime ontology: concepts/morphisms captured at Vocabulary construction.
    Captured {
        concepts: Vec<ConceptName>,
        morphisms: Vec<Morphism>,
    },
}

impl Vocabulary {
    pub fn domain(&self) -> String {
        self.module_path.domain()
    }

    pub fn name(&self) -> &str {
        self.ontology_name.as_str()
    }

    /// Typed concept names — calculated each call for static ontologies,
    /// cloned from the captured list for runtime ontologies.
    pub fn concepts(&self) -> Vec<ConceptName> {
        match &self.source_of_truth {
            Source::Static { concepts, .. } => concepts(),
            Source::Captured { concepts, .. } => concepts.clone(),
        }
    }

    /// Full morphisms with source/target/kind.
    pub fn morphisms(&self) -> Vec<Morphism> {
        match &self.source_of_truth {
            Source::Static { morphisms, .. } => morphisms(),
            Source::Captured { morphisms, .. } => morphisms.clone(),
        }
    }

    /// Non-allocating count of concepts — for hot paths where only the count is needed.
    /// Captured variant returns the stored length without cloning.
    pub fn concept_count(&self) -> usize {
        match &self.source_of_truth {
            Source::Static { concepts, .. } => concepts().len(),
            Source::Captured { concepts, .. } => concepts.len(),
        }
    }

    /// Non-allocating count of morphisms.
    pub fn morphism_count(&self) -> usize {
        match &self.source_of_truth {
            Source::Static { morphisms, .. } => morphisms().len(),
            Source::Captured { morphisms, .. } => morphisms.len(),
        }
    }

    /// Static Vocabulary — concepts/morphisms pulled from Category/Entity each call.
    pub fn from_static<C: crate::category::Category, E: crate::category::entity::Concept>(
        name: impl Into<OntologyName>,
        module_path: impl Into<ModulePath>,
        source: impl Into<Citation>,
        being: Option<Being>,
    ) -> Self {
        Self {
            ontology_name: name.into(),
            module_path: module_path.into(),
            source: source.into(),
            being,
            source_of_truth: Source::Static {
                concepts: || {
                    use crate::category::Concept;
                    <E as Concept>::variants()
                        .iter()
                        .map(|v| {
                            let name = v.name();
                            if name.is_empty() {
                                // Fallback when derive hasn't set name() — use Debug
                                ConceptName::new(format!("{v:?}"))
                            } else {
                                ConceptName::new(name.to_string())
                            }
                        })
                        .collect()
                },
                morphisms: || {
                    use crate::category::{Concept, Relationship};
                    let name_of = |e: &<C as crate::category::Category>::Object| -> String {
                        let n = e.name();
                        if n.is_empty() {
                            format!("{e:?}")
                        } else {
                            n.to_string()
                        }
                    };
                    <C as crate::category::Category>::morphisms()
                        .iter()
                        .map(|m| {
                            Morphism::new(
                                ConceptName::new(name_of(&m.source())),
                                ConceptName::new(name_of(&m.target())),
                                MorphismKind::Custom(Cow::Owned(format!("{m:?}"))),
                            )
                        })
                        .collect()
                },
            },
        }
    }

    /// Runtime Vocabulary — concepts/morphisms captured from instance at construction.
    pub fn from_captured(
        name: impl Into<OntologyName>,
        module_path: impl Into<ModulePath>,
        source: impl Into<Citation>,
        being: Option<Being>,
        concepts: Vec<ConceptName>,
        morphisms: Vec<Morphism>,
    ) -> Self {
        Self {
            ontology_name: name.into(),
            module_path: module_path.into(),
            source: source.into(),
            being,
            source_of_truth: Source::Captured {
                concepts,
                morphisms,
            },
        }
    }

    /// Compatibility shim for `manual::<C, E>()` calls in descriptor.rs.
    pub fn from_ontology<C: crate::category::Category, E: crate::category::entity::Concept>(
        name: impl Into<OntologyName>,
        module_path: impl Into<ModulePath>,
        source: impl Into<Citation>,
        being: Option<Being>,
    ) -> Self {
        Self::from_static::<C, E>(name, module_path, source, being)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ontology_name_from_static_is_borrowed() {
        let name = OntologyName::new_static("Biology");
        assert_eq!(name.as_str(), "Biology");
        assert!(matches!(name.0, Cow::Borrowed(_)));
    }

    #[test]
    fn ontology_name_from_owned_is_owned() {
        let name = OntologyName::new(String::from("Runtime"));
        assert_eq!(name.as_str(), "Runtime");
        assert!(matches!(name.0, Cow::Owned(_)));
    }

    #[test]
    fn citation_parses_single_entry() {
        let c = Citation::parse_static("Shannon (1948)");
        assert_eq!(c.entries().len(), 1);
        assert_eq!(c.entries()[0].authors, "Shannon");
        assert_eq!(c.entries()[0].year, Some(1948));
    }

    #[test]
    fn citation_parses_multiple_entries() {
        let c = Citation::parse_static("Shannon (1948); Jakobson (1960); Wiener (1948)");
        assert_eq!(c.entries().len(), 3);
        assert_eq!(c.entries()[0].authors, "Shannon");
        assert_eq!(c.entries()[1].authors, "Jakobson");
        assert_eq!(c.entries()[2].authors, "Wiener");
    }

    #[test]
    fn citation_parses_et_al() {
        let c = Citation::parse_static("McCrae et al. (2012, 2017)");
        assert_eq!(c.entries().len(), 1);
        assert_eq!(c.entries()[0].authors, "McCrae et al.");
        // Year can't be parsed from "2012, 2017" — ok, year stays None
    }

    #[test]
    fn citation_empty_string() {
        let c = Citation::parse_static("");
        assert!(c.is_empty());
        assert_eq!(c.entries().len(), 0);
    }

    #[test]
    fn citation_roundtrips_through_display() {
        let c = Citation::parse_static("Shannon (1948); Jakobson (1960)");
        assert_eq!(format!("{c}"), "Shannon (1948); Jakobson (1960)");
    }

    #[test]
    fn module_path_domain_strips_prefixes() {
        let p = ModulePath::new_static("pr4xis_domains::formal::math::ontology");
        assert_eq!(p.domain(), "formal.math");
    }

    #[test]
    fn wrappers_accept_static_str_and_string() {
        let _: OntologyName = "literal".into();
        let _: OntologyName = String::from("owned").into();
        let _: ModulePath = "a::b".into();
        let _: Citation = String::from("Author (2024)").into();
    }
}

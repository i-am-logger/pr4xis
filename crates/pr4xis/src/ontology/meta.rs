use std::borrow::Cow;
use std::fmt;

use crate::ontology::upper::being::Being;

/// Metadata about an ontology — for tracing and introspection.
///
/// Stays as `&'static str` because it's only constructed at compile time
/// from `stringify!` and `module_path!()`. Keeping Copy is valuable here.
#[derive(Debug, Clone, Copy)]
pub struct OntologyMeta {
    pub name: &'static str,
    pub module_path: &'static str,
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
        concepts: fn() -> Vec<String>,
        morphisms: fn() -> Vec<String>,
    },
    /// Runtime ontology: concepts/morphisms captured at Vocabulary construction.
    Captured {
        concepts: Vec<String>,
        morphisms: Vec<String>,
    },
}

impl Vocabulary {
    pub fn domain(&self) -> String {
        self.module_path.domain()
    }

    pub fn name(&self) -> &str {
        self.ontology_name.as_str()
    }

    /// List of concept names — calculated each call for static ontologies,
    /// captured from the instance for runtime ontologies.
    /// Call `.len()` for the count.
    pub fn concepts(&self) -> Vec<String> {
        match &self.source_of_truth {
            Source::Static { concepts, .. } => concepts(),
            Source::Captured { concepts, .. } => concepts.clone(),
        }
    }

    /// List of morphism identifiers. Call `.len()` for the count.
    pub fn morphisms(&self) -> Vec<String> {
        match &self.source_of_truth {
            Source::Static { morphisms, .. } => morphisms(),
            Source::Captured { morphisms, .. } => morphisms.clone(),
        }
    }

    /// Static Vocabulary — concepts/morphisms pulled from Category/Entity each call.
    pub fn from_static<C: crate::category::Category, E: crate::category::entity::Entity>(
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
                    <E as crate::category::entity::Entity>::variants()
                        .iter()
                        .map(|v| format!("{v:?}"))
                        .collect()
                },
                morphisms: || {
                    <C as crate::category::Category>::morphisms()
                        .iter()
                        .map(|m| format!("{m:?}"))
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
        concepts: Vec<String>,
        morphisms: Vec<String>,
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
    pub fn from_ontology<C: crate::category::Category, E: crate::category::entity::Entity>(
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

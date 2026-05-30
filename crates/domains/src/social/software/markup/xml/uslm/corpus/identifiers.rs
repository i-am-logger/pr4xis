//! `UsCodeTitleId` — typed identifier for a U.S. Code title.
//!
//! The LRC's USLM URN scheme uses `/us/usc/t<N>` as the canonical
//! identifier for a title (User Guide §V; 1 U.S.C. § 204 authorizes
//! the LRC to publish the USC). The integer N is a *projection* of
//! the URN, not a separate field — keeping the URN as the truth
//! source means UsCodeTitleId composes directly with the
//! identifier_format ontology, with no parallel "number" storage to
//! drift.
//!
//! Connection to other ontologies:
//! - identifier_format: UsCodeTitleId wraps Identifier with the
//!   UslmUrn format leaf. Grammar validation is delegated.
//! - source_taxonomy: a UsCodeTitleId instance is-a of
//!   SourceTaxonomyConcept::UsCodeTitle (the *kind*) — UsCodeTitleId
//!   is the *which* (Title 18 vs Title 49).
//! - English: the citation-form text ("18 U.S.C.", "title 18 of the
//!   United States Code") is an English noun phrase — produced by
//!   functor from UsCodeTitleId, lemmatized via English morphology.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec::Vec};

use crate::formal::meta::identifier_format::{Identifier, IdentifierParseError};

/// Typed identifier for a U.S. Code title — the *which* (Title 18,
/// Title 49, …) distinct from the *kind* (the
/// `SourceTaxonomyConcept::UsCodeTitle` concept).
///
/// Wraps an [`Identifier`] of format
/// [`IdentifierFormatConcept::UslmUrn`][crate::formal::meta::identifier_format::ontology::IdentifierFormatConcept::UslmUrn]
/// with the path `/us/usc/t<N>`. The numeric title number is a
/// method, derived from the URN.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UsCodeTitleId {
    identifier: Identifier,
}

/// Errors when constructing a [`UsCodeTitleId`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsCodeTitleIdError {
    /// The supplied URN/string failed USLM URN grammar validation.
    BadUrn(IdentifierParseError),
    /// The URN parsed as a valid USLM path, but its shape isn't
    /// `/us/usc/t<N>` — e.g. it points at a section or chapter, not
    /// a title.
    NotATitleUrn,
    /// The `<N>` part isn't a positive integer in `1..=54` (the
    /// authorized range of USC titles per 1 U.S.C. § 204).
    OutOfRange { raw: String },
    /// The praxis.toml source-name string isn't of the form
    /// `usc_title_<N>`.
    BadSourceName { raw: String },
}

impl UsCodeTitleId {
    /// Lift a numeric title number into a UsCodeTitleId.
    ///
    /// Accepted range: 1..=54 — the USC's currently authorized
    /// titles per 1 U.S.C. § 204. (Some numbers are reserved or
    /// unpublished — `try_from_number(2)` for instance — but the
    /// schema-level validity covers the structural range; semantic
    /// "this title is currently published" is a different ontology.)
    pub fn try_from_number(number: u32) -> Result<Self, UsCodeTitleIdError> {
        if !(1..=54).contains(&number) {
            return Err(UsCodeTitleIdError::OutOfRange {
                raw: number.to_string(),
            });
        }
        let urn = format!("/us/usc/t{number}");
        let identifier = Identifier::uslm_urn(urn).map_err(UsCodeTitleIdError::BadUrn)?;
        Ok(Self { identifier })
    }

    /// Lift a USLM URN string (e.g. `"/us/usc/t18"`) into a
    /// UsCodeTitleId. Validates that the path matches the
    /// title-level shape; lower paths (`/us/usc/t18/s1514A`) are
    /// rejected.
    pub fn try_from_urn(urn: impl Into<String>) -> Result<Self, UsCodeTitleIdError> {
        let urn = urn.into();
        let identifier = Identifier::uslm_urn(&urn).map_err(UsCodeTitleIdError::BadUrn)?;
        let segments: Vec<&str> = urn.trim_start_matches('/').split('/').collect();
        // Must be exactly: ["us", "usc", "t<N>"].
        if segments.len() != 3 || segments[0] != "us" || segments[1] != "usc" {
            return Err(UsCodeTitleIdError::NotATitleUrn);
        }
        let Some(num_str) = segments[2].strip_prefix('t') else {
            return Err(UsCodeTitleIdError::NotATitleUrn);
        };
        let Ok(n) = num_str.parse::<u32>() else {
            return Err(UsCodeTitleIdError::NotATitleUrn);
        };
        if !(1..=54).contains(&n) {
            return Err(UsCodeTitleIdError::OutOfRange {
                raw: num_str.to_string(),
            });
        }
        Ok(Self { identifier })
    }

    /// Parse a praxis.toml source-name key (e.g. `"usc_title_18"`)
    /// into a UsCodeTitleId. The convention is documented at the
    /// data_provisioning registry layer; this is the single point
    /// where the convention is recognized and dissolved into the
    /// typed ontology value.
    pub fn try_from_source_name(name: &str) -> Result<Self, UsCodeTitleIdError> {
        let Some(num_str) = name.strip_prefix("usc_title_") else {
            return Err(UsCodeTitleIdError::BadSourceName {
                raw: name.to_string(),
            });
        };
        let Ok(n) = num_str.parse::<u32>() else {
            return Err(UsCodeTitleIdError::BadSourceName {
                raw: name.to_string(),
            });
        };
        Self::try_from_number(n)
    }

    /// The title's numeric position in the U.S. Code (e.g. 18 for
    /// Title 18). Derived from the URN — single source of truth.
    pub fn number(&self) -> u32 {
        // The constructor validates the URN shape, so this parse
        // cannot fail.
        let segments: Vec<&str> = self
            .identifier
            .value()
            .trim_start_matches('/')
            .split('/')
            .collect();
        segments[2]
            .strip_prefix('t')
            .and_then(|s| s.parse::<u32>().ok())
            .expect("UsCodeTitleId invariant: URN path is /us/usc/t<N>")
    }

    /// The USLM URN identifier (e.g. `"/us/usc/t18"`).
    pub fn urn(&self) -> &str {
        self.identifier.value()
    }

    /// The underlying typed [`Identifier`] — useful for callers
    /// that need to compose with the identifier_format ontology.
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    /// The praxis.toml source-name convention key for this title,
    /// e.g. `"usc_title_18"`. Inverse of
    /// [`try_from_source_name`][Self::try_from_source_name].
    pub fn source_name(&self) -> String {
        format!("usc_title_{}", self.number())
    }

    /// Bluebook short citation form ("18 U.S.C.").
    ///
    /// Citation: The Bluebook: A Uniform System of Citation, 21st
    /// ed., Rule 12.1 (Basic Citation Forms) + Table T1.1. This is the English
    /// noun-phrase projection — the output is English text that
    /// downstream NLP can lemmatize / tokenize.
    pub fn short_citation(&self) -> String {
        format!("{} U.S.C.", self.number())
    }

    /// Bluebook long citation form ("title 18 of the United
    /// States Code").
    pub fn long_citation(&self) -> String {
        format!("title {} of the United States Code", self.number())
    }
}

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
//!   United States Code", "title 18") is an English noun phrase —
//!   produced by functor from UsCodeTitleId, lemmatized via English
//!   morphology. The forms are enumerated by
//!   [`UsCodeTitleCitationForm`], each carrying its own authority, so a
//!   consumer covers "every way this title is named" by iterating
//!   rather than by remembering to call three accessors.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec::Vec};

use crate::applied::data_provisioning::registry::data_sources;
use crate::formal::meta::identifier_format::{Identifier, IdentifierParseError};
use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

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

    /// This title's text in one of its published citation forms — the
    /// English noun-phrase projection of the URN, the output being English
    /// text downstream NLP can tokenize / lemmatize.
    ///
    /// The FORM is the parameter, so every published way of naming a title
    /// is one enumeration ([`UsCodeTitleCitationForm`], which carries each
    /// form's own authority) with one derivation from [`number`](Self::number).
    /// A consumer that must cover "every way this title is named" iterates
    /// [`UsCodeTitleCitationForm::ALL`] rather than listing accessors it might
    /// under-enumerate — the failure that let the bare designation ("title 42")
    /// go unrecognized while the other two forms resolved.
    pub fn citation(&self, form: UsCodeTitleCitationForm) -> String {
        let n = self.number();
        match form {
            UsCodeTitleCitationForm::Short => format!("{n} U.S.C."),
            UsCodeTitleCitationForm::Long => format!("title {n} of the United States Code"),
            UsCodeTitleCitationForm::Designation => format!("title {n}"),
        }
    }

    /// Every published citation form of THIS title, in
    /// [`UsCodeTitleCitationForm::ALL`] order — the complete set of surfaces a
    /// reader may name it by.
    pub fn citations(&self) -> impl Iterator<Item = String> + '_ {
        UsCodeTitleCitationForm::ALL
            .iter()
            .map(|&form| self.citation(form))
    }
}

/// The published forms in which a U.S. Code title is NAMED — each a distinct
/// convention with its own authority, not three spellings of one string.
///
/// A title is cited three ways in practice, and a reader types whichever the
/// context taught them. Enumerating them here (rather than as free-standing
/// accessors) makes "all the ways this title is named" a value the compiler
/// can hand a consumer whole: adding a fourth attested form extends every
/// consumer — the citation router, the definitional lexicon — at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UsCodeTitleCitationForm {
    /// The Bluebook short form — `"18 U.S.C."`.
    ///
    /// Citation: The Bluebook: A Uniform System of Citation, 21st ed.,
    /// Rule 12.1 (Basic Citation Forms) + Table T1.1.
    Short,
    /// The spelled-out cross-reference form — `"title 18 of the United States
    /// Code"`.
    ///
    /// Citation: Congress's OWN enacted usage — 18 U.S.C. § 2516(1)(a) names
    /// its predicate offenses as arising "under sections 2122 and 2274 through
    /// 2277 of title 42 of the United States Code". This is the statutory
    /// register, attested in the corpus this engine loads, not a style-guide
    /// recommendation.
    Long,
    /// The publisher's bare designation — `"title 18"`.
    ///
    /// Citation: the Law Revision Counsel's own USLM markup, which opens each
    /// title with `<num value="42">Title 42—</num><heading>THE PUBLIC HEALTH
    /// AND WELFARE</heading>` (published per 1 U.S.C. § 204; USLM User Guide
    /// §V). The `<num>` text minus its em-dash separator IS how the authority
    /// designates the title, and it is the form a reader asking "what is title
    /// 42" uses. Neither of the other two forms contains it, and no mechanical
    /// alias derives it from them, so an enumeration missing this leaf leaves
    /// the commonest phrasing unrecognized.
    Designation,
}

impl UsCodeTitleCitationForm {
    /// Every published form, in the order declared above (short → long →
    /// designation). Iterating this is the ONLY correct way to cover "all the
    /// ways a title is named"; a consumer naming forms individually silently
    /// under-covers when a form is added.
    pub const ALL: &'static [Self] = &[Self::Short, Self::Long, Self::Designation];
}

/// Which REGISTERED U.S. Code title, if any, does `surface` cite?
///
/// The routing question behind an abstention: the engine could not ground
/// "42 u.s.c § 300ii", and the reader wants to know which source would let it.
/// The answer is a registry source name (`usc_title_42`) — but a raw citation
/// surface matches no source name, so something has to bridge them.
///
/// That bridge used to live in the demonstrator's JavaScript, as a regular
/// expression over the citation form plus a hand-built `usc_title_${n}`. Both
/// are facts this module already owns — the second is literally
/// [`UsCodeTitleId::source_name`], whose inverse
/// [`UsCodeTitleId::try_from_source_name`] documents itself as "the single
/// point where the convention is recognized". Having a second point, in
/// another language, in the client, is exactly the drift that convention
/// exists to prevent.
///
/// So recognition here compares the surface against each REGISTERED title's
/// OWN published citation forms ([`UsCodeTitleId::citations`], every leaf of
/// [`UsCodeTitleCitationForm::ALL`]) rather than against a pattern describing
/// how citations are written. Nothing about citation syntax is encoded: add a
/// title to the registry and it becomes recognizable with no code change;
/// change the naming convention and only `source_name` moves; attest a fourth
/// citation form and it is recognized here without this function changing.
///
/// Comparison is over an orthographic key — case folded, punctuation dropped —
/// because "42 U.S.C.", "42 USC" and "42 u.s.c." are the same citation
/// differently typeset (Bluebook 21st ed. Rule 12.1 gives the canonical form;
/// readers type all three).
///
/// The match is the LONGEST one, not the first: title designations nest
/// orthographically ("title5" is a prefix of "title50"), so a first-hit scan
/// would resolve "title 50" to Title 5 or Title 50 depending only on where the
/// registry happened to sort them. Longest-key wins is the same
/// maximal-munch discipline the tokenizer's multiword collapse uses, and it
/// makes the answer a property of the citation rather than of registry order.
///
/// **Returns `None` freely, and that is not a shortfall.** This recognizes
/// citations that name a title in one of its published forms; a surface that
/// names one some other way ("the Public Health and Welfare title") resolves to
/// nothing, and the caller falls back to filtering the catalog by the surface
/// itself. An honest absence beats a guess: the alternative is inventing a
/// citation grammar here, which is the thing being removed.
pub fn title_cited_by(surface: &str) -> Option<UsCodeTitleId> {
    let key = citation_key(surface);
    if key.is_empty() {
        return None;
    }
    registered_titles()
        .filter_map(|id| {
            // The longest of THIS title's forms that the surface carries — its
            // strength of evidence. `None` when the surface carries none.
            id.citations()
                .map(|c| citation_key(&c))
                .filter(|c| !c.is_empty() && key.contains(c))
                .map(|c| c.len())
                .max()
                .map(|len| (len, id))
        })
        // Strictly-greater keeps the FIRST title at a given strength, so a
        // genuine tie (two titles matched by equally long keys — impossible
        // for distinct numbers today, but not a property to rely on silently)
        // resolves by registry order rather than by iteration accident.
        .fold(
            None,
            |best: Option<(usize, UsCodeTitleId)>, (len, id)| match &best {
                Some((best_len, _)) if *best_len >= len => best,
                _ => Some((len, id)),
            },
        )
        .map(|(_, id)| id)
}

/// Every REGISTERED U.S. Code title as a typed id — the registry filtered to
/// the `UsCodeTitle` kind and dissolved through
/// [`UsCodeTitleId::try_from_source_name`]. Registration is a claim about what
/// this deployment KNOWS OF, not about what it currently HOLDS; a consumer that
/// needs the held set must derive it from loaded content (see
/// `usc_title_lexicon::loaded_titles`).
pub fn registered_titles() -> impl Iterator<Item = UsCodeTitleId> {
    data_sources()
        .iter()
        .filter(|e| e.kind == SourceTaxonomyConcept::UsCodeTitle)
        .filter_map(|e| UsCodeTitleId::try_from_source_name(&e.name).ok())
}

/// Fold a citation surface to its orthographic key: lowercase, alphanumerics
/// only. Typesetting differences ("U.S.C." / "USC" / "u.s.c.") collapse;
/// nothing about citation STRUCTURE is interpreted.
fn citation_key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

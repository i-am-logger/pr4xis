//! Projection: a `citations.toml` registry entry → a typed
//! [`CitationAssessment`].
//!
//! The audit gate used to inspect one raw field (`verified_by`) of each
//! registry entry. This module replaces that flat check with a typed
//! projection into the [`super::ontology`] dimensions: it reads the
//! grounded fields of an entry and decides, *per dimension*, whether
//! that dimension is [`DimensionStatus::Verified`] or `Unverified`. The
//! gate then folds the resulting [`CitationAssessment`] through
//! [`assess`](super::assessment::assess) to a
//! [`CitationVerdict`](super::assessment::CitationVerdict) and reports
//! *which* dimension fails — strengthening the old single-field gate
//! into a dimensional one.
//!
//! The projection takes a small, self-contained [`EntryFields`] /
//! [`VersionFiber`] view rather than depending on the audit test's
//! deserialization structs: the test fills these from its `Citation`
//! schema, and any other caller (a future `toml-bytes ⇄ record` lens)
//! can fill them too. Each field→dimension rule below is justified by
//! that dimension's definition in [`super::ontology`] — this is a
//! grounded mapping, not an arbitrary one.
//!
//! # Per-dimension mapping (each grounded in [`super::ontology`])
//!
//! - **Existence** ([`Existence`](super::ontology::CitationQualityConcept::Existence))
//!   — "whether the cited work exists at all, as opposed to being
//!   fabricated." A work's identity is established by a named author, a
//!   title, and at least one identifier (year, DOI, URL, or ISBN). With
//!   all three present the entry names a locatable work; missing any one
//!   leaves the identity unestablished.
//! - **ClaimSupport** ([`ClaimSupport`](super::ontology::CitationQualityConcept::ClaimSupport))
//!   — "whether the cited work actually supports the asserted claim."
//!   Established by a non-empty `content_summary` (a statement of what
//!   the section says) that has been confirmed (`verified_by` non-empty).
//!   This is the dimension the old flat `verified_by` gate covered.
//! - **LocatorAccuracy** ([`LocatorAccuracy`](super::ontology::CitationQualityConcept::LocatorAccuracy))
//!   — "whether the pinpoint locator resolves to the right place."
//!   Established by a non-empty `section_or_page`.
//! - **BibliographicAccuracy** ([`BibliographicAccuracy`](super::ontology::CitationQualityConcept::BibliographicAccuracy))
//!   — "whether author, title, edition, and year are correct." Requires
//!   author, title, publisher, and year all present.
//! - **FormatConformance** ([`FormatConformance`](super::ontology::CitationQualityConcept::FormatConformance))
//!   — "whether the citation conforms to the required style." An entry
//!   that deserialized under the registry schema conforms to it by
//!   construction, so this dimension is `Verified` for every entry the
//!   loader accepts (the Info-severity dimension).
//!
//! ## Multi-version entries
//!
//! An entry whose work has several published versions (the version
//! adjunction — see the `citations.toml` header) carries a
//! version-independent `claim` and one [`VersionFiber`] per version,
//! with the flat `section_or_page` / `verified_by` left empty.
//! Existence and BibliographicAccuracy take author/title from the flat
//! entry (these identify the *work*, version-independent), but the
//! *identifier* (year / DOI / URL / ISBN) of a multi-version work is
//! version-located — each `[[versions]]` fiber carries its own
//! `year`/`url` — so Existence's identifier requirement is satisfied
//! when every fiber carries one, just as ClaimSupport and
//! LocatorAccuracy read per-fiber. ClaimSupport requires the flat
//! `claim` plus *every* fiber being confirmed; LocatorAccuracy requires
//! *every* fiber having a locator — the adjunction must be total over
//! the registered versions.
//!
//! # Literature
//!
//! See [`super::ontology`] for the dimension definitions and their
//! grounding (ISO/IEC 25012:2008; Wang & Strong 1996; Sarol et al. 2024;
//! Guyatt et al. 2008 GRADE).

#[allow(unused_imports)]
use alloc::{string::String, vec::Vec};

use super::assessment::{DimensionStatus, VerificationMethod};
use super::record_lens::CitationAssessment;

/// One version-fiber of a multi-version registry entry: a locator and a
/// confirmation flag for the entry's `claim` *in a single published
/// version*. The caller (audit test / future lens) fills these from the
/// `[[citations.<slug>.versions]]` tables.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VersionFiber {
    /// Whether this fiber carries a non-empty `section_or_page`.
    pub has_locator: bool,
    /// Whether this fiber carries a non-empty `verified_by`.
    pub is_verified: bool,
    /// Whether this fiber carries an identifier (`year` set OR a
    /// non-empty `url`) — the version-located identity of the work
    /// (Existence). A multi-version work's identifier lives per-version,
    /// not on the flat block.
    pub has_identifier: bool,
}

/// The grounded subset of a `citations.toml` entry the projection reads.
/// Each `bool` is "the corresponding registry field is present
/// (non-empty / set)" — the caller computes presence so the projection
/// stays free of the registry's deserialization types.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntryFields {
    /// `authors` non-empty.
    pub has_authors: bool,
    /// `title` non-empty.
    pub has_title: bool,
    /// `publisher` non-empty.
    pub has_publisher: bool,
    /// `year` set.
    pub has_year: bool,
    /// At least one of `year` / `doi` / `url` / `isbn` present — the
    /// identifier that, with author + title, establishes the work's
    /// identity (Existence).
    pub has_identifier: bool,
    /// `section_or_page` non-empty (the flat locator; empty for
    /// multi-version entries).
    pub has_section_or_page: bool,
    /// `content_summary` non-empty.
    pub has_content_summary: bool,
    /// `verified_by` non-empty (the flat confirmation; empty for
    /// multi-version entries).
    pub has_verified_by: bool,
    /// `claim` non-empty (the version-independent statement; only used
    /// by multi-version entries).
    pub has_claim: bool,
    /// The raw `verification_method` string, parsed by
    /// [`parse_verification_method`].
    pub verification_method: String,
    /// The version fibers, empty for single-version entries.
    pub versions: Vec<VersionFiber>,
}

/// Parse a registry `verification_method` string into the typed
/// [`VerificationMethod`]. Recognizes the machine / human families and
/// falls back to `Unverified` for anything else (including the empty
/// string and the registry's human-workflow labels such as
/// `"web-fetched"` / `"book-in-hand"`, which describe *how a human
/// consulted the source* — a human attestation rather than a
/// reproducible machine check).
pub fn parse_verification_method(raw: &str) -> VerificationMethod {
    // Normalize: lowercase, drop separators, so "machine_checked",
    // "machine-checked", and "MachineChecked" all collapse to one key.
    let key: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    match key.as_str() {
        "machine" | "machinechecked" => VerificationMethod::MachineChecked,
        "human" | "humanattested" => VerificationMethod::HumanAttested,
        // The registry's source-consultation labels are human-mediated:
        // a person fetched / read the work and attested. Daubert: the
        // reliability rests on the attester, so HumanAttested.
        "webfetched" | "bookinhand" | "doiresolved" | "libraryloan" => {
            VerificationMethod::HumanAttested
        }
        _ => VerificationMethod::Unverified,
    }
}

/// Project a registry entry into a typed [`CitationAssessment`].
///
/// Each dimension's Verified/Unverified decision follows the rule
/// documented on the module, grounded in that dimension's definition in
/// [`super::ontology`]. The returned assessment folds (via
/// [`CitationAssessment::verdict`]) to the entry's dimensional verdict.
pub fn project_entry(fields: &EntryFields) -> CitationAssessment {
    let multi_version = !fields.versions.is_empty();

    // Existence — the work's identity: author + title (version-
    // independent) + an identifier. For a single-version entry the
    // identifier is the flat year/doi/url/isbn; for a multi-version
    // entry it is version-located, so it is established when every fiber
    // carries one (year or url) — mirroring how ClaimSupport and
    // LocatorAccuracy read per-fiber. (ontology: Existence — "whether
    // the cited work exists at all, as opposed to being fabricated".)
    let identifier_present = if multi_version {
        fields.versions.iter().all(|v| v.has_identifier)
    } else {
        fields.has_identifier
    };
    let existence = verified_if(fields.has_authors && fields.has_title && identifier_present);

    // BibliographicAccuracy — author + title + publisher + year, all of
    // the *work* (version-independent). (ontology: BibliographicAccuracy
    // — "whether author, title, edition, and year are correct".)
    let bibliographic_accuracy = verified_if(
        fields.has_authors && fields.has_title && fields.has_publisher && fields.has_year,
    );

    // ClaimSupport — the sound-gate dimension covered by the old flat
    // gate. (ontology: ClaimSupport — "whether the cited work actually
    // supports the asserted claim".)
    let claim_support = if multi_version {
        // The version-independent claim must be stated and confirmed in
        // every registered version (the adjunction is total).
        verified_if(fields.has_claim && fields.versions.iter().all(|v| v.is_verified))
    } else {
        verified_if(fields.has_content_summary && fields.has_verified_by)
    };

    // LocatorAccuracy — a resolvable pinpoint. (ontology:
    // LocatorAccuracy — "whether the pinpoint locator resolves to the
    // right place".)
    let locator_accuracy = if multi_version {
        verified_if(fields.versions.iter().all(|v| v.has_locator))
    } else {
        verified_if(fields.has_section_or_page)
    };

    // FormatConformance — the entry deserialized under the schema, so it
    // conforms by construction. (ontology: FormatConformance —
    // "whether the citation conforms to the required style"; Info
    // severity.)
    let format_conformance = DimensionStatus::Verified;

    CitationAssessment {
        existence,
        claim_support,
        locator_accuracy,
        bibliographic_accuracy,
        format_conformance,
        method: parse_verification_method(&fields.verification_method),
    }
}

/// `Verified` iff the grounded condition holds, else `Unverified` — the
/// conservative default (an unconfirmed dimension is treated as not yet
/// established; see [`DimensionStatus`]).
fn verified_if(cond: bool) -> DimensionStatus {
    if cond {
        DimensionStatus::Verified
    } else {
        DimensionStatus::Unverified
    }
}

// =============================================================================
// Tests — one per dimension proving the Verified/Unverified boundary.
// Each references the grounding dimension in `super::ontology`.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formal::meta::citation_quality::assessment::CitationVerdict;

    /// A fully-populated single-version entry — every dimension Verified.
    fn full_entry() -> EntryFields {
        EntryFields {
            has_authors: true,
            has_title: true,
            has_publisher: true,
            has_year: true,
            has_identifier: true,
            has_section_or_page: true,
            has_content_summary: true,
            has_verified_by: true,
            has_claim: false,
            verification_method: String::from("web-fetched"),
            versions: Vec::new(),
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn full_entry_is_all_verified_and_valid() {
        let a = project_entry(&full_entry());
        assert_eq!(a.existence, DimensionStatus::Verified);
        assert_eq!(a.claim_support, DimensionStatus::Verified);
        assert_eq!(a.locator_accuracy, DimensionStatus::Verified);
        assert_eq!(a.bibliographic_accuracy, DimensionStatus::Verified);
        assert_eq!(a.format_conformance, DimensionStatus::Verified);
        assert_eq!(a.verdict(), CitationVerdict::Valid);
    }

    // ── Existence boundary (ontology::Existence) ───────────────────
    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn existence_needs_authors_title_and_identifier() {
        // Drop the identifier → Existence Unverified.
        let no_id = EntryFields {
            has_identifier: false,
            ..full_entry()
        };
        assert_eq!(project_entry(&no_id).existence, DimensionStatus::Unverified);
        // Drop the authors → Existence Unverified.
        let no_authors = EntryFields {
            has_authors: false,
            ..full_entry()
        };
        assert_eq!(
            project_entry(&no_authors).existence,
            DimensionStatus::Unverified
        );
        // Drop the title → Existence Unverified.
        let no_title = EntryFields {
            has_title: false,
            ..full_entry()
        };
        assert_eq!(
            project_entry(&no_title).existence,
            DimensionStatus::Unverified
        );
    }

    // ── ClaimSupport boundary (ontology::ClaimSupport) ─────────────
    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn claim_support_needs_summary_and_verified_by() {
        let no_summary = EntryFields {
            has_content_summary: false,
            ..full_entry()
        };
        assert_eq!(
            project_entry(&no_summary).claim_support,
            DimensionStatus::Unverified
        );
        let unverified = EntryFields {
            has_verified_by: false,
            ..full_entry()
        };
        assert_eq!(
            project_entry(&unverified).claim_support,
            DimensionStatus::Unverified
        );
        // Sound gate: an unconfirmed ClaimSupport drives Invalid.
        assert_eq!(
            project_entry(&unverified).verdict(),
            CitationVerdict::Invalid
        );
    }

    // ── LocatorAccuracy boundary (ontology::LocatorAccuracy) ───────
    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn locator_accuracy_needs_section_or_page() {
        let no_locator = EntryFields {
            has_section_or_page: false,
            ..full_entry()
        };
        let a = project_entry(&no_locator);
        assert_eq!(a.locator_accuracy, DimensionStatus::Unverified);
        // Non-blocking: only ValidWithIssues, not Invalid.
        assert_eq!(a.verdict(), CitationVerdict::ValidWithIssues);
    }

    // ── BibliographicAccuracy boundary ─────────────────────────────
    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn bibliographic_accuracy_needs_author_title_publisher_year() {
        // Dropping any one of the four required fields unverifies
        // BibliographicAccuracy; the full entry has it Verified.
        let drop_authors = EntryFields {
            has_authors: false,
            ..full_entry()
        };
        let drop_title = EntryFields {
            has_title: false,
            ..full_entry()
        };
        let drop_publisher = EntryFields {
            has_publisher: false,
            ..full_entry()
        };
        let drop_year = EntryFields {
            has_year: false,
            ..full_entry()
        };
        for f in [drop_authors, drop_title, drop_publisher, drop_year] {
            assert_eq!(
                project_entry(&f).bibliographic_accuracy,
                DimensionStatus::Unverified
            );
        }
        assert_eq!(
            project_entry(&full_entry()).bibliographic_accuracy,
            DimensionStatus::Verified
        );
    }

    // ── FormatConformance (ontology::FormatConformance) ────────────
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn format_conformance_always_verified_for_loaded_entry() {
        // Even a near-empty entry conforms to the schema by construction.
        let empty = EntryFields::default();
        assert_eq!(
            project_entry(&empty).format_conformance,
            DimensionStatus::Verified
        );
    }

    // ── VerificationMethod parsing (assessment::VerificationMethod) ─
    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn verification_method_parses_families() {
        assert_eq!(
            parse_verification_method("machine"),
            VerificationMethod::MachineChecked
        );
        assert_eq!(
            parse_verification_method("machine_checked"),
            VerificationMethod::MachineChecked
        );
        assert_eq!(
            parse_verification_method("MachineChecked"),
            VerificationMethod::MachineChecked
        );
        assert_eq!(
            parse_verification_method("human"),
            VerificationMethod::HumanAttested
        );
        assert_eq!(
            parse_verification_method("human_attested"),
            VerificationMethod::HumanAttested
        );
        // Registry source-consultation labels are human attestations.
        assert_eq!(
            parse_verification_method("web-fetched"),
            VerificationMethod::HumanAttested
        );
        assert_eq!(
            parse_verification_method("book-in-hand"),
            VerificationMethod::HumanAttested
        );
        // Anything unrecognized / empty → Unverified.
        assert_eq!(
            parse_verification_method(""),
            VerificationMethod::Unverified
        );
        assert_eq!(
            parse_verification_method("guessed"),
            VerificationMethod::Unverified
        );
    }

    // ── Multi-version projection ───────────────────────────────────
    fn multi_version_entry(all_verified: bool, all_located: bool, has_claim: bool) -> EntryFields {
        EntryFields {
            has_authors: true,
            has_title: true,
            has_publisher: true,
            has_year: true,
            has_identifier: true,
            // Flat locator / verified_by are empty for versioned entries.
            has_section_or_page: false,
            has_content_summary: false,
            has_verified_by: false,
            has_claim,
            verification_method: String::from("web-fetched"),
            versions: alloc::vec![
                VersionFiber {
                    has_locator: all_located,
                    is_verified: all_verified,
                    has_identifier: true,
                },
                VersionFiber {
                    has_locator: all_located,
                    is_verified: all_verified,
                    has_identifier: true,
                },
            ],
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn multi_version_existence_from_flat_entry() {
        // Author/title come from the flat work fields; Bibliographic too.
        let e = multi_version_entry(true, true, true);
        let a = project_entry(&e);
        assert_eq!(a.existence, DimensionStatus::Verified);
        assert_eq!(a.bibliographic_accuracy, DimensionStatus::Verified);
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn multi_version_existence_identifier_is_per_fiber() {
        // A multi-version work's identifier (year/url) is version-located:
        // Existence requires every fiber to carry one. The flat block has
        // no year/url/isbn/doi (has_identifier defaults false), so the
        // fiber identifiers are what establish Existence here.
        let all_id = multi_version_entry(true, true, true); // fibers have ids
        assert_eq!(project_entry(&all_id).existence, DimensionStatus::Verified);
        // Drop an identifier on one fiber → Existence Unverified (blocking).
        let one_no_id = EntryFields {
            versions: alloc::vec![
                VersionFiber {
                    has_locator: true,
                    is_verified: true,
                    has_identifier: true,
                },
                VersionFiber {
                    has_locator: true,
                    is_verified: true,
                    has_identifier: false,
                },
            ],
            ..multi_version_entry(true, true, true)
        };
        let a = project_entry(&one_no_id);
        assert_eq!(a.existence, DimensionStatus::Unverified);
        assert_eq!(a.verdict(), CitationVerdict::Invalid);
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn multi_version_claim_support_needs_claim_and_all_fibers_verified() {
        // claim + all fibers verified → ClaimSupport Verified.
        assert_eq!(
            project_entry(&multi_version_entry(true, true, true)).claim_support,
            DimensionStatus::Verified
        );
        // Missing claim → Unverified.
        assert_eq!(
            project_entry(&multi_version_entry(true, true, false)).claim_support,
            DimensionStatus::Unverified
        );
        // A fiber unverified → Unverified.
        let one_bad = EntryFields {
            versions: alloc::vec![
                VersionFiber {
                    has_locator: true,
                    is_verified: true,
                    has_identifier: true,
                },
                VersionFiber {
                    has_locator: true,
                    is_verified: false,
                    has_identifier: true,
                },
            ],
            ..multi_version_entry(true, true, true)
        };
        assert_eq!(
            project_entry(&one_bad).claim_support,
            DimensionStatus::Unverified
        );
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn multi_version_locator_needs_every_fiber_located() {
        assert_eq!(
            project_entry(&multi_version_entry(true, true, true)).locator_accuracy,
            DimensionStatus::Verified
        );
        let one_unlocated = EntryFields {
            versions: alloc::vec![
                VersionFiber {
                    has_locator: true,
                    is_verified: true,
                    has_identifier: true,
                },
                VersionFiber {
                    has_locator: false,
                    is_verified: true,
                    has_identifier: true,
                },
            ],
            ..multi_version_entry(true, true, true)
        };
        assert_eq!(
            project_entry(&one_unlocated).locator_accuracy,
            DimensionStatus::Unverified
        );
    }

    // ── Property-based: the projection respects the sound-gate fold ─
    use proptest::prelude::*;

    fn arb_bool() -> impl Strategy<Value = bool> {
        proptest::bool::ANY
    }

    fn arb_entry() -> impl Strategy<Value = EntryFields> {
        (
            arb_bool(), // authors
            arb_bool(), // title
            arb_bool(), // publisher
            arb_bool(), // year
            arb_bool(), // identifier
            arb_bool(), // section_or_page
            arb_bool(), // content_summary
            arb_bool(), // verified_by
        )
            .prop_map(
                |(authors, title, publisher, year, identifier, sop, summary, verified)| {
                    EntryFields {
                        has_authors: authors,
                        has_title: title,
                        has_publisher: publisher,
                        has_year: year,
                        has_identifier: identifier,
                        has_section_or_page: sop,
                        has_content_summary: summary,
                        has_verified_by: verified,
                        has_claim: false,
                        verification_method: String::new(),
                        versions: Vec::new(),
                    }
                },
            )
    }

    proptest! {
        /// The verdict is Invalid iff a sound-gate dimension (Existence
        /// or ClaimSupport) projects to Unverified — i.e. the projection
        /// composes correctly with the assess() fold (ontology
        /// sound-gate = {Existence, ClaimSupport}).
        #[test]
        fn prop_invalid_iff_sound_gate_unverified(f in arb_entry()) {
            let a = project_entry(&f);
            let blocking_gap = a.existence == DimensionStatus::Unverified
                || a.claim_support == DimensionStatus::Unverified;
            prop_assert_eq!(
                a.verdict() == CitationVerdict::Invalid,
                blocking_gap
            );
        }

        /// FormatConformance is Verified for every projected entry (the
        /// Info-severity dimension; the entry deserialized under schema).
        #[test]
        fn prop_format_always_verified(f in arb_entry()) {
            prop_assert_eq!(
                project_entry(&f).format_conformance,
                DimensionStatus::Verified
            );
        }
    }

    pr4xis::register_praxis_value!(prop_invalid_iff_sound_gate_unverified, Honest, Verifiable);
    pr4xis::register_praxis_value!(prop_format_always_verified, Verifiable);
}

//! 18 U.S.C. § 1514A — Sarbanes–Oxley § 806 whistleblower protection.
//!
//! Two data paths coexist during the migration to whole-title
//! USLM:
//!
//! - [`statute`] returns the legacy hand-curated Statute, built
//!   from the `[structural."sox_1514a@2002"]` block in praxis.lock.
//!   28 terms, 18 relations, practitioner-doctrinal naming (Covered
//!   Employer, Prohibition on Retaliation, …). Consumed by the
//!   existing test suite and downstream code.
//! - [`statute_from_uslm`] returns the USLM-derived Statute,
//!   sourced from the embedded Title 18 corpus at
//!   [`super::us_code::title_18`]. Term naming follows USLM
//!   headings verbatim. Used by future code that wants the
//!   whole-title-default path.
//!
//! The migration sequence: build out test coverage for the USLM
//! path, then incrementally point consumers from `statute()` to
//! `statute_from_uslm()`. Once all consumers move, the hand-curated
//! `[structural.*]` block can be deleted.
//!
//! Source: 18 U.S.C. § 1514A (2002, Sarbanes–Oxley Act § 806).

include!(concat!(env!("OUT_DIR"), "/sox_1514a_codegen.rs"));

pub mod canonical_audit;
pub mod proof_standard;

use std::sync::OnceLock;

use super::{Statute, StatuteConstructError};
use crate::applied::data_provisioning::registry;

/// USLM identifier for 18 U.S.C. § 1514A.
pub const IDENTIFIER: &str = "/us/usc/t18/s1514A";

/// The live `Statute` instance for 18 U.S.C. § 1514A, lazily
/// constructed from `praxis.lock`'s `[structural."sox_1514a@2002"]`
/// block. The instance is built once per process and cached.
pub fn statute() -> &'static Statute {
    static INSTANCE: OnceLock<Statute> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let data = registry::structural_for("sox_1514a", "2002").expect(
            "praxis.lock must contain [structural.\"sox_1514a@2002\"] — \
             see crates/domains/build.rs and praxis.lock root",
        );
        Statute::from_structural("sox_1514a", "2002", data)
            .expect("sox_1514a@2002 structural data must validate against Statute::from_structural")
    })
}

/// Construct-time errors are surfaced as `Result` for callers that
/// want non-panicking access — useful in property-based tests that
/// re-construct the statute and want to assert no error path is
/// taken.
pub fn try_statute() -> Result<Statute, StatuteConstructError> {
    let data = registry::structural_for("sox_1514a", "2002")
        .expect("praxis.lock must contain [structural.\"sox_1514a@2002\"]");
    Statute::from_structural("sox_1514a", "2002", data)
}

/// USLM-derived Statute for 18 U.S.C. § 1514A, looked up via the
/// generic [`super::us_code::section`] dispatch using a typed
/// [`UsCodeTitleId`] instead of a hard-coded module path. Lazily
/// constructed, cached, panics if Title 18 doesn't carry § 1514A
/// (a build-time invariant).
///
/// Term naming follows USLM `<heading>` text verbatim — differs
/// from the practitioner-doctrinal naming in [`statute`].
pub fn statute_from_uslm() -> &'static Statute {
    use crate::social::software::markup::xml::uslm::corpus::UsCodeTitleId;
    static INSTANCE: OnceLock<Statute> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let title_18 =
            UsCodeTitleId::try_from_number(18).expect("Title 18 is a valid USC title number");
        let section = super::us_code::section(&title_18, IDENTIFIER).unwrap_or_else(|| {
            panic!(
                "{} USLM is missing section {IDENTIFIER}",
                title_18.short_citation()
            )
        });
        section.to_statute("sox_1514a", "2002")
    })
}

#[cfg(test)]
mod tests;

//! 18 U.S.C. § 1514A — Sarbanes–Oxley § 806 whistleblower protection.
//!
//! Loaded from the unified `UsCode::loaded()` corpus by typed USLM URN
//! (`/us/usc/t18/s1514A`). Title 18 USLM XML must be on disk under
//! `crates/domains/data/legal/uscode/usc_title_18/` for the lookup to
//! succeed — `pr4xis update usc_title_18` materializes it.
//!
//! Citation: 18 U.S.C. § 1514A (2002, Sarbanes–Oxley Act § 806);
//! 1 U.S.C. § 204 (Code authority); LRC, *USLM XML User Guide* §V
//! (USC URN hierarchy).
//!
//! The contributing-factor proof-standard ontology that § 1514A
//! incorporates by reference through § 1514A(b)(2)(C) lives at
//! [`crate::social::compliance::proof_standards::air21_framework`] —
//! the same AIR21 § 42121(b)(2)(B) framework that FRSA, CFPA, NDAA,
//! and other federal whistleblower statutes also import.

use std::sync::OnceLock;

use super::Statute;

/// USLM identifier for 18 U.S.C. § 1514A.
pub const IDENTIFIER: &str = "/us/usc/t18/s1514A";

/// The live `Statute` instance for 18 U.S.C. § 1514A, looked up by
/// typed USLM URN against the unified
/// [`UsCode`](crate::social::software::markup::xml::uslm::corpus::UsCode)
/// corpus. Lazily constructed, cached, panics if § 1514A is not in the
/// loaded corpus (a build-time invariant when Title 18 USLM is registered).
///
/// Term naming follows USLM `<heading>` text verbatim.
pub fn statute() -> &'static Statute {
    use crate::formal::meta::identifier_format::Identifier;
    use crate::social::software::markup::xml::uslm::corpus::loaded as usc_loaded;
    static INSTANCE: OnceLock<Statute> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let urn = Identifier::uslm_urn(IDENTIFIER)
            .expect("SOX 1514A IDENTIFIER must be a valid USLM URN");
        let section = usc_loaded()
            .section_by_urn(&urn)
            .unwrap_or_else(|| panic!("section {IDENTIFIER} not in loaded UsCode corpus"));
        section.to_statute("sox_1514a", "2002")
    })
}

#[cfg(test)]
mod tests;

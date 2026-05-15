//! 18 U.S.C. § 1514A — Sarbanes–Oxley § 806 whistleblower protection.
//!
//! Civil action to protect against retaliation in fraud cases. The
//! ontology is auto-generated at build time from the
//! `[structural."sox_1514a@2002"]` block in workspace-root
//! `praxis.lock`. 28 statutory terms (Covered Employer, Protected
//! Activity, Causation, Remedies, …) and 18 relations across 7
//! relation kinds (Requires, Composes, AlternativeTo, Implies,
//! SafeHarborFor, ExhaustionRequiredFor, Precedes) — see the lock
//! block for the full list.
//!
//! Source: 18 U.S.C. § 1514A (2002, Sarbanes–Oxley Act § 806). For
//! the verbatim statutory text consult `praxis.toml`'s URL field;
//! this module exposes the structural ontology only, not the
//! verbatim text.

include!(concat!(env!("OUT_DIR"), "/sox_1514a_codegen.rs"));

pub mod proof_standard;

use std::sync::OnceLock;

use super::{Statute, StatuteConstructError};
use crate::applied::data_provisioning::registry;

/// The live `Statute` instance for 18 U.S.C. § 1514A, lazily
/// constructed from `praxis.lock`'s `[structural."sox_1514a@2002"]`
/// block. The instance is built once per process and cached.
///
/// Panics if `praxis.lock` is missing the structural block or the
/// block fails [`Statute::from_structural`] validation — both
/// represent build-time drift that the
/// `LockManifestAgreement` / `DecoderTotalityPerKind` axioms should
/// have already caught at workspace-test time.
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

#[cfg(test)]
mod tests;

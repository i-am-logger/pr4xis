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

#[cfg(test)]
mod tests;

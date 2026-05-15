//! 49 U.S.C. § 42121 — AIR21 whistleblower protection and procedural
//! framework.
//!
//! The Wendell H. Ford Aviation Investment and Reform Act for the
//! 21st Century (AIR21; Pub. L. 106-181, Apr. 5, 2000) added § 42121
//! to Title 49, prohibiting retaliation against air-carrier employees
//! who provide air-safety information. The procedural framework in
//! § 42121(b) — particularly the four-clause burden-shifting
//! structure in § 42121(b)(2)(B)(i)-(iv) — is *incorporated by
//! reference* into the SOX whistleblower civil action via
//! 18 U.S.C. § 1514A(b)(2)(C). The contributing-factor causation
//! standard, the clear-and-convincing same-action defense, and the
//! investigation-gate / merits-adjudication two-stage structure all
//! originate here.
//!
//! Dodd-Frank Wall Street Reform and Consumer Protection Act, § 922
//! (2010), amended § 42121's procedural posture (notably extending
//! the de novo court right's trigger window). The current version is
//! the 2010 post-Dodd-Frank text; the manifest pins version `"2010"`.
//!
//! # Structure mirrors sox_1514a
//!
//! The ontology is auto-generated at build time from the
//! `[structural."air21_42121@2010"]` block in workspace-root
//! `praxis.lock`. 17 statutory terms (Discrimination Prohibited,
//! four Protected-Activity variants, the four Required-Showing
//! clauses, the procedural sequence b1 → b2 → b3 → b4/b5) and 21
//! relations.
//!
//! The codegen produces `Air2142121Id` (the concept enum) plus
//! `CODEGEN_DATA`. The runtime `Statute` instance is exposed through
//! [`statute()`] — same `OnceLock` pattern as
//! `social::compliance::statutes::sox_1514a::statute`.

include!(concat!(env!("OUT_DIR"), "/air21_42121_codegen.rs"));

use std::sync::OnceLock;

use super::{Statute, StatuteConstructError};
use crate::applied::data_provisioning::registry;

/// The live `Statute` instance for 49 U.S.C. § 42121, lazily
/// constructed from `praxis.lock`'s `[structural."air21_42121@2010"]`
/// block. Cached once per process.
///
/// Panics if `praxis.lock` is missing the structural block or the
/// block fails [`Statute::from_structural`] validation.
pub fn statute() -> &'static Statute {
    static INSTANCE: OnceLock<Statute> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let data = registry::structural_for("air21_42121", "2010").expect(
            "praxis.lock must contain [structural.\"air21_42121@2010\"] — \
             see crates/domains/build.rs and praxis.lock root",
        );
        Statute::from_structural("air21_42121", "2010", data).expect(
            "air21_42121@2010 structural data must validate against Statute::from_structural",
        )
    })
}

/// Fallible accessor for tests that want to assert no error path is
/// reachable from the current praxis.lock data.
pub fn try_statute() -> Result<Statute, StatuteConstructError> {
    let data = registry::structural_for("air21_42121", "2010")
        .expect("praxis.lock must contain [structural.\"air21_42121@2010\"]");
    Statute::from_structural("air21_42121", "2010", data)
}

#[cfg(test)]
mod tests;

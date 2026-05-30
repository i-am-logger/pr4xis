//! 49 U.S.C. § 42121 — AIR21 protection of employees providing air
//! safety information; the burden-shifting framework SOX § 1514A
//! imports by reference.
//!
//! Loaded from the unified `UsCode::loaded()` corpus by typed USLM URN
//! (`/us/usc/t49/s42121`). Title 49 USLM XML must be on disk.
//!
//! Citation: 49 U.S.C. § 42121 (AIR21 § 519, 2000; substantive amend.
//! Dodd-Frank 2010); 1 U.S.C. § 204; LRC USLM XML User Guide §V.

use std::sync::OnceLock;

use super::Statute;

pub const IDENTIFIER: &str = "/us/usc/t49/s42121";

pub fn statute() -> &'static Statute {
    use crate::formal::meta::identifier_format::Identifier;
    use crate::social::software::markup::xml::uslm::corpus::loaded as usc_loaded;
    static INSTANCE: OnceLock<Statute> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let urn = Identifier::uslm_urn(IDENTIFIER)
            .expect("AIR21 42121 IDENTIFIER must be a valid USLM URN");
        let section = usc_loaded()
            .section_by_urn(&urn)
            .unwrap_or_else(|| panic!("section {IDENTIFIER} not in loaded UsCode corpus"));
        section.to_statute("air21_42121", "2010")
    })
}

#[cfg(test)]
mod tests;

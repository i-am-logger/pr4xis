//! Realized toponymy mechanics: `Place`/`Country`/`Region` types and the
//! `capital_of`/`contains`/`borders` predicates, populated from the REAL
//! GeoNames `countryInfo.txt` gazetteer (`[sources.geonames_countryinfo]`;
//! see [`super::reader`]/[`super::store`] for the load path) — no literal
//! toponym strings live in this file.
//!
//! `borders` used to be realized by delegating to `formal::spatial::rcc8`'s
//! 1-D interval classifier over synthetic footprints (a toy fixture's own
//! device). GeoNames' `countryInfo.txt` carries no polygon/interval geometry
//! at all — only the ISO/Country/Capital/Continent/neighbours columns — so a
//! geometric footprint for 252 real countries would have to be INVENTED,
//! which the fixture never actually needed and this real corpus must not
//! do. `contains`/`borders` are therefore grounded directly in the loaded
//! relational facts (continent membership; the neighbours adjacency list),
//! and `super::ontology`'s 3 axioms verify the STRUCTURAL properties Randell,
//! Cui & Cohn (1992) §3 assign to the RCC-8 relations EC/TPP/NTPP directly
//! (their own algebraic definitions — symmetry, irreflexivity, functional
//! part-of — not a re-derivation through the interval model), rather than
//! reproducing `rcc8::interval::classify` over fabricated 1-D coordinates.

#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// A named toponymic root: a display name plus its ISO 3166-1 alpha-2 code
/// (a Country) or GeoNames continent code (used as a Region's `code`, see
/// [`Region`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub name: String,
    pub iso: String,
}

/// A country record loaded from one GeoNames `countryInfo.txt` row: its
/// `Place` (name + ISO 3166-1 alpha-2 code), its capital (absent for the
/// handful of uninhabited/non-sovereign territories the source lists —
/// `Option`, never a blank string), its GeoNames continent code, and its
/// SYMMETRIZED bordering-country ISO codes (see [`super::store`]'s
/// `symmetrize` — the raw per-row `neighbours` column is directional and not
/// perfectly mutually consistent for 2 defunct legacy entries, so the store
/// unions both directions before `borders` is ever queried, making `borders`
/// exactly the RCC-8 EC relation's own symmetry property, Randell, Cui &
/// Cohn 1992 §3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Country {
    pub place: Place,
    pub capital: Option<String>,
    pub continent: String,
    pub neighbours: Vec<String>,
}

/// A Region — a GeoNames continent, identified by its 2-letter code (EU, AS,
/// AF, NA, SA, OC, AN — GeoNames' own continent-code vocabulary, carried
/// through unchanged rather than decoded to an English name no column in the
/// source actually supplies).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Region {
    pub code: String,
}

/// The capital of `country` — `None` for the source's uninhabited/
/// non-sovereign entries that carry no `Capital` value (Antarctica, Bouvet
/// Island, Heard Island & McDonald Islands, Tokelau, the U.S. Minor Outlying
/// Islands, Bonaire/Saint Eustatius/Saba, as of the 2026-07-21 dump).
#[must_use]
pub fn capital_of(country: &Country) -> Option<&str> {
    country.capital.as_deref()
}

/// Does `region` contain `country` — Randell, Cui & Cohn's (1992 §3) proper-
/// part relation (TPP/NTPP), realized here as continent membership: `region`
/// is exactly the continent GeoNames' own `Continent` column assigns
/// `country`. Every country has exactly one containing Region under this
/// realization (a country's `continent` field is a single code, never a
/// list), the functional/antisymmetric shape §3 requires of a proper-part
/// relation — a Region is never, in turn, a proper part of one of the
/// countries it contains.
#[must_use]
pub fn contains(region: &Region, country: &Country) -> bool {
    country.continent == region.code
}

/// Do `a` and `b` border each other — Randell, Cui & Cohn's (1992 §3)
/// Externally-Connected (EC) relation, realized here as GeoNames'
/// `neighbours` adjacency, SYMMETRIZED by [`super::store`] before this
/// function ever sees it (so `borders(a, b) == borders(b, a)` holds by
/// construction, matching §3's own classification of EC as its own converse
/// — the same property `formal::spatial::rcc8::ontology`'s
/// `ProperPartInversesAreSymmetricPairs` axiom documents for TPP/NTPP's
/// distinct converses, TPPi/NTPPi).
#[must_use]
pub fn borders(a: &Country, b: &Country) -> bool {
    a.neighbours.iter().any(|iso| iso == &b.place.iso)
}

/// True iff `country`'s ISO code names itself — `borders` must never hold
/// for this case (irreflexive, Randell, Cui & Cohn 1992 §3: EC excludes the
/// degenerate case a region touches itself).
#[must_use]
pub fn is_self(a: &Country, b: &Country) -> bool {
    a.place.iso == b.place.iso
}

#[cfg(test)]
pub(super) fn fixture_country(
    iso: &str,
    name: &str,
    capital: Option<&str>,
    continent: &str,
    neighbours: &[&str],
) -> Country {
    Country {
        place: Place {
            name: name.to_string(),
            iso: iso.to_string(),
        },
        capital: capital.map(str::to_string),
        continent: continent.to_string(),
        neighbours: neighbours.iter().map(|s| s.to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn france() -> Country {
        fixture_country("FR", "France", Some("Paris"), "EU", &["DE", "ES"])
    }

    fn germany() -> Country {
        fixture_country("DE", "Germany", Some("Berlin"), "EU", &["FR", "PL"])
    }

    fn poland() -> Country {
        fixture_country("PL", "Poland", Some("Warsaw"), "EU", &["DE"])
    }

    fn atlantis_no_capital() -> Country {
        fixture_country("AQ", "Antarctica", None, "AN", &[])
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn capital_of_returns_the_countrys_capital() {
        assert_eq!(capital_of(&france()), Some("Paris"));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn capital_of_is_none_when_the_source_row_has_no_capital() {
        assert_eq!(capital_of(&atlantis_no_capital()), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn contains_holds_for_a_country_in_the_region() {
        let eu = Region {
            code: "EU".to_string(),
        };
        assert!(contains(&eu, &france()));
        assert!(contains(&eu, &germany()));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn contains_does_not_hold_for_a_country_in_a_different_continent() {
        let na = Region {
            code: "NA".to_string(),
        };
        assert!(!contains(&na, &france()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn adjacent_countries_border_each_other() {
        assert!(borders(&france(), &germany()));
        assert!(borders(&germany(), &france()));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn non_adjacent_countries_do_not_border() {
        assert!(!borders(&france(), &poland()));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_country_never_borders_itself() {
        let fr = france();
        assert!(is_self(&fr, &fr));
        assert!(!borders(&fr, &fr));
    }
}

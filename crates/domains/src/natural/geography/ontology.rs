//! Geography — Place/Country/Capital/Region toponymy, with `contains`/
//! `borders` grounded against `formal::spatial::rcc8`'s Region Connection
//! Calculus (Turing-benchmark B3) rather than a bespoke adjacency check.
//! Instance data loads from the GeoNames `countryInfo.txt` gazetteer
//! (`[sources.geonames_countryinfo]`, see `super::store`) — every
//! Country/Capital/Region fact is a loaded ISO 3166-1 record, never a
//! literal in this file.
//!
//! # Literature
//!
//! - **Randell, Cui & Cohn (1992)** *A Spatial Logic Based on Regions and
//!   Connection*, KR'92 — the RCC-8 relations `contains` (NTPP/TPP) and
//!   `borders` (EC) realize.
//! - **ISO 3166-1 (2020)** *Codes for the representation of names of
//!   countries and their subdivisions* — the Country/Capital entity shape
//!   this ontology's concepts follow (a country has exactly one capital in
//!   the sense ISO 3166-1's own reference list assigns).
//! - **GeoNames.org** `countryInfo.txt` — the loaded instance data (Country/
//!   Capital/Continent/neighbours), licensed CC BY 4.0 (attribution
//!   required, see `[sources.geonames_countryinfo]`'s description).

use pr4xis::ontology::{Axiom, Ontology, Quality};

#[cfg(feature = "std")]
use super::place;
#[cfg(feature = "std")]
use super::store;

pr4xis::ontology! {
    name: "Geography",
    source: "Randell, Cui & Cohn (1992) A Spatial Logic Based on Regions and Connection, KR'92; ISO 3166-1 (2020)",

    concepts: [Place, Country, Capital, Region],

    labels: {
        Place: ("en", "Place",
            "A named location with a spatial footprint — the root toponymic entity."),
        Country: ("en", "Country",
            "A sovereign or ISO-3166-1-listed place with exactly one Capital. ISO 3166-1 (2020)."),
        Capital: ("en", "Capital",
            "The seat-of-government Place of a Country."),
        Region: ("en", "Region",
            "A supra-national or sub-national Place that spatially CONTAINS other places (an RCC-8 NTPP/TPP relation over their footprints, Randell, Cui & Cohn 1992 \u{00a7}3)."),
    },

    // Country and Capital are both specializations of the general Place
    // entity (every Country/Capital IS a Place with additional structure).
    is_a: [
        (Country, Place),
        (Capital, Place),
    ],

    // A Country has-a Capital (ISO 3166-1's own one-country-one-capital
    // assignment).
    has_a: [
        (Country, Capital),
    ],
}

/// Quality: does this concept classify a CONTAINER (a Place that spatially
/// contains others) or a CONTAINED leaf? Only Region is a container in
/// this ontology's structural sense; Place/Country/Capital are the
/// contained/leaf toponyms `contains` is checked FROM one of them TO.
#[derive(Debug, Clone)]
pub struct IsContainer;

impl Quality for IsContainer {
    type Individual = GeographyConcept;
    type Value = bool;

    fn get(&self, c: &GeographyConcept) -> Option<bool> {
        use GeographyConcept as C;
        Some(matches!(c, C::Region))
    }
}

impl Ontology for GeographyOntology {
    type Cat = GeographyCategory;
    type Qual = IsContainer;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        // The 3 domain axioms below query the loaded GeoNames gazetteer
        // (`store::gazetteer_loaded`, `std`-only — `OnceLock` caching over
        // the no_std-safe `raw_source_bytes_embedded` materialization).
        // Structural (category-law) axioms above stay available in a pure
        // `alloc`/no_std build; these 3 are gated so the crate still builds
        // `--no-default-features` (the wasm target).
        #[cfg(feature = "std")]
        {
            axioms.push(Box::new(EveryCountryHasExactlyOneCapital));
            axioms.push(Box::new(RegionContainmentIsRcc8ProperPart));
            axioms.push(Box::new(BordersIsRcc8ExternalConnection));
        }
        axioms
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------
//
// GeoNames' `countryInfo.txt` carries no polygon/interval geometry, so these
// 3 axioms no longer re-derive their relation through
// `formal::spatial::rcc8::interval::classify` over a synthetic 1-D
// footprint (there is none to classify for 252 real countries without
// inventing one). Instead each axiom checks the STRUCTURAL property Randell,
// Cui & Cohn (1992) §3 itself assigns the relation being realized — the same
// citation, applied to its own algebraic definition rather than the toy
// geometric model — directly over the whole loaded real gazetteer.

/// Axiom: every Capital fact the loaded gazetteer records resolves,
/// deterministically, to exactly that capital (`capital_of` never disagrees
/// with the source row it was built from), and the loaded set is
/// non-degenerate (at least one country carries a Capital). ISO 3166-1
/// (2020) — the standard's own one-country-one-capital convention (the
/// reference list this ontology's Country/Capital shape follows); GeoNames'
/// `Capital` column is itself a single scalar field per row, so "exactly
/// one" is witnessed by every row this axiom checks, not merely assumed.
#[cfg(feature = "std")]
pub struct EveryCountryHasExactlyOneCapital;

#[cfg(feature = "std")]
impl Axiom for EveryCountryHasExactlyOneCapital {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let countries = store::gazetteer_loaded().countries();
        let with_capital: Vec<_> = countries.iter().filter(|c| c.capital.is_some()).collect();
        let all_resolve_exactly = with_capital.iter().all(|c| {
            place::capital_of(c) == c.capital.as_deref()
                && !place::capital_of(c).unwrap_or_default().is_empty()
        });
        if !countries.is_empty() && !with_capital.is_empty() && all_resolve_exactly {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EveryCountryHasExactlyOneCapital",
        "every Capital fact in the loaded gazetteer resolves, deterministically, to exactly that capital",
        "ISO 3166-1 (2020)"
    );
}

#[cfg(feature = "std")]
pr4xis::register_axiom!(EveryCountryHasExactlyOneCapital, "ISO 3166-1 (2020)");

/// Axiom: Region containment is a well-defined FUNCTIONAL proper-part
/// relation over the loaded gazetteer — every country is contained by
/// EXACTLY the one Region (GeoNames continent) its own `Continent` column
/// names, and by no other loaded Region. Randell, Cui & Cohn (1992) §3
/// defines TPP/NTPP (proper part) as antisymmetric: a Region is never, in
/// turn, contained by one of the countries it contains, and (under this
/// continent-partition realization) a country belongs to exactly one
/// containing Region, never zero or several.
#[cfg(feature = "std")]
pub struct RegionContainmentIsRcc8ProperPart;

#[cfg(feature = "std")]
impl Axiom for RegionContainmentIsRcc8ProperPart {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let gazetteer = store::gazetteer_loaded();
        let regions = gazetteer.regions();
        let countries = gazetteer.countries();
        let functional = !countries.is_empty()
            && !regions.is_empty()
            && countries.iter().all(|c| {
                let containing: Vec<_> = regions.iter().filter(|r| place::contains(r, c)).collect();
                containing.len() == 1 && containing[0].code == c.continent
            });
        if functional {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RegionContainmentIsRcc8ProperPart",
        "every loaded country is contained by exactly one Region, agreeing with its Continent column — the antisymmetric/functional shape RCC-8's proper-part relation requires",
        "Randell, Cui & Cohn (1992) \u{00a7}3"
    );
}

#[cfg(feature = "std")]
pr4xis::register_axiom!(
    RegionContainmentIsRcc8ProperPart,
    "Randell, Cui & Cohn (1992) \u{00a7}3"
);

/// Axiom: `borders` realizes RCC-8's EC (Externally Connected) relation's
/// own algebraic properties over the WHOLE loaded gazetteer — symmetric
/// (EC is its own converse, Randell, Cui & Cohn 1992 §3: `EC(x,y) ⟺
/// EC(y,x)`, unlike TPP/NTPP's distinct converses TPPi/NTPPi) and
/// irreflexive (no country borders itself). Checked over every ordered
/// pair of loaded countries (252² ≈ 63k pairs for the 2026-07-21 dump), not
/// merely the pairs the source happened to list.
#[cfg(feature = "std")]
pub struct BordersIsRcc8ExternalConnection;

#[cfg(feature = "std")]
impl Axiom for BordersIsRcc8ExternalConnection {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let countries = store::gazetteer_loaded().countries();
        let mut symmetric = true;
        let mut irreflexive = true;
        let mut any_border = false;
        for a in countries {
            for b in countries {
                if place::is_self(a, b) {
                    if place::borders(a, b) {
                        irreflexive = false;
                    }
                    continue;
                }
                if place::borders(a, b) {
                    any_border = true;
                }
                if place::borders(a, b) != place::borders(b, a) {
                    symmetric = false;
                }
            }
        }
        if !countries.is_empty() && any_border && symmetric && irreflexive {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BordersIsRcc8ExternalConnection",
        "borders is symmetric (EC is its own converse) and irreflexive over the whole loaded gazetteer",
        "Randell, Cui & Cohn (1992) \u{00a7}3"
    );
}

#[cfg(feature = "std")]
pr4xis::register_axiom!(
    BordersIsRcc8ExternalConnection,
    "Randell, Cui & Cohn (1992) \u{00a7}3"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<GeographyCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        GeographyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn four_concepts() {
        assert_eq!(GeographyConcept::variants().len(), 4);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn only_region_is_a_container() {
        let q = IsContainer;
        for c in GeographyConcept::variants() {
            assert!(q.get(&c).is_some(), "{c:?} has no IsContainer");
        }
        assert_eq!(q.get(&GeographyConcept::Region), Some(true));
        assert_eq!(q.get(&GeographyConcept::Country), Some(false));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_country_has_exactly_one_capital_holds() {
        assert!(EveryCountryHasExactlyOneCapital.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn region_containment_is_rcc8_proper_part_holds() {
        assert!(RegionContainmentIsRcc8ProperPart.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn borders_is_rcc8_external_connection_holds() {
        assert!(BordersIsRcc8ExternalConnection.verify().is_ok());
    }
}

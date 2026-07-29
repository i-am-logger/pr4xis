//! The loaded, indexed GeoNames gazetteer — the process-wide store
//! [`super::place`]'s `capital_of`/`contains`/`borders` predicates and
//! [`super::ontology`]'s 3 axioms query.
//!
//! Mirrors [`crate::cognitive::linguistics::conceptnet::store`]'s shape: a
//! `OnceLock`-cached, indexed view over the reader's typed output, built
//! once, queried many times.
//!
//! ## Symmetrizing `neighbours`
//!
//! GeoNames' raw per-row `neighbours` column is DIRECTIONAL (country X's row
//! lists X's neighbours) and not perfectly mutually consistent: verified
//! 2026-07-21 that 646 of 654 directed neighbour pairs in the 2026-07-21 dump
//! agree in both directions; the 8 exceptions are all incident to the two
//! defunct legacy entries `CS` (Serbia and Montenegro, dissolved 2006) and
//! `AN` (Netherlands Antilles, dissolved 2010) — their successor states'
//! rows no longer list them back. Since Randell, Cui & Cohn (1992 §3)
//! classify EC as its own converse (a genuinely symmetric relation, not a
//! directional one like TPP/TPPi), [`symmetrize`] unions both directions
//! before [`super::place::borders`] is ever queried, so `borders(a, b) ==
//! borders(b, a)` holds by CONSTRUCTION over the whole loaded gazetteer —
//! never left to depend on which of the two rows happened to list the pair.

#[allow(unused_imports)]
use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec::Vec,
};

use super::place::{Country, Region};

/// Union every directed `(iso, neighbour)` pair with its reverse, then
/// rewrite each [`Country`]'s `neighbours` to the resulting symmetric
/// adjacency set (sorted, deduplicated — `BTreeSet` iteration order). See
/// the module doc for why this is the RCC-8-faithful realization of EC, not
/// an invented fact: every edge in the output was asserted by SOME row in
/// the source, just possibly only the other endpoint's row.
#[must_use]
pub fn symmetrize(mut countries: Vec<Country>) -> Vec<Country> {
    let mut adjacency: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for country in &countries {
        for neighbour in &country.neighbours {
            adjacency
                .entry(country.place.iso.clone())
                .or_default()
                .insert(neighbour.clone());
            adjacency
                .entry(neighbour.clone())
                .or_default()
                .insert(country.place.iso.clone());
        }
    }
    for country in &mut countries {
        if let Some(set) = adjacency.get(&country.place.iso) {
            country.neighbours = set.iter().cloned().collect();
        }
    }
    countries
}

/// The loaded, indexed gazetteer.
#[derive(Debug, Clone, Default)]
pub struct GazetteerStore {
    countries: Vec<Country>,
}

impl GazetteerStore {
    /// Build the store from the reader's typed output, symmetrizing
    /// `neighbours` first (see [`symmetrize`]).
    #[must_use]
    pub fn from_countries(countries: Vec<Country>) -> Self {
        Self {
            countries: symmetrize(countries),
        }
    }

    /// Every loaded country, in source order.
    #[must_use]
    pub fn countries(&self) -> &[Country] {
        &self.countries
    }

    /// Look up a loaded country by its ISO 3166-1 alpha-2 code.
    #[must_use]
    pub fn by_iso(&self, iso: &str) -> Option<&Country> {
        self.countries.iter().find(|c| c.place.iso == iso)
    }

    /// Every distinct Region (GeoNames continent code) at least one loaded
    /// country belongs to, sorted by code.
    #[must_use]
    pub fn regions(&self) -> Vec<Region> {
        let codes: BTreeSet<String> = self.countries.iter().map(|c| c.continent.clone()).collect();
        codes.into_iter().map(|code| Region { code }).collect()
    }
}

/// The process-wide loaded gazetteer — the committed `geonames_countryinfo@
/// 2026-07-21` `.prx` decoded, parsed, symmetrized, and indexed once.
/// Mirrors [`crate::cognitive::linguistics::conceptnet::store::conceptnet_loaded`]'s
/// caching shape.
#[cfg(feature = "std")]
#[must_use]
pub fn gazetteer_loaded() -> &'static GazetteerStore {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<GazetteerStore> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        use crate::applied::data_provisioning::decoders::plaintext_tsv;
        use crate::applied::data_provisioning::raw_source_prx::raw_source_bytes_embedded;

        const GAZETTEER_PRX: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/geography/geonames-countryinfo-2026-07-21.prx"
        ));

        let bytes = raw_source_bytes_embedded("geonames_countryinfo", "2026-07-21", GAZETTEER_PRX);
        let records = plaintext_tsv::decode(&bytes).unwrap_or_else(|e| {
            panic!("geonames_countryinfo committed .prx archive failed to decode: {e}")
        });
        let countries = super::reader::read_countries(&records);
        GazetteerStore::from_countries(countries)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::natural::geography::place::fixture_country;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn symmetrize_adds_the_missing_reverse_edge() {
        // Hungary's row lists the (now-defunct) `CS` neighbour; `CS`'s own
        // row is the one carrying the forward edge. Before symmetrizing,
        // only `CS -> HU` exists; after, `HU -> CS` must too.
        let cs = fixture_country("CS", "Serbia and Montenegro", None, "EU", &["HU"]);
        let hu = fixture_country("HU", "Hungary", Some("Budapest"), "EU", &[]);
        let symmetrized = symmetrize(alloc::vec![cs, hu]);
        let hu = symmetrized.iter().find(|c| c.place.iso == "HU").unwrap();
        assert!(hu.neighbours.contains(&"CS".to_string()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn store_indexes_by_iso_and_derives_regions() {
        let fr = fixture_country("FR", "France", Some("Paris"), "EU", &["DE"]);
        let de = fixture_country("DE", "Germany", Some("Berlin"), "EU", &["FR"]);
        let jp = fixture_country("JP", "Japan", Some("Tokyo"), "AS", &[]);
        let store = GazetteerStore::from_countries(alloc::vec![fr, de, jp]);
        assert_eq!(store.countries().len(), 3);
        assert_eq!(store.by_iso("FR").unwrap().place.name, "France");
        assert!(store.by_iso("ZZ").is_none());
        let region_codes: Vec<String> = store.regions().into_iter().map(|r| r.code).collect();
        assert_eq!(
            region_codes,
            alloc::vec!["AS".to_string(), "EU".to_string()]
        );
    }
}

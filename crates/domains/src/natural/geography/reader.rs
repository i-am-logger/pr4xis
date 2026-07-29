//! Interpret the generic TSV record stream
//! ([`crate::applied::data_provisioning::decoders::plaintext_tsv`]'s decode
//! target) as the GeoNames `countryInfo.txt` field shape:
//! `ISO<TAB>ISO3<TAB>ISO-Numeric<TAB>fips<TAB>Country<TAB>Capital<TAB>
//! Area(in sq km)<TAB>Population<TAB>Continent<TAB>tld<TAB>CurrencyCode<TAB>
//! CurrencyName<TAB>Phone<TAB>PostalCodeFormat<TAB>PostalCodeRegex<TAB>
//! Languages<TAB>geonameid<TAB>neighbours<TAB>EquivalentFipsCode` (19
//! columns; the decoder already dropped the `#`-prefixed header comments).
//!
//! Mirrors [`crate::cognitive::linguistics::conceptnet::reader`]'s division
//! of labor: the generic decoder turns raw bytes into a structure-preserving
//! record stream; this module says what the FIELDS mean. Fail-closed
//! per-row, not per-file — a malformed row (wrong column count, or missing
//! the two columns every row must carry) is skipped, never blanks out the
//! rest.

#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use super::place::{Country, Place};
use crate::applied::data_provisioning::decoders::plaintext_tsv::TsvRecords;

/// GeoNames' own column count (verified against the 2026-07-21 dump: 252
/// data rows, each exactly 19 tab-separated fields, including trailing empty
/// fields for an absent `Capital`/`neighbours`).
const COLUMN_COUNT: usize = 19;
const COL_ISO: usize = 0;
const COL_COUNTRY: usize = 4;
const COL_CAPITAL: usize = 5;
const COL_CONTINENT: usize = 8;
const COL_NEIGHBOURS: usize = 17;

/// Interpret a decoded TSV record stream as [`Country`] rows (NOT yet
/// symmetrized — see [`super::store::symmetrize`], which every loader calls
/// before a `Country`'s `neighbours` is queried through
/// [`super::place::borders`]). A record that doesn't have exactly
/// `COLUMN_COUNT` fields, or whose ISO/Country/Continent columns are
/// blank, is skipped rather than causing the whole load to fail — the same
/// discipline `conceptnet::reader::read_conceptnet` applies to its own TSV.
#[must_use]
pub fn read_countries(records: &TsvRecords) -> Vec<Country> {
    let mut countries = Vec::new();
    for record in records {
        if record.len() != COLUMN_COUNT {
            continue;
        }
        let iso = record[COL_ISO].trim();
        let name = record[COL_COUNTRY].trim();
        let continent = record[COL_CONTINENT].trim();
        if iso.is_empty() || name.is_empty() || continent.is_empty() {
            continue;
        }
        let capital = record[COL_CAPITAL].trim();
        let capital = if capital.is_empty() {
            None
        } else {
            Some(capital.to_string())
        };
        let neighbours: Vec<String> = record[COL_NEIGHBOURS]
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        countries.push(Country {
            place: Place {
                name: name.to_string(),
                iso: iso.to_string(),
            },
            capital,
            continent: continent.to_string(),
            neighbours,
        });
    }
    countries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(fields: &[&str]) -> Vec<String> {
        fields.iter().map(|s| (*s).to_string()).collect()
    }

    fn geonames_row(
        iso: &str,
        country: &str,
        capital: &str,
        continent: &str,
        neighbours: &str,
    ) -> Vec<String> {
        // Exactly 19 columns, in GeoNames' own order; only the 5 columns
        // this reader consumes carry non-placeholder values.
        row(&[
            iso, "X3X", "0", "XX", country, capital, "0", "0", continent, ".xx", "XXX", "X", "0",
            "", "", "", "0", neighbours, "",
        ])
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reads_a_well_formed_row_with_capital_and_neighbours() {
        let records: TsvRecords = alloc::vec![geonames_row("FR", "France", "Paris", "EU", "DE,ES")];
        let countries = read_countries(&records);
        assert_eq!(countries.len(), 1);
        assert_eq!(countries[0].place.iso, "FR");
        assert_eq!(countries[0].place.name, "France");
        assert_eq!(countries[0].capital.as_deref(), Some("Paris"));
        assert_eq!(countries[0].continent, "EU");
        assert_eq!(
            countries[0].neighbours,
            alloc::vec!["DE".to_string(), "ES".to_string()]
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn empty_capital_and_neighbours_become_none_and_empty_not_blank_strings() {
        let records: TsvRecords = alloc::vec![geonames_row("AQ", "Antarctica", "", "AN", "")];
        let countries = read_countries(&records);
        assert_eq!(countries.len(), 1);
        assert_eq!(countries[0].capital, None);
        assert!(countries[0].neighbours.is_empty());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn skips_rows_with_the_wrong_column_count() {
        let records: TsvRecords = alloc::vec![row(&["FR", "France"])];
        assert!(read_countries(&records).is_empty());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn skips_rows_missing_iso_country_or_continent() {
        let records: TsvRecords = alloc::vec![geonames_row("", "France", "Paris", "EU", "")];
        assert!(read_countries(&records).is_empty());
    }
}

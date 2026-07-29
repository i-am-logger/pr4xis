//! Generated-from-loaded-data test families over the REAL GeoNames
//! `countryInfo.txt` gazetteer (`store::gazetteer_loaded`) — one assertion
//! per loaded row/edge, not a handful of hand-picked example countries.
//! Mirrors the "generate one test per loaded data row" discipline
//! `praxis-corpus-tests/build.rs` uses for the caregiver/adversarial
//! question corpus: this dataset (252 rows) is small and in-crate enough
//! that a plain `#[test]` iterating the loaded store, asserting per row,
//! serves the same purpose without needing a separate codegen build script.

use super::place;
use super::store::gazetteer_loaded;

/// Capital-of graph walk: for EVERY loaded country row with a non-empty
/// `Capital`, the `capital_of` edge resolves to exactly that string. One
/// assertion per real `(country, capital)` edge in the 2026-07-21 dump.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn capital_of_graph_walk_over_every_loaded_country_with_a_capital() {
    let countries = gazetteer_loaded().countries();
    let mut checked = 0usize;
    for country in countries {
        if let Some(expected) = country.capital.as_deref() {
            assert_eq!(
                place::capital_of(country),
                Some(expected),
                "capital_of({}) should resolve to the loaded Capital column {expected:?}",
                country.place.name
            );
            checked += 1;
        }
    }
    assert!(
        checked > 200,
        "expected the large majority of the ~252-row 2026-07-21 dump to carry a Capital \
         (got {checked}) — the loader may be dropping rows"
    );
}

/// RCC containment walk: for EVERY loaded country, its loaded
/// continent-proper-part-of edge resolves to the Region matching its
/// `Continent` column, and to no other loaded Region.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn rcc_containment_walk_over_every_loaded_country() {
    let gazetteer = gazetteer_loaded();
    let regions = gazetteer.regions();
    assert!(
        !regions.is_empty(),
        "expected at least one loaded Region (GeoNames continent code)"
    );
    for country in gazetteer.countries() {
        let matching = regions
            .iter()
            .find(|r| r.code == country.continent)
            .unwrap_or_else(|| {
                panic!(
                    "{}'s Continent code {:?} has no matching loaded Region",
                    country.place.name, country.continent
                )
            });
        assert!(
            place::contains(matching, country),
            "Region {:?} should contain {} (its own Continent column)",
            matching.code,
            country.place.name
        );
        for other in &regions {
            if other.code != matching.code {
                assert!(
                    !place::contains(other, country),
                    "{} should not ALSO be contained by unrelated Region {:?}",
                    country.place.name,
                    other.code
                );
            }
        }
    }
}

/// RCC EC/borders walk: for EVERY loaded country with a non-empty
/// (symmetrized) neighbours list, every listed neighbour ISO code resolves
/// to a loaded country carrying an RCC-8 EC (`borders`) edge back.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn rcc_ec_borders_walk_over_every_loaded_neighbour_edge() {
    let gazetteer = gazetteer_loaded();
    let mut edges_checked = 0usize;
    for country in gazetteer.countries() {
        for neighbour_iso in &country.neighbours {
            let neighbour = gazetteer.by_iso(neighbour_iso).unwrap_or_else(|| {
                panic!(
                    "{}'s neighbour ISO code {neighbour_iso:?} does not resolve to a loaded \
                     country",
                    country.place.name
                )
            });
            assert!(
                place::borders(country, neighbour),
                "{} should border its loaded neighbour {}",
                country.place.name,
                neighbour.place.name
            );
            assert!(
                place::borders(neighbour, country),
                "borders must be symmetric: {} borders {} but not vice versa",
                country.place.name,
                neighbour.place.name
            );
            edges_checked += 1;
        }
    }
    assert!(
        edges_checked > 600,
        "expected several hundred symmetrized border edges over the 2026-07-21 dump \
         (got {edges_checked}) — the loader or symmetrization may be dropping edges"
    );
}

/// Strip doc-comment/comment lines (`///`, `//!`, `//`) and everything from
/// the first `#[cfg(test)]` marker onward from `text`, leaving only
/// executable PRODUCTION code — the scan surface
/// [`no_hardcoded_toponym_strings_in_production_loader_code`] checks. Doc
/// comments legitimately name real example places for illustration (e.g.
/// `place.rs`'s own `capital_of` doc lists the source's no-`Capital`
/// territories by name); `#[cfg(test)]` code legitimately uses small literal
/// fixtures. Neither is "hardcoded data" in the sense this lint guards
/// against — a literal string VALUE embedded in the executable path that
/// should have loaded from the corpus instead.
fn strip_comments_and_test_code(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#[cfg(test)]") {
            break;
        }
        if trimmed.starts_with("///") || trimmed.starts_with("//!") || trimmed.starts_with("//") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Regression lint: no literal toponym string from the REAL loaded corpus
/// (a country name or capital, ≥5 chars to avoid short-name false
/// positives) appears anywhere in the PRODUCTION (non-comment,
/// non-`#[cfg(test)]`) source of `place.rs`/`reader.rs`/`store.rs` — the
/// exact hardcoding this task eliminated (`sample_countries`/
/// `sample_region`'s literal `Country` structs) must never creep back in.
/// Data-driven rather than a fixed deny-list: it checks the production code
/// against every name the loaded gazetteer itself carries, so it catches
/// ANY future hardcoded toponym matching real data, not just the ones this
/// fix removed.
#[pr4xis::praxis_value(Honest)]
#[test]
fn no_hardcoded_toponym_strings_in_production_loader_code() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/natural/geography");
    let files = ["place.rs", "reader.rs", "store.rs"];
    let gazetteer = gazetteer_loaded();
    for file in files {
        let path = alloc::format!("{dir}/{file}");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let production = strip_comments_and_test_code(&text);
        for country in gazetteer.countries() {
            if country.place.name.len() >= 5 {
                assert!(
                    !production.contains(country.place.name.as_str()),
                    "{file}'s production code hardcodes the country name {:?} — load it \
                     from the gazetteer instead",
                    country.place.name
                );
            }
            if let Some(capital) = country.capital.as_deref()
                && capital.len() >= 5
            {
                assert!(
                    !production.contains(capital),
                    "{file}'s production code hardcodes the capital {capital:?} — load \
                     it from the gazetteer instead"
                );
            }
        }
    }
}

/// A specific, real, independently-verifiable fact from the loaded corpus
/// (not a hand-curated fixture): France's capital is Paris, and Germany is
/// a Region-EU member bordering France — cross-checks the loader against
/// ground truth a reader can verify without running the corpus walk above.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn france_and_germany_resolve_correctly_from_the_real_corpus() {
    let gazetteer = gazetteer_loaded();
    let france = gazetteer
        .by_iso("FR")
        .expect("FR (France) is in the 2026-07-21 GeoNames dump");
    assert_eq!(place::capital_of(france), Some("Paris"));
    assert_eq!(france.continent, "EU");
    let germany = gazetteer
        .by_iso("DE")
        .expect("DE (Germany) is in the 2026-07-21 GeoNames dump");
    assert!(place::borders(france, germany));
    assert!(place::borders(germany, france));
}

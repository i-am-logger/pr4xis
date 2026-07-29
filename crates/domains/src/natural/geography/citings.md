# Citings — Geography -- Place/Country/Capital/Region toponymy

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms.

## Primary sources

- Randell, Cui & Cohn 1992: *A Spatial Logic Based on Regions and Connection*, KR'92 -- §3: the RCC-8 relations `contains` (TPP/NTPP) and `borders` (EC), and their ALGEBRAIC properties (symmetry, irreflexivity, functional part-of), ground `RegionContainmentIsRcc8ProperPart` and `BordersIsRcc8ExternalConnection`, checked directly over the loaded gazetteer.
- ISO 3166-1 2020: *Codes for the representation of names of countries and their subdivisions* -- the Country/Capital entity shape and one-country-one-capital convention grounding `EveryCountryHasExactlyOneCapital`.
- GeoNames.org `countryInfo.txt`, fetched 2026-07-21 from `download.geonames.org/export/dump/countryInfo.txt` -- the loaded INSTANCE DATA (`[sources.geonames_countryinfo]` in `praxis.toml`): 252 country/territory rows, `ISO/ISO3/ISO-Numeric/fips/Country/Capital/Area/Population/Continent/tld/CurrencyCode/CurrencyName/Phone/PostalCodeFormat/PostalCodeRegex/Languages/geonameid/neighbours/EquivalentFipsCode`. Data licensed **Creative Commons Attribution (CC BY) 4.0** — GeoNames.org's export terms (<https://www.geonames.org/export/>) require attribution: "You should give credit to GeoNames when using data or web services with a link or another reference to GeoNames." See `crates/domains/data/geography/geonames-LICENSE.txt`.

## Cross-references

- Source attributions per axiom: see the `Source` column in the `## Axioms` table in [`README.md`](README.md)
- Code-level citations: `grep -n 'Randell\|ISO 3166\|GeoNames' ontology.rs place.rs reader.rs store.rs` in this directory
- Composed with (not built from): `../../formal/spatial/rcc8/` -- NO LONGER a direct `rcc8::interval::classify` function call (GeoNames carries no polygon/interval geometry); the 3 axioms cite the SAME paper (Randell, Cui & Cohn 1992 §3) but check its algebraic relation-properties over the real loaded data instead. See `rcc8/README.md` and `rcc8/citings.md`'s own updated cross-reference notes.
- Registry entry: `[sources.geonames_countryinfo]` in the workspace-root `praxis.toml`; pinned in `praxis.lock`'s `[hashes]`/`[compact_archive_signatures]`.

## Resolved (previously pending)

- [x] If a real GeoNames/ISO 3166-1 data source is registered, replace the hand-curated `sample_countries`/`sample_region` gazetteer and re-verify all 3 axioms against it — DONE: `[sources.geonames_countryinfo]` registered 2026-07-21, `place.rs`/`ontology.rs` load and verify against the real 252-row corpus (`store::gazetteer_loaded`), `tests_loaded.rs` adds the generated-from-loaded-data test families.

## Pending verification

Open items for human review:

- [ ] Cross-check both primary sources against `docs/papers/references.md`; add entries if absent

---

- **Document date:** 2026-07-21 (updated for the GeoNames real-corpus migration; originally 2026-07-12)
- **How this file is maintained:** initialized alongside the ontology's first commit (turing-benchmark B3). Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.

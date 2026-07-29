//! Geography — Place/Country/Capital/Region toponymy, grounded against
//! `formal::spatial::rcc8`'s Region Connection Calculus, loaded from the
//! GeoNames `countryInfo.txt` gazetteer (`[sources.geonames_countryinfo]`).

pub mod ontology;
pub mod place;
pub mod reader;
pub mod store;
#[cfg(all(test, feature = "std"))]
mod tests_loaded;

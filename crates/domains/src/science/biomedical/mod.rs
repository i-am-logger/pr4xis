/// Biomedical ontologies — from molecular to organism level.
///
/// Migrated from burp (bioelectric research platform).
/// Models biology, bioelectricity, biochemistry, biophysics, chemistry,
/// hematology, immunology, molecular biology, pharmacology, regeneration,
/// pathology, electrophysiology, mechanobiology, acoustics (bone conduction),
/// analytical methods, optimization, and ontology diagnostics.
///
/// Key references:
/// - Levin, "Bioelectric signaling" (2021) — bioelectricity as information
/// - Alberts et al., "Molecular Biology of the Cell" (6th ed, 2015)
/// - Kandel et al., "Principles of Neural Science" (6th ed, 2021)
pub mod acoustics;
pub mod adjunctions;
pub mod analytical_methods;
pub mod biochemistry;
pub mod bioelectricity;
pub mod biology;
pub mod biophysics;
pub mod chemistry;
pub mod derivation;
pub mod electrophysiology;
pub mod gap_analysis;
pub mod hematology;
pub mod immunology;
pub mod mechanobiology;
pub mod molecular;
pub mod ontology_diagnostics;
pub mod optimization;
pub mod pathology;
pub mod pharmacology;
pub mod recommendation;
pub mod regeneration;

#[cfg(test)]
pub mod composition_tests;

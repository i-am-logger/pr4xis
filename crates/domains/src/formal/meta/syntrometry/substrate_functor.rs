//! Reverse object mapping: Pr4xisSubstrate → Syntrometry.
//!
//! The reverse direction *cannot* satisfy the strict `Functor` laws under
//! the current category structures — Pr4xisSubstrate is dense, Syntrometry
//! is kinded, and the #98 research note
//! (`docs/research/kinded-functor-failures.md`) established that dense
//! sources can't map into kinded targets via strict functors (self-loops
//! from non-identity compositions collapse to Identity in dense source but
//! would need Composed in kinded target). What *is* well-defined is the
//! **object mapping** — which substrate primitive each syntrometric
//! concept was implicitly claimed to live at by the forward lineage
//! functor. That's what powers the gap analysis in
//! [`crate::formal::meta::gap_analysis::analyze_syntrometry_substrate`].
//!
//! No `impl Functor` is provided; the `map_substrate` free function is
//! exposed for the gap analysis module to call directly. Attempting to
//! lift this into a full [`pr4xis::category::Functor`] would re-create the
//! failure mode documented in the research note.

use super::ontology::SyntrometryConcept;
use super::substrate::Pr4xisSubstrateConcept;

/// The canonical syntrometric representative of each substrate primitive.
///
/// | Pr4xisSubstrate | Syntrometry canonical representative |
/// |---|---|
/// | `SubEntity`      | `Predicate`    |
/// | `SubMorphism`    | `Koordination` |
/// | `SubCategory`    | `Syntrix`      |
/// | `SubFunctor`     | `Korporator`   |
/// | `SubEndofunctor` | `Synkolator`   |
/// | `SubOntology`    | `Predikatrix`  |
pub fn map_substrate(c: &Pr4xisSubstrateConcept) -> SyntrometryConcept {
    use Pr4xisSubstrateConcept as P;
    use SyntrometryConcept as S;
    match c {
        // Core primitives.
        P::SubEntity => S::Predicate,
        P::SubMorphism => S::Koordination,
        P::SubCategory => S::Syntrix,
        P::SubFunctor => S::Korporator,
        P::SubEndofunctor => S::Synkolator,
        P::SubOntology => S::Predikatrix,
        // Architectural primitives.
        P::SubEigenform => S::Telecenter,
        P::SubIntention => S::Maxime,
        P::SubStagingLevel => S::Transzendenzstufe,
        P::SubSystemOfSystems => S::Metroplex,
        // Refined sub-kinds — each has a distinct syntrometric counterpart
        // (opposition-structure was removed; it now lives in the dedicated
        // Dialectics ontology reached via `SyntrometryToDialectics`).
        P::SubProductCategory => S::Aspekt,
        P::SubLeveledEntity => S::SyntrixLevel,
        P::SubMereologicalMorphism => S::Part,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;

    /// Every substrate primitive has a canonical syntrometric representative.
    #[test]
    fn every_substrate_primitive_maps() {
        for p in Pr4xisSubstrateConcept::variants() {
            let _ = map_substrate(&p);
        }
    }
}

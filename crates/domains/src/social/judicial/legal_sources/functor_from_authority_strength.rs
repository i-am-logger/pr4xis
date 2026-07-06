//! Cross-functor: the `AuthorityStrength` ontology → the `LegalSources`
//! ontology.
//!
//! `AuthorityStrength` classifies a legal source by *how much binding
//! force* it carries (the vertical binding/persuasive tier ordering).
//! `LegalSources` classifies it by *what kind of source* it is (statute,
//! regulation, case law, …). These are orthogonal dimensions of the same
//! underlying sources. This functor **relates** them — carrying each
//! binding-force leaf to its source-type — without merging the two
//! taxonomies (per the integration-via-functors discipline).
//!
//! # The object map (Kephart-style projection table)
//!
//! | AuthorityStrength concept | LegalSources concept | Why |
//! |---|---|---|
//! | `ConstitutionalText`               | `Constitution` | the constitutional instrument |
//! | `FederalStatute`                   | `Statute`      | enacted legislation |
//! | `FederalRegulation`                | `Regulation`   | administrative rulemaking |
//! | `SupremeCourtPrecedent`            | `Precedent`    | reported case law |
//! | `ControllingCircuitPrecedent`      | `Precedent`    | reported case law |
//! | `SisterCircuitPrecedent`           | `Precedent`    | reported case law |
//! | `DistrictCourtPrecedent`           | `Precedent`    | reported case law |
//! | `AdministrativeReviewBoardDecision`| `Precedent`    | agency adjudication (a decided case) |
//! | `SecondarySource`                  | `LegalSource`  | **partial arm** — see below |
//! | `AuthorityStrength` (root)         | `LegalSource`  | abstract genus ↦ genus |
//! | `BindingAuthority` (branch)        | `LegalSource`  | abstract branch ↦ genus |
//! | `PersuasiveAuthority` (branch)     | `LegalSource`  | abstract branch ↦ genus |
//!
//! **The `SecondarySource` arm is partial/degenerate.** Treatises, law-
//! review articles, and Restatements are persuasive authority (Garner
//! 2016) but are *not* a formal source of law — LKIF-Core has no class
//! for them, since they are commentary *about* the law rather than a
//! source of it. Rather than fabricate a source-type, the functor sends
//! `SecondarySource` to the genus `LegalSource`, the least-committal
//! image. This keeps `map_object` total while recording honestly that no
//! faithful source-type exists.
//!
//! # Why the functor laws hold
//!
//! Both categories, restricted to their `Subsumption` + identity
//! morphisms, are **thin** (at most one morphism between any ordered
//! pair — they are preorder categories). The object map is *monotone*:
//! every source subsumption edge `A ⊑ B` maps to a target morphism
//! `map(A) ⊑ map(B)` (or an identity when `map(A) == map(B)`). A monotone
//! map between thin categories extends uniquely to a functor and
//! preserves composition automatically, so `assert_functor_laws`
//! passes. This functor is **not faithful** — five precedent-family
//! concepts collapse onto `Precedent`, and four abstract/secondary
//! concepts collapse onto `LegalSource`.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category};

use super::ontology::{LegalSourcesCategory, LegalSourcesConcept, LegalSourcesRelation};
use crate::social::judicial::authority_strength::ontology::{
    AuthorityStrengthCategory, AuthorityStrengthConcept, AuthorityStrengthRelation,
};

/// Object map: a binding-force concept ↦ its legal-source type.
fn map_authority(c: &AuthorityStrengthConcept) -> LegalSourcesConcept {
    use AuthorityStrengthConcept as A;
    use LegalSourcesConcept as L;
    match c {
        A::ConstitutionalText => L::Constitution,
        A::FederalStatute => L::Statute,
        A::FederalRegulation => L::Regulation,
        // Every precedent-family concept — including the agency
        // adjudication (ARB) — is reported case law.
        A::SupremeCourtPrecedent
        | A::ControllingCircuitPrecedent
        | A::SisterCircuitPrecedent
        | A::DistrictCourtPrecedent
        | A::AdministrativeReviewBoardDecision => L::Precedent,
        // Partial arm: secondary sources are not a formal source-type;
        // send to the genus. See the module doc-comment.
        A::SecondarySource => L::LegalSource,
        // Abstract root + branches ↦ the genus.
        A::AuthorityStrength | A::BindingAuthority | A::PersuasiveAuthority => L::LegalSource,
    }
}

/// Morphism map: carry a source subsumption/identity edge to the unique
/// target morphism between the mapped endpoints. Both categories are
/// thin (preorder) on Subsumption + identity, so this selection is
/// unambiguous: an identity when the endpoints collapse, otherwise the
/// single materialised subsumption edge `map(from) ⊑ map(to)`.
fn map_authority_morphism(m: &AuthorityStrengthRelation) -> LegalSourcesRelation {
    let s = map_authority(&m.source());
    let t = map_authority(&m.target());
    if s == t {
        LegalSourcesCategory::identity(&s)
    } else {
        LegalSourcesCategory::morphisms()
            .into_iter()
            .find(|r| r.source() == s && r.target() == t)
            .expect("monotone object map: target subsumption edge must exist in the closed set")
    }
}

pr4xis::functor! {
    name: AuthorityStrengthToLegalSources,
    source: AuthorityStrengthCategory,
    target: LegalSourcesCategory,
    citation: "Hoekstra et al. (2007) LKIF-Core; Schauer (2009) Thinking Like a Lawyer (binding force vs. source type as orthogonal dimensions); Salmond on Jurisprudence",
    map_object: |obj: &AuthorityStrengthConcept| -> LegalSourcesConcept { map_authority(obj) },
    map_morphism: |m: &AuthorityStrengthRelation| -> LegalSourcesRelation {
        map_authority_morphism(m)
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Functor;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<AuthorityStrengthToLegalSources>();
    }

    #[pr4xis::praxis_value(Verifiable, Explainable)]
    #[test]
    fn functor_has_meta() {
        let meta = AuthorityStrengthToLegalSources::meta();
        assert_eq!(meta.name.as_str(), "AuthorityStrengthToLegalSources");
        assert!(!meta.citation.as_str().is_empty());
        assert!(meta.module_path.as_str().contains("legal_sources"));
    }

    /// Concrete sanity: every binding-force leaf lands on its documented
    /// source-type.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn object_map_matches_table() {
        use AuthorityStrengthConcept as A;
        use LegalSourcesConcept as L;
        let f = AuthorityStrengthToLegalSources::map_object;
        assert_eq!(f(&A::ConstitutionalText), L::Constitution);
        assert_eq!(f(&A::FederalStatute), L::Statute);
        assert_eq!(f(&A::FederalRegulation), L::Regulation);
        assert_eq!(f(&A::SupremeCourtPrecedent), L::Precedent);
        assert_eq!(f(&A::ControllingCircuitPrecedent), L::Precedent);
        assert_eq!(f(&A::SisterCircuitPrecedent), L::Precedent);
        assert_eq!(f(&A::DistrictCourtPrecedent), L::Precedent);
        assert_eq!(f(&A::AdministrativeReviewBoardDecision), L::Precedent);
        // Partial arm + abstract concepts collapse onto the genus.
        assert_eq!(f(&A::SecondarySource), L::LegalSource);
        assert_eq!(f(&A::AuthorityStrength), L::LegalSource);
        assert_eq!(f(&A::BindingAuthority), L::LegalSource);
        assert_eq!(f(&A::PersuasiveAuthority), L::LegalSource);
    }
}

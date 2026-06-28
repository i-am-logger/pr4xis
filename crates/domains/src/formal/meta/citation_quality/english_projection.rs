//! Functor: CitationQuality → English. Projects each citation-quality
//! dimension to its canonical English head-noun phrase, giving Praxis a
//! plain-language reading of the model for explanation surfaces (the
//! chat reply that tells a user *why* a citation is `ValidWithIssues`)
//! rather than an opaque enum.
//!
//! ## Categorical setting
//!
//! Per Mac Lane *Categories for the Working Mathematician* §I.3, a
//! functor F: C → D is a structure-preserving map. Here:
//!
//! - **C** is [`super::ontology::CitationQualityCategory`] — the five
//!   dimensions plus the `CitationQuality` root, with the `Subsumption`
//!   (`is_a`) morphisms the ontology macro encodes.
//! - **D** is [`CitationQualityEnglishLabelCategory`] — one object per
//!   dimension, labelled by its canonical English phrase, carrying the
//!   same subsumption edges so the functor preserves the source's
//!   structure (Spivak 2014 §5), not just its names.
//!
//! Identity preservation and composition preservation both hold by
//! construction and are checked by `assert_functor_laws` in the tests.
//!
//! ## Citation
//!
//! - Mac Lane, S. (1998) *Categories for the Working Mathematician*,
//!   Springer GTM 5, 2nd ed., §I.1 (identities), §I.3 (functors).
//! - Spivak, D. I. (2014) *Category Theory for the Sciences*, MIT
//!   Press, §5 (functorial structure preservation).
//! - Smith, B. et al. (2005) "Relations in biomedical ontologies",
//!   *Genome Biology* 6:R46 — OBO-RO relation-kind tagging.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::ontology::{CitationQualityCategory, CitationQualityConcept, CitationQualityRelation};

// =============================================================================
// Target category — one English-labelled object per dimension.
// =============================================================================

/// The English-projection label of a [`CitationQualityConcept`]. Each
/// variant is the dimension paired with the canonical English phrase
/// for it (see [`canonical_english_phrase`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, pr4xis::category::Concept)]
pub enum CitationQualityEnglishLabel {
    CitationQuality,
    Existence,
    ClaimSupport,
    LocatorAccuracy,
    BibliographicAccuracy,
    FormatConformance,
}

/// The canonical English head-noun phrase for each citation-quality
/// concept — the plain-language name an explanation surface uses.
pub fn canonical_english_phrase(c: CitationQualityConcept) -> &'static str {
    use CitationQualityConcept as C;
    match c {
        C::CitationQuality => "citation quality",
        C::Existence => "existence",
        C::ClaimSupport => "claim support",
        C::LocatorAccuracy => "locator accuracy",
        C::BibliographicAccuracy => "bibliographic accuracy",
        C::FormatConformance => "format conformance",
    }
}

/// Relation-kind tag for [`CitationQualityEnglishLabelCategory`]:
/// `Identity` (Mac Lane §I.1) and `Subsumption` (the projected `is_a`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CitationQualityEnglishLabelRelationKind {
    Identity,
    Subsumption,
}

/// Morphism in [`CitationQualityEnglishLabelCategory`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CitationQualityEnglishLabelMorphism {
    pub from: CitationQualityEnglishLabel,
    pub to: CitationQualityEnglishLabel,
    pub kind: CitationQualityEnglishLabelRelationKind,
}

impl CitationQualityEnglishLabelMorphism {
    /// Identity morphism on `label` (Mac Lane §I.1).
    pub fn identity(label: CitationQualityEnglishLabel) -> Self {
        Self {
            from: label,
            to: label,
            kind: CitationQualityEnglishLabelRelationKind::Identity,
        }
    }

    /// Subsumption (`is_a`) morphism `child → parent`.
    pub fn subsumption(
        child: CitationQualityEnglishLabel,
        parent: CitationQualityEnglishLabel,
    ) -> Self {
        Self {
            from: child,
            to: parent,
            kind: CitationQualityEnglishLabelRelationKind::Subsumption,
        }
    }
}

impl Arrow for CitationQualityEnglishLabelMorphism {
    type Object = CitationQualityEnglishLabel;
    type Kind = CitationQualityEnglishLabelRelationKind;

    fn source(&self) -> CitationQualityEnglishLabel {
        self.from
    }
    fn target(&self) -> CitationQualityEnglishLabel {
        self.to
    }
    fn kind(&self) -> CitationQualityEnglishLabelRelationKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new(format!(
                "CitationQualityEnglishLabel-{:?}-{:?}-{:?}",
                self.kind, self.from, self.to
            )),
            description: Label::new(format!(
                "{:?} morphism on citation-quality English labels {:?} → {:?}",
                self.kind, self.from, self.to
            )),
            citation: Citation::parse_static(
                "Mac Lane (1998) Categories for the Working Mathematician §I.1 (identities), \
                 §I.3 (functors); Spivak (2014) Category Theory for the Sciences §5 \
                 (functorial structure preservation); Smith et al. (2005) Genome Biology 6:R46 \
                 OBO-RO (relation-kind tagging)",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

/// The Subsumption edges `(child, parent)` in the projection category —
/// the image of the CitationQuality ontology's `is_a` hierarchy. Every
/// dimension subsumes under the `CitationQuality` root; the root has no
/// parent, so the relation is already transitively closed.
fn subsumption_pairs() -> [(CitationQualityEnglishLabel, CitationQualityEnglishLabel); 5] {
    use CitationQualityEnglishLabel as L;
    [
        (L::Existence, L::CitationQuality),
        (L::ClaimSupport, L::CitationQuality),
        (L::LocatorAccuracy, L::CitationQuality),
        (L::BibliographicAccuracy, L::CitationQuality),
        (L::FormatConformance, L::CitationQuality),
    ]
}

/// Category of English-projection labels for citation-quality concepts.
pub struct CitationQualityEnglishLabelCategory;

impl Category for CitationQualityEnglishLabelCategory {
    type Object = CitationQualityEnglishLabel;
    type Morphism = CitationQualityEnglishLabelMorphism;

    fn identity(obj: &CitationQualityEnglishLabel) -> CitationQualityEnglishLabelMorphism {
        CitationQualityEnglishLabelMorphism::identity(*obj)
    }

    fn compose(
        f: &CitationQualityEnglishLabelMorphism,
        g: &CitationQualityEnglishLabelMorphism,
    ) -> Option<CitationQualityEnglishLabelMorphism> {
        if f.to != g.from {
            return None;
        }
        use CitationQualityEnglishLabelRelationKind as K;
        match (f.kind, g.kind) {
            (K::Identity, _) => Some(*g),
            (_, K::Identity) => Some(*f),
            // Subsumption is transitive (OBO-RO `transitive_over`); here
            // the only composable chains end at the root, which has no
            // outgoing subsumption, so this arm is vacuous in practice
            // but kept for totality.
            (K::Subsumption, K::Subsumption) => Some(
                CitationQualityEnglishLabelMorphism::subsumption(f.from, g.to),
            ),
        }
    }

    fn morphisms() -> Vec<CitationQualityEnglishLabelMorphism> {
        let mut out: Vec<CitationQualityEnglishLabelMorphism> =
            CitationQualityEnglishLabel::variants()
                .into_iter()
                .map(CitationQualityEnglishLabelMorphism::identity)
                .collect();
        for (child, parent) in subsumption_pairs() {
            out.push(CitationQualityEnglishLabelMorphism::subsumption(
                child, parent,
            ));
        }
        out
    }
}

impl pr4xis::category::NamedCategory for CitationQualityEnglishLabelCategory {
    fn ontology_name() -> OntologyName {
        OntologyName::new_static("CitationQualityEnglishLabel")
    }
}

// =============================================================================
// Object map + the functor.
// =============================================================================

/// Map a [`CitationQualityConcept`] to its English-projection label.
/// Bijection between the six concepts and the six label variants.
pub fn project_concept(c: CitationQualityConcept) -> CitationQualityEnglishLabel {
    use CitationQualityConcept as C;
    use CitationQualityEnglishLabel as L;
    match c {
        C::CitationQuality => L::CitationQuality,
        C::Existence => L::Existence,
        C::ClaimSupport => L::ClaimSupport,
        C::LocatorAccuracy => L::LocatorAccuracy,
        C::BibliographicAccuracy => L::BibliographicAccuracy,
        C::FormatConformance => L::FormatConformance,
    }
}

/// The canonical English phrase for a projected label — the
/// composition `canonical_english_phrase ∘ (project_concept⁻¹)`,
/// exposed directly for explanation surfaces.
pub fn label_phrase(label: CitationQualityEnglishLabel) -> &'static str {
    use CitationQualityEnglishLabel as L;
    let concept = match label {
        L::CitationQuality => CitationQualityConcept::CitationQuality,
        L::Existence => CitationQualityConcept::Existence,
        L::ClaimSupport => CitationQualityConcept::ClaimSupport,
        L::LocatorAccuracy => CitationQualityConcept::LocatorAccuracy,
        L::BibliographicAccuracy => CitationQualityConcept::BibliographicAccuracy,
        L::FormatConformance => CitationQualityConcept::FormatConformance,
    };
    canonical_english_phrase(concept)
}

pr4xis::functor! {
    name: CitationQualityToEnglish,
    source: CitationQualityCategory,
    target: CitationQualityEnglishLabelCategory,
    citation: "Mac Lane (1998) Categories for the Working Mathematician §I.3 (Functors); \
               Spivak (2014) Category Theory for the Sciences §5 (functorial structure \
               preservation)",
    map_object: |c: &CitationQualityConcept| -> CitationQualityEnglishLabel {
        project_concept(*c)
    },
    map_morphism: |m: &CitationQualityRelation| -> CitationQualityEnglishLabelMorphism {
        // Identity maps to identity on the projected object (Mac Lane
        // §I.3); Subsumption edges project to Subsumption edges,
        // preserving the `is_a` hierarchy (Spivak 2014 §5). The macro
        // emits Parthood/Causation/Opposition slots even though the
        // CitationQuality ontology has no such edges; they collapse to
        // identities (never exercised — no source morphisms of those
        // kinds exist).
        use super::ontology::CitationQualityRelationKind as SK;
        let src = project_concept(m.from);
        let dst = project_concept(m.to);
        match m.kind {
            SK::Identity => CitationQualityEnglishLabelMorphism::identity(src),
            SK::Subsumption => CitationQualityEnglishLabelMorphism::subsumption(src, dst),
            SK::Parthood | SK::Causation | SK::Opposition => {
                CitationQualityEnglishLabelMorphism::identity(src)
            }
        }
    },
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Functor;
    use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn target_category_laws_pass() {
        assert_category_laws::<CitationQualityEnglishLabelCategory>();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_pass() {
        assert_functor_laws::<CitationQualityToEnglish>();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_preserves_identity_explicit() {
        for c in CitationQualityConcept::variants() {
            let id_src = CitationQualityCategory::identity(&c);
            let mapped = CitationQualityToEnglish::map_morphism(&id_src);
            let id_tgt = CitationQualityEnglishLabelCategory::identity(
                &CitationQualityToEnglish::map_object(&c),
            );
            assert_eq!(
                mapped, id_tgt,
                "identity on {c:?} must map to target identity"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn project_concept_is_injective() {
        let labels: Vec<_> = CitationQualityConcept::variants()
            .into_iter()
            .map(project_concept)
            .collect();
        let mut deduped = labels.clone();
        deduped.sort_by_key(|l| format!("{l:?}"));
        deduped.dedup();
        assert_eq!(
            labels.len(),
            deduped.len(),
            "project_concept must be injective"
        );
    }

    #[pr4xis::praxis_value(Explainable, Deterministic)]
    #[test]
    fn every_concept_has_nonempty_english_phrase() {
        for c in CitationQualityConcept::variants() {
            assert!(
                !canonical_english_phrase(c).is_empty(),
                "{c:?} must have an English phrase"
            );
            // The label round-trips to the same phrase.
            assert_eq!(
                canonical_english_phrase(c),
                label_phrase(project_concept(c))
            );
        }
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn functor_meta_carries_citation() {
        let meta = CitationQualityToEnglish::meta();
        assert_eq!(meta.name.as_str(), "CitationQualityToEnglish");
        assert!(meta.citation.as_str().contains("Mac Lane"));
    }

    // ── Property-based laws (Mac Lane §I.3) ────────────────────────────
    use proptest::prelude::*;

    fn arb_concept() -> impl Strategy<Value = CitationQualityConcept> {
        let v = CitationQualityConcept::variants();
        (0..v.len()).prop_map(move |i| CitationQualityConcept::variants()[i])
    }

    proptest! {
        /// Composition preservation: for any composable pair of source
        /// morphisms, F(g∘f) = F(g)∘F(f) (Mac Lane §I.3).
        #[test]
        fn prop_functor_preserves_composition(i in 0usize..256, j in 0usize..256) {
            let ms = CitationQualityCategory::morphisms();
            let f = &ms[i % ms.len()];
            let g = &ms[j % ms.len()];
            if let Some(fg) = CitationQualityCategory::compose(f, g) {
                let mapped = CitationQualityToEnglish::map_morphism(&fg);
                let mf = CitationQualityToEnglish::map_morphism(f);
                let mg = CitationQualityToEnglish::map_morphism(g);
                prop_assert_eq!(
                    Some(mapped),
                    CitationQualityEnglishLabelCategory::compose(&mf, &mg)
                );
            }
        }

        /// Identity preservation: F(id_c) = id_{F(c)} (Mac Lane §I.3).
        #[test]
        fn prop_functor_preserves_identity(c in arb_concept()) {
            let mapped = CitationQualityToEnglish::map_morphism(&CitationQualityCategory::identity(&c));
            let id_tgt = CitationQualityEnglishLabelCategory::identity(
                &CitationQualityToEnglish::map_object(&c),
            );
            prop_assert_eq!(mapped, id_tgt);
        }

        /// The object map is injective: distinct concepts get distinct
        /// labels, and every label round-trips to the same phrase.
        #[test]
        fn prop_object_map_injective_and_phrase_roundtrips(a in arb_concept(), b in arb_concept()) {
            prop_assert_eq!(a == b, project_concept(a) == project_concept(b));
            prop_assert_eq!(canonical_english_phrase(a), label_phrase(project_concept(a)));
        }
    }

    pr4xis::register_praxis_value!(prop_functor_preserves_composition, Extensible);
    pr4xis::register_praxis_value!(prop_functor_preserves_identity, Extensible);
    pr4xis::register_praxis_value!(
        prop_object_map_injective_and_phrase_roundtrips,
        Deterministic,
        Verifiable
    );
}

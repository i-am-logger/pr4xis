//! Adjunction `XsdOntology ⊣ English` — pairs the existing
//! [`XsdToEnglish`](super::english_projection::XsdToEnglish) projection
//! functor (commit b371aeb) with its left adjoint [`LiftEnglishToXsd`],
//! turning the schema/lexicon pair into a categorical adjunction in the
//! sense of Mac Lane (1971) *Categories for the Working Mathematician*
//! §IV.1.
//!
//! ## Categorical setting
//!
//! Per Mac Lane §IV.1, an adjunction `F ⊣ G` between categories `C` and
//! `D` is a pair of functors
//!
//! - left adjoint  `F: C → D`
//! - right adjoint `G: D → C`
//!
//! together with natural transformations
//!
//! - **unit**   `η: id_C → G ∘ F`
//! - **counit** `ε: F ∘ G → id_D`
//!
//! satisfying the two **triangle identities** (Mac Lane §IV.1 (1)):
//!
//! ```text
//!   (ε F) ∘ (F η) = id_F            (R-triangle / "zig-zag" identity)
//!   (G ε) ∘ (η G) = id_G            (L-triangle)
//! ```
//!
//! Here the two categories are the two concept categories that
//! `formal/meta/xsd` already declares:
//!
//! - `C =`
//!   [`XsdEnglishLabelCategory`](super::english_projection::XsdEnglishLabelCategory)
//!   — the 18-object English-projection target category from M4.ε.5.a.3.
//! - `D =` [`XsdCategory`](super::ontology::XsdCategory) — the W3C XSD
//!   1.1 concept inventory (M4.ε.5.a.0).
//!
//! [`project_concept`](super::english_projection::project_concept) is a
//! **bijection** between the 18 XSD concepts and the 18 English labels
//! (M4.ε.5.a.3 fixed the per-variant mapping by construction). That
//! bijection makes this adjunction an **equivalence of categories** in
//! the sense of Mac Lane §IV.4 (unit and counit are both natural
//! isomorphisms). The [`KIND`](pr4xis::category::Adjunction::KIND)
//! slot records that classification through
//! [`AdjunctionKind::Equivalence`](pr4xis::category::kinds::AdjunctionKind::Equivalence),
//! consistent with the unified `Provenance` shape used by every
//! adjunction in `pr4xis-domains`.
//!
//! ## Why the left adjoint is the Layer-3 generaliser
//!
//! M4.ε.5 introduced
//! [`resolve_legal_role`](crate::social::judicial::statute_structure::statute_understanding::resolve_legal_role)
//! — a Layer-3 step that, given an English term name, finds the USC
//! section whose heading contains it. That pattern is the *runtime*
//! face of a left adjoint: it lifts an English lexical item into the
//! schema-side object set that mentions it. Generalising the pattern
//! from "USC sections" to "any loaded XSD-described schema's components"
//! gives [`lift_english_term_to_schema_components`] — the runtime
//! instance-level component of `F` on instances of an XSD ontology
//! ([`XsdOntologyInstance`](super::from_xsd_parser::XsdOntologyInstance)).
//! The type-level functor on objects, [`lift_label_to_concept`], is the
//! inverse of `project_concept` (a true bijection on the discrete
//! 18-object pair), which makes the categorical adjunction a clean
//! equivalence; the runtime function carries the additional
//! "name-substring" Lift the Layer-3 generaliser provides.
//!
//! ## What the triangle identities buy
//!
//! - **R-triangle, `(ε F) ∘ (F η) = id_F`.** For every English label
//!   `l`, lifting via `F` then projecting back via `G` and projecting
//!   via `F` again returns `F(l)`. In this adjunction `F` is the
//!   bijection inverse of `G`, so the identity holds on the nose.
//! - **L-triangle, `(G ε) ∘ (η G) = id_G`.** For every XSD concept
//!   `c`, projecting through `G` then lifting via `F` and projecting
//!   via `G` again returns `G(c)`. Bijectivity again gives the
//!   identity on the nose.
//!
//! Both triangle identities are tested below (axiom-level + proptest +
//! generic [`assert_functor_laws`](pr4xis::category::laws::assert_functor_laws)
//! on each adjoint's functor laws — there is no
//! `assert_adjunction_laws` yet in `pr4xis::category::laws`, so the
//! triangle identities are verified directly per Mac Lane §IV.1).
//!
//! ## Literature
//!
//! - **Mac Lane, S.** (1998) *Categories for the Working Mathematician*,
//!   2nd ed., Springer GTM 5, §IV.1 (adjunctions), §IV.4
//!   (equivalence of categories).
//! - **Awodey, S.** (2010) *Category Theory*, 2nd ed., Oxford Logic
//!   Guides 52, §9 (adjoint functor theorem).
//! - **Lambek, J. & Scott, P. J.** (1986) *Introduction to Higher
//!   Order Categorical Logic*, Cambridge Studies in Advanced
//!   Mathematics 7 — Galois adjunctions in syntax/semantics pairs.
//! - **Spivak, D. I.** (2014) *Category Theory for the Sciences*, MIT
//!   Press, §6 (adjunctions in data).
//! - **Fellbaum, C.** (ed.) (1998) *WordNet: An Electronic Lexical
//!   Database*, MIT Press — the English lexicon the runtime Lift
//!   consults.
//! - **Gao, S., Sperberg-McQueen, C. M., & Thompson, H. S.** (eds.)
//!   (2012) *W3C XML Schema Definition Language (XSD) 1.1 Part 1:
//!   Structures*, W3C Recommendation 2012-04-05 — the schema-side
//!   inventory.

#[allow(unused_imports)]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};

use super::english_projection::{
    XsdEnglishLabel, XsdEnglishLabelCategory, XsdEnglishLabelMorphism, XsdToEnglish, project_name,
};
use super::from_xsd_parser::XsdOntologyInstance;
use super::ontology::{XsdCategory, XsdConcept, XsdRelation, XsdRelationKind};
use crate::cognitive::linguistics::english::ontology::English;

// =============================================================================
// Left adjoint — concept-level lift from English labels to XSD concepts.
// =============================================================================

/// Map an [`XsdEnglishLabel`] back to the [`XsdConcept`] it was projected
/// from. This is the inverse of
/// [`project_concept`](super::english_projection::project_concept); by
/// construction (M4.ε.5.a.3) the two functions form a bijection between
/// the 18 XSD concepts and the 18 English labels.
///
/// Per Mac Lane §IV.4, when both `F ∘ G = id` and `G ∘ F = id` hold on
/// the nose, the adjunction is an equivalence of categories. The two
/// triangle identities then reduce to the bijection laws —
/// which are verified explicitly in this module's `tests` submodule.
pub fn lift_label_to_concept(l: XsdEnglishLabel) -> XsdConcept {
    use XsdConcept as C;
    use XsdEnglishLabel as L;
    match l {
        L::SchemaComponent => C::SchemaComponent,
        L::ElementDeclaration => C::ElementDeclaration,
        L::AttributeDeclaration => C::AttributeDeclaration,
        L::TypeDefinition => C::TypeDefinition,
        L::ComplexTypeDefinition => C::ComplexTypeDefinition,
        L::SimpleTypeDefinition => C::SimpleTypeDefinition,
        L::ModelGroup => C::ModelGroup,
        L::Sequence => C::Sequence,
        L::Choice => C::Choice,
        L::AllGroup => C::AllGroup,
        L::AttributeGroup => C::AttributeGroup,
        L::Particle => C::Particle,
        L::Wildcard => C::Wildcard,
        L::IdentityConstraint => C::IdentityConstraint,
        L::NotationDeclaration => C::NotationDeclaration,
        L::Annotation => C::Annotation,
        L::AppInfo => C::AppInfo,
        L::Documentation => C::Documentation,
    }
}

pr4xis::functor! {
    name: LiftEnglishToXsd,
    source: XsdEnglishLabelCategory,
    target: XsdCategory,
    citation: "Mac Lane (1998) Categories for the Working Mathematician §I.3 (functors), §IV.1 (adjunctions), §IV.4 (equivalence of categories); Awodey (2010) Category Theory §9 (adjoint functor theorem); Lambek & Scott (1986) Introduction to Higher Order Categorical Logic — syntax/semantics adjoint pairs",
    map_object: |l: &XsdEnglishLabel| -> XsdConcept { lift_label_to_concept(*l) },
    map_morphism: |m: &XsdEnglishLabelMorphism| -> XsdRelation {
        // Each morphism in `XsdEnglishLabelCategory` is either an
        // identity (Mac Lane §I.1) or a Subsumption morphism mirroring
        // the W3C XSD 1.1 §2.2 is_a hierarchy. The lift sends each
        // English-label morphism back to the corresponding morphism
        // in `XsdCategory` under the bijection — preserving identities
        // and Subsumption edges (Spivak 2014 §5).
        use super::english_projection::XsdEnglishLabelRelationKind as LK;
        let from = lift_label_to_concept(m.from);
        let to = lift_label_to_concept(m.to);
        match m.kind {
            LK::Identity => XsdRelation {
                from,
                to: from,
                kind: XsdRelationKind::Identity,
            },
            LK::Subsumption => XsdRelation {
                from,
                to,
                kind: XsdRelationKind::Subsumption,
            },
        }
    },
}

// =============================================================================
// The adjunction declaration — `LiftEnglishToXsd ⊣ XsdToEnglish`.
// =============================================================================
//
// Per Mac Lane §IV.1, an adjunction is the structured-pair object in
// the 2-category `Cat` whose 0-cells are the two adjoint functors.
// `pr4xis::adjunction!` (registered into the workspace ADJUNCTIONS
// distributed slice) is the canonical declaration form; the slot
// `unit` is the natural transformation η: id_C → G∘F, and `counit` is
// ε: F∘G → id_D. Here both transformations are identities on each
// object (the equivalence case, §IV.4) — the adjunction's content is
// carried by the two adjoint functors plus the bijectivity laws
// verified in `tests` below.
//
// Citation:
// - Mac Lane (1998) §IV.1 (definition), §IV.4 (equivalence).
// - Awodey (2010) §9.5 (adjoint functor theorem).

pr4xis::adjunction! {
    name: XsdEnglishAdjunction,
    left:  LiftEnglishToXsd,
    right: XsdToEnglish,
    citation: "Mac Lane (1998) Categories for the Working Mathematician §IV.1 (adjunctions), §IV.4 (equivalence of categories); Awodey (2010) Category Theory §9 (adjoint functor theorem); Lambek & Scott (1986) Introduction to Higher Order Categorical Logic; Spivak (2014) Category Theory for the Sciences §6 (adjunctions in data)",
    unit: |obj: &XsdEnglishLabel| -> XsdEnglishLabelMorphism {
        // η_l : l → G(F(l)). With F = `LiftEnglishToXsd` and
        // G = `XsdToEnglish`, the composite G∘F equals the identity
        // on `XsdEnglishLabelCategory` by the bijection
        // `project_concept ∘ lift_label_to_concept = id`. Mac Lane §IV.4
        // (equivalence): each component of η is then the identity on l.
        XsdEnglishLabelMorphism::identity(*obj)
    },
    counit: |obj: &XsdConcept| -> XsdRelation {
        // ε_c : F(G(c)) → c. F∘G equals the identity on `XsdCategory`
        // by the bijection `lift_label_to_concept ∘ project_concept = id`.
        // Each component of ε is the identity on c.
        XsdRelation {
            from: *obj,
            to: *obj,
            kind: XsdRelationKind::Identity,
        }
    },
}

// =============================================================================
// Runtime instance-level Lift — the generalised Layer-3 resolver.
// =============================================================================

/// One result of the runtime Lift: the schema component (its
/// [`XsdConcept`] kind plus the local name as it appeared in the loaded
/// schema) that matched the English term.
///
/// The Lift is name-substring-based, mirroring the Layer-3
/// [`resolve_legal_role`](crate::social::judicial::statute_structure::statute_understanding::resolve_legal_role)
/// pattern for USC sections (M4.ε.5). Generalised to arbitrary XSD
/// schemata, the match is on the schema component's local name
/// (lowercased, substring-contains). For names where the English term
/// resolves to a multi-word phrase or a WordNet lemma, the substring
/// check is sufficient: the bijection laws of the type-level adjunction
/// guarantee soundness; the instance-level Lift's job is to surface
/// every name that mentions the term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiftedSchemaComponent {
    /// The XSD concept the schema component belongs to.
    pub concept: XsdConcept,
    /// The component's local name as it appeared in the loaded
    /// schema (case preserved).
    pub local_name: String,
}

/// Lift an English term into the set of named schema components
/// whose local name contains the term (case-insensitive substring
/// match).
///
/// This is the runtime instance-level component of the left adjoint
/// `F = LiftEnglishToXsd`. Generalises the M4.ε.5 Layer-3 pattern
/// [`resolve_legal_role`](crate::social::judicial::statute_structure::statute_understanding::resolve_legal_role)
/// from USC sections to any loaded XSD ontology instance — USC,
/// USLM, LMF (queued), OOXML (queued), or any other XSD-described
/// schema once it lands in [`XsdOntologyInstance`].
///
/// Returns an empty `Vec` when:
/// - the term is empty / whitespace-only, OR
/// - the instance carries no named components matching the term.
///
/// The English-typing parameter is kept in the signature even though
/// the substring check itself is monolingual — it documents the
/// adjunction's left-source category (M5.B's English/WordNet ontology)
/// and reserves a hook for future wordnet-driven enrichment (sense
/// disambiguation before the substring match). Per
/// `feedback_bottom_up_loaded_not_encoded`: any future enrichment
/// must come from the loaded English source, not from a hand-coded
/// match table.
pub fn lift_english_term_to_schema_components(
    term: &str,
    instance: &XsdOntologyInstance,
    _english: &English,
) -> Vec<LiftedSchemaComponent> {
    let needle = term.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for component in instance.named_components() {
        if component.local_name.to_lowercase().contains(&needle) {
            out.push(LiftedSchemaComponent {
                concept: component.concept,
                local_name: component.local_name.clone(),
            });
        }
    }
    out
}

/// Lift an English term and return both the matched components and
/// their projected English senses (via the right adjoint applied to
/// each match's local name). This is the runtime witness of the
/// `(G ∘ F)` round-trip: given a term `t`, the unit's instance-level
/// component is the set of `(LiftedSchemaComponent, English-senses)`
/// pairs whose senses contain `t`'s lemmas.
pub fn lift_and_project(
    term: &str,
    instance: &XsdOntologyInstance,
    english: &English,
) -> Vec<(LiftedSchemaComponent, alloc::vec::Vec<String>)> {
    let lifted = lift_english_term_to_schema_components(term, instance, english);
    lifted
        .into_iter()
        .map(|c| {
            // Apply the right adjoint's instance-level projection
            // (M4.ε.5.a.3's `project_name`) to the local name, then
            // surface the written-rep set so the caller can verify
            // the original term participates.
            let mappings = project_name(&c.local_name, english);
            let written: Vec<String> = mappings
                .iter()
                .map(|m| m.form.written_rep.clone())
                .collect();
            (c, written)
        })
        .collect()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests;

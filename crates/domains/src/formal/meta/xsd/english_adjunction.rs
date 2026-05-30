//! XsdOntology ⊣ English adjunction.
//!
//! This is an *adjunction*, not a *lens*. The right adjoint
//! (XsdOntology → English) projects element/attribute names +
//! documentation through WordNet — a LOSSY projection: multiple
//! distinct SchemaComponents can project to overlapping English
//! sense sets, so the projection cannot bijectively invert back to
//! the original schema component.
//!
//! What makes the round-trip work is the **complement**: the
//! structural XSD ontology data itself, which English does not
//! carry. Bancilhon & Spyratos (1981) formalized this as the
//! "constant complement" of a view; Foster et al. (2007) bundle
//! the complement implicitly into the `put` function of a
//! well-behaved lens. Hofmann, Pierce & Wagner (2011) make the
//! complement explicit in their symmetric-lens framework.
//!
//! For praxis: chat reasons through `&English` + `&UsCode` together
//! — the English adjoint plus the structural complement. Together
//! they're sufficient. English alone is not.
//!
//! # Where this sits in the praxis categorical chain
//!
//! See [`formal::meta::categorical_structure`](super::super::categorical_structure)
//! for the full architectural picture: lenses compose horizontally
//! (bytes ↔ XML AST ↔ XSD ontology — lossless); adjunctions fan
//! outward (XSD ontology ⊣ English / Statute / OWL — lossy
//! projections).
//!
//! # Literature
//!
//! - Bancilhon, F. & Spyratos, N. (1981). "Update Semantics of
//!   Relational Views". *ACM Transactions on Database Systems*
//!   6(4):557–575. [Constant-complement formulation of view-update.]
//! - Foster, J. N.; Greenwald, M. B.; Moore, J. T.; Pierce, B. C.;
//!   Schmitt, A. (2007). "Combinators for Bidirectional Tree
//!   Transformations". *ACM TOPLAS* 29(3) Article 17. §3 lens
//!   composition combinators.
//! - Hofmann, M.; Pierce, B. C.; Wagner, D. (2011). "Symmetric
//!   Lenses". *POPL '11* — explicit-complement symmetric lenses.
//! - Mac Lane, S. (1971). *Categories for the Working
//!   Mathematician*. §IV.1 (Adjunctions), §IV.4 (Equivalence of
//!   Categories — the strictly-stronger form not applicable here).

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
/// [`project_concept`](super::english_projection::project_concept) on
/// the discrete 18-object label set; by construction (M4.ε.5.a.3) the
/// two functions form a bijection between the 18 XSD concepts and the
/// 18 English labels.
///
/// On this discrete pair the bijection makes the *type-level*
/// triangle identities collapse to identity, but the broader
/// `XsdOntology ⊣ English` projection is still an adjunction with
/// complement (see module docs) — the instance-level English
/// projection (`project_name` on WordNet senses) is lossy.
/// Adjunction discipline (Mac Lane §IV.1) is the correct frame;
/// equivalence (§IV.4) is the strictly stronger claim that does
/// *not* hold once the projection ranges over WordNet senses.
pub fn lift_label_to_concept(l: XsdEnglishLabel) -> XsdConcept {
    use XsdConcept as C;
    use XsdEnglishLabel as L;
    match l {
        L::SchemaDocument => C::SchemaDocument,
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
        L::SchemaCompositionDirective => C::SchemaCompositionDirective,
        L::SchemaImport => C::SchemaImport,
        L::SchemaInclude => C::SchemaInclude,
        L::SchemaRedefine => C::SchemaRedefine,
        L::SchemaOverride => C::SchemaOverride,
        L::TypeConstructionConstruct => C::TypeConstructionConstruct,
        L::ComplexContent => C::ComplexContent,
        L::SimpleContent => C::SimpleContent,
        L::Restriction => C::Restriction,
        L::Extension => C::Extension,
        L::ListType => C::ListType,
        L::UnionType => C::UnionType,
        L::ConstrainingFacet => C::ConstrainingFacet,
        L::LengthFacet => C::LengthFacet,
        L::MinLengthFacet => C::MinLengthFacet,
        L::MaxLengthFacet => C::MaxLengthFacet,
        L::PatternFacet => C::PatternFacet,
        L::EnumerationFacet => C::EnumerationFacet,
        L::WhiteSpaceFacet => C::WhiteSpaceFacet,
        L::MaxInclusiveFacet => C::MaxInclusiveFacet,
        L::MaxExclusiveFacet => C::MaxExclusiveFacet,
        L::MinExclusiveFacet => C::MinExclusiveFacet,
        L::MinInclusiveFacet => C::MinInclusiveFacet,
        L::TotalDigitsFacet => C::TotalDigitsFacet,
        L::FractionDigitsFacet => C::FractionDigitsFacet,
        L::ExplicitTimezoneFacet => C::ExplicitTimezoneFacet,
        L::AssertionFacet => C::AssertionFacet,
        L::Key => C::Key,
        L::KeyRef => C::KeyRef,
        L::Unique => C::Unique,
        L::Selector => C::Selector,
        L::Field => C::Field,
        L::Assert => C::Assert,
        L::OpenContent => C::OpenContent,
        L::DefaultOpenContent => C::DefaultOpenContent,
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
// ε: F∘G → id_D. On the discrete 18-object label/concept pair both
// transformations reduce to per-object identities; the broader
// `XsdOntology ⊣ English` projection is still an adjunction whose
// round-trip is recovered by the *constant complement* (Bancilhon &
// Spyratos 1981) — the structural XSD ontology data the English
// projection drops. See module docs and
// `formal::meta::categorical_structure` for the architectural
// picture.
//
// Citation:
// - Mac Lane (1998) §IV.1 (adjunction definition).
// - Bancilhon & Spyratos (1981) ACM TODS 6(4) — constant complement.
// - Hofmann, Pierce & Wagner (2011) POPL — explicit-complement
//   symmetric lenses.

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

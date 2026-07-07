//! Functor: SourceTaxonomy → SourceRole.
//!
//! The object action assigns every source *kind* (a leaf or family of the
//! [`SourceTaxonomyConcept`] taxonomy) the [`SourceRoleConcept`] role that
//! names the activity consuming it — a `prov:Role` per PROV-O (Lebo, Sahoo &
//! McGuinness 2013, §3). This is the single authoritative kind→role table:
//! the catalog, the self-model, and `is_chat_loadable` all read it, so the
//! chat-loadable / provisioning-substrate / not-yet-loadable partition is
//! derived once, from the taxonomy, and never re-encoded as an ad-hoc
//! per-name filter.
//!
//! The map is grouped by taxonomy *family* (the `is_a` families of
//! `SourceTaxonomy`), not by source name:
//!
//! - **ChatKnowledge** — the `Lexicon`/`Language` open-class corpus, the
//!   `OntologyVocabulary` (OWL) leaf, and the loadable `LegalCorpus` leaves
//!   with a runtime decoder (`UsCodeTitle` USLM, `LegalLexicon` LMF). These
//!   are `prov:used` by the reasoner.
//! - **DecoderInput** — the `SchemaSpec`, `TestSuite`, `TypographyResource`,
//!   and `ControlledVocabulary` families, plus the closed-class /
//!   inflectional / derivational `Lexicon` substrate that grounds the
//!   language engine. These are `prov:used` by a decoding activity to decode
//!   or ground *other* sources.
//! - **NotYetLoadable** — the PDF-published `LegalCorpus` leaves whose
//!   canonical encoding (`ContentType::Pdf`) has no runtime decoder yet
//!   (`Statute`, `UsFederalStatute`, `Regulation`, `ConstitutionalArticle`,
//!   `ProceduralRule`, `CaseLaw`). Registered and hash-pinned, but consumed
//!   by no activity until a PDF loader lands.
//!
//! Source: Lebo, Sahoo & McGuinness (2013) PROV-O, W3C Recommendation.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::ontology::{
    IsChatLoadable, SourceRoleCategory, SourceRoleConcept, SourceRoleRelation,
    SourceRoleRelationKind,
};
use crate::applied::data_provisioning::ontology::RegistryEntry;
use crate::formal::meta::source_taxonomy::ontology::{
    SourceTaxonomyCategory, SourceTaxonomyConcept, SourceTaxonomyRelationKind,
};
use pr4xis::ontology::Quality;

/// The functor `SourceTaxonomy → SourceRole`: each source kind is assigned
/// the `prov:Role` naming the activity that consumes it.
pub struct SourceKindToRole;

impl Functor for SourceKindToRole {
    type Source = SourceTaxonomyCategory;
    type Target = SourceRoleCategory;

    fn map_object(obj: &SourceTaxonomyConcept) -> SourceRoleConcept {
        use SourceRoleConcept as R;
        use SourceTaxonomyConcept as C;
        match obj {
            // === ChatKnowledge — prov:used by the reasoner ===
            //
            // The open-class natural-language lexicon and its abstract
            // family: WordNet is the reasoner's general-vocabulary anchor.
            C::Lexicon | C::Language | C::DomainLexicon => R::ChatKnowledge,
            // The published OWL vocabularies (SPAR CiTO/DoCO/C4O/BiRO/PROV-O,
            // OLiA) load as ontologies the reasoner queries.
            C::OntologyVocabulary => R::ChatKnowledge,
            // Loadable legal corpus: whole U.S. Code titles (USLM decoder)
            // and the legal lexicon (LMF decoder) are reasoned-over knowledge.
            // The abstract `LegalCorpus` root sits with its loadable leaves.
            C::LegalCorpus | C::UsCodeTitle | C::LegalLexicon => R::ChatKnowledge,

            // === NotYetLoadable — prov:used by no activity yet ===
            //
            // The PDF-published legal leaves: their canonical encoding is
            // `ContentType::Pdf`, for which no runtime decoder exists yet, so
            // no activity can consume them. They re-classify automatically
            // once a PDF loader lands (their encoding gains a decoder).
            C::Statute
            | C::UsFederalStatute
            | C::Regulation
            | C::ConstitutionalArticle
            | C::ProceduralRule
            | C::CaseLaw => R::NotYetLoadable,

            // === DecoderInput — prov:used by a decoding / provisioning activity ===
            //
            // Schema specifications (XSD/DTD/OOXML/conceptual specs) ground
            // the content-type ontologies that decode other documents.
            C::SchemaSpec
            | C::XmlSchemaDefinition
            | C::XmlDocumentTypeDefinition
            | C::OoxmlSchemaArchive
            | C::ConceptualSpec
            // Schema-vocabulary names are decoder substrate, not chat corpus.
            | C::SchemaVocabulary => R::DecoderInput,
            // Conformance test suites certify decoders; consumed by the test
            // harness, never reasoned over.
            C::TestSuite | C::XmlSchemaTestSuite | C::XmlConformanceTestSuite => {
                R::DecoderInput
            }
            // Typographic mapping tables ground the PDF glyph decoder.
            C::TypographyResource | C::TypographicGlyphSet => R::DecoderInput,
            // Controlled vocabularies are consumed by their specific decoders
            // / validators (window-state atoms, the OLiA→CCG projection, the
            // math-operator vocabulary, the colour-scheme collection), not by
            // the chat reasoner.
            C::ControlledVocabulary
            | C::WindowStateVocabulary
            | C::LexicalCategoryProjection
            | C::MathOperatorVocabulary
            | C::ColorSchemeVocabulary => R::DecoderInput,
            // The closed-class / inflectional / derivational lexical substrate
            // IS the language engine (function words, AGID inflections, CatVar
            // derivations) — consumed by the linguistics pipeline, not browsed
            // as knowledge.
            C::ClosedClassLexicon | C::InflectionLexicon | C::DerivationalLexicon => {
                R::DecoderInput
            }

            // The abstract taxonomy root maps to the abstract role root.
            C::Source => R::SourceRole,
        }
    }

    fn map_morphism(m: &<SourceTaxonomyCategory as Category>::Morphism) -> SourceRoleRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity is preserved; every other taxonomy morphism collapses to
        // the most general structural kind (OBO-RO Subsumption) between the
        // mapped roles — the role category is a coarsening of the taxonomy.
        match m.kind() {
            SourceTaxonomyRelationKind::Identity => SourceRoleCategory::identity(&from),
            _ => SourceRoleRelation {
                from,
                to,
                kind: SourceRoleRelationKind::Subsumption,
            },
        }
    }
}

pr4xis::register_functor!(
    SourceKindToRole,
    "Lebo, Sahoo & McGuinness (2013) PROV-O: The PROV Ontology, W3C Recommendation — §3 prov:Role"
);

// ---------------------------------------------------------------------------
// Ontology queries (through the functor — never a hardcoded kind check)
// ---------------------------------------------------------------------------

/// The `prov:Role` of a source *kind* — the functor's object action, exposed
/// as a total function beside [`canonical_encoding`](crate::applied::data_provisioning::ontology::canonical_encoding).
pub fn source_role(kind: SourceTaxonomyConcept) -> SourceRoleConcept {
    SourceKindToRole::map_object(&kind)
}

/// Whether a registry entry is loadable *as chat knowledge* — the composite
/// query `IsChatLoadable ∘ SourceKindToRole` applied to the entry's kind.
///
/// This is the knowledge-boundary predicate the source catalog filters on:
/// only `ChatKnowledge`-role sources are counted toward the "N of M loaded"
/// denominator the chat UI shows. `DecoderInput` and `NotYetLoadable` sources
/// stay first-class in the provisioning registry but are not chat-loadable.
pub fn is_chat_loadable(entry: &RegistryEntry) -> bool {
    IsChatLoadable.get(&source_role(entry.kind)) == Some(true)
}

// ---------------------------------------------------------------------------
// Registry-facing axioms (consult the functor + registry)
// ---------------------------------------------------------------------------

/// Axiom: the role functor is a *total partition* of the registry — every
/// registered source maps to exactly one of the three concrete roles, and the
/// three role-counts sum to the registry size.
///
/// PROV-O (Lebo et al. 2013) §3: an entity has exactly one function w.r.t. a
/// given activity. The partition mirrors the `canonical_encoding` totality:
/// no registered source is left without a role, none straddles two.
pub struct RolePartitionIsTotal;

impl Axiom for RolePartitionIsTotal {
    fn verify(&self) -> Verdict {
        use SourceRoleConcept as R;
        let mut chat = 0usize;
        let mut decoder = 0usize;
        let mut pending = 0usize;
        for entry in crate::applied::data_provisioning::registry::data_sources() {
            match source_role(entry.kind) {
                R::ChatKnowledge => chat += 1,
                R::DecoderInput => decoder += 1,
                R::NotYetLoadable => pending += 1,
                // A registered (leaf) kind must never resolve to the abstract
                // root — that would be an unclassified source.
                R::SourceRole => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
            }
        }
        let total = crate::applied::data_provisioning::registry::data_sources().len();
        if chat + decoder + pending == total {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RolePartitionIsTotal",
        "every registered source maps to exactly one concrete role; the three role-counts sum to the registry size",
        "Lebo, Sahoo & McGuinness (2013) PROV-O: The PROV Ontology, W3C Recommendation — §3 prov:Role"
    );
}

pr4xis::register_axiom!(
    RolePartitionIsTotal,
    "Lebo, Sahoo & McGuinness (2013) PROV-O: The PROV Ontology, W3C Recommendation — §3 prov:Role"
);

/// Axiom: every *registered* source kind resolves to a concrete role (never
/// the abstract `SourceRole` root). Because the registry only ever declares
/// taxonomy *leaves* ([`KindIsTaxonomyLeaf`](crate::applied::data_provisioning::ontology::KindIsTaxonomyLeaf)),
/// this asserts the functor sends every leaf that can be registered to a real
/// role — the object-level counterpart of `RolePartitionIsTotal`.
pub struct EveryRegisteredKindHasConcreteRole;

impl Axiom for EveryRegisteredKindHasConcreteRole {
    fn verify(&self) -> Verdict {
        for entry in crate::applied::data_provisioning::registry::data_sources() {
            if source_role(entry.kind) == SourceRoleConcept::SourceRole {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EveryRegisteredKindHasConcreteRole",
        "every registered source kind maps to a concrete role, not the abstract root",
        "Lebo, Sahoo & McGuinness (2013) PROV-O: The PROV Ontology, W3C Recommendation — §3 prov:Role"
    );
}

pr4xis::register_axiom!(
    EveryRegisteredKindHasConcreteRole,
    "Lebo, Sahoo & McGuinness (2013) PROV-O: The PROV Ontology, W3C Recommendation — §3 prov:Role"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<SourceKindToRole>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn chat_knowledge_kinds_map_to_chat_knowledge() {
        use SourceRoleConcept as R;
        use SourceTaxonomyConcept as C;
        for kind in [
            C::Language,
            C::OntologyVocabulary,
            C::UsCodeTitle,
            C::LegalLexicon,
        ] {
            assert_eq!(source_role(kind), R::ChatKnowledge, "{kind:?}");
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn decoder_input_kinds_map_to_decoder_input() {
        use SourceRoleConcept as R;
        use SourceTaxonomyConcept as C;
        for kind in [
            C::XmlSchemaDefinition,
            C::XmlDocumentTypeDefinition,
            C::TypographicGlyphSet,
            C::ColorSchemeVocabulary,
            C::MathOperatorVocabulary,
            C::ClosedClassLexicon,
            C::InflectionLexicon,
            C::DerivationalLexicon,
            C::XmlConformanceTestSuite,
        ] {
            assert_eq!(source_role(kind), R::DecoderInput, "{kind:?}");
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn pdf_legal_kinds_map_to_not_yet_loadable() {
        use SourceRoleConcept as R;
        use SourceTaxonomyConcept as C;
        for kind in [
            C::Statute,
            C::UsFederalStatute,
            C::Regulation,
            C::ConstitutionalArticle,
            C::ProceduralRule,
            C::CaseLaw,
        ] {
            assert_eq!(source_role(kind), R::NotYetLoadable, "{kind:?}");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_taxonomy_leaf_maps_to_a_concrete_role() {
        use crate::formal::meta::source_taxonomy::ontology::is_leaf;
        use pr4xis::category::FinitelyGenerated;
        for c in SourceTaxonomyConcept::variants() {
            if !is_leaf(c) {
                continue;
            }
            assert_ne!(
                source_role(c),
                SourceRoleConcept::SourceRole,
                "leaf {c:?} must map to a concrete role"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn role_partition_is_total_over_the_registry() {
        RolePartitionIsTotal
            .verify()
            .unwrap_or_else(|c| panic!("{}", c.meta().description.as_str()));
        EveryRegisteredKindHasConcreteRole
            .verify()
            .unwrap_or_else(|c| panic!("{}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_chat_loadable_selects_only_chat_knowledge_entries() {
        // Every registered entry: is_chat_loadable iff its role is ChatKnowledge.
        for entry in crate::applied::data_provisioning::registry::data_sources() {
            let by_role = source_role(entry.kind) == SourceRoleConcept::ChatKnowledge;
            assert_eq!(is_chat_loadable(entry), by_role, "{}", entry.name);
        }
    }
}

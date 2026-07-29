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
            // Loadable legal corpus: whole U.S. Code titles (USLM decoder) are
            // reasoned-over knowledge. The abstract `LegalCorpus` root sits
            // with its loadable leaves.
            C::LegalCorpus | C::UsCodeTitle => R::ChatKnowledge,
            // The Caregiver AI Challenge definitional lexicons (WN-LMF, same
            // LMF decoder as LegalLexicon): cited caregiving / HCBS-compliance
            // glosses the reasoner composes and recites — chat knowledge.
            C::CaregivingLexicon | C::HcbsComplianceLexicon => R::ChatKnowledge,

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
            | C::RealizationFrameTable
            | C::MathOperatorVocabulary
            | C::ColorSchemeVocabulary
            | C::SupertagCostTable
            | C::QuoteGlyphVocabulary
            | C::DashPunctuationVocabulary
            | C::EnglishGraphemeClasses
            | C::CaseFoldingTable
            | C::AssociativeConceptTable
            | C::FrameSemanticTable
            | C::SumoWordNetMappingTable
            | C::GazetteerTable
            // `LegalLexicon` is a statutory RECOGNIZER vocabulary, not a
            // definitional one, and it belongs here for a structural reason
            // anyone can check: `us_legal_lexicon.xml` carries 606 lexical
            // entries sharing just 19 synsets, and those synsets are word
            // CLASSES — `lt-us-state`, `lt-citation-abbr`, `lt-eng-compound`,
            // `lt-named-entity`, even `lt-tokenizer-artifact`. Their glosses
            // read "Month abbreviation per GPO Style Manual §9.20". Compare a
            // real definitional lexicon: caregiving is 173 synsets to 361
            // entries, near one cited gloss per term.
            //
            // Classifying it as ChatKnowledge put it in the loadable-source
            // catalog with no load route, and would have been actively harmful
            // if one were added: `alzheimer`, `parkinson` and `tourette` sit in
            // `lt-named-entity`, so a reasoner composing this could answer
            // "What is Alzheimer's?" with "a named entity" — a class label
            // wearing an answer's clothes, on precisely the terms Track 1
            // leads with. `chat_lexicons`'s own module docs already state this
            // reading (Pustejovsky 1995 domain-scoped qualia; Solan 1993
            // statutory terms of art) and deliberately exclude it from chat
            // composition; this classification now agrees with them.
            | C::LegalLexicon => R::DecoderInput,
            // The closed-class / inflectional / derivational / verb-class /
            // predicate-argument lexical substrate IS the language engine
            // (function words, AGID inflections, CatVar derivations, VerbNet
            // class membership, PropBank roleset membership) — consumed by
            // the linguistics pipeline, not browsed as knowledge.
            C::ClosedClassLexicon
            | C::InflectionLexicon
            | C::DerivationalLexicon
            | C::VerbClassLexicon
            | C::PredicateArgumentLexicon => R::DecoderInput,

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
// Registry-facing axiom (consults the functor + registry)
// ---------------------------------------------------------------------------

/// True iff `kind` resolves to a CONCRETE role — never the abstract `SourceRole`
/// root. The per-entry leg [`EveryRegisteredKindHasConcreteRole`] folds over the
/// registry, factored out so a teeth test can drive it with the mis-mapped
/// abstract-root kind the live registry never declares (every registered kind is
/// a taxonomy leaf).
pub(crate) fn maps_to_concrete_role(kind: SourceTaxonomyConcept) -> bool {
    source_role(kind) != SourceRoleConcept::SourceRole
}

/// Axiom: every *registered* source kind resolves to exactly one CONCRETE role
/// (never the abstract `SourceRole` root). Because the registry only ever
/// declares taxonomy *leaves*
/// ([`KindIsTaxonomyLeaf`](crate::applied::data_provisioning::ontology::KindIsTaxonomyLeaf)),
/// this asserts the functor sends every leaf that can be registered to a real
/// role — no registered source is left unclassified, and, since `source_role` is
/// a *total function* into the three concrete roles (its `match` is exhaustive
/// and only `C::Source` — never registered — hits the abstract root), a
/// concrete role for every entry IS the total partition of the registry (mirrors
/// the [`canonical_encoding`](crate::applied::data_provisioning::ontology::canonical_encoding)
/// totality).
///
/// This is the ONE registry-partition axiom (the former `RolePartitionIsTotal`
/// was logically equivalent — both are `Ok` iff no kind maps to the abstract
/// root — and additionally carried a VACUOUS "the three role-counts sum to the
/// registry size" leg, which a total exhaustive match always satisfies; it was
/// collapsed into this one). Real teeth: it is FALSIFIED by any kind that maps to
/// the abstract root (`maps_to_concrete_role` returns `false` for `C::Source`),
/// proven by `abstract_root_kind_is_rejected`.
pub struct EveryRegisteredKindHasConcreteRole;

impl Axiom for EveryRegisteredKindHasConcreteRole {
    fn verify(&self) -> Verdict {
        for entry in crate::applied::data_provisioning::registry::data_sources() {
            if !maps_to_concrete_role(entry.kind) {
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
        // `LegalLexicon` is deliberately NOT here — it is a recognizer
        // vocabulary (19 word-class synsets over 606 entries), asserted as
        // `DecoderInput` below. It sat in this list while carrying no load
        // route at all, which is how the catalog came to show a source a
        // reader could neither load nor reason with.
        for kind in [C::Language, C::OntologyVocabulary, C::UsCodeTitle] {
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
            C::SupertagCostTable,
            C::ClosedClassLexicon,
            C::InflectionLexicon,
            C::DerivationalLexicon,
            C::PredicateArgumentLexicon,
            // A statutory recognizer vocabulary — surface forms tagged with a
            // word class, consumed by decoding, never recited as an answer.
            C::LegalLexicon,
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
    fn every_registered_kind_has_a_concrete_role_over_the_registry() {
        EveryRegisteredKindHasConcreteRole
            .verify()
            .unwrap_or_else(|c| panic!("{}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn abstract_root_kind_is_rejected() {
        // TEETH for the collapsed partition axiom: the abstract taxonomy root
        // `C::Source` maps to the abstract role root `R::SourceRole`, so it is NOT
        // a concrete role — the ONE kind the axiom must reject. The live registry
        // declares only leaves, so this is the synthetic mis-mapping that proves
        // the axiom's `!maps_to_concrete_role` guard can FAIL, not just pass.
        assert!(
            !maps_to_concrete_role(SourceTaxonomyConcept::Source),
            "the abstract Source root must NOT resolve to a concrete role"
        );
        // Every concrete leaf, by contrast, passes the guard.
        assert!(maps_to_concrete_role(SourceTaxonomyConcept::Language));
        assert!(maps_to_concrete_role(
            SourceTaxonomyConcept::XmlSchemaDefinition
        ));
        assert!(maps_to_concrete_role(SourceTaxonomyConcept::Statute));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn chat_loadable_cardinality_is_eighteen() {
        // The pinned chat-loadable count. Derived through the functor, not
        // hardcoded per name: the ChatKnowledge-role kinds registered in
        // `praxis.toml` are `9 UsCodeTitle + 6 OntologyVocabulary + 1 Language
        // + 1 CaregivingLexicon + 1 HcbsComplianceLexicon`.
        //
        // Was 19, and included `LegalLexicon`. That source carries 606 lexical
        // entries over 19 word-CLASS synsets — a statutory RECOGNIZER
        // vocabulary, not a definitional one — and it reached the catalog as a
        // loadable row with no load route at all. Reclassifying it to
        // `DecoderInput` (the role reserved for sources a decoder uses and the
        // reasoner never queries) dropped the denominator by one. This gate
        // caught that the moment it happened, which is the point of pinning it:
        // a reclassification is a deliberate act, and the build should stop and
        // make someone say so rather than let a catalog quietly change size.
        let loadable = crate::applied::data_provisioning::registry::data_sources()
            .iter()
            .filter(|e| is_chat_loadable(e))
            .count();
        assert_eq!(
            loadable, 18,
            "expected 18 chat-loadable registered sources (the pinned figure)"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_chat_loadable_selects_only_chat_knowledge_entries() {
        // Every registered entry: the catalog-facing `is_chat_loadable` composite
        // agrees with the role partition — is_chat_loadable iff its functor image
        // is ChatKnowledge. This pins the consumer-facing predicate against the
        // live registry through the functor (a broken `is_chat_loadable` — e.g.
        // one that also admitted DecoderInput — would disagree here).
        let mut loadable = 0usize;
        let mut not_loadable = 0usize;
        for entry in crate::applied::data_provisioning::registry::data_sources() {
            let by_role = source_role(entry.kind) == SourceRoleConcept::ChatKnowledge;
            assert_eq!(is_chat_loadable(entry), by_role, "{}", entry.name);
            if by_role {
                loadable += 1;
            } else {
                not_loadable += 1;
            }
        }
        // Non-vacuous: the loadable set is a genuine PROPER, NON-EMPTY subset of
        // the registry — there really are chat-loadable sources AND there really
        // are non-loadable (decoder-input / not-yet-loadable) ones. Were the
        // functor to collapse every kind into one role, this would fail.
        assert!(loadable > 0, "some registered source must be chat-loadable");
        assert!(
            not_loadable > 0,
            "some registered source must NOT be chat-loadable"
        );
    }
}

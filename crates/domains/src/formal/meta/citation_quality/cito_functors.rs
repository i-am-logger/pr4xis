//! CiTO functor web — the interpretation layer over the loaded CiTO
//! (Citation Typing Ontology) vocabulary.
//!
//! The `crate::social::software::markup::xml::owl::loaded_vocabularies`
//! module exposes CiTO as a runtime corpus (`loaded_vocabulary("cito")`), a
//! ~97-object
//! [`OwlVocabularyCategory`](crate::social::software::markup::xml::owl::vocabulary::OwlVocabularyCategory)
//! whose objects are CiTO entities. That category is *loaded*, so a
//! `pr4xis::functor!` cannot source from it without its laws verifying
//! vacuously before `install()` (the law axioms iterate
//! `Source::morphisms()`, which reads the `ACTIVE` singleton). Following
//! the XSD precedent
//! ([`crate::formal::meta::xsd::english_projection`]), the functor web
//! sources from a *finite interpretation enum* —
//! [`CitoCitationType`] — and a separate runtime resolver
//! (`classify_cito_iri`, gated by the `fetch` feature) bridges loaded
//! CiTO IRIs to the enum.
//!
//! [`CitoCitationType`] is **our interpretation layer, not a re-encoding
//! of CiTO**: one variant per CiTO citing-direction `cites`-subproperty
//! that bears on citation *validity* (those mapped in the design tables
//! below). Each variant carries its canonical CiTO IRI, and the
//! corpus-wide audit (`tests::audit_enum_against_loaded_cito`) binds
//! the enum to loaded CiTO: every variant must resolve to a loaded
//! property, and every loaded `cites`-subproperty must be either mapped
//! or on the documented [`OMIT_IRIS`] allow-list — no silent drift.
//!
//! ## The three functors (source = [`CitoCitationTypeCategory`])
//!
//! - [`CitoToEnglish`] — each type to its canonical English phrase.
//! - [`CitoToCitationQuality`] — each type to the validity *dimension*
//!   it bears on. Image ⊆ {`ClaimSupport`, `LocatorAccuracy`}: a CiTO
//!   citation *type* presupposes the cited work exists and says nothing
//!   about the *record's* bibliographic/format accuracy, so Existence /
//!   BibliographicAccuracy / FormatConformance are never in the image
//!   ([`ImageExcludesRecordDimensions`]).
//! - [`CitoToCommunication`] — each type to the Jakobson (1960)
//!   communication function it realises, landing on the
//!   [`CommunicationConcept`] that function focuses
//!   ([`JakobsonFunction::focused_component`]). Stance types →
//!   Emotive (Sender); informational/usage types → Referential
//!   (Context); quotation/excerpt types → Metalingual (Code).
//!
//! ## Adjunction + lens
//!
//! - [`CitoQualityAdjunction`] — forget-citation-type ⊣ free, a
//!   free-forgetful reflection mirroring
//!   [`crate::formal::meta::xsd::english_adjunction`]. The forgetful
//!   right adjoint is [`CitoToCitationQuality`]; the free left adjoint
//!   [`FreeCitoFromQuality`] sends each dimension to its canonical
//!   witness type.
//! - [`CitoTypeQualityLens`] — `CitoCitationType ⇄ CitationQualityConcept`
//!   (its [`CitoToCitationQuality`] projection), a well-behaved lens
//!   (Foster et al. 2007 §2.2).
//!
//! # Literature
//!
//! - **Peroni, S. & Shotton, D. (2012)** "FaBiO and CiTO: ontologies
//!   for describing bibliographic resources and citations", *Journal of
//!   Web Semantics* 17:33–43 — CiTO characterises citations "both
//!   factually and rhetorically" (the factual/rhetorical split this
//!   layer reads).
//! - **Jakobson, R. (1960)** "Linguistics and Poetics", in *Style in
//!   Language*, ed. T. Sebeok, MIT Press, pp. 350–377 — the six
//!   communication functions.
//! - **Gilbert, G. N. (1977)** "Referencing as Persuasion", *Social
//!   Studies of Science* 7(1):113–122 — citation as authorial stance /
//!   rhetorical warrant (the Emotive vs Referential split).
//! - **Teufel, S., Siddharthan, A. & Tidhar, D. (2006)** "Automatic
//!   classification of citation function", *Proceedings of EMNLP 2006*,
//!   pp. 103–110 — citation-function categories (stance / usage /
//!   neutral) grounding the three-bucket partition.
//! - **Mac Lane, S. (1998)** *Categories for the Working Mathematician*,
//!   Springer GTM 5, 2nd ed., §I.3 (functors), §IV.1 (adjunctions),
//!   §IV.4 (reflections).
//! - **Foster, J. N. et al. (2007)** "Combinators for Bidirectional Tree
//!   Transformations", *ACM TOPLAS* 29(3) Art. 17, §2.2 (lens laws).
//! - **Smith, B. et al. (2005)** "Relations in biomedical ontologies",
//!   *Genome Biology* 6:R46 — OBO-RO relation-kind tagging.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{AdjunctionKind, Arrow, Category, Concept, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::ontology::{
    CitationQualityCategory, CitationQualityConcept, CitationQualityRelation,
    CitationQualityRelationKind,
};
use crate::formal::information::communication::ontology::{
    CommunicationCategory, CommunicationConcept, CommunicationRelation, CommunicationRelationKind,
    JakobsonFunction,
};
use crate::formal::meta::lens_composition::lens::Lens;

/// The CiTO namespace prefix for `cito:`-local IRIs.
pub const CITO: &str = "http://purl.org/spar/cito/";
/// The root citing property; every mapped type subsumes under it
/// (`rdfs:subPropertyOf cito:cites`).
pub const CITES_IRI: &str = "http://purl.org/spar/cito/cites";

// =============================================================================
// The interpretation ontology — one variant per mapped CiTO citing type.
// =============================================================================

/// A CiTO citing-direction citation type that bears on citation
/// *validity* — our interpretation layer over loaded CiTO. Each variant
/// is one `cites`-subproperty (Peroni & Shotton 2012); its canonical
/// CiTO IRI is [`CitoCitationType::iri`]. Purely affective / social /
/// production CiTO types (`ridicules`, `plagiarizes`, `credits`,
/// `retracts`, `compiles`, …) bear on no validity dimension and are
/// **not** variants — see [`OMIT_IRIS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum CitoCitationType {
    // ── ClaimSupport, Referential (informational / usage) ──
    CitesAsEvidence,
    CitesAsAuthority,
    CitesAsSourceDocument,
    CitesAsDataSource,
    UsesDataFrom,
    UsesMethodIn,
    UsesConclusionsFrom,
    CitesForInformation,
    Describes,
    Documents,
    Discusses,
    Reviews,
    LinksTo,
    CitesAsRelated,
    CitesAsRecommendedReading,
    Extends,
    Updates,
    ObtainsBackgroundFrom,
    ContainsAssertionFrom,
    CitesAsPotentialSolution,
    // ── ClaimSupport, Emotive (authorial stance) ──
    Supports,
    Confirms,
    AgreesWith,
    Disputes,
    Refutes,
    Corrects,
    Critiques,
    DisagreesWith,
    Qualifies,
    ObtainsSupportFrom,
    RepliesTo,
    SpeculatesOn,
    // ── LocatorAccuracy, Metalingual (reproduces cited code) ──
    IncludesQuotationFrom,
    IncludesExcerptFrom,
}

impl CitoCitationType {
    /// Every interpretation variant, in declaration order.
    pub fn all() -> Vec<Self> {
        Self::variants()
    }

    /// The canonical CiTO IRI for this citation type (Peroni & Shotton
    /// 2012). The identity key bridging the enum to loaded CiTO.
    pub fn iri(&self) -> &'static str {
        use CitoCitationType as T;
        match self {
            T::CitesAsEvidence => "http://purl.org/spar/cito/citesAsEvidence",
            T::CitesAsAuthority => "http://purl.org/spar/cito/citesAsAuthority",
            T::CitesAsSourceDocument => "http://purl.org/spar/cito/citesAsSourceDocument",
            T::CitesAsDataSource => "http://purl.org/spar/cito/citesAsDataSource",
            T::UsesDataFrom => "http://purl.org/spar/cito/usesDataFrom",
            T::UsesMethodIn => "http://purl.org/spar/cito/usesMethodIn",
            T::UsesConclusionsFrom => "http://purl.org/spar/cito/usesConclusionsFrom",
            T::CitesForInformation => "http://purl.org/spar/cito/citesForInformation",
            T::Describes => "http://purl.org/spar/cito/describes",
            T::Documents => "http://purl.org/spar/cito/documents",
            T::Discusses => "http://purl.org/spar/cito/discusses",
            T::Reviews => "http://purl.org/spar/cito/reviews",
            T::LinksTo => "http://purl.org/spar/cito/linksTo",
            T::CitesAsRelated => "http://purl.org/spar/cito/citesAsRelated",
            T::CitesAsRecommendedReading => "http://purl.org/spar/cito/citesAsRecommendedReading",
            T::Extends => "http://purl.org/spar/cito/extends",
            T::Updates => "http://purl.org/spar/cito/updates",
            T::ObtainsBackgroundFrom => "http://purl.org/spar/cito/obtainsBackgroundFrom",
            T::ContainsAssertionFrom => "http://purl.org/spar/cito/containsAssertionFrom",
            T::CitesAsPotentialSolution => "http://purl.org/spar/cito/citesAsPotentialSolution",
            T::Supports => "http://purl.org/spar/cito/supports",
            T::Confirms => "http://purl.org/spar/cito/confirms",
            T::AgreesWith => "http://purl.org/spar/cito/agreesWith",
            T::Disputes => "http://purl.org/spar/cito/disputes",
            T::Refutes => "http://purl.org/spar/cito/refutes",
            T::Corrects => "http://purl.org/spar/cito/corrects",
            T::Critiques => "http://purl.org/spar/cito/critiques",
            T::DisagreesWith => "http://purl.org/spar/cito/disagreesWith",
            T::Qualifies => "http://purl.org/spar/cito/qualifies",
            T::ObtainsSupportFrom => "http://purl.org/spar/cito/obtainsSupportFrom",
            T::RepliesTo => "http://purl.org/spar/cito/repliesTo",
            T::SpeculatesOn => "http://purl.org/spar/cito/speculatesOn",
            T::IncludesQuotationFrom => "http://purl.org/spar/cito/includesQuotationFrom",
            T::IncludesExcerptFrom => "http://purl.org/spar/cito/includesExcerptFrom",
        }
    }

    /// Resolve a CiTO IRI to the interpretation variant it names, or
    /// `None` for an IRI no variant carries.
    pub fn from_iri(iri: &str) -> Option<Self> {
        Self::variants().into_iter().find(|t| t.iri() == iri)
    }
}

/// Loaded CiTO `cites`-subproperties that bear on **no** citation
/// *validity* dimension and are deliberately excluded from
/// [`CitoCitationType`]. Each pair is `(IRI, reason)` — the audit
/// requires every unmapped loaded `cites`-subproperty to appear here, so
/// the OMITs are explicit and documented, never silent.
pub const OMIT_IRIS: &[(&str, &str)] = &[
    (
        "http://purl.org/spar/cito/credits",
        "attribution courtesy ('acknowledges contributions'); bears on no validity dimension",
    ),
    (
        "http://purl.org/spar/cito/retracts",
        "status act ('formal retraction'), not a per-dimension validity assessment",
    ),
    (
        "http://purl.org/spar/cito/compiles",
        "production relation ('used to create or compile'); not citation validity",
    ),
    (
        "http://purl.org/spar/cito/ridicules",
        "purely affective ('ridicules the cited entity'); no validity bearing",
    ),
    (
        "http://purl.org/spar/cito/derides",
        "purely affective ('expresses derision'); no validity bearing",
    ),
    (
        "http://purl.org/spar/cito/parodies",
        "affective/artistic imitation 'for comic effect'; no validity bearing",
    ),
    (
        "http://purl.org/spar/cito/plagiarizes",
        "illicit reuse 'without formal acknowledgement'; no validity bearing",
    ),
    (
        "http://purl.org/spar/cito/citesAsMetadataDocument",
        "metadata-container pointer ('container of metadata describing the citing entity'); \
         about the record's metadata, not a claim-validity dimension",
    ),
    (
        "http://schema.org/citation",
        "schema.org alignment alias of cites; not a CiTO interpretation type",
    ),
];

// =============================================================================
// The interpretation category — a free-forgetful preorder by dimension.
// =============================================================================
//
// Objects are [`CitoCitationType`]s. Morphisms: identities, plus a
// `Generalizes` edge `a → witness(dim(a))` from every type to the
// canonical witness of its CitationQuality dimension. This makes the
// category the reflective shape the forget-free adjunction needs (Mac
// Lane §IV.4): the unit η_a : a → witness(dim(a)) is a real morphism,
// and witnesses are fixed points. The only composable `Generalizes`
// chains end at a witness (which generalises only to itself), so
// composition is closed.

/// Relation-kind tag for [`CitoTypeMorphism`] (Smith et al. 2005
/// OBO-RO): `Identity` only — [`CitoCitationTypeCategory`] is discrete
/// (Mac Lane §I.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CitoTypeRelationKind {
    /// `id_a : a → a` (Mac Lane §I.1).
    Identity,
}

/// A morphism of [`CitoCitationTypeCategory`] — an identity (the category
/// is discrete).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CitoTypeMorphism {
    pub from: CitoCitationType,
    pub to: CitoCitationType,
    pub kind: CitoTypeRelationKind,
}

impl CitoTypeMorphism {
    /// Identity morphism on `t` (Mac Lane §I.1).
    pub fn identity(t: CitoCitationType) -> Self {
        Self {
            from: t,
            to: t,
            kind: CitoTypeRelationKind::Identity,
        }
    }
}

impl Arrow for CitoTypeMorphism {
    type Object = CitoCitationType;
    type Kind = CitoTypeRelationKind;

    fn source(&self) -> CitoCitationType {
        self.from
    }
    fn target(&self) -> CitoCitationType {
        self.to
    }
    fn kind(&self) -> CitoTypeRelationKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new(format!(
                "CitoType-{:?}-{:?}-{:?}",
                self.kind, self.from, self.to
            )),
            description: Label::new(format!(
                "{:?} morphism on CiTO citation types {:?} → {:?}",
                self.kind, self.from, self.to
            )),
            citation: Citation::parse_static(
                "Mac Lane (1998) Categories for the Working Mathematician §I.1 (identities); \
                 Smith et al. (2005) Genome Biology 6:R46 OBO-RO (relation-kind tagging); \
                 Peroni & Shotton (2012) J. Web Semantics 17:33-43",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

/// The canonical witness citation type for a CitationQuality dimension —
/// the representative the free left adjoint picks for that dimension.
/// `ClaimSupport ↦ citesAsEvidence` (the paradigmatic claim-supporting
/// citation, CiTO "source of factual evidence"); `LocatorAccuracy ↦
/// includesQuotationFrom` (the paradigmatic pinpoint-bearing citation).
/// `None` for dimensions outside [`CitoToCitationQuality`]'s image.
pub fn witness_of(dim: CitationQualityConcept) -> Option<CitoCitationType> {
    use CitationQualityConcept as D;
    match dim {
        D::ClaimSupport => Some(CitoCitationType::CitesAsEvidence),
        D::LocatorAccuracy => Some(CitoCitationType::IncludesQuotationFrom),
        D::Existence | D::BibliographicAccuracy | D::FormatConformance | D::CitationQuality => None,
    }
}

/// The interpretation category over [`CitoCitationType`] — **discrete**
/// (objects + identities only; Mac Lane §I.1). Discrete because the
/// citation types carry no inter-type structure *in our interpretation
/// layer* (CiTO's own `subPropertyOf` taxonomy lives in loaded CiTO, the
/// [`OwlVocabularyCategory`](crate::social::software::markup::xml::owl::vocabulary::OwlVocabularyCategory),
/// not here). Being discrete makes the three
/// projection functors out of it structure-preserving by construction —
/// only identities compose. The free-forgetful reflection lives between
/// the witness retract [`CitoWitnessCategory`] and [`ImageDimensionCategory`].
pub struct CitoCitationTypeCategory;

impl Category for CitoCitationTypeCategory {
    type Object = CitoCitationType;
    type Morphism = CitoTypeMorphism;

    fn identity(obj: &CitoCitationType) -> CitoTypeMorphism {
        CitoTypeMorphism::identity(*obj)
    }

    fn compose(f: &CitoTypeMorphism, g: &CitoTypeMorphism) -> Option<CitoTypeMorphism> {
        // Discrete: the only morphisms are identities, so a composable
        // pair is `id_a ∘ id_a = id_a`.
        if f.to != g.from {
            return None;
        }
        Some(*f)
    }

    fn morphisms() -> Vec<CitoTypeMorphism> {
        CitoCitationType::variants()
            .into_iter()
            .map(CitoTypeMorphism::identity)
            .collect()
    }
}

impl pr4xis::category::NamedCategory for CitoCitationTypeCategory {
    fn ontology_name() -> OntologyName {
        OntologyName::new_static("CitoCitationType")
    }
}

// =============================================================================
// Object maps (the design tables, as code).
// =============================================================================

/// CiTO citation type → the CitationQuality validity *dimension* it
/// bears on. Image ⊆ {`ClaimSupport`, `LocatorAccuracy`}. Each row is
/// justified by the CiTO `rdfs:comment` matched against the dimension
/// definition (see [`super::ontology`]).
pub fn map_to_dimension(t: CitoCitationType) -> CitationQualityConcept {
    use CitationQualityConcept as D;
    use CitoCitationType as T;
    match t {
        // Quotation/excerpt: "includes one or more quotations/excerpts"
        // — the distinctive verifiable burden is that the pinpoint
        // resolves to the right place (LocatorAccuracy).
        T::IncludesQuotationFrom | T::IncludesExcerptFrom => D::LocatorAccuracy,
        // Everything else asserts that the cited work backs the citing
        // claim (support / evidence / data / method / stance toward the
        // cited claim) — ClaimSupport.
        T::CitesAsEvidence
        | T::CitesAsAuthority
        | T::CitesAsSourceDocument
        | T::CitesAsDataSource
        | T::UsesDataFrom
        | T::UsesMethodIn
        | T::UsesConclusionsFrom
        | T::CitesForInformation
        | T::Describes
        | T::Documents
        | T::Discusses
        | T::Reviews
        | T::LinksTo
        | T::CitesAsRelated
        | T::CitesAsRecommendedReading
        | T::Extends
        | T::Updates
        | T::ObtainsBackgroundFrom
        | T::ContainsAssertionFrom
        | T::CitesAsPotentialSolution
        | T::Supports
        | T::Confirms
        | T::AgreesWith
        | T::Disputes
        | T::Refutes
        | T::Corrects
        | T::Critiques
        | T::DisagreesWith
        | T::Qualifies
        | T::ObtainsSupportFrom
        | T::RepliesTo
        | T::SpeculatesOn => D::ClaimSupport,
    }
}

/// CiTO citation type → the Jakobson (1960) communication function it
/// realises. Stance types (the citing author's attitude toward the
/// cited claim) → Emotive (Gilbert 1977 referencing-as-persuasion);
/// informational / usage types → Referential (orientation to the
/// referent; Teufel et al. 2006 usage/neutral); quotation/excerpt types
/// → Metalingual (re-presenting the cited code).
pub fn map_to_jakobson(t: CitoCitationType) -> JakobsonFunction {
    use CitoCitationType as T;
    use JakobsonFunction as J;
    match t {
        // Authorial stance toward the cited claim (Jakobson emotive;
        // Gilbert 1977).
        T::Supports
        | T::Confirms
        | T::AgreesWith
        | T::Disputes
        | T::Refutes
        | T::Corrects
        | T::Critiques
        | T::DisagreesWith
        | T::Qualifies
        | T::ObtainsSupportFrom
        | T::RepliesTo
        | T::SpeculatesOn => J::Emotive,
        // Quotation/excerpt re-presents the cited code/utterance and
        // points at it as object (Jakobson metalingual — focus on the
        // code).
        T::IncludesQuotationFrom | T::IncludesExcerptFrom => J::Metalingual,
        // Orientation to the referent — informational / usage citation
        // (Jakobson referential; Teufel et al. 2006).
        T::CitesAsEvidence
        | T::CitesAsAuthority
        | T::CitesAsSourceDocument
        | T::CitesAsDataSource
        | T::UsesDataFrom
        | T::UsesMethodIn
        | T::UsesConclusionsFrom
        | T::CitesForInformation
        | T::Describes
        | T::Documents
        | T::Discusses
        | T::Reviews
        | T::LinksTo
        | T::CitesAsRelated
        | T::CitesAsRecommendedReading
        | T::Extends
        | T::Updates
        | T::ObtainsBackgroundFrom
        | T::ContainsAssertionFrom
        | T::CitesAsPotentialSolution => J::Referential,
    }
}

/// The image of a [`CitoCitationTypeCategory`] morphism under
/// [`CitoToCitationQuality`]. The source is discrete (only identities),
/// so every morphism maps to the identity on the mapped dimension — the
/// `Generalizes` endpoints share a dimension, keeping the map total.
fn dimension_morphism(m: &CitoTypeMorphism) -> CitationQualityRelation {
    let from = map_to_dimension(m.from);
    CitationQualityRelation {
        from,
        to: from,
        kind: CitationQualityRelationKind::Identity,
    }
}

// =============================================================================
// CitoToCitationQuality.
// =============================================================================

pr4xis::functor! {
    name: CitoToCitationQuality,
    source: CitoCitationTypeCategory,
    target: CitationQualityCategory,
    citation: "Peroni & Shotton (2012) J. Web Semantics 17:33-43 (CiTO citation types); \
               Sarol et al. (2024) Bioinformatics 40(7):btae420 (claim-support errors); \
               Mac Lane (1998) Categories for the Working Mathematician §I.3 (functors)",
    map_object: |t: &CitoCitationType| -> CitationQualityConcept { map_to_dimension(*t) },
    map_morphism: |m: &CitoTypeMorphism| -> CitationQualityRelation { dimension_morphism(m) },
}

/// Axiom: [`CitoToCitationQuality`]'s image excludes Existence,
/// BibliographicAccuracy, and FormatConformance. A CiTO citation *type*
/// presupposes the cited work exists and is silent on the *record's*
/// bibliographic/format accuracy — it can only bear on ClaimSupport or
/// LocatorAccuracy.
pub struct ImageExcludesRecordDimensions;

impl Axiom for ImageExcludesRecordDimensions {
    fn verify(&self) -> Verdict {
        use CitationQualityConcept as D;
        use pr4xis::category::Functor;
        for t in CitoCitationType::variants() {
            let dim = CitoToCitationQuality::map_object(&t);
            if matches!(
                dim,
                D::Existence | D::BibliographicAccuracy | D::FormatConformance | D::CitationQuality
            ) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ImageExcludesRecordDimensions",
        "CitoToCitationQuality's image is a subset of {ClaimSupport, LocatorAccuracy}",
        "Peroni & Shotton (2012) J. Web Semantics 17:33-43; Sarol et al. (2024) Bioinformatics 40(7):btae420"
    );
}

pr4xis::register_axiom!(
    ImageExcludesRecordDimensions,
    "Peroni & Shotton (2012) J. Web Semantics 17:33-43"
);

// =============================================================================
// CitoToCommunication.
// =============================================================================

/// The image of a [`CitoCitationTypeCategory`] morphism under
/// [`CitoToCommunication`]. The source is discrete, so every morphism is
/// an identity and maps to the identity on the focused
/// [`CommunicationConcept`] — sidestepping the fact that the
/// Communication category has no generic arrow between two arbitrary
/// Jakobson-focus components.
fn communication_morphism(m: &CitoTypeMorphism) -> CommunicationRelation {
    let from = map_to_jakobson(m.from).focused_component();
    CommunicationRelation {
        from,
        to: from,
        kind: CommunicationRelationKind::Identity,
    }
}

pr4xis::functor! {
    name: CitoToCommunication,
    source: CitoCitationTypeCategory,
    target: CommunicationCategory,
    citation: "Jakobson (1960) Linguistics and Poetics (six communication functions); \
               Gilbert (1977) Referencing as Persuasion, Social Studies of Science 7(1):113-122 \
               (citation as authorial stance); Teufel, Siddharthan & Tidhar (2006) Automatic \
               classification of citation function, EMNLP 2006:103-110 (citation-function classes)",
    map_object: |t: &CitoCitationType| -> CommunicationConcept {
        map_to_jakobson(*t).focused_component()
    },
    map_morphism: |m: &CitoTypeMorphism| -> CommunicationRelation { communication_morphism(m) },
}

// =============================================================================
// CitoToEnglish.
// =============================================================================

/// The English-projection label of a [`CitoCitationType`] — the
/// dimension paired with its canonical English phrase
/// ([`canonical_english_phrase`]). One object per citation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CitoEnglishLabel(pub CitoCitationType);

/// The canonical English phrase for a CiTO citation type — the
/// plain-language reading an explanation surface uses, the CiTO
/// `rdfs:label` (Peroni & Shotton 2012; verified by the runtime audit
/// against loaded CiTO labels).
pub fn canonical_english_phrase(t: CitoCitationType) -> &'static str {
    use CitoCitationType as T;
    match t {
        T::CitesAsEvidence => "cites as evidence",
        T::CitesAsAuthority => "cites as authority",
        T::CitesAsSourceDocument => "cites as source document",
        T::CitesAsDataSource => "cites as data source",
        T::UsesDataFrom => "uses data from",
        T::UsesMethodIn => "uses method in",
        T::UsesConclusionsFrom => "uses conclusions from",
        T::CitesForInformation => "cites for information",
        T::Describes => "describes",
        T::Documents => "documents",
        T::Discusses => "discusses",
        T::Reviews => "reviews",
        T::LinksTo => "links to",
        T::CitesAsRelated => "cites as related",
        T::CitesAsRecommendedReading => "cites as recommended reading",
        T::Extends => "extends",
        T::Updates => "updates",
        T::ObtainsBackgroundFrom => "obtains background from",
        T::ContainsAssertionFrom => "contains assertion from",
        T::CitesAsPotentialSolution => "cites as potential solution",
        T::Supports => "supports",
        T::Confirms => "confirms",
        T::AgreesWith => "agrees with",
        T::Disputes => "disputes",
        T::Refutes => "refutes",
        T::Corrects => "corrects",
        T::Critiques => "critiques",
        T::DisagreesWith => "disagrees with",
        T::Qualifies => "qualifies",
        T::ObtainsSupportFrom => "obtains support from",
        T::RepliesTo => "replies to",
        T::SpeculatesOn => "speculates on",
        T::IncludesQuotationFrom => "includes quotation from",
        T::IncludesExcerptFrom => "includes excerpt from",
    }
}

/// Relation-kind tag for [`CitoEnglishLabelCategory`] — `Identity` only
/// (the label category is discrete, mirroring the discrete source; Mac
/// Lane §I.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CitoEnglishLabelRelationKind {
    Identity,
}

/// Morphism in [`CitoEnglishLabelCategory`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CitoEnglishLabelMorphism {
    pub from: CitoEnglishLabel,
    pub to: CitoEnglishLabel,
    pub kind: CitoEnglishLabelRelationKind,
}

impl CitoEnglishLabelMorphism {
    pub fn identity(l: CitoEnglishLabel) -> Self {
        Self {
            from: l,
            to: l,
            kind: CitoEnglishLabelRelationKind::Identity,
        }
    }
}

impl Arrow for CitoEnglishLabelMorphism {
    type Object = CitoEnglishLabel;
    type Kind = CitoEnglishLabelRelationKind;

    fn source(&self) -> CitoEnglishLabel {
        self.from
    }
    fn target(&self) -> CitoEnglishLabel {
        self.to
    }
    fn kind(&self) -> CitoEnglishLabelRelationKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new(format!(
                "CitoEnglishLabel-{:?}-{:?}-{:?}",
                self.kind, self.from, self.to
            )),
            description: Label::new(format!(
                "{:?} morphism on CiTO English labels {:?} → {:?}",
                self.kind, self.from, self.to
            )),
            citation: Citation::parse_static(
                "Mac Lane (1998) Categories for the Working Mathematician §I.3 (functors); \
                 Peroni & Shotton (2012) J. Web Semantics 17:33-43 (CiTO rdfs:label); \
                 Smith et al. (2005) Genome Biology 6:R46 OBO-RO (relation-kind tagging)",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

impl Concept for CitoEnglishLabel {}
impl FinitelyGenerated for CitoEnglishLabel {
    fn variants() -> Vec<Self> {
        CitoCitationType::variants()
            .into_iter()
            .map(CitoEnglishLabel)
            .collect()
    }
}

/// Category of English-projection labels for CiTO citation types —
/// discrete (one object per citation type, identities only), mirroring
/// the discrete [`CitoCitationTypeCategory`].
pub struct CitoEnglishLabelCategory;

impl Category for CitoEnglishLabelCategory {
    type Object = CitoEnglishLabel;
    type Morphism = CitoEnglishLabelMorphism;

    fn identity(obj: &CitoEnglishLabel) -> CitoEnglishLabelMorphism {
        CitoEnglishLabelMorphism::identity(*obj)
    }

    fn compose(
        f: &CitoEnglishLabelMorphism,
        g: &CitoEnglishLabelMorphism,
    ) -> Option<CitoEnglishLabelMorphism> {
        if f.to != g.from {
            return None;
        }
        Some(*f)
    }

    fn morphisms() -> Vec<CitoEnglishLabelMorphism> {
        CitoEnglishLabel::variants()
            .into_iter()
            .map(CitoEnglishLabelMorphism::identity)
            .collect()
    }
}

impl pr4xis::category::NamedCategory for CitoEnglishLabelCategory {
    fn ontology_name() -> OntologyName {
        OntologyName::new_static("CitoEnglishLabel")
    }
}

/// The English phrase carried by a projected label.
pub fn label_phrase(l: CitoEnglishLabel) -> &'static str {
    canonical_english_phrase(l.0)
}

pr4xis::functor! {
    name: CitoToEnglish,
    source: CitoCitationTypeCategory,
    target: CitoEnglishLabelCategory,
    citation: "Mac Lane (1998) Categories for the Working Mathematician §I.3 (functors); \
               Peroni & Shotton (2012) J. Web Semantics 17:33-43 (CiTO rdfs:label); \
               Spivak (2014) Category Theory for the Sciences §5 (functorial structure preservation)",
    map_object: |t: &CitoCitationType| -> CitoEnglishLabel { CitoEnglishLabel(*t) },
    map_morphism: |_m: &CitoTypeMorphism| -> CitoEnglishLabelMorphism {
        // Discrete source: every morphism is an identity; map it to the
        // identity on the projected label (Mac Lane §I.3).
        CitoEnglishLabelMorphism::identity(CitoEnglishLabel(_m.from))
    },
}

// =============================================================================
// Adjunction: forget-citation-type ⊣ free, on the witness retract.
// =============================================================================
//
// CitoToCitationQuality is many-to-one (34 types → 2 image dimensions),
// so the free-forgetful pair is taken on the *reflective core*: the
// witness types (one per image dimension, [`CitoWitnessType`]) and the
// image dimensions ([`ImageDimension`]). Both are discrete two-object
// categories and the forget/free maps are mutually inverse bijections, so
// the adjunction is an *equivalence* (Mac Lane §IV.4) — the same shape as
// the XSD `XsdEnglishAdjunction`, whose unit/counit reduce to identities.
// This is the honest categorical core of "forget a citation type to its
// dimension, freely reconstruct the canonical witness for a dimension";
// the full 34-type forgetful projection is [`CitoToCitationQuality`].

/// The free left adjoint's witness for a dimension — the canonical CiTO
/// citation type representing an image dimension. Total over the image
/// `{ClaimSupport, LocatorAccuracy}`; for any other dimension there is no
/// witness, so callers use [`CitoWitnessType::for_dimension`] which is
/// `Option`-typed.
pub fn free_witness(dim: CitationQualityConcept) -> CitoCitationType {
    witness_of(dim).unwrap_or(CitoCitationType::CitesAsEvidence)
}

/// A canonical *witness* citation type — the fixed-point retract of
/// [`CitoCitationType`] under the forget-free reflection: exactly one
/// type per image dimension ([`witness_of`]). The adjunction and the
/// well-behaved lens [`CitoTypeQualityLens`] are defined on these (not on
/// all 34 citation types) because the dimension projection is
/// many-to-one; on the witness retract it is a bijection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum CitoWitnessType {
    /// `citesAsEvidence` — the ClaimSupport witness.
    CitesAsEvidence,
    /// `includesQuotationFrom` — the LocatorAccuracy witness.
    IncludesQuotationFrom,
}

impl CitoWitnessType {
    /// The underlying citation type.
    pub fn as_type(self) -> CitoCitationType {
        match self {
            Self::CitesAsEvidence => CitoCitationType::CitesAsEvidence,
            Self::IncludesQuotationFrom => CitoCitationType::IncludesQuotationFrom,
        }
    }
    /// The image dimension this witness bears on.
    pub fn dimension(self) -> CitationQualityConcept {
        map_to_dimension(self.as_type())
    }
    /// The witness for an image dimension, or `None` outside the image.
    pub fn for_dimension(d: CitationQualityConcept) -> Option<Self> {
        match witness_of(d)? {
            CitoCitationType::CitesAsEvidence => Some(Self::CitesAsEvidence),
            CitoCitationType::IncludesQuotationFrom => Some(Self::IncludesQuotationFrom),
            _ => None,
        }
    }
}

/// The two CitationQuality dimensions in [`CitoToCitationQuality`]'s image
/// — the discrete codomain of the forgetful functor's reflective core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum ImageDimension {
    /// `ClaimSupport`.
    ClaimSupport,
    /// `LocatorAccuracy`.
    LocatorAccuracy,
}

impl ImageDimension {
    /// The CitationQuality concept this image dimension is.
    pub fn concept(self) -> CitationQualityConcept {
        match self {
            Self::ClaimSupport => CitationQualityConcept::ClaimSupport,
            Self::LocatorAccuracy => CitationQualityConcept::LocatorAccuracy,
        }
    }
}

/// Discrete category over [`CitoWitnessType`] (objects + identities).
pub struct CitoWitnessCategory;

/// Discrete relation kind — identities only (Mac Lane §I.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscreteKind {
    Identity,
}

/// Identity morphism on a [`CitoWitnessType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CitoWitnessMorphism {
    pub object: CitoWitnessType,
    pub kind: DiscreteKind,
}

impl Arrow for CitoWitnessMorphism {
    type Object = CitoWitnessType;
    type Kind = DiscreteKind;
    fn source(&self) -> CitoWitnessType {
        self.object
    }
    fn target(&self) -> CitoWitnessType {
        self.object
    }
    fn kind(&self) -> DiscreteKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new(format!("CitoWitness-id-{:?}", self.object)),
            description: Label::new("identity on a CiTO witness type"),
            citation: Citation::parse_static(
                "Mac Lane (1998) Categories for the Working Mathematician §I.1 (identities)",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

impl Category for CitoWitnessCategory {
    type Object = CitoWitnessType;
    type Morphism = CitoWitnessMorphism;
    fn identity(obj: &CitoWitnessType) -> CitoWitnessMorphism {
        CitoWitnessMorphism {
            object: *obj,
            kind: DiscreteKind::Identity,
        }
    }
    fn compose(f: &CitoWitnessMorphism, g: &CitoWitnessMorphism) -> Option<CitoWitnessMorphism> {
        (f.object == g.object).then_some(*f)
    }
    fn morphisms() -> Vec<CitoWitnessMorphism> {
        CitoWitnessType::variants()
            .into_iter()
            .map(|o| CitoWitnessMorphism {
                object: o,
                kind: DiscreteKind::Identity,
            })
            .collect()
    }
}

impl pr4xis::category::NamedCategory for CitoWitnessCategory {
    fn ontology_name() -> OntologyName {
        OntologyName::new_static("CitoWitness")
    }
}

/// Discrete category over [`ImageDimension`] (objects + identities).
pub struct ImageDimensionCategory;

/// Identity morphism on an [`ImageDimension`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageDimensionMorphism {
    pub object: ImageDimension,
    pub kind: DiscreteKind,
}

impl Arrow for ImageDimensionMorphism {
    type Object = ImageDimension;
    type Kind = DiscreteKind;
    fn source(&self) -> ImageDimension {
        self.object
    }
    fn target(&self) -> ImageDimension {
        self.object
    }
    fn kind(&self) -> DiscreteKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new(format!("ImageDimension-id-{:?}", self.object)),
            description: Label::new("identity on an image CitationQuality dimension"),
            citation: Citation::parse_static(
                "Mac Lane (1998) Categories for the Working Mathematician §I.1 (identities)",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

impl Category for ImageDimensionCategory {
    type Object = ImageDimension;
    type Morphism = ImageDimensionMorphism;
    fn identity(obj: &ImageDimension) -> ImageDimensionMorphism {
        ImageDimensionMorphism {
            object: *obj,
            kind: DiscreteKind::Identity,
        }
    }
    fn compose(
        f: &ImageDimensionMorphism,
        g: &ImageDimensionMorphism,
    ) -> Option<ImageDimensionMorphism> {
        (f.object == g.object).then_some(*f)
    }
    fn morphisms() -> Vec<ImageDimensionMorphism> {
        ImageDimension::variants()
            .into_iter()
            .map(|o| ImageDimensionMorphism {
                object: o,
                kind: DiscreteKind::Identity,
            })
            .collect()
    }
}

impl pr4xis::category::NamedCategory for ImageDimensionCategory {
    fn ontology_name() -> OntologyName {
        OntologyName::new_static("ImageDimension")
    }
}

/// Map a witness to its image dimension (the forgetful direction).
pub fn witness_to_image(w: CitoWitnessType) -> ImageDimension {
    match w {
        CitoWitnessType::CitesAsEvidence => ImageDimension::ClaimSupport,
        CitoWitnessType::IncludesQuotationFrom => ImageDimension::LocatorAccuracy,
    }
}

/// Map an image dimension to its witness (the free direction).
pub fn image_to_witness(d: ImageDimension) -> CitoWitnessType {
    match d {
        ImageDimension::ClaimSupport => CitoWitnessType::CitesAsEvidence,
        ImageDimension::LocatorAccuracy => CitoWitnessType::IncludesQuotationFrom,
    }
}

pr4xis::functor! {
    name: ForgetCitoType,
    source: CitoWitnessCategory,
    target: ImageDimensionCategory,
    citation: "Peroni & Shotton (2012) J. Web Semantics 17:33-43 (CiTO citation types); \
               Mac Lane (1998) Categories for the Working Mathematician §IV.1 (adjunctions), \
               §IV.4 (equivalence of categories)",
    map_object: |w: &CitoWitnessType| -> ImageDimension { witness_to_image(*w) },
    map_morphism: |m: &CitoWitnessMorphism| -> ImageDimensionMorphism {
        ImageDimensionMorphism {
            object: witness_to_image(m.object),
            kind: DiscreteKind::Identity,
        }
    },
}

pr4xis::functor! {
    name: FreeCitoFromQuality,
    source: ImageDimensionCategory,
    target: CitoWitnessCategory,
    citation: "Mac Lane (1998) Categories for the Working Mathematician §IV.1 (adjunctions), \
               §IV.4 (equivalence of categories); Awodey (2010) Category Theory §9 (adjoint functor theorem)",
    map_object: |d: &ImageDimension| -> CitoWitnessType { image_to_witness(*d) },
    map_morphism: |m: &ImageDimensionMorphism| -> CitoWitnessMorphism {
        CitoWitnessMorphism {
            object: image_to_witness(m.object),
            kind: DiscreteKind::Identity,
        }
    },
}

// `CitoQualityAdjunction` is the forget-citation-type ⊣ free reflection
// (Mac Lane §IV.4) on the reflective core. Left adjoint F =
// `FreeCitoFromQuality` (ImageDimension → CitoWitness) reconstructs the
// canonical witness for a dimension; right adjoint G = `ForgetCitoType`
// (CitoWitness → ImageDimension) forgets a witness to its dimension.
// `image_to_witness` and `witness_to_image` are mutually inverse, so both
// composites are identities and the unit/counit are per-object
// identities (an equivalence; Mac Lane §IV.4).
pr4xis::adjunction! {
    name: CitoQualityAdjunction,
    left: FreeCitoFromQuality,
    right: ForgetCitoType,
    citation: "Mac Lane (1998) Categories for the Working Mathematician §IV.1 (adjunctions), \
               §IV.4 (equivalence of categories); Awodey (2010) Category Theory §9 (adjoint functor theorem); \
               Spivak (2014) Category Theory for the Sciences §6 (adjunctions in data)",
    unit: |d: &ImageDimension| -> ImageDimensionMorphism {
        // η_d : d → G(F(d)) = d (bijection), so η is the identity.
        ImageDimensionMorphism {
            object: *d,
            kind: DiscreteKind::Identity,
        }
    },
    counit: |w: &CitoWitnessType| -> CitoWitnessMorphism {
        // ε_w : F(G(w)) → w = w (bijection), so ε is the identity.
        CitoWitnessMorphism {
            object: *w,
            kind: DiscreteKind::Identity,
        }
    },
}

/// Marker carrying the `FreeForgetful` adjunction-kind classification for
/// [`CitoQualityAdjunction`] (the `adjunction!` macro emits the default
/// `Generic`; this records the reflection's free-forgetful nature, Mac
/// Lane §IV.4). Its `unit`/`counit` delegate to [`CitoQualityAdjunction`].
pub struct CitoQualityAdjunctionKind;

impl pr4xis::category::Adjunction for CitoQualityAdjunctionKind {
    type Left = FreeCitoFromQuality;
    type Right = ForgetCitoType;
    const KIND: AdjunctionKind = AdjunctionKind::FreeForgetful;
    fn unit(obj: &ImageDimension) -> ImageDimensionMorphism {
        <CitoQualityAdjunction as pr4xis::category::Adjunction>::unit(obj)
    }
    fn counit(obj: &CitoWitnessType) -> CitoWitnessMorphism {
        <CitoQualityAdjunction as pr4xis::category::Adjunction>::counit(obj)
    }
    fn meta() -> Provenance {
        <CitoQualityAdjunction as pr4xis::category::Adjunction>::meta()
    }
}

// =============================================================================
// Lens: CitoWitnessType ⇄ CitationQualityConcept (the projection).
// =============================================================================

/// A well-behaved lens `CitoWitnessType ⇄ CitationQualityConcept` — the
/// [`ForgetCitoType`] projection on the witness retract (Foster et al.
/// 2007 §2.2). `get` reads the witness's dimension; `put` selects the
/// witness for the written dimension, falling back to the source witness
/// for an out-of-image dimension (keeping the round-trip total). On the
/// image the projection is a bijection, so GetPut / PutGet / PutPut all
/// hold.
#[derive(Debug, Clone, Copy, Default)]
pub struct CitoTypeQualityLens;

impl Lens for CitoTypeQualityLens {
    type Source = CitoWitnessType;
    type View = CitationQualityConcept;
    type Error = core::convert::Infallible;

    fn get(&self, source: &CitoWitnessType) -> Result<CitationQualityConcept, Self::Error> {
        Ok(source.dimension())
    }

    fn put(
        &self,
        view: &CitationQualityConcept,
        source: &CitoWitnessType,
    ) -> Result<CitoWitnessType, Self::Error> {
        // PutGet: an image dimension selects its witness, whose dimension
        // reads back as the view. GetPut: writing back the source's own
        // dimension yields the same witness. PutPut: `put` ignores the
        // source except as the out-of-image fallback, and an image view
        // fully determines the witness — so put(v2, put(v1, s)) =
        // put(v2, s) (Foster et al. 2007 §2.2).
        Ok(CitoWitnessType::for_dimension(*view).unwrap_or(*source))
    }
}

// =============================================================================
// Runtime resolver — the loaded leg (bridges loaded CiTO IRIs to the enum).
// =============================================================================

#[cfg(all(feature = "fetch", any(test, feature = "codegen")))]
pub use loaded::*;

#[cfg(all(feature = "fetch", any(test, feature = "codegen")))]
mod loaded {
    use super::*;
    use crate::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary;

    /// Resolve a loaded CiTO property IRI to its [`CitoCitationType`]
    /// interpretation, or `None` when the IRI is not a mapped
    /// citing-direction citation type. The loaded leg, mirroring the XSD
    /// `project_name` resolver: confirms the IRI is loaded and subsumes
    /// under `cito:cites` before routing to the enum.
    pub fn classify_cito_iri(iri: &str, vocab: &LoadedOwlVocabulary) -> Option<CitoCitationType> {
        // Must be a loaded property that is_a cito:cites (or is cites
        // itself — not mapped, returns None via from_iri).
        vocab.find(iri)?;
        if iri != CITES_IRI && !vocab.is_a(iri, CITES_IRI) {
            return None;
        }
        CitoCitationType::from_iri(iri)
    }

    /// Every loaded CiTO property IRI that subsumes under `cito:cites`
    /// (the citing-direction citation types — `rdfs:subPropertyOf
    /// cito:cites`, transitively). The denominator of the corpus-wide
    /// OMIT audit.
    pub fn loaded_cites_subproperties(vocab: &LoadedOwlVocabulary) -> Vec<String> {
        vocab
            .properties()
            .into_iter()
            .filter(|iri| vocab.is_a(iri, CITES_IRI))
            .map(|iri| iri.to_string())
            .collect()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Functor;
    use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};
    use proptest::prelude::*;

    // ── Category laws ────────────────────────────────────────────────

    #[test]
    fn interpretation_category_laws() {
        assert_category_laws::<CitoCitationTypeCategory>();
    }

    #[test]
    fn english_label_category_laws() {
        assert_category_laws::<CitoEnglishLabelCategory>();
    }

    // ── Functor laws (non-vacuous — finite enum source) ──────────────

    #[test]
    fn cito_to_english_functor_laws() {
        assert_functor_laws::<CitoToEnglish>();
    }

    #[test]
    fn cito_to_citation_quality_functor_laws() {
        assert_functor_laws::<CitoToCitationQuality>();
    }

    #[test]
    fn cito_to_communication_functor_laws() {
        assert_functor_laws::<CitoToCommunication>();
    }

    #[test]
    fn source_category_is_nonempty() {
        // The whole point of the finite-enum source: laws are real, over
        // 34 objects (not an empty loaded category before install()).
        assert!(CitoCitationType::variants().len() >= 30);
        assert_eq!(
            CitoCitationTypeCategory::morphisms().len(),
            CitoCitationType::variants().len(),
            "discrete: one identity per object"
        );
    }

    // ── Object-map tables ────────────────────────────────────────────

    #[test]
    fn quality_image_is_claim_support_or_locator() {
        use CitationQualityConcept as D;
        for t in CitoCitationType::variants() {
            let d = map_to_dimension(t);
            assert!(
                matches!(d, D::ClaimSupport | D::LocatorAccuracy),
                "{t:?} maps to {d:?}, outside the image"
            );
        }
    }

    #[test]
    fn quotation_types_bear_on_locator() {
        assert_eq!(
            map_to_dimension(CitoCitationType::IncludesQuotationFrom),
            CitationQualityConcept::LocatorAccuracy
        );
        assert_eq!(
            map_to_dimension(CitoCitationType::IncludesExcerptFrom),
            CitationQualityConcept::LocatorAccuracy
        );
    }

    #[test]
    fn evidence_bears_on_claim_support() {
        assert_eq!(
            map_to_dimension(CitoCitationType::CitesAsEvidence),
            CitationQualityConcept::ClaimSupport
        );
    }

    #[test]
    fn image_excludes_record_dimensions_axiom() {
        assert!(ImageExcludesRecordDimensions.verify().is_ok());
    }

    #[test]
    fn jakobson_buckets_only_emotive_referential_metalingual() {
        use JakobsonFunction as J;
        for t in CitoCitationType::variants() {
            let j = map_to_jakobson(t);
            assert!(
                matches!(j, J::Emotive | J::Referential | J::Metalingual),
                "{t:?} maps to {j:?}, outside the grounded three buckets"
            );
        }
    }

    #[test]
    fn communication_object_map_lands_on_focused_component() {
        // Emotive → Sender, Referential → Context, Metalingual → Code.
        assert_eq!(
            CitoToCommunication::map_object(&CitoCitationType::Disputes),
            CommunicationConcept::Sender
        );
        assert_eq!(
            CitoToCommunication::map_object(&CitoCitationType::CitesAsEvidence),
            CommunicationConcept::Context
        );
        assert_eq!(
            CitoToCommunication::map_object(&CitoCitationType::IncludesQuotationFrom),
            CommunicationConcept::Code
        );
    }

    #[test]
    fn english_phrases_nonempty_and_roundtrip() {
        for t in CitoCitationType::variants() {
            assert!(
                !canonical_english_phrase(t).is_empty(),
                "{t:?} has no phrase"
            );
            assert_eq!(
                canonical_english_phrase(t),
                label_phrase(CitoToEnglish::map_object(&t))
            );
        }
    }

    // ── IRI round-trip ───────────────────────────────────────────────

    #[test]
    fn iri_roundtrip_is_injective() {
        for t in CitoCitationType::variants() {
            assert_eq!(CitoCitationType::from_iri(t.iri()), Some(t));
            assert!(t.iri().starts_with(CITO), "{t:?} IRI not in cito namespace");
        }
        // No two variants share an IRI.
        let mut iris: Vec<&str> = CitoCitationType::variants()
            .iter()
            .map(|t| t.iri())
            .collect();
        iris.sort_unstable();
        let n = iris.len();
        iris.dedup();
        assert_eq!(iris.len(), n, "duplicate IRIs across variants");
    }

    #[test]
    fn omit_list_is_disjoint_from_mapped() {
        for (iri, reason) in OMIT_IRIS {
            assert!(!reason.is_empty(), "OMIT {iri} missing reason");
            assert!(
                CitoCitationType::from_iri(iri).is_none(),
                "OMIT {iri} is also a mapped variant"
            );
        }
    }

    // ── Adjunction — triangle identities + naturality (Mac Lane §IV.1) ──

    use pr4xis::category::Adjunction;

    #[test]
    fn free_forget_are_mutually_inverse_bijections() {
        // G∘F = id on ImageDimension and F∘G = id on CitoWitnessType — the
        // reflective core is an equivalence (Mac Lane §IV.4).
        for d in ImageDimension::variants() {
            assert_eq!(witness_to_image(image_to_witness(d)), d);
        }
        for w in CitoWitnessType::variants() {
            assert_eq!(image_to_witness(witness_to_image(w)), w);
        }
    }

    #[test]
    fn r_triangle_identity_holds_per_dimension() {
        // ε_{F(d)} ∘ F(η_d) = id_{F(d)}. With η/ε identities it reduces to
        // an identity composition on CitoWitnessCategory.
        for d in ImageDimension::variants() {
            let eta = CitoQualityAdjunction::unit(&d);
            let f_eta = FreeCitoFromQuality::map_morphism(&eta);
            let fd = FreeCitoFromQuality::map_object(&d);
            let eps_fd = CitoQualityAdjunction::counit(&fd);
            let composed = CitoWitnessCategory::compose(&f_eta, &eps_fd).expect("composable");
            assert_eq!(composed, CitoWitnessCategory::identity(&fd));
        }
    }

    #[test]
    fn l_triangle_identity_holds_per_witness() {
        // G(ε_w) ∘ η_{G(w)} = id_{G(w)}.
        for w in CitoWitnessType::variants() {
            let eps = CitoQualityAdjunction::counit(&w);
            let g_eps = ForgetCitoType::map_morphism(&eps);
            let gw = ForgetCitoType::map_object(&w);
            let eta_gw = CitoQualityAdjunction::unit(&gw);
            let composed = ImageDimensionCategory::compose(&eta_gw, &g_eps).expect("composable");
            assert_eq!(composed, ImageDimensionCategory::identity(&gw));
        }
    }

    #[test]
    fn adjunction_meta_carries_citation() {
        let meta = <CitoQualityAdjunction as Adjunction>::meta();
        assert_eq!(meta.name.as_str(), "CitoQualityAdjunction");
        assert!(meta.citation.as_str().contains("Mac Lane"));
        assert_eq!(
            <CitoQualityAdjunctionKind as Adjunction>::KIND,
            AdjunctionKind::FreeForgetful
        );
    }

    #[test]
    fn witness_categories_laws() {
        assert_category_laws::<CitoWitnessCategory>();
        assert_category_laws::<ImageDimensionCategory>();
    }

    #[test]
    fn free_and_forget_functor_laws() {
        assert_functor_laws::<FreeCitoFromQuality>();
        assert_functor_laws::<ForgetCitoType>();
    }

    // ── Lens laws (Foster et al. 2007 §2.2) ──────────────────────────

    use crate::formal::meta::lens_composition::lens::{
        get_put_holds, put_get_holds, put_put_holds,
    };

    #[test]
    fn lens_well_behaved_on_samples() {
        let r = CitoWitnessType::CitesAsEvidence;
        assert!(get_put_holds(&CitoTypeQualityLens, &r));
        assert!(put_get_holds(
            &CitoTypeQualityLens,
            &CitationQualityConcept::LocatorAccuracy,
            &r
        ));
        assert!(put_put_holds(
            &CitoTypeQualityLens,
            &CitationQualityConcept::LocatorAccuracy,
            &CitationQualityConcept::ClaimSupport,
            &r
        ));
    }

    #[test]
    fn lens_get_is_witness_dimension() {
        assert_eq!(
            CitoTypeQualityLens
                .get(&CitoWitnessType::CitesAsEvidence)
                .unwrap(),
            CitationQualityConcept::ClaimSupport
        );
        assert_eq!(
            CitoTypeQualityLens
                .get(&CitoWitnessType::IncludesQuotationFrom)
                .unwrap(),
            CitationQualityConcept::LocatorAccuracy
        );
    }

    // ── proptest property coverage ───────────────────────────────────

    fn arb_type() -> impl Strategy<Value = CitoCitationType> {
        proptest::sample::select(CitoCitationType::variants())
    }

    fn arb_witness() -> impl Strategy<Value = CitoWitnessType> {
        proptest::sample::select(CitoWitnessType::variants())
    }

    fn arb_image_dim() -> impl Strategy<Value = CitationQualityConcept> {
        prop_oneof![
            Just(CitationQualityConcept::ClaimSupport),
            Just(CitationQualityConcept::LocatorAccuracy),
        ]
    }

    proptest! {
        /// CitoToEnglish object map is injective.
        #[test]
        fn prop_english_object_injective(a in arb_type(), b in arb_type()) {
            prop_assert_eq!(a == b, CitoToEnglish::map_object(&a) == CitoToEnglish::map_object(&b));
        }

        /// CitoToCitationQuality identity preservation (Mac Lane §I.3).
        #[test]
        fn prop_quality_preserves_identity(t in arb_type()) {
            let mapped = CitoToCitationQuality::map_morphism(&CitoCitationTypeCategory::identity(&t));
            let id_tgt = CitationQualityCategory::identity(&CitoToCitationQuality::map_object(&t));
            prop_assert_eq!(mapped, id_tgt);
        }

        /// CitoToCommunication identity preservation.
        #[test]
        fn prop_communication_preserves_identity(t in arb_type()) {
            let mapped = CitoToCommunication::map_morphism(&CitoCitationTypeCategory::identity(&t));
            let id_tgt = CommunicationCategory::identity(&CitoToCommunication::map_object(&t));
            prop_assert_eq!(mapped, id_tgt);
        }

        /// CitoToEnglish composition preservation over the source morphisms.
        #[test]
        fn prop_english_preserves_composition(i in 0usize..512, j in 0usize..512) {
            let ms = CitoCitationTypeCategory::morphisms();
            let f = &ms[i % ms.len()];
            let g = &ms[j % ms.len()];
            if let Some(gf) = CitoCitationTypeCategory::compose(f, g) {
                let mapped = CitoToEnglish::map_morphism(&gf);
                let mf = CitoToEnglish::map_morphism(f);
                let mg = CitoToEnglish::map_morphism(g);
                prop_assert_eq!(Some(mapped), CitoEnglishLabelCategory::compose(&mf, &mg));
            }
        }

        /// Lens GetPut / PutGet / PutPut over the witness retract.
        #[test]
        fn prop_lens_get_put(w in arb_witness()) {
            prop_assert!(get_put_holds(&CitoTypeQualityLens, &w));
        }

        #[test]
        fn prop_lens_put_get(w in arb_witness(), d in arb_image_dim()) {
            prop_assert!(put_get_holds(&CitoTypeQualityLens, &d, &w));
        }

        #[test]
        fn prop_lens_put_put(w in arb_witness(), d1 in arb_image_dim(), d2 in arb_image_dim()) {
            prop_assert!(put_put_holds(&CitoTypeQualityLens, &d1, &d2, &w));
        }
    }

    // ── Corpus-wide audit against loaded CiTO (the bottom-up guardrail) ──

    #[cfg(all(feature = "fetch", feature = "codegen"))]
    mod loaded_audit {
        use super::*;
        use crate::social::software::markup::xml::owl::loaded_vocabularies::loaded_vocabulary;

        /// (a) Every CitoCitationType variant resolves to a loaded CiTO
        /// property (the enum is a faithful index into loaded CiTO), and
        /// (b) every loaded cito:cites-subproperty is either mapped by
        /// classify_cito_iri or on the documented OMIT allow-list — no
        /// silent drops.
        #[test]
        fn audit_enum_against_loaded_cito() {
            let Some(cito) = loaded_vocabulary("cito") else {
                panic!("cito must be a registered, on-disk OntologyVocabulary");
            };

            // (a) Every variant resolves and round-trips through the
            // resolver, with its loaded label matching our phrase.
            for t in CitoCitationType::variants() {
                let idx = cito
                    .find(t.iri())
                    .unwrap_or_else(|| panic!("{t:?} IRI {} not loaded in CiTO", t.iri()));
                assert_eq!(
                    cito.entity(idx).unwrap().kind,
                    crate::social::software::markup::xml::owl::vocabulary::OwlEntityKind::ObjectProperty,
                    "{t:?} must be an ObjectProperty in CiTO"
                );
                assert!(
                    cito.is_a(t.iri(), CITES_IRI),
                    "{t:?} must subsume under cito:cites"
                );
                assert_eq!(
                    classify_cito_iri(t.iri(), cito),
                    Some(t),
                    "resolver must round-trip {t:?}"
                );
                // The canonical phrase equals the loaded CiTO rdfs:label.
                let label = cito.label_of(t.iri()).unwrap_or("");
                assert_eq!(
                    label,
                    canonical_english_phrase(t),
                    "phrase for {t:?} must match loaded CiTO rdfs:label"
                );
            }

            // (b) Every loaded cites-subproperty is mapped or OMITted.
            let omit: alloc::collections::BTreeSet<&str> =
                OMIT_IRIS.iter().map(|(iri, _)| *iri).collect();
            for iri in loaded_cites_subproperties(cito) {
                if iri == CITES_IRI {
                    continue; // the root itself is not a citation *type*.
                }
                let mapped = classify_cito_iri(&iri, cito).is_some();
                let omitted = omit.contains(iri.as_str());
                assert!(
                    mapped ^ omitted,
                    "loaded cites-subproperty {iri} is neither mapped nor on the OMIT allow-list \
                     (or is both) — silent drift"
                );
            }
        }

        /// The resolver rejects non-citing IRIs and non-loaded IRIs.
        #[test]
        fn resolver_rejects_non_citing_and_unknown() {
            let Some(cito) = loaded_vocabulary("cito") else {
                panic!("cito must be registered");
            };
            // isCitedBy is loaded but not a cites-subproperty → None.
            assert_eq!(
                classify_cito_iri("http://purl.org/spar/cito/isCitedBy", cito),
                None
            );
            // Unknown IRI → None.
            assert_eq!(
                classify_cito_iri("http://purl.org/spar/cito/notARealProperty", cito),
                None
            );
        }
    }
}

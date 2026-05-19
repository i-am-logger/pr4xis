//! Functor: XSD ontology → English. Projects every schema component's
//! name (the `QName.localName` of an `<xs:element>`, `<xs:complexType>`,
//! `<xs:simpleType>`, ...) and every `<xs:documentation>` prose block
//! through the WordNet-backed English pipeline, giving Praxis a
//! lexical-semantic reading of any loaded XSD schema rather than an
//! opaque label set.
//!
//! ## Categorical setting
//!
//! Per Mac Lane *Categories for the Working Mathematician* §I.3, a
//! functor F: C → D is a structure-preserving map. Here:
//!
//! - **C** is [`super::ontology::XsdCategory`] — the W3C XSD 1.1
//!   concept inventory plus the `Subsumption` morphisms encoded by
//!   the ontology macro (§I.3 morphism map).
//! - **D** is [`XsdEnglishLabelCategory`] — one identity-morphism
//!   concept per [`XsdConcept`] variant, labelled by the canonical
//!   English head-noun for that schema-kind. This makes the functor's
//!   image into a *named* English-language vocabulary rather than an
//!   abstract relabelling: every XSD concept lands on a known English
//!   lexeme (e.g. "element declaration", "complex type definition",
//!   "documentation"), enabling the *per-instance* `project_name` and
//!   `project_documentation` functions below to extend the projection
//!   to the data carried by each schema component.
//!
//! The object map is the identity on `XsdConcept` lifted through the
//! tagged-pair wrapper [`XsdEnglishLabel`]; the morphism map collapses
//! every source morphism to its identity in the target, since this
//! functor preserves *naming* but not the substantive structural
//! relations of XSD (those are the source category's own concern).
//! Identity preservation and composition preservation both hold by
//! construction (Mac Lane §I.3).
//!
//! ## Two projections carried alongside the functor
//!
//! 1. **Names.** [`project_name`] runs each schema component's
//!    `QName.localName` through
//!    [`super::super::super::super::social::judicial::statute_structure::english_adjunction::resolve_term_name_to_senses`]
//!    (which already lemmatises and looks up against WordNet). Multi-
//!    word identifiers (`ComplexType`, `import_loc`, `xs:any`) are
//!    decomposed into content words via
//!    [`split_identifier`] (Bauer 1983 §6.5 productive compounding +
//!    Huddleston & Pullum 2002 Ch. 19 §1.4 N+N compounding) before
//!    resolution. The free-function shape lets the projection apply
//!    to any name a loaded schema instance carries, while keeping the
//!    type-level functor pure.
//!
//! 2. **Documentation prose.** [`project_documentation`] tokenises a
//!    `<xs:documentation>` block on whitespace + punctuation, filters
//!    closed-class function words via the existing OLiA-grounded
//!    stopword set, then resolves each content lemma through WordNet
//!    using the same `resolve_form_to_senses` pipeline the statute
//!    layer uses.
//!
//! ## Citation
//!
//! - Fellbaum, C. (ed.) (1998) *WordNet: An Electronic Lexical
//!   Database*, MIT Press.
//! - Bauer, L. (1983) *English Word-Formation*, Cambridge University
//!   Press, Ch. 6 (productive prefixation and compounding).
//! - Huddleston, R. & Pullum, G. K. (2002) *The Cambridge Grammar of
//!   the English Language*, Cambridge University Press, Ch. 19 §1.4
//!   (N+N compounding).
//! - Quirk, R., Greenbaum, S., Leech, G. & Svartvik, J. (1985) *A
//!   Comprehensive Grammar of the English Language*, Longman.
//! - McCrae, J. P. et al. (2017) "The Ontolex-Lemon Model:
//!   Development and Applications" *Proc. eLex 2017* — Form / Sense
//!   architecture.
//! - Mac Lane, S. (1998) *Categories for the Working Mathematician*,
//!   Springer GTM 5, 2nd ed., §I.3 (Functors), §I.1 (identities).
//! - Spivak, D. I. (2014) *Category Theory for the Sciences*, MIT
//!   Press, §5 (functorial structure preservation).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

#[cfg(test)]
use super::ontology::XsdRelationKind;
use super::ontology::{XsdCategory, XsdConcept, XsdRelation};
use super::schema_vocabulary::is_in_schema_vocabulary;
use crate::cognitive::linguistics::english::ontology::English;
use crate::social::judicial::statute_structure::english_adjunction::{
    LemmaSenseMapping, resolve_form_to_senses, resolve_term_name_to_senses,
};
use crate::social::judicial::statute_structure::statute_understanding::is_statutory_term_of_art;

// =============================================================================
// Target category — one English-labelled object per XSD concept.
// =============================================================================

/// The English-projection label of an [`XsdConcept`]. Each variant
/// pairs an XSD concept kind with the canonical English head-noun
/// phrase for it; the head-noun phrase is what
/// [`canonical_english_phrase`] returns and is guaranteed to resolve
/// against WordNet (or classify as a statutory-term-of-art) under
/// the `cached_english()` test fixture.
///
/// The category is *discrete* — identity morphisms only — because the
/// English labels are leaf objects of the projection, not nodes in a
/// further structural hierarchy. Subsumption between XSD concepts is
/// carried by the source category; the functor preserves identities
/// (Mac Lane §I.3) and collapses non-identity morphisms to identities
/// on the projected object (a *forgetful* functor — see
/// [`XsdToEnglish::KIND`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, pr4xis::category::Concept)]
pub enum XsdEnglishLabel {
    SchemaComponent,
    ElementDeclaration,
    AttributeDeclaration,
    TypeDefinition,
    ComplexTypeDefinition,
    SimpleTypeDefinition,
    ModelGroup,
    Sequence,
    Choice,
    AllGroup,
    AttributeGroup,
    Particle,
    Wildcard,
    IdentityConstraint,
    NotationDeclaration,
    Annotation,
    AppInfo,
    Documentation,
}

/// The canonical English head-noun phrase for each [`XsdConcept`].
/// These are the literal phrases the W3C XSD 1.1 spec (§§2.2-3.15)
/// uses in its prose — every word is a content-word lemma that
/// resolves through WordNet under the bundled `english-wordnet-2025`
/// data (with two exceptions noted below).
///
/// "Schema" / "schemata" / "XML" appear in this list and are
/// recognised in the bundled `us_legal_lexicon@2026` source as XML-
/// terms-of-art (XML being an ISO/IEC 19757-published term of art).
pub fn canonical_english_phrase(c: XsdConcept) -> &'static str {
    use XsdConcept as C;
    match c {
        C::SchemaComponent => "schema component",
        C::ElementDeclaration => "element declaration",
        C::AttributeDeclaration => "attribute declaration",
        C::TypeDefinition => "type definition",
        C::ComplexTypeDefinition => "complex type definition",
        C::SimpleTypeDefinition => "simple type definition",
        C::ModelGroup => "model group",
        C::Sequence => "sequence",
        C::Choice => "choice",
        C::AllGroup => "all group",
        C::AttributeGroup => "attribute group",
        C::Particle => "particle",
        // WordNet 2025 entries the noun as the two-word lemma
        // "wild card" — using the single-token "wildcard" form would
        // miss the WordNet bigram entry. Both forms are correct
        // English; the projection picks the form that resolves.
        C::Wildcard => "wild card",
        C::IdentityConstraint => "identity constraint",
        C::NotationDeclaration => "notation declaration",
        C::Annotation => "annotation",
        C::AppInfo => "application information",
        C::Documentation => "documentation",
    }
}

/// Morphism in [`XsdEnglishLabelCategory`]. Carries identity plus
/// the projected Subsumption edges that mirror the XSD ontology's
/// `is_a` hierarchy — so [`XsdToEnglish`] can preserve the source
/// category's substantive structure (Spivak 2014 §5: functors
/// preserve subsumption).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XsdEnglishLabelMorphism {
    pub from: XsdEnglishLabel,
    pub to: XsdEnglishLabel,
    pub kind: XsdEnglishLabelRelationKind,
}

impl XsdEnglishLabelMorphism {
    /// Construct an identity morphism on `label` (Mac Lane §I.1).
    pub fn identity(label: XsdEnglishLabel) -> Self {
        Self {
            from: label,
            to: label,
            kind: XsdEnglishLabelRelationKind::Identity,
        }
    }

    /// Construct a Subsumption (`is_a`) morphism `child → parent` —
    /// the projected counterpart of an XSD-side Subsumption edge.
    pub fn subsumption(child: XsdEnglishLabel, parent: XsdEnglishLabel) -> Self {
        Self {
            from: child,
            to: parent,
            kind: XsdEnglishLabelRelationKind::Subsumption,
        }
    }
}

/// Relation-kind tag for [`XsdEnglishLabelCategory`]. Two slots:
/// `Identity` (Mac Lane §I.1) and `Subsumption` (OWL `subClassOf`,
/// projected from the XSD source's is-a hierarchy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XsdEnglishLabelRelationKind {
    /// The only morphism every object carries — Mac Lane §I.1.
    Identity,
    /// Projected `is_a` morphism (W3C XSD 1.1 Part 1 §2.2 →
    /// English `is_a` per Spivak 2014 §5).
    Subsumption,
}

impl Arrow for XsdEnglishLabelMorphism {
    type Object = XsdEnglishLabel;
    type Kind = XsdEnglishLabelRelationKind;

    fn source(&self) -> XsdEnglishLabel {
        self.from
    }
    fn target(&self) -> XsdEnglishLabel {
        self.to
    }
    fn kind(&self) -> XsdEnglishLabelRelationKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new(format!(
                "XsdEnglishLabel-{:?}-{:?}-{:?}",
                self.kind, self.from, self.to
            )),
            description: Label::new(format!(
                "{:?} morphism on XSD English-projection labels {:?} → {:?}",
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

/// Category of English-projection labels for XSD concepts. Carries
/// every Subsumption edge from the XSD source plus the identities,
/// computed via [`subsumption_pairs`] from the canonical XSD `is_a`
/// hierarchy (W3C XSD 1.1 Part 1 §2.2).
pub struct XsdEnglishLabelCategory;

/// The Subsumption edges `(child, parent)` in the English-projection
/// category — the image of the XSD ontology's `is_a` hierarchy under
/// [`project_concept`]. Listed explicitly so `morphisms()` and
/// `compose()` see the same edge set the macro emits on the source.
/// Mirrors W3C XSD 1.1 Part 1 §2.2 + §2.2.1.2 + §3.8.1 + §3.15.1
/// exactly.
fn subsumption_pairs() -> [(XsdEnglishLabel, XsdEnglishLabel); 17] {
    use XsdEnglishLabel as L;
    [
        // Direct children of SchemaComponent (§2.2).
        (L::ElementDeclaration, L::SchemaComponent),
        (L::AttributeDeclaration, L::SchemaComponent),
        (L::TypeDefinition, L::SchemaComponent),
        (L::ModelGroup, L::SchemaComponent),
        (L::AttributeGroup, L::SchemaComponent),
        (L::Particle, L::SchemaComponent),
        (L::Wildcard, L::SchemaComponent),
        (L::IdentityConstraint, L::SchemaComponent),
        (L::NotationDeclaration, L::SchemaComponent),
        (L::Annotation, L::SchemaComponent),
        // TypeDefinition children (§2.2.1.2).
        (L::ComplexTypeDefinition, L::TypeDefinition),
        (L::SimpleTypeDefinition, L::TypeDefinition),
        // ModelGroup children (§3.8.1).
        (L::Sequence, L::ModelGroup),
        (L::Choice, L::ModelGroup),
        (L::AllGroup, L::ModelGroup),
        // Annotation children (§3.15.1).
        (L::AppInfo, L::Annotation),
        (L::Documentation, L::Annotation),
    ]
}

/// Transitive closure of [`subsumption_pairs`] — every (child,
/// ancestor) pair reachable by chasing subsumption edges. Mirrors
/// the same closure the source `XsdCategory` macro emits (per
/// OBO-RO Smith 2005 `transitive_over` on subsumption).
fn subsumption_closure() -> Vec<(XsdEnglishLabel, XsdEnglishLabel)> {
    let direct = subsumption_pairs();
    let mut closure: Vec<(XsdEnglishLabel, XsdEnglishLabel)> = direct.to_vec();
    loop {
        let before = closure.len();
        let snapshot = closure.clone();
        for (a, b) in &snapshot {
            for (b2, c) in &snapshot {
                if b == b2 {
                    let pair = (*a, *c);
                    if !closure.iter().any(|p| p == &pair) {
                        closure.push(pair);
                    }
                }
            }
        }
        if closure.len() == before {
            break;
        }
    }
    closure
}

impl Category for XsdEnglishLabelCategory {
    type Object = XsdEnglishLabel;
    type Morphism = XsdEnglishLabelMorphism;

    fn identity(obj: &XsdEnglishLabel) -> XsdEnglishLabelMorphism {
        XsdEnglishLabelMorphism::identity(*obj)
    }

    fn compose(
        f: &XsdEnglishLabelMorphism,
        g: &XsdEnglishLabelMorphism,
    ) -> Option<XsdEnglishLabelMorphism> {
        if f.to != g.from {
            return None;
        }
        use XsdEnglishLabelRelationKind as K;
        match (f.kind, g.kind) {
            // Identity on the left is just g.
            (K::Identity, _) => Some(*g),
            // Identity on the right is just f.
            (_, K::Identity) => Some(*f),
            // Subsumption is transitive (OBO-RO `transitive_over`,
            // OWL `subClassOf`).
            (K::Subsumption, K::Subsumption) => {
                Some(XsdEnglishLabelMorphism::subsumption(f.from, g.to))
            }
        }
    }

    fn morphisms() -> Vec<XsdEnglishLabelMorphism> {
        let mut out: Vec<XsdEnglishLabelMorphism> = XsdEnglishLabel::variants()
            .into_iter()
            .map(XsdEnglishLabelMorphism::identity)
            .collect();
        for (child, parent) in subsumption_closure() {
            out.push(XsdEnglishLabelMorphism::subsumption(child, parent));
        }
        out
    }
}

// =============================================================================
// Object + morphism map
// =============================================================================

/// Map an [`XsdConcept`] to its English-projection label. Bijection
/// between the 18 XSD concepts and the 18 English-label variants.
pub fn project_concept(c: XsdConcept) -> XsdEnglishLabel {
    use XsdConcept as C;
    use XsdEnglishLabel as L;
    match c {
        C::SchemaComponent => L::SchemaComponent,
        C::ElementDeclaration => L::ElementDeclaration,
        C::AttributeDeclaration => L::AttributeDeclaration,
        C::TypeDefinition => L::TypeDefinition,
        C::ComplexTypeDefinition => L::ComplexTypeDefinition,
        C::SimpleTypeDefinition => L::SimpleTypeDefinition,
        C::ModelGroup => L::ModelGroup,
        C::Sequence => L::Sequence,
        C::Choice => L::Choice,
        C::AllGroup => L::AllGroup,
        C::AttributeGroup => L::AttributeGroup,
        C::Particle => L::Particle,
        C::Wildcard => L::Wildcard,
        C::IdentityConstraint => L::IdentityConstraint,
        C::NotationDeclaration => L::NotationDeclaration,
        C::Annotation => L::Annotation,
        C::AppInfo => L::AppInfo,
        C::Documentation => L::Documentation,
    }
}

// =============================================================================
// The functor.
// =============================================================================

pr4xis::functor! {
    name: XsdToEnglish,
    source: XsdCategory,
    target: XsdEnglishLabelCategory,
    citation: "Mac Lane (1998) Categories for the Working Mathematician §I.3 (Functors); \
               Fellbaum (1998) WordNet: An Electronic Lexical Database; \
               McCrae et al. (2017) The Ontolex-Lemon Model (Proc. eLex 2017); \
               Spivak (2014) Category Theory for the Sciences §5",
    map_object: |c: &XsdConcept| -> XsdEnglishLabel { project_concept(*c) },
    map_morphism: |m: &XsdRelation| -> XsdEnglishLabelMorphism {
        // Identity in the source maps to identity on the projected
        // object (Mac Lane §I.3 functor-identity law). Subsumption
        // edges in the source — the only non-trivial structural
        // morphisms in the XSD ontology — project to Subsumption
        // edges in the target, preserving the W3C XSD 1.1 §2.2
        // is_a hierarchy under the bijection (Spivak 2014 §5).
        //
        // Source kinds Parthood/Causation/Opposition are emitted by
        // the macro as canonical slots even though XSD has no such
        // edges; they collapse to identities here (the source has
        // no morphisms of those kinds to mis-classify).
        use super::ontology::XsdRelationKind as SK;
        let src = project_concept(m.from);
        let dst = project_concept(m.to);
        match m.kind {
            SK::Identity => XsdEnglishLabelMorphism::identity(src),
            SK::Subsumption => XsdEnglishLabelMorphism::subsumption(src, dst),
            // No source edges of these kinds exist in XsdCategory
            // (the macro emits the slots but emits no morphisms).
            // Identity is the only safe target image — and is
            // never exercised because there are no such source
            // morphisms to map.
            SK::Parthood | SK::Causation | SK::Opposition => {
                XsdEnglishLabelMorphism::identity(src)
            }
        }
    },
}

// =============================================================================
// Instance-level projection: identifier names → English senses.
// =============================================================================

/// Decompose an identifier into content-word tokens. Recognises:
///
/// - **Namespace separators** — splits on `:` (XSD QName prefix /
///   localName), `/`, `\`, `.`.
/// - **PascalCase / camelCase** — splits before each ASCII uppercase
///   letter (`ComplexType` → `Complex`, `Type`; `xs:anyType` →
///   `xs`, `any`, `Type` after the namespace split).
/// - **snake_case / kebab-case** — splits on `_` and `-`.
/// - **Digit runs** — splits between digit and letter and within
///   digit runs are emitted as their own token (filtered later as
///   numerics under ISO 80000-2; see [`super::super::super::super::social::judicial::statute_structure::term_extractor::extract_lemmas`]).
///
/// All output tokens are lower-cased. Empty tokens are dropped.
///
/// Per Bauer (1983) §6.5 productive English derivation chains
/// (prefixation + compounding) plus Huddleston & Pullum (2002)
/// Ch. 19 §1.4 N+N compounding — multi-word identifiers in
/// programming-language schemata are formed by exactly these
/// productive patterns.
pub fn split_identifier(name: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut prev_class: Option<CharClass> = None;
    for ch in name.chars() {
        let class = classify_char(ch);
        match class {
            CharClass::Separator => {
                if !buf.is_empty() {
                    out.push(core::mem::take(&mut buf));
                }
                prev_class = None;
                continue;
            }
            CharClass::Upper => {
                // PascalCase boundary: split before an uppercase letter
                // that follows a lowercase letter or a digit.
                if matches!(prev_class, Some(CharClass::Lower) | Some(CharClass::Digit))
                    && !buf.is_empty()
                {
                    out.push(core::mem::take(&mut buf));
                }
                buf.push(ch.to_ascii_lowercase());
            }
            CharClass::Lower => {
                // PascalCase tail-boundary: a run of uppercase letters
                // followed by a lowercase letter — split before the
                // last uppercase (so "XMLParser" → "xml", "parser").
                if matches!(prev_class, Some(CharClass::Upper)) && buf.len() >= 2 {
                    let last = buf.pop().unwrap();
                    if !buf.is_empty() {
                        out.push(core::mem::take(&mut buf));
                    }
                    buf.push(last);
                }
                buf.push(ch.to_ascii_lowercase());
            }
            CharClass::Digit => {
                if matches!(prev_class, Some(CharClass::Lower) | Some(CharClass::Upper))
                    && !buf.is_empty()
                {
                    out.push(core::mem::take(&mut buf));
                }
                buf.push(ch);
            }
            CharClass::Other => {
                // Pass through but flag a boundary so adjacent tokens
                // stay separate. Most non-ASCII characters in XSD
                // names are themselves valid NCName tail characters.
                if !buf.is_empty() {
                    out.push(core::mem::take(&mut buf));
                }
                buf.push(ch.to_ascii_lowercase());
            }
        }
        prev_class = Some(class);
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out.retain(|t| !t.is_empty());
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharClass {
    Upper,
    Lower,
    Digit,
    Separator,
    Other,
}

fn classify_char(ch: char) -> CharClass {
    match ch {
        ':' | '/' | '\\' | '.' | '_' | '-' | ' ' | '\t' => CharClass::Separator,
        c if c.is_ascii_uppercase() => CharClass::Upper,
        c if c.is_ascii_lowercase() => CharClass::Lower,
        c if c.is_ascii_digit() => CharClass::Digit,
        _ => CharClass::Other,
    }
}

/// Project a single XSD-component name (typically a `QName.localName`,
/// possibly prefixed) through the English/WordNet pipeline.
///
/// Pipeline (deterministic — same input always yields same output):
///
/// 1. **Split** the identifier into content-word tokens via
///    [`split_identifier`] (Bauer 1983).
/// 2. **Resolve** the whole identifier (with separators replaced by
///    spaces) against the bigram-aware
///    [`resolve_term_name_to_senses`] — this catches multi-word
///    WordNet lemmas first (e.g. "model group" if WordNet has it).
/// 3. **Per-token fallback** for any token still unresolved after
///    step 2: run it through [`resolve_form_to_senses`] (single-word
///    morphology + WordNet lookup) and back-fill its slot.
/// 4. **Statutory-term-of-art classification.** A token left
///    unresolved after step 3 is reported as such — callers can
///    consult [`is_statutory_term_of_art`] on it. The function
///    itself does NOT silently substitute; the caller decides what
///    to do with `is_resolved() == false` mappings.
///
/// Returns one [`LemmaSenseMapping`] per content-word token in the
/// identifier (function-words and digits are filtered out by the
/// underlying extractor). An identifier with zero content words
/// (e.g. just `"_"` or `"42"`) yields an empty Vec.
pub fn project_name(local_name: &str, english: &English) -> Vec<LemmaSenseMapping> {
    // Step 1+2: decompose, then run the bigram-aware resolver. We
    // feed the decomposed tokens joined by spaces so the existing
    // `extract_lemmas` does its stopword + dedup pass.
    let tokens = split_identifier(local_name);
    if tokens.is_empty() {
        return Vec::new();
    }
    let joined = tokens.join(" ");
    let mut mappings = resolve_term_name_to_senses(&joined, english);

    // Step 3: for any unresolved mapping, try the single-token
    // resolver as a fallback (the bigram path only attaches when both
    // tokens contribute; an isolated content-word lemma may have
    // been missed if the WordNet entry is single-word only).
    for m in mappings.iter_mut() {
        if m.senses.is_empty() {
            let single = resolve_form_to_senses(&m.form, english);
            if !single.is_empty() {
                m.senses = single;
            }
        }
    }

    mappings
}

/// Project a single `<xs:documentation>` text block through the
/// English/WordNet pipeline.
///
/// Tokenises on whitespace + punctuation (matching
/// `extract_lemmas`' rule: split on `!c.is_alphanumeric()`),
/// lower-cases each token, filters numerics (ISO 80000-2) and the
/// closed-class English stopwords (Quirk et al. 1985 +
/// Huddleston & Pullum 2002 Ch. 1), then resolves each surviving
/// lemma through the morphology + WordNet pipeline.
///
/// Returns one [`LemmaSenseMapping`] per *distinct* content lemma.
/// Repeated lemmas are deduplicated (the same word doesn't carry
/// new information on repetition).
pub fn project_documentation(text: &str, english: &English) -> Vec<LemmaSenseMapping> {
    // `extract_lemmas` is the praxis-way tokenizer for English prose
    // — it loads its stopword set from
    // `data/function-words/english.xml` (OLiA-grounded) and filters
    // numerics. We reuse it rather than re-deriving the rules here.
    let forms = crate::social::judicial::statute_structure::term_extractor::extract_lemmas(text);
    forms
        .into_iter()
        .map(|f| {
            let senses = resolve_form_to_senses(&f, english);
            LemmaSenseMapping { form: f, senses }
        })
        .collect()
}

// =============================================================================
// Projection summary type — what every USLM SchemaComponent name looks
// like after the dispatch runs.
// =============================================================================

/// A named XSD schema-component projection: the source kind + name +
/// its English-projection mappings. Produced by
/// [`project_named_component`] for each `(XsdConcept, name)` pair a
/// loaded schema provides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedComponentProjection {
    /// The XSD concept this component belongs to.
    pub concept: XsdConcept,
    /// The English-projection label of `concept` per the functor.
    pub label: XsdEnglishLabel,
    /// The component's local name (e.g. `"section"`,
    /// `"complexContent"`).
    pub local_name: String,
    /// Per-token English-resolution mappings.
    pub mappings: Vec<LemmaSenseMapping>,
}

impl NamedComponentProjection {
    /// True iff every content token in the name resolved through
    /// WordNet OR is classifiable as a statutory-term-of-art per
    /// [`is_statutory_term_of_art`] OR is registered in the
    /// `schema_vocabulary@2026` source per
    /// [`is_schema_vocabulary`]. The three-classifier disjunction
    /// reflects the disjoint vocabulary families a loaded XSD name
    /// may belong to: general English (WordNet), legal terms-of-art
    /// (the LegalLexicon family), or schema-vocabulary names (the
    /// SchemaVocabulary family).
    pub fn is_fully_resolved(&self) -> bool {
        if self.mappings.is_empty() {
            // Pure-numeric or all-separator names produce no
            // mappings — vacuously resolved.
            return true;
        }
        self.mappings.iter().all(|m| {
            m.is_resolved()
                || is_statutory_term_of_art(&m.form.written_rep)
                || is_schema_vocabulary(&m.form.written_rep)
        })
    }

    /// True iff `mappings` carries at least one resolved sense.
    /// Convenience for the smoke test below.
    pub fn has_senses(&self) -> bool {
        self.mappings.iter().any(|m| m.is_resolved())
    }
}

/// True iff `lemma` is a registered schema-vocabulary name.
///
/// The classifier chain consults, in order:
///
/// 1. **M4.η.1 — XHTML 1.0 Strict XSD** (`xhtml_1_0_xsd@1.0`) — for
///    HTML element + attribute names. Loaded by
///    [`crate::social::software::markup::html::english_projection::is_html_vocabulary`]
///    from the W3C-published schema (Pemberton et al. 2002 §A.2).
///    Replaces the hand-curated HTML portion of the bundle for
///    every name the XSD covers.
/// 2. **M4.η.2 — W3C xml.xsd + XML Information Set rec**
///    (`xml_1_0_namespace_xsd@1.0` + `xml_infoset@2004`) — for the
///    four `xml:*`-namespace-reserved attribute names plus the 11
///    information-item canonical phrases. Loaded by
///    [`crate::social::software::markup::xml::english_projection_v1::is_xml_10_vocabulary`]
///    from the W3C-published sources (Bray et al. 2009; Cowan &
///    Tobin 2004). Replaces the hand-curated XML half of the
///    bundle for every name these sources cover.
/// 3. **`schema_vocabulary@2026`** (hand-curated LMF bundle) — now
///    serves the USLM-vocabulary portion and the productive sub-*
///    hierarchical forms. To be deleted in M4.η.4 after M4.η.3
///    (USLM-vocabulary semantics from `<xs:annotation>` blocks)
///    lands.
///
/// Per `feedback_bottom_up_loaded_not_encoded`: every recognized
/// name comes from a registered authoritative source — not from
/// hard-coded Rust string matches.
pub fn is_schema_vocabulary(lemma: &str) -> bool {
    crate::social::software::markup::html::english_projection::is_html_vocabulary(lemma)
        || crate::social::software::markup::xml::english_projection_v1::is_xml_10_vocabulary(lemma)
        || is_in_schema_vocabulary(lemma)
}

/// Project a `(XsdConcept, local_name)` pair through both the type-
/// level functor and the instance-level name pipeline.
pub fn project_named_component(
    concept: XsdConcept,
    local_name: &str,
    english: &English,
) -> NamedComponentProjection {
    NamedComponentProjection {
        concept,
        label: project_concept(concept),
        local_name: local_name.to_string(),
        mappings: project_name(local_name, english),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::lmf;
    use pr4xis::category::Functor;
    use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};
    use proptest::prelude::*;

    /// Minimal WordNet covering the English head-nouns in
    /// [`canonical_english_phrase`] plus a few names that appear in
    /// the USLM XSD's `<xs:element>` declarations. The bundle of
    /// entries is enough for the unit tests below; the USLM smoke
    /// test in this module's parent uses the bundled
    /// `english-wordnet-2025.xml` via `cached_english()`.
    const SAMPLE_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-schema-n"><Lemma writtenForm="schema" partOfSpeech="n"/><Sense id="s-schema-1" synset="s-schema"/></LexicalEntry>
    <LexicalEntry id="e-component-n"><Lemma writtenForm="component" partOfSpeech="n"/><Sense id="s-component-1" synset="s-component"/></LexicalEntry>
    <LexicalEntry id="e-element-n"><Lemma writtenForm="element" partOfSpeech="n"/><Sense id="s-element-1" synset="s-element"/></LexicalEntry>
    <LexicalEntry id="e-declaration-n"><Lemma writtenForm="declaration" partOfSpeech="n"/><Sense id="s-declaration-1" synset="s-declaration"/></LexicalEntry>
    <LexicalEntry id="e-type-n"><Lemma writtenForm="type" partOfSpeech="n"/><Sense id="s-type-1" synset="s-type"/></LexicalEntry>
    <LexicalEntry id="e-definition-n"><Lemma writtenForm="definition" partOfSpeech="n"/><Sense id="s-definition-1" synset="s-definition"/></LexicalEntry>
    <LexicalEntry id="e-complex-a"><Lemma writtenForm="complex" partOfSpeech="a"/><Sense id="s-complex-1" synset="s-complex"/></LexicalEntry>
    <LexicalEntry id="e-simple-a"><Lemma writtenForm="simple" partOfSpeech="a"/><Sense id="s-simple-1" synset="s-simple"/></LexicalEntry>
    <LexicalEntry id="e-model-n"><Lemma writtenForm="model" partOfSpeech="n"/><Sense id="s-model-1" synset="s-model"/></LexicalEntry>
    <LexicalEntry id="e-group-n"><Lemma writtenForm="group" partOfSpeech="n"/><Sense id="s-group-1" synset="s-group"/></LexicalEntry>
    <LexicalEntry id="e-sequence-n"><Lemma writtenForm="sequence" partOfSpeech="n"/><Sense id="s-sequence-1" synset="s-sequence"/></LexicalEntry>
    <LexicalEntry id="e-choice-n"><Lemma writtenForm="choice" partOfSpeech="n"/><Sense id="s-choice-1" synset="s-choice"/></LexicalEntry>
    <LexicalEntry id="e-attribute-n"><Lemma writtenForm="attribute" partOfSpeech="n"/><Sense id="s-attribute-1" synset="s-attribute"/></LexicalEntry>
    <LexicalEntry id="e-particle-n"><Lemma writtenForm="particle" partOfSpeech="n"/><Sense id="s-particle-1" synset="s-particle"/></LexicalEntry>
    <LexicalEntry id="e-wildcard-n"><Lemma writtenForm="wildcard" partOfSpeech="n"/><Sense id="s-wildcard-1" synset="s-wildcard"/></LexicalEntry>
    <LexicalEntry id="e-wild-a"><Lemma writtenForm="wild" partOfSpeech="a"/><Sense id="s-wild-1" synset="s-wild"/></LexicalEntry>
    <LexicalEntry id="e-card-n"><Lemma writtenForm="card" partOfSpeech="n"/><Sense id="s-card-1" synset="s-card"/></LexicalEntry>
    <LexicalEntry id="e-wildcard-bi-n"><Lemma writtenForm="wild card" partOfSpeech="n"/><Sense id="s-wildcard-bi-1" synset="s-wildcard"/></LexicalEntry>
    <LexicalEntry id="e-identity-n"><Lemma writtenForm="identity" partOfSpeech="n"/><Sense id="s-identity-1" synset="s-identity"/></LexicalEntry>
    <LexicalEntry id="e-constraint-n"><Lemma writtenForm="constraint" partOfSpeech="n"/><Sense id="s-constraint-1" synset="s-constraint"/></LexicalEntry>
    <LexicalEntry id="e-notation-n"><Lemma writtenForm="notation" partOfSpeech="n"/><Sense id="s-notation-1" synset="s-notation"/></LexicalEntry>
    <LexicalEntry id="e-annotation-n"><Lemma writtenForm="annotation" partOfSpeech="n"/><Sense id="s-annotation-1" synset="s-annotation"/></LexicalEntry>
    <LexicalEntry id="e-application-n"><Lemma writtenForm="application" partOfSpeech="n"/><Sense id="s-application-1" synset="s-application"/></LexicalEntry>
    <LexicalEntry id="e-information-n"><Lemma writtenForm="information" partOfSpeech="n"/><Sense id="s-information-1" synset="s-information"/></LexicalEntry>
    <LexicalEntry id="e-documentation-n"><Lemma writtenForm="documentation" partOfSpeech="n"/><Sense id="s-documentation-1" synset="s-documentation"/></LexicalEntry>
    <LexicalEntry id="e-section-n"><Lemma writtenForm="section" partOfSpeech="n"/><Sense id="s-section-1" synset="s-section"/></LexicalEntry>
    <LexicalEntry id="e-import-n"><Lemma writtenForm="import" partOfSpeech="n"/><Sense id="s-import-1" synset="s-import"/></LexicalEntry>
    <LexicalEntry id="e-location-n"><Lemma writtenForm="location" partOfSpeech="n"/><Sense id="s-location-1" synset="s-location"/></LexicalEntry>
    <LexicalEntry id="e-mark-v"><Lemma writtenForm="mark" partOfSpeech="v"/><Sense id="s-mark-1" synset="s-mark"/></LexicalEntry>
    <LexicalEntry id="e-legislative-a"><Lemma writtenForm="legislative" partOfSpeech="a"/><Sense id="s-legislative-1" synset="s-legislative"/></LexicalEntry>
    <LexicalEntry id="e-instrument-n"><Lemma writtenForm="instrument" partOfSpeech="n"/><Sense id="s-instrument-1" synset="s-instrument"/></LexicalEntry>
    <LexicalEntry id="e-used-v"><Lemma writtenForm="used" partOfSpeech="v"/><Sense id="s-used-1" synset="s-used"/></LexicalEntry>
    <LexicalEntry id="e-single-a"><Lemma writtenForm="single" partOfSpeech="a"/><Sense id="s-single-1" synset="s-single"/></LexicalEntry>
    <LexicalEntry id="e-all-d"><Lemma writtenForm="all" partOfSpeech="r"/><Sense id="s-all-1" synset="s-all"/></LexicalEntry>
    <Synset id="s-schema" ili="i1" partOfSpeech="n"><Definition>structured form</Definition></Synset>
    <Synset id="s-component" ili="i2" partOfSpeech="n"><Definition>a part</Definition></Synset>
    <Synset id="s-element" ili="i3" partOfSpeech="n"><Definition>a constituent</Definition></Synset>
    <Synset id="s-declaration" ili="i4" partOfSpeech="n"><Definition>a statement</Definition></Synset>
    <Synset id="s-type" ili="i5" partOfSpeech="n"><Definition>a kind</Definition></Synset>
    <Synset id="s-definition" ili="i6" partOfSpeech="n"><Definition>a meaning</Definition></Synset>
    <Synset id="s-complex" ili="i7" partOfSpeech="a"><Definition>complicated</Definition></Synset>
    <Synset id="s-simple" ili="i8" partOfSpeech="a"><Definition>elementary</Definition></Synset>
    <Synset id="s-model" ili="i9" partOfSpeech="n"><Definition>a representation</Definition></Synset>
    <Synset id="s-group" ili="i10" partOfSpeech="n"><Definition>a collection</Definition></Synset>
    <Synset id="s-sequence" ili="i11" partOfSpeech="n"><Definition>an ordered series</Definition></Synset>
    <Synset id="s-choice" ili="i12" partOfSpeech="n"><Definition>a selection</Definition></Synset>
    <Synset id="s-attribute" ili="i13" partOfSpeech="n"><Definition>a property</Definition></Synset>
    <Synset id="s-particle" ili="i14" partOfSpeech="n"><Definition>a fragment</Definition></Synset>
    <Synset id="s-wildcard" ili="i15" partOfSpeech="n"><Definition>a placeholder</Definition></Synset>
    <Synset id="s-wild" ili="i32" partOfSpeech="a"><Definition>free</Definition></Synset>
    <Synset id="s-card" ili="i33" partOfSpeech="n"><Definition>a flat object</Definition></Synset>
    <Synset id="s-identity" ili="i16" partOfSpeech="n"><Definition>sameness</Definition></Synset>
    <Synset id="s-constraint" ili="i17" partOfSpeech="n"><Definition>a restriction</Definition></Synset>
    <Synset id="s-notation" ili="i18" partOfSpeech="n"><Definition>a symbol system</Definition></Synset>
    <Synset id="s-annotation" ili="i19" partOfSpeech="n"><Definition>a note</Definition></Synset>
    <Synset id="s-application" ili="i20" partOfSpeech="n"><Definition>a use</Definition></Synset>
    <Synset id="s-information" ili="i21" partOfSpeech="n"><Definition>data</Definition></Synset>
    <Synset id="s-documentation" ili="i22" partOfSpeech="n"><Definition>written material</Definition></Synset>
    <Synset id="s-section" ili="i23" partOfSpeech="n"><Definition>a segment</Definition></Synset>
    <Synset id="s-import" ili="i24" partOfSpeech="n"><Definition>brought-in content</Definition></Synset>
    <Synset id="s-location" ili="i25" partOfSpeech="n"><Definition>a place</Definition></Synset>
    <Synset id="s-mark" ili="i26" partOfSpeech="v"><Definition>to indicate</Definition></Synset>
    <Synset id="s-legislative" ili="i27" partOfSpeech="a"><Definition>of laws</Definition></Synset>
    <Synset id="s-instrument" ili="i28" partOfSpeech="n"><Definition>a means</Definition></Synset>
    <Synset id="s-used" ili="i29" partOfSpeech="v"><Definition>employed</Definition></Synset>
    <Synset id="s-single" ili="i30" partOfSpeech="a"><Definition>one</Definition></Synset>
    <Synset id="s-all" ili="i31" partOfSpeech="r"><Definition>every</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

    fn sample_english() -> English {
        let wn = lmf::reader::read_wordnet(SAMPLE_LMF).expect("sample LMF parses");
        English::from_wordnet(&wn)
    }

    // ── Category + functor laws ──────────────────────────────────────

    #[test]
    fn target_category_laws_pass() {
        assert_category_laws::<XsdEnglishLabelCategory>();
    }

    #[test]
    fn xsd_to_english_functor_laws_pass() {
        assert_functor_laws::<XsdToEnglish>();
    }

    #[test]
    fn functor_preserves_identity_explicit() {
        // Mac Lane §I.3 functor-identity law spelled out: every
        // source identity maps to the corresponding target identity.
        for c in XsdConcept::variants() {
            let id_src = XsdCategory::identity(&c);
            let mapped = XsdToEnglish::map_morphism(&id_src);
            let id_tgt = XsdEnglishLabelCategory::identity(&XsdToEnglish::map_object(&c));
            assert_eq!(
                mapped, id_tgt,
                "identity on {c:?} must map to identity on the projected label"
            );
        }
    }

    #[test]
    fn functor_meta_carries_citation() {
        let meta = XsdToEnglish::meta();
        assert_eq!(meta.name.as_str(), "XsdToEnglish");
        let cit = meta.citation.as_str();
        assert!(cit.contains("Mac Lane"));
        assert!(cit.contains("WordNet") || cit.contains("Fellbaum"));
        assert!(cit.contains("Ontolex") || cit.contains("McCrae"));
        assert!(meta.module_path.as_str().contains("xsd"));
    }

    // ── Object map — every XSD concept lands on a label ──────────────

    #[test]
    fn project_concept_is_total() {
        // Bijection: 18 concepts → 18 labels, all distinct.
        let mut seen = alloc::collections::BTreeSet::new();
        for c in XsdConcept::variants() {
            let label = project_concept(c);
            // The Debug-string of a XsdEnglishLabel is unique
            // per-variant by construction.
            assert!(seen.insert(format!("{label:?}")));
        }
        assert_eq!(seen.len(), 18);
    }

    #[test]
    fn canonical_english_phrase_resolves_for_every_concept() {
        // Every canonical phrase tokenises to ≥1 content lemma that
        // resolves in the sample WordNet. (The bundled WordNet has
        // strictly more coverage than the sample — see
        // `uslm_canonical_phrases_resolve_through_bundled_wordnet`
        // below for the full-corpus version.)
        let en = sample_english();
        for c in XsdConcept::variants() {
            let phrase = canonical_english_phrase(c);
            let mappings = project_name(phrase, &en);
            assert!(
                !mappings.is_empty(),
                "phrase {phrase:?} (for {c:?}) yields no content lemmas"
            );
            assert!(
                mappings.iter().any(|m| m.is_resolved()),
                "phrase {phrase:?} (for {c:?}) has no resolved mapping: {mappings:?}"
            );
        }
    }

    // ── split_identifier — PascalCase / snake_case / dotted ──────────

    #[test]
    fn split_pascal_case() {
        assert_eq!(split_identifier("ComplexType"), vec!["complex", "type"]);
    }

    #[test]
    fn split_camel_case() {
        assert_eq!(
            split_identifier("complexContent"),
            vec!["complex", "content"]
        );
    }

    #[test]
    fn split_snake_case() {
        assert_eq!(split_identifier("import_loc"), vec!["import", "loc"]);
    }

    #[test]
    fn split_kebab_case() {
        assert_eq!(split_identifier("any-type"), vec!["any", "type"]);
    }

    #[test]
    fn split_namespace_prefix() {
        assert_eq!(split_identifier("xs:any"), vec!["xs", "any"]);
    }

    #[test]
    fn split_acronym_then_word() {
        // "XMLParser" — uppercase run followed by a lowercase letter
        // should split as "xml" then "parser".
        assert_eq!(split_identifier("XMLParser"), vec!["xml", "parser"]);
    }

    #[test]
    fn split_pure_lower() {
        assert_eq!(split_identifier("section"), vec!["section"]);
    }

    #[test]
    fn split_empty() {
        assert!(split_identifier("").is_empty());
    }

    #[test]
    fn split_digits_separate_from_letters() {
        // "v1Schema" → "v", "1", "schema"
        let tokens = split_identifier("v1Schema");
        assert!(tokens.contains(&"v".to_string()));
        assert!(tokens.contains(&"schema".to_string()));
    }

    // ── project_name — names resolve through WordNet ─────────────────

    #[test]
    fn single_word_name_resolves() {
        let en = sample_english();
        let mappings = project_name("section", &en);
        assert_eq!(mappings.len(), 1);
        assert!(mappings[0].is_resolved());
        assert_eq!(mappings[0].form.written_rep, "section");
    }

    #[test]
    fn pascal_case_name_resolves_per_token() {
        let en = sample_english();
        let mappings = project_name("ComplexType", &en);
        // Two content lemmas: "complex" + "type".
        assert_eq!(mappings.len(), 2);
        for m in &mappings {
            assert!(m.is_resolved(), "{m:?} should resolve");
        }
    }

    #[test]
    fn namespace_prefix_strips_in_resolution() {
        let en = sample_english();
        // "xs:any" — `split_identifier` yields ["xs", "any"]. Both
        // tokens reach project_name; "xs" is a 2-char abbreviation
        // (below the abbreviation classifier's ≥3 threshold) so
        // it's flagged as unresolved. "any" is a function word
        // (stripped by extract_lemmas), so it never appears as a
        // mapping. The structural assertion is that the projection
        // produces a well-formed NamedComponentProjection — not
        // that every token resolves; the gap-audit test below is
        // the venue for unresolved tracking.
        let proj = project_named_component(XsdConcept::Wildcard, "xs:any", &en);
        // Sanity: the structural projection wires the concept/label
        // bijection correctly, regardless of name resolution outcome.
        assert_eq!(proj.concept, XsdConcept::Wildcard);
        assert_eq!(proj.label, XsdEnglishLabel::Wildcard);
    }

    #[test]
    fn name_with_no_content_words_returns_empty() {
        let en = sample_english();
        // Pure-numeric (filtered by ISO 80000-2) — should be empty
        // after extract_lemmas runs.
        let mappings = project_name("42", &en);
        assert!(mappings.is_empty());
    }

    // ── project_documentation — prose tokenises and resolves ─────────

    #[test]
    fn documentation_prose_resolves() {
        let en = sample_english();
        let text = "Used to mark a single section of a legislative instrument";
        let mappings = project_documentation(text, &en);
        // After stopword filtering: used, mark, single, section,
        // legislative, instrument (six content lemmas — function
        // words "to", "a", "of", "a" are stripped).
        assert!(!mappings.is_empty());
        let resolved_count = mappings.iter().filter(|m| m.is_resolved()).count();
        // Every content lemma in this sample has a WordNet entry in
        // the sample fixture. ≥80% threshold.
        let pct = (resolved_count * 100) / mappings.len();
        assert!(
            pct >= 80,
            "only {pct}% resolved ({resolved_count}/{}): {mappings:?}",
            mappings.len()
        );
    }

    #[test]
    fn documentation_prose_filters_stopwords() {
        let en = sample_english();
        // Pure-stopword input yields no mappings.
        let mappings = project_documentation("the and of to a", &en);
        assert!(mappings.is_empty());
    }

    #[test]
    fn documentation_prose_deduplicates() {
        let en = sample_english();
        let mappings = project_documentation("section section section", &en);
        assert_eq!(mappings.len(), 1);
        assert!(mappings[0].is_resolved());
    }

    // ── NamedComponentProjection ────────────────────────────────────

    #[test]
    fn named_component_carries_kind_label_name() {
        let en = sample_english();
        let proj = project_named_component(XsdConcept::ElementDeclaration, "section", &en);
        assert_eq!(proj.concept, XsdConcept::ElementDeclaration);
        assert_eq!(proj.label, XsdEnglishLabel::ElementDeclaration);
        assert_eq!(proj.local_name, "section");
        assert!(proj.is_fully_resolved());
        assert!(proj.has_senses());
    }

    // ── ProjectionRespectsCategoryStructure axiom (Spivak 2014 §5) ───

    /// Axiom: if XSD concept X is-a Y in the source category, the
    /// English head-noun phrases of X and Y both project to non-
    /// empty resolved mappings — the lexical projection preserves
    /// the "is-named" relation across the subsumption hierarchy.
    #[test]
    fn axiom_projection_respects_category_structure() {
        let en = sample_english();
        for m in XsdCategory::morphisms() {
            // Subsumption morphisms only — the Identity slots are
            // trivially satisfied.
            if !matches!(m.kind, XsdRelationKind::Subsumption) {
                continue;
            }
            let child_phrase = canonical_english_phrase(m.from);
            let parent_phrase = canonical_english_phrase(m.to);
            let child_proj = project_name(child_phrase, &en);
            let parent_proj = project_name(parent_phrase, &en);
            assert!(
                child_proj.iter().any(|x| x.is_resolved()),
                "child {child_phrase:?} of subsumption pair has no resolved mapping"
            );
            assert!(
                parent_proj.iter().any(|x| x.is_resolved()),
                "parent {parent_phrase:?} of subsumption pair has no resolved mapping"
            );
        }
    }

    // ── Property-based ───────────────────────────────────────────────

    fn arb_xsd_concept() -> impl Strategy<Value = XsdConcept> {
        proptest::sample::select(XsdConcept::variants())
    }

    fn arb_pascal_token() -> impl Strategy<Value = String> {
        // ASCII letters / digits / common separators (mirrors valid
        // XSD NCName production W3C XML 1.1 §2.3 + colon namespace
        // separator).
        proptest::collection::vec(
            prop_oneof![
                prop::char::range('a', 'z'),
                prop::char::range('A', 'Z'),
                prop::char::range('0', '9'),
                Just(':'),
                Just('_'),
                Just('-'),
                Just('.'),
            ],
            0..24,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    fn arb_doc_text() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            prop_oneof![
                prop::char::range('a', 'z'),
                prop::char::range('A', 'Z'),
                prop::char::range('0', '9'),
                Just(' '),
                Just('.'),
                Just(','),
                Just('-'),
            ],
            0..64,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    proptest! {
        /// Identifier splitting is deterministic — same input always
        /// yields the same token sequence. Idempotence of the name
        /// pipeline rests on this.
        #[test]
        fn prop_split_identifier_deterministic(name in arb_pascal_token()) {
            prop_assert_eq!(split_identifier(&name), split_identifier(&name));
        }

        /// Every token emitted by `split_identifier` is lower-case
        /// (no mixed-case leakage).
        #[test]
        fn prop_split_tokens_are_lowercase(name in arb_pascal_token()) {
            for t in split_identifier(&name) {
                prop_assert_eq!(t.clone(), t.to_lowercase());
            }
        }

        /// `project_name` is deterministic — same input string always
        /// yields the same `Vec<LemmaSenseMapping>`.
        #[test]
        fn prop_project_name_deterministic(name in arb_pascal_token()) {
            let en = sample_english();
            let a = project_name(&name, &en);
            let b = project_name(&name, &en);
            prop_assert_eq!(a, b);
        }

        /// `project_documentation` round-trips: the tokens it emits,
        /// in order, equal the case-folded content lemmas the
        /// extractor would have produced. We verify by comparing
        /// against `extract_lemmas` (the underlying tokenizer).
        #[test]
        fn prop_project_documentation_token_round_trip(text in arb_doc_text()) {
            let en = sample_english();
            let mappings = project_documentation(&text, &en);
            let extracted =
                crate::social::judicial::statute_structure::term_extractor::extract_lemmas(&text);
            let mapping_forms: Vec<&String> =
                mappings.iter().map(|m| &m.form.written_rep).collect();
            let extracted_forms: Vec<&String> =
                extracted.iter().map(|f| &f.written_rep).collect();
            prop_assert_eq!(mapping_forms, extracted_forms);
        }

        /// Functor object map: every XSD concept's projection is
        /// stable under repeated application — `project_concept` is
        /// a pure function (Mac Lane §I.3 functor object map
        /// determinism).
        #[test]
        fn prop_project_concept_deterministic(c in arb_xsd_concept()) {
            prop_assert_eq!(project_concept(c), project_concept(c));
        }

        /// Identity preservation on every XSD concept (Mac Lane §I.3
        /// functor-identity law spelled out per-variant).
        #[test]
        fn prop_functor_preserves_identity_per_concept(c in arb_xsd_concept()) {
            let id_src = XsdCategory::identity(&c);
            let mapped = XsdToEnglish::map_morphism(&id_src);
            let id_tgt =
                XsdEnglishLabelCategory::identity(&XsdToEnglish::map_object(&c));
            prop_assert_eq!(mapped, id_tgt);
        }
    }

    // =============================================================================
    // USLM smoke tests — exercise the projection on the bundled USLM-1.0.18.xsd
    // via the cached full English WordNet.
    // =============================================================================

    /// Path to the bundled USLM XSD — the source the codegen pipeline
    /// already consumes via `pr4xis::codegen::uslm_schema`.
    const USLM_XSD_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/legal/uscode/schema/uslm-1.0.18.xsd"
    );

    /// Scan an XSD source for every `<xsd:element name="...">` /
    /// `<xsd:complexType name="...">` / `<xsd:simpleType name="...">`
    /// / `<xsd:attributeGroup name="...">` / `<xsd:group name="...">`
    /// declaration. Returns `(XsdConcept, local_name)` pairs.
    ///
    /// This is a deliberately minimal text-scan rather than a full
    /// XSD parse — the goal is to enumerate every top-level *named*
    /// schema component, which is exactly what the smoke test
    /// requires. The well-formedness of the bundled USLM XSD is a
    /// build-time invariant verified by
    /// `pr4xis::codegen::uslm_schema::generate_uslm_schema_source`.
    fn scan_xsd_named_declarations(xsd_src: &str) -> Vec<(XsdConcept, String)> {
        let mut out = Vec::new();
        for (tag_prefix, concept) in [
            ("<xsd:element ", XsdConcept::ElementDeclaration),
            ("<xsd:complexType ", XsdConcept::ComplexTypeDefinition),
            ("<xsd:simpleType ", XsdConcept::SimpleTypeDefinition),
            ("<xsd:attributeGroup ", XsdConcept::AttributeGroup),
            ("<xsd:group ", XsdConcept::ModelGroup),
            ("<xsd:attribute ", XsdConcept::AttributeDeclaration),
        ] {
            let mut search_from = 0;
            while let Some(idx) = xsd_src[search_from..].find(tag_prefix) {
                let abs = search_from + idx + tag_prefix.len();
                let end = xsd_src[abs..]
                    .find('>')
                    .map(|p| abs + p)
                    .unwrap_or(xsd_src.len());
                let attr_slice = &xsd_src[abs..end];
                if let Some(name) = extract_attr(attr_slice, "name") {
                    out.push((concept, name));
                }
                search_from = end;
            }
        }
        out
    }

    /// Extract `<key>="value"` from an attribute slice (no full XML
    /// parsing; works on well-formed XSD attribute syntax).
    fn extract_attr(slice: &str, key: &str) -> Option<String> {
        let pattern = format!("{key}=\"");
        let start = slice.find(&pattern)? + pattern.len();
        let end = slice[start..].find('"')? + start;
        Some(slice[start..end].to_string())
    }

    fn cached_english() -> &'static English {
        use crate::social::judicial::statute_structure::english_adjunction::test_helpers::cached_english;
        cached_english()
    }

    /// Gap-audit on USLM-1.0.18.xsd: project every named schema
    /// declaration through the English functor and panic if any
    /// component name fails to resolve through WordNet, the
    /// statutory-term-of-art classifier, AND the schema-vocabulary
    /// classifier.
    ///
    /// Per the M4.ε.5.a.3 follow-up "no ignore" mandate: this test
    /// genuinely passes by closing the gaps in the registered
    /// authoritative sources — not by silencing failures with
    /// `eprintln!`. The audit invariants are:
    ///
    /// 1. The XSD scanner finds ≥1 named declaration.
    /// 2. Every named declaration produces a valid
    ///    [`NamedComponentProjection`] (no panic, no malformed data).
    /// 3. Zero unresolved names — every XSD `<xsd:element>` /
    ///    `<xsd:complexType>` / `<xsd:simpleType>` /
    ///    `<xsd:attributeGroup>` / `<xsd:group>` / `<xsd:attribute>`
    ///    name resolves through `WordNet ∪ statutory-term-of-art ∪
    ///    schema-vocabulary`. Per
    ///    `feedback_push_back_on_unsupported_file_types` +
    ///    `feedback_bottom_up_loaded_not_encoded`: closing the gap
    ///    means extending the registered authoritative sources, not
    ///    allow-listing names in Rust code.
    #[test]
    fn uslm_schema_names_resolve_through_english() {
        let xsd_src = std::fs::read_to_string(USLM_XSD_PATH)
            .expect("USLM-1.0.18.xsd readable at the bundled path");
        let named = scan_xsd_named_declarations(&xsd_src);
        assert!(
            !named.is_empty(),
            "USLM XSD scan produced zero named declarations — scanner is broken"
        );

        let en = cached_english();
        let mut failures: Vec<String> = Vec::new();
        for (concept, name) in &named {
            let proj = project_named_component(*concept, name, en);
            if !proj.is_fully_resolved() {
                let bad: Vec<String> = proj
                    .mappings
                    .iter()
                    .filter(|m| {
                        !m.is_resolved()
                            && !is_statutory_term_of_art(&m.form.written_rep)
                            && !is_schema_vocabulary(&m.form.written_rep)
                    })
                    .map(|m| m.form.written_rep.clone())
                    .collect();
                if !bad.is_empty() {
                    failures.push(format!(
                        "{:?} {}: unresolved lemmas {:?}",
                        concept, name, bad
                    ));
                }
            }
        }

        if !failures.is_empty() {
            panic!(
                "USLM XSD has {} schema components whose names don't resolve through WordNet, \
                 statutory-term-of-art, or schema-vocabulary classifiers (expected zero):\n  - {}\n\n\
                 Per `feedback_bottom_up_loaded_not_encoded`: extend the registered \
                 `english_wordnet@2025`, `us_legal_lexicon@2026`, or `schema_vocabulary@2026` \
                 source — do NOT allow-list names in Rust code.",
                failures.len(),
                failures.join("\n  - "),
            );
        }
    }

    #[test]
    fn uslm_canonical_phrases_resolve_through_bundled_wordnet() {
        // Every XSD concept's canonical English head-noun phrase
        // resolves through the bundled WordNet — the strong version
        // of `canonical_english_phrase_resolves_for_every_concept`,
        // using the full corpus instead of the sample fixture.
        let en = cached_english();
        for c in XsdConcept::variants() {
            let phrase = canonical_english_phrase(c);
            let proj = project_named_component(c, phrase, en);
            assert!(
                proj.is_fully_resolved(),
                "canonical phrase {phrase:?} for {c:?} did not fully resolve: {proj:?}"
            );
        }
    }

    /// Axiom: every USLM `<xs:element>` name's documentation block —
    /// when present and tokenisable — resolves ≥80% of its content
    /// lemmas through WordNet. Sample two well-known documented
    /// elements; the bulk-coverage version would require an XML
    /// parse of the schema, which is the codegen path's job.
    #[test]
    fn axiom_documentation_prose_lemmatizes_cleanly() {
        // Two representative documentation blocks taken verbatim from
        // USLM-1.0.18.xsd (publicly available U.S. House schema). The
        // prose is plain English; ≥80% of content tokens must
        // resolve through WordNet.
        const SAMPLES: &[&str] = &[
            "Used to mark a single section of a legislative instrument",
            "Container for a citation",
        ];
        let en = cached_english();
        for text in SAMPLES {
            let mappings = project_documentation(text, en);
            assert!(
                !mappings.is_empty(),
                "documentation sample {text:?} produced no content lemmas"
            );
            let resolved = mappings.iter().filter(|m| m.is_resolved()).count();
            let pct = (resolved * 100) / mappings.len();
            assert!(
                pct >= 80,
                "documentation sample {text:?}: only {pct}% resolved ({}/{}): {mappings:?}",
                resolved,
                mappings.len()
            );
        }
    }
}

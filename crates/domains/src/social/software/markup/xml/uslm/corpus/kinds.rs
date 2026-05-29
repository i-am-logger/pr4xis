//! USLM dispatch enums — `ContainerKind`, `SubdivisionKind`,
//! `UsCodeAdditionalContainer`, and the various heading / formula /
//! form / amendment / inline / quoted-variant tag families.
//!
//! All XSD-grounded dispatch goes through these enums. The
//! `from_xsd_element` constructors confirm an element is declared by
//! the loaded USLM XSD ontology (and, where applicable, is a
//! `substitutionGroup="level"` member) before projecting to the
//! runtime variant. Per W3C XSD 1.1 Part 1 §3.3 (Element
//! Declarations) and §3.3.6 (Substitution Groups).

#[allow(unused_imports)]
use alloc::{string::String, vec::Vec};

use crate::formal::meta::xsd::from_xsd_parser::XsdOntologyInstance;

/// USLM hierarchy kinds between Title and Section.
///
/// Listed in canonical USLM nesting order. Different titles use
/// different combinations: Title 18 uses Part > Chapter > Section;
/// Title 49 uses Subtitle > Part > Chapter > Subchapter > Section.
/// The schema doesn't enforce a strict order so any subset can
/// appear in any title.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerKind {
    /// `<subtitle>` — used in long titles (Title 49, Title 26).
    Subtitle,
    /// `<part>` — Roman-numeral subdivisions.
    Part,
    /// `<subpart>` — sub-Roman subdivisions within Parts.
    Subpart,
    /// `<chapter>` — the most common mid-level container.
    Chapter,
    /// `<subchapter>` — sub-Arabic subdivisions within Chapters.
    Subchapter,
}

impl ContainerKind {
    /// Parse a USLM container element tag name. Returns `None`
    /// for non-container tags.
    ///
    /// **Prefer [`ContainerKind::from_xsd_element`]** — the
    /// XSD-grounded path that confirms the element is a
    /// substitutionGroup="level" member in the loaded USLM XSD
    /// ontology before accepting the name. This unguarded parse
    /// remains for backwards compatibility while the M4.ε.5.a.5
    /// consumer migration proceeds; downstream code should thread the
    /// XSD instance and call `from_xsd_element` instead.
    pub fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "subtitle" => Self::Subtitle,
            "part" => Self::Part,
            "subpart" => Self::Subpart,
            "chapter" => Self::Chapter,
            "subchapter" => Self::Subchapter,
            _ => return None,
        })
    }

    /// XSD-grounded variant of [`ContainerKind::parse`]. Confirms that
    /// `tag` is declared by the loaded USLM XSD ontology AND is a
    /// member of the `substitutionGroup="level"` family before
    /// projecting to the runtime enum variant.
    ///
    /// Per W3C XSD 1.1 Part 1 §3.3.6 (Substitution Groups), the
    /// `substitutionGroup` head `level` collects every USLM
    /// hierarchical level element (subtitle, part, subpart, chapter,
    /// subchapter, section, subsection, paragraph, …) into one
    /// reflexive-transitive membership predicate. ContainerKind
    /// variants are exactly the proper-container subset — level
    /// members that wrap nested levels rather than carrying
    /// subdivision content (the `section` leaf is split off into the
    /// section-leaf branch by the lens walker).
    ///
    /// Per W3C XSD 1.1 Part 1 §3.3 (Element Declarations), an element
    /// name with no matching `<xsd:element>` declaration in the loaded
    /// XSD has no `{type definition}` to dispatch on; this function
    /// returns `None` for such names rather than guessing.
    ///
    /// Citation: LRC USLM XML User Guide § V (level hierarchy).
    pub fn from_xsd_element(tag: &str, xsd: &XsdOntologyInstance) -> Option<Self> {
        // Query 1 — W3C XSD 1.1 Part 1 §3.3 Element Declarations: is
        // `tag` declared by the loaded XSD?
        xsd.lookup_element(tag)?;
        // Query 2 — W3C XSD 1.1 Part 1 §3.3.6 Substitution Groups: is
        // `tag` a (reflexive-transitive) member of the `"level"` head?
        if !xsd.is_member_of_substitution_group(tag, "level") {
            return None;
        }
        // Projection from the XSD-grounded name set to the runtime
        // enum variant. The set of accepted names is the intersection
        // of ContainerKind::all() with the loaded XSD's level-group
        // members — i.e. the proper-container subset, excluding the
        // section leaf and below.
        Self::parse(tag)
    }

    /// Enumerate every ContainerKind variant the loaded USLM XSD
    /// admits — projected from the
    /// `substitutionGroup="level"` family (W3C XSD 1.1 Part 1
    /// §3.3.6) via [`Self::from_xsd_element`]. The returned set is
    /// derived from the loaded schema, not a hand-curated Rust
    /// slice, per `feedback_bottom_up_loaded_not_encoded`.
    ///
    /// Order matches the XSD's declaration order with `level`
    /// itself elided (the head is the group, not a container
    /// instance). Non-container level-group members (the section
    /// leaf and the [`SubdivisionKind`] / heading siblings) are
    /// filtered out by [`Self::from_xsd_element`]'s name-to-variant
    /// projection.
    pub fn load_from_xsd(xsd: &XsdOntologyInstance) -> Vec<Self> {
        xsd.substitution_group_members("level")
            .into_iter()
            .filter_map(|name| Self::from_xsd_element(name, xsd))
            .collect()
    }

    /// Canonical USLM tag name.
    pub fn tag(self) -> &'static str {
        match self {
            Self::Subtitle => "subtitle",
            Self::Part => "part",
            Self::Subpart => "subpart",
            Self::Chapter => "chapter",
            Self::Subchapter => "subchapter",
        }
    }

    /// Canonical USLM nesting depth: Subtitle (0) → Part (1) →
    /// Subpart (2) → Chapter (3) → Subchapter (4). Used to assert
    /// strict-nesting invariants in tests.
    pub fn nesting_depth(self) -> usize {
        match self {
            Self::Subtitle => 0,
            Self::Part => 1,
            Self::Subpart => 2,
            Self::Chapter => 3,
            Self::Subchapter => 4,
        }
    }
}

/// USLM hierarchy levels below a §, per the published schema.
///
/// USLM defines a strict nesting order: Subsection ⊐ Paragraph ⊐
/// Subparagraph ⊐ Clause ⊐ Subclause ⊐ Item ⊐ Subitem. Each level
/// uses a different numbering convention (a/b/c, 1/2/3, A/B/C,
/// i/ii/iii, …) that Bluebook §3.3 formalizes for citation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubdivisionKind {
    Subsection,
    Paragraph,
    Subparagraph,
    Clause,
    Subclause,
    Item,
    Subitem,
}

impl SubdivisionKind {
    /// Parse a USLM container element tag name into the typed
    /// variant. Returns `None` for unknown element names — callers
    /// should treat that as a structural anomaly to flag, not
    /// silently absorb.
    ///
    /// **Prefer [`SubdivisionKind::from_xsd_element`]** — the
    /// XSD-grounded path that confirms membership in the loaded USLM
    /// XSD's `substitutionGroup="level"` family before projecting.
    pub fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "subsection" => Self::Subsection,
            "paragraph" => Self::Paragraph,
            "subparagraph" => Self::Subparagraph,
            "clause" => Self::Clause,
            "subclause" => Self::Subclause,
            "item" => Self::Item,
            "subitem" => Self::Subitem,
            _ => return None,
        })
    }

    /// XSD-grounded variant of [`SubdivisionKind::parse`]. Confirms
    /// that `tag` is declared by the loaded USLM XSD ontology AND is
    /// a member of the `substitutionGroup="level"` family before
    /// projecting to the runtime variant.
    ///
    /// Per W3C XSD 1.1 Part 1 §3.3.6 (Substitution Groups), every
    /// USLM hierarchical level element shares the `"level"` head;
    /// subdivision kinds (subsection / paragraph / subparagraph /
    /// clause / subclause / item / subitem) are the level-group
    /// members below the section leaf. The dispatch consults the
    /// XSD's loaded knowledge first; the projection to the runtime
    /// enum variant is the second step (W3C XSD 1.1 Part 1 §3.3
    /// Element Declarations).
    ///
    /// Citation: LRC USLM XML User Guide § V (level hierarchy).
    pub fn from_xsd_element(tag: &str, xsd: &XsdOntologyInstance) -> Option<Self> {
        xsd.lookup_element(tag)?;
        if !xsd.is_member_of_substitution_group(tag, "level") {
            return None;
        }
        Self::parse(tag)
    }

    /// Enumerate every SubdivisionKind variant the loaded USLM XSD
    /// admits — projected from the `substitutionGroup="level"`
    /// family (W3C XSD 1.1 Part 1 §3.3.6) via
    /// [`Self::from_xsd_element`]. Mirror of
    /// [`ContainerKind::load_from_xsd`] for the level-group members
    /// below `section`.
    pub fn load_from_xsd(xsd: &XsdOntologyInstance) -> Vec<Self> {
        xsd.substitution_group_members("level")
            .into_iter()
            .filter_map(|name| Self::from_xsd_element(name, xsd))
            .collect()
    }

    /// Canonical USLM tag name.
    pub fn tag(self) -> &'static str {
        match self {
            Self::Subsection => "subsection",
            Self::Paragraph => "paragraph",
            Self::Subparagraph => "subparagraph",
            Self::Clause => "clause",
            Self::Subclause => "subclause",
            Self::Item => "item",
            Self::Subitem => "subitem",
        }
    }

    /// The depth of this kind in USLM's nesting order, 0-indexed
    /// from the closest §-child level. A Subsection is depth 0; a
    /// Subitem is depth 6.
    pub fn nesting_depth(self) -> usize {
        match self {
            Self::Subsection => 0,
            Self::Paragraph => 1,
            Self::Subparagraph => 2,
            Self::Clause => 3,
            Self::Subclause => 4,
            Self::Item => 5,
            Self::Subitem => 6,
        }
    }
}

/// Heading-like elements with semantic refinement beyond the basic
/// `<heading>`. Per LRC USLM User Guide § "Heading Variants".
///
/// USC titles use only `<heading>` (already modeled on
/// [`super::runtime_types::UsCodeTitle`],
/// [`super::runtime_types::UsCodeContainer`],
/// [`super::runtime_types::UsCodeSection`], etc.); the variants
/// below appear in bills, public laws, and CFR rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsCodeHeadingVariant {
    /// `<heading>` — the standard heading element (level inferred
    /// from parent container).
    Heading,
    /// `<subheading>` — a heading nested below a primary heading.
    Subheading,
    /// `<crossHeading>` — a heading that spans / cuts across the
    /// standard hierarchy (sidebar-style emphasis).
    CrossHeading,
    /// `<docTitle>` — title of a non-USC document (bill title,
    /// public-law title).
    DocTitle,
    /// `<longTitle>` — the full official title of an act.
    LongTitle,
    /// `<shortTitle>` — the short-title-by-which-the-act-may-be-cited
    /// (e.g. "Sarbanes-Oxley Act of 2002").
    ShortTitle,
}

impl UsCodeHeadingVariant {
    /// Canonical name → variant projection — the ontology semantic
    /// of the enum. The set of names this function accepts is the
    /// USLM 1.0 heading vocabulary (LRC USLM User Guide §
    /// "Heading Variants"). Use [`Self::from_xsd_element`] to
    /// additionally verify that the name is declared by the loaded
    /// USLM XSD before projection.
    pub fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "heading" => Self::Heading,
            "subheading" => Self::Subheading,
            "crossHeading" => Self::CrossHeading,
            "docTitle" => Self::DocTitle,
            "longTitle" => Self::LongTitle,
            "shortTitle" => Self::ShortTitle,
            _ => return None,
        })
    }

    /// XSD-grounded projection: confirms the name is declared as an
    /// `<xs:element>` in the loaded USLM XSD (W3C XSD 1.1 Part 1
    /// §3.3) before delegating to [`Self::parse`].
    pub fn from_xsd_element(tag: &str, xsd: &XsdOntologyInstance) -> Option<Self> {
        xsd.lookup_element(tag)?;
        Self::parse(tag)
    }
}

/// Additional hierarchical containers from LRC USLM User Guide
/// § "Hierarchical Containers" that don't appear in retro-
/// converted USC titles. Used by bills and public laws.
///
/// USC titles use only [`ContainerKind`] (Subtitle/Part/Subpart/
/// Chapter/Subchapter); these variants exist in the schema but
/// are zero-populated in pl-119-90.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsCodeAdditionalContainer {
    /// `<division>` — top-level division in bills (e.g. "Division A").
    Division,
    /// `<article>` — used in constitutional articles, treaties, and
    /// some uniform-act-style sources.
    Article,
    /// `<subarticle>` — sub-article level.
    Subarticle,
    /// `<preamble>` — recitals preceding the enacting clause.
    Preamble,
    /// `<preliminary>` — pre-statutory introductory text.
    Preliminary,
    /// `<appendix>` — appendix container at end of document.
    Appendix,
    /// `<subsubitem>` — one level below `<subitem>` (deepest USLM
    /// subdivision granularity).
    Subsubitem,
}

impl UsCodeAdditionalContainer {
    pub fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "division" => Self::Division,
            "article" => Self::Article,
            "subarticle" => Self::Subarticle,
            "preamble" => Self::Preamble,
            "preliminary" => Self::Preliminary,
            "appendix" => Self::Appendix,
            "subsubitem" => Self::Subsubitem,
            _ => return None,
        })
    }

    /// XSD-grounded variant of [`UsCodeAdditionalContainer::parse`].
    ///
    /// Most of these variants are members of the loaded USLM XSD's
    /// `substitutionGroup="level"` family (per W3C XSD 1.1 Part 1
    /// §3.3.6). The exception is `preamble`, which the loaded XSD
    /// declares at a different position in the level taxonomy
    /// alongside `<longTitle>` / `<docTitle>` rather than as a level
    /// member. For names not in the level group we still attempt the
    /// `<xs:element>` lookup so the dispatch reflects the loaded
    /// ontology rather than a hand-coded set.
    ///
    /// Per W3C XSD 1.1 Part 1 §3.3 (Element Declarations), names that
    /// don't have a matching `<xsd:element>` in the loaded XSD return
    /// `None`.
    ///
    /// Citation: LRC USLM XML User Guide § V (level hierarchy) and
    /// § "Hierarchical Containers".
    pub fn from_xsd_element(tag: &str, xsd: &XsdOntologyInstance) -> Option<Self> {
        // Element-declaration grounding (W3C XSD 1.1 Part 1 §3.3).
        xsd.lookup_element(tag)?;
        // Most additional-container variants are level-group members;
        // `appendix` and `preamble` may sit outside the level family
        // in some USLM revisions but are still XSD-declared, so the
        // grounding above is the load-bearing check. The projection
        // below applies once the name is confirmed to be in the loaded
        // schema.
        Self::parse(tag)
    }

    /// Enumerate every UsCodeAdditionalContainer variant the loaded
    /// USLM XSD admits. Unlike [`ContainerKind`] and
    /// [`SubdivisionKind`], the additional-container family is NOT
    /// substitution-group-defined as a single block — `preamble`
    /// and `appendix` sit outside the `level` head — so we walk
    /// every loaded `<xs:element>` declaration and project the ones
    /// our enum recognises.
    pub fn load_from_xsd(xsd: &XsdOntologyInstance) -> Vec<Self> {
        xsd.declared_element_names()
            .into_iter()
            .filter_map(|name| Self::from_xsd_element(name, xsd))
            .collect()
    }

    /// Canonical USLM tag name.
    pub fn tag(self) -> &'static str {
        match self {
            Self::Division => "division",
            Self::Article => "article",
            Self::Subarticle => "subarticle",
            Self::Preamble => "preamble",
            Self::Preliminary => "preliminary",
            Self::Appendix => "appendix",
            Self::Subsubitem => "subsubitem",
        }
    }
}

/// Quoted-content variants per LRC USLM User Guide § "Quoted
/// Content". USC titles use only the generic `<quotedContent>`
/// (already modeled as
/// [`super::runtime_types::UsCodeQuotedContent`]); the variants
/// below appear in non-USC documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsCodeQuotedVariant {
    /// `<quotedText>` — quoted text that's not statutory itself
    /// (e.g. quoting a court opinion).
    QuotedText,
    /// `<recital>` — a "Whereas…" recital in a bill's preamble.
    Recital,
    /// `<statement>` — an attributed statement (e.g. floor remarks).
    Statement,
}

impl UsCodeQuotedVariant {
    /// Canonical name → variant projection — the USLM 1.0
    /// quoted-content vocabulary per LRC USLM User Guide §
    /// "Quoted Content Variants". Use [`Self::from_xsd_element`]
    /// to additionally verify XSD declaration.
    pub fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "quotedText" => Self::QuotedText,
            "recital" => Self::Recital,
            "statement" => Self::Statement,
            _ => return None,
        })
    }

    /// XSD-grounded projection — confirms the name is declared as
    /// an `<xs:element>` by the loaded USLM XSD (W3C XSD 1.1 Part 1
    /// §3.3) before delegating to [`Self::parse`].
    pub fn from_xsd_element(tag: &str, xsd: &XsdOntologyInstance) -> Option<Self> {
        xsd.lookup_element(tag)?;
        Self::parse(tag)
    }
}

/// Legislative-formula elements per LRC USLM User Guide
/// § "Legislative Formulae". These are the ritual phrases that
/// surround statutory text in bills:
///
/// - `<enactingFormula>` — "Be it enacted by the Senate and House
///   of Representatives of the United States of America in Congress
///   assembled,"
/// - `<amendingFormula>` — "is amended—"
/// - `<approved>` / `<made>` — date-of-presidential-approval blocks
/// - `<action>` / `<instruction>` — direct-action verbs in
///   amendment instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsCodeLegislativeFormula {
    EnactingFormula,
    AmendingFormula,
    Approved,
    Made,
    Action,
    Instruction,
}

impl UsCodeLegislativeFormula {
    /// Canonical name → variant projection — the USLM 1.0
    /// legislative-formula vocabulary per LRC USLM User Guide §
    /// "Legislative Formulae". Use [`Self::from_xsd_element`] for
    /// XSD-grounded dispatch.
    pub fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "enactingFormula" => Self::EnactingFormula,
            "amendingFormula" => Self::AmendingFormula,
            "approved" => Self::Approved,
            "made" => Self::Made,
            "action" => Self::Action,
            "instruction" => Self::Instruction,
            _ => return None,
        })
    }

    /// XSD-grounded projection — confirms `<xs:element>` declaration
    /// in the loaded USLM XSD (W3C XSD 1.1 Part 1 §3.3).
    pub fn from_xsd_element(tag: &str, xsd: &XsdOntologyInstance) -> Option<Self> {
        xsd.lookup_element(tag)?;
        Self::parse(tag)
    }
}

/// Form elements per LRC USLM User Guide § "Form Elements". Used
/// in tax forms / SEC forms encoded as USLM; zero in USC titles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsCodeFormElement {
    /// `<checkBox>` — a check-the-box form field.
    CheckBox,
    /// `<fillIn>` — a fill-in-the-blank field.
    FillIn,
    /// `<block>` — a block-level group on a form.
    /// (Distinct from XHTML `<table>` rows.)
    Block,
    /// `<row>` — a non-table row (used in forms).
    Row,
    /// `<set>` — a grouping of related form fields.
    Set,
}

impl UsCodeFormElement {
    /// Canonical name → variant projection — the USLM 1.0 form
    /// vocabulary per LRC USLM User Guide § "Form Elements". Use
    /// [`Self::from_xsd_element`] for XSD-grounded dispatch.
    pub fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "checkBox" => Self::CheckBox,
            "fillIn" => Self::FillIn,
            "block" => Self::Block,
            "row" => Self::Row,
            "set" => Self::Set,
            _ => return None,
        })
    }

    /// XSD-grounded projection — confirms `<xs:element>` declaration
    /// in the loaded USLM XSD (W3C XSD 1.1 Part 1 §3.3).
    pub fn from_xsd_element(tag: &str, xsd: &XsdOntologyInstance) -> Option<Self> {
        xsd.lookup_element(tag)?;
        Self::parse(tag)
    }
}

/// W3C HTML insertion/deletion convention (HTML 4.01 §9.4 "Marking
/// Document Changes") adopted by USLM for legislative amendment
/// markup. Identifies whether a text fragment is being added to or
/// removed from the underlying statute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsCodeAmendmentKind {
    /// `<ins>` — text inserted by this amendment.
    Insertion,
    /// `<del>` — text deleted by this amendment.
    Deletion,
}

impl UsCodeAmendmentKind {
    /// Canonical name → variant projection — `ins`/`del` per HTML
    /// 4.01 §9.4 *Marking Document Changes*, adopted by USLM 1.0
    /// for legislative-amendment markup. Use
    /// [`Self::from_xsd_element`] for XSD-grounded dispatch.
    pub fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "ins" => Self::Insertion,
            "del" => Self::Deletion,
            _ => return None,
        })
    }

    /// XSD-grounded projection — confirms `<xs:element>` declaration
    /// in the loaded USLM XSD (W3C XSD 1.1 Part 1 §3.3).
    pub fn from_xsd_element(tag: &str, xsd: &XsdOntologyInstance) -> Option<Self> {
        xsd.lookup_element(tag)?;
        Self::parse(tag)
    }
}

/// USLM-recognized inline kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InlineKind {
    /// Plain text (no inline markup ornament). The default kind
    /// for text directly inside a heading/chapeau/content.
    PlainText,
    /// `<inline class="...">` — USLM's generic inline-markup
    /// element. Class typically `"small-caps"` or layout class.
    Inline,
    /// `<i>` — italic. Standard usage: statutory terms-of-art,
    /// foreign words, references to laws by short title.
    Italic,
    /// `<b>` — bold.
    Bold,
    /// `<sup>` — superscript (footnote markers, ordinal exponents).
    Superscript,
    /// `<sub>` — subscript.
    Subscript,
    /// `<span>` — generic inline span (often class-bearing).
    Span,
    /// `<a href="...">` — anchor / hyperlink. Distinct from
    /// `<ref>` in USLM: `<a>` is HTML-style; `<ref>` is the
    /// citation-graph link.
    Anchor,
}

impl InlineKind {
    /// Canonical name → variant projection — the USLM 1.0 inline
    /// markup vocabulary (LRC USLM User Guide § "Inline Markup").
    /// Returns `None` for non-inline tags. Use
    /// [`Self::from_xsd_element`] for XSD-grounded dispatch.
    pub fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "inline" => Self::Inline,
            "i" => Self::Italic,
            "b" => Self::Bold,
            "sup" => Self::Superscript,
            "sub" => Self::Subscript,
            "span" => Self::Span,
            "a" => Self::Anchor,
            _ => return None,
        })
    }

    /// XSD-grounded projection — confirms `<xs:element>` declaration
    /// in the loaded USLM XSD (W3C XSD 1.1 Part 1 §3.3).
    pub fn from_xsd_element(tag: &str, xsd: &XsdOntologyInstance) -> Option<Self> {
        xsd.lookup_element(tag)?;
        Self::parse(tag)
    }
}

/// HTML 4.01 §11.2.6 distinguishes header (`<th>`) from data (`<td>`)
/// cells; the visual rendering convention is different (bold vs
/// regular) and the semantic role is different (label vs value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsCodeTableCellKind {
    /// `<th>` — header cell. Labels the row or column it belongs to.
    Header,
    /// `<td>` — data cell. Carries a tabular value.
    Data,
}

/// Classification of [`super::runtime_types::UsCodeNote`] per LRC
/// USLM User Guide § 6.2 "Note Topics". The kind is derived from
/// the `topic` attribute on `<note>`, not a separate stored value
/// — the topic is the source of truth.
///
/// Observed `topic` values in LRC release pl-119-90:
/// `enacting`, `editorialNotes`, `statutoryNotes`, `amendments`,
/// `codification`, `dispositionOfSections`, `effectiveDate`,
/// `separability`, `miscellaneous`, `footnote`. The classification
/// here is structural — what kind of legal-research role does the
/// note play — not stylistic.
///
/// Citation: LRC, *USLM XML User Guide* (USLM-1.0.15.xsd) § 6.2
/// "Note Topics"; LRC's Office of the Law Revision Counsel
/// editorial-note conventions documented at uscode.house.gov.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsCodeNoteKind {
    /// Material added by the codifier (LRC's Office of the Law
    /// Revision Counsel). Includes section-history headers,
    /// cross-references between related sections, etc.
    Editorial,
    /// Material from the underlying public laws — text that didn't
    /// fit into a numbered subsection but is still statutory.
    Statutory,
    /// Records amendments / re-codifications to the section text
    /// (topic `amendments` or `codification`). The change-history
    /// of the statute.
    Change,
    /// The originating enacting clause, typically with the Pub. L.
    /// citation that put this section into the USC.
    Enacting,
    /// Footnote (carries `type="footnote"` rather than a topic).
    /// Differs from a structural note by being typeset as a
    /// footnote at render time.
    Footnote,
    /// A note whose `topic` doesn't yet have a typed classification
    /// in this ontology. Caller should treat as a tripwire — the
    /// topic vocabulary in the LRC's published guidance may have
    /// extended, and `UsCodeNoteKind::parse` should be updated.
    Unrecognized,
}

impl UsCodeNoteKind {
    /// Classify a `<note>` from its `topic` and `type` attributes.
    /// Both are optional in the schema; if neither carries a
    /// recognized value, returns [`UsCodeNoteKind::Unrecognized`].
    pub fn parse(topic: Option<&str>, type_attr: Option<&str>) -> Self {
        // `type="footnote"` takes precedence over the topic.
        if type_attr == Some("footnote") {
            return Self::Footnote;
        }
        match topic {
            Some("editorialNotes") => Self::Editorial,
            Some("statutoryNotes") => Self::Statutory,
            Some("amendments") | Some("codification") => Self::Change,
            Some("enacting") => Self::Enacting,
            // Topics observed but not yet classified as a kind:
            // dispositionOfSections, effectiveDate, separability,
            // miscellaneous. The Editorial / Statutory / Change /
            // Enacting / Footnote partition is structural, not
            // exhaustive over the topic vocabulary.
            _ => Self::Unrecognized,
        }
    }
}

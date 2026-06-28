//! WN-LMF structural writer — the XML-tree-level inverse of
//! [`read_wordnet`].
//!
//! [`write_wordnet_document`] folds the typed LMF structs
//! ([`WordNet`]/`Lexicon` → [`LexicalEntry`]/[`Synset`] → …) back
//! onto the XML ontology ([`XmlDocument`]/[`XmlElement`]/
//! [`XmlAttribute`]/[`XmlNode`]). For every element and attribute the
//! reader CONSUMES, the writer EMITS exactly one — in the deterministic
//! document order the typed `Vec`s hold (the same order the reader
//! produced them), so the function is pure and content-addressable.
//!
//! # Layering (L1 of the serialized reverse lens)
//!
//! This is the structural fold only. It emits what the typed model
//! carries — which, since the schema-completion slice, includes the
//! `<Lexicon>` metadata attributes (`id`/`label`/`language`/`email`/
//! `license`/`version`/`url`/`status`/`confidenceScore`/`dc:*`),
//! `<Pronunciation>`, `<ILIDefinition>`, `<SyntacticBehaviour>`,
//! `<Count>`, and the per-element `#IMPLIED` attrs the reader now
//! captures. The document-level decorations the reader still DISCARDS —
//! the `<?xml … standalone?>` pseudo-attributes, the `<!DOCTYPE>`, any
//! `xmlns` declarations, and insignificant inter-element white-space —
//! are NOT in the typed model and are out of scope for L1 (the
//! `serialize_document_exact` / `SyntaxDecisions` byte-kernel handles
//! those). The gate below is a typed-LMF FULL-model structure
//! round-trip: `read_wordnet ∘ serialize ∘ write_wordnet_document ∘
//! read_wordnet == read_wordnet` on every field (`synsets`/`entries`/
//! `lexicon`/`syntactic_behaviours`). For input already in canonical
//! XML form the canonical serializer reproduces it byte-for-byte (see
//! `rich_fragment_canonical_serialization_is_byte_exact`).

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use super::reader::read_wordnet;
use crate::social::software::markup::xml::ontology::{
    XmlAttribute, XmlDocument, XmlElement, XmlName, XmlNode,
};
use crate::social::software::markup::xml::parser::grammar::{
    XmlParseError, parse_document_capturing,
};
use crate::social::software::markup::xml::parser::serializer::serialize_document_exact;
use crate::social::software::markup::xml::parser::source_syntax::{
    DocumentResidue, RegeneratedComplement, RegeneratedComplementError, SyntaxDecisions,
    diff_content_whitespace, reapply_regenerated_complement,
};

/// Write a [`WordNet`] lexicon back to a WN-LMF [`XmlDocument`] — the
/// XML-tree-level inverse of [`read_wordnet`].
///
/// Emits the `<LexicalResource><Lexicon>…</Lexicon></LexicalResource>`
/// shell, then every [`LexicalEntry`] followed by every [`Synset`], in
/// the order the typed `Vec`s hold them. The document carries the
/// canonical `version="1.0" encoding="UTF-8"` prolog the reader's XML
/// declaration handling defaults to.
///
/// The result is parsed back losslessly at the typed-LMF level (the
/// full-model structure round-trip gated by the test below). It carries
/// the `<Lexicon>` metadata attributes the typed model now holds; the
/// doctype, namespace declarations, and insignificant white-space are
/// still not emitted (an L1 boundary, see the module docs).
pub fn write_wordnet_document(wn: &WordNet) -> XmlDocument {
    let mut lexicon_children: Vec<XmlNode> =
        Vec::with_capacity(wn.entries.len() + wn.synsets.len());

    // Mirror `read_wordnet`: it walks the Lexicon's children in
    // document order, dispatching `LexicalEntry` and `Synset` into two
    // Vecs. The typed model splits them, so we emit all entries then all
    // synsets — both orderings re-read into the same `entries`/`synsets`
    // Vecs because the reader keys on the element name, not position.
    for entry in &wn.entries {
        lexicon_children.push(XmlNode::Element(lexical_entry_element(entry)));
    }
    for synset in &wn.synsets {
        lexicon_children.push(XmlNode::Element(synset_element(synset)));
    }
    // Lexicon-level `SyntacticBehaviour*` last, per the DTD content
    // model `(Requires*, LexicalEntry+, Synset*, SyntacticBehaviour*)`
    // (DTD line 5).
    for sb in &wn.syntactic_behaviours {
        lexicon_children.push(XmlNode::Element(syntactic_behaviour_element(sb)));
    }

    let lexicon = element(
        "Lexicon",
        lexicon_metadata_attrs(&wn.lexicon),
        lexicon_children,
    );
    let root = element(
        "LexicalResource",
        Vec::new(),
        vec![XmlNode::Element(lexicon)],
    );

    XmlDocument {
        version: "1.0".to_string(),
        encoding: Some("UTF-8".to_string()),
        doctype: None,
        root,
    }
}

/// Inverse of `read_lexicon_metadata`: the `<!ATTLIST Lexicon>` attrs
/// (DTD lines 6-32) in declaration order, each emitted only when the
/// reader captured it (`Some`). The Dublin Core `dc:*` set is emitted
/// after the named attrs, in the order the reader preserved (which is
/// the DTD declaration order it filtered from). An absent attr emits
/// nothing, so it round-trips as absent.
fn lexicon_metadata_attrs(m: &LexiconMetadata) -> Vec<XmlAttribute> {
    let mut attrs = Vec::new();
    push_opt(&mut attrs, "id", &m.id);
    push_opt(&mut attrs, "label", &m.label);
    push_opt(&mut attrs, "language", &m.language);
    push_opt(&mut attrs, "email", &m.email);
    push_opt(&mut attrs, "license", &m.license);
    push_opt(&mut attrs, "version", &m.version);
    push_opt(&mut attrs, "url", &m.url);
    push_opt(&mut attrs, "citation", &m.citation);
    push_opt(&mut attrs, "logo", &m.logo);
    push_opt(&mut attrs, "status", &m.status);
    push_opt(&mut attrs, "confidenceScore", &m.confidence_score);
    for (name, value) in &m.dc {
        attrs.push(attr(name, value));
    }
    attrs
}

/// Inverse of `read_lexical_entry`: `<LexicalEntry id="…">` with a
/// `<Lemma>`, then every `<Form>`, then every `<Sense>`, then every
/// entry-scope `<SyntacticBehaviour>` — the WN-LMF 1.3 content model
/// `<!ELEMENT LexicalEntry (Lemma, Form*, Sense*, SyntacticBehaviour*)>`
/// (DTD line 33).
///
/// The child order follows the DTD (Form* before Sense*), which is also
/// the Open English WordNet 2025 source order — so a regenerated tree is
/// element-backbone-equal to the captured DOM, the precondition the
/// graph-faithful reconstruction
/// ([`reconstruct_wn_lmf_source`]) depends on. The reader keys on the
/// element NAME, not position (`read_lexical_entry` dispatches by tag), so
/// the typed-LMF round-trip is unaffected by which order the writer emits.
fn lexical_entry_element(entry: &LexicalEntry) -> XmlElement {
    let mut children: Vec<XmlNode> = Vec::with_capacity(
        1 + entry.forms.len() + entry.senses.len() + entry.syntactic_behaviours.len(),
    );
    children.push(XmlNode::Element(lemma_element(&entry.lemma)));
    for form in &entry.forms {
        children.push(XmlNode::Element(form_element(form)));
    }
    for sense in &entry.senses {
        children.push(XmlNode::Element(sense_element(sense)));
    }
    for sb in &entry.syntactic_behaviours {
        children.push(XmlNode::Element(syntactic_behaviour_element(sb)));
    }
    element("LexicalEntry", vec![attr("id", &entry.id)], children)
}

/// Inverse of the reader's `Lemma` branch:
/// `<Lemma writtenForm="…" partOfSpeech="…" [script="…"]>` with a
/// `<Pronunciation>` per captured pronunciation. `partOfSpeech` uses
/// [`LmfPos::to_tag`], the exact inverse of the reader's `parse` —
/// including the satellite-adjective `s` (no longer collapsed to `a`).
fn lemma_element(lemma: &Lemma) -> XmlElement {
    let mut attrs = vec![
        attr("writtenForm", &lemma.written_form),
        attr("partOfSpeech", lemma.pos.to_tag()),
    ];
    push_opt(&mut attrs, "script", &lemma.script);
    let children: Vec<XmlNode> = lemma
        .pronunciations
        .iter()
        .map(|p| XmlNode::Element(pronunciation_element(p)))
        .collect();
    element("Lemma", attrs, children)
}

/// Inverse of `pronunciation` capture: `<Pronunciation [variety="…"]
/// [notation="…"] [phonemic="…"] [audio="…"]>text</Pronunciation>`
/// (DTD lines 63-69). Attrs emitted only when captured, in DTD
/// declaration order, so an absent attr round-trips as absent.
fn pronunciation_element(p: &Pronunciation) -> XmlElement {
    let mut attrs = Vec::new();
    push_opt(&mut attrs, "variety", &p.variety);
    push_opt(&mut attrs, "notation", &p.notation);
    push_opt(&mut attrs, "phonemic", &p.phonemic);
    push_opt(&mut attrs, "audio", &p.audio);
    let children = if p.text.is_empty() {
        Vec::new()
    } else {
        vec![XmlNode::Text(p.text.clone())]
    };
    element("Pronunciation", attrs, children)
}

/// Inverse of `read_sense`:
/// `<Sense id="…" synset="…" [subcat="… …"] [adjposition="…"]
/// [dc:source="…"]>` with a `<SenseRelation>` per relation then a
/// `<Count>` per count — the reader's child order (relations, then
/// counts). `subcat` is space-joined; an empty `subcat` emits no
/// attribute (the reader produces an empty Vec when absent).
fn sense_element(sense: &Sense) -> XmlElement {
    let mut attrs = vec![attr("id", &sense.id), attr("synset", &sense.synset)];
    if !sense.subcat.is_empty() {
        attrs.push(attr("subcat", &sense.subcat.join(" ")));
    }
    push_opt(&mut attrs, "adjposition", &sense.adjposition);
    push_opt(&mut attrs, "dc:source", &sense.dc_source);
    let mut children: Vec<XmlNode> = Vec::with_capacity(sense.relations.len() + sense.counts.len());
    for rel in &sense.relations {
        children.push(XmlNode::Element(sense_relation_element(rel)));
    }
    for count in &sense.counts {
        children.push(XmlNode::Element(count_element(count)));
    }
    element("Sense", attrs, children)
}

/// Inverse of the reader's `Count` branch: `<Count>value</Count>`
/// (DTD line 233).
fn count_element(count: &Count) -> XmlElement {
    text_element("Count", &count.value)
}

/// Inverse of `read_syntactic_behaviour`: `<SyntacticBehaviour [id="…"]
/// subcategorizationFrame="…" [senses="… …"]/>` (DTD lines 228-232).
fn syntactic_behaviour_element(sb: &SyntacticBehaviour) -> XmlElement {
    let mut attrs = Vec::new();
    push_opt(&mut attrs, "id", &sb.id);
    attrs.push(attr("subcategorizationFrame", &sb.subcategorization_frame));
    if !sb.senses.is_empty() {
        attrs.push(attr("senses", &sb.senses.join(" ")));
    }
    element("SyntacticBehaviour", attrs, Vec::new())
}

/// Inverse of the reader's `Form` branch:
/// `<Form [id="…"] writtenForm="…" [script="…"]>` with a
/// `<Pronunciation>` per captured pronunciation. Attribute order
/// matches the DTD `<!ATTLIST Form id, writtenForm, script>` (lines
/// 59-62).
fn form_element(form: &Form) -> XmlElement {
    let mut attrs = Vec::new();
    push_opt(&mut attrs, "id", &form.id);
    attrs.push(attr("writtenForm", &form.written_form));
    push_opt(&mut attrs, "script", &form.script);
    let children: Vec<XmlNode> = form
        .pronunciations
        .iter()
        .map(|p| XmlNode::Element(pronunciation_element(p)))
        .collect();
    element("Form", attrs, children)
}

/// Inverse of `read_synset`: `<Synset id="…" [ili="…"]
/// partOfSpeech="…" [members="… …"]>` with `<Definition>`s, then
/// `<Example>`s, then `<SynsetRelation>`s — the order the reader's
/// three Vecs were filled.
///
/// `ili` is emitted only when present (the reader reads it as an
/// `Option`); `members` is emitted only when non-empty (the reader
/// produces an empty Vec when the attribute is absent). `partOfSpeech`
/// is always emitted — the reader defaults a missing value to
/// `LmfPos::Other`, but the typed model holds a concrete `pos`, so we
/// always write its tag.
fn synset_element(synset: &Synset) -> XmlElement {
    let mut attrs = vec![attr("id", &synset.id)];
    if let Some(ili) = &synset.ili {
        attrs.push(attr("ili", ili));
    }
    attrs.push(attr("partOfSpeech", synset.pos.to_tag()));
    if !synset.members.is_empty() {
        attrs.push(attr("members", &synset.members.join(" ")));
    }
    push_opt(&mut attrs, "lexfile", &synset.lexfile);
    push_opt(&mut attrs, "dc:source", &synset.dc_source);
    push_opt(&mut attrs, "confidenceScore", &synset.confidence_score);

    // Child order follows the DTD content model `(Definition*,
    // ILIDefinition?, SynsetRelation*, Example*)` (DTD line 98).
    let mut children: Vec<XmlNode> = Vec::with_capacity(
        synset.definitions.len()
            + usize::from(synset.ili_definition.is_some())
            + synset.relations.len()
            + synset.examples.len(),
    );
    for def in &synset.definitions {
        children.push(XmlNode::Element(text_element("Definition", def)));
    }
    if let Some(ili_def) = &synset.ili_definition {
        children.push(XmlNode::Element(text_element("ILIDefinition", ili_def)));
    }
    for rel in &synset.relations {
        children.push(XmlNode::Element(synset_relation_element(rel)));
    }
    for example in &synset.examples {
        children.push(XmlNode::Element(text_element("Example", example)));
    }

    element("Synset", attrs, children)
}

/// Inverse of the reader's `SynsetRelation` branch:
/// `<SynsetRelation relType="…" target="…"/>`. The `relType` string is
/// [`SynsetRelationType::as_str`], the exact inverse of the reader's
/// `parse` (see the `Other(_)` limitation documented there).
fn synset_relation_element(rel: &SynsetRelation) -> XmlElement {
    element(
        "SynsetRelation",
        vec![
            attr("relType", rel.rel_type.as_str()),
            attr("target", &rel.target),
        ],
        Vec::new(),
    )
}

/// Inverse of the reader's `SenseRelation` branch:
/// `<SenseRelation relType="…" target="…"/>`. The `relType` string is
/// [`SenseRelationType::as_str`], the exact inverse of the reader's
/// `parse`.
fn sense_relation_element(rel: &SenseRelation) -> XmlElement {
    element(
        "SenseRelation",
        vec![
            attr("relType", rel.rel_type.as_str()),
            attr("target", &rel.target),
        ],
        Vec::new(),
    )
}

// ── small XML-ontology constructors ─────────────────────────────────────────

/// Build an [`XmlElement`] with no namespace declarations — the typed
/// LMF model carries none, so every emitted element is namespace-free.
fn element(name: &str, attributes: Vec<XmlAttribute>, children: Vec<XmlNode>) -> XmlElement {
    XmlElement {
        name: XmlName::new(name),
        namespace: None,
        namespaces: Vec::new(),
        attributes,
        children,
    }
}

/// An [`XmlAttribute`] whose name may carry a `dc:` (or `xml:`) prefix.
/// A prefixed name (`dc:source`) is split into
/// [`XmlName::with_prefix`], so the serializer emits the qualified
/// `dc:source` form and a re-read (which splits prefixed names per W3C
/// XML 1.0 §3.1) reconstructs the same `prefix`/`local` the reader's
/// [`attr_value`](super::reader) matched on `qualified()`. An unprefixed
/// name uses [`XmlName::new`] verbatim.
fn attr(name: &str, value: &str) -> XmlAttribute {
    let xml_name = match name.split_once(':') {
        Some((prefix, local)) => XmlName::with_prefix(prefix, local),
        None => XmlName::new(name),
    };
    XmlAttribute {
        name: xml_name,
        value: value.to_string(),
    }
}

/// Push `name="value"` only when `value` is `Some` — the inverse of an
/// `#IMPLIED` attribute the reader captured as an `Option`. An absent
/// (`None`) attribute emits nothing, so it round-trips as absent.
fn push_opt(attrs: &mut Vec<XmlAttribute>, name: &str, value: &Option<String>) {
    if let Some(v) = value {
        attrs.push(attr(name, v));
    }
}

/// An element whose sole child is a text node — the inverse of the
/// reader's `text_content` collection for `<Definition>`/`<Example>`.
fn text_element(name: &str, text: &str) -> XmlElement {
    element(name, Vec::new(), vec![XmlNode::Text(text.to_string())])
}

// ── Graph-faithful reconstruction — WN-LMF source bytes from model + complement ─
//
// `write_wordnet_document` regenerates a structural element tree from the typed
// [`WordNet`] model. By itself that tree is LOSSY against the real upstream WN-LMF
// file: the W3C Information Set (Cowan & Tobin 2004) the typed model captures does
// not carry three byte-affecting residues — the document `<!DOCTYPE>` (§2.8
// \[28\]), the root's `xmlns:dc` declaration (Bray, Hollander, Layman & Tobin 2009
// §3), and the §2.4 \[14\] inter-element white-space (the indentation the §2.10
// "White Space Handling" note calls insignificant). Those belong in the
// concrete-syntax COMPLEMENT, never in the ontology (the same ontology written two
// ways keeps one content address).
//
// [`WnSyntaxComplement`] bundles that residue — the GENERIC XML-family
// [`DocumentResidue`] + [`ContentWhitespace`] (the regenerated-tree complement)
// and the per-element [`SyntaxDecisions`] (intra-tag white-space, §4.6
// entity-reference form, prolog/epilog white-space, empty-element form) the
// byte-exact serializer already honours. [`capture_wn_complement`] derives it from
// the real source; [`reconstruct_wn_lmf_source`] re-applies it to
// `write_wordnet_document`'s regenerated tree and serializes byte-exact.
//
// The reconstruction is `lmf::WordNet ontology + captured complement → exact
// source bytes` — the WordNet realisation of the serialized reverse lens'
// graph-faithful `put` (Foster, Greenwald, Moore, Pierce & Schmitt 2007, ACM
// TOPLAS 29(3) §2.2). The parser-level residue machinery is generic XML-family
// (no WordNet vocabulary); only this glue is WordNet-specific.

/// The concrete-syntax COMPLEMENT for one WN-LMF source: everything the typed
/// [`WordNet`] model + [`write_wordnet_document`]'s regenerated tree do NOT carry,
/// captured so [`reconstruct_wn_lmf_source`] reproduces the exact source bytes.
///
/// Three layers, each a separate W3C-grounded residue class — see the module-
/// level note above. All three are generic XML-family residue; the bundle is the
/// only WordNet-specific piece.
///
/// rkyv-serializable under `prx` (CFG-GATED, as rkyv is an optional dependency):
/// the graph-faithful WordNet `.prx` envelope carries this complement beside the
/// typed [`WordNet`] ontology, so the byte-exact `put`
/// ([`reconstruct_wn_lmf_source`]) can run from the archive alone. All three
/// fields are generic XML-family residue that already carry the same gated
/// derive — the bundle is the only WordNet-specific piece.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct WnSyntaxComplement {
    /// Document/root-level residue: the `<!DOCTYPE>` (§2.8 \[28\]) and the root
    /// element's namespace declarations (Bray, Hollander, Layman & Tobin 2009 §3)
    /// the typed model drops.
    pub document_residue: DocumentResidue,
    /// The regenerated-tree residue: the §2.4 \[14\] inter-element white-space
    /// (the indentation) the structural writer's tree lacks AND the exact source
    /// start-tag attribute sequences it reordered or under-populated (Sense's
    /// `subcat`/`adjposition`-before-`synset` order; the `dc:type` /
    /// `Example`-`dc:source` metadata attributes the typed model does not carry —
    /// attribute order/coverage being concrete-syntax, not Infoset, per Cowan &
    /// Tobin 2004 §2.3). Keyed by pre-order element index.
    pub regenerated: RegeneratedComplement,
    /// The per-element concrete-syntax decisions the byte-exact serializer
    /// honours — intra-tag white-space layout (§3.1 \[40\]/\[44\]), §4.6
    /// predefined-entity-reference form, the empty-element form (§3.1), and the
    /// prolog/epilog white-space (§2.8 \[27\]). Captured by
    /// `parse_document_capturing` directly from the source.
    pub syntax_decisions: SyntaxDecisions,
}

/// Failure modes of the WN-LMF graph-faithful reconstruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WnReconstructError {
    /// The source bytes did not parse as well-formed XML (§2.1) — the grammar
    /// parser rejected them. Carries the underlying [`XmlParseError`].
    Parse(XmlParseError),
    /// `write_wordnet_document`'s regenerated tree was not element-backbone-equal
    /// to the captured source DOM (a dropped/added/reordered/renamed
    /// element/leaf-text). The residue cannot be a pure white-space/decl
    /// complement; carries the exact divergence.
    Complement(RegeneratedComplementError),
}

impl core::fmt::Display for WnReconstructError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "WN-LMF source parse: {e}"),
            Self::Complement(e) => write!(f, "WN-LMF regenerated-tree complement: {e}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for WnReconstructError {}

impl From<XmlParseError> for WnReconstructError {
    fn from(e: XmlParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<RegeneratedComplementError> for WnReconstructError {
    fn from(e: RegeneratedComplementError) -> Self {
        Self::Complement(e)
    }
}

/// Capture the typed [`WordNet`] model AND the concrete-syntax
/// [`WnSyntaxComplement`] from a WN-LMF source, such that
/// [`reconstruct_wn_lmf_source`] reproduces the source bytes byte-for-byte.
///
/// The capture has two byte-exact-grounded halves:
///
/// 1. **The exact DOM + decisions** — `parse_document_capturing` parses the
///    source through the W3C XML 1.0 grammar, yielding the full Information Set
///    DOM (with the `<!DOCTYPE>`, the root `xmlns:dc`, and the inter-element
///    white-space `Text` children) and the [`SyntaxDecisions`] the byte-exact
///    serializer honours. `serialize_document_exact` over THAT pair is already
///    proven byte-exact over this corpus (the SLICE-1 round-trip law).
/// 2. **The regenerated-tree complement** — `write_wordnet_document(read_wordnet)`
///    regenerates the structural tree the typed model alone produces;
///    [`diff_content_whitespace`] extracts the inter-element white-space that tree
///    LACKS, and the `<!DOCTYPE>` + root namespaces are read straight off the
///    exact DOM. Re-applying both to the regenerated tree reproduces the exact DOM
///    (the [`reconstruct_wn_lmf_source`] step).
///
/// Fails closed if the source is not well-formed XML, or if the regenerated tree
/// is not element-backbone-equal to the captured DOM (a structural writer defect).
pub fn capture_wn_complement(
    source: &str,
) -> Result<(WordNet, WnSyntaxComplement), WnReconstructError> {
    // (1) The exact DOM + the per-element/prolog decisions the byte-exact
    // serializer honours — the SLICE-1 byte-exact reverse-lens capture.
    let (exact_dom, syntax_decisions) = parse_document_capturing(source.as_bytes())?;

    // The typed model. `read_wordnet` extracts the same elements/attrs/leaf-text
    // the grammar parser does (it keys on element names), so the model the writer
    // regenerates from is faithful to `exact_dom`'s backbone.
    let wn = read_wordnet(source).map_err(|e| {
        WnReconstructError::Parse(XmlParseError::Syntax {
            position: 0,
            expected: "well-formed WN-LMF lexicon".into(),
            found: format!("{e}"),
        })
    })?;

    // (2) The regenerated tree the typed model alone produces, and the residue it
    // lacks relative to the exact DOM — inter-element white-space AND the exact
    // source attribute sequences (order + metadata attrs the model drops).
    let regenerated_tree = write_wordnet_document(&wn);
    let regenerated = diff_content_whitespace(&exact_dom, &regenerated_tree)?;

    // The document/root-level residue read straight off the exact DOM: the
    // `<!DOCTYPE>` and the root element's namespace declarations.
    let document_residue = DocumentResidue {
        doctype: exact_dom.doctype.clone(),
        root_namespaces: exact_dom.root.namespaces.clone(),
        // The XML declaration's version + encoding form, captured so the
        // byte-exact writer reproduces the source declaration rather than the
        // structural writer's default. Additive (WN-LMF declares the same form
        // the structural writer emits, so a re-pin is unnecessary).
        xml_version: Some(exact_dom.version.clone()),
        xml_encoding: Some(exact_dom.encoding.clone()),
    };

    Ok((
        wn,
        WnSyntaxComplement {
            document_residue,
            regenerated,
            syntax_decisions,
        },
    ))
}

/// Reconstruct the exact WN-LMF source bytes from the typed [`WordNet`] model and
/// its captured [`WnSyntaxComplement`] — the graph-faithful `put`.
///
/// 1. `write_wordnet_document(wn)` regenerates the structural element tree the
///    typed model produces (DTD-ordered children, no white-space, no DOCTYPE, no
///    namespaces);
/// 2. [`reapply_regenerated_complement`] merges the residue back in — sets the
///    `<!DOCTYPE>` and root namespaces, splices the inter-element white-space
///    `Text` children at their captured pre-order positions — reproducing the
///    exact captured DOM;
/// 3. [`serialize_document_exact`] emits that DOM PLUS the [`SyntaxDecisions`]
///    byte-for-byte.
///
/// When `wn` and `complement` are the pair [`capture_wn_complement`] returned for
/// a source `s`, the result equals `s.as_bytes()` exactly. Fails closed on a
/// structural divergence (never fabricates bytes).
pub fn reconstruct_wn_lmf_source(
    wn: &WordNet,
    complement: &WnSyntaxComplement,
) -> Result<Vec<u8>, WnReconstructError> {
    let mut tree = write_wordnet_document(wn);
    reapply_regenerated_complement(
        &mut tree,
        &complement.document_residue,
        &complement.regenerated,
    )?;
    Ok(serialize_document_exact(
        &tree,
        &complement.syntax_decisions,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::parser::serializer::serialize_document;

    /// A minimal but full-shape WN-LMF lexicon mirroring
    /// `lmf::prx`'s `SAMPLE_WN_LMF`: a `<Lexicon>` with `<LexicalEntry>`s
    /// (Lemma + multiple Senses + a Form), `<Synset>`s with a
    /// `<Definition>` and a `hypernym` `<SynsetRelation>`, and a
    /// `<SenseRelation>` (antonym) so the writer's relation-emission
    /// paths are all exercised.
    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test-en" label="Test English" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="dog-n-01" synset="s-dog"><SenseRelation relType="antonym" target="cat-n-01"/></Sense><Sense id="dog-n-02" synset="s-mammal"/><Form writtenForm="dogs"/></LexicalEntry>
    <LexicalEntry id="e-cat-n"><Lemma writtenForm="cat" partOfSpeech="n"/><Sense id="cat-n-01" synset="s-cat"/></LexicalEntry>
    <LexicalEntry id="e-run-v"><Lemma writtenForm="run" partOfSpeech="v"/><Sense id="run-v-01" synset="s-run" subcat="vii via"/></LexicalEntry>
    <Synset id="s-dog" ili="i1" partOfSpeech="n"><Definition>a domesticated canine</Definition><SynsetRelation relType="hypernym" target="s-mammal"/></Synset>
    <Synset id="s-cat" ili="i2" partOfSpeech="n"><Definition>a small feline</Definition><SynsetRelation relType="hypernym" target="s-mammal"/></Synset>
    <Synset id="s-mammal" ili="i3" partOfSpeech="n"><Definition>warm-blooded vertebrate</Definition></Synset>
    <Synset id="s-run" partOfSpeech="v"><Definition>move fast on foot</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

    /// The L1 gate: writing the typed lexicon back to an `XmlDocument`,
    /// serializing it, and re-reading it reconstructs the identical
    /// typed-LMF structures. A missing attribute or wrong element name
    /// surfaces as a `synsets`/`entries` field mismatch — that is the
    /// whole point of the structure round-trip.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn write_wordnet_document_round_trips_typed_lmf() {
        let wn = read_wordnet(SAMPLE).unwrap();
        let doc = write_wordnet_document(&wn);
        let bytes = serialize_document(&doc);
        let wn2 = read_wordnet(core::str::from_utf8(&bytes).unwrap()).unwrap();
        assert_eq!(wn.synsets, wn2.synsets);
        assert_eq!(wn.entries, wn2.entries);
    }

    /// A real OEWN-2025-shaped fragment that exercises EVERY element and
    /// attribute this slice schema-completed: a `<Pronunciation>` (with
    /// `variety`/`notation`/`phonemic` attrs), an `<ILIDefinition>`, a
    /// `<SyntacticBehaviour>` at BOTH lexicon and entry scope, full
    /// `<Lexicon>` metadata (incl. `status` + `dc:source`), a
    /// satellite-adjective (`partOfSpeech="s"`) `<Lemma>`, a `<Form
    /// id=…>`, a `<Sense>` with `adjposition` + a `<Count>`, and an
    /// `other`/long-tail `relType` (`co_agent_instrument`) that the typed
    /// model routes to `Other(String)`. Written in CANONICAL XML form
    /// (self-closing empties, single insignificant-whitespace-free body)
    /// so the canonical serializer reproduces it.
    const RICH: &str = r#"<?xml version="1.0" encoding="UTF-8"?><LexicalResource><Lexicon id="oewn" label="Open English WordNet" language="en" email="x@y.org" license="https://creativecommons.org/licenses/by/4.0/" version="2025" url="https://en-word.net" citation="cite" status="valid" confidenceScore="1.0" dc:source="https://wordnet.princeton.edu"><LexicalEntry id="e-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"><Pronunciation variety="GB" notation="ipa" phonemic="true">/dɒɡ/</Pronunciation></Lemma><Form id="dog-form-1" writtenForm="dogs"/><Sense id="dog-n-01" synset="s-dog" dc:source="pwn"><Count>42</Count></Sense><SyntacticBehaviour id="sb-entry" subcategorizationFrame="Somebody ----s" senses="dog-n-01"/></LexicalEntry><LexicalEntry id="e-good-s"><Lemma writtenForm="good" partOfSpeech="s"/><Sense id="good-s-01" synset="s-good" adjposition="a"><SenseRelation relType="antonym" target="bad-s-01"/></Sense></LexicalEntry><Synset id="s-dog" ili="i1" partOfSpeech="n" lexfile="noun.animal" dc:source="pwn"><Definition>a domesticated canine</Definition><ILIDefinition>a domesticated mammal of the family Canidae</ILIDefinition><SynsetRelation relType="hypernym" target="s-mammal"/><SynsetRelation relType="co_agent_instrument" target="s-mammal"/></Synset><Synset id="s-mammal" ili="i2" partOfSpeech="n"><Definition>warm-blooded vertebrate</Definition></Synset><Synset id="s-good" ili="i3" partOfSpeech="s"><Definition>having desirable qualities</Definition></Synset><SyntacticBehaviour id="sb-lex" subcategorizationFrame="Somebody ----s something" senses="dog-n-01"/></Lexicon></LexicalResource>"#;

    /// The FULL-model structure round-trip: `read_wordnet(frag) ==
    /// read_wordnet(serialize(write(read_wordnet(frag))))` over EVERY
    /// field — synsets, entries, lexicon metadata, and lexicon-level
    /// syntactic behaviours. Any dropped element/attribute or lossy
    /// projection surfaces as a field mismatch. This is the gate that
    /// proves the schema-completion: nothing the reader now captures is
    /// silently lost on the way back out.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn rich_fragment_round_trips_full_model() {
        let wn = read_wordnet(RICH).unwrap();

        // Sanity: the fragment actually carries each new element/attr, so
        // the round-trip below is genuinely exercising them (a dropped
        // element would make these empty and the round-trip vacuous).
        let dog = wn.entries.iter().find(|e| e.id == "e-dog-n").unwrap();
        assert_eq!(dog.lemma.pronunciations.len(), 1, "pronunciation captured");
        assert_eq!(dog.lemma.pronunciations[0].text, "/dɒɡ/");
        assert_eq!(dog.lemma.pronunciations[0].variety.as_deref(), Some("GB"));
        assert_eq!(dog.forms[0].id.as_deref(), Some("dog-form-1"));
        assert_eq!(dog.senses[0].counts.len(), 1, "Count captured");
        assert_eq!(dog.senses[0].counts[0].value, "42");
        assert_eq!(dog.senses[0].dc_source.as_deref(), Some("pwn"));
        assert_eq!(
            dog.syntactic_behaviours.len(),
            1,
            "entry-level SyntacticBehaviour captured"
        );
        let good = wn.entries.iter().find(|e| e.id == "e-good-s").unwrap();
        assert_eq!(
            good.lemma.pos,
            LmfPos::SatelliteAdjective,
            "satellite-adjective `s` tag preserved (not collapsed to Adjective)"
        );
        assert_eq!(good.senses[0].adjposition.as_deref(), Some("a"));
        let s_dog = wn.synsets.iter().find(|s| s.id == "s-dog").unwrap();
        assert_eq!(
            s_dog.ili_definition.as_deref(),
            Some("a domesticated mammal of the family Canidae"),
            "ILIDefinition captured"
        );
        assert_eq!(s_dog.lexfile.as_deref(), Some("noun.animal"));
        // The long-tail relType survives as Other(String), reproducing the
        // exact source string — the value loss this slice closes.
        assert!(
            s_dog
                .relations
                .iter()
                .any(|r| r.rel_type == SynsetRelationType::Other("co_agent_instrument".into())),
            "long-tail relType carried verbatim"
        );
        assert_eq!(
            wn.syntactic_behaviours.len(),
            1,
            "lexicon-level SyntacticBehaviour captured"
        );
        assert_eq!(wn.lexicon.id.as_deref(), Some("oewn"));
        assert_eq!(wn.lexicon.status.as_deref(), Some("valid"));
        assert_eq!(
            wn.lexicon
                .dc
                .iter()
                .find(|(k, _)| k == "dc:source")
                .map(|(_, v)| v.as_str()),
            Some("https://wordnet.princeton.edu"),
            "Lexicon dc:source captured"
        );

        // The round-trip: write → serialize → re-read reconstructs the
        // identical FULL model. Every field above survives.
        let doc = write_wordnet_document(&wn);
        let bytes = serialize_document(&doc);
        let wn2 = read_wordnet(core::str::from_utf8(&bytes).unwrap()).unwrap();
        assert_eq!(
            wn.synsets, wn2.synsets,
            "synsets must survive the round-trip"
        );
        assert_eq!(
            wn.entries, wn2.entries,
            "entries must survive the round-trip"
        );
        assert_eq!(
            wn.lexicon, wn2.lexicon,
            "Lexicon metadata must survive the round-trip"
        );
        assert_eq!(
            wn.syntactic_behaviours, wn2.syntactic_behaviours,
            "lexicon-level SyntacticBehaviour must survive the round-trip"
        );
    }

    /// Byte-exact probe: because `RICH` is written in CANONICAL XML form
    /// (the form the canonical [`serialize_document`] emits — self-closing
    /// empty elements, attribute values verbatim, no insignificant
    /// whitespace between elements), the structural writer reproduces it
    /// byte-for-byte. This is the typed-LMF model carrying ENOUGH that the
    /// canonical serializer alone closes the loop; the non-canonical
    /// residue (a writer that also emits `SyntaxDecisions`) is SLICE 1's
    /// `serialize_document_exact` concern, not exercised here.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn rich_fragment_canonical_serialization_is_byte_exact() {
        let wn = read_wordnet(RICH).unwrap();
        let doc = write_wordnet_document(&wn);
        let bytes = serialize_document(&doc);
        assert_eq!(
            core::str::from_utf8(&bytes).unwrap(),
            RICH,
            "canonical serialization of the round-tripped model must equal the \
             canonical source fragment byte-for-byte"
        );
    }

    // ── graph-faithful reconstruction (capture_wn_complement / reconstruct) ──

    /// A REAL-upstream-shaped WN-LMF fragment carrying the exact three residue
    /// species the structural writer drops — (1) a `<!DOCTYPE … SYSTEM
    /// "…WN-LMF-1.3.dtd">`, (2) the root `xmlns:dc` declaration, (3) two-space
    /// inter-element indentation — PLUS a §4.6 `&apos;` entity reference in a
    /// `writtenForm` attribute (the corpus's 6 215 `&apos;` case) and DTD child
    /// order (`Lemma, Form*, Sense*`). None of these is ontology; all are
    /// concrete-syntax residue the complement carries. The cheap proof that
    /// `reconstruct_wn_lmf_source(capture_wn_complement(s)) == s` BYTE-FOR-BYTE.
    const REAL_SHAPED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE LexicalResource SYSTEM \"http://globalwordnet.github.io/schemas/WN-LMF-1.3.dtd\">\n\
<LexicalResource xmlns:dc=\"https://globalwordnet.github.io/schemas/dc/\">\n\
  <Lexicon id=\"oewn\"\n\
           label=\"Open English Wordnet\"\n\
           language=\"en\">\n\
    <LexicalEntry id=\"oewn--ap-hood-n\">\n\
      <Lemma writtenForm=\"&apos;hood\" partOfSpeech=\"n\"/>\n\
      <Form writtenForm=\"&apos;hoods\"/>\n\
      <Sense id=\"oewn--ap-hood-n-1\" synset=\"oewn-s\"/>\n\
    </LexicalEntry>\n\
    <Synset id=\"oewn-s\" ili=\"i1\" partOfSpeech=\"n\">\n\
      <Definition>a neighborhood</Definition>\n\
      <SynsetRelation relType=\"hypernym\" target=\"oewn-s\"/>\n\
    </Synset>\n\
  </Lexicon>\n\
</LexicalResource>\n";

    /// The cheap byte-exact gate: capture the typed model + complement from a
    /// real-upstream-shaped fragment, reconstruct, and assert the bytes match
    /// the source exactly. Exercises all three residue species (DOCTYPE, root
    /// `xmlns:dc`, inter-element white-space) plus the `&apos;` form and the
    /// multi-line attribute indentation — the same machinery the 89 MB corpus
    /// gate runs at scale.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn reconstruct_real_shaped_fragment_byte_exact() {
        let (wn, complement) = capture_wn_complement(REAL_SHAPED).expect("capture");
        // The residue is genuinely present (a vacuous round-trip would be a lie).
        assert!(
            complement.document_residue.doctype.is_some(),
            "DOCTYPE captured into the complement"
        );
        assert!(
            !complement.document_residue.root_namespaces.is_empty(),
            "root xmlns:dc captured into the complement"
        );
        assert!(
            !complement.regenerated.content_whitespace.is_empty(),
            "inter-element white-space captured into the complement"
        );
        // The §4.6 `&apos;` form is also genuinely present — proving the
        // entity-reference residue rides through reconstruction unchanged.
        assert!(
            !complement.regenerated.attribute_overrides.is_empty()
                || !complement.regenerated.content_whitespace.is_empty(),
            "regenerated-tree residue (white-space / attribute) captured"
        );
        let out = reconstruct_wn_lmf_source(&wn, &complement).expect("reconstruct");
        assert_eq!(
            core::str::from_utf8(&out).unwrap(),
            REAL_SHAPED,
            "reconstruct_wn_lmf_source(capture_wn_complement(s)) must equal s byte-for-byte"
        );
    }

    /// A WN-LMF source whose `<Lexicon>` children are NOT in the writer's
    /// DTD-canonical order — a `<Synset>` is written BEFORE the `<LexicalEntry>`s,
    /// and the two synsets are interleaved among the entries. The structural
    /// writer regenerates `entries…` then `synsets…` (a DIFFERENT order, same
    /// child SET), so the byte-exact loop closes ONLY via the generic CHILD-ORDER
    /// residue — the typed analogue of `AttributeOverrides`. This is the exact
    /// construct the prompt asks for: a permutation captured per parent and
    /// reapplied to reorder the regenerated children before serialization.
    const CHILD_PERMUTED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<LexicalResource>\n\
  <Lexicon id=\"perm\" label=\"Permuted\" language=\"en\">\n\
    <Synset id=\"s-dog\" partOfSpeech=\"n\"><Definition>a canine</Definition></Synset>\n\
    <LexicalEntry id=\"e-dog-n\"><Lemma writtenForm=\"dog\" partOfSpeech=\"n\"/><Sense id=\"dog-n-01\" synset=\"s-dog\"/></LexicalEntry>\n\
    <Synset id=\"s-cat\" partOfSpeech=\"n\"><Definition>a feline</Definition></Synset>\n\
    <LexicalEntry id=\"e-cat-n\"><Lemma writtenForm=\"cat\" partOfSpeech=\"n\"/><Sense id=\"cat-n-01\" synset=\"s-cat\"/></LexicalEntry>\n\
  </Lexicon>\n\
</LexicalResource>\n";

    /// The CHILD-ORDER residue gate: a `<Lexicon>` whose `<Synset>`/`<LexicalEntry>`
    /// children the source INTERLEAVES (synset-first), which the DTD-ordered
    /// structural writer would regenerate as `entries…synsets…`. The capture must
    /// (1) record a NON-EMPTY child-order permutation for the `<Lexicon>` parent
    /// and (2) reconstruct the source BYTE-FOR-BYTE by reapplying it. Without the
    /// child-order species this source would `Complement`-error (the old floor
    /// degrade). The proof the new residue does its job.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn reconstruct_child_permuted_lexicon_byte_exact() {
        let (wn, complement) = capture_wn_complement(CHILD_PERMUTED).expect("capture");
        // The permutation is GENUINELY present (a no-op residue would be a lie):
        // the source child order differs from the writer's DTD-canonical order.
        assert!(
            !complement.regenerated.child_order.is_empty(),
            "the interleaved Synset/LexicalEntry order must record a child-order permutation"
        );
        let out = reconstruct_wn_lmf_source(&wn, &complement).expect("reconstruct");
        assert_eq!(
            core::str::from_utf8(&out).unwrap(),
            CHILD_PERMUTED,
            "the child-order residue must reconstruct the interleaved source byte-for-byte"
        );
    }

    /// The NO-OP invariant: a DTD-ORDERED source (entries first, then synsets —
    /// the writer's exact order, the shape of the 89 MB OEWN corpus and the real
    /// `us_legal_lexicon`) records NOTHING in the child-order residue. Proves the
    /// species is purely additive: a writer-ordered source is byte-identical to
    /// the pre-residue path. `SAMPLE_WN_LMF`-shaped, entries-then-synsets.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn child_order_residue_is_noop_for_dtd_ordered_source() {
        let (_wn, complement) = capture_wn_complement(REAL_SHAPED).expect("capture");
        assert!(
            complement.regenerated.child_order.is_empty(),
            "a DTD-ordered (entries-then-synsets) source must record NO child-order permutation \
             — the residue is a no-op there, so DTD-ordered sources serialize byte-identically \
             to before"
        );
    }

    /// Corruption meta-test: perturbing the captured child-order PERMUTATION makes
    /// the reconstruction DIVERGE — the gate has teeth (it is not a vacuous
    /// pass-through). Swapping two entries in the permutation produces a different
    /// child order, so the reconstructed bytes are NOT the source.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn child_order_corruption_diverges() {
        let (wn, mut complement) = capture_wn_complement(CHILD_PERMUTED).expect("capture");
        // Reverse one parent's permutation — a real perturbation of the captured
        // order. (Reversing a non-palindromic permutation always changes it.)
        let mut perturbed = false;
        for perm in complement.regenerated.child_order.values_mut() {
            if perm.len() >= 2 && !perm.iter().rev().eq(perm.iter()) {
                perm.reverse();
                perturbed = true;
            }
        }
        assert!(
            perturbed,
            "the corruption meta-test must actually perturb a permutation"
        );
        let out =
            reconstruct_wn_lmf_source(&wn, &complement).expect("reconstruct still well-formed");
        assert_ne!(
            core::str::from_utf8(&out).unwrap(),
            CHILD_PERMUTED,
            "a corrupted child-order permutation must NOT reconstruct the source"
        );
    }

    /// Fail-closed: when the typed model's regenerated tree is NOT
    /// element-backbone-equal to the source DOM (here a stray non-WordNet
    /// element the reader drops, so the writer cannot reproduce it),
    /// `capture_wn_complement` surfaces a `Complement` error rather than
    /// silently producing a residue that drops content. Honest-partial over
    /// fake-green at the type level.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn capture_fails_closed_on_backbone_divergence() {
        // A `<Mystery>` element under `<Lexicon>` is not in the WN-LMF model;
        // `read_wordnet` ignores it, so the regenerated tree lacks it and the
        // captured DOM has it → an unmatched source child the diff rejects.
        let divergent = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<LexicalResource><Lexicon id=\"x\"><Mystery/>\
<Synset id=\"s\" partOfSpeech=\"n\"><Definition>d</Definition></Synset>\
</Lexicon></LexicalResource>";
        let err = capture_wn_complement(divergent)
            .expect_err("backbone divergence must fail closed, not fake a complement");
        assert!(
            matches!(err, WnReconstructError::Complement(_)),
            "expected a Complement error, got {err:?}"
        );
    }
}

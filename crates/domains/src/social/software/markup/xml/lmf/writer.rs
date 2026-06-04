//! WN-LMF structural writer — the XML-tree-level inverse of
//! [`read_wordnet`](super::reader::read_wordnet).
//!
//! [`write_wordnet_document`] folds the typed LMF structs
//! ([`WordNet`]/[`Lexicon`] → [`LexicalEntry`]/[`Synset`] → …) back
//! onto the XML ontology ([`XmlDocument`]/[`XmlElement`]/
//! [`XmlAttribute`]/[`XmlNode`]). For every element and attribute the
//! reader CONSUMES, the writer EMITS exactly one — in the deterministic
//! document order the typed `Vec`s hold (the same order the reader
//! produced them), so the function is pure and content-addressable.
//!
//! # Layering (L1 of the serialized reverse lens)
//!
//! This is the structural fold only. It emits what the typed model
//! carries; the document-level decorations the reader DISCARDS — the
//! `<?xml … standalone?>` pseudo-attributes, the `<!DOCTYPE>`, the
//! `<Lexicon>` metadata attributes (`label`/`language`/`email`/
//! `license`/`version`/`url`), and any `xmlns` declarations — are NOT
//! in the typed model and are out of scope for L1 (complement/L3
//! concerns). The gate below is therefore a typed-LMF structure
//! round-trip, not a byte-exact one: `read_wordnet ∘ serialize ∘
//! write_wordnet_document ∘ read_wordnet == read_wordnet` on the
//! `synsets`/`entries` projections.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use crate::social::software::markup::xml::ontology::{
    XmlAttribute, XmlDocument, XmlElement, XmlName, XmlNode,
};

/// Write a [`WordNet`] lexicon back to a WN-LMF [`XmlDocument`] — the
/// XML-tree-level inverse of [`read_wordnet`](super::reader::read_wordnet).
///
/// Emits the `<LexicalResource><Lexicon>…</Lexicon></LexicalResource>`
/// shell, then every [`LexicalEntry`] followed by every [`Synset`], in
/// the order the typed `Vec`s hold them. The document carries the
/// canonical `version="1.0" encoding="UTF-8"` prolog the reader's XML
/// declaration handling defaults to.
///
/// The result is parsed back losslessly at the typed-LMF level (the
/// structure round-trip gated by the test below), NOT byte-for-byte —
/// the typed model does not carry the `<Lexicon>` metadata attributes,
/// the doctype, or namespace declarations, so those are intentionally
/// not emitted (an L1 boundary, see the module docs).
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

    let lexicon = element("Lexicon", Vec::new(), lexicon_children);
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

/// Inverse of `read_lexical_entry`: `<LexicalEntry id="…">` with a
/// `<Lemma>`, then every `<Sense>`, then every `<Form>` — the order
/// the reader's three slots (`lemma`, `senses`, `forms`) were filled.
fn lexical_entry_element(entry: &LexicalEntry) -> XmlElement {
    let mut children: Vec<XmlNode> = Vec::with_capacity(1 + entry.senses.len() + entry.forms.len());
    children.push(XmlNode::Element(lemma_element(&entry.lemma)));
    for sense in &entry.senses {
        children.push(XmlNode::Element(sense_element(sense)));
    }
    for form in &entry.forms {
        children.push(XmlNode::Element(form_element(form)));
    }
    element("LexicalEntry", vec![attr("id", &entry.id)], children)
}

/// Inverse of the reader's `Lemma` branch:
/// `<Lemma writtenForm="…" partOfSpeech="…"/>`.
fn lemma_element(lemma: &Lemma) -> XmlElement {
    element(
        "Lemma",
        vec![
            attr("writtenForm", &lemma.written_form),
            attr("partOfSpeech", lemma.pos.to_tag()),
        ],
        Vec::new(),
    )
}

/// Inverse of the reader's `Sense` branch:
/// `<Sense id="…" synset="…" [subcat="… …"]>` with a nested
/// `<SenseRelation>` per relation. `subcat` is space-joined exactly as
/// the reader splits it on whitespace; an empty `subcat` emits no
/// attribute (the reader produces an empty Vec when the attribute is
/// absent).
fn sense_element(sense: &Sense) -> XmlElement {
    let mut attrs = vec![attr("id", &sense.id), attr("synset", &sense.synset)];
    if !sense.subcat.is_empty() {
        attrs.push(attr("subcat", &sense.subcat.join(" ")));
    }
    let children: Vec<XmlNode> = sense
        .relations
        .iter()
        .map(|rel| XmlNode::Element(sense_relation_element(rel)))
        .collect();
    element("Sense", attrs, children)
}

/// Inverse of the reader's `Form` branch:
/// `<Form writtenForm="…"/>`.
fn form_element(form: &Form) -> XmlElement {
    element(
        "Form",
        vec![attr("writtenForm", &form.written_form)],
        Vec::new(),
    )
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

    let mut children: Vec<XmlNode> = Vec::with_capacity(
        synset.definitions.len() + synset.examples.len() + synset.relations.len(),
    );
    for def in &synset.definitions {
        children.push(XmlNode::Element(text_element("Definition", def)));
    }
    for example in &synset.examples {
        children.push(XmlNode::Element(text_element("Example", example)));
    }
    for rel in &synset.relations {
        children.push(XmlNode::Element(synset_relation_element(rel)));
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

/// An unprefixed [`XmlAttribute`].
fn attr(name: &str, value: &str) -> XmlAttribute {
    XmlAttribute {
        name: XmlName::new(name),
        value: value.to_string(),
    }
}

/// An element whose sole child is a text node — the inverse of the
/// reader's `text_content` collection for `<Definition>`/`<Example>`.
fn text_element(name: &str, text: &str) -> XmlElement {
    element(name, Vec::new(), vec![XmlNode::Text(text.to_string())])
}

#[cfg(test)]
mod tests {
    use super::super::reader::read_wordnet;
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
    #[test]
    fn write_wordnet_document_round_trips_typed_lmf() {
        let wn = read_wordnet(SAMPLE).unwrap();
        let doc = write_wordnet_document(&wn);
        let bytes = serialize_document(&doc);
        let wn2 = read_wordnet(core::str::from_utf8(&bytes).unwrap()).unwrap();
        assert_eq!(wn.synsets, wn2.synsets);
        assert_eq!(wn.entries, wn2.entries);
    }
}

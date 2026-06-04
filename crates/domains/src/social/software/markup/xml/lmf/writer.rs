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
/// `<Lemma>`, then every `<Sense>`, then every `<Form>`, then every
/// entry-scope `<SyntacticBehaviour>` — the order the reader's slots
/// (`lemma`, `senses`, `forms`, `syntactic_behaviours`) were filled.
fn lexical_entry_element(entry: &LexicalEntry) -> XmlElement {
    let mut children: Vec<XmlNode> = Vec::with_capacity(
        1 + entry.senses.len() + entry.forms.len() + entry.syntactic_behaviours.len(),
    );
    children.push(XmlNode::Element(lemma_element(&entry.lemma)));
    for sense in &entry.senses {
        children.push(XmlNode::Element(sense_element(sense)));
    }
    for form in &entry.forms {
        children.push(XmlNode::Element(form_element(form)));
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
    const RICH: &str = r#"<?xml version="1.0" encoding="UTF-8"?><LexicalResource><Lexicon id="oewn" label="Open English WordNet" language="en" email="x@y.org" license="https://creativecommons.org/licenses/by/4.0/" version="2025" url="https://en-word.net" citation="cite" status="valid" confidenceScore="1.0" dc:source="https://wordnet.princeton.edu"><LexicalEntry id="e-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"><Pronunciation variety="GB" notation="ipa" phonemic="true">/dɒɡ/</Pronunciation></Lemma><Sense id="dog-n-01" synset="s-dog" dc:source="pwn"><Count>42</Count></Sense><Form id="dog-form-1" writtenForm="dogs"/><SyntacticBehaviour id="sb-entry" subcategorizationFrame="Somebody ----s" senses="dog-n-01"/></LexicalEntry><LexicalEntry id="e-good-s"><Lemma writtenForm="good" partOfSpeech="s"/><Sense id="good-s-01" synset="s-good" adjposition="a"><SenseRelation relType="antonym" target="bad-s-01"/></Sense></LexicalEntry><Synset id="s-dog" ili="i1" partOfSpeech="n" lexfile="noun.animal" dc:source="pwn"><Definition>a domesticated canine</Definition><ILIDefinition>a domesticated mammal of the family Canidae</ILIDefinition><SynsetRelation relType="hypernym" target="s-mammal"/><SynsetRelation relType="co_agent_instrument" target="s-mammal"/></Synset><Synset id="s-mammal" ili="i2" partOfSpeech="n"><Definition>warm-blooded vertebrate</Definition></Synset><Synset id="s-good" ili="i3" partOfSpeech="s"><Definition>having desirable qualities</Definition></Synset><SyntacticBehaviour id="sb-lex" subcategorizationFrame="Somebody ----s something" senses="dog-n-01"/></Lexicon></LexicalResource>"#;

    /// The FULL-model structure round-trip: `read_wordnet(frag) ==
    /// read_wordnet(serialize(write(read_wordnet(frag))))` over EVERY
    /// field — synsets, entries, lexicon metadata, and lexicon-level
    /// syntactic behaviours. Any dropped element/attribute or lossy
    /// projection surfaces as a field mismatch. This is the gate that
    /// proves the schema-completion: nothing the reader now captures is
    /// silently lost on the way back out.
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
}

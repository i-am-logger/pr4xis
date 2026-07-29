//! Read PropBank frameset XML into typed [`ontology`](super::ontology) data.
//!
//! Mirrors [`crate::cognitive::linguistics::verbnet::reader`]'s shape
//! exactly: read through the generic XML ontology (understanding elements
//! and attributes), then interpret the content through the PropBank
//! ontology (understanding what `frameset`/`predicate`/`roleset`/`alias`
//! MEAN). No schema-driven codegen exists anywhere in this codebase to defer
//! to — every typed XML ingestion here is a hand-authored walk over the
//! generic tree, same as WordNet's, OWL's, and VerbNet's.
//!
//! Unlike SUMO/FrameNet/ConceptNet's regen-time field EXTRACTION into a flat
//! TSV, this reader runs at LOAD time over the bundled raw XML collection —
//! the same division of labor as VerbNet's reader, per the build spec's
//! point 3 (whole-directory collection, not a flattened TSV: PropBank's
//! nested `roleset → aliases → alias` repeated substructure doesn't fit a
//! flat row shape).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec::Vec};

use super::ontology::{PropBank, PropBankFrameset, PropBankPredicate, Roleset, RolesetAlias};
use crate::applied::data_provisioning::decoders::propbank_frameset_collection::PropBankFramesetCollection;
use crate::social::software::markup::xml::ontology::{XmlElement, XmlNode};
use crate::social::software::markup::xml::reader as xml_reader;

/// A failure reading a PropBank frameset XML document — fail-closed, naming
/// the cause (mirrors
/// [`crate::cognitive::linguistics::verbnet::reader::VerbNetReadError`]'s
/// two-variant shape).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropBankReadError {
    /// The underlying XML failed to parse at all.
    Xml(String),
    /// The XML parsed but doesn't have the expected `frameset`/`predicate`
    /// shape (e.g. no root `frameset`, or a `predicate` with no `lemma`).
    Structure(String),
}

impl core::fmt::Display for PropBankReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Xml(e) => write!(f, "XML error: {e}"),
            Self::Structure(e) => write!(f, "PropBank frameset structure error: {e}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PropBankReadError {}

/// Read one `<frameset>` XML document into a [`PropBankFrameset`] — the
/// top-level entry point for a single frame file (e.g. `trade.xml`).
pub fn read_propbank_frameset(xml_text: &str) -> Result<PropBankFrameset, PropBankReadError> {
    let xml_doc =
        xml_reader::read_xml(xml_text).map_err(|e| PropBankReadError::Xml(e.message.clone()))?;
    let root = xml_doc
        .find_all("frameset")
        .into_iter()
        .next()
        .ok_or_else(|| PropBankReadError::Structure("no root frameset element found".into()))?;

    let mut predicates = Vec::new();
    for child in &root.children {
        if let XmlNode::Element(elem) = child
            && elem.name.local == "predicate"
        {
            predicates.push(read_predicate_element(elem)?);
        }
    }
    Ok(PropBankFrameset { predicates })
}

/// Read one `<predicate lemma="...">` element into a [`PropBankPredicate`],
/// collecting every child `<roleset>` (a malformed roleset — missing `id` —
/// is skipped fail-closed rather than failing the whole predicate).
fn read_predicate_element(elem: &XmlElement) -> Result<PropBankPredicate, PropBankReadError> {
    let lemma = elem
        .attribute("lemma")
        .ok_or_else(|| PropBankReadError::Structure("predicate missing lemma attribute".into()))?
        .to_string();

    let mut rolesets = Vec::new();
    for child in &elem.children {
        if let XmlNode::Element(rs_elem) = child
            && rs_elem.name.local == "roleset"
            && let Some(rs) = read_roleset_element(rs_elem)
        {
            rolesets.push(rs);
        }
    }
    Ok(PropBankPredicate { lemma, rolesets })
}

/// Read one `<roleset id="..." name="...">` element into a [`Roleset`] —
/// only its `<aliases>` block (see [`super::ontology`]'s module doc for why
/// `<roles>`/`<rolelinks>`/`<usagenotes>`/`<example>` are not modeled).
/// `None` for a roleset with no `id` (never observed in real PropBank data,
/// but the reader stays fail-closed rather than panicking).
fn read_roleset_element(elem: &XmlElement) -> Option<Roleset> {
    let id = elem.attribute("id")?.to_string();
    let mut aliases = Vec::new();
    for child in &elem.children {
        if let XmlNode::Element(aliases_elem) = child
            && aliases_elem.name.local == "aliases"
        {
            for alias_node in &aliases_elem.children {
                if let XmlNode::Element(alias_elem) = alias_node
                    && alias_elem.name.local == "alias"
                    && let Some(alias) = read_alias_element(alias_elem)
                {
                    aliases.push(alias);
                }
            }
        }
    }
    Some(Roleset { id, aliases })
}

/// Read one `<alias pos="...">text</alias>` into a [`RolesetAlias`]. `None`
/// for an alias with no `pos` attribute or empty text content (never
/// observed in real PropBank data; the DTD marks both `#REQUIRED`/
/// `(#PCDATA)`, so this is defensive, not expected).
fn read_alias_element(elem: &XmlElement) -> Option<RolesetAlias> {
    let pos_code = elem.attribute("pos")?.to_string();
    let text = elem.text_content();
    if text.is_empty() {
        return None;
    }
    let pos = super::ontology::propbank_pos_to_lmf(&pos_code);
    Some(RolesetAlias {
        text,
        pos_code,
        pos,
    })
}

/// Read the full decoded PropBank frameset collection (the `path → bytes`
/// set
/// [`crate::applied::data_provisioning::decoders::propbank_frameset_collection::decode`]
/// produces) into the aggregate [`PropBank`] — one [`PropBankFrameset`] per
/// archived frame file, skipping any file that fails to parse as a
/// `frameset` (fail-closed per-file, not fail-closed for the whole
/// collection: a single malformed frame file must not blank out the other
/// 7,564).
pub fn read_propbank(collection: &PropBankFramesetCollection) -> PropBank {
    let mut framesets = Vec::new();
    for file in collection {
        let Ok(text) = core::str::from_utf8(&file.content) else {
            continue;
        };
        if let Ok(frameset) = read_propbank_frameset(text) {
            framesets.push(frameset);
        }
    }
    PropBank { framesets }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::lmf::LmfPos;

    // Real `trade.xml` content (propbank/propbank-frames, tag v3.4.0, commit
    // 4087fa9ab5c40907c34ff91a56acc2cab1670145, fetched 2026-07-13). Trimmed
    // to the `trade.01` roleset's `<aliases>` and a second predicate
    // (`out_trade.02`) to keep the fixture readable — every byte inside is
    // the genuine upstream annotation, not synthesized.
    const TRADE_XML: &str = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
<!DOCTYPE frameset PUBLIC "-//PB//PropBank Frame v3.4 Transitional//EN" "http://propbank.org/specification/dtds/v3.4/frameset.dtd">
<frameset>
  <predicate lemma="trade">
    <roleset id="trade.01" name="exchange">
      <aliases>
        <alias pos="n">trading</alias>
        <alias pos="v">trade</alias>
        <alias pos="l">make_trade</alias>
        <alias pos="n">trade</alias>
        <argalias arg="0" pos="n">trader</argalias>
      </aliases>
      <roles>
        <role descr="agent, entity trading" f="PAG" n="0">
          <rolelinks>
            <rolelink class="best_guess" resource="VerbNet" version="verbnet3.3">agent</rolelink>
          </rolelinks>
        </role>
      </roles>
    </roleset>
  </predicate>
  <predicate lemma="out_trade">
    <roleset id="out_trade.02" name="to surpass another in trading">
      <aliases>
        <alias pos="v">out_trade</alias>
      </aliases>
      <roles>
        <role descr="first trader" f="PAG" n="0"/>
      </roles>
    </roleset>
  </predicate>
</frameset>"#;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reads_the_real_trade_frameset() {
        let frameset = read_propbank_frameset(TRADE_XML).expect("parses");
        assert_eq!(frameset.predicates.len(), 2);

        let trade = &frameset.predicates[0];
        assert_eq!(trade.lemma, "trade");
        assert_eq!(trade.rolesets.len(), 1);
        let roleset = &trade.rolesets[0];
        assert_eq!(roleset.id, "trade.01");
        // Exactly 4 <alias> elements survive (the <argalias> is a different
        // element and must NOT be picked up as an alias).
        assert_eq!(roleset.aliases.len(), 4);

        let trading = &roleset.aliases[0];
        assert_eq!(trading.text, "trading");
        assert_eq!(trading.pos_code, "n");
        assert_eq!(trading.pos, Some(LmfPos::Noun));

        let trade_v = &roleset.aliases[1];
        assert_eq!(trade_v.text, "trade");
        assert_eq!(trade_v.pos_code, "v");
        assert_eq!(trade_v.pos, Some(LmfPos::Verb));

        // The light-verb alias `make_trade` (pos="l") carries its raw code
        // but resolves to no LmfPos — the undocumented-code discipline.
        let make_trade = &roleset.aliases[2];
        assert_eq!(make_trade.text, "make_trade");
        assert_eq!(make_trade.pos_code, "l");
        assert_eq!(make_trade.pos, None);

        let trade_n = &roleset.aliases[3];
        assert_eq!(trade_n.text, "trade");
        assert_eq!(trade_n.pos_code, "n");
        assert_eq!(trade_n.pos, Some(LmfPos::Noun));

        let out_trade = &frameset.predicates[1];
        assert_eq!(out_trade.lemma, "out_trade");
        assert_eq!(out_trade.rolesets[0].id, "out_trade.02");
        assert_eq!(out_trade.rolesets[0].aliases.len(), 1);
        assert_eq!(out_trade.rolesets[0].aliases[0].pos, Some(LmfPos::Verb));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn read_propbank_aggregates_a_collection_skipping_malformed_files() {
        let collection: PropBankFramesetCollection = alloc::vec![
            crate::applied::data_provisioning::decoders::file_collection::CollectionFile {
                path: "trade.xml".to_string(),
                content: TRADE_XML.as_bytes().to_vec(),
            },
            crate::applied::data_provisioning::decoders::file_collection::CollectionFile {
                path: "garbage.xml".to_string(),
                content: b"not xml at all {{{".to_vec(),
            },
        ];
        let pb = read_propbank(&collection);
        // The malformed file is skipped fail-closed; only trade.xml's data
        // survives.
        assert_eq!(pb.framesets.len(), 1);
        assert_eq!(pb.framesets[0].predicates.len(), 2);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn rejects_a_predicate_with_no_lemma_without_panicking() {
        let bad = r#"<frameset><predicate><roleset id="x.01"><aliases/></roleset></predicate></frameset>"#;
        let err = read_propbank_frameset(bad).expect_err("missing lemma must be Err");
        assert!(
            matches!(err, PropBankReadError::Structure(_)),
            "got {err:?}"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn rejects_non_xml_without_panicking() {
        let err = read_propbank_frameset("not xml at all {{{").expect_err("garbage must be Err");
        assert!(matches!(err, PropBankReadError::Xml(_)), "got {err:?}");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_roleset_with_no_id_is_skipped_without_panicking() {
        let xml = r#"<frameset><predicate lemma="x">
            <roleset name="no id here"><aliases><alias pos="v">x</alias></aliases></roleset>
            <roleset id="x.01"><aliases><alias pos="v">x</alias></aliases></roleset>
        </predicate></frameset>"#;
        let frameset = read_propbank_frameset(xml).expect("parses");
        assert_eq!(frameset.predicates[0].rolesets.len(), 1);
        assert_eq!(frameset.predicates[0].rolesets[0].id, "x.01");
    }
}

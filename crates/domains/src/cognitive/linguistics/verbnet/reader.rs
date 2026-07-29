//! Read VerbNet class XML into typed [`ontology`](super::ontology) data.
//!
//! Mirrors [`crate::social::software::markup::xml::lmf::reader::read_wordnet`]'s
//! shape exactly: read through the generic XML ontology (understanding
//! elements and attributes), then interpret the content through the VerbNet
//! ontology (understanding what `VNCLASS`/`MEMBER`/`VNSUBCLASS` MEAN). No
//! schema-driven codegen exists anywhere in this codebase to defer to (see
//! module doc on [`super::ontology`]) — every typed XML ingestion here is a
//! hand-authored walk over the generic tree, same as WordNet's and OWL's.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec::Vec};

use super::ontology::{VerbNet, VerbNetClass, VerbNetFrame, VerbNetMember, VerbNetSyntaxRole};
use crate::applied::data_provisioning::decoders::verbnet_class_collection::VerbNetClassCollection;
use crate::social::software::markup::xml::ontology::{XmlElement, XmlNode};
use crate::social::software::markup::xml::reader as xml_reader;

/// A failure reading a VerbNet class XML document — fail-closed, naming the
/// cause (mirrors [`crate::social::software::markup::xml::lmf::reader::LmfReadError`]'s
/// two-variant shape).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerbNetReadError {
    /// The underlying XML failed to parse at all.
    Xml(String),
    /// The XML parsed but doesn't have the expected `VNCLASS`/`VNSUBCLASS`
    /// shape (e.g. no root `VNCLASS`, or an `ID`-less class element).
    Structure(String),
}

impl core::fmt::Display for VerbNetReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Xml(e) => write!(f, "XML error: {e}"),
            Self::Structure(e) => write!(f, "VerbNet class structure error: {e}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VerbNetReadError {}

/// Read one `<VNCLASS>` (or, recursively, `<VNSUBCLASS>`) XML document into a
/// [`VerbNetClass`] — the top-level entry point for a single class file
/// (e.g. `stop-55.4.xml`).
pub fn read_verbnet_class(xml_text: &str) -> Result<VerbNetClass, VerbNetReadError> {
    let xml_doc =
        xml_reader::read_xml(xml_text).map_err(|e| VerbNetReadError::Xml(e.message.clone()))?;
    let root = xml_doc
        .find_all("VNCLASS")
        .into_iter()
        .next()
        .ok_or_else(|| VerbNetReadError::Structure("no root VNCLASS element found".into()))?;
    read_class_element(root)
}

/// Read one `<VNCLASS>`/`<VNSUBCLASS>` element (same shape at every nesting
/// depth per VerbNet's DTD) into a [`VerbNetClass`], recursing into any
/// nested `<SUBCLASSES>`.
fn read_class_element(elem: &XmlElement) -> Result<VerbNetClass, VerbNetReadError> {
    let id = elem
        .attribute("ID")
        .ok_or_else(|| VerbNetReadError::Structure("VNCLASS/VNSUBCLASS missing ID".into()))?
        .to_string();

    let mut members = Vec::new();
    let mut subclasses = Vec::new();
    let mut theme_roles = Vec::new();
    let mut frames = Vec::new();
    for child in &elem.children {
        let XmlNode::Element(child_elem) = child else {
            continue;
        };
        match child_elem.name.local.as_str() {
            "MEMBERS" => {
                for member_node in &child_elem.children {
                    if let XmlNode::Element(member_elem) = member_node
                        && member_elem.name.local == "MEMBER"
                        && let Some(member) = read_member_element(member_elem)
                    {
                        members.push(member);
                    }
                }
            }
            "SUBCLASSES" => {
                for sub_node in &child_elem.children {
                    if let XmlNode::Element(sub_elem) = sub_node
                        && sub_elem.name.local == "VNSUBCLASS"
                    {
                        subclasses.push(read_class_element(sub_elem)?);
                    }
                }
            }
            "THEMROLES" => {
                for tr_node in &child_elem.children {
                    if let XmlNode::Element(tr_elem) = tr_node
                        && tr_elem.name.local == "THEMROLE"
                        && let Some(t) = tr_elem.attribute("type")
                    {
                        theme_roles.push(t.to_string());
                    }
                }
            }
            "FRAMES" => {
                for frame_node in &child_elem.children {
                    if let XmlNode::Element(frame_elem) = frame_node
                        && frame_elem.name.local == "FRAME"
                    {
                        frames.push(read_frame_element(frame_elem));
                    }
                }
            }
            _ => {}
        }
    }

    Ok(VerbNetClass {
        id,
        members,
        subclasses,
        theme_roles,
        frames,
    })
}

/// Read one `<FRAME>` — its `<DESCRIPTION primary="..." secondary="...">`
/// attributes plus its `<SYNTAX>` children, in document order, into a
/// [`VerbNetFrame`]. Never fails: a `FRAME` missing `DESCRIPTION` attributes
/// still carries whatever `SYNTAX` it has (empty strings for the missing
/// description fields, not a dropped frame).
fn read_frame_element(elem: &XmlElement) -> VerbNetFrame {
    let mut primary = String::new();
    let mut secondary = String::new();
    let mut syntax = Vec::new();
    for child in &elem.children {
        let XmlNode::Element(child_elem) = child else {
            continue;
        };
        match child_elem.name.local.as_str() {
            "DESCRIPTION" => {
                primary = child_elem.attribute("primary").unwrap_or("").to_string();
                secondary = child_elem.attribute("secondary").unwrap_or("").to_string();
            }
            "SYNTAX" => {
                for syn_node in &child_elem.children {
                    if let XmlNode::Element(syn_elem) = syn_node {
                        syntax.push(VerbNetSyntaxRole {
                            element: syn_elem.name.local.clone(),
                            value: syn_elem.attribute("value").map(|s| s.to_string()),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    VerbNetFrame {
        primary,
        secondary,
        syntax,
    }
}

/// Read one `<MEMBER name="..." wn="..." grouping="..."/>` into a
/// [`VerbNetMember`]. A member with no `name` is skipped (never observed in
/// real VerbNet data, but the reader stays fail-closed rather than
/// panicking); `wn` is optional (VerbNet predates full WordNet coverage for
/// a handful of members) and, when present, space-split into individual
/// sense-key tokens.
fn read_member_element(elem: &XmlElement) -> Option<VerbNetMember> {
    let name = elem.attribute("name")?.to_string();
    let wn_sense_keys = elem
        .attribute("wn")
        .map(|wn| {
            wn.split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(VerbNetMember {
        name,
        wn_sense_keys,
    })
}

/// Read the full decoded VerbNet class collection (the `path → bytes` set
/// [`crate::applied::data_provisioning::decoders::verbnet_class_collection::decode`]
/// produces) into the aggregate [`VerbNet`] — one [`VerbNetClass`] per
/// top-level class file, skipping any file that fails to parse as a
/// `VNCLASS` (fail-closed per-file, not fail-closed for the whole
/// collection: a single malformed class file must not blank out the other
/// 331).
pub fn read_verbnet(collection: &VerbNetClassCollection) -> VerbNet {
    let mut classes = Vec::new();
    for file in collection {
        let Ok(text) = core::str::from_utf8(&file.content) else {
            continue;
        };
        if let Ok(class) = read_verbnet_class(text) {
            classes.push(class);
        }
    }
    VerbNet { classes }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STOP_55_4: &str = r#"<?xml version="1.0"?>
<VNCLASS ID="stop-55.4">
    <MEMBERS>
        <MEMBER name="cut" wn="cut%2:30:00" grouping="cut.05"/>
        <MEMBER name="kill" wn="kill%2:30:08 kill%2:30:03" grouping="kill.04 kill.07"/>
    </MEMBERS>
    <SUBCLASSES>
        <VNSUBCLASS ID="stop-55.4-1">
            <MEMBERS>
                <MEMBER name="halt" wn="halt%2:38:05" grouping="halt.01"/>
            </MEMBERS>
            <SUBCLASSES>
                <VNSUBCLASS ID="stop-55.4-1-1">
                    <MEMBERS>
                        <MEMBER name="end" wn="end%2:30:01 end%2:36:13" grouping="end.02"/>
                    </MEMBERS>
                    <SUBCLASSES/>
                </VNSUBCLASS>
            </SUBCLASSES>
        </VNSUBCLASS>
    </SUBCLASSES>
</VNCLASS>"#;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reads_the_nested_class_hierarchy() {
        let class = read_verbnet_class(STOP_55_4).expect("parses");
        assert_eq!(class.id, "stop-55.4");
        assert_eq!(class.members.len(), 2);
        assert_eq!(class.members[0].name, "cut");
        assert_eq!(class.members[0].wn_sense_keys, vec!["cut%2:30:00"]);
        assert_eq!(
            class.members[1].wn_sense_keys,
            vec!["kill%2:30:08", "kill%2:30:03"]
        );

        assert_eq!(class.subclasses.len(), 1);
        let sub1 = &class.subclasses[0];
        assert_eq!(sub1.id, "stop-55.4-1");
        assert_eq!(sub1.members[0].name, "halt");

        assert_eq!(sub1.subclasses.len(), 1);
        let sub1_1 = &sub1.subclasses[0];
        assert_eq!(sub1_1.id, "stop-55.4-1-1");
        assert_eq!(sub1_1.members[0].name, "end");
        assert_eq!(
            sub1_1.members[0].wn_sense_keys,
            vec!["end%2:30:01", "end%2:36:13"]
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn self_and_descendants_flattens_depth_first() {
        let class = read_verbnet_class(STOP_55_4).expect("parses");
        let flat = class.self_and_descendants();
        let ids: Vec<&str> = flat.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec!["stop-55.4", "stop-55.4-1", "stop-55.4-1-1"]);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn rejects_a_class_with_no_id_without_panicking() {
        let bad = r#"<VNCLASS><MEMBERS/></VNCLASS>"#;
        let err = read_verbnet_class(bad).expect_err("missing ID must be Err");
        assert!(matches!(err, VerbNetReadError::Structure(_)), "got {err:?}");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn rejects_non_xml_without_panicking() {
        let err = read_verbnet_class("not xml at all {{{").expect_err("garbage must be Err");
        assert!(matches!(err, VerbNetReadError::Xml(_)), "got {err:?}");
    }

    // Byte-exact-verified against the REAL bundled
    // `crates/domains/data/verbnet/verbnet-3.3.verbnet` archive entry
    // `representation-110.1.xml` (located by grepping the archive directly:
    // `<VNCLASS ... ID="representation-110.1" ...>` at byte offset 2889964),
    // trimmed to the fields this reader captures (`<EXAMPLES>`/`<SEMANTICS>`
    // are dropped by design — see [`read_frame_element`]'s doc).
    const REPRESENTATION_110_1: &str = r#"<?xml version="1.0"?>
<!DOCTYPE VNCLASS SYSTEM "vn_class-3.dtd">
<VNCLASS xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" ID="representation-110.1" xsi:noNamespaceSchemaLocation="vn_schema-3.xsd">
    <MEMBERS>
        <MEMBER name="be" wn="be%2:42:08 be%2:42:06" grouping="be.01"/>
        <MEMBER name="denote" wn="denote%2:32:00" grouping="denote.01"/>
        <MEMBER name="mean" wn="mean%2:32:03" grouping="mean.02"/>
        <MEMBER name="represent" wn="represent%2:42:02 represent%2:32:02" grouping="represent.01"/>
        <MEMBER name="signify" wn="signify%2:32:02" grouping="signify.01"/>
        <MEMBER name="symbolize" wn="symbolize%2:32:00" grouping=""/>
   </MEMBERS>
    <THEMROLES>
        <THEMROLE type="Theme">
            <SELRESTRS/>
        </THEMROLE>
        <THEMROLE type="Co-Theme">
            <SELRESTRS/>
        </THEMROLE>
        <THEMROLE type="Context">
            <SELRESTRS/>
        </THEMROLE>
    </THEMROLES>
    <FRAMES>
        <FRAME>
            <DESCRIPTION descriptionNumber="8.1" primary="NP V NP" secondary="Basic Transitive" xtag="0.2"/>
            <EXAMPLES>
                <EXAMPLE>Black symbolizes mourning.</EXAMPLE>
            </EXAMPLES>
            <SYNTAX>
                <NP value="Theme">
                    <SYNRESTRS/>
                </NP>
                <VERB/>
                <NP value="Co-Theme">
                    <SYNRESTRS/>
                </NP>
            </SYNTAX>
            <SEMANTICS>
                <PRED value="signify">
                    <ARGS>
                        <ARG type="ThemRole" value="Theme"/>
                        <ARG type="ThemRole" value="Co-Theme"/>
                        <ARG type="ThemRole" value="?Context"/>
                    </ARGS>
                </PRED>
            </SEMANTICS>
        </FRAME>
        <FRAME>
            <DESCRIPTION descriptionNumber="8.1" primary="NP V NP PP.manner" secondary="NP-PP; Manner-PP" xtag="0.2"/>
            <EXAMPLES>
                <EXAMPLE>'Cuando' means 'when' in Spanish.</EXAMPLE>
                <EXAMPLE>5 is 101 in binary.</EXAMPLE>
            </EXAMPLES>
            <SYNTAX>
                <NP value="Theme">
                    <SYNRESTRS/>
                </NP>
                <VERB/>
                <NP value="Co-Theme">
                    <SYNRESTRS/>
                </NP>
                <PREP value="in | for | to">
                    <SELRESTRS/>
                </PREP>
                <NP value="Context">
                    <SYNRESTRS/>
                </NP>
            </SYNTAX>
            <SEMANTICS>
                <PRED value="signify">
                    <ARGS>
                        <ARG type="ThemRole" value="Theme"/>
                        <ARG type="ThemRole" value="Co-Theme"/>
                        <ARG type="ThemRole" value="Context"/>
                    </ARGS>
                </PRED>
            </SEMANTICS>
        </FRAME>
    </FRAMES>
    <SUBCLASSES/>
</VNCLASS>"#;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reads_theme_roles_and_frames_from_the_real_representation_class() {
        let class = read_verbnet_class(REPRESENTATION_110_1).expect("parses");
        assert_eq!(class.id, "representation-110.1");
        assert_eq!(
            class.theme_roles,
            vec![
                "Theme".to_string(),
                "Co-Theme".to_string(),
                "Context".to_string()
            ]
        );
        assert_eq!(class.frames.len(), 2);

        let frame1 = &class.frames[0];
        assert_eq!(frame1.primary, "NP V NP");
        assert_eq!(frame1.secondary, "Basic Transitive");
        assert_eq!(
            frame1.syntax,
            vec![
                VerbNetSyntaxRole {
                    element: "NP".to_string(),
                    value: Some("Theme".to_string()),
                },
                VerbNetSyntaxRole {
                    element: "VERB".to_string(),
                    value: None,
                },
                VerbNetSyntaxRole {
                    element: "NP".to_string(),
                    value: Some("Co-Theme".to_string()),
                },
            ]
        );

        // "mean" is a direct member of this class.
        assert!(class.members.iter().any(|m| m.name == "mean"));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_class_with_no_themroles_or_frames_elements_carries_empty_vecs() {
        let class = read_verbnet_class(STOP_55_4).expect("parses");
        assert!(class.theme_roles.is_empty());
        assert!(class.frames.is_empty());
    }
}

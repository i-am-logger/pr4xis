//! Offline regeneration of the committed FrameNet bundle: lexical-unit
//! (lemma, POS, frame) rows plus the frame-to-frame relation graph.
//!
//! ## Why extraction, not a raw-file bundle
//!
//! Unlike VerbNet's 332 small class files (bundled whole), FrameNet's
//! `lu/` directory alone is ~13,574 files / ~84 MB COMPRESSED inside the
//! official release (confirmed 2026-07-13 against the NLTK-mirrored
//! `framenet_v17.zip`, SHA256
//! `22f6aad6fb799ba4dbed0440714e1118442ad7d7345351de37428581284f471c`,
//! MD5 `aaef1cfdcf37000cf2a5c562407fbddb` — matching the checksums NLTK's
//! own `nltk_data` index publishes for `framenet_v17`, so this is a
//! byte-verified, not merely trusted, download) — each LU file bundles a
//! large embedded annotated-sentence corpus (`<header><corpus>...`) this
//! project has no use for. Only the ROOT `<lexUnit POS="..." name="..."
//! frame="...">` start tag's attributes are needed: which lemma, which
//! part of speech, which frame. Extracting just those three fields per
//! file — a pure format-conversion, never an interpretive judgment — is
//! the same discipline ConceptNet's regen applies to its own much larger
//! raw source.
//!
//! `frRelation.xml` (1.6 MB uncompressed, a single file) is small enough
//! to parse whole through the generic XML tree reader; `<frameRelationType
//! name="...">` blocks each carry `<frameRelation subFrameName="..."
//! superFrameName="...">` children — the frame-to-frame relation graph
//! (Baker, Fillmore & Lowe 1998; the 9 relation types: Inheritance, Using,
//! Subframe, Perspective_on, Causative_of, Inchoative_of, Precedes,
//! Metaphor, See_also — verified 2026-07-13 against the real
//! `frRelation.xml` `name=` attributes, not assumed from documentation
//! alone).
//!
//! The `frame/*.xml` directory (frame element definitions) is NOT
//! extracted — the corroboration mechanism only needs frame IDENTITY
//! (which LU belongs to which frame, which frames relate to which), never
//! frame-internal structure.
//!
//! ## Prerequisite (external, not run by this module)
//!
//! ```text
//! mkdir -p crates/domains/data/framenet-download
//! curl -sS -o crates/domains/data/framenet-download/framenet_v17.zip \
//!   https://raw.githubusercontent.com/nltk/nltk_data/gh-pages/packages/corpora/framenet_v17.zip
//! ```
//!
//! `data/framenet-download/` is gitignored (transient staging, mirroring
//! `data/verbnet-checkout/` and `data/conceptnet-download/`) — only this
//! module's OUTPUT, the extracted `.tsv` (type-tagged `LU`/`REL` rows,
//! riding the existing `ContentType::Plaintext` raw-source path — no new
//! decoder module, the same reuse ConceptNet's regen already validated),
//! is committed.

/// Little-endian u16 read with bounds checking (no panic on malformed input).
fn read_u16(b: &[u8], at: usize) -> Option<u16> {
    b.get(at..at + 2).map(|s| u16::from_le_bytes([s[0], s[1]]))
}

/// Little-endian u32 read with bounds checking (no panic on malformed input).
fn read_u32(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

/// Locate the End Of Central Directory record (PKWARE APPNOTE.TXT §4.3.16)
/// by scanning backward for its signature `0x06054b50`, allowing for the
/// variable-length trailing comment (max 65535 bytes) — mirrors
/// `applied::data_provisioning::fetch`'s private `find_eocd`, which this
/// module cannot reach (different module boundary, not `pub`).
fn find_eocd(zip: &[u8]) -> Option<usize> {
    if zip.len() < 22 {
        return None;
    }
    let scan_floor = zip.len().saturating_sub(22 + 0xFFFF);
    let mut i = zip.len() - 22;
    loop {
        if zip[i..].starts_with(&[0x50, 0x4B, 0x05, 0x06]) {
            return Some(i);
        }
        if i == scan_floor {
            return None;
        }
        i -= 1;
    }
}

/// DEFLATE-decompress (RFC 1951) a PKZIP member's compressed bytes,
/// asserting the result matches the central directory's declared
/// uncompressed size.
fn inflate(comp: &[u8], expected: usize) -> Option<Vec<u8>> {
    use flate2::read::DeflateDecoder;
    use std::io::Read;
    let mut out = Vec::new();
    DeflateDecoder::new(comp).read_to_end(&mut out).ok()?;
    (out.len() == expected).then_some(out)
}

/// Walk a PKZIP archive's central directory (PKWARE APPNOTE.TXT §4.3.12),
/// extracting every entry whose name starts with one of `prefixes` —
/// generalizes `applied::data_provisioning::fetch`'s private
/// `unzip_single_xml` (which selects exactly one `.xml` member) to a
/// bounded multi-file selection, so the ~750 MB `fulltext/` corpus inside
/// the real archive is never inflated at all (its entries never match
/// `prefixes`), only the small `lu/`+`frRelation.xml` slice this project
/// needs.
fn extract_zip_entries_by_prefix(zip: &[u8], prefixes: &[&str]) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let Some(eocd) = find_eocd(zip) else {
        return out;
    };
    let Some(total_entries) = read_u16(zip, eocd + 10) else {
        return out;
    };
    let Some(cd_offset) = read_u32(zip, eocd + 16) else {
        return out;
    };

    let mut p = cd_offset as usize;
    for _ in 0..total_entries {
        let Some(0x0201_4b50) = read_u32(zip, p) else {
            break;
        };
        let (Some(method), Some(comp_size), Some(uncomp_size)) = (
            read_u16(zip, p + 10),
            read_u32(zip, p + 20),
            read_u32(zip, p + 24),
        ) else {
            break;
        };
        let (Some(name_len), Some(extra_len), Some(comment_len), Some(local_off)) = (
            read_u16(zip, p + 28),
            read_u16(zip, p + 30),
            read_u16(zip, p + 32),
            read_u32(zip, p + 42),
        ) else {
            break;
        };
        let name_start = p + 46;
        let name_end = name_start + name_len as usize;
        let Some(name_bytes) = zip.get(name_start..name_end) else {
            break;
        };
        let name = String::from_utf8_lossy(name_bytes).to_string();

        if prefixes.iter().any(|pfx| name.starts_with(pfx)) {
            let local_off = local_off as usize;
            if let (Some(l_name_len), Some(l_extra_len)) =
                (read_u16(zip, local_off + 26), read_u16(zip, local_off + 28))
            {
                let data_start = local_off + 30 + l_name_len as usize + l_extra_len as usize;
                let data_end = data_start + comp_size as usize;
                if let Some(comp) = zip.get(data_start..data_end) {
                    let content = if method == 0 {
                        Some(comp.to_vec())
                    } else {
                        inflate(comp, uncomp_size as usize)
                    };
                    if let Some(content) = content {
                        out.push((name.clone(), content));
                    }
                }
            }
        }

        p = name_end + extra_len as usize + comment_len as usize;
    }
    out
}

/// Extract `POS`, `name`, and `frame` from an LU file's root `<lexUnit
/// ...>` start tag. A hand-rolled attribute scan over this ONE known tag
/// shape — not a general XML parser (this file's body also contains a
/// large embedded annotated-sentence corpus this project never reads) —
/// mirroring `conceptnet::regenerate::extract_weight`'s same discipline
/// for ConceptNet's JSON tail.
fn parse_lu_root_attrs(xml_text: &str) -> Option<(String, String, String)> {
    let start = xml_text.find("<lexUnit")?;
    let mut end = None;
    let mut in_quote = false;
    for (i, c) in xml_text[start..].char_indices() {
        match c {
            '"' => in_quote = !in_quote,
            '>' if !in_quote => {
                end = Some(start + i);
                break;
            }
            _ => {}
        }
    }
    let tag = &xml_text[start..end?];
    let pos = attribute_value(tag, "POS")?;
    let name = attribute_value(tag, "name")?;
    let frame = attribute_value(tag, "frame")?;
    Some((pos, name, frame))
}

/// Find `key="value"` within `tag` and return `value`. Fails closed
/// (`None`) on a missing or malformed attribute, never a panic.
fn attribute_value(tag: &str, key: &str) -> Option<String> {
    let needle = alloc::format!("{key}=\"");
    let idx = tag.find(&needle)?;
    let value_start = idx + needle.len();
    let value_end = value_start + tag[value_start..].find('"')?;
    Some(tag[value_start..value_end].to_string())
}

/// Map a FrameNet `POS` attribute value to the shared [`LmfPos`] vocabulary
/// (Universal-Dependencies-flavored, already covering both WordNet's
/// open-class tags AND closed-class function-word categories — see that
/// type's own doc). No separate "FrameNet POS" type is minted; FrameNet's
/// ten tags map onto `LmfPos`'s existing variants exactly.
fn framenet_pos_to_lmf(pos: &str) -> Option<crate::social::software::markup::xml::lmf::LmfPos> {
    use crate::social::software::markup::xml::lmf::LmfPos;
    Some(match pos {
        "V" => LmfPos::Verb,
        "N" => LmfPos::Noun,
        "A" => LmfPos::Adjective,
        "ADV" => LmfPos::Adverb,
        "PREP" => LmfPos::Preposition,
        "NUM" => LmfPos::Numeral,
        "INTJ" => LmfPos::Interjection,
        "ART" => LmfPos::Determiner,
        "C" | "SCON" => LmfPos::Conjunction,
        _ => return None,
    })
}

/// Parse `frRelation.xml`'s `<frameRelationType name="...">` blocks, each
/// containing `<frameRelation subFrameName="..." superFrameName="...">`
/// children, into flat `(relation_name, sub_frame, super_frame)` rows.
fn parse_frame_relations(xml_text: &str) -> Vec<(String, String, String)> {
    use crate::social::software::markup::xml::ontology::XmlNode;
    use crate::social::software::markup::xml::reader::read_xml;

    let Ok(doc) = read_xml(xml_text) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for rel_type in doc.find_all("frameRelationType") {
        let Some(rel_name) = rel_type.attribute("name") else {
            continue;
        };
        for child in &rel_type.children {
            let XmlNode::Element(elem) = child else {
                continue;
            };
            if elem.name.local != "frameRelation" {
                continue;
            }
            let (Some(sub), Some(sup)) = (
                elem.attribute("subFrameName"),
                elem.attribute("superFrameName"),
            ) else {
                continue;
            };
            rows.push((rel_name.to_string(), sub.to_string(), sup.to_string()));
        }
    }
    rows
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
#[ignore]
fn regenerate_framenet_archive() {
    let zip_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/framenet-download/framenet_v17.zip");
    let zip =
        std::fs::read(&zip_path).unwrap_or_else(|e| panic!("read {}: {e}", zip_path.display()));

    let entries =
        extract_zip_entries_by_prefix(&zip, &["framenet_v17/lu/", "framenet_v17/frRelation.xml"]);
    eprintln!("extracted {} zip entries", entries.len());

    let mut lu_rows: Vec<String> = Vec::new();
    let mut frame_rel_rows: Vec<String> = Vec::new();
    let mut skipped_closed_class = 0usize;
    let mut skipped_malformed = 0usize;

    for (name, content) in &entries {
        if name == "framenet_v17/frRelation.xml" {
            let text = String::from_utf8_lossy(content);
            for (rel, sub, sup) in parse_frame_relations(&text) {
                frame_rel_rows.push(alloc::format!("REL\t{rel}\t{sub}\t{sup}"));
            }
            continue;
        }
        if !name.ends_with(".xml") || name == "framenet_v17/lu/" {
            continue;
        }
        let text = String::from_utf8_lossy(content);
        let Some((pos, name_attr, frame)) = parse_lu_root_attrs(&text) else {
            skipped_malformed += 1;
            continue;
        };
        let Some(lmf) = framenet_pos_to_lmf(&pos) else {
            skipped_malformed += 1;
            continue;
        };
        if !lmf.is_open_class() {
            skipped_closed_class += 1;
            continue;
        }
        // `name_attr` is `lemma.pos` (e.g. "cause.v"); strip the known
        // trailing `.<pos>` suffix to recover the bare lemma.
        let lemma = name_attr
            .rsplit_once('.')
            .map(|(lemma, _)| lemma)
            .unwrap_or(&name_attr);
        lu_rows.push(alloc::format!("LU\t{lemma}\t{}\t{frame}", lmf.to_tag()));
    }

    lu_rows.sort();
    lu_rows.dedup();
    frame_rel_rows.sort();
    frame_rel_rows.dedup();

    eprintln!(
        "lu rows: {} (skipped {skipped_closed_class} closed-class, {skipped_malformed} malformed)",
        lu_rows.len()
    );
    eprintln!("frame relation rows: {}", frame_rel_rows.len());

    // ONE flat TSV, type-tagged per row (`LU`/`REL`) — rides the
    // existing generic `ContentType::Plaintext` raw-source path
    // (the SAME reuse ConceptNet's regen validated), rather than
    // minting a new collection-shaped `ContentType` + decoder module
    // for what is, after extraction, just two logical row shapes.
    let mut rows = lu_rows;
    rows.extend(frame_rel_rows);
    let out_text = rows.join("\n");

    let out =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/framenet/framenet-1.7.tsv");
    std::fs::create_dir_all(out.parent().expect("has parent")).expect("mkdir data/framenet");
    std::fs::write(&out, out_text.as_bytes())
        .unwrap_or_else(|e| panic!("write {}: {e}", out.display()));
    eprintln!(
        "wrote {} ({} bytes) address {}",
        out.display(),
        out_text.len(),
        pr4xis_runtime::address::ContentAddress::of(out_text.as_bytes()).to_hex()
    );
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    const REAL_LU2_SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<?xml-stylesheet type="text/xsl" href="lexUnit.xsl"?>
<lexUnit status="Finished_Initial" POS="V" name="cause.v" ID="2" frame="Causation" frameID="5" totalAnnotated="116" xsi:schemaLocation="../schema/lexUnit.xsd" xmlns="http://framenet.icsi.berkeley.edu" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <header>
        <corpus description="BNC2" name="BNC2" ID="111"/>
    </header>
</lexUnit>"#;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parses_the_real_lu2_root_tag() {
        let (pos, name, frame) = parse_lu_root_attrs(REAL_LU2_SAMPLE).expect("parses");
        assert_eq!(pos, "V");
        assert_eq!(name, "cause.v");
        assert_eq!(frame, "Causation");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn rejects_text_with_no_lexunit_tag_without_panicking() {
        assert_eq!(parse_lu_root_attrs("<notLexUnit/>"), None);
        assert_eq!(parse_lu_root_attrs(""), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn framenet_pos_maps_onto_lmf_pos_exactly() {
        use crate::social::software::markup::xml::lmf::LmfPos;
        assert_eq!(framenet_pos_to_lmf("V"), Some(LmfPos::Verb));
        assert_eq!(framenet_pos_to_lmf("N"), Some(LmfPos::Noun));
        assert_eq!(framenet_pos_to_lmf("A"), Some(LmfPos::Adjective));
        assert_eq!(framenet_pos_to_lmf("ADV"), Some(LmfPos::Adverb));
        assert_eq!(framenet_pos_to_lmf("PREP"), Some(LmfPos::Preposition));
        assert_eq!(framenet_pos_to_lmf("NUM"), Some(LmfPos::Numeral));
        assert_eq!(framenet_pos_to_lmf("INTJ"), Some(LmfPos::Interjection));
        assert_eq!(framenet_pos_to_lmf("ART"), Some(LmfPos::Determiner));
        assert_eq!(framenet_pos_to_lmf("C"), Some(LmfPos::Conjunction));
        assert_eq!(framenet_pos_to_lmf("SCON"), Some(LmfPos::Conjunction));
        assert_eq!(framenet_pos_to_lmf("bogus"), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn open_class_filter_keeps_only_wordnet_comparable_pos() {
        use crate::social::software::markup::xml::lmf::LmfPos;
        assert!(LmfPos::Verb.is_open_class());
        assert!(LmfPos::Noun.is_open_class());
        assert!(!LmfPos::Preposition.is_open_class());
        assert!(!LmfPos::Numeral.is_open_class());
    }

    const REAL_FRRELATION_SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<frameRelations XMLCreated="11/07/2016" xmlns="http://framenet.icsi.berkeley.edu">
    <frameRelationType subFrameName="Inchoative/state" superFrameName="Causative" name="Causative_of" ID="10">
        <frameRelation subID="670" supID="276" subFrameName="Moving_in_place" superFrameName="Cause_to_move_in_place" ID="1208">
            <FERelation subID="4489" supID="2402" subFEName="Angle" superFEName="Angle" ID="7532"/>
        </frameRelation>
        <frameRelation subID="830" supID="236" subFrameName="Absorb_heat" superFrameName="Apply_heat" ID="1209"/>
    </frameRelationType>
</frameRelations>"#;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parses_real_frame_relation_rows() {
        let rows = parse_frame_relations(REAL_FRRELATION_SAMPLE);
        assert_eq!(
            rows,
            alloc::vec![
                (
                    "Causative_of".to_string(),
                    "Moving_in_place".to_string(),
                    "Cause_to_move_in_place".to_string()
                ),
                (
                    "Causative_of".to_string(),
                    "Absorb_heat".to_string(),
                    "Apply_heat".to_string()
                ),
            ]
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn rejects_malformed_xml_without_panicking() {
        assert_eq!(parse_frame_relations("not xml at all {{{"), Vec::new());
    }
}

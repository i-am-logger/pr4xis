//! Offline regeneration of the committed SUMO bundle: one
//! `concept_value<TAB>term<TAB>relation_code<TAB>oewn_synset_id` row per
//! (RESOLVED synset, SUMO term, relation) triple. The 4th column is the real,
//! external OEWN synset id the row's `concept_value` resolves to (added for
//! [`super::sssom`]'s SSSOM `subject_id` — see that module) — carried on EVERY
//! row (not just `EQ`) so the TSV shape stays uniform.
//!
//! ## Why extraction AND offline resolution
//!
//! SUMO's `WordNetMappings30-{noun,verb,adj,adv}.txt` files are annotated
//! COPIES of the Princeton WordNet **3.0** `data.*` files (Niles & Pease 2001,
//! 2003): each data line is a full WNDB synset record with `&%<term><suffix>`
//! tokens appended. This project needs each annotation's synset, its SUMO term,
//! and the relation — a pure format-conversion of the annotations themselves.
//!
//! But the synset must be RESOLVED to this project's `ConceptId`, and SUMO's
//! PWN-3.0 synset offsets do NOT match Open English WordNet 2025's synset ids
//! (measured: only ~385 of 82,115 noun offsets collide). The version-stable
//! bridge is the WordNet SENSE KEY — every SUMO WNDB record carries its synset's
//! member words with their `lex_id`s, and `lex_filenum` + `ss_type` are on the
//! line, so a sense key `lemma%ss_type:lex_filenum:lex_id` reconstructs exactly.
//! OEWN's `Sense` ids ARE the sense-key-derived form (see
//! [`oewn_sense_id_for_sense_key`](crate::cognitive::linguistics::verbnet::store::oewn_sense_id_for_sense_key)),
//! so this regen — exactly like VerbNet's —
//! parses the OEWN XML ONCE, offline, resolves each SUMO synset's sense keys to
//! a `ConceptId` VALUE (stable across load paths, per VerbNet's store doc), and
//! bakes that value into the committed table. A SUMO synset none of whose sense
//! keys resolve against this WordNet build (or whose offset also misses) is
//! dropped offline. The extracted table rides the existing
//! `ContentType::Plaintext` raw-source path (no new decoder module).
//!
//! ## Prerequisite (external, not run by this module)
//!
//! ```text
//! mkdir -p crates/domains/data/sumo-download
//! for f in noun verb adj adv; do
//!   curl -sSL -o crates/domains/data/sumo-download/WordNetMappings30-$f.txt \
//!     https://raw.githubusercontent.com/ontologyportal/sumo/master/WordNetMappings/WordNetMappings30-$f.txt
//! done
//! ```
//!
//! `data/sumo-download/` is gitignored (transient staging, mirroring
//! `data/framenet-download/` and `data/conceptnet-download/`) — only this
//! module's OUTPUT, the extracted+resolved `.tsv`, is committed. The source has
//! no tagged release; the fetch is pinned by recording the `master` commit SHA
//! in the `[sources.sumo]` registry description. The resolution reads the
//! committed WordNet XML (`data/wordnet/english-wordnet-2025.xml`).

use super::ontology::SumoRelationKind;

/// Is `c` a character that may appear inside a SUMO term name? SUMO terms are
/// CamelCase identifiers that may also carry digits and hyphens (e.g.
/// `Bank-FinancialOrganization`, `USMilitaryRankO2`, `CD-ROM` — all observed in
/// the real files), so the term alphabet is `[A-Za-z0-9_-]`. The boundary
/// character that ENDS a term is either a legend suffix (a real annotation) or
/// anything else (a bare/spaced annotation this module drops fail-closed).
fn is_term_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// Scan one gloss half for every `&%<term><suffix>` annotation, returning the
/// `(term, suffix_char)` pairs. An `&%` whose term is immediately followed by a
/// non-suffix character (a space-separated or bare annotation — ~18 exist
/// across the four files, e.g. `&%IleocolicArtery` with no suffix, or
/// `&%IntentionalPsychologicalProcess +` with a stray space) is dropped
/// fail-closed: without a valid attached suffix there is no relation to assert.
fn parse_annotations(gloss: &str) -> alloc::vec::Vec<(alloc::string::String, char)> {
    use alloc::string::String;
    use alloc::vec::Vec;
    let chars: Vec<char> = gloss.chars().collect();
    let n = chars.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < n {
        if chars[i] == '&' && chars[i + 1] == '%' {
            let start = i + 2;
            let mut j = start;
            while j < n && is_term_char(chars[j]) {
                j += 1;
            }
            if j > start && j < n && SumoRelationKind::from_suffix(chars[j]).is_some() {
                let term: String = chars[start..j].iter().collect();
                out.push((term, chars[j]));
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// One parsed SUMO data line: the synset's WNDB identity (offset + POS letter),
/// the `lex_filenum`, the member `(lemma, lex_id)` list (for sense-key
/// reconstruction), and the SUMO `(term, suffix)` annotations.
struct SumoLine {
    offset: alloc::string::String,
    pos: alloc::string::String,
    lex_filenum: alloc::string::String,
    words: alloc::vec::Vec<(alloc::string::String, u8)>,
    annotations: alloc::vec::Vec<(alloc::string::String, char)>,
}

/// Parse one raw WordNetMappings data line. Returns `None` for any line that is
/// not a synset data record — cheaply skipping `;;` comment lines and blank
/// lines by requiring the line to start with eight ASCII digits followed by a
/// space (the WNDB `synset_offset` + separator). The synset-data half (before
/// the first `|`) yields the offset (token 0), `lex_filenum` (token 1), POS
/// letter (token 2), and the `w_cnt` member `(word, lex_id)` pairs (WNDB
/// `data.*` format: `w_cnt` is a 2-hex-digit count, each word followed by a
/// 1-hex-digit `lex_id`). The gloss half (after the first `|`) carries the `&%`
/// annotations.
fn parse_data_line(line: &str) -> Option<SumoLine> {
    use alloc::string::ToString;
    use alloc::vec::Vec;

    let bytes = line.as_bytes();
    if bytes.len() < 9 || bytes[8] != b' ' || !bytes[..8].iter().all(u8::is_ascii_digit) {
        return None;
    }
    let (data_half, gloss_half) = line.split_once('|')?;
    let tokens: Vec<&str> = data_half.split_whitespace().collect();
    // offset, lex_filenum, ss_type, w_cnt + at least one (word, lex_id) pair.
    if tokens.len() < 6 {
        return None;
    }
    let offset = tokens[0].to_string();
    let lex_filenum = tokens[1].to_string();
    let pos = tokens[2].to_string();
    let w_cnt = usize::from_str_radix(tokens[3], 16).ok()?;

    let mut words = Vec::with_capacity(w_cnt);
    for i in 0..w_cnt {
        let word = tokens.get(4 + 2 * i)?;
        let lex_id_hex = tokens.get(5 + 2 * i)?;
        let lex_id = u8::from_str_radix(lex_id_hex, 16).ok()?;
        words.push(((*word).to_string(), lex_id));
    }

    Some(SumoLine {
        offset,
        pos,
        lex_filenum,
        words,
        annotations: parse_annotations(gloss_half),
    })
}

/// The WNDB `ss_type` digit for a SUMO POS letter (Fellbaum 1998, WNDB(5WN)):
/// 1=noun, 2=verb, 3=adjective, 4=adverb, 5=adjective-satellite. `None` for any
/// other letter (never observed in these files).
fn ss_type(pos: &str) -> Option<&'static str> {
    Some(match pos {
        "n" => "1",
        "v" => "2",
        "a" => "3",
        "r" => "4",
        "s" => "5",
        _ => return None,
    })
}

/// Reconstruct the WordNet sense key for a synset member and lower it to the
/// OEWN `Sense` id form (`oewn-<lemma>__<ss>.<lexfn>.<lexid>..`). The WNDB word
/// may carry a syntactic marker like `word(a)`/`word(p)`/`word(ip)` for
/// adjective position — stripped for the lemma. `None` if the POS letter has no
/// `ss_type`. Satellite adjectives (ss_type 5) would additionally need a
/// head_word:head_id in the true sense key, which this reconstruction omits, so
/// satellites resolve only via the offset fallback — an honest limitation.
fn oewn_sense_id_for_member(
    lemma: &str,
    lex_id: u8,
    lex_filenum: &str,
    pos: &str,
) -> Option<alloc::string::String> {
    use crate::cognitive::linguistics::verbnet::store::oewn_sense_id_for_sense_key;
    let ss = ss_type(pos)?;
    let bare = lemma
        .split('(')
        .next()
        .unwrap_or(lemma)
        .to_ascii_lowercase();
    if bare.is_empty() {
        return None;
    }
    let sense_key = alloc::format!("{bare}%{ss}:{lex_filenum}:{lex_id:02}");
    oewn_sense_id_for_sense_key(&sense_key)
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
#[ignore]
fn regenerate_sumo_archive() {
    use crate::cognitive::linguistics::english::ontology::English;
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;
    use std::collections::HashMap;

    // 1. Parse the committed OEWN XML once and build the resolution maps.
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let xml_path = manifest.join("data/wordnet/english-wordnet-2025.xml");
    let xml = std::fs::read_to_string(&xml_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", xml_path.display()));
    let wn = read_wordnet(&xml).expect("parse OEWN XML");
    let en = English::from_wordnet(&wn);

    // synset id string -> ConceptId value.
    let mut synset_to_concept: HashMap<&str, u64> = HashMap::new();
    // ConceptId value -> its CANONICAL OEWN synset id string (the reverse of
    // the map above, built in the same pass) — the real, external, dereferenceable
    // OEWN synset identity for the SSSOM `subject_id` (see `super::sssom`). Used
    // for every resolved row regardless of which route (offset or sense-key)
    // found the concept, so the emitted id is always the canonical synset OEWN
    // itself resolved to, not the (possibly different) synset a member's sense
    // key happened to be reconstructed from.
    let mut concept_to_synset: HashMap<u64, &str> = HashMap::new();
    for synset in &wn.synsets {
        if let Some(view) = en.concept_by_synset(&synset.id) {
            let cv = view.id().value();
            synset_to_concept.insert(synset.id.as_str(), cv);
            concept_to_synset.entry(cv).or_insert(synset.id.as_str());
        }
    }
    // OEWN Sense id string -> ConceptId value (via the sense's synset).
    let mut sense_to_concept: HashMap<&str, u64> = HashMap::new();
    for entry in &wn.entries {
        for sense in &entry.senses {
            if let Some(&cv) = synset_to_concept.get(sense.synset.as_str()) {
                sense_to_concept.insert(sense.id.as_str(), cv);
            }
        }
    }

    // 2. Resolve each SUMO synset and emit resolved rows.
    let mut rows: Vec<String> = Vec::new();
    let mut synsets_seen = 0usize;
    let mut synsets_resolved = 0usize;
    let mut resolved_via_offset = 0usize;
    let mut resolved_via_sense = 0usize;
    for pos_file in ["noun", "verb", "adj", "adv"] {
        let path = manifest.join(format!(
            "data/sumo-download/WordNetMappings30-{pos_file}.txt"
        ));
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let mut file_rows = 0usize;
        for line in text.lines() {
            let Some(parsed) = parse_data_line(line) else {
                continue;
            };
            if parsed.annotations.is_empty() {
                continue;
            }
            synsets_seen += 1;

            // Resolve the synset to a ConceptId value: offset route first (the
            // rare stable synset), then each member's sense-key route.
            let offset_id = format!("oewn-{}-{}", parsed.offset, parsed.pos);
            let concept = if let Some(&cv) = synset_to_concept.get(offset_id.as_str()) {
                resolved_via_offset += 1;
                Some(cv)
            } else {
                let mut found = None;
                for (lemma, lex_id) in &parsed.words {
                    let Some(sid) =
                        oewn_sense_id_for_member(lemma, *lex_id, &parsed.lex_filenum, &parsed.pos)
                    else {
                        continue;
                    };
                    if let Some(&cv) = sense_to_concept.get(sid.as_str()) {
                        found = Some(cv);
                        break;
                    }
                }
                if found.is_some() {
                    resolved_via_sense += 1;
                }
                found
            };
            let Some(concept) = concept else {
                continue;
            };
            synsets_resolved += 1;
            // The canonical OEWN synset id this concept resolves to (see
            // `concept_to_synset` above) — always present once `concept` is
            // resolved, since both routes above resolve THROUGH `synset_to_concept`.
            let oewn_synset_id = concept_to_synset.get(&concept).unwrap_or_else(|| {
                panic!("concept {concept} resolved with no canonical synset id")
            });

            for (term, suffix) in &parsed.annotations {
                let Some(relation) = SumoRelationKind::from_suffix(*suffix) else {
                    continue;
                };
                rows.push(format!(
                    "{concept}\t{term}\t{}\t{oewn_synset_id}",
                    relation.to_code()
                ));
                file_rows += 1;
            }
        }
        eprintln!("{pos_file}: {file_rows} resolved rows");
    }

    rows.sort();
    rows.dedup();
    eprintln!(
        "synsets: {synsets_seen} annotated, {synsets_resolved} resolved \
         ({resolved_via_offset} via offset, {resolved_via_sense} via sense key), \
         {} dropped as unmapped",
        synsets_seen - synsets_resolved
    );
    eprintln!("resolved rows after dedup: {}", rows.len());

    let out_text = rows.join("\n");
    let out = manifest.join("data/sumo/sumo-wordnetmappings-30.tsv");
    std::fs::create_dir_all(out.parent().expect("has parent")).expect("mkdir data/sumo");
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

    // Real lines from the WordNetMappings30-*.txt files (do not synthesize SUMO
    // term names — these are the genuine annotations).
    const ENTITY_NOUN: &str = "00001740 03 n 01 entity 0 003 ~ 00001930 n 0000 ~ 00002137 n 0000 ~ 04424418 n 0000 | that which is perceived or known or inferred to have its own distinct existence (living or nonliving) &%Entity=";
    const DRIVE_BY_NOUN: &str = "00219738 04 n 01 drive-by_killing 0 001 @ 00225150 n 0000 | homicide committed by shooting from a moving automobile &%Shooting+ &%Murder+";
    const FED_RESERVE_NOUN: &str = "08350919 14 n 02 Federal_Reserve_Bank 0 reserve_bank 0 002 @ 08420278 n 0000 #m 08350470 n 0000 | one of 12 regional banks that monitor and act as depositories for banks in their region &%Bank-FinancialOrganization+";
    const HEGIRA_NOUN: &str = "00060548 04 n 02 Hegira 1 Hejira 1 001 @i 00058743 n 0000 | the flight of Muhammad from Mecca to Medina in 622 which marked the beginning of the Muslim era; the Muslim calendar begins in that year &%Escaping@";
    const COVER_NOUN: &str = "01049992 04 n 01 cover 2 001 @ 01048912 n 0000 | a false identity and background (especially one created for an undercover agent); \"her new name and passport are cover for her next assignment\" &%Disseminating[";

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parses_offset_pos_words_and_a_single_annotation() {
        let p = parse_data_line(ENTITY_NOUN).expect("parses");
        assert_eq!(p.offset, "00001740");
        assert_eq!(p.pos, "n");
        assert_eq!(p.lex_filenum, "03");
        assert_eq!(p.words, alloc::vec![("entity".to_string(), 0u8)]);
        assert_eq!(p.annotations, alloc::vec![("Entity".to_string(), '=')]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parses_two_annotations_and_two_member_words() {
        let p = parse_data_line(FED_RESERVE_NOUN).expect("parses");
        assert_eq!(
            p.words,
            alloc::vec![
                ("Federal_Reserve_Bank".to_string(), 0u8),
                ("reserve_bank".to_string(), 0u8)
            ]
        );
        assert_eq!(
            p.annotations,
            alloc::vec![("Bank-FinancialOrganization".to_string(), '+')]
        );

        let d = parse_data_line(DRIVE_BY_NOUN).expect("parses");
        assert_eq!(
            d.annotations,
            alloc::vec![("Shooting".to_string(), '+'), ("Murder".to_string(), '+')]
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn instance_and_complement_suffixes_map_to_codes() {
        let hegira = parse_data_line(HEGIRA_NOUN).expect("parses");
        assert_eq!(
            hegira.annotations,
            alloc::vec![("Escaping".to_string(), '@')]
        );
        assert_eq!(
            SumoRelationKind::from_suffix(hegira.annotations[0].1).map(SumoRelationKind::to_code),
            Some("INST")
        );
        let cover = parse_data_line(COVER_NOUN).expect("parses");
        assert_eq!(
            cover.annotations,
            alloc::vec![("Disseminating".to_string(), '[')]
        );
        assert_eq!(
            SumoRelationKind::from_suffix(cover.annotations[0].1).map(SumoRelationKind::to_code),
            Some("CSUB")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reconstructs_the_oewn_sense_id_for_a_member() {
        // entity, lex_id 0, lex_filenum 03, noun → sense key entity%1:03:00 →
        // oewn-entity__1.03.00.. (the OEWN Sense id form).
        assert_eq!(
            oewn_sense_id_for_member("entity", 0, "03", "n").as_deref(),
            Some("oewn-entity__1.03.00..")
        );
        // A word carrying an adjective syntactic marker keeps only the bare
        // lemma, and a hex lex_id becomes a 2-digit decimal.
        assert_eq!(
            oewn_sense_id_for_member("small(a)", 10, "00", "a").as_deref(),
            Some("oewn-small__3.00.10..")
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn skips_comment_and_blank_lines_without_panicking() {
        assert!(parse_data_line(";; This is an annotated version of the data.noun file").is_none());
        assert!(parse_data_line("").is_none());
        assert!(parse_data_line("   ").is_none());
        // The legend line's `&%Motion[` is inside a `;;` comment — must not be
        // mistaken for a data annotation.
        assert!(
            parse_data_line(";; (%ComplementFn &%Motion)+   now appears as  &%Motion[").is_none()
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn drops_a_bare_or_space_separated_annotation() {
        assert_eq!(parse_annotations("an artery &%IleocolicArtery"), Vec::new());
        assert_eq!(
            parse_annotations("gambling &%IntentionalPsychologicalProcess +"),
            Vec::new()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ss_type_maps_every_pos_letter() {
        assert_eq!(ss_type("n"), Some("1"));
        assert_eq!(ss_type("v"), Some("2"));
        assert_eq!(ss_type("a"), Some("3"));
        assert_eq!(ss_type("r"), Some("4"));
        assert_eq!(ss_type("s"), Some("5"));
        assert_eq!(ss_type("x"), None);
    }
}

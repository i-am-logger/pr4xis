//! English irregular forms — the dual-route lookup table, LOADED from AGID.
//!
//! Was a ~97-entry hand-coded table (Quirk et al. 1985 §3.21–3.59 + Pinker
//! 1991) carrying an untracked "until the AGID source is wired up" deferral
//! (audit 2026-06-12 D-1). Now the irregular forms are DERIVED from the
//! registered AGID inflection database (Atkinson 2016, `[sources.agid]`):
//! [`english_irregulars`] loads the committed `english-irregulars.tsv` slice
//! that the `#[ignore]` regenerate step extracts from AGID by keeping every
//! inflected form a productive English rule (Quirk §3) cannot generate — the
//! morphological EXCEPTIONS. Loaded-not-encoded; the regular-rule helpers exist
//! only as the build-time derivation tool, never as the working representation.

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec, vec::Vec};

use super::super::irregular::{IrregularForm, IrregularKind};

/// The committed irregular-forms slice, derived from the registered AGID
/// source (Atkinson 2016). One `surface<TAB>lemma<TAB>kind` row per irregular
/// form. Regenerate with `--ignored regenerate_english_irregulars_tsv`.
const IRREGULARS_TSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/morphology/english-irregulars.tsv"
));

/// English irregular forms — the loaded morphological-exception table.
///
/// Parsed from the committed AGID-derived TSV. Under `std` the parse is cached
/// process-wide (`OnceLock`): the lemmatizer looks up irregulars per token, so
/// re-parsing the multi-thousand-row TSV on every call would be a real
/// regression. The `no_std`/wasm surface (no `OnceLock`) keeps the
/// fresh-`Vec`-per-call parse.
pub fn english_irregulars() -> Vec<IrregularForm> {
    #[cfg(feature = "std")]
    {
        irregulars_cached().to_vec()
    }
    #[cfg(not(feature = "std"))]
    {
        parse_irregulars_tsv(IRREGULARS_TSV)
    }
}

/// The process-wide cached parse of the AGID-derived TSV (`std` only) — the slice
/// [`lookup_irregular`] scans without cloning or re-parsing.
#[cfg(feature = "std")]
fn irregulars_cached() -> &'static [IrregularForm] {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Vec<IrregularForm>> = OnceLock::new();
    CACHE.get_or_init(|| parse_irregulars_tsv(IRREGULARS_TSV))
}

fn parse_irregulars_tsv(tsv: &str) -> Vec<IrregularForm> {
    tsv.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let mut it = l.split('\t');
            let surface = it.next()?;
            let lemma = it.next()?;
            let kind = match it.next()? {
                "PluralNoun" => IrregularKind::PluralNoun,
                "PastTense" => IrregularKind::PastTense,
                "PastParticiple" => IrregularKind::PastParticiple,
                "Comparative" => IrregularKind::Comparative,
                "Superlative" => IrregularKind::Superlative,
                _ => return None,
            };
            Some(IrregularForm::new(surface, lemma, kind))
        })
        .collect()
}

/// English-specific irregular lookup — case-insensitive match against
/// [`english_irregulars`].
pub fn lookup_irregular(surface: &str) -> Vec<IrregularForm> {
    #[cfg(feature = "std")]
    {
        super::super::irregular::lookup_in(surface, irregulars_cached())
    }
    #[cfg(not(feature = "std"))]
    {
        super::super::irregular::lookup_in(surface, &english_irregulars())
    }
}

// =========================================================================
// Build-time AGID → irregulars extraction — the regenerate tool.
//
// These functions DERIVE the committed TSV from the registered AGID source;
// they are NOT the working representation (that is the loaded TSV above). The
// regular-inflection rules (Quirk et al. 1985 §3) are used ONLY to identify
// the EXCEPTIONS — an inflected form is irregular iff no productive rule
// generates it from the base.
// =========================================================================

#[cfg(test)]
mod regenerate {
    use super::*;

    /// On-disk path of the registered AGID source. Like the WordNet XML, the
    /// large source is fetched via `pr4xis update` (gitignored), NOT committed;
    /// only the small derived `english-irregulars.tsv` is git-tracked.
    const AGID_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/morphology/agid-infl.txt");

    fn is_vowel(c: char) -> bool {
        matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
    }

    fn ends_consonant_y(base: &str) -> Option<&str> {
        let stem = base.strip_suffix('y')?;
        match stem.chars().last() {
            Some(c) if !is_vowel(c) => Some(stem),
            _ => None,
        }
    }

    /// CVC: ends consonant–vowel–consonant (last not w/x/y) — the doubling
    /// environment (Quirk §3.x: stop→stopped, big→bigger).
    fn doubles_final(base: &str) -> Option<char> {
        let cs: Vec<char> = base.chars().collect();
        let n = cs.len();
        if n < 3 {
            return None;
        }
        let (c1, v, c2) = (cs[n - 3], cs[n - 2], cs[n - 1]);
        if !is_vowel(c1) && is_vowel(v) && !is_vowel(c2) && !matches!(c2, 'w' | 'x' | 'y') {
            Some(c2)
        } else {
            None
        }
    }

    /// Regular plural forms (Quirk et al. 1985 §3.21): +s; +es after a
    /// sibilant / final -o; consonant+y → -ies; f(e) → -ves.
    fn regular_plurals(base: &str) -> Vec<String> {
        let mut v = vec![format!("{base}s")];
        if base.ends_with('s')
            || base.ends_with('x')
            || base.ends_with('z')
            || base.ends_with("ch")
            || base.ends_with("sh")
            || base.ends_with('o')
        {
            v.push(format!("{base}es"));
        }
        if let Some(stem) = ends_consonant_y(base) {
            v.push(format!("{stem}ies"));
        }
        if let Some(stem) = base.strip_suffix("fe") {
            v.push(format!("{stem}ves"));
        } else if let Some(stem) = base.strip_suffix('f') {
            v.push(format!("{stem}ves"));
        }
        v
    }

    /// Regular -ed forms (past tense / past participle): +ed; final-e → +d;
    /// consonant+y → -ied; CVC doubling → +Ced.
    fn regular_ed(base: &str) -> Vec<String> {
        let mut v = vec![format!("{base}ed")];
        if base.ends_with('e') {
            v.push(format!("{base}d"));
        }
        if let Some(stem) = ends_consonant_y(base) {
            v.push(format!("{stem}ied"));
        }
        if let Some(c) = doubles_final(base) {
            v.push(format!("{base}{c}ed"));
        }
        v
    }

    /// Regular comparative/superlative with a given suffix (-er/-est).
    fn regular_degree(base: &str, suffix: &str) -> Vec<String> {
        let tail = &suffix[1..]; // "er"→"r", "est"→"st" for final-e bases
        let mut v = vec![format!("{base}{suffix}")];
        if base.ends_with('e') {
            v.push(format!("{base}{tail}"));
        }
        if let Some(stem) = ends_consonant_y(base) {
            v.push(format!("{stem}i{suffix}"));
        }
        if let Some(c) = doubles_final(base) {
            v.push(format!("{base}{c}{suffix}"));
        }
        v
    }

    /// The primary acceptable form of an AGID inflection group (forms are
    /// comma-separated; each may carry `~ < ! ?` tags, a variant level, and a
    /// `{explanation}`). Skips variant-level-2 (archaic/obscure/uncertain) and
    /// `!`-tagged forms (likely an inflection of a *similar* word). Returns the
    /// first surviving form's bare word.
    fn primary_form(group: &str) -> Option<String> {
        for entry in group.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let toks: Vec<&str> = entry.split_whitespace().collect();
            // variant level 2 → skip (archaic/obscure/uncertain)
            if toks.iter().any(|t| *t == "2") {
                continue;
            }
            let raw = toks[0];
            let word: String = raw
                .chars()
                .take_while(|c| c.is_ascii_alphabetic() || *c == '\'')
                .collect();
            if word.is_empty() {
                continue;
            }
            // a '!' immediately after the word marks a different base
            if raw[word.len()..].starts_with('!') {
                continue;
            }
            return Some(word);
        }
        None
    }

    /// Extract the irregular `(surface, lemma, kind)` rows from one AGID line
    /// (`<word> <pos>[?]: <groups separated by |>`).
    fn extract_line(line: &str, out: &mut Vec<(String, String, &'static str)>) {
        let Some((head, forms)) = line.split_once(':') else {
            return;
        };
        let mut hp = head.split_whitespace();
        let Some(base) = hp.next() else { return };
        let Some(pos_raw) = hp.next() else { return };
        let pos = pos_raw.trim_end_matches('?');
        // AGID keys proper nouns / acronyms too; the regular-form filter drops
        // their (regular) inflections, but skip non-lowercase bases up front to
        // keep the exception list to ordinary words.
        if !base.chars().all(|c| c.is_ascii_lowercase() || c == '\'') {
            return;
        }
        let groups: Vec<&str> = forms.split('|').map(|g| g.trim()).collect();
        let push_if_irregular =
            |form: Option<String>,
             regulars: &[String],
             kind: &'static str,
             out: &mut Vec<(String, String, &'static str)>| {
                if let Some(f) = form {
                    if !regulars.contains(&f) {
                        out.push((f, base.to_string(), kind));
                    }
                }
            };
        match pos {
            "N" => {
                push_if_irregular(
                    primary_form(groups[0]),
                    &regular_plurals(base),
                    "PluralNoun",
                    out,
                );
            }
            "V" => {
                // `be` has AGID's documented special slot order
                // (was | were | been | being | am | art | is | are).
                if base == "be" {
                    for (g, kind) in [(0, "PastTense"), (1, "PastTense"), (2, "PastParticiple")] {
                        if let Some(f) = groups.get(g).and_then(|x| primary_form(x)) {
                            out.push((f, base.to_string(), kind));
                        }
                    }
                    return;
                }
                let ed = regular_ed(base);
                // group[0] = past tense; group[1] = past participle when the
                // verb carries the full `past | pp | -ing | -s` layout.
                push_if_irregular(primary_form(groups[0]), &ed, "PastTense", out);
                if groups.len() >= 4 {
                    push_if_irregular(primary_form(groups[1]), &ed, "PastParticiple", out);
                }
            }
            "A" => {
                push_if_irregular(
                    primary_form(groups[0]),
                    &regular_degree(base, "er"),
                    "Comparative",
                    out,
                );
                if groups.len() >= 2 {
                    push_if_irregular(
                        primary_form(groups[1]),
                        &regular_degree(base, "est"),
                        "Superlative",
                        out,
                    );
                }
            }
            _ => {}
        }
    }

    fn extract_all(agid: &str) -> Vec<(String, String, &'static str)> {
        let mut out = Vec::new();
        for line in agid.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            extract_line(line, &mut out);
        }
        // Deterministic, deduplicated output for a reproducible commit.
        out.sort();
        out.dedup();
        out
    }

    /// Regenerate the committed `english-irregulars.tsv` from the vendored
    /// AGID source. `#[ignore]`d (it WRITES, asserting nothing) — run by hand
    /// when AGID changes:
    /// `cargo test -p pr4xis-domains -- --ignored regenerate_english_irregulars_tsv`.
    #[test]
    #[ignore]
    fn regenerate_english_irregulars_tsv() {
        let agid = match std::fs::read_to_string(AGID_PATH) {
            Ok(s) => s,
            Err(_) => {
                eprintln!(
                    "agid-infl.txt absent at {AGID_PATH} — fetch the AGID source first \
                     (`pr4xis update` / dev-data), then re-run; skipping regenerate."
                );
                return;
            }
        };
        let rows = extract_all(&agid);
        let mut tsv = String::from(
            "# English irregular forms — DERIVED from AGID (Atkinson 2016, [sources.agid])\n\
             # by regenerate_english_irregulars_tsv: every inflected form a productive\n\
             # English rule (Quirk et al. 1985 §3) cannot generate. surface<TAB>lemma<TAB>kind.\n\
             # DO NOT hand-edit — regenerate with --ignored regenerate_english_irregulars_tsv.\n",
        );
        for (surface, lemma, kind) in &rows {
            tsv.push_str(&format!("{surface}\t{lemma}\t{kind}\n"));
        }
        let out = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/morphology/english-irregulars.tsv"
        );
        std::fs::write(out, &tsv).expect("write english-irregulars.tsv");
        let src_addr = pr4xis_runtime::address::ContentAddress::of(agid.as_bytes()).to_hex();
        eprintln!("agid@2016.01.19 source blake3 = {src_addr}");
        eprintln!("extracted {} irregular rows", rows.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_frequency_irregulars_present() {
        let table = english_irregulars();
        let surfaces: Vec<&str> = table.iter().map(|f| f.surface.as_str()).collect();
        for w in &[
            "children", "men", "women", "feet", "mice", "data", "was", "were", "been", "had",
            "did", "said", "went", "got", "made", "took", "gave", "saw", "came", "knew", "thought",
            "found", "told", "left", "ran", "wrote", "spoke", "broke", "chose", "began", "better",
            "best", "worse", "worst",
        ] {
            assert!(
                surfaces.contains(w),
                "missing high-frequency irregular: {w}"
            );
        }
    }

    #[test]
    fn every_entry_well_formed() {
        // surface == lemma is legitimate for zero-change irregulars (cut→cut,
        // sheep→sheep, read→read) — the AGID-derived table includes them.
        for entry in english_irregulars() {
            assert!(!entry.surface.is_empty(), "empty surface");
            assert!(!entry.lemma.is_empty(), "empty lemma in entry {:?}", entry);
        }
    }

    #[test]
    fn children_maps_to_child() {
        let entries = lookup_irregular("children");
        assert!(
            entries
                .iter()
                .any(|e| e.lemma == "child" && e.kind == IrregularKind::PluralNoun)
        );
    }

    #[test]
    fn went_maps_to_go() {
        let entries = lookup_irregular("went");
        assert!(
            entries
                .iter()
                .any(|e| e.lemma == "go" && e.kind == IrregularKind::PastTense)
        );
    }

    #[test]
    fn better_maps_to_good_comparative() {
        let entries = lookup_irregular("better");
        assert!(
            entries
                .iter()
                .any(|e| e.lemma == "good" && e.kind == IrregularKind::Comparative)
        );
    }

    #[test]
    fn lookup_case_insensitive() {
        let lower = lookup_irregular("children");
        let upper = lookup_irregular("CHILDREN");
        let mixed = lookup_irregular("Children");
        assert_eq!(lower, upper);
        assert_eq!(lower, mixed);
        assert!(!lower.is_empty());
    }

    #[test]
    fn lookup_unknown_returns_empty() {
        assert!(lookup_irregular("nonsenseword").is_empty());
    }

    #[test]
    fn the_loaded_table_is_substantial() {
        // The whole point of loading AGID: far more coverage than the prior
        // ~97-entry hand table.
        assert!(
            english_irregulars().len() > 500,
            "AGID-derived irregulars should be comprehensive"
        );
    }
}

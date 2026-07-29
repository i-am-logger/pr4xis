//! English inflection GENERATION — the forward direction of the dual-route
//! model (Pinker 1991): memory route first (the loaded AGID-derived
//! [`super::irregular::english_irregulars`] table blocks the rule), then the productive rule
//! route.
//!
//! This module is the forward twin of the analysis-side machinery the
//! lemmatizer already applies — the same orthographic alternations
//! ([`super::allomorphy`], Spencer 1991 §5.2: silent-e elision, consonant
//! doubling) written in the generating direction. The rule route is
//! deliberately the DETERMINISTIC sub-case of the cited orthography:
//!
//! - silent-e elision before the vowel-initial suffix (exhale → exhaling;
//!   Spencer 1991 §5.2), gated on a consonant before the `e` so vowel-final
//!   stems keep it (see → seeing);
//! - consonant doubling only for MONOSYLLABIC C-V-C bases (run → running;
//!   Quirk et al. 1985 §3, the same C-V-C environment as the derivation
//!   tool's `regular_ed`). Polysyllabic doubling depends on stress (admit →
//!   admitting vs visit → visiting, Quirk §3) which orthography alone cannot
//!   decide — those forms are EXCEPTIONS by construction and live in the
//!   loaded AGID slice ([`IrregularKind::PresentParticiple`]), exactly the
//!   dual-route split: predictable = rule, unpredictable = memory.
//!
//! AGID is the ORACLE for this split: the regenerate tool stores every AGID
//! -ing form the rule route cannot produce, and the oracle sweep in
//! `english::irregular` asserts `ing_form(lemma)` reproduces AGID's column
//! for every verb — so a rule/table disagreement is a failing test, never a
//! silent mis-generation.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use super::super::irregular::IrregularKind;
use super::irregular::with_irregulars;

/// Can `c` head a syllable peak — the LOADED English grapheme-class table's
/// `is_syllabic` reading ([`crate::cognitive::linguistics::orthography::grapheme`]),
/// not a hardcoded vowel set. `y` answers TRUE here (Carney 1994 Ch.1 carries
/// it as dual; "rhythm" has a peak spelled only by `y`), which is why the
/// syllable counter below no longer needs its own `|| c == 'y'` rider.
fn is_syllabic(c: char) -> bool {
    crate::cognitive::linguistics::orthography::grapheme::classes().is_syllabic(c)
}

/// Does `c` count as a CONSONANT for these orthographic rules — the loaded
/// table's `is_consonantal` reading. `y` answers FALSE (Quirk et al. 1985
/// §3.21 turns on exactly that: "boy" → "boys", because the letter before the
/// final `y` is a vowel). Note this is NOT `!is_syllabic(c)`: a digit or an
/// accented letter is neither, and the table says so.
fn is_consonantal(c: char) -> bool {
    crate::cognitive::linguistics::orthography::grapheme::classes().is_consonantal(c)
}

/// One of the five BASIC vowel letters — the loaded table's strict reading,
/// which excludes `y`. Used where the cited rule is stated over the basic
/// vowels specifically (the C-V-C doubling environment), as distinct from
/// syllable-peak counting.
fn is_basic_vowel(c: char) -> bool {
    crate::cognitive::linguistics::orthography::grapheme::classes().is_basic_vowel(c)
}

/// One vowel GROUP = one syllable peak (orthographic approximation): counts
/// maximal runs of vowel letters. "run" → 1, "visit" → 2, "admit" → 2.
fn vowel_groups(base: &str) -> usize {
    let mut groups = 0;
    let mut in_group = false;
    for c in base.chars() {
        let v = is_syllabic(c);
        if v && !in_group {
            groups += 1;
        }
        in_group = v;
    }
    groups
}

/// C-V-C doubling environment (Quirk et al. 1985 §3; the same check as the
/// derivation tool's `regular_ed`): ends consonant–vowel–consonant with the
/// final consonant not w/x/y.
fn doubles_final(base: &str) -> Option<char> {
    let cs: Vec<char> = base.chars().collect();
    let n = cs.len();
    if n < 3 {
        return None;
    }
    let (c1, v, c2) = (cs[n - 3], cs[n - 2], cs[n - 1]);
    if !is_basic_vowel(c1) && is_basic_vowel(v) && is_consonantal(c2) && !matches!(c2, 'w' | 'x') {
        Some(c2)
    } else {
        None
    }
}

/// The RULE route for the -ing form (Quirk et al. 1985 §3; Spencer 1991 §5.2
/// alternations in the generating direction). Deterministic by construction —
/// the stress-dependent cases are loaded exceptions, not guesses here.
pub fn present_participle_rule(base: &str) -> String {
    let cs: Vec<char> = base.chars().collect();
    let n = cs.len();
    // Silent-e elision: a final e after a consonant elides before the
    // vowel-initial suffix (exhale → exhaling); after a vowel it stays
    // (see → seeing, echo the analysis-side `silent_e_restoration` gate).
    if n >= 2 && cs[n - 1] == 'e' && is_consonantal(cs[n - 2]) {
        let stem: String = cs[..n - 1].iter().collect();
        return format!("{stem}ing");
    }
    // Monosyllabic C-V-C doubling (run → running). Polysyllabic doubling is
    // stress-dependent → exceptions (admit → admitting is a loaded row).
    if vowel_groups(base) == 1
        && let Some(c) = doubles_final(base)
    {
        return format!("{base}{c}ing");
    }
    format!("{base}ing")
}

/// The -ing form of an English verb lemma — dual-route (Pinker 1991): the
/// loaded AGID exception table first (memory blocks the rule), else the
/// deterministic rule route. Deterministic: exception rows are scanned in
/// the committed table's sorted order and the first match wins.
pub fn ing_form(lemma: &str) -> String {
    let lower = lemma.to_ascii_lowercase();
    // Scans the cached table in place. Cloning it here (the previous
    // `english_irregulars()` call) copied ~7,470 rows per invocation on a path
    // `lexical_lookup_all` drives per token — see [`with_irregulars`] for the
    // measurement.
    let hit = with_irregulars(|rows| {
        rows.iter()
            .find(|entry| entry.kind == IrregularKind::PresentParticiple && entry.lemma == lower)
            .map(|entry| entry.surface.clone())
    });
    match hit {
        Some(surface) => surface,
        None => present_participle_rule(&lower),
    }
}

/// The RULE route for the regular `-ed` participle/preterite (Quirk et al.
/// 1985 §3.x; Spencer 1991 §5.2 alternations in the generating direction):
/// the productive candidate SET a base licenses, not a single canonical
/// spelling — like [`regular_plural_candidates`] and unlike
/// [`present_participle_rule`], English regular `-ed` affixation has more
/// than one attested regular sub-pattern (final-`e` takes bare `-d`,
/// consonant+`y` takes `-ied`, a C-V-C base doubles), so this returns every
/// candidate a productive rule can produce and lets the caller test
/// membership rather than equality.
///
/// This is the SAME rule the build-time AGID-exception extractor already
/// uses to decide "is this inflected form the productive default, or a
/// genuine irregular worth recording?"
/// (`english::irregular::regenerate`'s own `regular_ed`) — promoted here so
/// the runtime dual-route check below shares it rather than keeping two
/// independent copies of one cited rule, exactly as
/// [`regular_plural_candidates`]'s own doc records for the plural.
pub fn regular_past_participle_candidates(base: &str) -> Vec<String> {
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

/// Is `surface` the PAST/PASSIVE PARTICIPLE of the verb lemma `base` —
/// dual-route (Pinker 1991), the participial twin of [`is_plural_form_of`]:
///
/// 1. MEMORY, participle slot: if the loaded AGID table carries any
///    [`IrregularKind::PastParticiple`] row for `base`, those rows are the
///    ONLY accepted surfaces ("give" → "given"; "gave" is refused here
///    because AGID files it under the preterite slot instead).
/// 2. MEMORY, syncretic slot: else, if the table carries an
///    [`IrregularKind::PastTense`] row for `base` but no participle row,
///    that preterite surface IS the participle. This is AGID's own layout
///    contract, not an inference of ours: the extractor
///    (`english::irregular::regenerate`) records a participle row only for
///    the four-group `past | pp | -ing | -s` layout, and documents that the
///    three-group layout is `past | -ing | -s` — i.e. AGID asserts the two
///    slots coincide ("say" → "said", "hold" → "held").
/// 3. RULE: else, membership in [`regular_past_participle_candidates`]. This
///    is where English's regular syncretism lives — "provided" fills BOTH
///    the preterite and the participle slot (Huddleston & Pullum 2002 Ch.3
///    §1.4), which is precisely why a `-ed` surface needs an explicit
///    participial reading rather than only the finite one.
pub fn is_past_participle_form_of(base: &str, surface: &str) -> bool {
    let lower_base = base.to_ascii_lowercase();
    let lower_surface = surface.to_ascii_lowercase();
    // One in-place scan of the cached table for both memory slots — the same
    // no-clone discipline `ing_form`/`is_plural_form_of` document above.
    let memory_verdict = with_irregulars(|rows| {
        let mut participles = rows
            .iter()
            .filter(|e| e.kind == IrregularKind::PastParticiple && e.lemma == lower_base)
            .peekable();
        if participles.peek().is_some() {
            return Some(participles.any(|e| e.surface == lower_surface));
        }
        let mut preterites = rows
            .iter()
            .filter(|e| e.kind == IrregularKind::PastTense && e.lemma == lower_base)
            .peekable();
        if preterites.peek().is_some() {
            return Some(preterites.any(|e| e.surface == lower_surface));
        }
        None
    });
    match memory_verdict {
        Some(verdict) => verdict,
        None => regular_past_participle_candidates(&lower_base).contains(&lower_surface),
    }
}

/// The consonant-before-a-final-`y` test (Quirk et al. 1985 §3.21: "baby" →
/// "babies", but "boy" → "boys" because the `y` follows a vowel) — shared by
/// the plural rule below.
fn ends_consonant_y(base: &str) -> Option<&str> {
    let stem = base.strip_suffix('y')?;
    match stem.chars().last() {
        Some(c) if !is_basic_vowel(c) => Some(stem),
        _ => None,
    }
}

/// The RULE route for regular plural nouns (Quirk et al. 1985 §3.21): the
/// productive candidate SET a base licenses, not a single canonical
/// spelling — unlike [`present_participle_rule`], English regular
/// pluralization has more than one attested regular sub-pattern for some
/// bases (a final sibilant or `-o` takes `-es`: "bus" → "buses", "potato" →
/// "potatoes"; a final `f`/`fe` alternates to `-ves`: "leaf" → "leaves"),
/// so this returns every candidate a productive rule can produce and lets
/// the caller test membership rather than equality. The SAME rule the
/// build-time AGID-exception extractor already uses to decide "is this
/// inflected form the productive default, or a genuine irregular worth
/// recording?" (`english::irregular::regenerate::regular_plurals`) —
/// promoted here so the runtime dual-route check below can share it rather
/// than keep two independent copies of the same cited rule.
pub fn regular_plural_candidates(base: &str) -> Vec<String> {
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

/// Is `surface` a genuine plural inflection of the noun lemma `base` —
/// dual-route (Pinker 1991): the loaded AGID exception table first (memory
/// route; an irregular lemma's committed row is the ONLY accepted surface,
/// blocking the rule route exactly as [`ing_form`] blocks it for verbs —
/// "child" → "children" only, "childs" is rejected even though `+s` would
/// otherwise be a candidate), else membership in
/// [`regular_plural_candidates`] (rule route) when `base` carries no
/// irregular plural row at all.
pub fn is_plural_form_of(base: &str, surface: &str) -> bool {
    let lower_base = base.to_ascii_lowercase();
    let lower_surface = surface.to_ascii_lowercase();
    // Same in-place scan as `ing_form` above, for the same reason: this runs
    // per lemmatized stem candidate per token inside `lexical_lookup_all`.
    // `None` = the base carries no irregular plural row at all, which is the
    // condition for falling through to the rule route.
    let irregular_verdict = with_irregulars(|rows| {
        let mut matching = rows
            .iter()
            .filter(|e| e.kind == IrregularKind::PluralNoun && e.lemma == lower_base)
            .peekable();
        if matching.peek().is_some() {
            Some(matching.any(|e| e.surface == lower_surface))
        } else {
            None
        }
    });
    match irregular_verdict {
        Some(verdict) => verdict,
        None => regular_plural_candidates(&lower_base).contains(&lower_surface),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_rule_route_covers_the_predictable_spellings() {
        // Plain affixation, silent-e elision, vowel-final e-keeper, and
        // monosyllabic doubling — the deterministic Quirk §3 / Spencer §5.2
        // sub-cases.
        assert_eq!(present_participle_rule("cough"), "coughing");
        assert_eq!(present_participle_rule("exhale"), "exhaling");
        assert_eq!(present_participle_rule("see"), "seeing");
        assert_eq!(present_participle_rule("run"), "running");
        assert_eq!(present_participle_rule("visit"), "visiting");
        assert_eq!(present_participle_rule("carry"), "carrying");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_memory_route_blocks_the_rule() {
        // Loaded AGID exceptions win over the rule route (Pinker 1991:
        // "if a word can provide its own form from memory, the regular rule
        // is blocked"): ie→y (die → dying), the suppletive-paradigm "being",
        // k-insertion (traffic → trafficking), stress-driven doubling
        // (admit → admitting).
        assert_eq!(ing_form("die"), "dying");
        assert_eq!(ing_form("be"), "being");
        assert_eq!(ing_form("traffic"), "trafficking");
        assert_eq!(ing_form("admit"), "admitting");
        // ...and rule-route verbs pass straight through.
        assert_eq!(ing_form("cough"), "coughing");
        assert_eq!(ing_form("exhale"), "exhaling");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_participle_rule_route_covers_the_predictable_ed_spellings() {
        // The four productive `-ed` sub-patterns (Quirk et al. 1985 §3.x):
        // plain affixation, final-e bare `-d`, consonant+y → `-ied`,
        // monosyllabic C-V-C doubling.
        for (base, want) in [
            ("provide", "provided"),
            ("perform", "performed"),
            ("deliver", "delivered"),
            ("carry", "carried"),
            ("stop", "stopped"),
        ] {
            assert!(
                regular_past_participle_candidates(base)
                    .iter()
                    .any(|c| c == want),
                "{base} must license {want}: {:?}",
                regular_past_participle_candidates(base)
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn the_participle_memory_route_blocks_the_rule_and_the_preterite() {
        // Slot 1 — a lemma WITH its own loaded AGID participle row accepts
        // ONLY that row: "given", never the regular "*gived", and never the
        // PRETERITE "gave" (which AGID files under its own slot).
        assert!(is_past_participle_form_of("give", "given"));
        assert!(!is_past_participle_form_of("give", "gived"));
        assert!(!is_past_participle_form_of("give", "gave"));
        assert!(is_past_participle_form_of("write", "written"));
        assert!(!is_past_participle_form_of("write", "wrote"));
        // Slot 2 — AGID's three-group layout asserts the two slots coincide,
        // so the recorded preterite IS the participle ("said", "held").
        assert!(is_past_participle_form_of("say", "said"));
        assert!(is_past_participle_form_of("hold", "held"));
        // Rule route — the regular verb's syncretic `-ed` fills both slots.
        assert!(is_past_participle_form_of("provide", "provided"));
        assert!(is_past_participle_form_of("perform", "performed"));
        assert!(is_past_participle_form_of("deliver", "delivered"));
        assert!(is_past_participle_form_of("expose", "exposed"));
        // ...and refuses a surface no route licenses.
        assert!(!is_past_participle_form_of("provide", "providing"));
        assert!(!is_past_participle_form_of("provide", "provides"));
    }

    #[pr4xis::praxis_value(Verifiable, Deterministic)]
    #[test]
    fn generation_inverts_through_the_lemmatizer_orthography() {
        // The forward rule and the analysis-side allomorphy (Spencer §5.2,
        // allomorphy.rs) are the SAME alternations in opposite directions:
        // stripping "ing" from the generated form and expanding through the
        // cited inverse rules recovers the lemma.
        use super::super::allomorphy::english_allomorphy_rules;
        for lemma in ["cough", "exhale", "run", "carry", "visit", "see"] {
            let surface = present_participle_rule(lemma);
            let bare = surface.strip_suffix("ing").expect("rule appends ing");
            let mut candidates = vec![bare.to_string()];
            for rule in english_allomorphy_rules() {
                candidates.extend((rule.expand)(bare, &surface, "ing"));
            }
            assert!(
                candidates.iter().any(|c| c == lemma),
                "no inverse of {surface} recovers {lemma}: {candidates:?}"
            );
        }
    }
}

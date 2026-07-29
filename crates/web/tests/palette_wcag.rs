//! The palette WCAG gate — "the engine audits its own page", as a build-time test.
//!
//! Parses the shipped `docs/chat/tokens.css` into the theming ontology's
//! `Palette` (by `ColorSlot::key()` matching + `Rgb::from_hex`, the SAME lowering
//! the wasm `verify_palette` export uses) and asserts, for BOTH themes, that
//! every foreground/background pair the base16 token system renders clears its
//! WCAG 2.1 AA contrast demand — via the ontology's OWN axioms and contrast math,
//! never a ratio recomputed here. A palette regression (a hand-edited hex that
//! drops below AA) then fails a vanilla `cargo test`, which is the backing
//! guarantee for the self-audit exhibit.
//!
//! A failing pair is REPORTED with its computed ratio and required threshold —
//! the gate never weakens the threshold to pass.

use pr4xis::category::FinitelyGenerated;
use pr4xis::ontology::Axiom;
use pr4xis_domains::applied::hmi::theming::base16::{ColorSlot, Polarity};
use pr4xis_domains::applied::hmi::theming::ontology::{
    Palette, RenderedPairsMeetAa, WcagForegroundContrast, detect_polarity, rendered_pairs,
};
use pr4xis_domains::natural::colors::rgb::Rgb;
use pr4xis_domains::natural::colors::srgb;
use std::fs;
use std::path::PathBuf;

/// The shipped design-token stylesheet — the single source of truth for the
/// palette, reached from `crates/web/`.
fn tokens_css() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("chat")
        .join("tokens.css")
        .canonicalize()
        .expect("docs/chat/tokens.css must be reachable from crates/web/");
    fs::read_to_string(path).expect("docs/chat/tokens.css must be readable")
}

/// The flat property body of the CSS RULE whose selector is `selector` — the
/// occurrence immediately followed (past whitespace) by `{`. This SKIPS a mention
/// of the selector text inside the file's header comment (which documents these
/// selectors in backticks); matching the comment mention instead would land on
/// the next `{` — the wrong block — and silently read the wrong theme's tokens.
/// Each target block (`:root[data-theme="…"]`, or the `@media` light
/// `:root:not([data-theme="dark"])`) is flat — no nested braces — so the first
/// `}` closes it.
fn block<'a>(css: &'a str, selector: &str) -> &'a str {
    let mut from = 0;
    let start = loop {
        let rel = css[from..]
            .find(selector)
            .unwrap_or_else(|| panic!("tokens.css has no `{selector} {{ … }}` rule"));
        let abs = from + rel;
        // A real rule is `selector <ws> {`; a comment mention is followed by a
        // backtick or other text. Only a rule opens a block here.
        if css[abs + selector.len()..].trim_start().starts_with('{') {
            break abs;
        }
        from = abs + selector.len();
    };
    let after = &css[start..];
    let open = after.find('{').expect("rule must have an opening brace");
    let close = open
        + after[open..]
            .find('}')
            .expect("the block must have a closing brace");
    &after[open + 1..close]
}

/// Parse `--baseNN: #hex` custom properties from one CSS block into a `Palette`,
/// keyed by `ColorSlot::key()`. A slot the block does not define (e.g. the
/// base24 extension slots) is simply absent — the axioms skip absent slots.
fn parse_palette(css_block: &str) -> Palette {
    let mut palette = Palette::default();
    for slot in ColorSlot::variants() {
        let needle = format!("--{}:", slot.key());
        if let Some(pos) = css_block.find(&needle) {
            let after = &css_block[pos + needle.len()..];
            let value = &after[..after.find(';').unwrap_or(after.len())];
            if let Some(hash) = value.find('#')
                && let Some(hex) = value.get(hash..hash + 7)
                && let Some(rgb) = Rgb::from_hex(hex)
            {
                palette.insert(slot, rgb);
            }
        }
    }
    palette
}

/// The rendered pairs that FAIL their AA demand in `palette`, each as
/// `"<fg> on <bg>: <ratio> < <required>"` — the honest finding a regression
/// surfaces. Uses the ontology's own `srgb::contrast_ratio` and `ContrastDemand`
/// thresholds, so the gate cannot disagree with `RenderedPairsMeetAa`.
fn failing_pairs(palette: &Palette) -> Vec<String> {
    rendered_pairs()
        .into_iter()
        .filter_map(|pair| {
            let (fg, bg) = (
                palette.get(&pair.foreground)?,
                palette.get(&pair.background)?,
            );
            let ratio = srgb::contrast_ratio(fg, bg);
            let required = pair.demand.min_ratio(srgb::WcagLevel::AA);
            (ratio < required).then(|| {
                format!(
                    "{} on {}: {:.2} < {:.1}",
                    pair.foreground.key(),
                    pair.background.key(),
                    ratio.value,
                    required.value
                )
            })
        })
        .collect()
}

/// Assert one variant's palette: the detected polarity is as expected, and every
/// rendered pair — plus the base05/base00 primary-text axiom — clears AA.
fn assert_variant(palette: &Palette, variant: &str, expected: Polarity) {
    assert_eq!(
        detect_polarity(palette),
        Some(expected),
        "{variant}: detect_polarity must read base00 as {expected:?}"
    );
    let fails = failing_pairs(palette);
    assert!(
        fails.is_empty(),
        "{variant} tokens.css has rendered foreground/background pairs BELOW WCAG AA \
         (real finding — do NOT weaken the threshold, fix the token): {fails:?}"
    );
    assert!(
        RenderedPairsMeetAa {
            palette: palette.clone(),
        }
        .verify()
        .is_ok(),
        "{variant}: RenderedPairsMeetAa must hold (see the failing-pair list above)"
    );
    assert!(
        WcagForegroundContrast {
            palette: palette.clone(),
        }
        .verify()
        .is_ok(),
        "{variant}: base05-over-base00 primary text must clear WCAG AA"
    );
}

/// The extracted `base00` hex, for the self-check that the RIGHT rule was read.
fn base00_hex(palette: &Palette) -> Option<String> {
    palette.get(&ColorSlot::Base00).map(|c| c.to_hex())
}

#[test]
fn dark_tokens_meet_wcag_aa() {
    let css = tokens_css();
    let dark = parse_palette(block(&css, r#":root[data-theme="dark"]"#));
    // The 16 base slots must all be present in the shipped set.
    assert_eq!(
        dark.len(),
        16,
        "the dark tokens.css block must define all 16 base16 slots; got {}",
        dark.len()
    );
    // Self-check: we read the dark rule, not some other block.
    assert_eq!(
        base00_hex(&dark).as_deref(),
        Some("#0d1117"),
        "the dark block's base00 must be the file's actual #0d1117"
    );
    assert_variant(&dark, "DARK", Polarity::Dark);
}

#[test]
fn light_tokens_meet_wcag_aa() {
    let css = tokens_css();
    let light = parse_palette(block(&css, r#":root[data-theme="light"]"#));
    assert_eq!(
        light.len(),
        16,
        "the light tokens.css block must define all 16 base16 slots; got {}",
        light.len()
    );
    // Self-check (the guard that would have caught the header-comment mismatch):
    // the light rule's base00 is the file's actual #ffffff, NOT a dark value.
    assert_eq!(
        base00_hex(&light).as_deref(),
        Some("#ffffff"),
        "the light block's base00 must be the file's actual #ffffff"
    );
    assert_variant(&light, "LIGHT", Polarity::Light);

    // The other light shape — the `@media (prefers-color-scheme: light)` rule —
    // carries the SAME light set; handle it too and confirm it agrees, so both
    // light sources are correctly scoped.
    let media_light = parse_palette(block(&css, r#":root:not([data-theme="dark"])"#));
    assert_eq!(
        base00_hex(&media_light).as_deref(),
        Some("#ffffff"),
        "the @media light block's base00 must also be #ffffff"
    );
    assert_variant(&media_light, "LIGHT(@media)", Polarity::Light);
}

#[test]
fn dark_and_light_blocks_are_distinct() {
    // The regression guard: the two theme rules must extract DIFFERENT base00
    // values. Equal values mean the extractor grabbed the same rule for both
    // themes — exactly the failure a selector matched inside a comment produced.
    let css = tokens_css();
    let dark = parse_palette(block(&css, r#":root[data-theme="dark"]"#));
    let light = parse_palette(block(&css, r#":root[data-theme="light"]"#));
    assert_ne!(
        base00_hex(&dark),
        base00_hex(&light),
        "the dark and light blocks must extract different base00 values"
    );
}

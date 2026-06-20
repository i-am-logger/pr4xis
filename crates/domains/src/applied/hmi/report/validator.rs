#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::data_provisioning::decoders::theme_collection::{self, ThemeCollection};
use crate::applied::data_provisioning::raw_source_prx::raw_source_bytes_embedded;
use crate::applied::hmi::theming::base16::{ColorSlot, Polarity};
use crate::applied::hmi::theming::ontology::{
    LuminanceMonotonicity, Palette, WcagForegroundContrast,
};
use crate::natural::colors::Rgb;
use crate::natural::colors::srgb;
/// Theme validation against praxis axioms — for evaluation.
///
/// Loads the Base16/Base24 named color schemes from the registered
/// `tinted_schemes` source through the generalized content-addressed `.prx`
/// gate (`raw_source_prx` + `decoders::theme_collection`) — NOT from a git
/// submodule worktree — and validates each palette against the ontology axioms.
use pr4xis::category::FinitelyGenerated;
use pr4xis::ontology::Axiom;

/// The committed Base16/Base24 color-scheme collection `.prx` — the
/// content-addressed envelope carrying the deterministic directory archive of
/// every named-scheme YAML. The raw theme tree is FETCHED (the
/// i-am-logger/tinted-schemes fork) and gitignored; only this `.prx` ships,
/// loaded through the generalized fail-closed `[compact_archive_signatures]`
/// gate. `pr4xis update` regenerates the raw + this `.prx`.
const TINTED_SCHEMES_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/themes/tinted-schemes-2025.prx"
));

/// Load the Base16/Base24 color-scheme collection from the committed `.prx`
/// through the generalized gate, decoding the directory archive into its
/// `path → bytes` theme set. Panics fail-closed (a build-time invariant, like
/// every other committed-`.prx` consumer) if the `.prx` is absent / unpinned /
/// fails the content gate, or if the decoded archive is malformed — never a
/// silent empty load (the false-green the OLiA phase taught us to forbid).
#[must_use]
pub fn load_theme_collection() -> ThemeCollection {
    let bytes = raw_source_bytes_embedded("tinted_schemes", "2025", TINTED_SCHEMES_PRX);
    theme_collection::decode(&bytes)
        .unwrap_or_else(|e| panic!("tinted_schemes committed .prx archive failed to decode: {e}"))
}
/// Result of validating a single theme variant.
#[derive(Debug)]
/// Result of validating a single theme variant.
///
/// Inspired by W3C EARL (Evaluation and Report Language):
/// theme = TestSubject, axiom = TestCriterion, pass/fail = OutcomeValue
pub struct ThemeResult {
    pub theme: String,
    pub variant: String,
    pub scheme: String,
    pub slots_found: usize,
    pub luminance_monotone: bool,
    pub wcag_aa: bool,
    pub contrast_ratio: Option<f64>,
    /// Theme polarity classified from the base00 background luminance.
    /// `None` = the background slot is absent, so polarity is indeterminable
    /// (the honest "unknown" — a typed absence, not a `"unknown"` string).
    pub polarity: Option<Polarity>,
    /// Luminance trace: (slot_key, luminance) for base00-base07
    pub luminance_ramp: Vec<(String, f64)>,
    /// Where monotonicity breaks: index of first violation (None if monotone)
    pub mono_break_at: Option<usize>,
}
/// Parse a base16 YAML theme file into a Palette.
pub fn parse_yaml_theme(content: &str) -> Option<Palette> {
    let mut palette = Palette::new();

    let in_palette = content.contains("palette:");
    let lines: Vec<&str> = content.lines().collect();

    let mut reading_palette = !in_palette; // if no palette: key, assume flat format

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with("palette:") {
            reading_palette = true;
            continue;
        }

        if reading_palette {
            // Stop at next top-level key
            if !trimmed.is_empty()
                && !trimmed.starts_with("base0")
                && !trimmed.starts_with("base1")
                && !trimmed.starts_with('#')
                && !trimmed.starts_with("base")
                && trimmed.contains(':')
                && !trimmed.starts_with(' ')
                && in_palette
            {
                break;
            }

            // Parse baseXX: "#rrggbb" (YAML) or baseXX = "#rrggbb" (TOML)
            for slot in ColorSlot::variants() {
                let key = slot.key();
                let is_yaml = trimmed.starts_with(&format!("{key}:"))
                    || trimmed.starts_with(&format!("{key} :"));
                let is_toml = trimmed.starts_with(&format!("{key} ="))
                    || trimmed.starts_with(&format!("{key}="));

                if is_yaml || is_toml {
                    let delimiter = if is_toml { '=' } else { ':' };
                    if let Some(hex_part) = trimmed.split(delimiter).nth(1) {
                        // Take the first whitespace token FIRST (isolates the
                        // value — quoted `"#hex"` or bare `#hex` — from any
                        // trailing ` # comment`), THEN strip surrounding quotes.
                        // Ordering matters: trimming quotes before splitting left
                        // the closing quote glued to the value of a commented line
                        // (`"#ed5a56" # red` → `#ed5a56"`), dropping real slots.
                        let hex = hex_part
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .trim_matches('"')
                            .trim_matches('\'');
                        let hex = if hex.starts_with('#') {
                            hex
                        } else {
                            &format!("#{hex}")
                        };
                        if let Some(rgb) = Rgb::from_hex(hex) {
                            palette.insert(slot, rgb);
                        }
                    }
                }
            }
        }
    }

    if palette.is_empty() {
        None
    } else {
        Some(palette)
    }
}
/// Detailed validation result with trace data.
pub struct ValidationDetail {
    pub monotone: bool,
    pub wcag_aa: bool,
    pub contrast_ratio: Option<f64>,
    pub luminance_ramp: Vec<(String, f64)>,
    pub mono_break_at: Option<usize>,
}

/// Validate a palette against all axioms, returning trace data.
pub fn validate_palette(palette: &Palette) -> ValidationDetail {
    let mono_axiom = LuminanceMonotonicity {
        palette: palette.clone(),
    };
    let contrast_axiom = WcagForegroundContrast {
        palette: palette.clone(),
    };

    let cr = match (
        palette.get(&ColorSlot::Base00),
        palette.get(&ColorSlot::Base05),
    ) {
        (Some(bg), Some(fg)) => Some(srgb::contrast_ratio(fg, bg)),
        _ => None,
    };

    // Compute luminance ramp trace
    let ramp_slots = [
        ColorSlot::Base00,
        ColorSlot::Base01,
        ColorSlot::Base02,
        ColorSlot::Base03,
        ColorSlot::Base04,
        ColorSlot::Base05,
        ColorSlot::Base06,
        ColorSlot::Base07,
    ];
    let luminance_ramp: Vec<(String, f64)> = ramp_slots
        .iter()
        .filter_map(|s| {
            palette
                .get(s)
                .map(|rgb| (s.key().to_string(), srgb::relative_luminance(rgb)))
        })
        .collect();

    // Find where monotonicity breaks
    let mono_break_at = if luminance_ramp.len() >= 2 {
        // If first pair is increasing, check for any decrease (and vice versa)
        if luminance_ramp[0].1 < luminance_ramp[1].1 {
            // Expect increasing — find first decrease
            luminance_ramp
                .windows(2)
                .position(|w| w[0].1 >= w[1].1)
                .map(|p| p + 1)
        } else {
            // Expect decreasing — find first increase
            luminance_ramp
                .windows(2)
                .position(|w| w[0].1 <= w[1].1)
                .map(|p| p + 1)
        }
    } else {
        None
    };

    ValidationDetail {
        monotone: mono_axiom.verify().is_ok(),
        wcag_aa: contrast_axiom.verify().is_ok(),
        contrast_ratio: cr,
        luminance_ramp,
        mono_break_at,
    }
}
/// Validate one theme file (already loaded from the collection archive) into a
/// [`ThemeResult`], or `None` if its YAML carries no parseable palette. The
/// `scheme` is the corpus tag (`base16` / `base24`), `theme` the scheme name,
/// `variant` the file stem.
fn validate_theme_file(
    scheme: &str,
    theme: &str,
    variant: &str,
    content: &str,
) -> Option<ThemeResult> {
    let palette = parse_yaml_theme(content)?;
    let detail = validate_palette(&palette);

    // Classify polarity from the base00 background luminance into the typed
    // `Polarity`; an absent background is `None` (indeterminable).
    let polarity = palette.get(&ColorSlot::Base00).map(|bg| {
        if srgb::is_dark(bg) {
            Polarity::Dark
        } else {
            Polarity::Light
        }
    });

    Some(ThemeResult {
        theme: theme.to_string(),
        variant: variant.to_string(),
        scheme: scheme.to_string(),
        slots_found: palette.len(),
        luminance_monotone: detail.monotone,
        wcag_aa: detail.wcag_aa,
        contrast_ratio: detail.contrast_ratio,
        polarity,
        luminance_ramp: detail.luminance_ramp,
        mono_break_at: detail.mono_break_at,
    })
}

/// Scan and validate EVERY Base16/Base24 scheme in the committed collection
/// `.prx`, loaded through the generalized content-addressed gate (NOT a git
/// submodule). Each archived theme file's collection path is
/// `<scheme>/<theme>/<variant>.<ext>` (or `<scheme>/<variant>.<ext>` for a
/// loose top-level scheme); the leading path component is the corpus tag
/// (`base16` / `base24`). The result is one [`ThemeResult`] per parseable
/// palette across the whole corpus.
///
/// This is the generalized replacement for the old `std::fs`-of-submodule scan:
/// the bytes come from the gated `.prx`, so the load can never silently read a
/// stale or missing submodule worktree.
#[must_use]
pub fn scan_loaded_themes() -> Vec<ThemeResult> {
    scan_collection(&load_theme_collection())
}

/// Validate every theme file in an already-decoded collection — the pure core
/// of [`scan_loaded_themes`], split out so tests can drive it on a synthetic
/// collection without touching the committed `.prx`.
#[must_use]
pub fn scan_collection(collection: &ThemeCollection) -> Vec<ThemeResult> {
    let mut results = Vec::new();
    for file in collection {
        // Only YAML scheme files carry palettes.
        if !(file.path.ends_with(".yaml") || file.path.ends_with(".yml")) {
            continue;
        }
        let Ok(content) = core::str::from_utf8(&file.content) else {
            continue;
        };
        // Split the collection path `<scheme>/<theme>/<variant>.<ext>` (the
        // leading component is the corpus tag; the file stem is the variant; the
        // dir between, if any, is the theme name — falling back to the stem).
        let mut parts = file.path.split('/');
        let scheme = parts.next().unwrap_or("unknown");
        let rest: Vec<&str> = parts.collect();
        let file_name = rest.last().copied().unwrap_or("");
        let variant = file_name
            .rsplit_once('.')
            .map(|(stem, _ext)| stem)
            .unwrap_or(file_name);
        let theme = if rest.len() >= 2 {
            rest[rest.len() - 2]
        } else {
            variant
        };

        if let Some(r) = validate_theme_file(scheme, theme, variant, content) {
            results.push(r);
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applied::data_provisioning::decoders::theme_collection::{self, ThemeFile};
    use proptest::prelude::*;

    /// Workspace-relative path of the fetched-raw `.themes` archive (the
    /// gitignored, fetch-only canonical on-disk form of the collection — the
    /// staleness cross-check input).
    fn themes_archive_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/themes/tinted-schemes-2025.themes")
    }

    /// REGENERATE PATH (`--ignored`, WRITES, FETCH-ONLY): archive the fetched
    /// `tinted-schemes` theme tree into the deterministic `.themes` blob via
    /// [`theme_collection::archive_directory`], the SAME archive
    /// `pr4xis update` produces. Run by hand after fetching the source (the fork
    /// clone, or the legacy submodule worktree at `data/tinted-schemes/`), THEN
    /// `pr4xis compile --compact --lock` to emit + pin the committed `.prx`:
    /// `cargo test -p pr4xis-domains -- --ignored regenerate_tinted_schemes_archive`.
    /// The raw tree is fetch-only (gitignored); only the `.themes` blob (its
    /// canonical on-disk form) and the committed `.prx` persist.
    #[test]
    #[ignore]
    fn regenerate_tinted_schemes_archive() {
        // The fetched theme tree — the fork clone or the legacy submodule
        // worktree. `pr4xis update` would place it here.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/tinted-schemes");
        if !root.is_dir() {
            eprintln!(
                "tinted-schemes theme tree absent at {} — fetch it first \
                 (`pr4xis update` / git submodule), then re-run; skipping regenerate.",
                root.display()
            );
            return;
        }
        let blob = theme_collection::archive_directory(&root).expect("archive theme tree");
        let collection = theme_collection::decode(&blob).expect("decode freshly-archived blob");
        let out = themes_archive_path();
        std::fs::create_dir_all(out.parent().unwrap()).expect("mkdir data/themes");
        std::fs::write(&out, &blob).expect("write tinted-schemes-2025.themes");
        let addr = pr4xis_runtime::address::ContentAddress::of(&blob).to_hex();
        eprintln!(
            "regenerated {} ({} schemes, {} archive bytes) blake3 = {addr}",
            out.display(),
            collection.len(),
            blob.len()
        );
    }

    /// STALENESS GUARD (HARD-FAIL, no skip): the committed theme `.prx` loads +
    /// decodes to a collection that EQUALS the fetched-raw `.themes` archive on
    /// disk. The fetched raw is fetch-only (`pr4xis update`); a committed `.prx`
    /// with no raw to cross-check against is exactly the blind-spot this guard
    /// forbids — so an ABSENT `.themes` archive HARD-FAILS (it is not silently
    /// skipped). Mirrors `raw_source_prx::committed_prx_round_trips_to_fetched_raw_byte_exact`
    /// for the collection content type.
    #[test]
    fn committed_theme_prx_matches_fetched_raw() {
        let raw_path = themes_archive_path();
        let raw = std::fs::read(&raw_path).unwrap_or_else(|e| {
            panic!(
                "tinted_schemes FETCHED raw `{}` is absent ({e}) — it is fetch-only; \
                 run `pr4xis update` (or the submodule fetch + \
                 `--ignored regenerate_tinted_schemes_archive`) to regenerate it; the \
                 committed `.prx` cannot be staleness-checked without it",
                raw_path.display()
            )
        });
        // The committed `.prx` decodes to the SAME collection the raw archive
        // does — both the byte-archive and the decoded collection must agree.
        let from_prx = load_theme_collection();
        let from_raw =
            theme_collection::decode(&raw).expect("fetched-raw .themes archive must decode");
        assert_eq!(
            from_prx, from_raw,
            "committed tinted_schemes .prx drifted from the fetched-raw .themes \
             archive — re-run `pr4xis compile --compact --lock`"
        );
        // And the archive the `.prx` carries is byte-exact the fetched raw.
        let bytes = raw_source_bytes_embedded("tinted_schemes", "2025", TINTED_SCHEMES_PRX);
        assert_eq!(
            bytes, raw,
            "committed tinted_schemes .prx archive bytes drifted from the fetched-raw \
             .themes file — re-run `pr4xis compile --compact --lock`"
        );
    }

    proptest! {
        /// FORALL round-trip (the user-required property): a generated set of
        /// well-formed Base16 schemes → `.themes` archive → committed-`.prx`
        /// envelope → gated load → decode recovers the SAME schemes, and each
        /// recovered YAML re-parses to a full-ramp palette. This composes the
        /// collection ⇄ archive lens (`theme_collection`) with the bytes ⇄ `.prx`
        /// lens (`raw_source_prx`) — the exact load path the validator runs — so
        /// a break anywhere in {archive, encode, gate, decode, parse} fails HERE.
        #[test]
        fn prop_theme_collection_round_trips_through_prx(
            schemes in proptest::collection::vec(
                // 16 well-formed 24-bit hex slots → a synthetic base16 YAML.
                proptest::collection::vec(0u32..=0xFF_FFFF, 16..=16),
                1..12,
            )
        ) {
            use crate::applied::data_provisioning::raw_source_prx::{
                emit_raw_source_prx, load_raw_source_prx_gated, raw_source_archive_address,
            };
            use crate::applied::data_provisioning::registry::LockDigest;

            // The Base16 core ramp base00..base0F — the 16 reserved slots every
            // Base16 scheme binds (the first 16 of `ColorSlot::variants()`).
            let core_ramp: [ColorSlot; 16] = [
                ColorSlot::Base00, ColorSlot::Base01, ColorSlot::Base02, ColorSlot::Base03,
                ColorSlot::Base04, ColorSlot::Base05, ColorSlot::Base06, ColorSlot::Base07,
                ColorSlot::Base08, ColorSlot::Base09, ColorSlot::Base0A, ColorSlot::Base0B,
                ColorSlot::Base0C, ColorSlot::Base0D, ColorSlot::Base0E, ColorSlot::Base0F,
            ];

            // Build a synthetic Base16 collection: one YAML scheme per generated
            // slot vector, at `base16/scheme_<i>/default.yaml`.
            let mut files = Vec::new();
            for (i, slots) in schemes.iter().enumerate() {
                let mut yaml = String::from("system: \"base16\"\npalette:\n");
                for (slot, rgb) in core_ramp.iter().zip(slots) {
                    yaml.push_str(&format!("  {}: \"#{rgb:06x}\"\n", slot.key()));
                }
                files.push(ThemeFile {
                    path: format!("base16/scheme_{i}/default.yaml"),
                    content: yaml.into_bytes(),
                });
            }

            // Archive → wrap in the raw-source `.prx` → gated load → decode.
            let archive = theme_collection::encode_collection(&files);
            let prx = emit_raw_source_prx("tinted_schemes", "2025", &archive);
            let pin = LockDigest::address(raw_source_archive_address(&prx));
            let loaded_bytes = load_raw_source_prx_gated(&prx, &pin, "tinted_schemes@2025")
                .map_err(|e| TestCaseError::fail(format!("gated load: {e}")))?;
            let decoded = theme_collection::decode(&loaded_bytes)
                .map_err(|e| TestCaseError::fail(format!("decode: {e}")))?;

            // Round-trip equals the input (in sorted-path order — already sorted).
            let mut expect = files.clone();
            expect.sort_by(|a, b| a.path.cmp(&b.path));
            prop_assert_eq!(&decoded, &expect);

            // Every recovered scheme re-parses to a FULL-ramp palette — the
            // structural-validity leg the validator depends on.
            for file in &decoded {
                let content = core::str::from_utf8(&file.content).unwrap();
                let palette = parse_yaml_theme(content)
                    .ok_or_else(|| TestCaseError::fail("recovered scheme did not parse"))?;
                for slot in &core_ramp {
                    prop_assert!(
                        palette.get(slot).is_some(),
                        "recovered scheme missing slot {}", slot.key()
                    );
                }
            }
        }
    }

    #[test]
    fn test_parse_catppuccin_mocha() {
        let yaml = r##"
system: "base16"
name: "Catppuccin Mocha"
author: "https://github.com/catppuccin/catppuccin"
variant: "dark"
palette:
  base00: "#1e1e2e"
  base01: "#313244"
  base02: "#45475a"
  base03: "#6c7086"
  base04: "#a6adc8"
  base05: "#cdd6f4"
  base06: "#f5e0dc"
  base07: "#b4befe"
  base08: "#f38ba8"
  base09: "#fab387"
  base0A: "#f9e2af"
  base0B: "#a6e3a1"
  base0C: "#94e2d5"
  base0D: "#89b4fa"
  base0E: "#cba6f7"
  base0F: "#f2cdcd"
"##;
        let palette = parse_yaml_theme(yaml).unwrap();
        assert_eq!(palette.len(), 16);
    }

    #[test]
    fn test_catppuccin_monotonicity() {
        // Catppuccin Mocha base06 (rosewater) and base07 (lavender) have
        // lower luminance than base05 (text), so strict monotonicity fails.
        // This is a real finding — many popular themes violate this axiom.
        let yaml = r##"
palette:
  base00: "#1e1e2e"
  base01: "#313244"
  base02: "#45475a"
  base03: "#6c7086"
  base04: "#a6adc8"
  base05: "#cdd6f4"
  base06: "#f5e0dc"
  base07: "#b4befe"
  base08: "#f38ba8"
  base09: "#fab387"
  base0A: "#f9e2af"
  base0B: "#a6e3a1"
  base0C: "#94e2d5"
  base0D: "#89b4fa"
  base0E: "#cba6f7"
  base0F: "#f2cdcd"
"##;
        let palette = parse_yaml_theme(yaml).unwrap();
        let detail = validate_palette(&palette);
        // Catppuccin Mocha fails monotonicity: base06 (rosewater) < base05 (text)
        assert!(!detail.monotone);
        // Should have a break point
        assert!(detail.mono_break_at.is_some());
    }

    #[test]
    fn test_bad_contrast_detected() {
        let yaml = r##"
palette:
  base00: "#1e1e2e"
  base01: "#202030"
  base02: "#252535"
  base03: "#303040"
  base04: "#353545"
  base05: "#2a2a3a"
  base06: "#2f2f3f"
  base07: "#343444"
  base08: "#ff0000"
  base09: "#ff8800"
  base0A: "#ffff00"
  base0B: "#00ff00"
  base0C: "#00ffff"
  base0D: "#0000ff"
  base0E: "#ff00ff"
  base0F: "#884400"
"##;
        let palette = parse_yaml_theme(yaml).unwrap();
        let detail = validate_palette(&palette);
        assert!(!detail.wcag_aa, "should fail WCAG AA with similar fg/bg");
        assert!(detail.contrast_ratio.unwrap() < 4.5);
    }

    /// REAL EXERCISE (the false-green guard): load the WHOLE Base16/Base24
    /// scheme corpus from the committed `.prx` THROUGH THE GENERALIZED GATE and
    /// assert it is (a) non-empty and (b) structurally valid — every scheme that
    /// parses to a palette carries the full Base16 ramp `base00`..`base0F`. This
    /// is the test the OLiA phase taught us the load path MUST have: a loader
    /// that silently reads an empty/absent source can no longer pass, because
    /// the gated `.prx` load panics fail-closed on absence and these assertions
    /// HARD-FAIL on an empty or malformed corpus. No skip, no early return.
    #[test]
    fn real_exercise_loads_and_validates_themes() {
        let collection = load_theme_collection();
        assert!(
            !collection.is_empty(),
            "tinted_schemes .prx decoded to an EMPTY collection — the committed \
             archive carries no schemes (false-green: the load path must never be \
             silently empty)"
        );

        let results = scan_loaded_themes();
        assert!(
            !results.is_empty(),
            "scan_loaded_themes found NO parseable palettes in the loaded corpus — \
             the theme-load path is a false-green"
        );

        // STRUCTURAL VALIDITY: every parseable scheme carries the full Base16
        // ramp base00..base0F (the 16 slots both Base16 and Base24 share; Base24
        // adds base10..base17 on top, which Base16 schemes legitimately omit). We
        // re-parse each YAML file (the collection is the ground truth) and assert
        // the 16 core slots are present — so the load path can never silently
        // degrade to half-loaded palettes.
        let full_ramp = [
            ColorSlot::Base00,
            ColorSlot::Base01,
            ColorSlot::Base02,
            ColorSlot::Base03,
            ColorSlot::Base04,
            ColorSlot::Base05,
            ColorSlot::Base06,
            ColorSlot::Base07,
            ColorSlot::Base08,
            ColorSlot::Base09,
            ColorSlot::Base0A,
            ColorSlot::Base0B,
            ColorSlot::Base0C,
            ColorSlot::Base0D,
            ColorSlot::Base0E,
            ColorSlot::Base0F,
        ];
        let mut checked = 0usize;
        for file in &collection {
            if !(file.path.ends_with(".yaml") || file.path.ends_with(".yml")) {
                continue;
            }
            let Ok(content) = core::str::from_utf8(&file.content) else {
                continue;
            };
            let Some(palette) = parse_yaml_theme(content) else {
                continue;
            };
            for slot in &full_ramp {
                assert!(
                    palette.get(slot).is_some(),
                    "scheme `{}` is missing slot `{}` — a Base16/Base24 palette \
                     must bind every reserved slot base00..base0F",
                    file.path,
                    slot.key()
                );
            }
            checked += 1;
        }
        assert!(
            checked > 0,
            "no Base16/Base24 scheme passed structural validation — the corpus \
             decoded but carried no full-ramp palettes (false-green)"
        );

        let total = results.len();
        let mono_pass = results.iter().filter(|r| r.luminance_monotone).count();
        let wcag_pass = results.iter().filter(|r| r.wcag_aa).count();
        let dark_count = results
            .iter()
            .filter(|r| r.polarity == Some(Polarity::Dark))
            .count();
        let light_count = results
            .iter()
            .filter(|r| r.polarity == Some(Polarity::Light))
            .count();

        println!("\n═══════════════════════════════════");
        println!("  Theme Validation Results (from committed .prx)");
        println!("═══════════════════════════════════");
        println!("  Total schemes:          {}", total);
        println!("  Full-ramp validated:    {}", checked);
        println!(
            "  Luminance monotone:     {} ({:.0}%)",
            mono_pass,
            mono_pass as f64 / total as f64 * 100.0
        );
        println!(
            "  WCAG AA compliant:      {} ({:.0}%)",
            wcag_pass,
            wcag_pass as f64 / total as f64 * 100.0
        );
        println!("  Dark themes:            {}", dark_count);
        println!("  Light themes:           {}", light_count);
        println!("═══════════════════════════════════\n");
    }

    /// The Base16 AND Base24 corpora are BOTH present in the loaded collection —
    /// the archive captured the full theme set (not just one family). HARD-FAILS
    /// if either family is missing, so a half-archived `.prx` cannot pass.
    #[test]
    fn loaded_corpus_covers_base16_and_base24() {
        let results = scan_loaded_themes();
        let base16 = results.iter().filter(|r| r.scheme == "base16").count();
        let base24 = results.iter().filter(|r| r.scheme == "base24").count();
        assert!(
            base16 > 0,
            "the loaded collection has NO base16 schemes — the archive is incomplete"
        );
        assert!(
            base24 > 0,
            "the loaded collection has NO base24 schemes — the archive is incomplete"
        );
        println!("base16 schemes: {base16}, base24 schemes: {base24}");
    }
}

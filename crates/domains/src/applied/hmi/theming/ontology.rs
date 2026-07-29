#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use hashbrown::HashMap;

/// Theming ontology — formal structure of color schemes.
///
/// Connects base16/base24 slots to color science (sRGB, WCAG).
/// Axioms enforce scheme invariants that every valid theme must satisfy.
use super::base16::{ColorSlot, Polarity, SemanticRole};
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::natural::colors::oklab;
use crate::natural::colors::rgb::Rgb;
use crate::natural::colors::srgb;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Quality};

/// A concrete color palette: binds each slot to an Rgb color.
pub type Palette = HashMap<ColorSlot, Rgb>;

// ── Qualities ──

/// Quality: the semantic role of a color slot.
#[derive(Debug, Clone)]
pub struct SlotRole;

impl Quality for SlotRole {
    type Individual = ColorSlot;
    type Value = SemanticRole;
    fn get(&self, slot: &ColorSlot) -> Option<SemanticRole> {
        Some(slot.role())
    }
}

/// Quality: ANSI terminal index for a slot.
#[derive(Debug, Clone)]
pub struct AnsiIndex;

impl Quality for AnsiIndex {
    type Individual = ColorSlot;
    type Value = u8;
    fn get(&self, slot: &ColorSlot) -> Option<u8> {
        slot.ansi_index()
    }
}

/// A position in the base16 monotone luminance ramp (base00–base07).
///
/// Base16's `styling.md` defines these eight slots as a single ramp running
/// from darkest (`Base00`) to lightest (`Base07`, dark scheme — or the
/// reverse for a light scheme): a closed ordinal, not an arbitrary `u8` that
/// would let an out-of-ramp index type-check. `Ord` is derived from the
/// declaration order, which mirrors the ramp order exactly, so comparisons
/// (`<`, `<=`, …) stay meaningful.
///
/// Source: base16 spec (tinted-theming/base16-spec) `styling.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RampStep {
    Base00,
    Base01,
    Base02,
    Base03,
    Base04,
    Base05,
    Base06,
    Base07,
}

impl RampStep {
    /// The base16 ramp index (0-7) this step names.
    fn from_index(i: u8) -> Option<Self> {
        Some(match i {
            0 => Self::Base00,
            1 => Self::Base01,
            2 => Self::Base02,
            3 => Self::Base03,
            4 => Self::Base04,
            5 => Self::Base05,
            6 => Self::Base06,
            7 => Self::Base07,
            _ => return None,
        })
    }
}

/// Quality: position in the monotone luminance ramp.
#[derive(Debug, Clone)]
pub struct RampPosition;

impl Quality for RampPosition {
    type Individual = ColorSlot;
    type Value = RampStep;
    fn get(&self, slot: &ColorSlot) -> Option<RampStep> {
        slot.ramp_position().and_then(RampStep::from_index)
    }
}

// ── Relationships ──

/// A morphism from a base24 bright slot to its base16 origin.
///
/// Source: base24 spec — base12 is bright variant of base08, etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrightVariantOf {
    pub bright: ColorSlot,
    pub base: ColorSlot,
}

impl Arrow for BrightVariantOf {
    type Object = ColorSlot;
    type Kind = ();
    fn source(&self) -> ColorSlot {
        self.bright
    }
    fn target(&self) -> ColorSlot {
        self.base
    }
    fn kind(&self) {}
}

/// A morphism mapping a base16 slot to an ANSI terminal index.
///
/// Source: tinted-theming shell template convention
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnsiMapping {
    pub slot: ColorSlot,
    pub ansi: ColorSlot, // the slot that maps to the same ANSI index
}

impl Arrow for AnsiMapping {
    type Object = ColorSlot;
    type Kind = ();
    fn source(&self) -> ColorSlot {
        self.slot
    }
    fn target(&self) -> ColorSlot {
        self.ansi
    }
    fn kind(&self) {}
}

// ── Category ──

/// The theming category: slots as objects, bright-variant-of as morphisms.
pub struct ThemingCategory;

impl Category for ThemingCategory {
    type Object = ColorSlot;
    type Morphism = BrightVariantOf;

    fn identity(obj: &ColorSlot) -> BrightVariantOf {
        BrightVariantOf {
            bright: *obj,
            base: *obj,
        }
    }

    fn compose(f: &BrightVariantOf, g: &BrightVariantOf) -> Option<BrightVariantOf> {
        if f.base != g.bright {
            return None;
        }
        Some(BrightVariantOf {
            bright: f.bright,
            base: g.base,
        })
    }

    fn morphisms() -> Vec<BrightVariantOf> {
        let slots = ColorSlot::variants();
        // Identity morphisms + bright variant relationships
        let mut morphisms: Vec<BrightVariantOf> = slots
            .iter()
            .map(|s| BrightVariantOf {
                bright: *s,
                base: *s,
            })
            .collect();
        // Bright variant morphisms
        for slot in &slots {
            if let Some(base) = slot.bright_variant_of() {
                morphisms.push(BrightVariantOf {
                    bright: *slot,
                    base,
                });
            }
        }
        morphisms
    }
}

// ── Palette Axioms ──

/// Luminance monotonicity: base00 through base07 must form an ordered ramp.
///
/// Source: base16 styling.md — the monotone scale from darkest to lightest.
/// For dark themes: L(base00) < L(base01) < ... < L(base07)
/// For light themes: L(base00) > L(base01) > ... > L(base07)
pub struct LuminanceMonotonicity {
    pub palette: Palette,
}

impl Axiom for LuminanceMonotonicity {
    fn verify(&self) -> Verdict {
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
        let luminances: Vec<f64> = ramp_slots
            .iter()
            .filter_map(|s| {
                self.palette
                    .get(s)
                    .map(|rgb| srgb::relative_luminance(rgb).value)
            })
            .collect();

        if luminances.len() < 8 {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }

        // Must be monotone (either all increasing or all decreasing)
        let increasing = luminances.windows(2).all(|w| w[0] <= w[1]);
        let decreasing = luminances.windows(2).all(|w| w[0] >= w[1]);
        if increasing || decreasing {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LuminanceMonotonicity",
        "base00-base07 form a monotone luminance ramp (base16 spec)",
        "base16 styling.md — tinted-theming/base16-spec"
    );
}

/// WCAG AA compliance: foreground slots must have >= 4.5:1 contrast against background.
///
/// Source: WCAG 2.1 SC 1.4.3
pub struct WcagForegroundContrast {
    pub palette: Palette,
}

impl Axiom for WcagForegroundContrast {
    fn verify(&self) -> Verdict {
        let bg = match self.palette.get(&ColorSlot::Base00) {
            Some(c) => c,
            None => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let fg = match self.palette.get(&ColorSlot::Base05) {
            Some(c) => c,
            None => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        if srgb::wcag_compliant(fg, bg, srgb::WcagLevel::AA) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "WcagForegroundContrast",
        "foreground (base05) has >= 4.5:1 contrast against background (base00) (WCAG AA)",
        "W3C WCAG 2.1 SC 1.4.3 (Contrast Minimum)"
    );
}

/// The WCAG contrast a rendered foreground/background pair must meet, set by
/// what the foreground IS. WCAG 2.1 SC 1.4.3 gives the text thresholds (4.5:1
/// normal, 3:1 large); SC 1.4.11 gives 3:1 for UI components and graphical
/// objects. A rendered pair carries one of these, so the axiom checks each pair
/// against the RIGHT threshold rather than one blanket ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContrastDemand {
    /// Body / muted text — WCAG 2.1 SC 1.4.3 Contrast (Minimum), 4.5:1 at AA.
    NormalText,
    /// Large or bold text, a UI component, or a graphical boundary — WCAG 2.1
    /// SC 1.4.3 (large text) & SC 1.4.11 (Non-text Contrast), 3:1 at AA.
    LargeTextOrUi,
}

impl ContrastDemand {
    /// The minimum contrast ratio this demand requires at `level`, drawn from
    /// the sRGB [`WcagLevel`](srgb::WcagLevel) thresholds — never a bare 4.5 /
    /// 3.0 literal restated here.
    pub fn min_ratio(&self, level: srgb::WcagLevel) -> Quantity {
        match self {
            ContrastDemand::NormalText => level.min_contrast_normal(),
            ContrastDemand::LargeTextOrUi => level.min_contrast_large(),
        }
    }
}

/// A foreground painted over a background that the base16 token system renders —
/// the readability pairing a contrast axiom must check, tagged with its WCAG
/// demand. WHICH slots render over which (and at what demand) is base16
/// styling.md's role model, carried here as typed data rather than an ad-hoc
/// list buried in a UI or serialization layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderedPair {
    pub foreground: ColorSlot,
    pub background: ColorSlot,
    pub demand: ContrastDemand,
}

/// The WCAG demand at which `slot` renders as a readable foreground over the
/// default background, or `None` when the slot is itself a background (base00-02,
/// base07 "Light Background", base10-11) and is never painted as a foreground.
///
/// Grounded in base16 styling.md's per-slot roles: base03 (Comments) and
/// base04-06 (Dark / Default / Light Foreground) render as readable TEXT
/// (SC 1.4.3); base08-0F (chromatic accents) and the base24 bright variants
/// base12-17 render as syntax / heading / UI foregrounds — non-text or large
/// elements (SC 1.4.11 / SC 1.4.3 large). Two ramp slots are dual-role and are
/// resolved by the spec's naming: base03 is comment TEXT here (the readability-
/// conservative reading over its line-highlight background use); base07 is the
/// "Light Background" — in a light theme it equals base00, so treating it as a
/// foreground would be a 1:1 self-pair — a background.
fn rendered_demand(slot: ColorSlot) -> Option<ContrastDemand> {
    use ColorSlot::*;
    match slot {
        Base03 | Base04 | Base05 | Base06 => Some(ContrastDemand::NormalText),
        Base08 | Base09 | Base0A | Base0B | Base0C | Base0D | Base0E | Base0F | Base12 | Base13
        | Base14 | Base15 | Base16 | Base17 => Some(ContrastDemand::LargeTextOrUi),
        Base00 | Base01 | Base02 | Base07 | Base10 | Base11 => None,
    }
}

/// The foreground/background pairs the base16 token system renders — each
/// foreground slot (per `rendered_demand`'s role mapping) over the Default Background (base00),
/// with its WCAG demand. base00 is base16 styling.md's Default Background, the
/// surface every foreground and accent is specified against. Absent slots yield
/// no pair, so a base16-only palette produces exactly the base16 pairs.
pub fn rendered_pairs() -> Vec<RenderedPair> {
    ColorSlot::variants()
        .into_iter()
        .filter_map(|fg| {
            rendered_demand(fg).map(|demand| RenderedPair {
                foreground: fg,
                background: ColorSlot::Base00,
                demand,
            })
        })
        .collect()
}

/// Every rendered foreground/background pair meets its WCAG AA contrast demand.
///
/// Generalises [`WcagForegroundContrast`] (base05-over-base00 only) to the FULL
/// set of pairs the base16 token system renders ([`rendered_pairs`]): muted /
/// body text at 4.5:1 (SC 1.4.3) and chromatic accents / UI foregrounds at 3:1
/// (SC 1.4.11). A pair both of whose slots are present must clear its demand; an
/// absent slot is skipped (a partial palette is not spuriously failed) — the
/// same discipline [`BrightVariantBrighter`] uses.
///
/// Source: W3C WCAG 2.1 SC 1.4.3 (Contrast Minimum) & SC 1.4.11 (Non-text Contrast)
pub struct RenderedPairsMeetAa {
    pub palette: Palette,
}

impl Axiom for RenderedPairsMeetAa {
    fn verify(&self) -> Verdict {
        let ok = rendered_pairs().into_iter().all(|pair| {
            match (
                self.palette.get(&pair.foreground),
                self.palette.get(&pair.background),
            ) {
                (Some(fg), Some(bg)) => {
                    srgb::contrast_ratio(fg, bg) >= pair.demand.min_ratio(srgb::WcagLevel::AA)
                }
                // A slot absent from this palette is not a rendered pair for it.
                _ => true,
            }
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RenderedPairsMeetAa",
        "every rendered foreground/background pair meets WCAG AA (4.5:1 text, 3:1 large/UI)",
        "W3C WCAG 2.1 SC 1.4.3 (Contrast Minimum) & SC 1.4.11 (Non-text Contrast)"
    );
}

/// Bright variants must be brighter than their base counterparts.
///
/// Source: base24 spec — bright slots are lighter/more vivid versions.
pub struct BrightVariantBrighter {
    pub palette: Palette,
}

impl Axiom for BrightVariantBrighter {
    fn verify(&self) -> Verdict {
        let pairs = [
            (ColorSlot::Base12, ColorSlot::Base08),
            (ColorSlot::Base13, ColorSlot::Base0A),
            (ColorSlot::Base14, ColorSlot::Base0B),
            (ColorSlot::Base15, ColorSlot::Base0C),
            (ColorSlot::Base16, ColorSlot::Base0D),
            (ColorSlot::Base17, ColorSlot::Base0E),
        ];
        let ok = pairs.iter().all(|(bright, base)| {
            match (self.palette.get(bright), self.palette.get(base)) {
                (Some(b), Some(n)) => srgb::relative_luminance(b) >= srgb::relative_luminance(n),
                _ => true, // skip if slots not present (base16-only palette)
            }
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BrightVariantBrighter",
        "bright accent variants have higher luminance than their base (base24 spec)",
        "base24 spec — tinted-theming/base24"
    );
}

// ── Ramp series: ordinal step ramps bound to one slot ──

/// The UI role an ordinal colour ramp serves.
///
/// A ramp is not "four blues someone picked". It is a NAMED role bound to one
/// [`ColorSlot`], exactly as [`Vogix16Semantic`](super::schemes::Vogix16Semantic)
/// binds a semantic to a slot — `schemes.rs` states the governing rule: "This
/// decoupling of meaning from colour is the point of the scheme, and is why the
/// mapping is by slot/role, never by hue." A raw hex in a stylesheet is
/// precisely the violation that rule exists to prevent, so every ramp the token
/// system ships gets a variant here and derives its steps from the bound slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RampRole {
    /// The roadmap ladder's ordered stages — what is done, and what is next.
    ///
    /// Ordinal, not categorical: reordering the stages changes the meaning, so
    /// it takes one hue in monotone steps rather than distinct hues. Bound to
    /// the accent slot because a program ladder is the page's primary
    /// affordance, not a status readout — status slots (success / warning /
    /// danger) are reserved for valence, and these four classes carry none.
    ProgramStage,
}

impl RampRole {
    /// The slot every step of this ramp derives from.
    pub fn slot(&self) -> ColorSlot {
        match self {
            RampRole::ProgramStage => ColorSlot::Base0D,
        }
    }

    /// The semantic role the bound slot fills.
    pub fn semantic_role(&self) -> SemanticRole {
        self.slot().role()
    }

    /// The CSS custom-property stem, without the step index — the same
    /// typed-role-to-token-key projection `Vogix16Semantic::key()` performs.
    pub fn key_stem(&self) -> &'static str {
        match self {
            RampRole::ProgramStage => "--stage",
        }
    }
}

/// An ordered sequence of colour steps realising one [`RampRole`].
///
/// Steps run from MOST prominent to least. "Most prominent" is stated as
/// contrast against the surface rather than as "lightest" or "darkest",
/// because which of those it is flips between a dark and a light theme while
/// the ordering of the ladder does not — see [`RampMonotoneProminence`].
///
/// Source: Harrower & Brewer (2003), "ColorBrewer.org: An Online Tool for
/// Selecting Colour Schemes for Maps", The Cartographic Journal 40(1):27-37 —
/// single-hue sequential schemes; Few, "Bullet Graph Design Specification"
/// (rev. 2013-10-10) — "rather than using distinct hues, which might not be
/// distinguishable by those who are colorblind, encode these ranges as
/// distinct intensities … of a single hue", with a cap of five ranges and
/// three preferred.
#[derive(Debug, Clone, PartialEq)]
pub struct RampSeries {
    pub role: RampRole,
    /// Ordered most-prominent first.
    pub steps: Vec<Rgb>,
}

impl RampSeries {
    /// The slot every step derives from.
    pub fn slot(&self) -> ColorSlot {
        self.role.slot()
    }

    /// The CSS custom-property names this ramp publishes: `--stage-1 … --stage-N`.
    /// One-based, because the step index is an ordinal position in the ladder
    /// rather than an array offset.
    pub fn step_keys(&self) -> Vec<String> {
        (1..=self.steps.len())
            .map(|i| format!("{}-{}", self.role.key_stem(), i))
            .collect()
    }

    /// Few's cap on a qualitative range set: at most five, three preferred.
    ///
    /// Source: Few, "Bullet Graph Design Specification" (rev. 2013-10-10).
    pub fn max_steps() -> usize {
        5
    }

    /// The minimum Oklab lightness gap that keeps two adjacent steps readable
    /// as different steps.
    ///
    /// Perceptual, so it is stated in Oklab L (uniform) rather than WCAG
    /// relative luminance (linear in light): see
    /// [`oklab`]. The threshold is the one the
    /// runnable ordinal-ramp validator enforces (`ORDINAL_MIN_DL`), which
    /// operationalises Brewer's requirement that adjacent classes in a
    /// single-hue sequential scheme stay distinguishable.
    pub fn min_step_separation() -> Quantity {
        Quantity::from_unit(0.06, &unit::UNITLESS)
    }

    /// The maximum Oklab hue spread across the whole series before it stops
    /// being one hue and becomes a categorical palette wearing a ramp's
    /// clothes. Same source as [`min_step_separation`](Self::min_step_separation).
    pub fn max_hue_spread() -> Quantity {
        Quantity::from_unit(40.0, &unit::DEGREE)
    }
}

/// Oklab hue angle, as a typed [`Quantity`] over one full turn.
///
/// `atan2` already yields radians, so the quantity is built in radians and no
/// conversion happens here. Returning a typed angle rather than a bare `f64`
/// is load-bearing, not stylistic: [`Quantity::from_unit`] normalises to SI, so
/// a threshold declared in [`unit::DEGREE`] and a measurement left in raw
/// degrees are silently off by 180/π. Comparing `Quantity` to `Quantity` makes
/// that class of mistake impossible to write.
fn oklab_hue(color: &Rgb) -> Quantity {
    let lab = oklab::to_oklab(color);
    let mut radians = lab.b.value.atan2(lab.a.value);
    if radians < 0.0 {
        radians += core::f64::consts::TAU;
    }
    Quantity::from_unit(radians, &unit::RADIAN)
}

/// The smallest arc between two hue angles — never more than a half turn.
fn hue_arc(a: &Quantity, b: &Quantity) -> Quantity {
    let d = (a.value - b.value).abs();
    let arc = if d > core::f64::consts::PI {
        core::f64::consts::TAU - d
    } else {
        d
    };
    Quantity::from_unit(arc, &unit::RADIAN)
}

/// Every step of a ramp derives from the ONE slot the ramp is bound to.
///
/// Verified as hue agreement, on two counts, because neither implies the
/// other: the series' own hue spread stays within
/// [`RampSeries::max_hue_spread`] (it is ONE hue, not a categorical palette in
/// a ramp's clothes), AND every step sits within that same arc of the bound
/// slot's hue (it is one hue *derived from this slot*, not some other hue that
/// happens to be internally consistent). That is the checkable content of
/// "derives from the slot" — it is what fails when someone drops a hand-picked
/// hex into the ramp because it looked right, which is the failure mode
/// `schemes.rs`'s slot/role rule exists to prevent.
///
/// Source: Harrower & Brewer (2003), The Cartographic Journal 40(1):27-37 —
/// a sequential scheme is one hue; Few, Bullet Graph Design Specification
/// (rev. 2013-10-10) — intensities of a single hue, for colour-blind safety.
pub struct RampStepsShareSlot {
    pub series: RampSeries,
    pub palette: Palette,
}

impl Axiom for RampStepsShareSlot {
    fn verify(&self) -> Verdict {
        let Some(anchor) = self.palette.get(&self.series.slot()) else {
            // The slot is absent from this palette, so there is nothing to
            // derive from and nothing to check — the same partial-palette
            // discipline BrightVariantBrighter uses.
            return Ok(Box::new(SimpleProof::new(self.meta())));
        };
        let anchor_hue = oklab_hue(anchor);
        let limit = RampSeries::max_hue_spread();
        let hues: Vec<Quantity> = self.series.steps.iter().map(oklab_hue).collect();
        // The series is one hue …
        let internally_one_hue = hues
            .iter()
            .all(|a| hues.iter().all(|b| hue_arc(a, b) <= limit));
        // … and it is THIS slot's hue.
        let anchored = hues.iter().all(|h| hue_arc(h, &anchor_hue) <= limit);
        let ok = !self.series.steps.is_empty()
            && self.series.steps.len() <= RampSeries::max_steps()
            && internally_one_hue
            && anchored;
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RampStepsShareSlot",
        "every step of a ramp shares the hue of the one slot the ramp is bound to",
        "Harrower & Brewer (2003), The Cartographic Journal 40(1):27-37 — single-hue sequential schemes"
    );
}

/// Prominence decreases monotonically along a ramp, in BOTH polarities.
///
/// Stated as contrast against the surface, not as luminance: in a dark theme
/// the leading step is the LIGHTEST and in a light theme it is the DARKEST, so
/// "lighter"/"darker" flips between themes while "stands out most against the
/// surface" does not. That theme-independence is the whole point — the ladder's
/// first stage must lead the eye in either polarity.
///
/// Composes with [`LuminanceMonotonicity`], which makes the same
/// ordered-ramp claim about the base16 background scale in luminance terms;
/// this one is about an accent-derived ramp read against a surface.
///
/// Every step must additionally clear the non-text contrast floor, since a
/// ramp step paints a graphical object.
///
/// Source: W3C WCAG 2.1 SC 1.4.11 (Non-text Contrast) for the floor;
/// Harrower & Brewer (2003) for the ordered-scheme requirement.
pub struct RampMonotoneProminence {
    pub series: RampSeries,
    pub surface: Rgb,
}

impl Axiom for RampMonotoneProminence {
    fn verify(&self) -> Verdict {
        let contrasts: Vec<Quantity> = self
            .series
            .steps
            .iter()
            .map(|s| srgb::contrast_ratio(s, &self.surface))
            .collect();
        if contrasts.len() < 2 {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        let floor = ContrastDemand::LargeTextOrUi.min_ratio(srgb::WcagLevel::AA);
        let strictly_decreasing = contrasts.windows(2).all(|w| w[0].value > w[1].value);
        let all_clear_floor = contrasts.iter().all(|c| *c >= floor);
        if strictly_decreasing && all_clear_floor {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RampMonotoneProminence",
        "contrast against the surface strictly decreases along a ramp, every step clearing 3:1",
        "W3C WCAG 2.1 SC 1.4.11 (Non-text Contrast); Harrower & Brewer (2003)"
    );
}

/// Adjacent ramp steps are perceptually separated.
///
/// Monotone ordering alone does not make a ramp readable: four steps can
/// decrease in contrast and still look like one colour. The separation is
/// stated in Oklab lightness, which is perceptually uniform, so one threshold
/// means the same perceived gap at every point on the scale — WCAG relative
/// luminance would not, being linear in light rather than in perception.
///
/// Source: Harrower & Brewer (2003), The Cartographic Journal 40(1):27-37 —
/// adjacent classes of a sequential scheme must remain distinguishable;
/// Ottosson (2020) for the perceptual lightness axis the threshold is stated in.
pub struct RampStepSeparation {
    pub series: RampSeries,
}

impl Axiom for RampStepSeparation {
    fn verify(&self) -> Verdict {
        let lightnesses: Vec<f64> = self
            .series
            .steps
            .iter()
            .map(|s| oklab::lightness(s).value)
            .collect();
        if lightnesses.len() < 2 {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        let min = RampSeries::min_step_separation().value;
        let ok = lightnesses.windows(2).all(|w| (w[0] - w[1]).abs() >= min);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RampStepSeparation",
        "adjacent ramp steps differ by at least the minimum Oklab lightness gap",
        "Harrower & Brewer (2003), The Cartographic Journal 40(1):27-37; Ottosson (2020) Oklab"
    );
}

/// Polarity detection: derive dark/light from base00 luminance.
///
/// Source: base16 convention — base00 is the default background.
pub fn detect_polarity(palette: &Palette) -> Option<Polarity> {
    let bg = palette.get(&ColorSlot::Base00)?;
    if srgb::is_dark(bg) {
        Some(Polarity::Dark)
    } else {
        Some(Polarity::Light)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dark_palette() -> Palette {
        let mut p = HashMap::new();
        // Catppuccin Mocha-like dark palette
        p.insert(ColorSlot::Base00, Rgb::new(30, 30, 46)); // dark bg
        p.insert(ColorSlot::Base01, Rgb::new(49, 50, 68));
        p.insert(ColorSlot::Base02, Rgb::new(69, 71, 90));
        p.insert(ColorSlot::Base03, Rgb::new(88, 91, 112));
        p.insert(ColorSlot::Base04, Rgb::new(108, 112, 134));
        p.insert(ColorSlot::Base05, Rgb::new(205, 214, 244)); // light fg
        p.insert(ColorSlot::Base06, Rgb::new(216, 222, 233));
        p.insert(ColorSlot::Base07, Rgb::new(236, 239, 244));
        // Accents
        p.insert(ColorSlot::Base08, Rgb::new(243, 139, 168)); // red
        p.insert(ColorSlot::Base09, Rgb::new(250, 179, 135)); // orange
        p.insert(ColorSlot::Base0A, Rgb::new(249, 226, 175)); // yellow
        p.insert(ColorSlot::Base0B, Rgb::new(166, 227, 161)); // green
        p.insert(ColorSlot::Base0C, Rgb::new(148, 226, 213)); // cyan
        p.insert(ColorSlot::Base0D, Rgb::new(137, 180, 250)); // blue
        p.insert(ColorSlot::Base0E, Rgb::new(203, 166, 247)); // purple
        p.insert(ColorSlot::Base0F, Rgb::new(242, 205, 205)); // brown
        p
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn test_category_laws() {
        pr4xis::category::laws::assert_category_laws::<ThemingCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_slot_role_quality() {
        let role = SlotRole;
        assert_eq!(role.get(&ColorSlot::Base00), Some(SemanticRole::Background));
        assert_eq!(role.get(&ColorSlot::Base05), Some(SemanticRole::Foreground));
        assert_eq!(role.get(&ColorSlot::Base08), Some(SemanticRole::Accent));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_ansi_quality() {
        let ansi = AnsiIndex;
        assert_eq!(ansi.get(&ColorSlot::Base00), Some(0));
        assert_eq!(ansi.get(&ColorSlot::Base08), Some(1));
        assert_eq!(ansi.get(&ColorSlot::Base05), Some(7));
        // 16 slots have ANSI indices
        assert_eq!(ansi.individuals_with().len(), 16);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_luminance_monotonicity() {
        let palette = dark_palette();
        assert!(LuminanceMonotonicity { palette }.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_wcag_foreground_contrast() {
        let palette = dark_palette();
        assert!(WcagForegroundContrast { palette }.verify().is_ok());
    }

    /// [`dark_palette`] is a code-editor scheme (Catppuccin) whose comment /
    /// dim slots (base03/base04) sit below UI AA on purpose. Lift just those two
    /// to AA-clearing muted greys so the full rendered-pair axiom has a passing
    /// witness. (The SHIPPED tokens.css is checked live by
    /// `crates/web/tests/palette_wcag.rs`; this fixture only exercises the axiom
    /// logic.)
    fn aa_dark_palette() -> Palette {
        let mut p = dark_palette();
        p.insert(ColorSlot::Base03, Rgb::new(139, 148, 158)); // #8b949e
        p.insert(ColorSlot::Base04, Rgb::new(154, 164, 176)); // #9aa4b0
        p
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_contrast_demand_thresholds() {
        // The thresholds come from the sRGB WcagLevel, not a restated literal.
        assert_eq!(
            ContrastDemand::NormalText
                .min_ratio(srgb::WcagLevel::AA)
                .value,
            4.5
        );
        assert_eq!(
            ContrastDemand::LargeTextOrUi
                .min_ratio(srgb::WcagLevel::AA)
                .value,
            3.0
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_rendered_pairs_derivation() {
        let pairs = rendered_pairs();
        // The default text pair renders as NormalText.
        assert!(pairs.iter().any(|p| p.foreground == ColorSlot::Base05
            && p.background == ColorSlot::Base00
            && p.demand == ContrastDemand::NormalText));
        // A muted-text slot (comments) renders as NormalText.
        assert!(
            pairs.iter().any(
                |p| p.foreground == ColorSlot::Base03 && p.demand == ContrastDemand::NormalText
            )
        );
        // A chromatic accent renders as a UI / large pair (3:1).
        assert!(pairs.iter().any(
            |p| p.foreground == ColorSlot::Base0D && p.demand == ContrastDemand::LargeTextOrUi
        ));
        // Backgrounds are never a foreground: base00 (Default) and base07
        // ("Light Background", which equals base00 in a light theme).
        assert!(!pairs.iter().any(|p| p.foreground == ColorSlot::Base00));
        assert!(!pairs.iter().any(|p| p.foreground == ColorSlot::Base07));
        // Every pair renders over the default background.
        assert!(pairs.iter().all(|p| p.background == ColorSlot::Base00));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_rendered_pairs_meet_aa() {
        assert!(
            RenderedPairsMeetAa {
                palette: aa_dark_palette()
            }
            .verify()
            .is_ok()
        );
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn test_rendered_pairs_meet_aa_fails_on_low_muted_text() {
        // Drop the dim-text slot to a near-background grey: SC 1.4.3 fails, so
        // the axiom must refuse (it is not vacuously true).
        let mut p = aa_dark_palette();
        p.insert(ColorSlot::Base04, Rgb::new(60, 62, 74));
        assert!(RenderedPairsMeetAa { palette: p }.verify().is_err());
    }

    // ── Ramp series ──
    //
    // The four steps below are the EXACT values docs/chat/tokens.css ships for
    // `--stage-1 … --stage-4` in each polarity, so these axioms are checking
    // the ramp the page actually paints. Steps 1 and 2 are `var()` aliases in
    // the stylesheet (`--link-fg`, `--base0D`); the resolved hexes are written
    // out here because an axiom verifies colours, not indirection.

    /// The shipped dark-theme program ramp, most prominent first.
    fn dark_stage_ramp() -> RampSeries {
        RampSeries {
            role: RampRole::ProgramStage,
            steps: vec![
                Rgb::new(0x79, 0xc0, 0xff), // --stage-1 Done (= --link-fg)
                Rgb::new(0x58, 0xa6, 0xff), // --stage-2 Next (= --base0D)
                Rgb::new(0x45, 0x85, 0xd6), // --stage-3 Then
                Rgb::new(0x3f, 0x6c, 0xb4), // --stage-4 Hold
            ],
        }
    }

    /// The shipped light-theme program ramp, most prominent first.
    fn light_stage_ramp() -> RampSeries {
        RampSeries {
            role: RampRole::ProgramStage,
            steps: vec![
                Rgb::new(0x00, 0x47, 0x8a), // --stage-1 Done (= --link-fg)
                Rgb::new(0x00, 0x5e, 0xa2), // --stage-2 Next (= --base0D)
                Rgb::new(0x2b, 0x7a, 0xb4), // --stage-3 Then
                Rgb::new(0x53, 0x91, 0xc0), // --stage-4 Hold
            ],
        }
    }

    /// `--base01`, the card surface each ramp step is painted on.
    fn dark_card_surface() -> Rgb {
        Rgb::new(0x16, 0x1b, 0x22)
    }
    fn light_card_surface() -> Rgb {
        Rgb::new(0xf6, 0xf8, 0xfa)
    }
    /// `--base00`, the page background behind the card.
    fn dark_page() -> Rgb {
        Rgb::new(0x0d, 0x11, 0x17)
    }
    fn light_page() -> Rgb {
        Rgb::new(0xff, 0xff, 0xff)
    }

    /// A palette whose accent slot is the ramp's anchor, for the slot axiom.
    fn ramp_palette(anchor: Rgb) -> Palette {
        let mut p = HashMap::new();
        p.insert(ColorSlot::Base0D, anchor);
        p
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_ramp_role_binds_one_slot() {
        // The ramp is bound by SLOT and ROLE, never by a hue someone chose.
        assert_eq!(RampRole::ProgramStage.slot(), ColorSlot::Base0D);
        assert_eq!(RampRole::ProgramStage.semantic_role(), SemanticRole::Accent);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_ramp_step_keys_are_one_based_ordinals() {
        assert_eq!(
            dark_stage_ramp().step_keys(),
            vec!["--stage-1", "--stage-2", "--stage-3", "--stage-4"]
        );
    }

    /// The hue threshold and the hue measurement must be compared as typed
    /// angles, never as bare floats.
    ///
    /// `Quantity::from_unit` normalises to SI, so `max_hue_spread()` — declared
    /// in [`unit::DEGREE`] — carries **radians** in `.value`. Comparing that
    /// `0.698` against an arc left in raw degrees rejects every real ramp,
    /// because a legitimate 6.6° separation reads as `6.6 > 0.698`. Pinned here
    /// because the mistake is invisible at the call site and the types are the
    /// only thing that catch it.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_hue_threshold_and_measurement_share_si_units() {
        let limit = RampSeries::max_hue_spread();
        // Declared as 40 degrees; stored, like every Quantity, in SI.
        assert!((limit.value - 40.0_f64.to_radians()).abs() < 1e-12);

        let arc = hue_arc(
            &oklab_hue(&Rgb::new(0x58, 0xa6, 0xff)),
            &oklab_hue(&Rgb::new(0x79, 0xc0, 0xff)),
        );
        // Two adjacent steps of the shipped dark ramp: a small angle, well
        // inside the limit — which is only true if both sides are in SI.
        assert!(arc.value.to_degrees() < 10.0, "{}", arc.value.to_degrees());
        assert!(arc <= limit);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_ramp_steps_share_slot_both_themes() {
        for (ramp, anchor) in [
            (dark_stage_ramp(), Rgb::new(0x58, 0xa6, 0xff)),
            (light_stage_ramp(), Rgb::new(0x00, 0x5e, 0xa2)),
        ] {
            assert!(
                RampStepsShareSlot {
                    series: ramp,
                    palette: ramp_palette(anchor),
                }
                .verify()
                .is_ok()
            );
        }
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn test_ramp_steps_share_slot_refuses_a_foreign_hue() {
        // Swapping one step for the magenta slot is exactly the "it looked
        // right" failure the slot/role rule exists to prevent.
        let mut ramp = dark_stage_ramp();
        ramp.steps[2] = Rgb::new(0xbc, 0x8c, 0xff); // --base0E, magenta
        assert!(
            RampStepsShareSlot {
                series: ramp,
                palette: ramp_palette(Rgb::new(0x58, 0xa6, 0xff)),
            }
            .verify()
            .is_err()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_ramp_monotone_prominence_in_both_polarities() {
        // The theme-independent statement: prominence against the surface
        // decreases along the ladder whether the theme is dark or light, even
        // though "lighter"/"darker" flips between them. Checked against the
        // card surface AND the page background, since a card can sit on either.
        for (ramp, surfaces) in [
            (dark_stage_ramp(), [dark_card_surface(), dark_page()]),
            (light_stage_ramp(), [light_card_surface(), light_page()]),
        ] {
            for surface in surfaces {
                assert!(
                    RampMonotoneProminence {
                        series: ramp.clone(),
                        surface,
                    }
                    .verify()
                    .is_ok()
                );
            }
        }
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn test_ramp_monotone_prominence_refuses_a_reordered_ramp() {
        let mut ramp = dark_stage_ramp();
        ramp.steps.swap(0, 3);
        assert!(
            RampMonotoneProminence {
                series: ramp,
                surface: dark_card_surface(),
            }
            .verify()
            .is_err()
        );
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn test_ramp_monotone_prominence_refuses_a_step_below_the_ui_floor() {
        // A last step too close to the surface still decreases monotonically,
        // but stops being a visible graphical object (SC 1.4.11).
        let mut ramp = dark_stage_ramp();
        ramp.steps[3] = Rgb::new(0x1d, 0x27, 0x3a);
        assert!(
            RampMonotoneProminence {
                series: ramp,
                surface: dark_card_surface(),
            }
            .verify()
            .is_err()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_ramp_step_separation_both_themes() {
        assert!(
            RampStepSeparation {
                series: dark_stage_ramp()
            }
            .verify()
            .is_ok()
        );
        assert!(
            RampStepSeparation {
                series: light_stage_ramp()
            }
            .verify()
            .is_ok()
        );
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn test_ramp_step_separation_refuses_near_duplicate_steps() {
        // Monotone and one hue, but two steps a reader cannot tell apart.
        let mut ramp = dark_stage_ramp();
        ramp.steps[1] = Rgb::new(0x76, 0xbe, 0xfe);
        assert!(RampStepSeparation { series: ramp }.verify().is_err());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_ramp_respects_fews_qualitative_range_cap() {
        // Few caps a qualitative range set at five; the program ladder's four
        // stages sit inside it.
        assert!(dark_stage_ramp().steps.len() <= RampSeries::max_steps());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_detect_polarity_dark() {
        let palette = dark_palette();
        assert_eq!(detect_polarity(&palette), Some(Polarity::Dark));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_detect_polarity_light() {
        let mut palette = dark_palette();
        palette.insert(ColorSlot::Base00, Rgb::new(239, 241, 245)); // light bg
        assert_eq!(detect_polarity(&palette), Some(Polarity::Light));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_ramp_position_quality() {
        let ramp = RampPosition;
        assert_eq!(ramp.get(&ColorSlot::Base00), Some(RampStep::Base00));
        assert_eq!(ramp.get(&ColorSlot::Base07), Some(RampStep::Base07));
        assert_eq!(ramp.get(&ColorSlot::Base08), None); // accent, not ramp
        assert_eq!(ramp.individuals_with().len(), 8);
        // Ord is preserved: the ramp step ordering mirrors luminance order.
        assert!(RampStep::Base00 < RampStep::Base07);
        assert!(ramp.get(&ColorSlot::Base03).unwrap() < ramp.get(&ColorSlot::Base05).unwrap());
    }
}

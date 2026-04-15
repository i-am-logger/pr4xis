/// Theming — color schemes, palettes, and WCAG-enforced accessibility axioms.
///
/// The core theming ontology: base16/base24 slots, palette axioms
/// (luminance monotonicity, WCAG AA contrast, bright-variant brighter),
/// scheme mappings (Base16, Base24, Vogix16, Ansi16), variant metadata
/// (dark/light polarity), and theme-package structure.
pub mod base16;
pub mod ontology;
pub mod schemes;
pub mod theme_package;
pub mod variants;

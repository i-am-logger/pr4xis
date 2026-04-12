/// Theming ontology — color scheme specifications as formal structures.
///
/// Core theming (base16, ontology, schemes) provides the formal color model.
/// Extended modules (from vogix) provide surfaces, modes, keybindings,
/// visualization, validation, and reporting.
///
/// Sources:
/// - Base16: https://github.com/tinted-theming/home/blob/main/styling.md
/// - Base24: https://github.com/tinted-theming/base24/blob/main/styling.md
/// - ECMA-48 (5th Ed, 1991): SGR parameters 30-37, 90-97 for ANSI colors
/// - WCAG 2.1: contrast requirements for accessibility
/// - Bertin, "Semiology of Graphics" (1967): visual variables
/// - Cleveland & McGill, "Graphical Perception" (1984): perceptual task ranking
/// - Harel, "Statecharts" (1987): mode graphs, parallel regions
// Core
pub mod base16;
pub mod ontology;
pub mod schemes;

// Modes and input
pub mod keybindings;
pub mod modes;

// Theme structure
pub mod shader_params;
pub mod theme_package;
pub mod variants;

// Rendering surfaces
pub mod surfaces;

// Visualization theory
pub mod report;
pub mod report_spec;
pub mod visualization;

// Validation
pub mod validate_themes;

// Self-referential visualization
pub mod explorer;

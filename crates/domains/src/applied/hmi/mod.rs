/// HMI — Human-Machine Interface ontologies.
///
/// Umbrella for the interface layer: how pr4xis renders to and receives input
/// from humans via any surface (terminal, browser, chat, hardware RGB, shaders,
/// PDF, print). The sub-ontologies compose via functors:
///
/// ```text
/// data → visualization → surface → rendered artifact
///                         ↑
///                      theming
/// ```
///
/// Sub-ontologies:
/// - `theming`: base16/base24 color palettes, WCAG axioms, schemes, variants
/// - `surfaces`: abstract render targets (terminal, WM, hardware RGB, shaders, browser, chat)
/// - `visualization`: Bertin visual variables, Cleveland-McGill accuracy, Munzner, Wickham grammar
/// - `input`: keybindings, interaction modes
/// - `report`: data → visualization → surface → frozen artifact pipeline
/// - `explorer`: self-referential reasoning-trace visualization, shader params
///
/// Sources:
/// - Base16: https://github.com/tinted-theming/home/blob/main/styling.md
/// - Base24: https://github.com/tinted-theming/base24/blob/main/styling.md
/// - ECMA-48 (5th Ed, 1991): SGR parameters 30-37, 90-97 for ANSI colors
/// - WCAG 2.1: contrast requirements for accessibility
/// - Bertin, *Semiology of Graphics* (1967): visual variables
/// - Cleveland & McGill, "Graphical Perception" (1984): perceptual task ranking
/// - Harel, "Statecharts" (1987): mode graphs, parallel regions
/// - Beaudouin-Lafon, "Instrumental Interaction" (CHI 2000)
/// - Thimbleby, "User Interface Design with Matrix Algebra" (ACM TOCHI 2004)
pub mod explorer;
pub mod input;
pub mod report;
pub mod surfaces;
pub mod theming;
pub mod visualization;

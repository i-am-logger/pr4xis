/// Surfaces — abstract render targets for theming and content pipelines.
///
/// A Surface is anything that renders colors and content from a theme palette:
/// terminal emulators, window managers, hardware RGB devices, shaders, browsers,
/// chat UIs, PDF generators. Theme application is a functor from ThemeCategory
/// to SurfaceCategory; a theme change is a natural transformation that updates
/// all surface functors consistently.
pub mod ontology;

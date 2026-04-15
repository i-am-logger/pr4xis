/// Input — interaction modes and keybindings.
///
/// Models user input as a structured interaction: modes are states in a
/// statechart (Harel 1987), keybindings are the morphisms that trigger
/// transitions. The two together form the interaction algebra.
pub mod keybindings;
pub mod modes;

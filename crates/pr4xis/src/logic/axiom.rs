/// An axiom — a statement that must hold unconditionally.
///
/// Axioms are foundational truths about a domain. `holds()` verifies
/// the system is consistent with the axiom — the system cannot lie.
///
/// Used by both category-level structural checks (e.g. "no dead states")
/// and domain-level invariants (e.g. "energy is conserved").
///
/// The axiom's identity is its Rust type. Its human-readable name and
/// description live in the Lemon lexicon as LexicalEntries pointing to
/// this axiom via LexicalSense → reference. `description()` is a
/// transitional fallback until the lexicon is wired.
pub trait Axiom {
    /// English fallback — will be replaced by Lemon lexicon lookup.
    fn description(&self) -> &str;

    /// Verify this axiom holds.
    fn holds(&self) -> bool;
}

//! Per-crate completeness-gate support: emit this crate's own
//! `#[pr4xis::praxis_value(..)]` tag set so `scripts/constitution-gate.sh
//! pr4xis-chat` can diff it against `cargo test -p pr4xis-chat --lib -- --list`.
//! Mirrors `pr4xis::constitution::coverage_gate` (the core crate's own minimal
//! version) rather than `pr4xis-domains`'s fuller
//! `formal::meta::constitution_coverage` — this crate's tagged surface is the
//! pipeline/answer layer, not a domain corpus, so the per-guarantee partition
//! assertions belong to the richer crate, not duplicated here where they
//! would not hold honestly.

// One canonical emitter, expanded per binary. This body was hand-copied
// verbatim into four crates; `pr4xis::constitution_coverage_gate!` is that
// same body written once. Per BINARY, not per crate: linkme assembles a
// distributed slice at link time, so each test binary sees only its own tags.
pr4xis::constitution_coverage_gate!();

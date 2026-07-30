//! Per-crate completeness-gate support: emit this crate's own
//! `#[pr4xis::praxis_value(..)]` tag set so `scripts/constitution-gate.sh
//! pr4xis-runtime` can diff it against `cargo test -p pr4xis-runtime --lib --
//! --list`. Mirrors `pr4xis::constitution::coverage_gate` (the core crate's own
//! minimal version) rather than `pr4xis-domains`'s fuller
//! `formal::meta::constitution_coverage` — this crate's tagged surface is a
//! runtime kernel, not a domain corpus, so the per-guarantee partition
//! assertions (every one of the six guarantees has a primary witness) belong
//! to the richer crate, not duplicated here where they would not hold
//! honestly (this kernel's tests cluster on a handful of guarantees, and
//! forcing coverage of the rest would mean inventing secondary tags rather
//! than reporting what the tests actually witness).

// One canonical emitter, expanded per binary. This body was hand-copied
// verbatim into four crates; `pr4xis::constitution_coverage_gate!` is that
// same body written once. Per BINARY, not per crate: linkme assembles a
// distributed slice at link time, so each test binary sees only its own tags.
pr4xis::constitution_coverage_gate!();

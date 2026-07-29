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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use pr4xis::constitution::CONSTITUTION_TESTS;
    use std::fs;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn constitution_coverage() {
        let lines: Vec<String> = CONSTITUTION_TESTS
            .iter()
            .map(|t| format!("{}::{}", t.module, t.name))
            .collect();
        match std::env::var("PRAXIS_CONSTITUTION_TAGS_OUT") {
            Ok(path) => fs::write(&path, lines.join("\n")).expect("write constitution tags"),
            Err(_) => eprintln!("{}", lines.join("\n")),
        }
    }
}

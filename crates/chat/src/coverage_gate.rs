//! Per-crate completeness-gate support: emit this crate's own
//! `#[pr4xis::praxis_value(..)]` tag set so `scripts/constitution-gate.sh
//! pr4xis-chat` can diff it against `cargo test -p pr4xis-chat --lib -- --list`.
//! Mirrors `pr4xis::constitution::coverage_gate` (the core crate's own minimal
//! version) rather than `pr4xis-domains`'s fuller
//! `formal::meta::constitution_coverage` — this crate's tagged surface is the
//! pipeline/answer layer, not a domain corpus, so the per-guarantee partition
//! assertions belong to the richer crate, not duplicated here where they
//! would not hold honestly.

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

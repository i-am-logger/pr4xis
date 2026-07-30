//! The CLI's startup + subcommand contract, as an integration test.
//!
//! Two jobs, and the second is the reason this file exists at all.
//!
//! 1. CONTRACT. `.github/workflows/ci.yml` and `devenv.nix` both invoke
//!    `pr4xis update` and `pr4xis compile --compact` by name, and the `build`
//!    job's data fetch is `./target/release/pr4xis update`. Those are the
//!    load-bearing entry points of the whole pipeline, and nothing else checks
//!    that they still parse: a renamed or removed subcommand would surface as a
//!    confusing mid-job failure in CI rather than as a failing test here.
//!
//! 2. ARCHIVE MEMBERSHIP. `cargo nextest archive` builds a package's plain
//!    `[[bin]]` only when that package also has an integration-test or bench
//!    target. `crates/cli` had neither, so `target/release/pr4xis` was absent
//!    from the archive and the `build` job had to compile it in a separate
//!    `cargo build -p pr4xis-cli --release` step — which cost ~126s and was
//!    thrown away, because the `-p pr4xis-cli` resolve gives `pr4xis-domains` a
//!    different `serde_json` feature set (no `alloc`, which only
//!    `wasm-bindgen-test` pulls in) than `--workspace --all-targets` does, so
//!    the archive step rebuilt the domains rlib from scratch anyway.
//!    `crates/web` already has this shape (`crates/web/tests/`), which is
//!    exactly why `target/release/pr4xis-web` IS in the archive.
//!
//! So this file is a genuine contract test that also, as a side effect, puts
//! the binary where the archive can carry it. If it is deleted, the `build`
//! job's artifact upload silently produces nothing — `upload-artifact` defaults
//! to `if-no-files-found: warn`, so the job stays GREEN — and `lint-docs`,
//! `test`, `wasm-browser` and `corpus` all then fail at `chmod +x`.

use std::process::Command;

/// The binary under test, resolved by cargo for the integration target.
fn pr4xis() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pr4xis"))
}

/// `--help` must succeed and name every subcommand CI drives by name.
///
/// Asserting on the help text rather than on each subcommand's behaviour keeps
/// this test hermetic: `update` reaches the network and `compile` reads the
/// corpus, neither of which belongs in a smoke test.
#[test]
fn help_lists_every_subcommand_ci_invokes() {
    let out = pr4xis()
        .arg("--help")
        .output()
        .expect("the pr4xis binary must be executable");

    assert!(
        out.status.success(),
        "`pr4xis --help` exited {:?}; stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );

    let help = String::from_utf8_lossy(&out.stdout);
    // `update` and `compile` are invoked by ci.yml and devenv.nix; `chat` is the
    // default subcommand and `decompile` completes the emit/read pair.
    for sub in ["chat", "update", "compile", "decompile"] {
        assert!(
            help.contains(sub),
            "`pr4xis --help` no longer lists the `{sub}` subcommand, which CI \
             invokes by name. Renaming it breaks .github/workflows/ci.yml and \
             devenv.nix. Full help:\n{help}"
        );
    }
}

/// `compile --compact` must parse. CI runs exactly this flag pair in `test` and
/// `corpus`, and `devenv.nix`'s dev-ci runs it too; if the flag were renamed the
/// failure would appear as a mid-job parse error, not as a failing test.
#[test]
fn compile_accepts_the_compact_flag_ci_uses() {
    let out = pr4xis()
        .args(["compile", "--help"])
        .output()
        .expect("the pr4xis binary must be executable");

    assert!(
        out.status.success(),
        "`pr4xis compile --help` exited {:?}; stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );

    let help = String::from_utf8_lossy(&out.stdout);
    assert!(
        help.contains("--compact"),
        "`pr4xis compile` no longer accepts `--compact`, which ci.yml and \
         devenv.nix both invoke. Full help:\n{help}"
    );
}

/// An unknown subcommand must FAIL, not fall through to the default `chat`
/// session. Without this, a typo'd invocation in CI would start an interactive
/// chat and hang until the step timeout rather than erroring immediately.
#[test]
fn an_unknown_subcommand_is_refused() {
    let out = pr4xis()
        .arg("definitely-not-a-subcommand")
        .output()
        .expect("the pr4xis binary must be executable");

    assert!(
        !out.status.success(),
        "an unknown subcommand must be refused, not silently accepted — \
         otherwise a typo in CI starts an interactive chat and hangs"
    );
}

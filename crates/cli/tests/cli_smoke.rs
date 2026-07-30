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
//!    `cargo build -p pr4xis-cli --release` step. `crates/web` already has this
//!    shape (`crates/web/tests/`), which is exactly why
//!    `target/release/pr4xis-web` IS in the archive.
//!
//! So this file is a genuine contract test that also, as a side effect, puts
//! the binary where the archive can carry it. If it is deleted, the `build`
//! job's artifact upload silently produces nothing — `upload-artifact` defaults
//! to `if-no-files-found: warn`, so the job stays GREEN — and `lint-docs`,
//! `test`, `wasm-browser` and `corpus` all then fail at `chmod +x`.
//!
//! The assertions parse `--help` STRUCTURALLY rather than with `contains`,
//! because substring matching is not sound against this CLI's own help text:
//! `decompile`'s description reads "the inverse of `compile`", so
//! `contains("compile")` passes even with the `compile` subcommand deleted, and
//! `--defines`' description mentions "`--compact` runs", so
//! `contains("--compact")` passes even with the flag deleted. Both checks would
//! have been decorative.

use std::process::{Command, Stdio};

/// The binary under test, resolved by cargo for the integration target.
///
/// `stdin` is null so that a regression which falls through to the interactive
/// `chat` session fails immediately on EOF instead of inheriting the test
/// runner's stdin and blocking until nextest's `terminate-after`. A test whose
/// job is to prove the CLI cannot hang must not itself be able to hang.
fn pr4xis() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pr4xis"));
    cmd.stdin(Stdio::null());
    cmd
}

/// Run the binary with `args` and return stdout, asserting a clean exit.
fn help_text(args: &[&str]) -> String {
    let out = pr4xis()
        .args(args)
        .output()
        .expect("the pr4xis binary must be executable");
    assert!(
        out.status.success(),
        "`pr4xis {}` exited {:?}; stderr: {}",
        args.join(" "),
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The subcommand NAMES clap lists under `Commands:` — first token of each
/// entry line, stopping at the blank line that ends the section.
///
/// Structural on purpose: entry lines are indented and carry a description
/// after the name, and descriptions routinely name OTHER subcommands.
fn subcommands(help: &str) -> Vec<String> {
    help.lines()
        .skip_while(|l| l.trim() != "Commands:")
        .skip(1)
        .take_while(|l| !l.trim().is_empty())
        .filter_map(|l| l.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

/// Whether `flag` appears as an option DECLARATION rather than inside prose.
///
/// Declaration lines start (after trimming) with `-`, e.g. `--compact` or
/// `-h, --help`; wrapped description text is indented further and starts with
/// ordinary words. Requiring an exact token match on such a line is what keeps
/// `--defines`' mention of "`--compact` runs" from satisfying this.
fn declares_flag(help: &str, flag: &str) -> bool {
    help.lines()
        .map(str::trim)
        .filter(|l| l.starts_with('-'))
        .any(|l| {
            l.split(|c: char| c.is_whitespace() || c == ',')
                .any(|tok| tok == flag)
        })
}

/// `--help` must succeed and list every subcommand CI drives by name.
#[test]
fn help_lists_every_subcommand_ci_invokes() {
    let help = help_text(&["--help"]);
    let found = subcommands(&help);

    // `update` and `compile` are invoked by ci.yml and devenv.nix; `chat` is
    // the default subcommand and `decompile` completes the emit/read pair.
    for sub in ["chat", "update", "compile", "decompile"] {
        assert!(
            found.iter().any(|f| f == sub),
            "`pr4xis --help` no longer lists a `{sub}` subcommand. CI invokes it \
             by name, so renaming it breaks .github/workflows/ci.yml and \
             devenv.nix. Parsed command list: {found:?}\nFull help:\n{help}"
        );
    }
}

/// `compile --compact` must parse. CI runs exactly this flag pair in `test` and
/// `corpus`, and devenv.nix's dev-ci runs it too.
#[test]
fn compile_accepts_the_compact_flag_ci_uses() {
    let help = help_text(&["compile", "--help"]);
    assert!(
        declares_flag(&help, "--compact"),
        "`pr4xis compile` no longer DECLARES `--compact`, which ci.yml and \
         devenv.nix both invoke. (Checked as an option declaration, not a \
         substring — `--defines`' description also mentions `--compact`.)\n\
         Full help:\n{help}"
    );
}

/// An unknown subcommand must FAIL, not fall through to the default `chat`
/// session. Without this, a typo'd invocation in CI would start an interactive
/// chat and burn the step's whole timeout budget.
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

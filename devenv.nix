{ pkgs
, config
, lib
, ...
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ./crates/pr4xis/Cargo.toml);
  # The version is workspace-inherited (`version.workspace = true`), so read it
  # from the root `[workspace.package]` rather than the member's package table.
  workspaceToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  packageName = cargoToml.package.name;
  packageVersion = workspaceToml.workspace.package.version;
  packageDescription = cargoToml.package.description or "";
in
{
  # Set root explicitly for flake compatibility
  devenv.root = lib.mkDefault (builtins.toString ./.);

  dotenv.enable = true;
  imports = [
    ./nix/rust.nix
  ];

  # Additional packages for development
  packages = [
    pkgs.git
    pkgs.pkg-config
    pkgs.marp-cli
    pkgs.mdbook
    pkgs.miniserve
    pkgs.cargo-edit
    # Rust-native search tooling. The host shell's grep/rg/find wrappers are
    # broken in this environment ("-G: error while loading shared libraries"),
    # so make the canonical Rust replacements first-class in the devenv:
    # ripgrep = rg, fd = fd-find. Used by tooling + agents instead of the
    # broken host binaries.
    pkgs.ripgrep
    pkgs.fd
    # Mutation testing — operational error-rate measurement
    # (Daubert prong 3). Each mutant alters one expression; if the
    # tests still pass, that's a blind spot in coverage.
    pkgs.cargo-mutants
    # Faster test runner — used as cargo-mutants' --test-tool to
    # multiply mutation-testing speed (cargo-mutants per-mutant
    # cycle is dominated by test runtime).
    pkgs.cargo-nextest
    # Browser + WebDriver for the Rust-native wasm/web test layers (no
    # Node): `dev-test-wasm` (wasm-pack test --headless --firefox) and
    # `dev-e2e` (the crates/e2e fantoccini test) both drive a headless
    # Firefox via geckodriver. curl polls the web server + driver in
    # `dev-e2e` before the test runs.
    #
    # `firefox-bin` is the Mozilla-released prebuilt binary, not the
    # from-source nixpkgs build — the source build is unreliable in
    # current devenv-nixpkgs/rolling, but the prebuilt binary always
    # works. `allowUnfree` is required because Mozilla's binary
    # distribution has a non-free EULA (the source build is free).
    pkgs.firefox-bin
    pkgs.geckodriver
    pkgs.curl
  ];

  # Development scripts.
  #
  # All cargo invocations use --release. The CI workflow does the
  # same; running anything in debug locally would diverge from CI
  # semantics (different opt-level, different debug_assertions,
  # different `target/` subtree → cache miss). See
  # `.github/workflows/ci.yml`'s architecture comment and the
  # `feedback_no_debug_assert` rule.
  scripts.dev-test.exec = ''
    echo "Fetching external data (mirrors CI)..."
    cargo run -p pr4xis-cli --release --quiet -- update || {
      echo "pr4xis update failed — aborting dev-test to match CI behavior."
      exit 1
    }
    echo "Running tests via nextest --release (mirrors CI)..."
    RUSTFLAGS="-D warnings" cargo nextest run --workspace --profile ci --release
    echo "Running heavy-corpus tests (cargo test — parse each giant once)..."
    RUSTFLAGS="-D warnings" cargo test --manifest-path crates/praxis-corpus-tests/Cargo.toml --release
    echo "Running doc tests --release (nextest excludes them)..."
    RUSTFLAGS="-D warnings" cargo test --doc --workspace --release
  '';

  scripts.dev-fmt.exec = ''
    echo "Checking formatting..."
    treefmt --fail-on-change
  '';

  scripts.dev-lint.exec = ''
    echo "Running clippy --release..."
    cargo clippy --workspace --all-targets --release --quiet -- -D warnings
    cargo clippy --manifest-path crates/wasm/Cargo.toml --target wasm32-unknown-unknown --release --quiet -- -D warnings
  '';

  scripts.dev-check.exec = ''
    echo "Checking compilation..."
    cargo check --release --quiet
  '';

  scripts.dev-ci.exec = ''
    echo "Running full CI pipeline locally (everything --release, matches CI)..."
    echo "=== fmt ==="
    treefmt --fail-on-change || { echo "FAILED: fmt"; exit 1; }
    echo "=== fetch external data ==="
    cargo run -p pr4xis-cli --release --quiet -- update || { echo "FAILED: pr4xis update"; exit 1; }
    echo "=== compile compact .prx cache (parse-once fast load, matches CI) ==="
    cargo run -p pr4xis-cli --release --quiet -- compile --compact || { echo "FAILED: pr4xis compile"; exit 1; }
    echo "=== clippy (release) ==="
    cargo clippy --workspace --all-targets --release --quiet -- -D warnings || { echo "FAILED: clippy"; exit 1; }
    echo "=== docs (rustdoc, same flags as CI) ==="
    RUSTDOCFLAGS="-D warnings -D rustdoc::broken_intra_doc_links -D rustdoc::invalid_html_tags" \
      cargo doc --workspace --no-deps --release --quiet || { echo "FAILED: docs"; exit 1; }
    echo "=== doc tests (release) ==="
    RUSTFLAGS="-D warnings" cargo test --doc --workspace --release --quiet || { echo "FAILED: doc tests"; exit 1; }
    echo "=== mdbook test ==="
    mdbook test docs/ || { echo "FAILED: mdbook test"; exit 1; }
    echo "=== test (nextest --release, strict [profile.ci]) ==="
    RUSTFLAGS="-D warnings" cargo nextest run --workspace --profile ci --release || { echo "FAILED: test"; exit 1; }
    echo "=== heavy-corpus tests (cargo test — parse each giant once) ==="
    RUSTFLAGS="-D warnings" cargo test --manifest-path crates/praxis-corpus-tests/Cargo.toml --release || { echo "FAILED: corpus tests"; exit 1; }
    echo "=== clippy (wasm, release) ==="
    cargo clippy --manifest-path crates/wasm/Cargo.toml --target wasm32-unknown-unknown --release --quiet -- -D warnings || { echo "FAILED: clippy (wasm)"; exit 1; }
    echo "=== wasm native acceptance tests ==="
    RUSTFLAGS="-D warnings" cargo test --manifest-path crates/wasm/Cargo.toml --release || { echo "FAILED: wasm native acceptance tests"; exit 1; }
    echo "=== wasm browser tests ==="
    dev-test-wasm || { echo "FAILED: wasm browser tests"; exit 1; }
    echo "=== e2e (Rust WebDriver) ==="
    dev-e2e || { echo "FAILED: e2e"; exit 1; }
    echo "=== ALL PASSED ==="
  '';

  scripts.dev-build.exec = ''
    echo "Building ${packageName}..."
    cargo build --release
  '';

  scripts.dev-wasm.exec = ''
    echo "Building pr4xis-wasm..."
    cd crates/wasm
    wasm-pack build --target web --release
    echo "WASM ready at crates/wasm/pkg/"
  '';

  # Rust-native wasm browser tests (no Node): wasm-bindgen-test driven by
  # geckodriver + headless Firefox. Covers crates/wasm/tests/web.rs.
  scripts.dev-test-wasm.exec = ''
    echo "Running wasm-bindgen browser tests (headless Firefox)..."
    cd crates/wasm
    wasm-pack test --headless --firefox
  '';

  # Rust-native end-to-end click-through (no Node): build the wasm + stage
  # /sources, start pr4xis-web + geckodriver, then run the crates/e2e
  # fantoccini test which opens the page, clicks Load, and asserts the
  # source flips to Loaded. Server + driver are torn down on exit.
  scripts.dev-e2e.exec = ''
    echo "Building WASM (stages /sources)..."
    dev-wasm
    # Emit `.prx.gz` artifacts the wasm dual-load click-to-load path needs.
    # pr4xis-web serves `crates/wasm/ontologies/` at `/ontologies/`,
    # mirroring the Pages tree the release job assembles. Mirrors the
    # `.github/workflows/ci.yml` E2E step exactly so local == CI.
    echo "Emitting .prx.gz ontology artifacts (stages /ontologies)..."
    mkdir -p crates/wasm/ontologies
    cargo run -p pr4xis-domains --features "fetch codegen" --example emit_prx -- crates/wasm/ontologies
    echo "Starting pr4xis-web + geckodriver..."
    cargo run -p pr4xis-web 3000 &
    WEB=$!
    geckodriver --port 4444 &
    GECKO=$!
    cleanup() { kill "$WEB" "$GECKO" 2>/dev/null || true; }
    trap cleanup EXIT
    for _ in $(seq 1 90); do curl -sf http://localhost:3000/ >/dev/null 2>&1 && break; sleep 1; done
    for _ in $(seq 1 30); do curl -sf http://localhost:4444/status >/dev/null 2>&1 && break; sleep 1; done
    echo "Running Rust WebDriver E2E..."
    ( cd crates/e2e && cargo run )
  '';

  # Bump every direct dep in every Cargo.toml to its latest published version
  # (across-semver), then update Cargo.lock to the new resolution, then run
  # the test suite to catch any breaking-API changes. The "always be on
  # latest" policy: contributors run this periodically; new majors land in
  # PRs labelled fix(deps): with the API-migration commit alongside.
  # Mutation testing — run cargo-mutants on a focused module or
  # path. Each mutant changes one expression (e.g. `==` to `!=`,
  # `+` to `-`); a tested codebase kills the mutant by failing.
  # Surviving mutants are blind spots in the test suite. Compliance
  # win: this measures the actual operational error rate.
  #
  # Uses --in-place (no workspace copy — required since target/ is
  # large and copying it N times exhausts /tmp) plus cargo-nextest
  # which itself parallelizes tests across all available cores.
  # Net effect: mutants run serially, but each mutant's test cycle
  # uses every CPU thread on the machine.
  #
  # NOT IN CI by deliberate choice: workspace-wide is 4–8 hours per
  # run; even a focused subtree is too slow to block PR or master
  # CI cycles. Run locally on a regular cadence (per `feedback_local_
  # ci_parity`); add a scheduled cron job to ci.yml later if/when
  # the team commits to an operational-error-rate dashboard.
  #
  # Usage:
  #   dev-mutants                         — workspace-wide (slow)
  #   dev-mutants -- --file <path>        — focus on one file
  #   dev-mutants -- --package <name>     — focus on one crate
  scripts.dev-mutants.exec = ''
    echo "Running cargo-mutants --in-place + cargo-nextest..."
    echo "Tip: pass --file <path> or --package <name> to scope."
    cargo mutants --test-tool nextest --in-place "$@"
  '';

  scripts.dev-upgrade.exec = ''
    echo "=== cargo upgrade (cross-semver, edits Cargo.toml) ==="
    cargo upgrade --incompatible
    echo "=== cargo update (within-semver, edits Cargo.lock) ==="
    cargo update
    echo "=== cargo test (verify upgrades don't break) ==="
    cargo test --workspace --quiet
    echo ""
    echo "All deps updated to latest. Review the Cargo.toml + Cargo.lock"
    echo "diff and commit as 'fix(deps): bump to latest <date>' (or break"
    echo "into smaller PRs if a major-version migration touched a lot of"
    echo "call sites)."
  '';

  scripts.dev-web.exec = ''
    echo "Building WASM..."
    dev-wasm
    echo ""
    echo "Starting pr4xis-web with live reload..."
    echo "  /                — WASM chatbot"
    echo "  /decks/technical — presentation"
    echo "Watching crates/ for changes — WASM rebuilds automatically."
    echo ""
    cargo run -p pr4xis-web --release -- 4096
  '';

  # Environment variables
  env = {
    CARGO_TARGET_DIR = "./target";
  };

  # Development shell setup
  enterShell = ''
    clear
    ${pkgs.figlet}/bin/figlet "${packageName}"
    echo
    {
      ${pkgs.lib.optionalString (packageDescription != "") ''echo "• ${packageDescription}"''}
      echo -e "• \033[1mv${packageVersion}\033[0m"
      echo -e " \033[0;32m✓\033[0m Development environment ready"
    } | ${pkgs.boxes}/bin/boxes -d stone -a l -i none
    echo
    echo "Available scripts:"
    echo "  dev-ci        - Run full CI pipeline (fmt + clippy + check + test)"
    echo "  dev-test      - Run tests"
    echo "  dev-fmt       - Check formatting"
    echo "  dev-lint      - Run clippy"
    echo "  dev-check     - Check compilation"
    echo "  dev-build     - Build release"
    echo "  dev-data      - Fetch external data (WordNet, etc.) via 'pr4xis update'"
    echo "  dev-web       - Start dev server (/ = chatbot, /decks/technical = presentation)"
    echo "  dev-wasm      - Build WASM"
    echo "  dev-upgrade   - Bump all deps to latest (Cargo.toml + Cargo.lock) and run tests"
    echo "  dev-mutants   - Mutation testing via cargo-mutants (compliance: operational error rate)"
    echo ""
  '';

  scripts.dev-data.exec = ''
    echo "Fetching external data via 'pr4xis update'..."
    cargo run -p pr4xis-cli --release --quiet -- update || {
      echo "pr4xis update: one or more datasets could not be materialized."
      echo "If this is a fresh clone and the upstream release is not yet published,"
      echo "obtain the files manually and re-run 'pr4xis update --check'."
      exit 1
    }
  '';

  # https://devenv.sh/integrations/treefmt/
  treefmt = {
    enable = true;
    config = {
      settings.global.excludes = [
        ".devenv.flake.nix"
        ".devenv/"
      ];

      programs = {
        # Nix
        nixpkgs-fmt.enable = true;
        deadnix = {
          enable = true;
          no-underscore = true;
        };
        statix.enable = true;

        # Rust — use devenv toolchain (supports edition 2024)
        rustfmt = {
          enable = true;
          package = config.languages.rust.toolchainPackage;
        };

        # Shell
        shellcheck.enable = true;
        shfmt.enable = true;
      };
    };
  };

  # https://devenv.sh/git-hooks/
  git-hooks.settings.rust.cargoManifestPath = "./Cargo.toml";

  git-hooks.tools = {
    cargo = lib.mkForce config.languages.rust.toolchainPackage;
    clippy = lib.mkForce config.languages.rust.toolchainPackage;
    rustfmt = lib.mkForce config.languages.rust.toolchainPackage;
  };

  git-hooks.hooks = {
    treefmt.enable = true;
    clippy.enable = true;
  };

  # https://devenv.sh/tasks/
  tasks = {
    "test:fmt" = {
      exec = "treefmt --fail-on-change";
    };

    "test:clippy" = {
      exec = "cargo clippy --workspace --all-targets --quiet -- -D warnings && cargo clippy --manifest-path crates/wasm/Cargo.toml --target wasm32-unknown-unknown --quiet -- -D warnings";
    };

    "test:check" = {
      exec = "cargo check --quiet";
    };

    "test:unit" = {
      exec = "RUSTFLAGS='-D warnings' cargo test --quiet";
    };

    "test:wasm" = {
      exec = "RUSTFLAGS='-D warnings' cargo check --manifest-path crates/wasm/Cargo.toml --target wasm32-unknown-unknown --quiet";
    };
  };

  # https://devenv.sh/tests/
  enterTest = lib.mkForce "devenv tasks run test:fmt test:clippy test:check test:unit test:wasm";
}

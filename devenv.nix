{ pkgs
, config
, lib
, ...
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ./crates/pr4xis/Cargo.toml);
  packageName = cargoToml.package.name;
  packageVersion = cargoToml.package.version;
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
  ];

  # Development scripts
  scripts.dev-test.exec = ''
    echo "Fetching external data (mirrors CI)..."
    cargo run -p pr4xis-cli --release --quiet -- update || {
      echo "pr4xis update failed — aborting dev-test to match CI behavior."
      exit 1
    }
    echo "Running tests..."
    RUSTFLAGS="-D warnings" cargo test --workspace
  '';

  scripts.dev-fmt.exec = ''
    echo "Checking formatting..."
    treefmt --fail-on-change
  '';

  scripts.dev-lint.exec = ''
    echo "Running clippy..."
    cargo clippy --quiet -- -D warnings
    cargo clippy --manifest-path crates/wasm/Cargo.toml --target wasm32-unknown-unknown --quiet -- -D warnings
  '';

  scripts.dev-check.exec = ''
    echo "Checking compilation..."
    cargo check --quiet
  '';

  scripts.dev-ci.exec = ''
    echo "Running full CI pipeline locally..."
    echo "=== fmt ==="
    treefmt --fail-on-change || { echo "FAILED: fmt"; exit 1; }
    echo "=== clippy ==="
    cargo clippy --quiet -- -D warnings || { echo "FAILED: clippy"; exit 1; }
    echo "=== clippy (wasm) ==="
    cargo clippy --manifest-path crates/wasm/Cargo.toml --target wasm32-unknown-unknown --quiet -- -D warnings || { echo "FAILED: clippy (wasm)"; exit 1; }
    echo "=== check ==="
    cargo check --quiet || { echo "FAILED: check"; exit 1; }
    echo "=== fetch external data (mirrors CI) ==="
    cargo run -p pr4xis-cli --release --quiet -- update || { echo "FAILED: pr4xis update"; exit 1; }
    echo "=== test ==="
    RUSTFLAGS="-D warnings" cargo test --workspace --quiet || { echo "FAILED: test"; exit 1; }
    echo "=== wasm check ==="
    RUSTFLAGS="-D warnings" cargo check --manifest-path crates/wasm/Cargo.toml --target wasm32-unknown-unknown --quiet || { echo "FAILED: wasm check"; exit 1; }
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

  # Bump every direct dep in every Cargo.toml to its latest published version
  # (across-semver), then update Cargo.lock to the new resolution, then run
  # the test suite to catch any breaking-API changes. The "always be on
  # latest" policy: contributors run this periodically; new majors land in
  # PRs labelled fix(deps): with the API-migration commit alongside.
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
    cargo run -p pr4xis-web --release
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
      exec = "cargo clippy --quiet -- -D warnings && cargo clippy --manifest-path crates/wasm/Cargo.toml --target wasm32-unknown-unknown --quiet -- -D warnings";
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

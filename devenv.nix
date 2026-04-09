{ pkgs
, config
, lib
, ...
}:
let
  packageName = "praxis";
  packageVersion = "0.1.0";
  packageDescription = "Ontology-driven rule enforcement framework";
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
  ];

  # Development scripts
  scripts.dev-test.exec = ''
    echo "Running tests..."
    RUSTFLAGS="-D warnings" cargo test --workspace
  '';

  scripts.dev-fmt.exec = ''
    echo "Checking formatting..."
    cargo fmt --check
  '';

  scripts.dev-lint.exec = ''
    echo "Running clippy..."
    cargo clippy --quiet -- -D warnings
  '';

  scripts.dev-check.exec = ''
    echo "Checking compilation..."
    cargo check --quiet
  '';

  scripts.dev-ci.exec = ''
    echo "Running full CI pipeline locally..."
    echo "=== fmt ==="
    cargo fmt --check || { echo "FAILED: fmt"; exit 1; }
    echo "=== clippy ==="
    cargo clippy --quiet -- -D warnings || { echo "FAILED: clippy"; exit 1; }
    echo "=== check ==="
    cargo check --quiet || { echo "FAILED: check"; exit 1; }
    echo "=== test ==="
    RUSTFLAGS="-D warnings" cargo test --workspace --quiet || { echo "FAILED: test"; exit 1; }
    echo "=== ALL PASSED ==="
  '';

  scripts.dev-build.exec = ''
    echo "Building ${packageName}..."
    cargo build --release
  '';

  scripts.dev-wasm.exec = ''
    echo "Building praxis-wasm..."
    cd crates/wasm
    wasm-pack build --target web --release
    echo "Copying WASM to docs/chat/pkg/..."
    mkdir -p ../../docs/chat/pkg
    cp -r pkg/* ../../docs/chat/pkg/
    echo "WASM ready at docs/chat/"
  '';

  scripts.dev-site.exec = ''
    echo "Building site..."
    dev-wasm
    echo "Site ready. Serve with: cd docs && python3 -m http.server"
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
    echo ""
  '';

  # https://devenv.sh/git-hooks/
  git-hooks.settings.rust.cargoManifestPath = "./Cargo.toml";

  # Use the same Rust toolchain for git-hooks as for development
  git-hooks.tools = {
    cargo = lib.mkForce config.languages.rust.toolchainPackage;
    clippy = lib.mkForce config.languages.rust.toolchainPackage;
    rustfmt = lib.mkForce config.languages.rust.toolchainPackage;
  };

  git-hooks.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
  };

  # https://devenv.sh/tasks/
  tasks = {
    "test:fmt" = {
      exec = "cargo fmt --check";
    };

    "test:clippy" = {
      exec = "cargo clippy --quiet -- -D warnings";
    };

    "test:check" = {
      exec = "cargo check --quiet";
    };

    "test:unit" = {
      exec = "RUSTFLAGS='-D warnings' cargo test --quiet";
    };

    "test:wasm" = {
      exec = "cargo check --manifest-path crates/wasm/Cargo.toml --target wasm32-unknown-unknown --quiet";
    };
  };

  # https://devenv.sh/tests/
  # SKIP git-hooks during devenv test — hooks are a local pre-commit concern,
  # CI runs the same checks via tasks (fmt, clippy with -D warnings, tests).
  enterTest = lib.mkForce "SKIP=clippy,rustfmt devenv tasks run test:fmt test:clippy test:check test:unit test:wasm";
}

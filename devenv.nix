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
    pkgs.miniserve
  ];

  # Development scripts
  scripts.dev-test.exec = ''
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
    echo "Copying WASM to docs/chat/pkg/..."
    mkdir -p ../../docs/chat/pkg
    cp -r pkg/* ../../docs/chat/pkg/
    echo "WASM ready at docs/chat/"
  '';

  scripts.dev-site.exec = ''
    echo "Building WASM..."
    dev-wasm
    echo "Building presentation..."
    marp docs/presentations/overview.md -o docs/index.html --html
    echo "Site ready at docs/"
  '';

  scripts.dev-present.exec = ''
    echo "Building presentation..."
    marp docs/presentations/overview.md -o docs/index.html --html
    echo ""
    echo "Serving at http://localhost:8080"
    echo "  Presentation: http://localhost:8080"
    echo "  Chat: http://localhost:8080/chat/"
    echo ""
    echo "Press Ctrl+C to stop"
    brave http://localhost:8080 2>/dev/null || xdg-open http://localhost:8080 2>/dev/null &
    miniserve docs/ --port 8080 --index index.html
  '';

  scripts.dev-web.exec = ''
    echo "Building WASM..."
    dev-wasm
    echo "Building presentation..."
    marp docs/presentations/overview.md -o docs/index.html --html 2>/dev/null || true
    echo ""
    echo "Starting pr4xis-web with live reload..."
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
    echo "  dev-web       - Start dev server with live reload"
    echo "  dev-wasm      - Build WASM"
    echo ""
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

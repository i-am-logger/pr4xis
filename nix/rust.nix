{ pkgs, ... }:

{
  # Rust language configuration
  languages.rust = {
    enable = true;
    # https://devenv.sh/reference/options/#languagesrustchannel
    channel = "stable";

    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];

    targets = [
      "wasm32-unknown-unknown"
    ];

    # Warnings are errors by default for every cargo invocation in the
    # shell, matching CI. devenv folds this into the RUSTFLAGS env var
    # (languages.rust.rustflags). Cargo passes `--cap-lints allow` to
    # registry/git dependencies, so only the workspace's own crates are
    # held to this — third-party dep warnings never break the build.
    rustflags = "-D warnings";
  };

  # WASM tooling
  packages = [
    pkgs.wasm-pack
    pkgs.wasm-bindgen-cli
  ];
}

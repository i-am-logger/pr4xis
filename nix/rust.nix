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
  };

  # WASM tooling
  packages = [
    pkgs.wasm-pack
    pkgs.wasm-bindgen-cli
  ];
}

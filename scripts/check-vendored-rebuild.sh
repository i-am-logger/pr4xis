#!/usr/bin/env bash
# Build pr4xis-domains the way a downstream crate does and check two things:
#
#   1. the build script finds every committed `.prx` it reads, so the XML
#      codegen outputs are generated rather than stubbed, and
#   2. a second, unchanged build reruns nothing: pr4xis-domains is `Fresh`.
#
# The crates are laid out as `cargo package` produces them, which is also the
# layout `cargo vendor` and crates.io give a dependent: each crate unpacked on
# its own, with no workspace `praxis.toml` / `praxis.lock` around it. A consumer
# crate outside the workspace depends on the unpacked pr4xis-domains by path,
# and `[patch.crates-io]` points its pr4xis dependencies at their unpacked
# packages.
#
# Usage: scripts/check-vendored-rebuild.sh
#   GATE_DIR   scratch directory (default: a fresh `mktemp -d`, removed on exit)
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
crates=(pr4xis-derive pr4xis pr4xis-runtime pr4xis-domains)

if [ -n "${GATE_DIR:-}" ]; then
  gate=$GATE_DIR
  mkdir -p "$gate"
else
  gate=$(mktemp -d)
  trap 'rm -rf "$gate"' EXIT
fi
# The gate directory must sit outside the workspace, or the consumer would be
# taken for a workspace member and the unpacked crates would see praxis.toml.
case "$gate/" in
"$repo"/*)
  echo "GATE_DIR must be outside the repository: $gate" >&2
  exit 2
  ;;
esac

pkgid=$(cargo pkgid --manifest-path "$repo/Cargo.toml" -p pr4xis-domains)
version=${pkgid##*@}

echo "=== package ${crates[*]} (v$version) ==="
pkg_args=()
for c in "${crates[@]}"; do pkg_args+=(-p "$c"); done
cargo package --manifest-path "$repo/Cargo.toml" "${pkg_args[@]}" \
  --no-verify --allow-dirty --quiet

rm -rf "$gate/vendor" "$gate/consumer"
mkdir -p "$gate/vendor" "$gate/consumer/src"
for c in "${crates[@]}"; do
  tar -xzf "$repo/target/package/$c-$version.crate" -C "$gate/vendor"
done

cat >"$gate/consumer/Cargo.toml" <<EOF
[package]
name = "vendored-rebuild-gate"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
pr4xis-domains = { path = "../vendor/pr4xis-domains-$version" }

[patch.crates-io]
pr4xis-derive = { path = "../vendor/pr4xis-derive-$version" }
pr4xis = { path = "../vendor/pr4xis-$version" }
pr4xis-runtime = { path = "../vendor/pr4xis-runtime-$version" }

[workspace]
EOF
cat >"$gate/consumer/src/main.rs" <<'EOF'
use pr4xis_domains::social::software::markup::xml::spec_1_0::grammar;

fn main() {
    assert!(
        !grammar::CHAR_RANGES.is_empty() && grammar::is_char(u32::from('A')),
        "XML 1.0 grammar tables are the empty stub"
    );
    println!("XML 1.0 Char ranges: {}", grammar::CHAR_RANGES.len());
}
EOF
# Pin third-party versions to the workspace's own resolution.
cp "$repo/Cargo.lock" "$gate/consumer/Cargo.lock"

export CARGO_TARGET_DIR="$gate/target"
cd "$gate/consumer"

echo "=== first build ==="
cargo build --quiet
fail=0
cargo run --quiet || fail=1

echo "=== codegen outputs are generated, not stubbed ==="
for f in xml_namespace_schema_generated.rs xml_infoset_generated.rs xml_grammar_generated.rs; do
  out=$(find "$CARGO_TARGET_DIR/debug/build" -path '*/pr4xis-domains-*/out/'"$f" -printf '%T@ %p\n' |
    sort -rn | head -1 | cut -d' ' -f2-)
  if [ -z "$out" ]; then
    echo "FAIL: $f not produced" >&2
    fail=1
  elif grep -qE '^// (Stub:|.* codegen failed)' "$out"; then
    echo "FAIL: $f is a stub: $(head -1 "$out")" >&2
    fail=1
  else
    echo "ok: $f"
  fi
done

echo "=== second build (nothing changed) ==="
log=$(CARGO_LOG=cargo::core::compiler::fingerprint=info cargo build -v 2>&1)
if grep -q 'Fresh pr4xis-domains' <<<"$log" && ! grep -q 'Compiling pr4xis-domains' <<<"$log"; then
  echo "ok: pr4xis-domains is Fresh"
else
  echo "FAIL: pr4xis-domains rebuilt on an unchanged second build:" >&2
  grep -E 'dirty: |stale: ' <<<"$log" >&2 || true
  fail=1
fi

exit $fail

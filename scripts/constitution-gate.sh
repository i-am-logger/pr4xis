#!/usr/bin/env bash
# Completeness gate for the Constitution (identity-based).
#
# The constitution_coverage meta-test asserts *partition* coverage (every
# guarantee has a witness). This script asserts *completeness*: that the set of
# tagged tests is exactly the set of real tests. It diffs the live `--list`
# test names against the full paths the meta-test emits (one `PRAXIS_TAG` line
# per registered tag), in BOTH directions:
#
#   - untagged: a real test that carries no praxis_value tag.
#   - phantom:  a registered tag whose name matches no real test — almost
#               always a typo in a hand-written `register_praxis_value!` call.
#
# A count-only check cannot see these (an untagged test and a phantom tag net
# to the same count), so the gate compares identities, not counts.
#
#   scripts/constitution-gate.sh [crate] [--enforce] [--archive <file>]
#
# Without --enforce it reports the gaps (used during backfill). With --enforce
# it exits non-zero if anything is untagged or phantom (the CI gate).
#
# --archive reads a `cargo nextest archive` tarball instead of compiling. It is
# how CI runs the gate: the archive already exists (the `test` job runs from
# it), so the gate costs seconds instead of a full dev-profile compile of four
# crates. Without it the script compiles from source, which is what a local
# `dev-ci` run does.
#
# THE FEATURE SET IS THE INVARIANT, and the archive is what makes it right.
# Compiling from source runs `cargo test --lib` per crate, which resolves a
# NARROWER feature union than `--workspace` does — so feature-gated code was
# invisible to the gate and exempt from untagged=0: all 59 tests of
# `pr4xis`'s `codegen` module (lib.rs:34), pr4xis-domains' `fetch`-gated
# tests, and pr4xis-runtime's `emit`-gated tests. The archive is built
# `--workspace`, so the gate sees exactly what CI actually tests. Keep it that
# way: the gate's feature set must match the workspace-unified resolution.
#
# Notes:
#  - proptest! tests are one test function each (a property is one test).
#  - the constitution_coverage meta-test carries a tag like any other test, so
#    the gate is self-binding with no special case.
set -euo pipefail

crate="${1:-pr4xis-domains}"
shift || true
enforce=""
archive=""
while [ $# -gt 0 ]; do
  case "$1" in
  --enforce) enforce="--enforce" ;;
  --archive)
    archive="${2:?--archive needs a path}"
    shift
    ;;
  *)
    echo "unknown argument: $1" >&2
    exit 2
    ;;
  esac
  shift
done
libname="${crate//-/_}"

tagsfile=$(mktemp)
trap 'rm -f "$tagsfile"' EXIT

if [ -n "$archive" ]; then
  [ -f "$archive" ] || {
    echo "archive not found: $archive" >&2
    exit 2
  }

  # `--run-ignored all` is MANDATORY: without it the tagged #[ignore]d tests
  # vanish from `listed` while their tags remain registered, and the gate
  # reports them all as phantom.
  #
  # `binary_id(=<pkg>)` selects exactly the crate's LIB test binary —
  # integration binaries are `<pkg>::<name>`, so they do not match.
  listed=$(cargo nextest list --archive-file "$archive" --workspace-remap . \
    --run-ignored all -E "binary_id(=$crate)" --message-format json 2>/dev/null |
    jq -r --arg k "$crate" '."rust-suites"[$k].testcases // {} | keys[]' |
    sort -u)

  # Fail LOUDLY on an empty listing. An empty `listed` against an empty
  # `registered` nets to a spurious COMPLETE — the one failure mode of this
  # gate that is invisible.
  [ -n "$listed" ] || {
    echo "gate error: no tests listed for $crate in $archive" >&2
    exit 1
  }

  # One targeted run per crate, each writing its own tag file — a single
  # workspace-wide run cannot write four distinct files through one env var.
  #
  # `test(=constitution_coverage)` matches NOTHING (verified on cargo-nextest
  # 0.9.138) and makes `nextest run` exit 4, because nextest matches against
  # the full path. The regex form anchors on the final segment instead.
  PRAXIS_CONSTITUTION_TAGS_OUT="$tagsfile" \
    cargo nextest run --archive-file "$archive" --workspace-remap . \
    -E "binary_id(=$crate) and test(/(^|::)constitution_coverage\$/)" \
    --no-capture >/dev/null 2>&1
else
  # Source path (local `dev-ci`). `prx` gates pr4xis-domains' serialization
  # machinery (.prx codecs + ontology_archive / canonical_codec /
  # succinct_codec); without it those tests AND their tags are both absent, so
  # they match trivially and escape untagged=0.
  feature_args=()
  if [ "$crate" = "pr4xis-domains" ]; then
    feature_args=(--features prx)
  fi

  listed=$(cargo test -p "$crate" "${feature_args[@]}" --lib -- --list 2>/dev/null |
    command grep ': test$' |
    sed 's/: test$//' |
    sort -u)

  PRAXIS_CONSTITUTION_TAGS_OUT="$tagsfile" \
    cargo test -p "$crate" "${feature_args[@]}" --lib -- constitution_coverage >/dev/null 2>&1
fi

# Registered: every tag's full path, crate prefix stripped to match the listing.
# The coverage meta-test writes the full tag set to PRAXIS_CONSTITUTION_TAGS_OUT
# atomically — reading a file is deterministic, unlike scraping ~6.7k lines from
# `--nocapture` stdout (which dropped lines and made this gate flaky).
registered=$(sed "s/^${libname}:://" "$tagsfile" | sort -u)

untagged=$(comm -23 <(printf '%s\n' "$listed") <(printf '%s\n' "$registered") | command grep -v '^$' || true)
phantom=$(comm -13 <(printf '%s\n' "$listed") <(printf '%s\n' "$registered") | command grep -v '^$' || true)

n_listed=$(printf '%s\n' "$listed" | command grep -c . || true)
n_registered=$(printf '%s\n' "$registered" | command grep -c . || true)
n_untagged=$(printf '%s\n' "$untagged" | command grep -c . || true)
n_phantom=$(printf '%s\n' "$phantom" | command grep -c . || true)

echo "constitution completeness [$crate]: listed=$n_listed tagged=$n_registered untagged=$n_untagged phantom=$n_phantom"

if [ "$n_untagged" -gt 0 ]; then
  echo "── untagged (no praxis_value tag), first 20:"
  printf '%s\n' "$untagged" | head -20
fi
if [ "$n_phantom" -gt 0 ]; then
  echo "── phantom tags (name has no matching test — typo in register_praxis_value!?), first 20:"
  printf '%s\n' "$phantom" | head -20
fi

if [ "$n_untagged" -eq 0 ] && [ "$n_phantom" -eq 0 ]; then
  echo "COMPLETE: every test declares a guarantee, and every tag maps to a real test."
elif [ "$enforce" = "--enforce" ]; then
  exit 1
fi

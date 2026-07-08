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
#   scripts/constitution-gate.sh [crate] [--enforce]
#
# Without --enforce it reports the gaps (used during backfill). With --enforce
# it exits non-zero if anything is untagged or phantom (the CI gate).
#
# Notes:
#  - proptest! tests are one test function each (a property is one test).
#  - the constitution_coverage meta-test is infrastructure, not a guarantee
#    witness, so it is excluded.
set -euo pipefail

crate="${1:-pr4xis-domains}"
enforce="${2:-}"
libname="${crate//-/_}"

# In `pr4xis-domains`, `prx` gates the serialization machinery (the .prx codecs +
# the ontologies that describe them — ontology_archive, canonical_codec,
# succinct_codec). Without it the gate is MASKED: `cargo test --lib` with default
# features lists neither those tests NOR their tags, so they match trivially and
# the machinery escapes the untagged=0 invariant. `--features prx` is a strict
# superset of the default test set (no default-only tests exist), so enforcing
# under it covers everything. Only `pr4xis-domains` has this feature; other crates
# run with default features so the gate stays runnable for any `$crate`.
feature_args=()
if [ "$crate" = "pr4xis-domains" ]; then
  feature_args=(--features prx)
fi

# Listed: every lib test function (proptests once). No test is exempt — the
# meta-test itself carries a tag, so the self-binding gate has no special case.
listed=$(cargo test -p "$crate" "${feature_args[@]}" --lib -- --list 2>/dev/null |
  grep ': test$' |
  sed 's/: test$//' |
  sort -u)

# Registered: every tag's full path, crate prefix stripped to match --list.
# The coverage meta-test writes the full tag set to PRAXIS_CONSTITUTION_TAGS_OUT
# atomically — reading a file is deterministic, unlike scraping ~6.7k lines from
# `--nocapture` stdout (which dropped lines and made this gate flaky).
tagsfile=$(mktemp)
trap 'rm -f "$tagsfile"' EXIT
PRAXIS_CONSTITUTION_TAGS_OUT="$tagsfile" \
  cargo test -p "$crate" "${feature_args[@]}" --lib -- constitution_coverage >/dev/null 2>&1
registered=$(sed "s/^${libname}:://" "$tagsfile" | sort -u)

untagged=$(comm -23 <(printf '%s\n' "$listed") <(printf '%s\n' "$registered") | grep -v '^$' || true)
phantom=$(comm -13 <(printf '%s\n' "$listed") <(printf '%s\n' "$registered") | grep -v '^$' || true)

n_listed=$(printf '%s\n' "$listed" | grep -c . || true)
n_registered=$(printf '%s\n' "$registered" | grep -c . || true)
n_untagged=$(printf '%s\n' "$untagged" | grep -c . || true)
n_phantom=$(printf '%s\n' "$phantom" | grep -c . || true)

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

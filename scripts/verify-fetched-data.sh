#!/usr/bin/env bash
# Verify that every FETCHED data tree CI depends on is present on disk.
#
# Why this exists: the `build` job skips `pr4xis update` (and the ~126s CLI
# build that exists only to run it) when the praxis-data cache hits. That skip
# is only sound if the restored cache genuinely contains everything the archive
# build reads — and `crates/wasm/build.rs` is FAIL-OPEN: :29 tolerates a missing
# data root and :387 `continue`s past a missing .owl. So an incomplete restore
# would not error, it would silently produce a smaller artifact and let CI go
# green over it.
#
# This script converts that fail-open into a loud failure. It is the precondition
# check that makes skipping the fetch provable rather than hopeful.
#
#   scripts/verify-fetched-data.sh
#
# Exits non-zero, naming every missing path, if the restored tree is incomplete.
#
# The list below is the 15 files `pr4xis update` was measured re-downloading on
# every job of every run (master run 30502233952: 75 `[fetched]` across five
# jobs). Every one is gitignored — none is committed — so their presence can
# only come from a fetch or a cache restore.
set -uo pipefail

# Canonical list. MUST equal the praxis-data cache `path:` entries in ci.yml;
# the self-check at the bottom enforces that, so the two cannot drift.
FETCHED=(
  # WordNet has its OWN cache key (wordnet-english-2025-v1) and its own skip
  # condition in ci.yml, and crates/wasm/build.rs:29 is fail-open on it —
  # `if Path::new(wordnet_path).exists()`, so a missing XML emits an EMPTY
  # English bundle rather than failing. A wasm artifact with no English in it
  # would sail through a green build.
  crates/domains/data/wordnet/english-wordnet-2025.xml
  crates/domains/data/adobe/glyphlist.txt
  crates/domains/data/morphology/agid-infl.txt
  crates/domains/data/markup-schemas/lmf/wn_lmf_dtd-1.3.dtd
  crates/domains/data/markup-schemas/mods/mods_3_8-2018.xsd
  crates/domains/data/markup-schemas/xhtml/xhtml-1.0-strict.xsd
  crates/domains/data/markup-schemas/xml/xml.xsd
  crates/domains/data/markup-schemas/xml/xml-infoset.xhtml
  crates/domains/data/markup-schemas/xml/xml_1_0_fifth_edition-2008.xml
  crates/domains/data/markup-schemas/xsd/xsd_meta_schema-1.1.xsd
)

# Directory trees, checked for non-emptiness rather than by exact filename
# because their contents are version-pinned by praxis.lock and change with it.
FETCHED_TREES=(
  crates/domains/data/legal/uscode
  crates/domains/data/ontologies
  # THE EXTRACTED SUBTREES, not their parents. `xsts/` and `xmlconf/` each
  # hold a COMMITTED `.tar.prx` bundle, so both directories exist in a fresh
  # checkout and `[ -d ]` on them passes whether or not the fetch ever ran —
  # the check would have been satisfied by exactly the state it exists to
  # reject. The extracted trees below are what the markup-schemas tests read
  # and what only a real fetch (or a complete cache restore) produces.
  crates/domains/data/markup-schemas/xsts/xmlschema2006-11-06
  crates/domains/data/markup-schemas/xmlconf/xmlconf
)

fail=0

for f in "${FETCHED[@]}"; do
  if [ ! -s "$f" ]; then
    echo "::error::fetched data missing or empty: $f"
    fail=1
  fi
done

for d in "${FETCHED_TREES[@]}"; do
  if [ ! -d "$d" ]; then
    echo "::error::fetched data tree missing: $d"
    fail=1
  fi
done

# The OWL vocabularies specifically: build.rs:387 `continue`s past each missing
# one, so a partial restore here is the quietest possible failure. praxis.lock
# pins six.
owl_count=$(find crates/domains/data/ontologies -name '*.owl' 2>/dev/null | wc -l)
if [ "$owl_count" -lt 6 ]; then
  echo "::error::expected 6 fetched .owl vocabularies, found ${owl_count} — build.rs silently skips missing ones"
  fail=1
fi

# The USC titles the corpus and the wasm build both read.
xml_count=$(find crates/domains/data/legal/uscode -name '*.xml' 2>/dev/null | wc -l)
if [ "$xml_count" -lt 1 ]; then
  echo "::error::no USC title XML under crates/domains/data/legal/uscode"
  fail=1
fi

# ── Self-check: this list must match ci.yml's cache paths ────────────────────
# Without this, adding a path to the cache block without adding it here would
# leave a file cached but unverified — reintroducing exactly the silent-skip
# hazard this script exists to close.
CI=.github/workflows/ci.yml
if [ -f "$CI" ]; then
  # Every cached data path in ci.yml, from ALL cache blocks — WordNet has its
  # own key (wordnet-english-2025-v1) separate from praxis-data, and both are
  # skip conditions for the fetch, so both must be verified.
  # Two YAML shapes both appear: a block-scalar list entry (praxis-data caches
  # several paths) and an inline `path: <one>` (the WordNet cache has exactly
  # one). Comment lines are excluded so prose mentioning a path is not mistaken
  # for a cached one.
  ci_paths=$(awk '
    { line = $0; sub(/^[[:space:]]+/, "", line) }
    line ~ /^#/ { next }
    sub(/^path:[[:space:]]*/, "", line) || line ~ /^crates\/domains\/data\//  {
      if (line ~ /^crates\/domains\/data\//) print line
    }
  ' "$CI" | sed 's|/\*\.owl$||' | sort -u)
  mine=$(printf '%s\n' "${FETCHED[@]}" "${FETCHED_TREES[@]}" | sort -u)

  # CONTAINMENT, not equality. Verification is deliberately STRICTER than the
  # cache: ci.yml caches `markup-schemas/xsts`, while this script checks
  # `xsts/xmlschema2006-11-06` inside it, because the parent contains a
  # committed .tar.prx and so exists even when the fetch never ran. So the
  # relation to enforce is that every cached path has something verified at or
  # beneath it, and every verified path sits at or beneath something cached.
  while IFS= read -r c; do
    [ -n "$c" ] || continue
    printf '%s\n' "$mine" | command grep -q "^${c}" || {
      echo "::error::ci.yml caches ${c} but nothing here verifies it — a partial"
      echo "  restore of that path would pass unnoticed."
      fail=1
    }
  done <<<"$ci_paths"

  while IFS= read -r m; do
    [ -n "$m" ] || continue
    covered=0
    while IFS= read -r c; do
      [ -n "$c" ] || continue
      case "$m" in "$c"*) covered=1 ;; esac
    done <<<"$ci_paths"
    [ "$covered" -eq 1 ] || {
      echo "::error::${m} is verified here but is not in any ci.yml cache path,"
      echo "  so skipping the fetch would leave it absent."
      fail=1
    }
  done <<<"$mine"
fi

if [ "$fail" -ne 0 ]; then
  echo "::error::the restored data cache is incomplete — it is NOT safe to skip pr4xis update"
  exit 1
fi

echo "fetched data complete: ${#FETCHED[@]} files, ${#FETCHED_TREES[@]} trees, ${owl_count} .owl, ${xml_count} USC title XML"

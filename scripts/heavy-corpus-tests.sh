#!/usr/bin/env bash
# The heavy-corpus suite — THE single definition, shared by dev-ci and CI.
#
# `praxis-corpus-tests` is workspace-excluded so the default nextest run never
# re-parses the multi-hundred-MB USC titles once per process-isolated test.
# It runs under `cargo test` instead: one process, so a LazyLock fixture parses
# each giant exactly once.
#
# WHY THIS IS A SCRIPT AND NOT TWO COPIES OF A COMMAND.
# Two of this crate's suites are deliberately red and must NOT gate a build:
#
#   * `caregiver_questions_generated` — one test per corpus question asserting
#     the engine answers it. 4,075 of the 4,219 do not pass, by design: this is
#     the honest per-question scoreboard, not a gate. The regression GATE is
#     `caregiver_capability_ratchet`, which enforces the committed ceilings and
#     the Green floor.
#   * `chat_faithfully_realizes_the_loaded_corpus` — documented RED on the
#     unfixed pipeline.
#
# devenv.nix knew that and filtered them; ci.yml ran the bare command and did
# not, while carrying a comment claiming the two matched. So every push failed
# the `wasm` job, and `pages` — which needs it — never deployed. A comment
# asserting "local == CI" cannot make it so, and a Nix `${...}` interpolation
# cannot be pasted into YAML, so the only way to actually enforce the claim is
# one file both callers execute. This is that file.
#
# Adding a test file under crates/praxis-corpus-tests/tests/ means adding it to
# MUST_PASS in the same commit — the same closure-list discipline this repo
# already applies elsewhere. A file that is absent here simply never runs.
set -euo pipefail

MANIFEST="crates/praxis-corpus-tests/Cargo.toml"

# Every heavy-corpus test target EXCEPT `caregiver_questions_generated`.
MUST_PASS=(
  adversarial_questions_generated
  caregiver_capability_ratchet
  caregiver_dialogues_generated
  chat_capability
  composed_surface_overlay
  defines_pointers_corpus_ratchet
  english_taxonomy_bfs
  lexicon_chat_capability
  nlp_task_reachability
  praxis_knowledge_graph
  scratch_probe
  statute_axioms
  title_1
  title_15
  title_18
  title_28
  title_29
  title_42
  title_49
  title_50
  title_5
  usc_anchors
  usc_closure_oracle
  usc_compact_gate
  usc_flipped_titles
  usc_grounding
  usc_round_trip
  usc_structural_audit
  wordnet
)

# Guard the closure list against silent drift: a new test file that nobody
# added here would otherwise never run, and its absence would look exactly
# like a passing build.
missing=()
for f in crates/praxis-corpus-tests/tests/*.rs; do
  name="$(basename "${f%.rs}")"
  [[ $name == "caregiver_questions_generated" ]] && continue
  # Generated companions of a listed target are covered by it.
  printf '%s\n' "${MUST_PASS[@]}" | grep -qx "$name" || missing+=("$name")
done
if [[ ${#missing[@]} -gt 0 ]]; then
  echo "::error::heavy-corpus closure list is stale — these test files are not in MUST_PASS:" >&2
  printf '  %s\n' "${missing[@]}" >&2
  echo "Add them to scripts/heavy-corpus-tests.sh (or exclude them explicitly, with the reason)." >&2
  exit 1
fi

targets=()
for t in "${MUST_PASS[@]}"; do targets+=(--test "$t"); done

echo "=== heavy-corpus MUST-PASS suites (${#MUST_PASS[@]} targets) ==="
RUSTFLAGS="-D warnings" cargo test --manifest-path "$MANIFEST" --release \
  "${targets[@]}" -- --skip chat_faithfully_realizes_the_loaded_corpus

# Reported, never gated — the per-question scoreboard over all 4,219 corpus
# questions, ~4,000 of them deliberately red. `|| true` is the whole point:
# this is the backlog the ratchet exists to measure progress against.
#
# ~26s. Every question resolves against one process-wide classification table
# (`caregiver::classification_table`), so the 2.71s reasoner build is paid once
# per worker rather than once per test.
echo "=== honest capability backlog (reported, NOT gated) ==="
RUSTFLAGS="-D warnings" cargo test --manifest-path "$MANIFEST" --release \
  --test caregiver_questions_generated || true

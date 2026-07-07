use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use pr4xis::ontology::meta::OntologyName;
use pr4xis_chat as chat;

use pr4xis_domains::applied::data_provisioning::fetch::{self, FetchOptions, FetchOutcome};
use pr4xis_domains::applied::data_provisioning::registry::{by_name, data_sources};
use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
use pr4xis_domains::cognitive::linguistics::english::{English, english_load_owned};
use pr4xis_domains::cognitive::linguistics::language::Language;
use pr4xis_domains::cognitive::linguistics::pragmatics::speech_act::SpeechAct;
use pr4xis_domains::formal::information::dialogue::engine::{self, DialogueAction};
use pr4xis_domains::social::software::markup::xml::lmf;
use pr4xis_domains::social::software::markup::xml::owl::prx::EmittedArtifact;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::usc_runtime_ontology;

/// pr4xis — axiomatic intelligence via ontology.
#[derive(Parser, Debug)]
#[command(name = "pr4xis", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Start an interactive chat session (default when no subcommand is given).
    Chat {
        /// Load an additional ontology into the chat reasoner, so a NON-legal
        /// (or any) loaded ontology is chattable interactively. Repeatable.
        ///
        /// Accepts, in resolution order:
        ///   - a built-in demo ontology name — `dependability` (Avizienis et al.
        ///     2004 fault taxonomy) or `legal-sources` — compiled in-process;
        ///   - a registered OWL vocabulary name (e.g. `cito`, `doco`),
        ///     materialised from its committed compact `.prx.gz` (run
        ///     `pr4xis compile --compact` first if absent);
        ///   - a path to a `.owl` file, parsed and grounded by its `rdfs:label`s.
        ///
        /// The LegalSources base is always loaded (parity with the wasm runtime),
        /// so `is a statute a law` answers with no explicit `--load`.
        #[arg(long = "load", value_name = "NAME-OR-PATH")]
        load: Vec<String>,
    },
    /// Fetch and verify external data dependencies declared by the
    /// `applied/data_provisioning/` ontology.
    Update {
        /// Name of a specific dataset. Omit to operate on every registered entry.
        name: Option<String>,
        /// Verify current local state against declared identities without fetching.
        #[arg(long, conflicts_with_all = ["force", "lock"])]
        check: bool,
        /// Re-fetch even when a valid local copy already exists.
        #[arg(long)]
        force: bool,
        /// List every registered dataset with its current state.
        #[arg(long)]
        list: bool,
        /// Refuse to touch the network; useful for air-gapped builds.
        /// Combined with `--lock`, switches the lock rewrite to the
        /// CUSTODY RE-PIN mode (see `--lock`).
        #[arg(long)]
        offline: bool,
        /// Regenerate `praxis.lock` pins, preserving comments and key
        /// ordering. Two modes:
        ///
        /// Alone: downloads every (or one) registered entry, bypasses
        /// identity verification, writes bytes to disk, computes the
        /// content address, and rewrites the corresponding `[hashes]`
        /// line — the operator is regenerating the pin from authoritative
        /// bytes.
        ///
        /// With `--offline`: the CUSTODY RE-PIN — no network. Every
        /// on-disk source must FIRST verify against its EXISTING pin
        /// (under whatever algorithm the pin names); only then are the
        /// `[hashes]`, `[byte_exact_signatures]`, and
        /// `[canonical_signatures]` entries rewritten under the emit
        /// algorithm. ANY custody mismatch aborts the entire run before
        /// a single byte of `praxis.lock` is written.
        ///
        /// Mutually exclusive with `--check` (read-only).
        #[arg(long)]
        lock: bool,
    },
    /// Compile registered sources into verifiable `.prx` archives.
    ///
    /// Emits one content-addressed `.prx.gz` per registered OWL vocabulary,
    /// U.S. Code title (envelope + portable COMPACT), and WordNet language into
    /// the build cache (`.prx-cache/`), the parse-once artifacts the runtime
    /// loaders read instead of re-parsing the source per process. Requires the
    /// sources on disk — run `pr4xis update` first (or pass `--update`).
    ///
    /// By default this is a VERIFY pass (CI-safe, writes nothing): each emitted
    /// archive's content address must match its committed `praxis.lock` pin, and
    /// any drift fails closed. `--lock` switches to the maintainer WRITE mode
    /// that records the pins (run locally after a deliberate change; never in CI).
    Compile {
        /// WRITE each emitted archive's content address into `praxis.lock`
        /// (`[archive_signatures]` for the rkyv envelopes, the portable
        /// `[compact_archive_signatures]` for the compact archives). The
        /// maintainer re-pin mode — like `update --lock`, run locally, not in CI.
        /// Without it, `compile` VERIFIES against the committed pins instead.
        #[arg(long)]
        lock: bool,
        /// Provision any missing sources by running `update` first, instead of
        /// erroring. Convenience for a fresh checkout; CI provisions separately.
        #[arg(long)]
        update: bool,
        /// Emit (and verify/pin) ONLY the portable compact U.S. Code archives —
        /// the runtime fast-load cache `loaded()` reads. Skips the heavy rkyv
        /// envelopes (the `decompile`/distribution artifacts) and WordNet. The
        /// CI mode: it re-derives + verifies the committed `[compact_archive_signatures]`
        /// pins for ALL titles (including the giants the unit tests cap out of)
        /// in ~seconds, without re-emitting the toolchain-coupled envelopes.
        #[arg(long)]
        compact: bool,
    },
    /// Decompile a compiled `.prx` archive back to its exact source bytes —
    /// the inverse of `compile`, the `.prx → source` leg of the universal
    /// compiler.
    ///
    /// Resolves the registered source by name, loads the `.prx.gz` that
    /// `compile` wrote into `.prx-cache/`, regenerates the source bytes through
    /// the uniform decompile op (routing to the OWL / USC / WordNet reconstruct
    /// leaf), writes them to `--out` (or a default path), and prints the
    /// achieved round-trip fidelity tier. Every registered `.prx`-consumer
    /// source reconstructs at the `ByteExactGraphFaithful` tier — the bytes
    /// regenerate from the typed ontology graph plus its recorded
    /// concrete-syntax complement, gated by the content-address honesty check.
    Decompile {
        /// The registered source name to decompile (e.g. `cito`,
        /// `usc_title_18`, `english_wordnet`). Run `pr4xis update --list` to
        /// see registered names. Not required with `--meter`.
        #[arg(required_unless_present = "meter")]
        name: Option<String>,
        /// Where to write the reconstructed source bytes. Defaults to
        /// `<name>-<version>.<ext>` in the current directory.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Print the per-source completeness meter (every registered source's
        /// achieved tier + the named gap to graph-faithfulness) and exit,
        /// instead of decompiling. A non-failing report.
        #[arg(long, conflicts_with_all = ["out"])]
        meter: bool,
        /// After the decompile (which verifies byte-exactness against the
        /// source identity), additionally print a custody ATTESTATION for
        /// compliance consumers: the SHA-256 (NIST FIPS 180-4) of the
        /// decompiled bytes, the source identity, the blake3 archive
        /// address it was decompiled from, and an RFC 3339 UTC timestamp.
        /// Identity is intrinsic (BLAKE3); FIPS is the boundary speech-act.
        #[arg(long, conflicts_with = "meter")]
        fips: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Chat { load: Vec::new() }) {
        Command::Chat { load } => run_chat(&load),
        Command::Update {
            name,
            check,
            force,
            list,
            offline,
            lock,
        } => {
            if let Err(e) = run_update(name.as_deref(), check, force, list, offline, lock) {
                eprintln!("pr4xis update: {e}");
                std::process::exit(1);
            }
        }
        Command::Compile {
            lock,
            update,
            compact,
        } => {
            if let Err(e) = run_compile(lock, update, compact) {
                eprintln!("pr4xis compile: {e}");
                std::process::exit(1);
            }
        }
        Command::Decompile {
            name,
            out,
            meter,
            fips,
        } => {
            if let Err(e) = run_decompile(name.as_deref(), out.as_deref(), meter, fips) {
                eprintln!("pr4xis decompile: {e}");
                std::process::exit(1);
            }
        }
    }
}

// --------------------------------------------------------------------------
// `pr4xis update` — the data-provisioning CLI surface
// --------------------------------------------------------------------------

fn run_update(
    name: Option<&str>,
    check: bool,
    force: bool,
    list: bool,
    offline: bool,
    lock: bool,
) -> anyhow::Result<()> {
    if list {
        print_list();
        return Ok(());
    }

    let workspace_root = workspace_root()?;
    let opts = FetchOptions {
        check,
        force,
        offline,
        lock,
    };

    let outcomes = match name {
        Some(n) => {
            let entry = by_name(n).ok_or_else(|| anyhow::anyhow!("unknown dataset: {n}"))?;
            vec![fetch::fetch_entry(entry, opts, &workspace_root)]
        }
        None => fetch::fetch_all(opts, &workspace_root),
    };

    let mut any_failed = false;
    for outcome in &outcomes {
        print_outcome(outcome);
        if !outcome.is_ok() {
            any_failed = true;
        }
    }

    // Fail-closed ordering: a failed outcome (in the custody re-pin mode, a
    // VerificationFailed = custody break) aborts BEFORE any lock write — a
    // broken pin must surface, never be papered over by a fresh one.
    if any_failed {
        anyhow::bail!("one or more datasets failed to update; praxis.lock untouched");
    }

    if lock {
        apply_lock_outcomes(&outcomes, &workspace_root)?;
    }
    Ok(())
}

/// Walk the outcomes from a `--lock` run and rewrite `praxis.lock` so
/// every `Locked { address_hex, .. }` becomes the new pin for that source's
/// `"name@version"` key, in every lock space derived from the source bytes:
///
/// - `[hashes]` — always (the raw-bytes pin);
/// - `[byte_exact_signatures]` — when the key is already byte-exact-pinned:
///   the byte-exact law (`put(get(b)) == b`) makes its value EQUAL the raw
///   pin, so it moves in lockstep (the parser rejects divergence);
/// - `[canonical_signatures]` — when the key is already canonical-pinned:
///   the canonical form is re-derived from the SAME on-disk bytes, the OLD
///   pin is custody-verified against it under the pin's own algorithm
///   (fail-closed: a mismatch aborts before writing), and the fresh
///   emit-algorithm address of the canonical bytes is written.
///
/// The lockfile is read once, mutated in-memory across all outcomes, then
/// written once to keep the operation atomic-ish for the common case.
fn apply_lock_outcomes(outcomes: &[FetchOutcome], workspace_root: &Path) -> anyhow::Result<()> {
    use pr4xis_domains::applied::data_provisioning::lockfile::{
        set_byte_exact_signature, set_canonical_signature, set_hash,
    };
    use pr4xis_domains::applied::data_provisioning::registry::{
        lock_byte_exact_signatures, lock_canonical_signature,
    };
    let lock_path = workspace_root.join("praxis.lock");
    let original = std::fs::read_to_string(&lock_path)
        .map_err(|e| anyhow::anyhow!("read {}: {e}", lock_path.display()))?;
    let mut text = original.clone();
    for outcome in outcomes {
        if let FetchOutcome::Locked {
            name,
            path,
            address_hex,
            ..
        } = outcome
        {
            // Resolve `name → version` by looking up the registry entry.
            // (The outcome carries `name` only; the version lives in
            // praxis.toml via the parsed `RegistryEntry`.)
            let Some(entry) = by_name(name) else {
                eprintln!("  [warn]    {name}: not found in registry, skipping lock write");
                continue;
            };
            let key = format!("{}@{}", entry.name, entry.version);
            text = set_hash(&text, &key, address_hex)
                .map_err(|e| anyhow::anyhow!("praxis.lock rewrite for {key}: {e}"))?;
            // Byte-exact pin: equal to the raw pin by the byte-exact law.
            if lock_byte_exact_signatures().contains_key(&key) {
                text = set_byte_exact_signature(&text, &key, address_hex).map_err(|e| {
                    anyhow::anyhow!("praxis.lock [byte_exact_signatures] rewrite for {key}: {e}")
                })?;
            }
            // Canonical pin: re-derive the canonical form from the same
            // bytes, custody-verify the OLD pin, then re-pin.
            if let Some(old_pin) = lock_canonical_signature(&entry.name, &entry.version) {
                let bytes = std::fs::read(path)
                    .map_err(|e| anyhow::anyhow!("read {}: {e}", path.display()))?;
                let canonical = canonical_form_of(entry, &bytes)?;
                if !old_pin.verifies(&canonical) {
                    anyhow::bail!(
                        "CUSTODY BREAK: {key}'s re-derived canonical form does not verify \
                         against the existing [canonical_signatures] pin {old_pin} — refusing \
                         to re-pin; praxis.lock untouched"
                    );
                }
                let canonical_address =
                    pr4xis_runtime::address::ContentAddress::of(&canonical).to_hex();
                text = set_canonical_signature(&text, &key, &canonical_address).map_err(|e| {
                    anyhow::anyhow!("praxis.lock [canonical_signatures] rewrite for {key}: {e}")
                })?;
            }
        }
    }
    if text != original {
        std::fs::write(&lock_path, text)
            .map_err(|e| anyhow::anyhow!("write {}: {e}", lock_path.display()))?;
        println!("praxis.lock updated.");
    } else {
        println!("praxis.lock unchanged.");
    }
    Ok(())
}

/// The canonical-form bytes of `entry`'s source — the SAME derivation the
/// `.prx` load gate's graph-identity leg uses. Today exactly the OWL
/// vocabularies carry `[canonical_signatures]` pins, and their canonical
/// form is the W3C RDFC-1.0 canonical N-Quads of the source RDF graph
/// (`OwlLens::canonical`). A canonical-pinned source of any other content
/// type has no wired derivation here — fail closed naming the key, never
/// guess.
fn canonical_form_of(
    entry: &pr4xis_domains::applied::data_provisioning::ontology::RegistryEntry,
    bytes: &[u8],
) -> anyhow::Result<Vec<u8>> {
    use pr4xis_domains::formal::meta::well_behaved_lens::{DecompileKind, WellBehavedLens};
    use pr4xis_domains::social::software::markup::xml::owl::lens::OwlLens;
    match DecompileKind::from_content_type(entry.content_type()) {
        Some(DecompileKind::Owl) => OwlLens::canonical(bytes)
            .map_err(|e| anyhow::anyhow!("{}: canonical form underivable: {e}", entry.name)),
        other => anyhow::bail!(
            "{}@{} carries a [canonical_signatures] pin but no canonical derivation is wired \
             for its kind ({other:?}) — refusing to re-pin",
            entry.name,
            entry.version,
        ),
    }
}

// --------------------------------------------------------------------------
// `pr4xis compile` — emit verifiable `.prx` archives (the parse-once cache)
// --------------------------------------------------------------------------

fn run_compile(lock: bool, update: bool, compact: bool) -> anyhow::Result<()> {
    use pr4xis_domains::social::software::markup::xml::lmf::prx::{
        emit_all_compact_english_prx_gz, emit_all_wordnet_prx_gz, english_compact_prx_cache_dir,
    };
    use pr4xis_domains::social::software::markup::xml::owl::prx::{
        emit_all_compact_owl_prx_gz, emit_all_prx_gz as emit_all_owl_prx_gz,
        owl_compact_prx_cache_dir, owl_prx_cache_dir,
    };
    use pr4xis_domains::social::software::markup::xml::uslm::corpus::prx::{
        emit_all_compact_usc_prx_gz, emit_all_usc_prx_gz, usc_compact_prx_cache_dir,
        usc_prx_cache_dir,
    };
    let workspace_root = workspace_root()?;

    // Precondition: `compile` consumes the physical sources `pr4xis update`
    // provisions. A registered, pinned, compilable source that is not on disk is
    // the "forgot to run update" failure — alert (or auto-provision with
    // `--update`) instead of silently emitting nothing for it. Every leg
    // (compact + rkyv) needs its source on disk, so the same set is required
    // regardless of `--compact`.
    let missing = missing_compilable_sources(&workspace_root);
    if !missing.is_empty() {
        if update {
            println!(
                "provisioning {} missing source(s) via update…",
                missing.len()
            );
            run_update(None, false, false, false, false, false)?;
            let still = missing_compilable_sources(&workspace_root);
            if !still.is_empty() {
                anyhow::bail!("still missing after update: {}", still.join(", "));
            }
        } else {
            anyhow::bail!(
                "{} registered source(s) not provisioned: {} — run `pr4xis update` first \
                 (or `pr4xis compile --update`)",
                missing.len(),
                missing.join(", "),
            );
        }
    }

    let mut artifacts: Vec<EmittedArtifact> = Vec::new();
    // Compact USC archives pin into `[compact_archive_signatures]`, a different
    // lock space than the rkyv envelopes' `[archive_signatures]`, so they are
    // collected separately.
    let mut compact_artifacts: Vec<EmittedArtifact> = Vec::new();

    // The portable compact U.S. Code cache → `.prx-cache/usc-compact/` — the
    // corpus loader's FAST content-address-gated path. Always emitted (it is the
    // runtime fast-load artifact); under `--compact` it is the ONLY thing emitted.
    let usc_compact_dir = usc_compact_prx_cache_dir(&workspace_root);
    compact_artifacts.extend(
        emit_all_compact_usc_prx_gz(&usc_compact_dir)
            .map_err(|e| anyhow::anyhow!("emit USC compact: {e}"))?,
    );

    // The portable compact English cache → `.prx-cache/wordnet-compact/` — the
    // `english_loaded()` FAST content-address-gated path. Always emitted (the
    // runtime fast-load artifact); graceful-skip if the WordNet source is absent.
    let english_compact_dir = english_compact_prx_cache_dir(&workspace_root);
    compact_artifacts.extend(
        emit_all_compact_english_prx_gz(&english_compact_dir)
            .map_err(|e| anyhow::anyhow!("emit English compact: {e}"))?,
    );

    // The portable compact OWL cache → `.prx-cache/ontologies-compact/` — the
    // OWL-vocab loader's (`olia::reference_model` + `loaded_vocabularies`) FAST
    // content-address-gated path. The committed `data/ontologies/<name>.prx.gz`
    // is a copy of this output. Always emitted (the runtime fast-load artifact);
    // graceful-skip per source whose `.owl` is absent.
    let owl_compact_dir = owl_compact_prx_cache_dir(&workspace_root);
    compact_artifacts.extend(
        emit_all_compact_owl_prx_gz(&owl_compact_dir)
            .map_err(|e| anyhow::anyhow!("emit OWL compact: {e}"))?,
    );

    // The committed RAW-SOURCE `.prx` archives — every registered byte-stream
    // source (XSD / DTD / XHTML / XML-spec / OOXML zip / TSV / glyph list). These
    // are the `data/**/<stem>.prx` committed twins the generalized raw-source
    // loader (`raw_source_prx::load_raw_source`) reads; written beside their raw
    // source (NOT under `.prx-cache/`) since they ARE the committed artifact, and
    // pinned into the same `[compact_archive_signatures]` space.
    compact_artifacts.extend(
        pr4xis_domains::applied::data_provisioning::raw_source_prx::emit_all_compact_raw_source_prx()
            .map_err(|e| anyhow::anyhow!("emit raw-source .prx: {e}"))?
            .into_iter()
            .map(|r| EmittedArtifact {
                name: r.name,
                version: r.version,
                path: r.path,
                byte_len: r.byte_len,
                archive_address: r.archive_address,
            }),
    );

    if !compact {
        // The rkyv envelopes (the `decompile`/distribution artifacts) + WordNet.
        // Heavy and, for USC/WordNet, toolchain-coupled; skipped by `--compact`.
        // OWL → `.prx-cache/ontologies/` (the `pr4xis decompile` source; bundled
        // `.owl` makes these compile on a plain checkout).
        let owl_dir = owl_prx_cache_dir(&workspace_root);
        artifacts
            .extend(emit_all_owl_prx_gz(&owl_dir).map_err(|e| anyhow::anyhow!("emit OWL: {e}"))?);
        // USC rkyv envelopes → `.prx-cache/usc/` (the decompile source).
        let usc_dir = usc_prx_cache_dir(&workspace_root);
        artifacts
            .extend(emit_all_usc_prx_gz(&usc_dir).map_err(|e| anyhow::anyhow!("emit USC: {e}"))?);
        // WordNet/English → `.prx-cache/wordnet/`.
        let wn_dir = workspace_root.join(".prx-cache").join("wordnet");
        artifacts.extend(
            emit_all_wordnet_prx_gz(&wn_dir).map_err(|e| anyhow::anyhow!("emit WordNet: {e}"))?,
        );
    }

    let mut total_bytes: u64 = 0;
    for a in artifacts.iter().chain(&compact_artifacts) {
        total_bytes += a.byte_len;
        println!(
            "  compiled  {}@{}  {} bytes  {}",
            a.name, a.version, a.byte_len, a.archive_address
        );
    }
    println!(
        "{} archive(s) ({} compact), {total_bytes} bytes total → {}",
        artifacts.len() + compact_artifacts.len(),
        compact_artifacts.len(),
        workspace_root.join(".prx-cache").display()
    );

    if artifacts.is_empty() && compact_artifacts.is_empty() {
        eprintln!("  [warn]    no registered source on disk — run `pr4xis update` first");
    }
    if lock {
        // Maintainer WRITE mode: record the pins (local; never CI).
        apply_archive_signature_lock(&artifacts, &workspace_root)?;
        apply_compact_archive_signature_lock(&compact_artifacts, &workspace_root)?;
    } else {
        // Default VERIFY mode (CI-safe): each emitted archive's content address
        // must match its committed pin, else drift fails closed. An unpinned
        // source is reported (it just gets no fast path), never a failure.
        verify_archives_against_lock(&artifacts, &compact_artifacts)?;
    }
    Ok(())
}

/// Registered, pinned, COMPILABLE sources whose source file is not on disk —
/// `"name@version"` keys. "Compilable" = the kinds `compile` emits
/// (`OntologyVocabulary` OWL, `UsCodeTitle` USC, `Language` WordNet); other
/// registered kinds (conformance suites, …) are not flagged. "Pinned" = present
/// in `[hashes]` (a source with no source-pin is not yet provisioned-expected).
fn missing_compilable_sources(workspace_root: &Path) -> Vec<String> {
    use pr4xis_domains::applied::data_provisioning::registry::{data_sources, lock_hashes};
    use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept::{
        Language, OntologyVocabulary, UsCodeTitle,
    };
    let hashes = lock_hashes();
    let mut missing = Vec::new();
    for e in data_sources() {
        // Every leg of `compile` (rkyv envelopes AND the always-on compact
        // archives) needs its source on disk. `--compact` only skips the heavy
        // rkyv-envelope leg — it still emits the compact OWL / USC / WordNet
        // archives — so all three kinds are required regardless of `--compact`.
        let required = matches!(e.kind, OntologyVocabulary | UsCodeTitle | Language);
        if !required {
            continue;
        }
        let key = format!("{}@{}", e.name, e.version);
        if !hashes.contains_key(&key) {
            continue; // not source-pinned — not an expected-present source
        }
        if !workspace_root.join(e.local_path()).exists() {
            missing.push(key);
        }
    }
    missing.sort();
    missing
}

/// Verify each emitted archive's content address equals its committed
/// `praxis.lock` pin (drift = fail-closed); an unpinned archive is reported, not
/// failed. The read-only default `compile` runs in CI to catch a source/codec
/// change that no longer matches the pins, without mutating the lock.
fn verify_archives_against_lock(
    envelopes: &[EmittedArtifact],
    compact: &[EmittedArtifact],
) -> anyhow::Result<()> {
    use pr4xis_domains::applied::data_provisioning::registry::{
        LockDigest, lock_archive_signature, lock_compact_archive_signature,
    };
    let mut drift: Vec<String> = Vec::new();
    let mut unpinned = 0usize;
    // Typed comparison: the freshly emitted address (the emit leg) against
    // the pinned LockDigest — same algorithm, same digest.
    let mut check = |a: &EmittedArtifact, pin: Option<&LockDigest>, space: &str| match pin {
        Some(p) if *p == LockDigest::address(a.archive_address.clone()) => {}
        Some(p) => drift.push(format!(
            "{}@{} {space}: emitted {} ≠ pinned {}",
            a.name, a.version, a.archive_address, p
        )),
        None => unpinned += 1,
    };
    for a in envelopes {
        check(
            a,
            lock_archive_signature(&a.name, &a.version),
            "[archive_signatures]",
        );
    }
    for a in compact {
        check(
            a,
            lock_compact_archive_signature(&a.name, &a.version),
            "[compact_archive_signatures]",
        );
    }
    if !drift.is_empty() {
        anyhow::bail!(
            "praxis.lock pin drift ({} archive(s)) — re-run `pr4xis compile --lock` after \
             confirming the change is intended:\n  {}",
            drift.len(),
            drift.join("\n  "),
        );
    }
    let total = envelopes.len() + compact.len();
    println!(
        "verified {} archive(s) against praxis.lock pins ({unpinned} unpinned, no fast path).",
        total - unpinned,
    );
    Ok(())
}

/// Write each compiled archive's `MerkleRoot` into the
/// `[archive_signatures]` section of `praxis.lock`, preserving comments and
/// key ordering. The fail-closed `.prx.gz` load gate validates a loaded
/// archive's re-derived `MerkleRoot` against this pin. Mirrors
/// [`apply_lock_outcomes`]' `[hashes]` write.
fn apply_archive_signature_lock(
    artifacts: &[EmittedArtifact],
    workspace_root: &Path,
) -> anyhow::Result<()> {
    if artifacts.is_empty() {
        return Ok(());
    }
    let lock_path = workspace_root.join("praxis.lock");
    let original = std::fs::read_to_string(&lock_path)
        .map_err(|e| anyhow::anyhow!("read {}: {e}", lock_path.display()))?;
    let mut text = original.clone();
    for a in artifacts {
        let key = format!("{}@{}", a.name, a.version);
        text = pr4xis_domains::applied::data_provisioning::lockfile::set_archive_signature(
            &text,
            &key,
            &a.archive_address,
        )
        .map_err(|e| anyhow::anyhow!("praxis.lock [archive_signatures] rewrite for {key}: {e}"))?;
    }
    if text != original {
        std::fs::write(&lock_path, text)
            .map_err(|e| anyhow::anyhow!("write {}: {e}", lock_path.display()))?;
        println!("praxis.lock [archive_signatures] updated.");
    } else {
        println!("praxis.lock [archive_signatures] unchanged.");
    }
    Ok(())
}

/// Write each compiled COMPACT archive's content address into the
/// `[compact_archive_signatures]` section of `praxis.lock`. The write-side
/// companion to the compact runtime gate (`load_compact_usc_prx_gz_gated`);
/// unlike `[archive_signatures]` these addresses are portable across toolchains
/// (the compact codec is dependency-free bit-packing). Mirrors
/// [`apply_archive_signature_lock`].
fn apply_compact_archive_signature_lock(
    artifacts: &[EmittedArtifact],
    workspace_root: &Path,
) -> anyhow::Result<()> {
    if artifacts.is_empty() {
        return Ok(());
    }
    let lock_path = workspace_root.join("praxis.lock");
    let original = std::fs::read_to_string(&lock_path)
        .map_err(|e| anyhow::anyhow!("read {}: {e}", lock_path.display()))?;
    let mut text = original.clone();
    for a in artifacts {
        let key = format!("{}@{}", a.name, a.version);
        text = pr4xis_domains::applied::data_provisioning::lockfile::set_compact_archive_signature(
            &text,
            &key,
            &a.archive_address,
        )
        .map_err(|e| {
            anyhow::anyhow!("praxis.lock [compact_archive_signatures] rewrite for {key}: {e}")
        })?;
    }
    if text != original {
        std::fs::write(&lock_path, text)
            .map_err(|e| anyhow::anyhow!("write {}: {e}", lock_path.display()))?;
        println!("praxis.lock [compact_archive_signatures] updated.");
    } else {
        println!("praxis.lock [compact_archive_signatures] unchanged.");
    }
    Ok(())
}

// --------------------------------------------------------------------------
// `pr4xis decompile` — reconstruct source bytes from a compiled `.prx`
// --------------------------------------------------------------------------

fn run_decompile(
    name: Option<&str>,
    out: Option<&Path>,
    meter: bool,
    fips: bool,
) -> anyhow::Result<()> {
    use pr4xis_domains::formal::meta::well_behaved_lens::{
        DecompileKind, decompile, print_completeness_meter,
    };

    // `--meter` is a whole-system report, not tied to one source.
    if meter {
        print_completeness_meter();
        return Ok(());
    }

    // Required unless `--meter` (enforced by clap `required_unless_present`).
    let name = name.ok_or_else(|| anyhow::anyhow!("a source name is required (or use --meter)"))?;

    let workspace_root = workspace_root()?;

    // Resolve the registered source → its decompile leaf (OWL / USC / WordNet),
    // derived from the registry's ContentType (the single ContentType→kind map).
    let entry = by_name(name).ok_or_else(|| anyhow::anyhow!("unknown source: {name}"))?;
    let kind = DecompileKind::from_content_type(entry.content_type()).ok_or_else(|| {
        anyhow::anyhow!(
            "source `{name}` (content type {:?}) has no `.prx` consumer — only OWL, USC, and \
             WordNet sources can be decompiled today",
            entry.content_type()
        )
    })?;

    // Locate the `.prx.gz` exactly where `pr4xis compile` writes it.
    let prx_path = prx_cache_path(&workspace_root, kind, &entry.name, &entry.version);
    let prx_gz = std::fs::read(&prx_path).map_err(|e| {
        anyhow::anyhow!(
            "no compiled archive at {} ({e}) — run `pr4xis compile` first",
            prx_path.display()
        )
    })?;

    // The uniform decompile op: regenerate the source bytes AND the achieved
    // round-trip fidelity tier.
    let (source_bytes, fidelity) =
        decompile(&prx_gz, kind).map_err(|e| anyhow::anyhow!("decompile {name}: {e}"))?;

    // Write the reconstructed source to --out (or a sensible default).
    let out_path = match out {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from(format!(
            "{}-{}.{}",
            entry.name,
            entry.version,
            source_extension(kind)
        )),
    };
    std::fs::write(&out_path, &source_bytes)
        .map_err(|e| anyhow::anyhow!("write {}: {e}", out_path.display()))?;

    println!(
        "decompiled {}@{} → {} ({} bytes)",
        entry.name,
        entry.version,
        out_path.display(),
        source_bytes.len()
    );
    // Print the achieved fidelity tier honestly — floor vs graph-faithful.
    println!("  round-trip fidelity: {}", fidelity_label(fidelity));

    // `--fips`: the boundary attestation for compliance consumers. The
    // decompile above already discharged the byte-exactness honesty gate
    // against the source identity; this SPEAKS that custody in FIPS terms —
    // identity stays intrinsic (the BLAKE3 content addresses), the SHA-256
    // is the speech-act at the compliance boundary, computed through the
    // same multi-algorithm verify leg (`hash_hex`) every claim uses.
    if fips {
        use pr4xis_domains::social::software::markup::xml::owl::prx::prx_archive_address;
        use pr4xis_runtime::address::{HashAlgorithm, hash_hex};
        let sha256_hex = hash_hex(HashAlgorithm::Sha256, &source_bytes);
        let archive_address = prx_archive_address(&prx_gz)
            .map_err(|e| anyhow::anyhow!("archive address of {}: {e}", prx_path.display()))?;
        println!("  custody attestation (FIPS 180-4 boundary):");
        println!(
            "    source identity:        {}@{}",
            entry.name, entry.version
        );
        println!("    decompiled-bytes SHA-256 (NIST FIPS 180-4): {sha256_hex}");
        println!("    decompiled from archive (blake3 content address): {archive_address}");
        println!("    attested at (RFC 3339, UTC): {}", rfc3339_utc_now());
    }
    Ok(())
}

/// The current time as an RFC 3339 UTC timestamp (`YYYY-MM-DDThh:mm:ssZ`),
/// from `SystemTime` through [`rfc3339_utc`].
fn rfc3339_utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_secs() as i64;
    rfc3339_utc(secs)
}

/// Seconds since the Unix epoch → RFC 3339 UTC timestamp
/// (`YYYY-MM-DDThh:mm:ssZ`), via the proleptic-Gregorian civil-from-days
/// conversion (Hinnant's algorithm; RFC 3339 §5.6 grammar).
fn rfc3339_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let secs_of_day = secs.rem_euclid(86_400);
    let (h, m, s) = (
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60,
    );
    // Civil-from-days (Howard Hinnant, "chrono-Compatible Low-Level Date
    // Algorithms"): days since 1970-01-01 → (year, month, day).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097); // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let mo = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// The `.prx.gz` path `pr4xis compile` writes for a source of each
/// [`DecompileKind`], under `<workspace_root>/.prx-cache/`. The CLI-side mirror
/// of the emitters' `out_dir` choices, so decompile reads exactly what compile
/// wrote.
fn prx_cache_path(
    workspace_root: &Path,
    kind: pr4xis_domains::formal::meta::well_behaved_lens::DecompileKind,
    name: &str,
    version: &str,
) -> PathBuf {
    use pr4xis_domains::formal::meta::well_behaved_lens::DecompileKind;
    use pr4xis_domains::social::software::markup::xml::owl::prx::owl_prx_cache_dir;
    use pr4xis_domains::social::software::markup::xml::uslm::corpus::prx::usc_prx_cache_dir;
    let dir = match kind {
        DecompileKind::Owl => owl_prx_cache_dir(workspace_root),
        DecompileKind::UsCode => usc_prx_cache_dir(workspace_root),
        DecompileKind::WordNet => workspace_root.join(".prx-cache").join("wordnet"),
    };
    dir.join(format!("{name}-{version}.prx.gz"))
}

/// The published source extension for a decompiled artifact's default filename.
fn source_extension(
    kind: pr4xis_domains::formal::meta::well_behaved_lens::DecompileKind,
) -> &'static str {
    use pr4xis_domains::formal::meta::well_behaved_lens::DecompileKind;
    match kind {
        DecompileKind::Owl => "owl",
        DecompileKind::UsCode | DecompileKind::WordNet => "xml",
    }
}

/// Human label for the achieved round-trip fidelity tier — the honest STAGE-1
/// statement that today's reconstruction is floor-via-stored-complement.
fn fidelity_label(
    fidelity: pr4xis_domains::formal::meta::well_behaved_lens::RoundTripFidelity,
) -> &'static str {
    use pr4xis_domains::formal::meta::well_behaved_lens::RoundTripFidelity;
    match fidelity {
        RoundTripFidelity::RawBytesComplementFloor => {
            "RawBytesComplementFloor (byte-exact via the .prx's stored, content-address-gated \
             source complement — not yet from the ontology graph alone)"
        }
        RoundTripFidelity::ByteExactGraphFaithful => {
            "ByteExactGraphFaithful (regenerated from the ontology graph alone)"
        }
    }
}

fn print_list() {
    println!("Registered datasets:");
    for entry in data_sources() {
        let desc = entry.description.as_deref().unwrap_or("");
        println!(
            "  {}@{} [{:?}] {}",
            entry.name, entry.version, entry.kind, desc
        );
        println!("    remote: {}", entry.url);
        println!("    local:  {}", entry.local_path());
        println!("    content-type: {:?}", entry.content_type());
    }
}

fn print_outcome(outcome: &FetchOutcome) {
    match outcome {
        FetchOutcome::AlreadyVerified { name } => {
            println!("  [ok]      {name}: already verified");
        }
        FetchOutcome::Fetched { name, path, bytes } => {
            println!("  [fetched] {name}: {} ({} bytes)", path.display(), bytes);
        }
        FetchOutcome::VerificationFailed { name, path, reason } => {
            eprintln!("  [FAIL]    {name}: {} — {}", path.display(), reason);
        }
        FetchOutcome::MissingAndCheckOnly { name, path } => {
            eprintln!("  [missing] {name}: {}", path.display());
        }
        FetchOutcome::MissingAndOffline { name, path } => {
            eprintln!(
                "  [offline] {name}: {} — network access disabled",
                path.display()
            );
        }
        FetchOutcome::FetchError { name, reason } => {
            eprintln!("  [error]   {name}: {reason}");
        }
        FetchOutcome::Skipped { name, reason } => {
            println!("  [skipped] {name}: {reason}");
        }
        FetchOutcome::Locked {
            name,
            path,
            bytes,
            address_hex,
        } => {
            println!(
                "  [locked]  {name}: {} ({} bytes) address={}",
                path.display(),
                bytes,
                address_hex
            );
        }
    }
}

/// Locate the workspace root. `CARGO_MANIFEST_DIR` points at the CLI crate,
/// so the workspace root is two parents up. When invoked outside Cargo
/// (e.g., from an installed binary), fall back to the current directory.
fn workspace_root() -> anyhow::Result<PathBuf> {
    if let Ok(dir) = std::env::var("PR4XIS_WORKSPACE_ROOT") {
        return Ok(PathBuf::from(dir));
    }
    if let Some(root) = option_env!("CARGO_MANIFEST_DIR") {
        let p = Path::new(root);
        if let Some(parent) = p.parent().and_then(|p| p.parent()) {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(std::env::current_dir()?)
}

// --------------------------------------------------------------------------
// `pr4xis chat` — unchanged
// --------------------------------------------------------------------------

fn run_chat(load_specs: &[String]) {
    // The chat's English, through the single loader (`english_load_owned`,
    // content-addressed compact `.prx`, ms-cheap; XML fallback inside), honoring
    // the `WORDNET_XML` dev override. Materialized ONCE behind a process `OnceLock`
    // and shared as a `&'static English`: the `ComposedReasoner` now BORROWS its
    // English (single-substrate-instance ownership), so this is the one instance.
    fn load_chat_english() -> English {
        match std::env::var("WORDNET_XML") {
            Ok(path) => load_language(&path).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            }),
            Err(_) => english_load_owned(),
        }
    }
    fn chat_english_static() -> &'static English {
        static INSTANCE: std::sync::OnceLock<English> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(load_chat_english)
    }

    // Assemble the LOADED knowledge set the chat reasons over — grounded into
    // English by ONE `ComposedReasoner`, the SAME path the wasm runtime uses.
    // Every loaded ontology carries its `ontolex:Form` label surfaces, so a
    // concept is chattable by its natural language ("is a statute a law", "is a
    // dormant fault a fault"). `loaded_names` mirrors it for the startup banner.
    let mut loaded: Vec<pr4xis_runtime::ontology::RuntimeOntology> = Vec::new();
    let mut loaded_names: Vec<String> = Vec::new();

    // The always-loaded LegalSources BASE (parity with the wasm runtime): the
    // LKIF-Core formal sources-of-law taxonomy, compiled in-process via the
    // default lexicalizing `emit` so its labels ("law", "case law", "legal
    // document") ride as queryable Form surfaces. So "is a statute a law" answers
    // out of the box, no explicit `--load`.
    match legal_sources_base() {
        Ok(onto) => {
            loaded_names.push(onto.id().as_str().to_string());
            loaded.push(onto);
        }
        Err(e) => eprintln!("  [warn] LegalSources base unavailable: {e}"),
    }

    // Materialise the U.S. Code corpus through the runtime loader: the
    // content-addressed compact `.prx` fast path when `pr4xis compile` has
    // produced one (admitted through the fail-closed `praxis.lock` gate), the
    // USLM XML otherwise. A title absent on disk is skipped, so on a fresh
    // checkout the USC contributes nothing — run `pr4xis update` then
    // `pr4xis compile --compact` to provision it.
    let usc = pr4xis_domains::social::software::markup::xml::uslm::corpus::loaded();
    if usc.section_count() > 0 {
        match usc_runtime_ontology(usc, OntologyName::new("usc")) {
            Ok(onto) => {
                loaded_names.push(format!("usc ({} sections)", usc.section_count()));
                loaded.push(onto);
            }
            Err(e) => eprintln!("  [warn] U.S. Code corpus unavailable: {e}"),
        }
    }

    // Each `--load` spec: a built-in demo, a registered OWL vocab, or an .owl file.
    for spec in load_specs {
        match load_chat_ontology(spec) {
            Ok(onto) => {
                loaded_names.push(onto.id().as_str().to_string());
                loaded.push(onto);
            }
            Err(e) => eprintln!("  [warn] --load {spec}: {e}"),
        }
    }

    // ONE reasoner over English + the whole loaded set. It BORROWS the one shared
    // English the tokenizer also uses (`reasoner.english()`, the wasm pattern), so
    // English is resident once; the loaded ontologies are shared as `Rc` handles.
    let mut loaded_rc: Vec<std::rc::Rc<_>> = loaded.into_iter().map(std::rc::Rc::new).collect();
    // GROUNDING PASS: mint every loaded ontology's declared cross-ontology type
    // edges (a USC title into `LegalSources`, any instance-functor `.prx` into its
    // target) against the loaded set — the general grounding step, driven by the
    // functor each carries as data. Order-independent and idempotent.
    pr4xis_domains::formal::meta::grounding::ground_loaded_set(&mut loaded_rc);
    let reasoner = ComposedReasoner::new(chat_english_static(), loaded_rc);
    let language: &English = reasoner.english();

    println!("pr4xis — axiomatic intelligence");
    println!(
        "  {} concepts, {} words",
        language.concept_count(),
        language.word_count(),
    );
    println!(
        "  loaded ontologies: {}",
        if loaded_names.is_empty() {
            "(none)".to_string()
        } else {
            loaded_names.join(", ")
        }
    );
    println!("  type 'quit' to exit");
    println!();

    let mut engine = engine::dialogue_engine();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        // A broken pipe (output piped to a closed reader) must not panic the
        // REPL — treat a flush error as a clean exit.
        if stdout.flush().is_err() {
            break;
        }

        let mut input = String::new();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            // Non-UTF-8 / read error on stdin: skip the line rather than panic
            // (read_line returns Err on invalid UTF-8).
            Err(_) => continue,
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Farewell detection through the language's lexicon
        let clean = input.trim().to_lowercase();
        if let Some(entry) = language.lexical_lookup(&clean)
            && entry.is_farewell()
        {
            let _ = engine.next(DialogueAction::EndDialogue);
            break;
        }

        // Resolve anaphoric expressions via language lexicon + Centering Theory
        let resolved_input = resolve_pronouns(input, engine.situation(), language);

        // Process through praxis-chat (shared logic — zero I/O). Always route
        // through the composed reasoner: it grounds the LegalSources base plus
        // every `--load`ed ontology into English, so a loaded concept answers and
        // an unloaded one abstains — the same behavior the wasm runtime has.
        let (response_text, user_act, _sys_act) = {
            let r = chat::process_with_reasoner(language, &reasoner, &resolved_input);
            (r.response, r.user_act, r.system_act)
        };

        // Extract referents for discourse tracking
        let referents: Vec<String> = resolved_input
            .split_whitespace()
            .filter_map(|w| {
                let c = w
                    .trim_matches(|c: char| c.is_ascii_punctuation())
                    .to_lowercase();
                language
                    .lexical_lookup(&c)
                    .filter(|e| e.pos_tag().is_noun())
                    .map(|_| c)
            })
            .collect();

        // Feed through the dialogue engine
        engine = match engine.next(DialogueAction::UserUtterance {
            text: input.to_string(),
            speech_act: user_act,
            referents,
        }) {
            Ok(e) => e,
            Err(pr4xis::engine::EngineError::Violated { engine: e, .. }) => e,
            Err(pr4xis::engine::EngineError::LogicalError { engine: e, .. }) => e,
        };

        println!("{}", response_text);
        println!();

        engine = match engine.next(DialogueAction::SystemResponse {
            text: response_text,
            speech_act: SpeechAct::Assertion,
        }) {
            Ok(e) => e,
            Err(pr4xis::engine::EngineError::Violated { engine: e, .. }) => e,
            Err(pr4xis::engine::EngineError::LogicalError { engine: e, .. }) => e,
        };
    }
}

/// The LegalSources base ontology, compiled in-process via the default
/// lexicalizing `emit` (labels ride as `ontolex:Form` surfaces) and materialized.
/// The CLI's always-loaded base — parity with the wasm runtime's embedded base.
fn legal_sources_base() -> anyhow::Result<pr4xis_runtime::ontology::RuntimeOntology> {
    use pr4xis_domains::social::judicial::legal_sources::ontology::LegalSourcesCategory;
    use pr4xis_runtime::emit::emit;
    use pr4xis_runtime::ontology::materialize;
    materialize(
        emit::<LegalSourcesCategory>(),
        OntologyName::new_static("LegalSources"),
    )
    .map_err(|e| anyhow::anyhow!("LegalSources materialize failed: {e}"))
}

/// Resolve a `--load` spec to a [`RuntimeOntology`] the chat reasoner can ground.
/// Resolution order: a built-in demo ontology name, then a registered OWL
/// vocabulary name (from its committed compact `.prx.gz`), then an `.owl` file
/// path. Reuses the existing loaders — no special-casing per source.
fn load_chat_ontology(spec: &str) -> anyhow::Result<pr4xis_runtime::ontology::RuntimeOntology> {
    use pr4xis_domains::applied::dependability::ontology::DependabilityCategory;
    use pr4xis_domains::social::software::markup::xml::owl::bridge::owl_runtime_ontology;
    use pr4xis_domains::social::software::markup::xml::owl::loaded_vocabularies::loaded_vocabularies;
    use pr4xis_domains::social::software::markup::xml::owl::reader::read_owl;
    use pr4xis_domains::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary;
    use pr4xis_runtime::emit::emit;
    use pr4xis_runtime::ontology::materialize;

    // 1. Built-in demo ontologies, compiled in-process (always available — no
    //    disk artifacts). `dependability` is the non-legal demo Patrick asked for.
    match spec.to_lowercase().as_str() {
        "dependability" => {
            return materialize(
                emit::<DependabilityCategory>(),
                OntologyName::new_static("Dependability"),
            )
            .map_err(|e| anyhow::anyhow!("Dependability materialize failed: {e}"));
        }
        "legal-sources" | "legalsources" | "legal" => return legal_sources_base(),
        _ => {}
    }

    // 2. A registered OWL vocabulary by name (cito, doco, …), materialised from
    //    its committed compact `.prx.gz` through the SAME registry-driven loader
    //    every OWL-vocab consumer uses. Absent when the vocab has not been
    //    compiled (`pr4xis compile --compact`).
    if let Some(vocab) = loaded_vocabularies().get(spec) {
        return owl_runtime_ontology(vocab, OntologyName::new(spec.to_string()))
            .map_err(|e| anyhow::anyhow!("OWL vocab `{spec}` materialize failed: {e}"));
    }

    // 3. A filesystem path to a raw `.owl` file — parsed and grounded by its
    //    `rdfs:label`s (the §9 OWL path). Lets a user point at any OWL vocabulary.
    let path = Path::new(spec);
    if path.exists()
        && path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("owl"))
    {
        let xml = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("read {}: {e}", path.display()))?;
        let ont = read_owl(&xml).map_err(|e| anyhow::anyhow!("parse {}: {e}", path.display()))?;
        let vocab = LoadedOwlVocabulary::from_owl_ontology(&ont);
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("owl")
            .to_string();
        return owl_runtime_ontology(&vocab, OntologyName::new(name))
            .map_err(|e| anyhow::anyhow!("{}: materialize failed: {e}", path.display()));
    }

    anyhow::bail!(
        "unknown ontology `{spec}` — expected a built-in demo (`dependability`, \
         `legal-sources`), a registered OWL vocab name (compiled via \
         `pr4xis compile --compact`), or a path to a `.owl` file"
    )
}

/// Resolve anaphoric expressions using language lexicon + discourse state.
fn resolve_pronouns(input: &str, state: &engine::DialogueState, language: &dyn Language) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    let resolved: Vec<String> = words
        .iter()
        .map(|&word| {
            let clean = word
                .trim_matches(|c: char| c.is_ascii_punctuation())
                .to_lowercase();
            let is_anaphoric = language
                .lexical_lookup(&clean)
                .is_some_and(|e| e.is_anaphoric());
            if is_anaphoric && let Some(referent) = state.resolve_anaphor() {
                return referent.to_string();
            }
            word.to_string()
        })
        .collect();
    resolved.join(" ")
}

fn load_language(path: &str) -> Result<English, String> {
    if !Path::new(path).exists() {
        return Err(format!(
            "WordNet XML not found at: {}\nRun `pr4xis update wordnet` to fetch it, or set WORDNET_XML to an existing path.",
            path
        ));
    }

    eprint!("Loading English ontology... ");
    let xml = std::fs::read_to_string(path).map_err(|e| format!("Failed to read: {}", e))?;
    let wn =
        lmf::reader::read_wordnet(&xml).map_err(|e| format!("Failed to parse WordNet: {}", e))?;
    let language = English::from_wordnet(&wn);
    eprintln!(
        "done ({} concepts, {} words)",
        language.concept_count(),
        language.word_count()
    );
    Ok(language)
}

#[cfg(test)]
mod tests {
    use super::rfc3339_utc;

    /// Known answers cross-derived with GNU `date -u -d @<secs>`: the epoch,
    /// a leap day (2000-02-29), a recent date, and an end-of-year boundary.
    #[test]
    fn rfc3339_utc_matches_known_answers() {
        assert_eq!(rfc3339_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(rfc3339_utc(1_748_736_000), "2025-06-01T00:00:00Z");
        assert_eq!(rfc3339_utc(4_102_444_799), "2099-12-31T23:59:59Z");
    }
}

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::{Parser, Subcommand};
use pr4xis_chat as chat;

#[allow(dead_code)]
#[allow(clippy::invisible_characters)]
mod usc_codegen_output {
    include!(concat!(env!("OUT_DIR"), "/usc_codegen.rs"));
}
use pr4xis_domains::applied::data_provisioning::fetch::{self, FetchOptions, FetchOutcome};
use pr4xis_domains::applied::data_provisioning::registry::{by_name, data_sources};
use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::language::Language;
use pr4xis_domains::cognitive::linguistics::pragmatics::speech_act::SpeechAct;
use pr4xis_domains::formal::information::dialogue::engine::{self, DialogueAction};
use pr4xis_domains::social::software::markup::xml::lmf;
use pr4xis_domains::social::software::markup::xml::owl::prx::EmittedArtifact;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode;

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
    Chat,
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
        #[arg(long, conflicts_with = "lock")]
        offline: bool,
        /// Regenerate `praxis.lock` from the authoritative sources.
        /// Downloads every (or one) registered entry, bypasses identity
        /// verification, writes bytes to disk, computes the SHA-256, and
        /// rewrites the corresponding `[hashes]` line in `praxis.lock`
        /// while preserving comments and key ordering. Mutually
        /// exclusive with `--check` (read-only) and `--offline` (no
        /// network) — `--lock` is the only write-only mode.
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
    /// achieved round-trip fidelity tier. Today every source reconstructs at
    /// the `RawBytesComplementFloor` tier — byte-exact via the `.prx`'s stored,
    /// sha256-gated source complement, not yet from the ontology graph alone.
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
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Chat) {
        Command::Chat => run_chat(),
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
        Command::Decompile { name, out, meter } => {
            if let Err(e) = run_decompile(name.as_deref(), out.as_deref(), meter) {
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

    if lock {
        apply_lock_outcomes(&outcomes, &workspace_root)?;
    }

    if any_failed {
        anyhow::bail!("one or more datasets failed to update");
    }
    Ok(())
}

/// Walk the outcomes from a `--lock` run and rewrite `praxis.lock` so
/// every `Locked { sha256, .. }` becomes the new pin for that source's
/// `"name@version"` key. The lockfile is read once, mutated in-memory
/// across all outcomes, then written once to keep the operation
/// atomic-ish for the common case.
fn apply_lock_outcomes(outcomes: &[FetchOutcome], workspace_root: &Path) -> anyhow::Result<()> {
    let lock_path = workspace_root.join("praxis.lock");
    let original = std::fs::read_to_string(&lock_path)
        .map_err(|e| anyhow::anyhow!("read {}: {e}", lock_path.display()))?;
    let mut text = original.clone();
    for outcome in outcomes {
        if let FetchOutcome::Locked { name, sha256, .. } = outcome {
            // Resolve `name → version` by looking up the registry entry.
            // (The outcome carries `name` only; the version lives in
            // praxis.toml via the parsed `RegistryEntry`.)
            let Some(entry) = by_name(name) else {
                eprintln!("  [warn]    {name}: not found in registry, skipping lock write");
                continue;
            };
            let key = format!("{}@{}", entry.name, entry.version);
            text =
                pr4xis_domains::applied::data_provisioning::lockfile::set_hash(&text, &key, sha256)
                    .map_err(|e| anyhow::anyhow!("praxis.lock rewrite for {key}: {e}"))?;
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

// --------------------------------------------------------------------------
// `pr4xis compile` — emit verifiable `.prx` archives (the parse-once cache)
// --------------------------------------------------------------------------

fn run_compile(lock: bool, update: bool, compact: bool) -> anyhow::Result<()> {
    use pr4xis_domains::social::software::markup::xml::lmf::prx::emit_all_wordnet_prx_gz;
    use pr4xis_domains::social::software::markup::xml::owl::prx::{
        emit_all_prx_gz as emit_all_owl_prx_gz, owl_prx_cache_dir,
    };
    use pr4xis_domains::social::software::markup::xml::uslm::corpus::prx::{
        emit_all_compact_usc_prx_gz, emit_all_usc_prx_gz, usc_compact_prx_cache_dir,
        usc_prx_cache_dir,
    };
    let workspace_root = workspace_root()?;

    // Precondition: `compile` consumes the physical sources `pr4xis update`
    // provisions. A registered, pinned, compilable source that is not on disk is
    // the "forgot to run update" failure — alert (or auto-provision with
    // `--update`) instead of silently emitting nothing for it. `--compact` only
    // touches U.S. Code titles, so it only requires those.
    let missing = missing_compilable_sources(&workspace_root, compact);
    if !missing.is_empty() {
        if update {
            println!(
                "provisioning {} missing source(s) via update…",
                missing.len()
            );
            run_update(None, false, false, false, false, false)?;
            let still = missing_compilable_sources(&workspace_root, compact);
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
fn missing_compilable_sources(workspace_root: &Path, compact_only: bool) -> Vec<String> {
    use pr4xis_domains::applied::data_provisioning::registry::{data_sources, lock_hashes};
    use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept::{
        Language, OntologyVocabulary, UsCodeTitle,
    };
    let hashes = lock_hashes();
    let mut missing = Vec::new();
    for e in data_sources() {
        // `--compact` only emits U.S. Code titles, so only those are required.
        let required = if compact_only {
            matches!(e.kind, UsCodeTitle)
        } else {
            matches!(e.kind, OntologyVocabulary | UsCodeTitle | Language)
        };
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
        lock_archive_signature, lock_compact_archive_signature,
    };
    let mut drift: Vec<String> = Vec::new();
    let mut unpinned = 0usize;
    let mut check = |a: &EmittedArtifact, pin: Option<&str>, space: &str| match pin {
        Some(p) if p == a.archive_address => {}
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

fn run_decompile(name: Option<&str>, out: Option<&Path>, meter: bool) -> anyhow::Result<()> {
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
    Ok(())
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
            "RawBytesComplementFloor (byte-exact via the .prx's stored, sha256-gated source \
             complement — not yet from the ontology graph alone)"
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
            sha256,
        } => {
            println!(
                "  [locked]  {name}: {} ({} bytes) sha256={}",
                path.display(),
                bytes,
                sha256
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

fn run_chat() {
    let wordnet_path = std::env::var("WORDNET_XML")
        .unwrap_or_else(|_| "crates/domains/data/wordnet/english-wordnet-2025.xml".into());

    let language = match load_language(&wordnet_path) {
        Ok(lang) => Arc::new(lang),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Materialise the U.S. Code corpus from the build-time codegen static.
    // The CLI build.rs writes an empty stub when no USC XML is on disk, so
    // this call never panics — `usc.section_count()` is just 0 in that case.
    let usc = Arc::new(UsCode::from_codegen(&usc_codegen_output::CODEGEN_DATA));

    println!("pr4xis — axiomatic intelligence");
    println!(
        "  {} concepts, {} words, {} USC sections",
        language.concept_count(),
        language.word_count(),
        usc.section_count(),
    );
    println!("  type 'quit' to exit");
    println!();

    let mut engine = engine::dialogue_engine();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).unwrap() == 0 {
            break;
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
        let resolved_input = resolve_pronouns(input, engine.situation(), language.as_ref());

        // Process through praxis-chat (shared logic — zero I/O)
        let (response_text, user_act, _sys_act) = chat::process(&language, &resolved_input);

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

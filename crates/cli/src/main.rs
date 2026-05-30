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

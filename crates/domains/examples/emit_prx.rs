//! Emit a `.prx.gz` distribution artifact for every registered OWL
//! vocabulary into an output directory, round-trip-validating each.
//!
//! This is the build-side driver the release-gated CI `pages` job runs to
//! produce `pages/ontologies/*.prx.gz` — published to both the GitHub
//! release (canonical immutable archive) and GitHub Pages `/ontologies/`
//! (same-origin for the wasm loader). It is a thin shell over
//! [`emit_all_prx_gz`], which does the registry walk, the
//! `read_owl → builder → rkyv → gzip` emit, and the fail-closed
//! round-trip-load validation of each written file.
//!
//! Usage: `emit_prx [OUT_DIR]` (default `dist/ontologies`). `dist/` is a
//! build output and is gitignored.
//!
//! Requires features `fetch` (rkyv + gzip + the load gate) and `codegen`
//! (the `read_owl → owl_to_builder` emit path). Cited lineage carried by
//! the emitter: Foster et al. 2007 (the bytes ⇄ vocabulary lens), rkyv
//! (Hill, zero-copy archival), NIST FIPS 180-4 §6.2 (the SHA-256 content
//! address the load gate validates against `praxis.lock`).

use std::path::PathBuf;
use std::process::ExitCode;

use pr4xis_domains::social::software::markup::xml::owl::prx::emit_all_prx_gz;

fn main() -> ExitCode {
    // Output directory: first CLI arg, else `dist/ontologies` (gitignored).
    let out_dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "dist/ontologies".to_string()),
    );

    match emit_all_prx_gz(&out_dir) {
        Ok(artifacts) => {
            let mut total_bytes: u64 = 0;
            for art in &artifacts {
                total_bytes += art.byte_len;
                // name version bytes path — one line per artifact.
                println!(
                    "{} {} {} {}",
                    art.name,
                    art.version,
                    art.byte_len,
                    art.path.display()
                );
            }
            println!(
                "emitted {} artifact(s), {} bytes total → {}",
                artifacts.len(),
                total_bytes,
                out_dir.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("emit_prx: {e}");
            ExitCode::FAILURE
        }
    }
}

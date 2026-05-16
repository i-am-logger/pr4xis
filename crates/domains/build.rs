//! Build-time codegen for Static-store sources.
//!
//! For every `[sources.<name>]` entry in workspace-root `praxis.toml`
//! with `type = "Statute"` (and a matching `[structural."<name>@<version>"]`
//! block in `praxis.lock`), this build script runs the praxis statute
//! codegen and writes `$OUT_DIR/<name>_codegen.rs`. The corresponding
//! source module under `social::compliance::statutes::<name>::` includes
//! it via `include!(concat!(env!("OUT_DIR"), "/<name>_codegen.rs"))`.
//!
//! Mirrors the wordnet codegen pattern in `crates/wasm/build.rs`. Skips
//! gracefully if praxis.toml or praxis.lock is missing (downstream
//! consumers of pr4xis-domains as a published crate hit that branch
//! until the published-crate-bundles-its-own-praxis.toml story lands).
//!
//! The structural source of truth is `praxis.lock`; the manifest only
//! enumerates which entries need codegen. Drift between the lock's
//! structural data and the canonical sha256 in `[hashes]` is caught at
//! runtime by `LockManifestAgreement`.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// praxis.toml parsing — minimal, build-time only
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawManifest {
    #[serde(default)]
    sources: HashMap<String, RawSource>,
}

#[derive(Debug, Deserialize)]
struct RawSource {
    version: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    #[allow(dead_code)]
    url: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: Option<String>,
}

// ---------------------------------------------------------------------------
// praxis.lock parsing — minimal, build-time only
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawLockFile {
    #[serde(default)]
    #[allow(dead_code)]
    hashes: HashMap<String, String>,
    #[serde(default)]
    structural: HashMap<String, RawStructural>,
}

#[derive(Debug, Deserialize)]
struct RawStructural {
    #[serde(default)]
    description: String,
    #[serde(default)]
    terms: Vec<RawTerm>,
    #[serde(default)]
    relations: Vec<RawRelation>,
}

#[derive(Debug, Deserialize)]
struct RawTerm {
    id: String,
    name: String,
    #[serde(default)]
    definition: String,
    #[serde(default)]
    lemmas: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawRelation {
    from: String,
    to: String,
    relation: String,
}

// ---------------------------------------------------------------------------
// Build entry point
// ---------------------------------------------------------------------------

fn main() {
    // CARGO_MANIFEST_DIR is crates/domains/. Workspace root is two levels up.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is always set during builds");
    let workspace_root: PathBuf = PathBuf::from(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/domains has at least two ancestor directories")
        .to_path_buf();

    let manifest_path = workspace_root.join("praxis.toml");
    let lock_path = workspace_root.join("praxis.lock");

    if !manifest_path.exists() || !lock_path.exists() {
        println!(
            "cargo:warning=praxis.toml or praxis.lock not found at workspace root \
             ({}); skipping statute codegen.",
            workspace_root.display()
        );
        return;
    }

    println!("cargo:rerun-if-changed={}", manifest_path.display());
    println!("cargo:rerun-if-changed={}", lock_path.display());

    let manifest_text = std::fs::read_to_string(&manifest_path).expect("read praxis.toml");
    let lock_text = std::fs::read_to_string(&lock_path).expect("read praxis.lock");

    let manifest: RawManifest = toml::from_str(&manifest_text).expect("parse praxis.toml");
    let lock: RawLockFile = toml::from_str(&lock_text).expect("parse praxis.lock");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is always set during builds");
    let out_dir = PathBuf::from(out_dir);

    let mut sorted_names: Vec<_> = manifest.sources.keys().collect();
    sorted_names.sort();

    for name in sorted_names {
        let src = &manifest.sources[name];
        // Codegen runs for `Statute` (jurisdiction-agnostic parent) and
        // every leaf concept that `is_a Statute` in SourceTaxonomy.
        // Listed explicitly here because build.rs reads praxis.toml as
        // raw TOML without the runtime taxonomy; new statute leaves
        // need an entry in both source_taxonomy/ontology.rs `is_a`
        // edges AND this list.
        if !matches!(src.kind.as_str(), "Statute" | "UsFederalStatute") {
            continue;
        }
        let key = format!("{}@{}", name, src.version);
        let Some(structural) = lock.structural.get(&key) else {
            println!(
                "cargo:warning=Statute `{key}` has no [structural.*] block in \
                 praxis.lock; skipping codegen for this entry."
            );
            continue;
        };

        let code = generate_statute_module(name, structural);
        let out_path = out_dir.join(format!("{}_codegen.rs", name));
        std::fs::write(&out_path, code).expect("write generated statute module");

        eprintln!(
            "Generated statute `{name}`: {} terms, {} relations -> {}",
            structural.terms.len(),
            structural.relations.len(),
            out_path.display()
        );
    }
}

// ---------------------------------------------------------------------------
// Codegen — feed structural data into pr4xis::codegen::statute
// ---------------------------------------------------------------------------

fn generate_statute_module(name: &str, structural: &RawStructural) -> String {
    use pr4xis::codegen::statute::{
        RawRelation as PrRawRelation, RawStatuteDoc, RawTerm as PrRawTerm, build_from_doc,
    };
    use pr4xis::codegen::{GenerateConfig, generate_rust};

    let terms: Vec<PrRawTerm> = structural
        .terms
        .iter()
        .map(|t| PrRawTerm {
            id: t.id.clone(),
            name: t.name.clone(),
            definition: t.definition.clone(),
            lemmas: t.lemmas.clone(),
        })
        .collect();

    let relations: Vec<PrRawRelation> = structural
        .relations
        .iter()
        .filter_map(|r| {
            parse_simple_relation(&r.relation).map(|rel| PrRawRelation {
                from: r.from.clone(),
                to: r.to.clone(),
                relation: rel,
            })
        })
        .collect();

    let doc = RawStatuteDoc {
        name: name.to_string(),
        description: structural.description.clone(),
        terms,
        relations,
    };

    let builder = build_from_doc(&doc);
    let pascal = to_pascal_case(name);
    let config = GenerateConfig::new(&format!("{name}_codegen"), &format!("{pascal}Id"));
    generate_rust(&builder, &config)
}

fn parse_simple_relation(s: &str) -> Option<pr4xis::codegen::statute::RawRel> {
    use pr4xis::codegen::statute::RawRel;
    Some(match s {
        "Requires" => RawRel::Requires,
        "SubtypeOf" => RawRel::SubtypeOf,
        "Contradicts" => RawRel::Contradicts,
        "Negates" => RawRel::Negates,
        "AlternativeTo" => RawRel::AlternativeTo,
        "AffirmativeDefenseTo" => RawRel::AffirmativeDefenseTo,
        "SafeHarborFor" => RawRel::SafeHarborFor,
        "ExhaustionRequiredFor" => RawRel::ExhaustionRequiredFor,
        "Precedes" => RawRel::Precedes { max_days: None },
        "Implies" => RawRel::Implies { consequence: None },
        "Composes" => RawRel::Composes { into: None },
        "Triggers" => RawRel::Triggers { obligation: None },
        "Rebuts" => RawRel::Rebuts { burden: None },
        _ => return None,
    })
}

fn to_pascal_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut capitalize_next = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
            continue;
        }
        if capitalize_next {
            out.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

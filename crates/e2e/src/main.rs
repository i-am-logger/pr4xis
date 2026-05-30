//! End-to-end test for the pr4xis web app, in Rust.
//!
//! Drives a real headless browser through a WebDriver (geckodriver) with
//! [`fantoccini`] — the Rust-native alternative to JS E2E frameworks.
//! Exercises the click-to-load paths the unit tests can't:
//!   1. USLM source: click Load on a registered statute, verify the worker
//!      downloads the authoritative XML and materialises it into a live
//!      `UsCode` (the card flips to `.loaded`).
//!   2. OWL `.prx.gz`: click "Load .prx" on a registered OWL vocabulary,
//!      verify the worker downloads the gzipped rkyv envelope and the
//!      embedded source-hash gate validates it against the build-time
//!      praxis.lock pin (the row flips to `.loaded`). This is the dual-load
//!      capability's hash-validated leg, exercised end-to-end.
//!
//! Prereqs (CI wires these up — see `.github/workflows/ci.yml` `e2e`):
//!   - `pr4xis-web` serving the built wasm + staged `/sources/` + staged
//!     `/ontologies/` (default <http://localhost:3000>, override with
//!     `PRAXIS_WEB_URL`);
//!   - a WebDriver listening (default <http://localhost:4444>, override
//!     with `WEBDRIVER_URL`) — e.g. `geckodriver`.
//!
//! Exits non-zero on any failure (no live server / driver, element never
//! appears, load never completes, hash gate rejects), so CI fails loudly.

use std::time::Duration;

use fantoccini::{Client, ClientBuilder, Locator};

/// Wait for a CSS selector with a deadline, labeling the wait in stderr
/// so a `WaitTimeout` failure says exactly which step missed its budget.
/// On timeout, also dump the rendered DOM + console state so the next
/// debug iteration has the actual failure context, not a bare error.
async fn wait_for(
    client: &Client,
    label: &str,
    selector: &str,
    deadline: Duration,
) -> Result<fantoccini::elements::Element, Box<dyn std::error::Error>> {
    eprintln!(
        "[e2e] waiting up to {}s for {label}: {selector}",
        deadline.as_secs()
    );
    match client
        .wait()
        .at_most(deadline)
        .for_element(Locator::Css(selector))
        .await
    {
        Ok(el) => {
            eprintln!("[e2e]   found {label}");
            Ok(el)
        }
        Err(e) => {
            eprintln!("[e2e] TIMEOUT on {label}: {selector}");
            dump_page_state(client, label).await;
            Err(e.into())
        }
    }
}

/// On a wait-timeout, capture as much page-state as the WebDriver
/// protocol exposes: current URL, document title, every present
/// `data-source` / `data-ontology` attribute (so we see what the
/// catalog actually rendered), and any JS errors the page chose to
/// stash on `window.__praxis_e2e_errors__`. Best-effort — never
/// throws, since this runs from inside an error path.
async fn dump_page_state(client: &Client, label: &str) {
    eprintln!("[e2e] ---- page state at {label} timeout ----");
    if let Ok(url) = client.current_url().await {
        eprintln!("[e2e]   url:   {url}");
    }
    if let Ok(title) = client.title().await {
        eprintln!("[e2e]   title: {title}");
    }
    // Extract the bracketed key from the label (e.g. "ontology loaded
    // marker [biro]" → "biro") so the per-element probe can scope to
    // the actual failing item without us having to thread it separately.
    let key = label
        .rsplit_once('[')
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(k, _)| k.to_string())
        .unwrap_or_default();
    // Enumerate every data-source / data-ontology / data-load* element
    // present in the DOM right now. If the list is empty the catalog
    // never rendered; if it's populated with different titles the
    // selector is wrong; if our title is there but the card lacks
    // .loaded the load itself stalled.
    let probes: &[(&str, &str)] = &[
        (
            "data-source elements",
            "return Array.from(document.querySelectorAll('[data-source]')).map(e => e.getAttribute('data-source'));",
        ),
        (
            "data-ontology elements",
            "return Array.from(document.querySelectorAll('[data-ontology]')).map(e => e.getAttribute('data-ontology'));",
        ),
        (
            "data-load elements",
            "return Array.from(document.querySelectorAll('[data-load]')).map(e => e.getAttribute('data-load'));",
        ),
        (
            "data-load-prx elements",
            "return Array.from(document.querySelectorAll('[data-load-prx]')).map(e => e.getAttribute('data-load-prx'));",
        ),
        (
            "body text first 500 chars",
            "return (document.body && document.body.innerText || '').slice(0, 500);",
        ),
        (
            "praxis e2e errors (window.__praxis_e2e_errors__)",
            "return (window.__praxis_e2e_errors__ || []);",
        ),
        // The selector that just timed out is the only one we have to
        // explain — dump everything about every element that even
        // mentions the failing key in a data-* attribute. Shows the
        // card's actual class list, the bytes of its visible text, and
        // the immediate descendant kinds (badge, progress, retry).
        (
            "elements matching the timeout's data-* key (outerHTML, classes, text)",
            "var key = arguments[0] || ''; \
             var hits = []; \
             document.querySelectorAll('[data-source],[data-ontology],[data-load],[data-load-prx]').forEach(e => { \
               var v = e.getAttribute('data-source') || e.getAttribute('data-ontology') || e.getAttribute('data-load') || e.getAttribute('data-load-prx'); \
               if (v && v === key) { \
                 hits.push({ tag: e.tagName, classes: e.className, dataset: Object.assign({}, e.dataset), text: (e.innerText || '').slice(0, 240), html: (e.outerHTML || '').slice(0, 600) }); \
               } \
             }); \
             return hits;",
        ),
    ];
    for (name, script) in probes {
        // The per-element probe (last entry, recognised by mention of
        // arguments[0]) needs the failing key passed in. Cheap match
        // on substring keeps the probe table flat.
        let args = if script.contains("arguments[0]") {
            vec![serde_json::Value::String(key.clone())]
        } else {
            vec![]
        };
        match client.execute(script, args).await {
            Ok(v) => eprintln!("[e2e]   {name}: {v}"),
            Err(e) => eprintln!("[e2e]   {name}: <execute failed: {e}>"),
        }
    }
    eprintln!("[e2e] ----");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = env_or("PRAXIS_WEB_URL", "http://localhost:3000");
    let webdriver = env_or("WEBDRIVER_URL", "http://localhost:4444");
    // A small title keeps the USLM download + parse fast. usc_title_1 ≈ 0.3 MB.
    let title = env_or("PRAXIS_E2E_TITLE", "usc_title_1");
    // Smallest registered OWL vocabulary: biro (~15 entities) — the .prx.gz
    // envelope is a few KB, so the validated-load round-trip stays fast.
    let ontology = env_or("PRAXIS_E2E_ONTOLOGY", "biro");
    // Second OWL vocabulary loaded unconditionally — cito (~83 KB source,
    // well-annotated; exercises the rdf:nodeID + xml:lang + rdf:datatype
    // paths the smaller vocabs don't hit). Override via
    // `PRAXIS_E2E_ONTOLOGY_2` for hygiene.
    let ontology_2 = env_or("PRAXIS_E2E_ONTOLOGY_2", "cito");

    // Headless Firefox via geckodriver.
    let mut caps = serde_json::map::Map::new();
    caps.insert(
        "moz:firefoxOptions".to_string(),
        serde_json::json!({ "args": ["-headless"] }),
    );

    let client = ClientBuilder::rustls()?
        .capabilities(caps)
        .connect(&webdriver)
        .await?;

    let outcome = run_all(&client, &base, &title, &ontology, &ontology_2).await;
    // Always release the browser session, then surface the result.
    let _ = client.close().await;
    outcome?;

    println!(
        "E2E OK: {title} (USLM source) + {ontology} + {ontology_2} \
         (OWL .prx.gz, hash-validated) materialised into live ontologies."
    );
    Ok(())
}

async fn run_all(
    client: &fantoccini::Client,
    base: &str,
    title: &str,
    ontology: &str,
    ontology_2: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    client.goto(base).await?;
    run_source(client, title).await?;
    // Same page session — the worker holds the wasm runtime; loading a
    // second source compounds on the first. After this the catalog reports
    // two loaded entries.
    run_ontology_prx(client, ontology).await?;
    // Load a second OWL vocabulary alongside the first to exercise the
    // dual-load path on more than one source kind. Both ontology cards
    // end up `.loaded` in the same worker session.
    run_ontology_prx(client, ontology_2).await?;
    Ok(())
}

/// Click-to-load a USLM statute via its authoritative source XML.
async fn run_source(
    client: &fantoccini::Client,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. The self-model page is the default tab. Wait for the source
    //    catalog to render — this only happens after the Web Worker has
    //    initialised the wasm and the first self_describe round-trips.
    let card = format!("[data-source=\"{title}\"]");
    wait_for(
        client,
        &format!("source card [{title}]"),
        &card,
        Duration::from_secs(60),
    )
    .await?;

    // 2. The title starts Available, with a Load button.
    let load = format!("[data-load=\"{title}\"]");
    let load_btn = wait_for(
        client,
        &format!("source load button [{title}]"),
        &load,
        Duration::from_secs(30),
    )
    .await?;

    // 3. Click Load — the worker downloads the authoritative USLM XML and
    //    parses it into a live UsCode off the main thread.
    load_btn.click().await?;

    // 4. On success the catalog re-renders and the card carries `.loaded`.
    let loaded = format!(".source-card.loaded[data-source=\"{title}\"]");
    wait_for(
        client,
        &format!("source loaded marker [{title}]"),
        &loaded,
        Duration::from_secs(120),
    )
    .await?;

    Ok(())
}

/// Click-to-load an OWL vocabulary via its hash-validated `.prx.gz`
/// distribution envelope. Exercises the embedded source-hash gate end-to-end:
/// a tampered envelope would surface as `Failed` in the row and the
/// `.loaded` selector would never match.
async fn run_ontology_prx(
    client: &fantoccini::Client,
    ontology: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Wait for the OWL vocabulary's row in the Ontologies catalog.
    let card = format!("[data-ontology=\"{ontology}\"]");
    wait_for(
        client,
        &format!("ontology card [{ontology}]"),
        &card,
        Duration::from_secs(60),
    )
    .await?;

    // 2. The "Load .prx" button — the hash-validated leg.
    let load = format!("[data-load-prx=\"{ontology}\"]");
    let load_btn = wait_for(
        client,
        &format!("ontology load-prx button [{ontology}]"),
        &load,
        Duration::from_secs(30),
    )
    .await?;

    // 3. Click Load .prx — the worker streams the `.prx.gz`, the wasm gate
    //    gunzips, bytecheck-validates the rkyv envelope, and asserts the
    //    embedded source-hash equals the praxis.lock pin baked into the
    //    build manifest. Fail-closed.
    load_btn.click().await?;

    // 4. On success the catalog re-renders and the card carries `.loaded`.
    let loaded = format!(".source-card.loaded[data-ontology=\"{ontology}\"]");
    wait_for(
        client,
        &format!("ontology loaded marker [{ontology}]"),
        &loaded,
        Duration::from_secs(60),
    )
    .await?;

    Ok(())
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

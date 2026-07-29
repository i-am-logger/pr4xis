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

/// Deadline for the WebDriver `newSession` handshake.
///
/// `ClientBuilder::connect` performs `newSession` and carries no deadline of
/// its own, so a geckodriver that accepts the TCP connection but never answers
/// blocks forever: every per-assertion deadline in `run_all` applies only
/// AFTER `connect` returns. Without this the whole harness is unbounded, which
/// is the class of hang that let one CI job sit for 182 minutes.
///
/// 60s is roughly 10x the observed handshake — headless Firefox starts in
/// ~3-6s on `ubuntu-latest` — so it cannot fire on ordinary slowness.
const NEW_SESSION_TIMEOUT: Duration = Duration::from_secs(60);

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
            "data-load-fast elements",
            "return Array.from(document.querySelectorAll('[data-load-fast]')).map(e => e.getAttribute('data-load-fast'));",
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
             document.querySelectorAll('[data-source],[data-ontology],[data-load],[data-load-prx],[data-load-fast]').forEach(e => { \
               var v = e.getAttribute('data-source') || e.getAttribute('data-ontology') || e.getAttribute('data-load') || e.getAttribute('data-load-prx') || e.getAttribute('data-load-fast'); \
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
    // A DIFFERENT title for the zero-copy fast-load button (task #21) — must
    // differ from `title` above, since loading a title via either route
    // flips its card to `.loaded` and removes BOTH buttons, so the same
    // title can't exercise both click paths in one page session.
    let title_fast = env_or("PRAXIS_E2E_TITLE_FAST", "usc_title_18");
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

    // Bound the handshake (see NEW_SESSION_TIMEOUT). Split across two
    // statements because `capabilities` returns `&mut Self` while `connect`
    // borrows `&self` — the builder has to outlive the future being timed.
    let mut builder = ClientBuilder::rustls()?;
    builder.capabilities(caps);
    let client = tokio::time::timeout(NEW_SESSION_TIMEOUT, builder.connect(&webdriver))
        .await
        .map_err(|_| {
            format!(
                "WebDriver newSession did not answer within {}s at {webdriver}",
                NEW_SESSION_TIMEOUT.as_secs()
            )
        })??;

    let outcome = run_all(&client, &base, &title, &title_fast, &ontology, &ontology_2).await;
    // Always release the browser session, then surface the result.
    let _ = client.close().await;
    outcome?;

    println!(
        "E2E OK: {title} (USLM source) + {title_fast} (zero-copy rkyv archive) + \
         {ontology} + {ontology_2} (OWL .prx.gz, hash-validated) materialised into \
         live ontologies."
    );
    Ok(())
}

async fn run_all(
    client: &fantoccini::Client,
    base: &str,
    title: &str,
    title_fast: &str,
    ontology: &str,
    ontology_2: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // The app is ONE page with three hash-routed tabs, and Chat is the
    // default. The source catalog these tests drive lives in the Engine tab,
    // so land there explicitly rather than relying on which tab opens first:
    // a card in a hidden tab is present in the DOM and still unclickable, so
    // the selector waits succeed and the click fails "could not be scrolled
    // into view" — which reads like a layout bug rather than a routing one.
    client.goto(&format!("{base}#engine")).await?;
    // Let eager residency settle before driving a single click.
    //
    // The boot deliberately does not await the eager loads, so for the first
    // seconds the catalog is in motion: a card can carry a Load button for a
    // source whose fetch is already in flight, and the re-render that lands
    // when it arrives removes the very element a click was about to hit. That
    // is a real race for a reader, not only for this suite, which is why the
    // page publishes the settled marker rather than the suite guessing a
    // sleep. Generous, because this waits on ~35 MB of Title 42.
    wait_for(
        client,
        "eager residency settled (the catalog has stopped moving)",
        "body[data-eager-residency='settled']",
        Duration::from_secs(180),
    )
    .await?;
    run_source(client, title).await?;
    // Same page session — the worker holds the wasm runtime; loading a
    // second source compounds on the first. After this the catalog reports
    // two loaded entries.
    run_usc_archive_fast(client, title_fast).await?;
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
    // 1. `run_all` has already routed to the Engine tab. Wait for the source
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

/// Click-to-load a USC title via its zero-copy, pre-projected `rkyv`
/// archive (task #21) — the "Load (fast)" button `stage_usc_archives`
/// (build.rs) offers alongside the raw-XML route `run_source` exercises.
/// Same loaded-marker shape (`data-source`, not a separate attribute) since
/// both routes install the SAME `RuntimeOntology` under the SAME name —
/// only the load PATH differs.
async fn run_usc_archive_fast(
    client: &fantoccini::Client,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let card = format!("[data-source=\"{title}\"]");
    wait_for(
        client,
        &format!("source card [{title}]"),
        &card,
        Duration::from_secs(60),
    )
    .await?;

    let load = format!("[data-load-fast=\"{title}\"]");
    let load_btn = wait_for(
        client,
        &format!("source load-fast button [{title}]"),
        &load,
        Duration::from_secs(30),
    )
    .await?;

    // Click Load (fast) — the worker streams the `.cprx`, and the wasm gate
    // re-derives the archive's Merkle root and refuses on mismatch. No
    // client-side USLM XML parse happens on this path.
    load_btn.click().await?;

    let loaded = format!(".source-card.loaded[data-source=\"{title}\"]");
    wait_for(
        client,
        &format!("source loaded marker [{title}]"),
        &loaded,
        Duration::from_secs(60),
    )
    .await?;

    Ok(())
}

/// Put an eagerly-resident OWL vocabulary DOWN and pick it back UP, exercising
/// the hash-validated `.prx.gz` distribution envelope on the way back in.
///
/// This used to be a plain click-to-load, which stopped working the moment the
/// deployment declared every published vocabulary eagerly resident: the page
/// renders a Load button only on a card that is NOT loaded, so after boot there
/// was no button left to click, and the suite waited 30s for an element that by
/// then could not exist. The failure looked like a timeout — the shape a
/// genuinely broken page also has — and it gated the deploy.
///
/// The round-trip is the honest replacement, and a stronger test than the
/// original: it drives the same fail-closed load leg (the worker streams the
/// `.prx.gz`, the wasm gate gunzips, bytecheck-validates the rkyv envelope and
/// asserts the embedded source-hash equals the `praxis.lock` pin baked into the
/// build manifest — a tampered envelope surfaces as `Failed` and `.loaded`
/// never matches), and additionally proves the unload path a reader depends on
/// to decline a corpus the deployment fetched on their behalf.
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

    // 2. Put it down. Eager residency is `Residency::Eager` — releasable,
    //    because the deployment fetched it without being asked. The button
    //    exists only if the engine says so, so this also witnesses that.
    let unload = format!("[data-unload=\"{ontology}\"]");
    let unload_btn = wait_for(
        client,
        &format!("ontology unload button [{ontology}] (eager sources are releasable)"),
        &unload,
        Duration::from_secs(60),
    )
    .await?;
    unload_btn.click().await?;

    // 3. Released, the card returns to the available side of the knowledge
    //    boundary and the load routes come back with it.
    let load = format!("[data-load-prx=\"{ontology}\"]");
    let load_btn = wait_for(
        client,
        &format!("ontology load-prx button [{ontology}] returns after unload"),
        &load,
        Duration::from_secs(60),
    )
    .await?;

    // 4. Pick it back up through the hash-validated envelope.
    load_btn.click().await?;

    // 5. On success the catalog re-renders and the card carries `.loaded`.
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

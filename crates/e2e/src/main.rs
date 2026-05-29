//! End-to-end test for the pr4xis web app, in Rust.
//!
//! Drives a real headless browser through a WebDriver (geckodriver) with
//! [`fantoccini`] — the Rust-native alternative to JS E2E frameworks.
//! Exercises the full click-to-load path the unit tests can't: open the
//! page, click Load on a registered statute, and verify the worker
//! downloads the authoritative USLM XML and materializes it into a live
//! ontology (the card flips to Loaded).
//!
//! Prereqs (CI wires these up — see `.github/workflows/ci.yml` `e2e`):
//!   - `pr4xis-web` serving the built wasm + staged `/sources/` (default
//!     <http://localhost:3000>, override with `PRAXIS_WEB_URL`);
//!   - a WebDriver listening (default <http://localhost:4444>, override
//!     with `WEBDRIVER_URL`) — e.g. `geckodriver`.
//!
//! Exits non-zero on any failure (no live server / driver, element never
//! appears, load never completes), so CI fails loudly.

use std::time::Duration;

use fantoccini::{ClientBuilder, Locator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = env_or("PRAXIS_WEB_URL", "http://localhost:3000");
    let webdriver = env_or("WEBDRIVER_URL", "http://localhost:4444");
    // A small title keeps the download + parse fast. usc_title_1 ≈ 0.3 MB.
    let title = env_or("PRAXIS_E2E_TITLE", "usc_title_1");

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

    let outcome = run(&client, &base, &title).await;
    // Always release the browser session, then surface the result.
    let _ = client.close().await;
    outcome?;

    println!("E2E OK: {title} downloaded from source and materialized into a live ontology.");
    Ok(())
}

async fn run(
    client: &fantoccini::Client,
    base: &str,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    client.goto(base).await?;

    // 1. The self-model page is the default tab. Wait for the source
    //    catalog to render — this only happens after the Web Worker has
    //    initialised the wasm and the first self_describe round-trips.
    let card = format!("[data-source=\"{title}\"]");
    client
        .wait()
        .at_most(Duration::from_secs(60))
        .for_element(Locator::Css(&card))
        .await?;

    // 2. The title starts Available, with a Load button.
    let load = format!("[data-load=\"{title}\"]");
    let load_btn = client
        .wait()
        .at_most(Duration::from_secs(30))
        .for_element(Locator::Css(&load))
        .await?;

    // 3. Click Load — the worker downloads the authoritative USLM XML and
    //    parses it into a live UsCode off the main thread.
    load_btn.click().await?;

    // 4. On success the catalog re-renders and the card carries `.loaded`.
    let loaded = format!(".source-card.loaded[data-source=\"{title}\"]");
    client
        .wait()
        .at_most(Duration::from_secs(120))
        .for_element(Locator::Css(&loaded))
        .await?;

    Ok(())
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

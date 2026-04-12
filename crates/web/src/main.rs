use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use tiny_http::{Header, Response, Server, StatusCode};

/// Global reload counter — incremented when watched files change.
static RELOAD_COUNTER: AtomicU64 = AtomicU64::new(0);

fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let workspace = find_workspace_root();
    let root = workspace.join("docs");
    eprintln!("pr4xis-web");
    eprintln!("  http://localhost:{port}/          — presentation");
    eprintln!("  http://localhost:{port}/chat/     — WASM chatbot");
    eprintln!();

    // Watch for .rs file changes in crates/ — trigger WASM rebuild.
    let crates_dir = workspace.join("crates");
    thread::spawn(move || watch_and_rebuild(&crates_dir));

    let server = Server::http(format!("0.0.0.0:{port}")).expect("failed to bind");
    let server = Arc::new(server);

    // Serve requests on thread pool.
    let num_threads = 4;
    let mut handles = Vec::with_capacity(num_threads);
    for _ in 0..num_threads {
        let server = Arc::clone(&server);
        let root = root.clone();
        handles.push(thread::spawn(move || {
            for request in server.incoming_requests() {
                handle_request(&root, request);
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
}

fn handle_request(root: &Path, request: tiny_http::Request) {
    let url = request.url().to_string();

    // SSE endpoint for live reload.
    if url == "/livereload" {
        serve_sse(request);
        return;
    }

    // Resolve file path.
    let rel = url.trim_start_matches('/');
    let mut path = root.join(rel);

    // Directory → index.html
    if path.is_dir() {
        path = path.join("index.html");
    }

    // 404
    if !path.exists() || !path.starts_with(root) {
        let _ = request.respond(
            Response::from_string("404 Not Found")
                .with_status_code(StatusCode(404))
                .with_header(content_type("text/plain")),
        );
        return;
    }

    let data = match fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            let _ = request.respond(
                Response::from_string(format!("500: {e}")).with_status_code(StatusCode(500)),
            );
            return;
        }
    };

    let mime = mime_for_path(&path);

    // Inject livereload script into HTML responses.
    if mime == "text/html" {
        let html = String::from_utf8_lossy(&data);
        let injected = inject_livereload(&html);
        let _ = request.respond(
            Response::from_string(injected)
                .with_header(content_type(mime))
                .with_header(no_cache())
                .with_header(cross_origin_opener())
                .with_header(cross_origin_embedder()),
        );
    } else {
        let _ = request.respond(
            Response::from_data(data)
                .with_header(content_type(mime))
                .with_header(cache_header(mime))
                .with_header(cross_origin_opener())
                .with_header(cross_origin_embedder()),
        );
    }
}

fn serve_sse(request: tiny_http::Request) {
    let counter = RELOAD_COUNTER.load(Ordering::Relaxed);

    // Long-poll: wait up to 30s for a change, then send the event.
    let start = std::time::Instant::now();
    loop {
        let current = RELOAD_COUNTER.load(Ordering::Relaxed);
        if current != counter {
            // File changed — tell client to reload.
            let body = "data: reload\n\n".to_string();
            let _ = request.respond(
                Response::from_string(body)
                    .with_header(content_type("text/event-stream"))
                    .with_header(no_cache()),
            );
            return;
        }
        if start.elapsed() > Duration::from_secs(30) {
            // Timeout — send keepalive, client will reconnect.
            let _ = request.respond(
                Response::from_string("data: ping\n\n")
                    .with_header(content_type("text/event-stream"))
                    .with_header(no_cache()),
            );
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

/// Watch crates/ for .rs changes. Rebuild WASM on change.
fn watch_and_rebuild(crates_dir: &Path) {
    let mut mtimes: HashMap<PathBuf, SystemTime> = HashMap::new();

    // Initial scan.
    scan_rs_files(crates_dir, &mut mtimes);

    loop {
        thread::sleep(Duration::from_secs(1));

        let mut changed = false;
        let mut current: HashMap<PathBuf, SystemTime> = HashMap::new();
        scan_rs_files(crates_dir, &mut current);

        // Detect new or modified files.
        for (path, mtime) in &current {
            match mtimes.get(path) {
                Some(prev) if prev == mtime => {}
                _ => {
                    eprintln!("  changed: {}", path.display());
                    changed = true;
                }
            }
        }

        if changed {
            eprintln!("rebuilding WASM...");
            let workspace = crates_dir.parent().unwrap();
            // Inherit current PATH (from devenv shell) so wasm-pack is found
            let status = std::process::Command::new("wasm-pack")
                .args(["build", "--target", "web", "--release"])
                .current_dir(workspace.join("crates/wasm"))
                .env("PATH", std::env::var("PATH").unwrap_or_default())
                .status();

            match status {
                Ok(s) if s.success() => {
                    // Copy built WASM to docs/chat/pkg/
                    let src = workspace.join("crates/wasm/pkg");
                    let dst = workspace.join("docs/chat/pkg");
                    let _ = fs::create_dir_all(&dst);
                    if let Ok(entries) = fs::read_dir(&src) {
                        for entry in entries.flatten() {
                            let target = dst.join(entry.file_name());
                            let _ = fs::copy(entry.path(), target);
                        }
                    }
                    eprintln!("  WASM rebuilt OK");
                    RELOAD_COUNTER.fetch_add(1, Ordering::Relaxed);
                }
                Ok(s) => eprintln!("  WASM build failed (exit {})", s),
                Err(e) => eprintln!("  failed to run wasm-pack: {e}"),
            }

            mtimes = current;
        }
    }
}

fn scan_rs_files(dir: &Path, out: &mut HashMap<PathBuf, SystemTime>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip target/ and pkg/ directories.
            let name = path.file_name().unwrap_or_default();
            if name == "target" || name == "pkg" {
                continue;
            }
            scan_rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs")
            && let Ok(meta) = fs::metadata(&path)
            && let Ok(mtime) = meta.modified()
        {
            out.insert(path, mtime);
        }
    }
}

fn inject_livereload(html: &str) -> String {
    let script = r#"<script>
(function() {
  function poll() {
    fetch('/livereload')
      .then(r => r.text())
      .then(t => { if (t.includes('reload')) location.reload(); else poll(); })
      .catch(() => setTimeout(poll, 2000));
  }
  poll();
})();
</script>"#;

    if let Some(pos) = html.rfind("</body>") {
        let mut out = String::with_capacity(html.len() + script.len() + 1);
        out.push_str(&html[..pos]);
        out.push_str(script);
        out.push('\n');
        out.push_str(&html[pos..]);
        out
    } else {
        format!("{html}\n{script}")
    }
}

fn find_workspace_root() -> PathBuf {
    // Walk up from cwd to find workspace root (has Cargo.toml with [workspace]).
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists()
            && let Ok(content) = fs::read_to_string(&cargo_toml)
            && content.contains("[workspace]")
        {
            return dir;
        }
        if !dir.pop() {
            eprintln!("could not find workspace root — using cwd");
            return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }
    }
}

fn mime_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("wasm") => "application/wasm",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("mp4") => "video/mp4",
        Some("ico") => "image/x-icon",
        Some("txt" | "md") => "text/plain",
        _ => "application/octet-stream",
    }
}

fn content_type(mime: &str) -> Header {
    Header::from_bytes("Content-Type", mime.as_bytes()).expect("valid header")
}

fn no_cache() -> Header {
    Header::from_bytes("Cache-Control", "no-cache, no-store, must-revalidate")
        .expect("valid header")
}

/// Enable high-resolution performance.now() in browsers.
/// Without these headers, timers are clamped to ~100µs (Spectre mitigation).
fn cross_origin_opener() -> Header {
    Header::from_bytes("Cross-Origin-Opener-Policy", "same-origin").expect("valid header")
}

fn cross_origin_embedder() -> Header {
    Header::from_bytes("Cross-Origin-Embedder-Policy", "require-corp").expect("valid header")
}

fn cache_header(mime: &str) -> Header {
    // Cache WASM and assets, not HTML.
    if mime == "application/wasm" || mime.starts_with("image/") {
        Header::from_bytes("Cache-Control", "public, max-age=3600").expect("valid header")
    } else {
        no_cache()
    }
}

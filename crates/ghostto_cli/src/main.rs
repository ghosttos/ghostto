//! `ghostto` — start the local server, print the URL, wait for Ctrl-C.
//!
//! M0: no guest boots yet. The tab loads the UI and connects to `/ws`.

use ghostto_http::{bind, serve};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ghostto=info,tower_http=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    let web_dir = web_dir();

    let bound = match bind().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("ghostto: {e}");
            eprintln!("ghostto: is another instance already running?");
            std::process::exit(1);
        }
    };

    println!("ghostto listening on http://{}", bound.addr);
    println!("open the URL above; press Ctrl-C to stop");

    tokio::select! {
        res = serve(bound, &web_dir) => {
            if let Err(e) = res {
                eprintln!("ghostto: server error: {e}");
                std::process::exit(1);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\nghostto: shutting down");
        }
    }

    Ok(())
}

/// Locate `web/` relative to: `GHOSTTO_WEB_DIR`, then the executable,
/// then the current directory. Same resolution order as the README.
fn web_dir() -> String {
    if let Ok(dir) = std::env::var("GHOSTTO_WEB_DIR") {
        return dir;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("web");
            if candidate.is_dir() {
                return candidate.to_string_lossy().into_owned();
            }
        }
    }
    "web".to_string()
}

//! HTTP server and port policy.
//!
//! Rules (from `README.md`):
//! - bind `127.0.0.1` only, never `0.0.0.0`
//! - try `7777`, fall back to `7771`, else fail loudly
//! - serve `web/` on the root, WebSocket at `/ws`

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;

pub const PREFERRED_PORT: u16 = 7777;
pub const FALLBACK_PORT: u16 = 7771;

pub struct Bound {
    pub listener: tokio::net::TcpListener,
    pub addr: SocketAddr,
}

/// Bind `127.0.0.1:7777`, then `:7771`. Both busy → hard error.
pub async fn bind() -> std::io::Result<Bound> {
    for port in [PREFERRED_PORT, FALLBACK_PORT] {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok(Bound { listener, addr }),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                info!(port, "port busy, trying next");
            }
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        "both 7777 and 7771 are in use",
    ))
}

/// Router for the local UI + WebSocket.
pub fn app(web_dir: &str) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(ghostto_ws::upgrade))
        .fallback_service(ServeDir::new(web_dir))
        .layer(TraceLayer::new_for_http())
}

pub async fn serve(bound: Bound, web_dir: &str) -> std::io::Result<()> {
    axum::serve(bound.listener, app(web_dir)).await
}

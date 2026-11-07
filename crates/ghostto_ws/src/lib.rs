//! WebSocket signal channel at `/ws`.
//!
//! M0 scope: accept upgrades, send `Event::Ready`, echo `Ping` as a
//! `Log` event, return `Event::Error { "not yet implemented" }` for
//! commands that will be wired to the engine in M1+.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use ghostto_proto::{Command, Event};
use tracing::{debug, info, warn};

/// Axum handler — upgrade and run the per-connection loop.
pub async fn upgrade(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle)
}

async fn handle(mut socket: WebSocket) {
    info!("ws: client connected");

    let ready = Event::Ready {
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    if send_event(&mut socket, &ready).await.is_err() {
        return;
    }

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => match serde_json::from_str::<Command>(&text) {
                Ok(cmd) => {
                    debug!(?cmd, "ws: received");
                    dispatch(&mut socket, cmd).await;
                }
                Err(e) => {
                    let _ = send_event(
                        &mut socket,
                        &Event::Error { message: format!("bad message: {e}") },
                    )
                    .await;
                }
            },
            Ok(Message::Binary(_)) => {
                warn!("ws: binary frames from clients are ignored in M0");
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(e) => {
                warn!(?e, "ws: error");
                break;
            }
        }
    }

    info!("ws: client disconnected");
}

async fn send_event(socket: &mut WebSocket, ev: &Event) -> Result<(), ()> {
    let text = serde_json::to_string(ev).map_err(|_| ())?;
    socket.send(Message::Text(text)).await.map_err(|_| ())
}

async fn dispatch(socket: &mut WebSocket, cmd: Command) {
    match cmd {
        Command::Ping { t } => {
            let _ = send_event(
                socket,
                &Event::Log { level: "debug".into(), message: format!("pong {t}") },
            )
            .await;
        }
        _ => {
            let _ = send_event(
                socket,
                &Event::Error { message: "not yet implemented (M0 stub)".into() },
            )
            .await;
        }
    }
}

//! Optional web server (feature `web-server`, OFF by default; also off at runtime unless
//! `--enable` is passed). Serves the Trunk-built `dist/` directory and a WebSocket device
//! bridge stub at `/ws`.
//!
//! The browser UI cannot access server devices itself, so this process is the only place
//! where device access (GQRX TCP, USB, ...) may live. A real backend should implement
//! [`DeviceBackend`]. **There is no authentication**: bind to 127.0.0.1 (default) unless you
//! put it behind an authenticating reverse proxy.
//!
//! Usage: `rustcars-web --enable [--bind 127.0.0.1:8080] [--dist dist]`
use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use std::{net::SocketAddr, path::PathBuf};
use tower_http::services::ServeDir;

/// Server-side device bridge. One instance per WebSocket connection.
/// Future backends (GQRX remote-control TCP, USB SDR, ...) implement this trait.
pub trait DeviceBackend: Send {
    fn name(&self) -> &'static str;
    /// Handle one text command from the browser and return a text reply.
    fn handle(&mut self, cmd: &str) -> String;
}

/// Stub backend: `ping` -> `pong`, `name` -> backend name, anything else is echoed.
/// It performs NO device access.
pub struct EchoBackend;

impl DeviceBackend for EchoBackend {
    fn name(&self) -> &'static str {
        "echo-stub"
    }
    fn handle(&mut self, cmd: &str) -> String {
        match cmd.trim() {
            "ping" => "pong".to_owned(),
            "name" => self.name().to_owned(),
            other => format!("echo: {other}"),
        }
    }
}

struct Args {
    enable: bool,
    bind: SocketAddr,
    dist: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        enable: false,
        bind: SocketAddr::from(([127, 0, 0, 1], 8080)),
        dist: PathBuf::from("dist"),
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--enable" => a.enable = true,
            "--bind" => {
                let v = it.next().ok_or("--bind needs ADDR:PORT")?;
                a.bind = v.parse().map_err(|e| format!("bad --bind {v}: {e}"))?;
            }
            "--dist" => a.dist = it.next().ok_or("--dist needs a directory")?.into(),
            "-h" | "--help" => {
                return Err(
                    "usage: rustcars-web --enable [--bind 127.0.0.1:8080] [--dist dist]".into(),
                );
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(a)
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| bridge(socket, Box::new(EchoBackend)))
}

async fn bridge(mut socket: WebSocket, mut backend: Box<dyn DeviceBackend>) {
    while let Some(Ok(msg)) = socket.recv().await {
        let reply = match msg {
            Message::Text(t) => backend.handle(t.as_str()),
            Message::Close(_) => break,
            _ => continue,
        };
        if socket.send(Message::Text(reply.into())).await.is_err() {
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    if !args.enable {
        eprintln!(
            "rustcars-web is disabled. Pass --enable to start it (see README, Web interface)."
        );
        std::process::exit(1);
    }
    eprintln!(
        "WARNING: rustcars-web has NO authentication. Anyone who can reach {} can use the device bridge.",
        args.bind
    );
    if !args.bind.ip().is_loopback() {
        eprintln!(
            "WARNING: listening on a non-loopback address ({}); this exposes the bridge to your network.",
            args.bind
        );
    }
    if !args.dist.join("index.html").exists() {
        eprintln!(
            "note: {}/index.html not found; run `trunk build --release --features web` first.",
            args.dist.display()
        );
    }
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new(&args.dist));
    let listener = tokio::net::TcpListener::bind(args.bind)
        .await
        .expect("bind failed");
    eprintln!(
        "serving {} on http://{}  (WebSocket bridge: /ws)",
        args.dist.display(),
        args.bind
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .expect("server error");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn echo_backend() {
        let mut b = EchoBackend;
        assert_eq!(b.handle("ping"), "pong");
        assert_eq!(b.handle("name"), "echo-stub");
        assert_eq!(b.handle("x"), "echo: x");
    }
}

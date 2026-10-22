# 9. Desktop and the Optional Web Interface

## 9.1 Build modes

| Mode | Command | Notes |
|---|---|---|
| Desktop (default) | `cargo run` | Native eframe window, no web dependencies |
| Browser | `trunk serve` / `trunk build --release` | `--features web`, wasm32, served from `dist/` |
| Server bridge | `cargo run --features web-server --bin rustcars-web -- --enable` | Serves `dist/` plus a WebSocket at `/ws` |

`DemoApp` is the shared `eframe::App` used by both the desktop and web builds. Do the same in your own app: keep one `App` type and a thin, target-specific `main`.

## 9.2 Where devices live

The web UI runs in the **visitor's browser**. It can't open the server's USB devices or connect to a GQRX TCP port on the server. All device access goes through the server bridge:

- implement `DeviceBackend` (`name`, `handle(cmd) -> reply`) in `src/bin/rustcars-web.rs`;
- the browser sends text commands over `/ws`;
- the shipped `EchoBackend` only answers `ping` with `pong`, so it is a stub.

## 9.3 Security

- **Off by default:** both features are off, and the server also refuses to start without `--enable`.
- **Localhost first:** it binds `127.0.0.1` by default. `--bind 0.0.0.0:8080` exposes it on the network and prints a warning.
- **No authentication:** the bridge has none. Put an authenticating reverse proxy with TLS, or a VPN, in front of it before exposing it.
- **Static hosting:** GitHub Pages can host the static UI only, with no bridge. Pages for a private repository requires a paid plan.

## 9.4 Portability rules

- Use `ui.input(|i| i.time)` or the `web-time` crate instead of `std::time::Instant`.
- No blocking I/O in the UI thread. Talk to the bridge asynchronously.
- `apply_theme` forces the dark theme, because browsers otherwise apply the user's light/dark preference.

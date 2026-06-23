# AETHER TUNNEL KNOWLEDGE

## OVERVIEW

`apps/aether-tunnel` is a standalone outbound proxy node. It connects back to an Aether server over WebSocket tunnel connections and does not need to expose an inbound public port.

## WHERE TO LOOK

| Task | Location | Notes |
|---|---|---|
| Binary entry | `src/main.rs` | CLI, config loading, setup/status/logs/service commands |
| Tunnel runtime | `src/runtime` and related modules | WebSocket connection pool and stream forwarding |
| Install docs | `README.md` | User-facing install, config, and runtime behavior |
| Unix installer | `install.sh` | Downloads `tunnel-v*` artifacts and appends config |
| Windows installer | `install.ps1` | Windows install/update path |
| Config examples | `*.toml` files if present | Multiple `[[servers]]` entries are expected |

## CONVENTIONS

The tunnel release line uses `tunnel-v*` tags and must not rely on the repository generic latest release. Installer logic should keep filtering tunnel artifacts explicitly.

Configuration precedence is CLI first, then `AETHER_TUNNEL_*` environment variables, then `aether-tunnel.toml`.

Installer paths append a `[[servers]]` entry and skip duplicates by `aether_url + node_name`; they should not overwrite an existing config file.

IPv4-only and IPv6-only are mutually exclusive connection selection modes. They affect the WebSocket tunnel connection to Aether, not provider upstream routing.

## ANTI-PATTERNS

Do not make the tunnel listen publicly by default. The product model is outbound tunnel registration from the node to Aether.

Do not use `/releases/latest` when installing tunnel artifacts. The generic latest release may not be a tunnel release.

Do not conflate `aether_outbound_proxy_url` with `upstream_proxy_url`; the former is for Aether API and tunnel WebSocket connections, the latter is for provider upstream traffic.

## COMMANDS

```bash
cargo run -p aether-tunnel -- setup
cargo run -p aether-tunnel -- status
cargo test -p aether-tunnel
cargo clippy -p aether-tunnel --all-targets -- -D warnings
```

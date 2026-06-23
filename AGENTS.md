# PROJECT KNOWLEDGE BASE

Generated: 2026-06-08T18:41:13Z
Commit: 4933ae90
Branch: main

## OVERVIEW

Aether is a self-hosted AI infrastructure platform. The backend is a Rust workspace with gateway and tunnel applications, and the management UI is a Vue 3 + Vite single-page app under `frontend`.

## STRUCTURE

```text
/root/Aether/
|-- apps/aether-gateway/     # main HTTP gateway, admin APIs, routing, runtime orchestration
|-- apps/aether-tunnel/      # standalone outbound tunnel node and installer
|-- crates/                  # shared Rust crates for data, routing, providers, runtime, billing
|-- frontend/                # Vue management console and public guide pages
|-- docs/                    # architecture and API docs
|-- Cargo.toml               # workspace membership and shared Rust dependencies
|-- Makefile                 # local dev, migration, and backfill entrypoints
|-- install.sh               # host install path for compose/single-node/system service modes
|-- update.sh                # Docker Compose update path
`-- docker-compose*.yml      # standard, single-node, local, and release-local deployments
```

## WHERE TO LOOK

| Task | Location | Notes |
|---|---|---|
| Start the service | `apps/aether-gateway/src/main.rs` | CLI flags, runtime setup, DB export/import, app state assembly |
| Build HTTP routes | `apps/aether-gateway/src/router.rs` | Router composition, static frontend attachment, TCP serving |
| Gateway state | `apps/aether-gateway/src/state` | `AppState` and runtime/config wiring |
| Admin and public APIs | `apps/aether-gateway/src/handlers` | Split by `admin`, `internal`, `proxy`, `public`, `shared` |
| Data access | `crates/aether-data` | Concrete SQL drivers, repositories, migrations, export/import |
| Logical schema | `crates/aether-data-schema` and `crates/aether-data/schema` | Generator plus logical TOML and generated SQL outputs |
| Frontend app | `frontend/src/main.ts`, `frontend/src/router/index.ts` | Vue bootstrap and route map |
| Frontend API/auth | `frontend/src/api/client.ts`, `frontend/src/stores/auth.ts` | Axios client, token refresh, cross-tab auth sync |
| Tunnel node | `apps/aether-tunnel` | Standalone WebSocket tunnel binary, setup, installers |
| Local dev | `Makefile`, `frontend/package.json` | `make dev` starts backend plus Vite |
| CI expectations | `.github/workflows/rust-ci.yml` | Rust fmt, clippy, nextest, and DB smoke tests |

## CODE MAP

| Symbol | Type | Location | Role |
|---|---|---|---|
| `main` | binary entry | `apps/aether-gateway/src/main.rs` | Gateway CLI and service bootstrap |
| `build_router_with_state` | function export | `apps/aether-gateway/src/lib.rs`, `router.rs` | Builds the main Axum router around `AppState` |
| `serve_tcp` | function export | `apps/aether-gateway/src/lib.rs`, `router.rs` | Starts gateway TCP listener |
| `AppState` | exported state | `apps/aether-gateway/src/state` | Shared gateway dependency graph |
| `ApiClient` | frontend class | `frontend/src/api/client.ts` | Central Axios client, auth headers, refresh handling |
| `routes` | frontend route table | `frontend/src/router/index.ts` | Public, user dashboard, and admin route structure |

LSP was attempted during generation but was not usable in this environment: Rust lacked `rust-analyzer`, and TypeScript lacked `typescript-language-server`. Symbol facts above come from direct file reads and `rg`-style inspection.

## CONVENTIONS

Use the workspace root as the default command directory for Rust work. `Cargo.toml` owns workspace membership and shared dependency versions; package-specific `Cargo.toml` files should not drift dependency versions without a reason.

Use `Makefile` for the common local surfaces. `make dev` runs gateway and frontend together, `make dev-backend` runs only `cargo run -p aether-gateway`, `make dev-frontend` runs Vite, and `make migration` / `make backfill` pass `--migrate` / `--apply-backfills` through the gateway binary.

The root instructions include `/root/.codex/RTK.md`, which asks for shell commands to be prefixed with `rtk`. During this init run, `rtk` was not installed; if it is still missing, say so and use normal commands rather than blocking the task.

Docker Compose deployments use Docker update semantics. The admin version panel may detect releases, but Docker users update with `./update.sh`; backend self-update is for systemd, launchd, or binary installs.

## ANTI-PATTERNS

Do not treat `frontend/`, `apps/aether-gateway/`, and `crates/aether-data/` as generic directories. Each has a child `AGENTS.md` because its conventions are materially different from the repo root.

Do not edit generated schema output as the source of truth. Start schema work in `crates/aether-data/schema/logical/*.toml` unless the child data guidance says a driver fragment is still intentionally hand-maintained.

Do not write new values into usage legacy columns described as deprecated compatibility mirrors. The authoritative ownership has moved to settlement, billing fact, audit, or body blob tables.

Do not run broad workspace verification when a scoped command is enough for the touched surface. Gateway, data, frontend, and tunnel have narrower commands listed in their scoped guidance.

## COMMANDS

```bash
make dev
make dev-backend
make dev-frontend
make migration
make backfill
cargo fmt --all --check
cargo clippy -p aether-gateway --all-targets -- -D warnings
cargo clippy -p aether-data --all-targets -- -D warnings
cargo nextest run -p aether-gateway
cargo nextest run -p aether-data
cd frontend && npm run build:with-typecheck
cd frontend && npm run test:run
```

## NOTES

The repository is large: discovery found roughly 2.5k non-target files and more than 120k core source/config lines. Prefer scoped reads and scoped verification.

Existing memory for prior Aether work mentions complete backup coverage, Docker online-update boundaries, data capacity work, and fork-based PR publishing. Treat those as historical hints only; re-check current code before relying on them.

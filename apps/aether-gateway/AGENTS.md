# AETHER GATEWAY KNOWLEDGE

## OVERVIEW

`apps/aether-gateway` is the main Aether service. It owns the HTTP gateway, admin and public APIs, runtime orchestration, background tasks, usage capture, wallet/runtime hooks, embedded tunnel control, and frontend static serving.

## STRUCTURE

```text
apps/aether-gateway/
|-- src/main.rs          # binary bootstrap, CLI flags, runtime config, import/export
|-- src/lib.rs           # module boundary and public exports
|-- src/router.rs        # Axum router composition and serving
|-- src/handlers/        # request handlers by API surface
|-- src/state/           # AppState and runtime dependency wiring
|-- src/control/         # control-plane behavior and tests
|-- src/execution_runtime/ # local execution runtime and stream/sync paths
|-- src/ai_serving/      # AI serving planning, adaptation, finalization
|-- src/tests/           # large gateway integration test tree
`-- src/test_support.rs  # shared test helpers when compiled for tests
```

## WHERE TO LOOK

| Task | Location | Notes |
|---|---|---|
| Gateway startup | `src/main.rs` | CLI, deployment topology, runtime backend, DB migration/export/import |
| Public exports | `src/lib.rs` | `AppState`, `GatewayDataConfig`, `build_router`, `serve_tcp` |
| Route graph | `src/router.rs` | Admin/public/proxy route assembly and static frontend |
| API handlers | `src/handlers/{admin,internal,proxy,public,shared}` | Keep new endpoints in the matching API surface |
| State wiring | `src/state` | Avoid bypassing `AppState` composition |
| Runtime execution | `src/execution_runtime` | Stream/sync execution contracts and local runtime serving |
| AI serving decisions | `src/ai_serving` | Candidate planning, adaptation, and response finalization |
| Gateway tests | `src/tests` and focused module tests | Tests are grouped by business domain, not by generic unit/integration labels |

## CONVENTIONS

Keep `src/main.rs` as process orchestration. Business logic belongs in modules under `src/` or shared crates, then is wired through `AppState` and router builders.

`src/lib.rs` is intentionally the gateway boundary. Add exports only when another crate, binary path, or test surface needs them.

Handlers are organized by API surface. Prefer adding a focused module under the existing `handlers` branch over expanding a large sibling file.

Gateway tests are heavy. When changing a narrow surface, prefer a filtered package test first, then broaden only when the touched behavior crosses handler, state, and runtime boundaries.

## ANTI-PATTERNS

Do not add data repository behavior directly to gateway state modules. Data access belongs in `crates/aether-data` or contracts, then is wired into gateway state.

Do not add frontend route or auth behavior in gateway code unless it is an API contract or static serving concern.

Do not use Docker self-update assumptions here. Docker deployments update through root `update.sh`; gateway self-update behavior is for service or binary installs.

## COMMANDS

```bash
cargo run -p aether-gateway
cargo run -p aether-gateway -- --migrate
cargo run -p aether-gateway -- --apply-backfills
cargo clippy -p aether-gateway --all-targets -- -D warnings
cargo nextest run -p aether-gateway
```

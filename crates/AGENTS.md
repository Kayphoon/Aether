# RUST CRATES KNOWLEDGE

## OVERVIEW

`crates/` contains the shared Rust workspace libraries used by gateway and tunnel apps. The crates split contracts, data access, runtime state, routing, provider transport, AI serving, billing, wallet, OAuth, cache, scheduler, dispatch, task runtime, and test support.

## WHERE TO LOOK

| Task | Location | Notes |
|---|---|---|
| Shared DTOs and errors | `aether-data-contracts`, `aether-contracts` | Put cross-crate shapes here before app-specific implementations |
| Concrete persistence | `aether-data` | SQL drivers, repositories, lifecycle, export/import |
| Logical schema generator | `aether-data-schema` | Parses logical TOML and emits driver DDL |
| Provider formats | `aether-ai-formats` | API format and protocol conversion |
| Serving decisions | `aether-ai-serving` | Candidate and execution planning support |
| Provider upstreams | `aether-provider-transport`, `aether-provider-pool` | Transport rules, provider adapters, pool/service behavior |
| Routing logic | `aether-routing-core` | Policies, conditions, mutations, ranking, validation |
| Runtime coordination | `aether-runtime`, `aether-runtime-state`, `aether-task-runtime` | Logging/runtime setup, memory/Redis state, async task runtime |
| Usage and billing | `aether-usage-runtime`, `aether-billing`, `aether-wallet` | Usage aggregation, billing rules, wallet records |
| Tests and fixtures | `aether-testkit` | Shared helpers and benchmark-style bins |

## CONVENTIONS

Workspace dependency versions are centralized in the root `Cargo.toml`. Prefer adding dependency versions there when multiple crates will use them.

Use contracts crates for shapes that another crate must compile against. Keep app-only state or request-specific helpers inside the app that owns them.

Provider and routing crates should remain app-agnostic. Gateway can compose them, but core policy, transport, format conversion, and ranking logic should not depend on gateway handlers.

Use `thiserror`, typed errors, and explicit domain records over stringly-typed cross-crate contracts.

## ANTI-PATTERNS

Do not move concrete SQL into contracts crates. Contracts describe behavior and records; `aether-data` owns the driver implementations.

Do not add gateway-specific environment parsing or CLI behavior into shared crates unless the crate already owns that runtime concern.

Do not duplicate provider-specific normalization between `aether-ai-formats`, `aether-provider-transport`, and gateway code. First identify which layer owns the format, transport, or orchestration behavior.

## COMMANDS

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude aether-gateway --exclude aether-data --all-targets -- -D warnings
cargo nextest run --workspace --exclude aether-gateway --exclude aether-data
cargo test -p aether-testkit
```

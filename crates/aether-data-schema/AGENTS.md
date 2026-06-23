# AETHER DATA SCHEMA KNOWLEDGE

## OVERVIEW

`crates/aether-data-schema` is the logical schema generator for `crates/aether-data`. It parses logical TOML definitions, validates metadata, emits driver-specific DDL, and checks generated artifacts for drift.

## WHERE TO LOOK

| Task | Location | Notes |
|---|---|---|
| Generator entry | `src/main.rs` or bin target | CLI handling for `check`, `generate`, `print` |
| Logical input | `../aether-data/schema/logical/*.toml` | Human-maintained source for table structure |
| Generated output | `../aether-data/schema/generated/{postgres,mysql,sqlite}/baseline` | Machine-written SQL plus manifests |
| Driver fragments | `../aether-data/schema/drivers/{postgres,mysql,sqlite}` | Hand-maintained fragments used by compose flow |
| Runtime migrations | `../aether-data/migrations/{postgres,mysql,sqlite}` | Owned by `aether-data`, not by this crate |

## CONVENTIONS

Generated files and manifests are expected to be deterministic. A stale README, missing generated file, extra generated file, or stale manifest is a check failure.

The generator owns schema shape validation. Runtime migration behavior, export/import, backfills, and repository implementations stay in `aether-data`.

Current logical coverage includes identity, provider catalog, auth config, proxy nodes, wallet/payment/refund/redeem-code/settlement tables, usage capture, and portable stats aggregation tables.

## ANTI-PATTERNS

Do not patch generated SQL to make a check pass. Change `schema/logical/*.toml` or generator logic, then regenerate.

Do not add runtime migration semantics to this crate. Keep migration execution and `sqlx::migrate!` paths in `aether-data`.

Do not assume Postgres-only historical migrations are covered by logical schema unless their shape has been normalized or intentionally represented as an override.

## COMMANDS

```bash
cargo run -p aether-data-schema --bin aether-schema -- check
cargo run -p aether-data-schema --bin aether-schema -- generate
cargo run -p aether-data-schema --bin aether-schema -- print --driver postgres
cargo run -p aether-data-schema --bin aether-schema -- print --driver mysql
cargo run -p aether-data-schema --bin aether-schema -- print --driver sqlite
```

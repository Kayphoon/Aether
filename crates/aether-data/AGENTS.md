# AETHER DATA KNOWLEDGE

## OVERVIEW

`crates/aether-data` is the concrete runtime data-access crate. It owns SQL drivers, repository implementations, migration/backfill/export workflows, and backend composition for app-facing read/write/worker/lock/lease handles.

## STRUCTURE

```text
crates/aether-data/
|-- src/database.rs              # logical SQL driver selection
|-- src/config.rs                # data-layer config and wiring
|-- src/driver/{postgres,mysql,sqlite}
|-- src/repository/<domain>/     # repository traits/types plus driver implementations
|-- src/backend/                 # composition root and backend-owned workflows
|-- src/lifecycle/               # migrate, backfill, export/import
|-- migrations/{postgres,mysql,sqlite}
|-- schema/logical/              # source of truth for logical table definitions
|-- schema/drivers/              # hand-maintained driver fragments still in use
|-- schema/generated/            # generated audit output
`-- backfills/{postgres,mysql,sqlite}
```

## WHERE TO LOOK

| Task | Location | Notes |
|---|---|---|
| Add a repository | `src/repository/<domain>` | Use explicit `postgres.rs`, `mysql.rs`, `sqlite.rs`, `memory.rs` files |
| Wire a repository | `src/backend/read.rs`, `src/backend/write.rs`, focused backend modules | Wire only after selected drivers exist |
| Database lifecycle | `src/lifecycle/{migrate,backfill,export}.rs` | Migrations, backfills, import/export |
| Full export/backup data | `src/backend/system.rs`, lifecycle export code | Verify aggregate snapshot and historical records, not only top-level counts |
| Schema source | `schema/logical/*.toml` | Start new table structure here |
| Generated SQL | `schema/generated/**` | Machine-written, checked in for drift review only |
| Existing executable migrations | `migrations/{postgres,mysql,sqlite}` | `sqlx::migrate!` embeds these paths |

## CONVENTIONS

Think in five layers: contracts in `aether-data-contracts`, driver primitives, repository implementations, backend composition, and lifecycle/schema maintenance.

The physical SQL is driver-specific where syntax, JSON, timestamps, indexes, locking, or upsert semantics differ. The portable contract is the Rust behavior and shape.

New table structure should start in `schema/logical/*.toml`, then run the schema compose/generate/check flow. Driver fragments under `schema/drivers` are for executable fragments not yet promoted to generated output.

`jsonb` is Postgres-only. MySQL and SQLite migrations must not contain it.

## ANTI-PATTERNS

Do not add domain queries to low-level pool modules in `src/driver`.

Do not put driver selection logic inside individual repository implementations.

Do not edit `schema/generated/**` by hand. Edit logical TOML or the appropriate driver fragment, then regenerate or compose.

Do not write new data to deprecated usage compatibility mirrors. Use settlement snapshots, billing facts, HTTP audits, or body blob owners as indicated by migration comments.

## COMMANDS

```bash
bash crates/aether-data/schema/compose_schema.sh generate
bash crates/aether-data/schema/compose_schema.sh compose
bash crates/aether-data/schema/compose_schema.sh check
cargo run -p aether-data-schema --bin aether-schema -- check
cargo clippy -p aether-data --all-targets -- -D warnings
cargo nextest run -p aether-data
cargo test -p aether-data sqlite --lib
```

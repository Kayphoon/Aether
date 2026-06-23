# Complete Backup Record Code Review

codeQualityStatus: BLOCK
recommendation: REQUEST_CHANGES
reportPath: .omo/evidence/complete-backup-record-code-review.md

Skill-perspective check ran. I loaded and applied `omo:remove-ai-slops` and `omo:programming` before judging test relevance and maintainability. The diff violates the `programming` and `remove-ai-slops` perspectives because it adds another sizable feature into an already oversized production handler instead of extracting the new backup-record surface into a focused module.

## CRITICAL

None.

## HIGH

`apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs:794` adds the complete-backup run recorder and `apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs:912` adds the backup-runs list builder inside `system_routes.rs`, which now measures 1275 pure LOC. The local gateway guidance says to prefer focused modules under the existing handler branch rather than expanding large sibling files, and the loaded `programming`/`remove-ai-slops` criteria treat files over 250 pure LOC as a maintainability defect. This change adds a new API surface, persistence write path, and response mapper to a file already far past that boundary, making the feature harder to review and increasing regression risk in unrelated system routes. This is a blocker for the quality gate.

## MEDIUM

`apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs:960` and `apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs:961` return stored `payload_json` and `result_json` verbatim from `background_task_runs`. The current writer stores only `{"scope":"complete_backup"}` and `{"exported_bytes": ...}`, and the integration evidence checks that `config_data` and `user_data` are absent. Still, this read endpoint is meant to be the complete-backup record surface; a small response whitelist would better enforce the "no backup payload/secret leakage" boundary even if future code accidentally writes richer JSON under the same task key.

`apps/aether-gateway/src/tests/control/admin/system.rs:1250` covers successful export recording and `apps/aether-gateway/src/tests/control/admin/system.rs:1147` covers exact task-key filtering, including a near-match. There is no regression test for the failure branch at `apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs:841`, so failed export recording is source-inspected rather than test-locked. The code does update the run to `failed`, but this is an important user-facing status boundary.

## LOW

`frontend/src/views/admin/system-settings/composables/useBackupRecords.ts:76` always requests the first default page, then truncates `response.items` to 10. This is acceptable for a "recent backup" section, but the API shape supports pagination and the UI does not expose older pages. That leaves a product limitation rather than a correctness defect.

Browser screenshot evidence was not available because the environment had no Chrome/Chromium/Playwright package. I inspected the provided Vitest render evidence instead; the CJK labels, empty state, error state, and status union mapping are covered at the component/composable level.

## Positive Findings

The route is classified as `admin_proxy` / `system_manage` / `backup_runs` with `admin:system`, so I did not find an auth or classification bypass in the scoped route. The new repository filter uses exact `task_key` equality across memory, Postgres, MySQL, and SQLite; the test includes `admin.system.data_export.extra` as a near-match and verifies it is excluded. The success writer stores only a summary result (`exported_bytes`) rather than the exported backup payload, and the frontend status union matches the backend `BackgroundTaskStatus` database values.

## Blockers

Refactor the new backup-record backend logic out of `apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs` into a focused module under the existing system handler tree, leaving `system_routes.rs` as route dispatch glue. After that, rerun the existing scoped backend and frontend checks, and ideally add a focused failure-recording regression test for the export error path.

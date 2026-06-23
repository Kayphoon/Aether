# Admin Complete Backup Runs

## TL;DR
> Summary:      Add admin-visible history records for every complete backup export by reusing `background_task_runs`, recording synchronous export success/failure, exposing a filtered `/api/admin/system/backup-runs` endpoint, and showing a compact Chinese history panel in System Settings.
> Deliverables:
> - Backend task key, run recording, filtered backup-runs endpoint, route classification, and tests.
> - Frontend API types/client and System Settings data-management UI showing latest status, count, and recent rows.
> - Manual QA evidence for HTTP export/listing and real Chrome admin-page rendering.
> Effort:       Medium
> Risk:         Medium - the main risk is exact filtering with the existing substring-only background task query; the executor must prove only `admin.system.data_export` rows appear.

## Scope
### Must have
- Reuse the existing `background_task_runs` persistence table and repository surface. Do not add a migration or a new backup-history table.
- Add a stable task key constant `TASK_KEY_ADMIN_SYSTEM_DATA_EXPORT` with value `admin.system.data_export` in `apps/aether-gateway/src/task_runtime/mod.rs`.
- Record one background task run for every `GET /api/admin/system/data/export`, including successful and failed export attempts.
- Preserve the existing export response body and audit behavior for `GET /api/admin/system/data/export`.
- Add `GET /api/admin/system/backup-runs` as an admin-system endpoint that returns only complete-backup export run records for `admin.system.data_export`.
- Keep the route under the existing `admin:system` permission boundary, matching `/api/admin/system/data/export` and `/api/admin/system/cleanup/runs`.
- Add Chinese UI in the System Settings data-management section showing latest backup status, total count, and recent rows.
- Keep Vue 3/Vite patterns; do not introduce React, a new frontend framework, or a global state store for this narrow history panel.
- Add agent-executable backend and frontend tests, plus real-surface HTTP and browser QA evidence.

### Must NOT have (guardrails, anti-slop, scope boundaries)
- Must not change complete backup payload shape, import/export document versions, or encryption behavior.
- Must not log or persist backup payload contents, provider secrets, API keys, passwords, OAuth secrets, user PII, or raw exported JSON in `background_task_runs.payload_json`, `result_json`, tests, logs, or evidence.
- Must not record standalone config export or users export as complete-backup runs.
- Must not expose generic `/api/admin/tasks` internals directly in the System Settings UI.
- Must not rely on human manual testing; every check must be agent-executed and must capture evidence under `.omo/evidence/`.
- Must not touch unrelated dirty files already present in provider quota/provider type work.
- Must not run broad workspace verification when scoped commands cover the touched surfaces.

## Verification strategy
> Zero human intervention - all verification is agent-executed.
- Test decision: TDD + Rust `cargo test`/`cargo nextest` targets and frontend Vitest/typecheck.
- QA policy: every task has agent-executed scenarios.
- Evidence: `.omo/evidence/task-<N>-<slug>.<ext>`

## Execution strategy
### Parallel execution waves
> Target 5-8 tasks per wave. <3 per wave (except final) = under-splitting.
> Extract shared dependencies as Wave-1 tasks to maximize parallelism.

Wave 1 (no dependencies):
- Task 1: Add backend task key and route classification for backup runs.
- Task 2: Add test-only background task repository wiring if no existing helper can seed/read runs.
- Task 3: Add frontend backup-run API types and client method.
- Task 4: Add frontend unit tests for backup-run API/client formatting seams.

Wave 2 (after Wave 1):
- Task 5: Record complete-backup export runs around `GET /api/admin/system/data/export`.
- Task 6: Add filtered `GET /api/admin/system/backup-runs` response builder and route handling.
- Task 7: Add System Settings data-management UI for backup runs.

Wave 3 (after Wave 2):
- Task 8: Add backend integration tests proving export recording and filtered listing.
- Task 9: Add frontend component/composable tests for loading, empty, success, and failure states.
- Task 10: Run scoped verification and real-surface QA evidence capture.

Critical path: Task 1 -> Task 5 -> Task 6 -> Task 8 -> Task 10

### Dependency matrix
| Task | Depends on | Blocks | Can parallelize with |
|------|------------|--------|----------------------|
| 1    | none       | 5, 6, 8 | 2, 3, 4             |
| 2    | none       | 5, 6, 8 | 1, 3, 4             |
| 3    | none       | 7, 9    | 1, 2, 4             |
| 4    | none       | 9       | 1, 2, 3             |
| 5    | 1, 2       | 8, 10   | 6, 7                |
| 6    | 1, 2       | 8, 10   | 5, 7                |
| 7    | 3          | 9, 10   | 5, 6                |
| 8    | 1, 2, 5, 6 | 10      | 9                   |
| 9    | 3, 4, 7    | 10      | 8                   |
| 10   | 5, 6, 7, 8, 9 | F1-F4 | none              |

## Todos
> Implementation + Test = ONE task. Never separate.
> Every task MUST have: References + Acceptance Criteria + QA Scenarios + Commit.

- [ ] 1. Register the complete-backup export task key and route classification

  What to do: Add `pub(crate) const TASK_KEY_ADMIN_SYSTEM_DATA_EXPORT: &str = "admin.system.data_export";` to `apps/aether-gateway/src/task_runtime/mod.rs` near the other admin task keys. Add a `TaskDefinition::new` entry with `TaskKind::OnDemand`, trigger `"manual"`, singleton `false`, persist history `true`, and `RETRY_ONCE`. Add `/api/admin/system/backup-runs` classification in `apps/aether-gateway/src/control/route/admin/system_families.rs` immediately beside `data_export`/`cleanup_runs`, with route kind `backup_runs`, route family `system_manage`, signature `admin:system`, and execution-runtime candidate `false`. Add a test in `apps/aether-gateway/src/control/tests/admin_core.rs` following the existing `classifies_admin_system_cleanup_runs_as_admin_proxy_route` style.
  Must NOT do: Do not change permissions for existing system routes. Do not classify this under generic task management. Do not change the existing `data_export` route kind.

  Parallelization: Can parallel: YES | Wave 1 | Blocks: [5, 6, 8] | Blocked by: []

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `apps/aether-gateway/src/task_runtime/mod.rs:18` - existing admin task key constants.
  - Pattern:  `apps/aether-gateway/src/task_runtime/mod.rs:50` - existing `TASK_DEFINITIONS` entries and fields.
  - Pattern:  `apps/aether-gateway/src/control/route/admin/system_families.rs:125` - current `GET /api/admin/system/data/export` classification.
  - Pattern:  `apps/aether-gateway/src/control/route/admin/system_families.rs:183` - current `GET /api/admin/system/cleanup/runs` classification.
  - Test:     `apps/aether-gateway/src/control/tests/admin_core.rs:160` - data export classification test.
  - Test:     `apps/aether-gateway/src/control/tests/admin_core.rs:327` - cleanup runs classification test.
  - External: `https://docs.rs/axum/latest/axum/index.html` - Axum JSON routes compose around handlers and route classification remains app-owned here.

  Acceptance criteria (agent-executable only):
  - [ ] `rtk cargo test -p aether-gateway classifies_admin_system_backup_runs_as_admin_proxy_route -- --exact` exits 0.
  - [ ] `rtk cargo test -p aether-gateway task_definitions -- --nocapture` or an equivalent focused test/assertion proves `admin.system.data_export` is registered with `on_demand`, `manual`, `persist_history=true`.

  QA scenarios (MANDATORY - task incomplete without these):
  > Name the exact tool AND its exact invocation - not "verify it works". Browser use: use Chrome to drive the page; if Chrome is not available, download and use agent-browser (https://github.com/vercel-labs/agent-browser). Computer use: OS-level GUI automation for a non-browser desktop app.
  ```
  Scenario: route classification accepts backup runs
    Tool:     bash
    Steps:    rtk cargo test -p aether-gateway classifies_admin_system_backup_runs_as_admin_proxy_route -- --exact 2>&1 | tee .omo/evidence/task-1-route-classification.txt
    Expected: command exits 0 and output includes classifies_admin_system_backup_runs_as_admin_proxy_route
    Evidence: .omo/evidence/task-1-route-classification.txt

  Scenario: wrong method is not added as a write route
    Tool:     bash
    Steps:    rtk cargo test -p aether-gateway classifies_admin_system_backup_runs_rejects_post_method -- --exact 2>&1 | tee .omo/evidence/task-1-route-classification-error.txt
    Expected: command exits 0 and test asserts POST /api/admin/system/backup-runs does not classify as backup_runs
    Evidence: .omo/evidence/task-1-route-classification-error.txt
  ```

  Commit: YES | Message: `feat(admin-system): register backup export run route` | Files: [apps/aether-gateway/src/task_runtime/mod.rs, apps/aether-gateway/src/control/route/admin/system_families.rs, apps/aether-gateway/src/control/tests/admin_core.rs]

- [ ] 2. Add test-only background task repository wiring

  What to do: If no existing helper can attach both `BackgroundTaskReadRepository` and `BackgroundTaskWriteRepository`, add one narrowly scoped test helper to `apps/aether-gateway/src/data/state/testing/mod.rs`, such as `with_background_task_repository_for_tests<T>(repository: Arc<T>) -> Self where T: BackgroundTaskReadRepository + BackgroundTaskWriteRepository + 'static`. Wire both `background_task_reader` and `background_task_writer` to the same repository. Add a small unit test or use it in Task 8 RED tests to prove a seeded `InMemoryBackgroundTaskRepository` can be read through `GatewayDataState::list_background_task_runs`.
  Must NOT do: Do not modify production constructors except through existing test-only `#[cfg(test)]` surfaces. Do not add a database migration. Do not introduce concrete SQL in gateway state.

  Parallelization: Can parallel: YES | Wave 1 | Blocks: [5, 6, 8] | Blocked by: []

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `apps/aether-gateway/src/data/state/testing/mod.rs:150` - existing reader-only test helper shape.
  - Pattern:  `apps/aether-gateway/src/data/state/testing/mod.rs:286` - existing repository helper shape that wires read/write surfaces.
  - Pattern:  `apps/aether-gateway/src/data/state/mod.rs:84` - background task repository trait imports already exist in gateway data state.
  - API/Type: `crates/aether-data-contracts/src/repository/background_tasks/types.rs:241` - read/write repository traits.
  - API/Type: `crates/aether-data/src/repository/background_tasks/memory.rs:21` - in-memory repository suitable for tests.
  - Test:     `apps/aether-gateway/src/data/state/runtime.rs:1989` - data state list method returns an empty page when no reader exists, so tests need explicit wiring.

  Acceptance criteria (agent-executable only):
  - [ ] `rtk cargo test -p aether-gateway background_task_repository_for_tests -- --nocapture` exits 0 if a new helper-specific test is added, or Task 8’s RED/GREEN tests exercise the helper and pass.
  - [ ] `rtk cargo test -p aether-gateway gateway_handles_admin_system_data_export_records_backup_run -- --exact` can construct a gateway with an in-memory background task repository without a real database.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: helper exposes seeded run through GatewayDataState
    Tool:     bash
    Steps:    rtk cargo test -p aether-gateway background_task_repository_for_tests -- --nocapture 2>&1 | tee .omo/evidence/task-2-background-task-test-helper.txt
    Expected: command exits 0 and the test reads the seeded run id through GatewayDataState
    Evidence: .omo/evidence/task-2-background-task-test-helper.txt

  Scenario: disabled data state still returns empty background task page
    Tool:     bash
    Steps:    rtk cargo test -p aether-gateway disabled_data_state_lists_no_background_task_runs -- --exact 2>&1 | tee .omo/evidence/task-2-background-task-test-helper-error.txt
    Expected: command exits 0 and the test asserts total=0/items=[] without panicking
    Evidence: .omo/evidence/task-2-background-task-test-helper-error.txt
  ```

  Commit: YES | Message: `test(gateway): add background task repository fixture` | Files: [apps/aether-gateway/src/data/state/testing/mod.rs, apps/aether-gateway/src/data/state/runtime.rs]

- [ ] 3. Add frontend backup-run API contract and client method

  What to do: In `frontend/src/api/admin.ts`, add explicit TypeScript interfaces for backup export run rows and response, for example `BackupRunRecord` and `BackupRunListResponse`. Fields must match the backend endpoint from Task 6: `id`, `status`, `message`, `created_by`, `created_at`, `started_at`, `finished_at`, `updated_at`, `duration_seconds`, `error_message`, `result`, and `total`. Add `adminApi.getBackupRuns()` using `apiClient.get<BackupRunListResponse>('/api/admin/system/backup-runs')`.
  Must NOT do: Do not reuse `AsyncTaskItem` directly in the System Settings API surface. Do not add `any`; use `unknown` or typed records for `result`. Do not bypass `apiClient`.

  Parallelization: Can parallel: YES | Wave 1 | Blocks: [7, 9] | Blocked by: []

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `frontend/src/api/admin.ts:360` - `CleanupRunRecord` local admin API type pattern.
  - Pattern:  `frontend/src/api/admin.ts:1464` - `getCleanupRuns()` client method.
  - Pattern:  `frontend/src/api/admin.ts:1031` - complete backup export client method.
  - API/Type: `frontend/src/api/async-tasks.ts:28` - generic async-task item fields if a field naming comparison is needed, but do not expose this type directly.
  - External: `https://vuejs.org/guide/typescript/composition-api` - Vue + TypeScript props/state should remain typed at component boundaries.

  Acceptance criteria (agent-executable only):
  - [ ] `cd frontend && rtk npm run type-check` exits 0 after the new types and method are added.
  - [ ] A Vitest test or API mock assertion proves `adminApi.getBackupRuns()` calls `/api/admin/system/backup-runs` and returns typed `items`/`total`.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: API client maps backup-runs endpoint
    Tool:     bash
    Steps:    cd frontend && rtk npm run test:run -- src/api/__tests__/admin-backup-runs.spec.ts 2>&1 | tee ../.omo/evidence/task-3-admin-api-backup-runs.txt
    Expected: command exits 0 and test asserts GET /api/admin/system/backup-runs
    Evidence: .omo/evidence/task-3-admin-api-backup-runs.txt

  Scenario: type checker rejects missing required response fields
    Tool:     bash
    Steps:    cd frontend && rtk npm run type-check 2>&1 | tee ../.omo/evidence/task-3-admin-api-backup-runs-error.txt
    Expected: command exits 0 with no TypeScript errors; no `any`, `@ts-ignore`, or `@ts-expect-error` is introduced
    Evidence: .omo/evidence/task-3-admin-api-backup-runs-error.txt
  ```

  Commit: YES | Message: `feat(frontend-api): add backup run history client` | Files: [frontend/src/api/admin.ts, frontend/src/api/__tests__/admin-backup-runs.spec.ts]

- [ ] 4. Add frontend unit-test scaffolding for data-management backup history

  What to do: Add a focused test file for the data-management history behavior, either `frontend/src/views/admin/system-settings/__tests__/DataManagementSection.spec.ts` for component rendering or a small extracted formatter test if the executor chooses to extract pure formatting helpers. Test Chinese status labels, empty state text, and failure text without mounting the whole System Settings page. Mock `adminApi.getBackupRuns` and toast behavior using the local Vitest patterns.
  Must NOT do: Do not add broad snapshot tests. Do not mount the entire app/router for a local table. Do not assert on styling implementation details beyond stable text and row presence.

  Parallelization: Can parallel: YES | Wave 1 | Blocks: [9] | Blocked by: []

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `frontend/src/views/admin/system-settings/__tests__/useConfigExportImport.spec.ts:1` - local Vitest mocking style.
  - Pattern:  `frontend/src/views/admin/system-settings/CleanupPolicySection.vue:486` - Chinese label formatter precedent.
  - Pattern:  `frontend/src/views/admin/system-settings/CleanupPolicySection.vue:502` - status label precedent.
  - Pattern:  `frontend/src/views/admin/system-settings/CleanupPolicySection.vue:514` - timestamp formatter precedent.
  - External: `https://vuejs.org/guide/essentials/reactivity-fundamentals.html` - `ref`-backed reactive state is the expected Composition API style.

  Acceptance criteria (agent-executable only):
  - [ ] `cd frontend && rtk npm run test:run -- src/views/admin/system-settings/__tests__/DataManagementSection.spec.ts` exits 0 once Task 7 is implemented.
  - [ ] Tests cover at least these text outcomes: `暂无备份记录`, `最近备份记录`, `成功`, `失败`, and the backend error fallback message.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: backup history labels render in Chinese
    Tool:     bash
    Steps:    cd frontend && rtk npm run test:run -- src/views/admin/system-settings/__tests__/DataManagementSection.spec.ts 2>&1 | tee ../.omo/evidence/task-4-data-management-history-tests.txt
    Expected: command exits 0 and tests assert Chinese labels for empty, success, and failed states
    Evidence: .omo/evidence/task-4-data-management-history-tests.txt

  Scenario: API failure renders graceful fallback
    Tool:     bash
    Steps:    cd frontend && rtk npm run test:run -- src/views/admin/system-settings/__tests__/DataManagementSection.spec.ts -t \"backup history failure\" 2>&1 | tee ../.omo/evidence/task-4-data-management-history-tests-error.txt
    Expected: command exits 0 and the test asserts a graceful Chinese error message instead of an unhandled rejection
    Evidence: .omo/evidence/task-4-data-management-history-tests-error.txt
  ```

  Commit: YES | Message: `test(system-settings): cover backup history states` | Files: [frontend/src/views/admin/system-settings/__tests__/DataManagementSection.spec.ts]

- [ ] 5. Record complete-backup export runs on every data export

  What to do: In `apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs`, wrap the existing `state.build_admin_system_data_export_payload().await?` path so every `GET /api/admin/system/data/export` creates a run with task key `admin.system.data_export`. Use `UpsertBackgroundTaskRun` with kind `BackgroundTaskKind::OnDemand`, trigger `"manual"`, created_by from `decision.admin_principal.user_id` if available, started and finished timestamps, status `Succeeded` on success or `Failed` on payload builder error, progress `100` on success and `0` or `100` with failed message on failure. Store only safe metadata: payload may be `{ "route": "/api/admin/system/data/export" }`; result may include counts such as `config_sections`, `user_count`, `standalone_key_count` only if they are derived without storing sensitive content. If count extraction is risky, store `{ "exported": true }` only. Preserve the original response JSON and audit attachment. If `has_background_task_data_writer()` is false, the export must still work and skip recording.
  Must NOT do: Do not persist the export payload or secrets. Do not turn the synchronous export into an async job. Do not change the filename/download flow. Do not mask the original export error; after recording a failed run, return the same error path the handler would have returned before.

  Parallelization: Can parallel: YES | Wave 2 | Blocks: [8, 10] | Blocked by: [1, 2]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs:321` - existing data export handler that must preserve response/audit.
  - Pattern:  `apps/aether-gateway/src/task_runtime/mod.rs:343` - `upsert_run_with_logging` helper for non-fatal record writes.
  - Pattern:  `apps/aether-gateway/src/task_runtime/mod.rs:301` - timestamp helper.
  - Pattern:  `apps/aether-gateway/src/task_runtime/mod.rs:308` - run id helper.
  - Pattern:  `apps/aether-gateway/src/task_runtime/mod.rs:506` - provider delete task run fields.
  - API/Type: `crates/aether-data-contracts/src/repository/background_tasks/types.rs:104` - `UpsertBackgroundTaskRun` contract and validation.
  - API/Type: `apps/aether-gateway/src/state/runtime/background_tasks.rs:50` - AppState upsert wrapper.
  - Test:     `apps/aether-gateway/src/tests/control/admin/system.rs:666` - local admin system export test pattern.

  Acceptance criteria (agent-executable only):
  - [ ] RED first: before production changes, add/run `rtk cargo test -p aether-gateway gateway_handles_admin_system_data_export_records_backup_run -- --exact` and capture failure showing no run is recorded.
  - [ ] GREEN: after implementation, the same test exits 0 and asserts exactly one `admin.system.data_export` run with status `succeeded`, trigger `manual`, `created_by=admin-user-123`, and no exported payload content in `payload_json`/`result_json`.
  - [ ] A failure-path test exits 0 and asserts a failed run is recorded when the payload builder returns an error.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: complete backup export records a successful run
    Tool:     bash
    Steps:    rtk cargo test -p aether-gateway gateway_handles_admin_system_data_export_records_backup_run -- --exact 2>&1 | tee .omo/evidence/task-5-export-run-recording.txt
    Expected: command exits 0 and assertions prove one succeeded admin.system.data_export run without payload contents
    Evidence: .omo/evidence/task-5-export-run-recording.txt

  Scenario: complete backup export records a failed run without hiding the error
    Tool:     bash
    Steps:    rtk cargo test -p aether-gateway gateway_handles_admin_system_data_export_records_failed_backup_run -- --exact 2>&1 | tee .omo/evidence/task-5-export-run-recording-error.txt
    Expected: command exits 0 and assertions prove failed status plus original error response behavior
    Evidence: .omo/evidence/task-5-export-run-recording-error.txt
  ```

  Commit: YES | Message: `feat(admin-system): record complete backup export runs` | Files: [apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs, apps/aether-gateway/src/tests/control/admin/system.rs]

- [ ] 6. Add filtered backup-runs endpoint

  What to do: Add a focused helper module under `apps/aether-gateway/src/handlers/admin/system/core/backup_runs.rs` or a clearly named helper in `system_routes.rs` if the module split is rejected by local style. The helper should call `state.list_background_task_runs` with query fields that restrict to the new task key, then enforce exact `run.task_key == TASK_KEY_ADMIN_SYSTEM_DATA_EXPORT` before serialization so substring collisions cannot leak. Return JSON with `items`, `total`, `latest`, and optional `page/page_size` if pagination is included. Each item should include Chinese-UI-friendly fields but not secrets: `id`, `status`, `message`, `created_by`, RFC3339 timestamps, `duration_seconds`, `error_message`, and safe `result`. Wire the route in `system_routes.rs` for route kind `backup_runs` and request path `/api/admin/system/backup-runs`.
  Must NOT do: Do not return rows for any task key other than exact `admin.system.data_export`. Do not expose cancellation/trigger controls. Do not include `payload_json` unless it is sanitized metadata. Do not rely only on substring filtering for correctness.

  Parallelization: Can parallel: YES | Wave 2 | Blocks: [8, 10] | Blocked by: [1, 2]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs:399` - cleanup-runs route location and JSON return style.
  - Pattern:  `apps/aether-gateway/src/handlers/admin/features/background_tasks/routes.rs:71` - generic run serialization fields and RFC3339 mapping.
  - Pattern:  `apps/aether-gateway/src/handlers/admin/features/background_tasks/routes.rs:65` - pages calculation if pagination is added.
  - Pattern:  `crates/aether-data/src/repository/background_tasks/memory.rs:42` - existing substring filter risk requiring exact post-filter.
  - Pattern:  `crates/aether-data/src/repository/background_tasks/sqlite.rs:78` - SQL substring filter risk requiring exact post-filter.
  - API/Type: `crates/aether-data-contracts/src/repository/background_tasks/types.rs:217` - `BackgroundTaskListQuery`.
  - External: `https://docs.rs/axum/latest/axum/struct.Json.html` - `Json` serializes response values and sets JSON content type.

  Acceptance criteria (agent-executable only):
  - [ ] `rtk cargo test -p aether-gateway gateway_lists_admin_system_backup_runs -- --exact` exits 0 and asserts HTTP 200, `total`, `latest`, and item fields.
  - [ ] `rtk cargo test -p aether-gateway gateway_lists_only_complete_backup_runs -- --exact` exits 0 and proves a seeded `admin.system.data_export.extra` or unrelated task row is excluded.
  - [ ] `GET /api/admin/system/backup-runs` remains available when no background task reader exists and returns `items: []`, `total: 0`, `latest: null`.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: backup-runs lists only complete backup export runs
    Tool:     bash
    Steps:    rtk cargo test -p aether-gateway gateway_lists_only_complete_backup_runs -- --exact 2>&1 | tee .omo/evidence/task-6-backup-runs-endpoint.txt
    Expected: command exits 0 and response excludes non-exact task keys
    Evidence: .omo/evidence/task-6-backup-runs-endpoint.txt

  Scenario: backup-runs empty state has stable response shape
    Tool:     bash
    Steps:    rtk cargo test -p aether-gateway gateway_lists_empty_admin_system_backup_runs -- --exact 2>&1 | tee .omo/evidence/task-6-backup-runs-endpoint-error.txt
    Expected: command exits 0 and response is HTTP 200 with items=[] total=0 latest=null
    Evidence: .omo/evidence/task-6-backup-runs-endpoint-error.txt
  ```

  Commit: YES | Message: `feat(admin-system): expose complete backup run history` | Files: [apps/aether-gateway/src/handlers/admin/system/core/mod.rs, apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs, apps/aether-gateway/src/handlers/admin/system/core/backup_runs.rs, apps/aether-gateway/src/tests/control/admin/system.rs]

- [ ] 7. Show backup history in System Settings data management

  What to do: Update `frontend/src/views/admin/system-settings/DataManagementSection.vue` to load backup runs on mount, refresh after clicking the complete backup export button, and show a compact `最近备份记录` panel near the existing `导出 / 导入` block. Use Chinese copy. Show total count, latest status, latest time, a refresh button, empty state `暂无备份记录`, loading state, error state, and up to 5 recent rows. Keep layout compact and consistent with existing `CleanupPolicySection.vue` table style. Emit or handle a refresh after aggregate export; the simplest plan is for `SystemSettings.vue` to call a `refreshBackupRuns` exposed via a component ref after `handleExportAggregate()` completes, or to move aggregate export handling into `DataManagementSection` only if that is less invasive. Prefer a local method on the section if it can call `adminApi.getBackupRuns()` directly while export still flows through the existing parent event.
  Must NOT do: Do not add a new page, route, global Pinia store, marketing text, or non-Chinese UI. Do not add nested cards inside cards; keep this as an unframed subsection/table inside the existing `CardSection`.

  Parallelization: Can parallel: YES | Wave 2 | Blocks: [9, 10] | Blocked by: [3]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `frontend/src/views/admin/system-settings/DataManagementSection.vue:1` - existing data-management card and export/import layout.
  - Pattern:  `frontend/src/views/admin/system-settings/DataManagementSection.vue:189` - computed data items including `aggregate`.
  - Pattern:  `frontend/src/views/admin/system-settings/CleanupPolicySection.vue:310` - table panel style inside an existing settings section.
  - Pattern:  `frontend/src/views/admin/system-settings/CleanupPolicySection.vue:476` - loading runs from `adminApi`.
  - Pattern:  `frontend/src/views/admin/system-settings/CleanupPolicySection.vue:549` - `onMounted` and interval cleanup precedent; for this feature use mount and manual refresh, and add interval only if product requires live polling.
  - Pattern:  `frontend/src/views/admin/SystemSettings.vue:37` - parent passes export/import loading props into `DataManagementSection`.
  - Pattern:  `frontend/src/views/admin/system-settings/composables/useConfigExportImport.ts:372` - complete backup export flow and success/error toasts.
  - External: `https://vuejs.org/guide/extras/composition-api-faq.html` - `<script setup>`, `ref`, and `onMounted` are standard Composition API patterns.

  Acceptance criteria (agent-executable only):
  - [ ] `cd frontend && rtk npm run test:run -- src/views/admin/system-settings/__tests__/DataManagementSection.spec.ts` exits 0 and covers render/update states.
  - [ ] `cd frontend && rtk npm run type-check` exits 0.
  - [ ] Browser QA captures a screenshot showing `最近备份记录`, latest status, total count, and at least one recent row after an export.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: System Settings shows recent backup history after export
    Tool:     playwright(real Chrome)
    Steps:    Start backend/frontend with `rtk make dev` in a tmux session named `ulw-qa-backup-history`, then run `rtk proxy node .omo/qa/task-7-system-settings-backup-history.mjs`. The script must launch installed Chrome through Playwright, use the executor's existing admin-auth setup, navigate to `http://127.0.0.1:5173/admin/system`, click `text=导出备份`, wait for `text=最近备份记录` and `text=成功`, and save the screenshot path below.
    Expected: screenshot contains 最近备份记录, total count >= 1, latest status 成功, and a row timestamp
    Evidence: .omo/evidence/task-7-system-settings-backup-history.png

  Scenario: System Settings shows backup history error state gracefully
    Tool:     playwright(real Chrome)
    Steps:    Run `rtk proxy node .omo/qa/task-7-system-settings-backup-history-error.mjs`. The script must launch installed Chrome through Playwright, intercept `**/api/admin/system/backup-runs` with HTTP 500 and body `{"detail":"boom"}`, navigate to `http://127.0.0.1:5173/admin/system`, wait for `text=数据管理`, capture the screenshot path below, and save console errors to `.omo/evidence/task-7-system-settings-backup-history-error-console.txt`.
    Expected: page remains usable and shows a Chinese backup-history load failure message without console unhandled promise errors
    Evidence: .omo/evidence/task-7-system-settings-backup-history-error.png
  ```

  Commit: YES | Message: `feat(system-settings): show complete backup history` | Files: [frontend/src/views/admin/system-settings/DataManagementSection.vue, frontend/src/views/admin/SystemSettings.vue]

- [ ] 8. Add backend integration tests for recording and listing

  What to do: In `apps/aether-gateway/src/tests/control/admin/system.rs`, add focused integration tests around the real gateway router. Required tests: `gateway_handles_admin_system_data_export_records_backup_run`, `gateway_handles_admin_system_data_export_records_failed_backup_run`, `gateway_lists_admin_system_backup_runs`, `gateway_lists_only_complete_backup_runs`, and `gateway_lists_empty_admin_system_backup_runs`. Use `InMemoryBackgroundTaskRepository` wired through Task 2. The successful export test can follow existing config/users export tests and then inspect the repository through the endpoint or data state. The failed export test should create a controlled missing dependency or repository error that makes payload building fail without exposing secrets. If a clean failure fixture is too invasive, test the failed-recording helper directly and keep an HTTP-level success test.
  Must NOT do: Do not weaken or delete existing export tests. Do not use real external upstream services. Do not assert on wall-clock exact timestamps; assert presence/order and valid RFC3339 shape.

  Parallelization: Can parallel: YES | Wave 3 | Blocks: [10] | Blocked by: [1, 2, 5, 6]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `apps/aether-gateway/src/tests/control/admin/system.rs:666` - config export local gateway test setup.
  - Pattern:  `apps/aether-gateway/src/tests/control/admin/system.rs:855` - users export local gateway test with in-memory repositories.
  - Pattern:  `apps/aether-gateway/src/tests/control/admin/system.rs:1017` - trusted admin request headers.
  - Pattern:  `apps/aether-gateway/src/tests/control/admin/system.rs:1098` - existing assertion that export stays local and does not hit upstream.
  - API/Type: `crates/aether-data/src/repository/background_tasks/memory.rs:51` - seed runs for list endpoint tests.
  - API/Type: `crates/aether-data-contracts/src/repository/background_tasks/types.rs:81` - stored run fields for seeded rows.
  - External: `https://docs.rs/tower/latest/tower/util/trait.ServiceExt.html` - service/request tests can use Tower service helpers where the repo already does.

  Acceptance criteria (agent-executable only):
  - [ ] `rtk cargo test -p aether-gateway gateway_handles_admin_system_data_export_records_backup_run -- --exact` exits 0.
  - [ ] `rtk cargo test -p aether-gateway gateway_handles_admin_system_data_export_records_failed_backup_run -- --exact` exits 0, or a helper-level failed-recording test exits 0 with a documented reason why HTTP failure could not be isolated cleanly.
  - [ ] `rtk cargo test -p aether-gateway gateway_lists_admin_system_backup_runs -- --exact` exits 0.
  - [ ] `rtk cargo test -p aether-gateway gateway_lists_only_complete_backup_runs -- --exact` exits 0.
  - [ ] `rtk cargo test -p aether-gateway gateway_lists_empty_admin_system_backup_runs -- --exact` exits 0.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: backend export and listing tests pass together
    Tool:     bash
    Steps:    rtk cargo test -p aether-gateway backup_run -- --nocapture 2>&1 | tee .omo/evidence/task-8-backend-integration.txt
    Expected: command exits 0 and all named tests pass
    Evidence: .omo/evidence/task-8-backend-integration.txt

  Scenario: failed export run recording is covered
    Tool:     bash
    Steps:    rtk cargo test -p aether-gateway gateway_handles_admin_system_data_export_records_failed_backup_run -- --exact 2>&1 | tee .omo/evidence/task-8-backend-integration-error.txt
    Expected: command exits 0 and the test asserts failed status plus safe error message
    Evidence: .omo/evidence/task-8-backend-integration-error.txt
  ```

  Commit: YES | Message: `test(admin-system): verify complete backup run history` | Files: [apps/aether-gateway/src/tests/control/admin/system.rs]

- [ ] 9. Add frontend rendering tests for backup history

  What to do: Complete the tests introduced in Task 4 after the UI exists. Mock `adminApi.getBackupRuns()` for empty, success, and failure states. If `DataManagementSection.vue` exposes refresh after export, test that the refresh button and export completion path call `getBackupRuns()` again. Verify Chinese labels and recent-row rendering. Keep tests deterministic by using fixed RFC3339 strings.
  Must NOT do: Do not use brittle snapshots. Do not test CSS class names except stable status classes if absolutely necessary. Do not depend on local timezone for exact formatted strings unless the formatter is injected or mocked.

  Parallelization: Can parallel: YES | Wave 3 | Blocks: [10] | Blocked by: [3, 4, 7]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `frontend/src/views/admin/system-settings/__tests__/useConfigExportImport.spec.ts:1` - Vitest setup style.
  - Pattern:  `frontend/src/views/admin/system-settings/CleanupPolicySection.vue:448` - local ref array state for runs.
  - Pattern:  `frontend/src/views/admin/system-settings/CleanupPolicySection.vue:476` - load method and loading flag.
  - Pattern:  `frontend/src/views/admin/system-settings/DataManagementSection.vue:177` - existing emitted events for export/file select.
  - API/Type: `frontend/src/api/admin.ts:360` - similar cleanup run row type precedent.

  Acceptance criteria (agent-executable only):
  - [ ] `cd frontend && rtk npm run test:run -- src/views/admin/system-settings/__tests__/DataManagementSection.spec.ts` exits 0.
  - [ ] The tests assert empty, success, failed, and API-error states.
  - [ ] The tests assert no backup payload fields are rendered.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: frontend backup history component tests pass
    Tool:     bash
    Steps:    cd frontend && rtk npm run test:run -- src/views/admin/system-settings/__tests__/DataManagementSection.spec.ts 2>&1 | tee ../.omo/evidence/task-9-frontend-rendering.txt
    Expected: command exits 0 and tests cover empty/success/failed/API-error states
    Evidence: .omo/evidence/task-9-frontend-rendering.txt

  Scenario: frontend type-check catches contract drift
    Tool:     bash
    Steps:    cd frontend && rtk npm run type-check 2>&1 | tee ../.omo/evidence/task-9-frontend-rendering-error.txt
    Expected: command exits 0 with no Vue or TypeScript diagnostics
    Evidence: .omo/evidence/task-9-frontend-rendering-error.txt
  ```

  Commit: YES | Message: `test(system-settings): verify backup history rendering` | Files: [frontend/src/views/admin/system-settings/__tests__/DataManagementSection.spec.ts]

- [ ] 10. Run scoped verification and real-surface QA

  What to do: Run the scoped backend tests, frontend tests, typecheck, and build checks. Then perform real-surface QA through HTTP and Chrome. For HTTP, start a local gateway with a test/admin setup appropriate for this repo, call `GET /api/admin/system/data/export` with trusted admin headers, then call `GET /api/admin/system/backup-runs` and save headers/body evidence. For browser QA, start frontend and backend, navigate to System Settings in real Chrome, trigger complete backup export, and capture a screenshot plus console log. Clean up any dev servers, tmux sessions, downloads, and temporary evidence helpers created during QA.
  Must NOT do: Do not declare completion from tests alone. Do not leave dev servers running. Do not commit downloaded backup JSON or secrets. Do not include raw backup payload in evidence; redact or avoid capturing body fields that contain secrets.

  Parallelization: Can parallel: NO | Wave 3 | Blocks: [F1, F2, F3, F4] | Blocked by: [5, 6, 7, 8, 9]

  References (executor has NO interview context - be exhaustive):
  - Command:  `apps/aether-gateway/AGENTS.md` - use scoped `cargo clippy -p aether-gateway --all-targets -- -D warnings` and `cargo nextest run -p aether-gateway` when backend changes warrant it.
  - Command:  `frontend/AGENTS.md` - use `cd frontend && npm run build:with-typecheck` and `cd frontend && npm run test:run`.
  - Pattern:   `apps/aether-gateway/src/tests/control/admin/system.rs:1017` - trusted admin HTTP headers for local admin route tests.
  - Pattern:   `frontend/package.json:9` - frontend build/typecheck script.
  - Pattern:   `frontend/package.json:13` - frontend test run script.
  - External:  `https://github.com/vercel-labs/agent-browser` - fallback browser tool only if Chrome is not available.

  Acceptance criteria (agent-executable only):
  - [ ] `rtk cargo fmt --all --check` exits 0.
  - [ ] `rtk cargo clippy -p aether-gateway --all-targets -- -D warnings` exits 0.
  - [ ] `rtk cargo nextest run -p aether-gateway` exits 0, or any pre-existing failure is named with exact failing test/log evidence.
  - [ ] `cd frontend && rtk npm run test:run` exits 0.
  - [ ] `cd frontend && rtk npm run build:with-typecheck` exits 0.
  - [ ] HTTP evidence proves export then backup-runs listing works.
  - [ ] Chrome screenshot evidence proves the admin-visible UI renders and updates.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: HTTP export creates visible backup run
    Tool:     curl
    Steps:    With the local gateway running, execute `curl -i -H 'x-aether-gateway: rust-phase3b' -H 'x-aether-trusted-user-id: admin-user-123' -H 'x-aether-trusted-user-role: admin' -H 'x-aether-trusted-session-id: session-123' http://127.0.0.1:8084/api/admin/system/data/export -o /tmp/aether-backup-export.json -D .omo/evidence/task-10-export.headers`, then execute `curl -i -H 'x-aether-gateway: rust-phase3b' -H 'x-aether-trusted-user-id: admin-user-123' -H 'x-aether-trusted-user-role: admin' -H 'x-aether-trusted-session-id: session-123' http://127.0.0.1:8084/api/admin/system/backup-runs | tee .omo/evidence/task-10-backup-runs-http.txt`
    Expected: first response is HTTP 200 without evidence storing raw backup body; second response is HTTP 200 JSON with total>=1 and latest.status="succeeded"
    Evidence: .omo/evidence/task-10-backup-runs-http.txt

  Scenario: browser admin data-management panel shows recent backup run
    Tool:     playwright(real Chrome)
    Steps:    Run `rtk proxy node .omo/qa/task-10-system-settings-backup-history.mjs`. The script must launch installed Chrome through Playwright, use the executor's existing admin-auth setup, navigate to `http://127.0.0.1:5173/admin/system`, wait for `text=数据管理`, click `text=导出备份`, wait for `text=最近备份记录` and `text=成功`, capture the screenshot path below, and write console output to `.omo/evidence/task-10-system-settings-backup-history-console.txt`.
    Expected: screenshot shows 数据管理, 最近备份记录, 成功, total count, and at least one recent row; console log has no uncaught errors
    Evidence: .omo/evidence/task-10-system-settings-backup-history.png
  ```

  Commit: NO | Message: `n/a` | Files: []

## Final verification wave (MANDATORY - after all implementation tasks)
> Runs in PARALLEL. ALL must APPROVE. Surface results to the caller and wait for an explicit "okay" before declaring complete.
- [ ] F1. Plan compliance audit - every task done, every acceptance criterion met
- [ ] F2. Code quality review - diagnostics clean, idioms match, no dead code
- [ ] F3. Real manual QA - every QA scenario executed with evidence captured
- [ ] F4. Scope fidelity - nothing extra shipped beyond Must-Have, nothing Must-NOT-Have introduced

## Commit strategy
- One logical change per commit. Conventional Commits (`<type>(<scope>): <subject>` body + footer).
- Atomic: every commit builds and passes tests on its own.
- No "WIP" / "fix typo squash later" commits on the final branch - clean up before merge.
- Reference the plan file path in the final commit footer: `Plan: .omo/plans/admin-complete-backup-runs.md`.

## Success criteria
- All Must-Have shipped; all QA scenarios pass with captured evidence; F1-F4 approved; commit history clean.

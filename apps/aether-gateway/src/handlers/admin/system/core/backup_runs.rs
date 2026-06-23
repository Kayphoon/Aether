use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::handlers::admin::shared::{query_param_value, unix_secs_to_rfc3339};
use crate::task_runtime::{self, TASK_KEY_SYSTEM_DATA_EXPORT};
use crate::GatewayError;
use aether_data_contracts::repository::background_tasks::{
    BackgroundTaskKind, BackgroundTaskListQuery, BackgroundTaskStatus, StoredBackgroundTaskRun,
    UpsertBackgroundTaskRun,
};
use serde_json::json;

const DEFAULT_PAGE_SIZE: usize = 20;
const MAX_PAGE_SIZE: usize = 100;

pub(super) async fn build_admin_system_data_export_payload_with_run_record(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<serde_json::Value, GatewayError> {
    let run_id = task_runtime::build_task_run_id();
    let started_at = task_runtime::now_unix_secs();
    if state.has_background_task_data_writer() {
        let definition = task_runtime::task_definition(TASK_KEY_SYSTEM_DATA_EXPORT);
        let run = UpsertBackgroundTaskRun {
            id: run_id.clone(),
            task_key: TASK_KEY_SYSTEM_DATA_EXPORT.to_string(),
            kind: BackgroundTaskKind::OnDemand,
            trigger: "manual".to_string(),
            status: BackgroundTaskStatus::Running,
            attempt: 1,
            max_attempts: definition
                .map(|item| item.retry_policy.max_attempts)
                .unwrap_or(1),
            owner_instance: Some(state.app().tunnel.local_instance_id().to_string()),
            progress_percent: 0,
            progress_message: Some("完整备份导出中".to_string()),
            payload_json: Some(json!({ "scope": "complete_backup" })),
            result_json: None,
            error_message: None,
            cancel_requested: false,
            created_by: request_context
                .decision()
                .and_then(|decision| decision.admin_principal.as_ref())
                .map(|principal| principal.user_id.clone()),
            created_at_unix_secs: started_at,
            started_at_unix_secs: Some(started_at),
            finished_at_unix_secs: None,
            updated_at_unix_secs: started_at,
        };
        let _ = task_runtime::upsert_run_with_logging(state.app(), run).await;
        task_runtime::append_event_with_logging(
            state.app(),
            &run_id,
            "started",
            "complete backup export started",
            None,
        )
        .await;
    }

    let payload = match state.build_admin_system_data_export_payload().await {
        Ok(payload) => payload,
        Err(error) => {
            record_failed_export(state, &run_id, &error).await;
            return Err(error);
        }
    };

    record_succeeded_export(state, &run_id, &payload).await;
    Ok(payload)
}

pub(super) async fn build_admin_system_backup_runs_payload(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<serde_json::Value, GatewayError> {
    let query = request_context.query_string();
    let page = query_param_value(query, "page")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);
    let page_size = query_param_value(query, "page_size")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1).saturating_mul(page_size);
    let response = state
        .list_background_task_runs(&BackgroundTaskListQuery {
            task_key: Some(TASK_KEY_SYSTEM_DATA_EXPORT.to_string()),
            task_key_substring: None,
            kind: Some(BackgroundTaskKind::OnDemand),
            status: None,
            trigger: None,
            offset,
            limit: page_size,
        })
        .await?;
    let pages = if response.total == 0 {
        0
    } else {
        (response.total + page_size - 1) / page_size
    };
    let items = response
        .items
        .iter()
        .map(backup_run_payload)
        .collect::<Vec<_>>();

    Ok(json!({
        "items": items,
        "total": response.total,
        "page": page,
        "page_size": page_size,
        "pages": pages,
    }))
}

async fn record_failed_export(state: &AdminAppState<'_>, run_id: &str, error: &GatewayError) {
    if !state.has_background_task_data_writer() {
        return;
    }

    let finished_at = task_runtime::now_unix_secs();
    let error_message = gateway_error_message(error);
    let _ = task_runtime::update_run_status(
        state.app(),
        run_id,
        BackgroundTaskStatus::Failed,
        Some(100),
        Some("完整备份导出失败".to_string()),
        None,
        Some(error_message.clone()),
        None,
        Some(finished_at),
    )
    .await;
    task_runtime::append_event_with_logging(
        state.app(),
        run_id,
        "failed",
        "complete backup export failed",
        Some(json!({ "error": error_message })),
    )
    .await;
}

async fn record_succeeded_export(
    state: &AdminAppState<'_>,
    run_id: &str,
    payload: &serde_json::Value,
) {
    if !state.has_background_task_data_writer() {
        return;
    }

    let finished_at = task_runtime::now_unix_secs();
    let exported_bytes = serde_json::to_vec(payload)
        .map(|bytes| bytes.len())
        .unwrap_or(0);
    let _ = task_runtime::update_run_status(
        state.app(),
        run_id,
        BackgroundTaskStatus::Succeeded,
        Some(100),
        Some("完整备份导出完成".to_string()),
        Some(json!({ "exported_bytes": exported_bytes })),
        None,
        None,
        Some(finished_at),
    )
    .await;
    task_runtime::append_event_with_logging(
        state.app(),
        run_id,
        "succeeded",
        "complete backup export succeeded",
        Some(json!({ "exported_bytes": exported_bytes })),
    )
    .await;
}

fn backup_run_payload(run: &StoredBackgroundTaskRun) -> serde_json::Value {
    json!({
        "id": run.id,
        "task_key": run.task_key,
        "kind": run.kind.as_database(),
        "trigger": run.trigger,
        "status": run.status.as_database(),
        "attempt": run.attempt,
        "max_attempts": run.max_attempts,
        "owner_instance": run.owner_instance,
        "progress_percent": run.progress_percent,
        "progress_message": run.progress_message,
        "payload": json!({ "scope": "complete_backup" }),
        "result": safe_backup_run_result(run.result_json.as_ref()),
        "error_message": run.error_message,
        "cancel_requested": run.cancel_requested,
        "created_by": run.created_by,
        "created_at": unix_secs_to_rfc3339(run.created_at_unix_secs),
        "started_at": run.started_at_unix_secs.and_then(unix_secs_to_rfc3339),
        "finished_at": run.finished_at_unix_secs.and_then(unix_secs_to_rfc3339),
        "updated_at": unix_secs_to_rfc3339(run.updated_at_unix_secs),
    })
}

fn safe_backup_run_result(result: Option<&serde_json::Value>) -> serde_json::Value {
    match result
        .and_then(|value| value.get("exported_bytes"))
        .and_then(serde_json::Value::as_u64)
    {
        Some(exported_bytes) => json!({ "exported_bytes": exported_bytes }),
        None => serde_json::Value::Null,
    }
}

fn gateway_error_message(error: &GatewayError) -> String {
    match error {
        GatewayError::UpstreamUnavailable { message, .. }
        | GatewayError::ControlUnavailable { message, .. }
        | GatewayError::Client { message, .. }
        | GatewayError::Internal(message) => message.clone(),
        GatewayError::LocalExecutionPlanningTimeout {
            phase, timeout_ms, ..
        } => format!("local execution planning timed out in {phase} after {timeout_ms}ms"),
    }
}

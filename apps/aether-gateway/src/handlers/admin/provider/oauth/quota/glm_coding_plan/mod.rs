mod metadata;
mod probe;

use super::shared::{
    build_quota_snapshot_payload, persist_provider_quota_refresh_state,
    quota_refresh_success_invalid_state,
};
use crate::handlers::admin::request::AdminAppState;
use crate::GatewayError;
use aether_contracts::ProxySnapshot;
use aether_data_contracts::repository::provider_catalog::{
    StoredProviderCatalogEndpoint, StoredProviderCatalogKey, StoredProviderCatalogProvider,
};
use serde_json::json;

use metadata::{
    key_write_failed_result, missing_key_result, transport_unavailable_result, unix_now,
};

enum RefreshOneKeyResult {
    MissingKey,
    TransportUnavailable,
    Outcome(RefreshOutcome),
}

pub(super) struct RefreshOutcome {
    pub(super) metadata: serde_json::Map<String, serde_json::Value>,
    pub(super) parsed_probe_count: usize,
    pub(super) oauth_invalid_at_unix_secs: Option<u64>,
    pub(super) oauth_invalid_reason: Option<String>,
}

pub(crate) async fn refresh_glm_coding_plan_provider_quota_locally(
    state: &AdminAppState<'_>,
    provider: &StoredProviderCatalogProvider,
    endpoint: &StoredProviderCatalogEndpoint,
    keys: Vec<StoredProviderCatalogKey>,
    proxy_override: Option<ProxySnapshot>,
) -> Result<Option<serde_json::Value>, GatewayError> {
    let mut results = Vec::new();
    let mut success_count = 0usize;
    let mut failed_count = 0usize;
    let query = probe::usage_query();

    for key in keys {
        match refresh_one_key(
            state,
            provider,
            endpoint,
            &key,
            &query,
            proxy_override.as_ref(),
        )
        .await?
        {
            RefreshOneKeyResult::MissingKey => {
                failed_count += 1;
                results.push(missing_key_result(&key));
            }
            RefreshOneKeyResult::TransportUnavailable => {
                failed_count += 1;
                results.push(transport_unavailable_result(&key));
            }
            RefreshOneKeyResult::Outcome(outcome) => {
                if outcome.parsed_probe_count == 0 {
                    failed_count += 1;
                    results.push(
                        persist_empty_probe_result(state, &key, outcome)
                            .await?
                            .unwrap_or_else(|| key_write_failed_result(&key)),
                    );
                } else {
                    match persist_success_result(state, &key, outcome).await? {
                        Some(result) => {
                            success_count += 1;
                            results.push(result);
                        }
                        None => {
                            failed_count += 1;
                            results.push(key_write_failed_result(&key));
                        }
                    }
                }
            }
        }
    }

    Ok(Some(json!({
        "success": success_count,
        "failed": failed_count,
        "total": results.len(),
        "results": results,
        "message": format!("已处理 {} 个 Key", results.len()),
        "auto_removed": 0,
    })))
}

async fn refresh_one_key(
    state: &AdminAppState<'_>,
    provider: &StoredProviderCatalogProvider,
    endpoint: &StoredProviderCatalogEndpoint,
    key: &StoredProviderCatalogKey,
    query: &str,
    proxy_override: Option<&ProxySnapshot>,
) -> Result<RefreshOneKeyResult, GatewayError> {
    let Some(transport) = state
        .read_provider_transport_snapshot(&provider.id, &endpoint.id, &key.id)
        .await?
    else {
        return Ok(RefreshOneKeyResult::TransportUnavailable);
    };
    let api_key = transport.key.decrypted_api_key.trim();
    if api_key.is_empty() || api_key == "__placeholder__" {
        return Ok(RefreshOneKeyResult::MissingKey);
    }

    let now_unix_secs = unix_now();
    let (prior_invalid_at, prior_invalid_reason) = quota_refresh_success_invalid_state(key);
    let mut outcome = RefreshOutcome {
        metadata: serde_json::Map::from_iter([("updated_at".to_string(), json!(now_unix_secs))]),
        parsed_probe_count: 0,
        oauth_invalid_at_unix_secs: prior_invalid_at,
        oauth_invalid_reason: prior_invalid_reason,
    };

    for (kind, parser_kind) in [
        ("model_usage", Some("model")),
        ("tool_usage", Some("tool")),
        ("quota_limit", None),
    ] {
        let result = probe::execute_kind(
            state,
            &transport,
            endpoint,
            api_key,
            kind,
            (kind != "quota_limit").then_some(query),
            proxy_override,
        )
        .await?;
        probe::handle_probe_result(kind, parser_kind, result, now_unix_secs, &mut outcome);
    }

    Ok(RefreshOneKeyResult::Outcome(outcome))
}

async fn persist_empty_probe_result(
    state: &AdminAppState<'_>,
    key: &StoredProviderCatalogKey,
    outcome: RefreshOutcome,
) -> Result<Option<serde_json::Value>, GatewayError> {
    if outcome.oauth_invalid_at_unix_secs.is_some()
        && !persist_provider_quota_refresh_state(
            state,
            &key.id,
            None,
            outcome.oauth_invalid_at_unix_secs,
            outcome.oauth_invalid_reason.clone(),
            None,
        )
        .await?
    {
        return Ok(None);
    }
    let status = if outcome.oauth_invalid_at_unix_secs.is_some() {
        "auth_invalid"
    } else {
        "no_metadata"
    };
    Ok(Some(json!({
        "key_id": key.id,
        "key_name": key.name,
        "status": status,
        "message": outcome
            .oauth_invalid_reason
            .unwrap_or_else(|| "响应中未包含可解析的额度信息".to_string()),
        "metadata": outcome.metadata,
    })))
}

async fn persist_success_result(
    state: &AdminAppState<'_>,
    key: &StoredProviderCatalogKey,
    outcome: RefreshOutcome,
) -> Result<Option<serde_json::Value>, GatewayError> {
    let metadata_update = json!({ "glm_coding_plan": outcome.metadata });
    let quota_snapshot = build_quota_snapshot_payload(
        "glm_coding_plan",
        key.status_snapshot.as_ref(),
        Some(&metadata_update),
    );
    let Some(quota_snapshot) = quota_snapshot else {
        return Ok(Some(json!({
            "key_id": key.id,
            "key_name": key.name,
            "status": "no_metadata",
            "message": "响应中未包含可解析的额度信息",
            "metadata": metadata_update["glm_coding_plan"],
        })));
    };
    if !persist_provider_quota_refresh_state(
        state,
        &key.id,
        Some(&metadata_update),
        outcome.oauth_invalid_at_unix_secs,
        outcome.oauth_invalid_reason,
        None,
    )
    .await?
    {
        return Ok(None);
    }
    Ok(Some(json!({
        "key_id": key.id,
        "key_name": key.name,
        "status": "success",
        "metadata": metadata_update["glm_coding_plan"],
        "quota_snapshot": quota_snapshot,
    })))
}
